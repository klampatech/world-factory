# WOR-921 Fix Report: Frontend API Proxy Configuration

## Problem
The frontend static file server (`npx serve`) does not proxy API requests to the backend. When the browser sends requests to `/api/v1/*`, the static server returns index.html instead of forwarding to the backend at `localhost:8080`, causing `SyntaxError: Unexpected token '<'` when parsing HTML as JSON.

## Root Cause
The `scripts/start-frontend.sh` script used `npx serve` which is a simple static file server without API proxy capabilities. The frontend preview script (`web/scripts/preview.js`) already has API proxy logic but was not being used.

## Solution
Updated `scripts/start-frontend.sh` to use the Node.js preview server (`web/scripts/preview.js`) instead of `npx serve`. The preview server already includes:
1. Static file serving from `web/dist/`
2. API proxy for `/api/*` and `/health` endpoints
3. Proper header forwarding (X-Forwarded-Proto, X-Forwarded-Host)
4. Error handling for backend unavailability

## Changes Made

### 1. `scripts/start-frontend.sh`
**Before:**
```bash
# Try vite first (for dev), fall back to a static server
if [ -f "vite.config.ts" ]; then
  npx vite preview --port "$PORT" --host 0.0.0.0 &
else
  npx --yes serve web -p "$PORT" -s &
fi
FRONTEND_PID=$!
```

**After:**
```bash
# Use the preview server with API proxy (web/scripts/preview.js)
BACKEND_URL="${BACKEND_URL:-http://localhost:8080}" node web/scripts/preview.js &
FRONTEND_PID=$!
```

### 2. `web/scripts/preview.js`
Added `FRONTEND_PORT` as an alternative env var for consistency:
```javascript
const PORT = process.env.PORT || process.env.FRONTEND_PORT || 8765;
```

## Verification
The preview server at `web/scripts/preview.js` proxies:
- `/api/*` → `{BACKEND_URL}/api/*`
- `/health` → `{BACKEND_URL}/health`

All requests are logged: `Proxying GET /api/v1/worlds -> http://localhost:8080/api/v1/worlds`

## Usage
```bash
# Start with default backend (localhost:8080)
./scripts/start-frontend.sh

# Start with custom backend URL
BACKEND_URL=http://localhost:3000 ./scripts/start-frontend.sh

# Start on custom port
./scripts/start-frontend.sh 8787
```

## Related Issues
- WOR-910: Vite proxy configuration (related frontend proxy issue)
- WOR-919: Smoke test that discovered this bug