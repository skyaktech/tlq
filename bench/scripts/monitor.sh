#!/usr/bin/env bash
set -euo pipefail

RESULTS_DIR="$1"
PORT="${2:-11337}"
STATS_FILE="$RESULTS_DIR/raw/queue_stats.csv"
DOCKER_FILE="$RESULTS_DIR/raw/docker_stats.csv"
START_TIME=$(date +%s)

echo "timestamp,elapsed_s,ready,processing,dead" > "$STATS_FILE"
echo "timestamp,elapsed_s,cpu_pct,mem_usage_mb,mem_pct" > "$DOCKER_FILE"

CONTAINER_ID=$(docker compose -f "$(dirname "$0")/../docker-compose.yml" ps -q tlq 2>/dev/null || true)

while true; do
    NOW=$(date +%s)
    ELAPSED=$((NOW - START_TIME))

    # Queue stats
    STATS=$(curl -s "http://localhost:${PORT}/stats" 2>/dev/null || echo "{}")
    READY=$(echo "$STATS" | jq -r '.ready // "N/A"')
    PROCESSING=$(echo "$STATS" | jq -r '.processing // "N/A"')
    DEAD=$(echo "$STATS" | jq -r '.dead // "N/A"')
    echo "$NOW,$ELAPSED,$READY,$PROCESSING,$DEAD" >> "$STATS_FILE"

    # Docker stats
    if [ -n "$CONTAINER_ID" ]; then
        DSTATS=$(docker stats "$CONTAINER_ID" --no-stream --format '{{.CPUPerc}},{{.MemUsage}},{{.MemPerc}}' 2>/dev/null || echo "N/A,N/A,N/A")
    else
        CONTAINER_ID=$(docker compose -f "$(dirname "$0")/../docker-compose.yml" ps -q tlq 2>/dev/null || true)
        DSTATS="N/A,N/A,N/A"
    fi

    if [ "$DSTATS" != "N/A,N/A,N/A" ]; then
        CPU_PCT=$(echo "$DSTATS" | cut -d',' -f1 | tr -d '%')
        MEM_RAW=$(echo "$DSTATS" | cut -d',' -f2 | cut -d'/' -f1 | xargs)
        MEM_PCT=$(echo "$DSTATS" | cut -d',' -f3 | tr -d '%')

        # Convert memory to MB
        if echo "$MEM_RAW" | grep -qi 'gib'; then
            MEM_MB=$(echo "$MEM_RAW" | grep -oE '[0-9.]+' | head -1 | awk '{printf "%.1f", $1 * 1024}')
        elif echo "$MEM_RAW" | grep -qi 'mib'; then
            MEM_MB=$(echo "$MEM_RAW" | grep -oE '[0-9.]+' | head -1)
        elif echo "$MEM_RAW" | grep -qi 'kib'; then
            MEM_MB=$(echo "$MEM_RAW" | grep -oE '[0-9.]+' | head -1 | awk '{printf "%.1f", $1 / 1024}')
        else
            MEM_MB="N/A"
        fi

        echo "$NOW,$ELAPSED,$CPU_PCT,$MEM_MB,$MEM_PCT" >> "$DOCKER_FILE"
    else
        echo "$NOW,$ELAPSED,N/A,N/A,N/A" >> "$DOCKER_FILE"
    fi

    sleep 5
done
