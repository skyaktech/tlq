#!/usr/bin/env bash
set -euo pipefail

STAGE="$1"
DURATION="$2"
PRODUCER_RATE="$3"
NUM_CONSUMERS="$4"
RESULTS_DIR="$5"
PORT="${6:-11337}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Stage $STAGE: ${DURATION}s | producer: ${PRODUCER_RATE}/s | consumers: ${NUM_CONSUMERS} ==="

PIDS=()

# Launch rate-controlled curl producer
bash "$SCRIPT_DIR/producer.sh" "$RESULTS_DIR" "$STAGE" "$PRODUCER_RATE" "$DURATION" "$PORT" &
PIDS+=($!)

# Launch consumers
for i in $(seq 1 "$NUM_CONSUMERS"); do
    bash "$SCRIPT_DIR/consumer.sh" "$RESULTS_DIR" "$STAGE" "$i" "$DURATION" "$PORT" &
    PIDS+=($!)
done

# Wait for all
for pid in "${PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

echo "=== Stage $STAGE complete ==="
