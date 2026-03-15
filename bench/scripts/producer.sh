#!/usr/bin/env bash
set -euo pipefail

RESULTS_DIR="$1"
STAGE="$2"
TARGET_RATE="$3"
DURATION="$4"
PORT="${5:-11337}"

LOG_FILE="$RESULTS_DIR/raw/producer_stage${STAGE}.csv"
TOTAL_FILE="$RESULTS_DIR/raw/producer_stage${STAGE}_total.txt"
echo "timestamp,sent" > "$LOG_FILE"

BASE_URL="http://localhost:${PORT}"
START_TIME=$(date +%s)
TOTAL_SENT=0
COUNTER=0

# Send in batches with sleep to approximate target rate
# Batch size adapts: small batches for low rates, larger for high rates
if [ "$TARGET_RATE" -le 100 ]; then
    BATCH=5
elif [ "$TARGET_RATE" -le 1000 ]; then
    BATCH=10
else
    BATCH=20
fi

# Sleep between batches to hit target rate
# batches_per_sec = target_rate / batch_size
# sleep = 1 / batches_per_sec = batch_size / target_rate
SLEEP_TIME=$(echo "scale=4; $BATCH / $TARGET_RATE" | bc)

while true; do
    ELAPSED=$(( $(date +%s) - START_TIME ))
    if [ "$ELAPSED" -ge "$DURATION" ]; then break; fi

    BATCH_SENT=0
    for _ in $(seq 1 "$BATCH"); do
        COUNTER=$((COUNTER + 1))
        curl -s -X POST "$BASE_URL/add" \
            -H "Content-Type: application/json" \
            -d "{\"body\":\"bench-${STAGE}-${COUNTER}\"}" > /dev/null 2>&1 &
        BATCH_SENT=$((BATCH_SENT + 1))
    done
    wait

    TOTAL_SENT=$((TOTAL_SENT + BATCH_SENT))

    # Log every ~5 seconds
    NOW=$(date +%s)
    SINCE_LOG=${LAST_LOG:-0}
    if [ $((NOW - SINCE_LOG)) -ge 5 ]; then
        echo "$NOW,$TOTAL_SENT" >> "$LOG_FILE"
        LAST_LOG=$NOW
    fi

    sleep "$SLEEP_TIME" 2>/dev/null || true
done

echo "$TOTAL_SENT" > "$TOTAL_FILE"
echo "Producer stage $STAGE done: sent=$TOTAL_SENT (target=${TARGET_RATE}/s)" >&2
