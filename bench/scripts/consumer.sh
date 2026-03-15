#!/usr/bin/env bash
set -euo pipefail

RESULTS_DIR="$1"
STAGE="$2"
CONSUMER_ID="$3"
DURATION="$4"
PORT="${5:-11337}"

LOG_FILE="$RESULTS_DIR/raw/consumer_stage${STAGE}_id${CONSUMER_ID}.csv"
echo "timestamp,action,count" > "$LOG_FILE"

BASE_URL="http://localhost:${PORT}"
START_TIME=$(date +%s)
FAIL_COUNT=0
MAX_FAILS=10
TOTAL_DELETED=0
TOTAL_ABANDONED=0

while true; do
    ELAPSED=$(( $(date +%s) - START_TIME ))
    if [ "$ELAPSED" -ge "$DURATION" ]; then break; fi

    # Get up to 20 messages per batch
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/get" \
        -H "Content-Type: application/json" \
        -d '{"count":20}' 2>/dev/null || echo -e "\n000")

    HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    if [ "$HTTP_CODE" != "200" ]; then
        FAIL_COUNT=$((FAIL_COUNT + 1))
        if [ "$FAIL_COUNT" -ge "$MAX_FAILS" ]; then
            echo "Consumer $CONSUMER_ID: $MAX_FAILS consecutive failures, exiting" >&2
            break
        fi
        sleep 1
        continue
    fi
    FAIL_COUNT=0

    MSG_COUNT=$(echo "$BODY" | jq 'length' 2>/dev/null || echo "0")
    if [ "$MSG_COUNT" -eq 0 ] || [ "$MSG_COUNT" = "null" ]; then
        sleep 0.1
        continue
    fi

    # Use jq to split: 80% delete (indices not divisible by 5), 20% abandon
    # This avoids the slow per-message bash loop
    DELETE_JSON=$(echo "$BODY" | jq -c '[to_entries[] | select(.key % 5 != 0) | .value.id]' 2>/dev/null)
    DEL_COUNT=$(echo "$DELETE_JSON" | jq 'length' 2>/dev/null || echo "0")
    ABANDON_COUNT=$((MSG_COUNT - DEL_COUNT))

    NOW=$(date +%s)

    if [ "$DEL_COUNT" -gt 0 ]; then
        curl -s -X POST "$BASE_URL/delete" \
            -H "Content-Type: application/json" \
            -d "{\"ids\":$DELETE_JSON}" > /dev/null 2>&1 || true
        echo "$NOW,delete,$DEL_COUNT" >> "$LOG_FILE"
        TOTAL_DELETED=$((TOTAL_DELETED + DEL_COUNT))
    fi

    if [ "$ABANDON_COUNT" -gt 0 ]; then
        echo "$NOW,abandon,$ABANDON_COUNT" >> "$LOG_FILE"
        TOTAL_ABANDONED=$((TOTAL_ABANDONED + ABANDON_COUNT))
    fi
done

echo "Consumer $CONSUMER_ID stage $STAGE done: deleted=$TOTAL_DELETED abandoned=$TOTAL_ABANDONED" >&2
