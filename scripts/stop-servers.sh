#!/bin/bash
# stop-servers.sh — Stop all World Factory servers
# Usage: ./scripts/stop-servers.sh

echo "=== Stopping World Factory servers ==="

# Stop frontend
if [ -f .frontend.pid ]; then
  FE_PID=$(cat .frontend.pid)
  if kill -0 "$FE_PID" 2>/dev/null; then
    echo "Stopping frontend (PID $FE_PID)..."
    kill "$FE_PID"
    rm .frontend.pid
  else
    echo "Frontend not running."
  fi
fi

# Stop API
if [ -f .api.pid ]; then
  API_PID=$(cat .api.pid)
  if kill -0 "$API_PID" 2>/dev/null; then
    echo "Stopping API (PID $API_PID)..."
    kill "$API_PID"
    rm .api.pid
  else
    echo "API server not running."
  fi
fi

echo "Done."
