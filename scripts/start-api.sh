#!/bin/bash
# start-api.sh — Start the World Factory API server
# Usage: ./scripts/start-api.sh [--port PORT]

set -e

PORT="${1:-3000}"
HOST="0.0.0.0"
MAX_WAIT=30
POLL_INTERVAL=2

echo "=== World Factory API Server ==="
echo "Building..."
cd "$(dirname "$0")/.."
cargo build --release --features api 2>&1 | tail -5

echo "Starting API server on $HOST:$PORT..."
cargo run --release --features api -- \
  --port "$PORT" \
  --host "$HOST" \
  --seed 42 \
  --width 64 \
  --height 64 \
  &
API_PID=$!

echo "API server PID: $API_PID"
echo "Waiting for server to be ready..."

for i in $(seq 1 $MAX_WAIT); do
  if curl -sf "http://localhost:$PORT/health" > /dev/null 2>&1; then
    echo "✅ API server ready at http://localhost:$PORT"
    echo "PID saved to .api.pid"
    echo "$API_PID" > .api.pid
    exit 0
  fi
  echo "  Attempt $i/$MAX_WAIT — not ready yet..."
  sleep $POLL_INTERVAL
done

echo "❌ API server did not respond within ${MAX_WAIT}s"
kill $API_PID 2>/dev/null || true
exit 1
