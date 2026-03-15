#!/usr/bin/env bash
set -euo pipefail

BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$BENCH_DIR"

# --- Defaults ---
DURATION_MIN=30
SKIP_BUILD=false
PORT=11337
CPUS="2.0"
MEMORY="1g"

# --- Parse args ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        --duration) DURATION_MIN="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --port) PORT="$2"; shift 2 ;;
        --cpus) CPUS="$2"; shift 2 ;;
        --memory) MEMORY="$2"; shift 2 ;;
        *) echo "Usage: $0 [--duration MINUTES] [--skip-build] [--port PORT] [--cpus CPUS] [--memory MEM]"; exit 1 ;;
    esac
done

DURATION_SEC=$((DURATION_MIN * 60))
STAGE_DURATION=$((DURATION_SEC / 3))
DRAIN_DURATION=60
WRK_BURST=30

# --- Prerequisites ---
source scripts/prereqs.sh

# --- Results directory ---
RUN_DATE=$(date +%Y-%m-%d_%H%M%S)
RESULTS_DIR="$BENCH_DIR/results/$RUN_DATE"
mkdir -p "$RESULTS_DIR/raw"

cat > "$RESULTS_DIR/config.txt" <<EOF
date=$RUN_DATE
duration_min=$DURATION_MIN
duration_sec=$DURATION_SEC
stage_duration_s=$STAGE_DURATION
drain_duration_s=$DRAIN_DURATION
wrk_burst_s=$WRK_BURST
port=$PORT
lock_duration=15
max_retries=3
worker_interval=5
cpu_limit=$CPUS
memory_limit=$MEMORY
stage1=producer:500/s consumers:5
stage2=producer:2000/s consumers:10
stage3=producer:5000/s consumers:20
wrk1=1t/2c
wrk2=2t/10c
wrk3=4t/50c
EOF

echo ""
echo "============================================"
echo "  TLQ Stress Test"
echo "  Duration: ${DURATION_MIN} min (${STAGE_DURATION}s per stage + ${DRAIN_DURATION}s drain)"
echo "  Latency benchmarks: 3 × ${WRK_BURST}s wrk bursts"
echo "  Resources: ${CPUS} CPUs, ${MEMORY} RAM"
echo "  Port: $PORT"
echo "  Results: $RESULTS_DIR"
echo "============================================"
echo ""

# --- Cleanup handler ---
MONITOR_PID=""
STAGE_PID=""

cleanup() {
    echo ""
    echo "Shutting down..."
    [ -n "$MONITOR_PID" ] && kill "$MONITOR_PID" 2>/dev/null && wait "$MONITOR_PID" 2>/dev/null || true
    [ -n "$STAGE_PID" ] && kill "$STAGE_PID" 2>/dev/null && wait "$STAGE_PID" 2>/dev/null || true
    jobs -p 2>/dev/null | xargs -r kill 2>/dev/null || true
    curl -s "http://localhost:${PORT}/stats" > "$RESULTS_DIR/raw/final_stats.json" 2>/dev/null || true
    bash scripts/report.sh "$RESULTS_DIR" 2>/dev/null || true
    docker compose down 2>/dev/null || true
    echo "Partial results in: $RESULTS_DIR"
    exit 1
}
trap cleanup SIGINT SIGTERM

# --- Generate docker-compose override for custom settings ---
cat > "$BENCH_DIR/docker-compose.override.yml" <<OVERRIDE
services:
  tlq:
    ports:
      - "${PORT}:${PORT}"
    environment:
      TLQ_PORT: ${PORT}
    deploy:
      resources:
        limits:
          cpus: "${CPUS}"
          memory: ${MEMORY}
OVERRIDE

# --- Start TLQ ---
echo "Starting TLQ container..."
if [ "$SKIP_BUILD" = true ]; then
    docker compose up -d
else
    docker compose up -d --build
fi

echo -n "Waiting for TLQ to be ready"
for i in $(seq 1 60); do
    if curl -sf "http://localhost:${PORT}/hello" > /dev/null 2>&1; then
        echo " OK"
        break
    fi
    if [ "$i" -eq 60 ]; then
        echo " TIMEOUT"
        docker compose logs
        docker compose down
        exit 1
    fi
    echo -n "."
    sleep 1
done

# =============================================
# Phase 1: Latency benchmarks (wrk bursts)
# =============================================
echo ""
echo "--- Phase 1: Latency Benchmarks ---"

WRK_SCRIPT="$BENCH_DIR/wrk/add.lua"
WRK_CONFIGS=("1 2" "2 10" "4 50")
WRK_LABELS=("1t/2c" "2t/10c" "4t/50c")

for s in 0 1 2; do
    STAGE_NUM=$((s + 1))
    read -r THREADS CONNS <<< "${WRK_CONFIGS[$s]}"

    # Purge before each burst for clean measurement
    curl -s -X POST "http://localhost:${PORT}/purge" \
        -H "Content-Type: application/json" -d '{}' > /dev/null 2>&1

    echo "  wrk burst $STAGE_NUM (${WRK_LABELS[$s]}, ${WRK_BURST}s)..."
    wrk -t"$THREADS" -c"$CONNS" -d"${WRK_BURST}s" \
        -s "$WRK_SCRIPT" --latency \
        "http://localhost:${PORT}/add" \
        > "$RESULTS_DIR/raw/wrk_stage_${STAGE_NUM}.txt" 2>&1
done

# Purge after all bursts
curl -s -X POST "http://localhost:${PORT}/purge" \
    -H "Content-Type: application/json" -d '{}' > /dev/null 2>&1

# =============================================
# Phase 2: Sustained load (producer + consumers)
# =============================================
echo ""
echo "--- Phase 2: Sustained Load Test ---"

# Start monitor (only for sustained phase)
bash scripts/monitor.sh "$RESULTS_DIR" "$PORT" &
MONITOR_PID=$!

#        stage  duration  producer_rate  consumers
STAGES=(
    "1  $STAGE_DURATION  500   5"
    "2  $STAGE_DURATION  2000  10"
    "3  $STAGE_DURATION  5000  20"
)

for stage_cfg in "${STAGES[@]}"; do
    read -r STAGE DUR RATE CONSUMERS <<< "$stage_cfg"
    bash scripts/stage.sh "$STAGE" "$DUR" "$RATE" "$CONSUMERS" "$RESULTS_DIR" "$PORT" &
    STAGE_PID=$!
    wait "$STAGE_PID" || true
    STAGE_PID=""
done

# =============================================
# Phase 3: Drain (consumers + reaper only)
# =============================================
echo ""
echo "=== Drain phase: ${DRAIN_DURATION}s (consumers + reaper only) ==="

DRAIN_PIDS=()
for i in $(seq 1 20); do
    bash scripts/consumer.sh "$RESULTS_DIR" "drain" "$i" "$DRAIN_DURATION" "$PORT" &
    DRAIN_PIDS+=($!)
done

for pid in "${DRAIN_PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

echo "=== Drain phase complete ==="

# --- Stop monitor ---
kill "$MONITOR_PID" 2>/dev/null && wait "$MONITOR_PID" 2>/dev/null || true
MONITOR_PID=""

# --- Final stats ---
echo ""
echo "Capturing final state..."
curl -s "http://localhost:${PORT}/stats" | jq . > "$RESULTS_DIR/raw/final_stats.json" 2>/dev/null || true

# --- Functional verification ---
echo "Running functional verification..."
VERIF_FILE="$RESULTS_DIR/raw/verification.json"
CHECKS="[]"

# Health check
if curl -sf "http://localhost:${PORT}/hello" > /dev/null 2>&1; then
    CHECKS=$(echo "$CHECKS" | jq '. + [{"name":"Health Check","pass":true,"detail":"GET /hello returned 200"}]')
else
    CHECKS=$(echo "$CHECKS" | jq '. + [{"name":"Health Check","pass":false,"detail":"GET /hello failed"}]')
fi

# Stats consistency
FINAL_STATS=$(cat "$RESULTS_DIR/raw/final_stats.json" 2>/dev/null || echo "{}")
READY=$(echo "$FINAL_STATS" | jq '.ready // -1')
PROCESSING=$(echo "$FINAL_STATS" | jq '.processing // -1')
DEAD=$(echo "$FINAL_STATS" | jq '.dead // -1')

if [ "$READY" -ge 0 ] && [ "$PROCESSING" -ge 0 ] && [ "$DEAD" -ge 0 ]; then
    CHECKS=$(echo "$CHECKS" | jq --arg d "ready=$READY, processing=$PROCESSING, dead=$DEAD" \
        '. + [{"name":"Stats Consistency","pass":true,"detail":$d}]')
else
    CHECKS=$(echo "$CHECKS" | jq '. + [{"name":"Stats Consistency","pass":false,"detail":"Negative or missing values"}]')
fi

# Reaper activity
TOTAL_ABANDONED=0
for f in "$RESULTS_DIR/raw/consumer_stage"*".csv"; do
    [ -f "$f" ] || continue
    N=$(awk -F',' '$2=="abandon" {sum+=$3} END {print sum+0}' "$f")
    TOTAL_ABANDONED=$((TOTAL_ABANDONED + N))
done

if [ "$DEAD" -gt 0 ]; then
    CHECKS=$(echo "$CHECKS" | jq --arg d "dead=$DEAD (reaper removed messages after max retries)" \
        '. + [{"name":"Reaper Activity","pass":true,"detail":$d}]')
elif [ "$TOTAL_ABANDONED" -gt 0 ] && [ "$PROCESSING" -lt "$TOTAL_ABANDONED" ]; then
    CHECKS=$(echo "$CHECKS" | jq --arg d "processing=$PROCESSING < abandoned=$TOTAL_ABANDONED (reaper reclaiming, dead=$DEAD)" \
        '. + [{"name":"Reaper Activity","pass":true,"detail":$d}]')
else
    CHECKS=$(echo "$CHECKS" | jq --arg d "dead=$DEAD, processing=$PROCESSING, abandoned=$TOTAL_ABANDONED" \
        '. + [{"name":"Reaper Activity","pass":false,"detail":$d}]')
fi

# Message accounting (only producer.sh totals — wrk burst messages are purged)
TOTAL_ADDED=0
for f in "$RESULTS_DIR/raw/producer_stage"*"_total.txt"; do
    [ -f "$f" ] || continue
    N=$(cat "$f")
    [ -n "$N" ] && TOTAL_ADDED=$((TOTAL_ADDED + N))
done

TOTAL_DELETED=0
for f in "$RESULTS_DIR/raw/consumer_stage"*".csv"; do
    [ -f "$f" ] || continue
    N=$(awk -F',' '$2=="delete" {sum+=$3} END {print sum+0}' "$f")
    TOTAL_DELETED=$((TOTAL_DELETED + N))
done

REMAINING=$((READY + PROCESSING))
ACCOUNTED=$((TOTAL_DELETED + DEAD + REMAINING))
DELTA=$((TOTAL_ADDED - ACCOUNTED))
if [ "${DELTA#-}" -lt "$((TOTAL_ADDED / 100 + 100))" ]; then
    ACCT_PASS=true
else
    ACCT_PASS=false
fi

CHECKS=$(echo "$CHECKS" | jq --arg d "added=$TOTAL_ADDED, deleted=$TOTAL_DELETED, dead=$DEAD, remaining=$REMAINING, delta=$DELTA" \
    --argjson p "$ACCT_PASS" \
    '. + [{"name":"Message Accounting","pass":$p,"detail":$d}]')

echo "$CHECKS" | jq . > "$VERIF_FILE"

# --- Generate report ---
echo ""
bash scripts/report.sh "$RESULTS_DIR"

# --- Cleanup ---
echo ""
echo "Stopping TLQ container..."
docker compose down

rm -f "$BENCH_DIR/docker-compose.override.yml"

echo ""
echo "============================================"
echo "  Benchmark complete!"
echo "  Results: $RESULTS_DIR"
echo "  Report:  $RESULTS_DIR/report.md"
echo "  HTML:    $RESULTS_DIR/report.html"
echo "============================================"
