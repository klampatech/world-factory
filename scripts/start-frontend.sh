#!/bin/bash
# start-frontend.sh — Start the World Factory frontend preview server
# Usage: ./scripts/start-frontend.sh [--port PORT]
#
# Prerequisites: API server must be running first (./scripts/start-api.sh)
# The frontend proxies /api/* requests to the API server.

set -e

PORT="${1:-8765}"
MAX_WAIT=30
POLL_INTERVAL=2

echo "=== World Factory Frontend Server ==="

if [ ! -f "package.json" ]; then
  echo "❌ No package.json found. Run from project root."
  exit 1
fi

# Check for vite or http-server
if ! command -v npx &>/dev/null; then
  echo "❌ npx not found. Install Node.js."
  exit 1
fi

if [ ! -d "web" ]; then
  echo "❌ No web/ directory found."
  exit 1
fi

if [ -f ".api.pid" ]; then
  API_PID=$(cat .api.pid)
  if kill -0 "$API_PID" 2>/dev/null; then
    echo "✅ API server is running (PID $API_PID)"
  else
    echo "⚠️  API server not running. Start it first: ./scripts/start-api.sh"
  fi
else
  echo "⚠️  No .api.pid found. Ensure API server is running first."
fi

echo "Starting frontend on http://localhost:$PORT ..."

# Use the preview server with API proxy (web/scripts/preview.js)
BACKEND_URL="${BACKEND_URL:-http://localhost:8080}" node web/scripts/preview.js &
FRONTEND_PID=$!

echo "Frontend PID: $FRONTEND_PID"

for i in $(seq 1 $MAX_WAIT); do
  if curl -sf "http://localhost:$PORT" > /dev/null 2>&1; then
    echo "✅ Frontend ready at http://localhost:$PORT"
    echo "$FRONTEND_PID" > .frontend.pid
    exit 0
  fi
  echo "  Attempt $i/$MAX_WAIT — not ready yet..."
  sleep $POLL_INTERVAL
done

echo "❌ Frontend did not respond within ${MAX_WAIT}s"
kill $FRONTEND_PID 2>/dev/null || true
exit 1
