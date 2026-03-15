#!/usr/bin/env bash
set -euo pipefail

RESULTS_DIR="$1"
REPORT="$RESULTS_DIR/report.md"
CONFIG_FILE="$RESULTS_DIR/config.txt"

# --- Helper functions ---

fmt_num() {
    echo "$1" | sed -E 's/([0-9])([0-9]{3})$/\1,\2/; s/([0-9])([0-9]{3}),/\1,\2,/; s/([0-9])([0-9]{3}),/\1,\2,/'
}

parse_wrk_field() {
    local file="$1" field="$2"
    grep "^${field}:" "$file" 2>/dev/null | head -1 | awk -F': ' '{print $2}' | tr -d ' '
}

parse_wrk_latency_pct() {
    local file="$1" pct="$2"
    grep -A5 "Latency Distribution" "$file" 2>/dev/null | grep "^ *${pct}%" | awk '{print $2}' || echo "N/A"
}

csv_stats_for_range() {
    local file="$1" col="$2" start="$3" end="$4"
    awk -F',' -v col="$col" -v s="$start" -v e="$end" '
    NR>1 && $2>=s && $2<e && $col != "N/A" && $col != "" {
        v=$col+0; sum+=v; count++
        if(count==1 || v<min) min=v
        if(count==1 || v>max) max=v
    }
    END {
        if(count>0) printf "%d,%d,%d", min, sum/count, max
        else printf "N/A,N/A,N/A"
    }' "$file"
}

sum_consumer_stats() {
    local dir="$1" stage="$2" action="$3"
    local total=0
    for f in "$dir"/raw/consumer_stage${stage}_id*.csv; do
        [ -f "$f" ] || continue
        local n
        n=$(awk -F',' -v act="$action" 'NR>1 && $2==act {sum+=$3} END {print sum+0}' "$f")
        total=$((total + n))
    done
    echo "$total"
}

# --- Read config ---

DURATION=$(grep "^duration_min=" "$CONFIG_FILE" | cut -d= -f2)
STAGE_DURATION=$(grep "^stage_duration_s=" "$CONFIG_FILE" | cut -d= -f2)
DATE=$(grep "^date=" "$CONFIG_FILE" | cut -d= -f2-)

S1_START=0
S1_END=$STAGE_DURATION
S2_START=$STAGE_DURATION
S2_END=$((STAGE_DURATION * 2))
S3_START=$((STAGE_DURATION * 2))
S3_END=$((STAGE_DURATION * 3))

# --- Totals ---

TOTAL_PRODUCED=0
for f in "$RESULTS_DIR/raw/producer_stage"*"_total.txt"; do
    [ -f "$f" ] || continue
    N=$(cat "$f"); [ -n "$N" ] && TOTAL_PRODUCED=$((TOTAL_PRODUCED + N))
done

TOTAL_CONSUMED=0
for f in "$RESULTS_DIR/raw/consumer_stage"*".csv"; do
    [ -f "$f" ] || continue
    N=$(awk -F',' '$2=="delete" {sum+=$3} END {print sum+0}' "$f")
    TOTAL_CONSUMED=$((TOTAL_CONSUMED + N))
done

PEAK_RPS=0
TOTAL_WRK_ERRORS=0
for s in 1 2 3; do
    WRK_FILE="$RESULTS_DIR/raw/wrk_stage_${s}.txt"
    [ -f "$WRK_FILE" ] || continue
    RPS=$(parse_wrk_field "$WRK_FILE" "Requests/sec")
    if [ -n "$RPS" ] && [ "$RPS" != "N/A" ]; then
        GT=$(echo "$RPS > $PEAK_RPS" | bc -l 2>/dev/null || echo "0")
        [ "$GT" = "1" ] && PEAK_RPS="$RPS"
    fi
    for etype in connect read write status timeout; do
        E=$(parse_wrk_field "$WRK_FILE" "Errors $etype")
        [ -n "$E" ] && [ "$E" != "N/A" ] && TOTAL_WRK_ERRORS=$((TOTAL_WRK_ERRORS + E))
    done
done

DOCKER_FILE="$RESULTS_DIR/raw/docker_stats.csv"
PEAK_MEM="0"; PEAK_CPU="0"
if [ -f "$DOCKER_FILE" ]; then
    PEAK_MEM=$(awk -F',' 'NR>1 && $4 != "N/A" && $4 != "" {v=$4+0; if(v>max) max=v} END {printf "%.1f", max+0}' "$DOCKER_FILE")
    PEAK_CPU=$(awk -F',' 'NR>1 && $3 != "N/A" && $3 != "" {v=$3+0; if(v>max) max=v} END {printf "%.1f", max+0}' "$DOCKER_FILE")
fi

FINAL_FILE="$RESULTS_DIR/raw/final_stats.json"
FINAL_DEAD=0; FINAL_READY=0; FINAL_PROC=0
if [ -f "$FINAL_FILE" ]; then
    FINAL_DEAD=$(jq '.dead // 0' "$FINAL_FILE")
    FINAL_READY=$(jq '.ready // 0' "$FINAL_FILE")
    FINAL_PROC=$(jq '.processing // 0' "$FINAL_FILE")
fi

VERIF_FILE="$RESULTS_DIR/raw/verification.json"
VERIF_PASS="N/A"
if [ -f "$VERIF_FILE" ]; then
    FAILS=$(jq '[.[] | select(.pass == false)] | length' "$VERIF_FILE" 2>/dev/null || echo "0")
    if [ "$FAILS" -eq 0 ]; then VERIF_PASS="ALL PASSED"; else VERIF_PASS="$FAILS FAILED"; fi
fi

# --- Generate report ---

cat > "$REPORT" <<HEADER
# TLQ Stress Test Report

**Date:** $DATE
**Duration:** $DURATION minutes ($((DURATION * 60))s sustained + 60s drain)
**Docker Limits:** $(grep "^cpu_limit=" "$CONFIG_FILE" | cut -d= -f2) CPUs, $(grep "^memory_limit=" "$CONFIG_FILE" | cut -d= -f2) RAM
**Reaper Config:** lock=15s, max_retries=3, worker_interval=5s

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Peak Throughput (wrk burst) | $(fmt_num "${PEAK_RPS%%.*}") req/s |
| Total Produced (sustained) | $(fmt_num "$TOTAL_PRODUCED") |
| Total Consumed | $(fmt_num "$TOTAL_CONSUMED") |
| Dead (reaper killed) | $(fmt_num "$FINAL_DEAD") |
| Final Queue | ready=$FINAL_READY processing=$FINAL_PROC |
| Peak CPU | ${PEAK_CPU}% |
| Peak Memory | ${PEAK_MEM} MB |
| wrk Errors | $TOTAL_WRK_ERRORS |
| Functional Checks | $VERIF_PASS |

---

HEADER

# --- Test Configuration ---

cat >> "$REPORT" <<'CONFIG_HDR'
## Test Configuration

| Parameter | Value |
|-----------|-------|
CONFIG_HDR

while IFS='=' read -r key val; do
    [ -n "$key" ] && echo "| $key | $val |" >> "$REPORT"
done < "$CONFIG_FILE"
echo "" >> "$REPORT"
echo "---" >> "$REPORT"
echo "" >> "$REPORT"

# --- Latency Benchmarks (wrk bursts) ---

echo "## Latency Benchmarks (wrk bursts)" >> "$REPORT"
echo "" >> "$REPORT"

WRK_LABELS=("1t/2c" "2t/10c" "4t/50c")
WRK_NAMES=("Low Concurrency" "Medium Concurrency" "High Concurrency")

echo "| Metric | ${WRK_NAMES[0]} (${WRK_LABELS[0]}) | ${WRK_NAMES[1]} (${WRK_LABELS[1]}) | ${WRK_NAMES[2]} (${WRK_LABELS[2]}) |" >> "$REPORT"
echo "|--------|------------|------------|------------|" >> "$REPORT"

# Collect data for all 3 bursts
declare -a WRK_RPS WRK_TREQ WRK_AVG WRK_P50 WRK_P75 WRK_P90 WRK_P99 WRK_MAX WRK_ERR
for s in 0 1 2; do
    WRK_FILE="$RESULTS_DIR/raw/wrk_stage_$((s+1)).txt"
    if [ -f "$WRK_FILE" ]; then
        WRK_RPS[$s]=$(parse_wrk_field "$WRK_FILE" "Requests/sec")
        WRK_TREQ[$s]=$(parse_wrk_field "$WRK_FILE" "Total requests")
        WRK_AVG[$s]=$(parse_wrk_field "$WRK_FILE" "Avg latency (ms)")
        WRK_P50[$s]=$(parse_wrk_latency_pct "$WRK_FILE" "50")
        WRK_P75[$s]=$(parse_wrk_latency_pct "$WRK_FILE" "75")
        WRK_P90[$s]=$(parse_wrk_latency_pct "$WRK_FILE" "90")
        WRK_P99[$s]=$(parse_wrk_latency_pct "$WRK_FILE" "99")
        WRK_MAX[$s]=$(parse_wrk_field "$WRK_FILE" "Max latency (ms)")
        ERRS=0
        for etype in connect read write status timeout; do
            E=$(parse_wrk_field "$WRK_FILE" "Errors $etype")
            [ -n "$E" ] && ERRS=$((ERRS + E))
        done
        WRK_ERR[$s]=$ERRS
    else
        WRK_RPS[$s]="N/A"; WRK_TREQ[$s]="N/A"; WRK_AVG[$s]="N/A"
        WRK_P50[$s]="N/A"; WRK_P75[$s]="N/A"; WRK_P90[$s]="N/A"; WRK_P99[$s]="N/A"
        WRK_MAX[$s]="N/A"; WRK_ERR[$s]="N/A"
    fi
done

echo "| Requests/sec | $(fmt_num "${WRK_RPS[0]}") | $(fmt_num "${WRK_RPS[1]}") | $(fmt_num "${WRK_RPS[2]}") |" >> "$REPORT"
echo "| Total Requests | $(fmt_num "${WRK_TREQ[0]}") | $(fmt_num "${WRK_TREQ[1]}") | $(fmt_num "${WRK_TREQ[2]}") |" >> "$REPORT"
echo "| Avg Latency | ${WRK_AVG[0]}ms | ${WRK_AVG[1]}ms | ${WRK_AVG[2]}ms |" >> "$REPORT"
echo "| p50 | ${WRK_P50[0]} | ${WRK_P50[1]} | ${WRK_P50[2]} |" >> "$REPORT"
echo "| p75 | ${WRK_P75[0]} | ${WRK_P75[1]} | ${WRK_P75[2]} |" >> "$REPORT"
echo "| p90 | ${WRK_P90[0]} | ${WRK_P90[1]} | ${WRK_P90[2]} |" >> "$REPORT"
echo "| p99 | ${WRK_P99[0]} | ${WRK_P99[1]} | ${WRK_P99[2]} |" >> "$REPORT"
echo "| Max Latency | ${WRK_MAX[0]}ms | ${WRK_MAX[1]}ms | ${WRK_MAX[2]}ms |" >> "$REPORT"
echo "| Errors | ${WRK_ERR[0]} | ${WRK_ERR[1]} | ${WRK_ERR[2]} |" >> "$REPORT"

echo "" >> "$REPORT"
echo "---" >> "$REPORT"
echo "" >> "$REPORT"

# --- Sustained Load Stages ---

echo "## Sustained Load Stages" >> "$REPORT"
echo "" >> "$REPORT"

STAGE_CONFIGS=("500/s, 5 consumers" "2,000/s, 10 consumers" "5,000/s, 20 consumers")
STAGE_NAMES=("Low Load" "Medium Load" "High Load")
STAGE_STARTS=($S1_START $S2_START $S3_START)
STAGE_ENDS=($S1_END $S2_END $S3_END)

for s in 0 1 2; do
    STAGE_NUM=$((s + 1))
    PROD_TOTAL_FILE="$RESULTS_DIR/raw/producer_stage${STAGE_NUM}_total.txt"

    echo "### Stage $STAGE_NUM: ${STAGE_NAMES[$s]} (${STAGE_CONFIGS[$s]})" >> "$REPORT"
    echo "" >> "$REPORT"

    PROD_SENT="N/A"
    [ -f "$PROD_TOTAL_FILE" ] && PROD_SENT=$(cat "$PROD_TOTAL_FILE")

    DELETED=$(sum_consumer_stats "$RESULTS_DIR" "$STAGE_NUM" "delete")
    ABANDONED=$(sum_consumer_stats "$RESULTS_DIR" "$STAGE_NUM" "abandon")
    TOTAL_STAGE=$((DELETED + ABANDONED))
    if [ "$TOTAL_STAGE" -gt 0 ]; then
        ABANDON_RATE=$(echo "scale=1; $ABANDONED * 100 / $TOTAL_STAGE" | bc)
    else
        ABANDON_RATE="0"
    fi

    cat >> "$REPORT" <<STAGE_TABLE
| Metric | Value |
|--------|-------|
| Produced | $(fmt_num "$PROD_SENT") |
| Deleted | $(fmt_num "$DELETED") |
| Abandoned | $(fmt_num "$ABANDONED") |
| Abandon Rate | ${ABANDON_RATE}% |

STAGE_TABLE

    STATS_FILE="$RESULTS_DIR/raw/queue_stats.csv"
    if [ -f "$STATS_FILE" ]; then
        READY_STATS=$(csv_stats_for_range "$STATS_FILE" 3 "${STAGE_STARTS[$s]}" "${STAGE_ENDS[$s]}")
        PROC_STATS=$(csv_stats_for_range "$STATS_FILE" 4 "${STAGE_STARTS[$s]}" "${STAGE_ENDS[$s]}")
        DEAD_STATS=$(csv_stats_for_range "$STATS_FILE" 5 "${STAGE_STARTS[$s]}" "${STAGE_ENDS[$s]}")

        R_MIN=$(fmt_num "$(echo "$READY_STATS" | cut -d',' -f1)")
        R_AVG=$(fmt_num "$(echo "$READY_STATS" | cut -d',' -f2)")
        R_MAX=$(fmt_num "$(echo "$READY_STATS" | cut -d',' -f3)")
        P_MIN=$(fmt_num "$(echo "$PROC_STATS" | cut -d',' -f1)")
        P_AVG=$(fmt_num "$(echo "$PROC_STATS" | cut -d',' -f2)")
        P_MAX=$(fmt_num "$(echo "$PROC_STATS" | cut -d',' -f3)")
        D_MIN=$(fmt_num "$(echo "$DEAD_STATS" | cut -d',' -f1)")
        D_AVG=$(fmt_num "$(echo "$DEAD_STATS" | cut -d',' -f2)")
        D_MAX=$(fmt_num "$(echo "$DEAD_STATS" | cut -d',' -f3)")

        cat >> "$REPORT" <<QDEPTH
**Queue Depth:**

| Metric | Ready | Processing | Dead |
|--------|-------|------------|------|
| Min | $R_MIN | $P_MIN | $D_MIN |
| Avg | $R_AVG | $P_AVG | $D_AVG |
| Max | $R_MAX | $P_MAX | $D_MAX |

QDEPTH
    fi

    echo "---" >> "$REPORT"
    echo "" >> "$REPORT"
done

# --- Drain Phase ---

DRAIN_DELETED=$(sum_consumer_stats "$RESULTS_DIR" "drain" "delete")
DRAIN_ABANDONED=$(sum_consumer_stats "$RESULTS_DIR" "drain" "abandon")

if [ "$((DRAIN_DELETED + DRAIN_ABANDONED))" -gt 0 ]; then
    cat >> "$REPORT" <<DRAIN
### Drain Phase (60s, consumers + reaper only)

| Metric | Value |
|--------|-------|
| Deleted | $(fmt_num "$DRAIN_DELETED") |
| Abandoned | $(fmt_num "$DRAIN_ABANDONED") |

---

DRAIN
fi

# --- Resource Utilization ---

echo "## Resource Utilization" >> "$REPORT"
echo "" >> "$REPORT"

if [ -f "$DOCKER_FILE" ]; then
    echo "| Stage | Avg CPU% | Peak CPU% | Avg Mem (MB) | Peak Mem (MB) |" >> "$REPORT"
    echo "|-------|----------|-----------|-------------|---------------|" >> "$REPORT"
    for s in 0 1 2; do
        STAGE_NUM=$((s + 1))
        CPU_STATS=$(csv_stats_for_range "$DOCKER_FILE" 3 "${STAGE_STARTS[$s]}" "${STAGE_ENDS[$s]}")
        MEM_STATS=$(csv_stats_for_range "$DOCKER_FILE" 4 "${STAGE_STARTS[$s]}" "${STAGE_ENDS[$s]}")
        CPU_AVG=$(echo "$CPU_STATS" | cut -d',' -f2)
        CPU_MAX=$(echo "$CPU_STATS" | cut -d',' -f3)
        MEM_AVG=$(echo "$MEM_STATS" | cut -d',' -f2)
        MEM_MAX=$(echo "$MEM_STATS" | cut -d',' -f3)
        echo "| $STAGE_NUM — ${STAGE_NAMES[$s]} | $CPU_AVG | $CPU_MAX | $MEM_AVG | $MEM_MAX |" >> "$REPORT"
    done
    echo "" >> "$REPORT"
fi

echo "---" >> "$REPORT"
echo "" >> "$REPORT"

# --- Queue Stats Over Time ---

echo "## Queue Stats Over Time" >> "$REPORT"
echo "" >> "$REPORT"

STATS_FILE="$RESULTS_DIR/raw/queue_stats.csv"
if [ -f "$STATS_FILE" ]; then
    echo "| Elapsed (s) | Ready | Processing | Dead |" >> "$REPORT"
    echo "|-------------|-------|------------|------|" >> "$REPORT"
    awk -F',' 'NR>1 && $2 % 30 < 5 {
        if ($2 != last) { printf "| %s | %\047d | %\047d | %\047d |\n", $2, $3, $4, $5; last=$2 }
    }' "$STATS_FILE" >> "$REPORT"
    echo "" >> "$REPORT"
fi

echo "---" >> "$REPORT"
echo "" >> "$REPORT"

# --- Functional Verification ---

echo "## Functional Verification" >> "$REPORT"
echo "" >> "$REPORT"

if [ -f "$VERIF_FILE" ]; then
    echo "| Check | Result | Details |" >> "$REPORT"
    echo "|-------|--------|---------|" >> "$REPORT"
    jq -r '.[] | "| \(.name) | \(if .pass then "PASS" else "FAIL" end) | \(.detail) |"' "$VERIF_FILE" >> "$REPORT"
    echo "" >> "$REPORT"
fi

# --- Final Stats ---

if [ -f "$FINAL_FILE" ]; then
    echo "## Final Queue State" >> "$REPORT"
    echo "" >> "$REPORT"
    echo '```json' >> "$REPORT"
    cat "$FINAL_FILE" >> "$REPORT"
    echo "" >> "$REPORT"
    echo '```' >> "$REPORT"
    echo "" >> "$REPORT"
fi

# --- Convert to HTML ---

echo "Generating HTML report..."
pandoc "$REPORT" -o "$RESULTS_DIR/report.html" --standalone \
    --metadata title="TLQ Stress Test Report" 2>/dev/null && \
    echo "HTML report: $RESULTS_DIR/report.html" || \
    echo "Warning: pandoc HTML conversion failed"

echo "Markdown report: $REPORT"
