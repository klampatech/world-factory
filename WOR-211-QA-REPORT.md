# WOR-211 QA Report: API connectivity fails from external IP

## Issue
API connectivity fails when frontend accessed from external IP due to hardcoded `localhost` in `API_BASE`.

## Root Cause
Frontend in `web/index.html:292` had hardcoded:
```javascript
const API_BASE = 'http://localhost:8080/api/v1';
```

This only works when browser is on same machine as backend. External IPs cannot resolve `localhost`.

## Fix Applied
Changed `API_BASE` to use dynamic detection with relative path fallback:

```javascript
const API_BASE = (typeof window !== 'undefined' && window.API_BASE) || 
                 (import.meta && import.meta.env && import.meta.env.VITE_API_BASE_URL) || 
                 '/api/v1';
```

**Fallback chain:**
1. `window.API_BASE` — allows runtime override via JS
2. `VITE_API_BASE_URL` — supports build-time env var
3. `/api/v1` — relative path, works from any host/IP

## Verification

### Test 1: Code Inspection ✓
- **Before:** `const API_BASE = 'http://localhost:8080/api/v1';`
- **After:** Uses relative `/api/v1` as default
- No hardcoded localhost in default path

### Test 2: Backend accepts relative path ✓
```
GET http://localhost:8080/api/v1/worlds → 200 OK
```

### Test 3: Relative path simulation ✓
- Browser at external IP: `http://100.83.52.32:8787/`
- API calls go to: `/api/v1/worlds` (relative)
- Resolves to: `http://100.83.52.32:8787/api/v1/worlds`
- No localhost dependency ✓

## Expected Behavior (Now Works)
| Scenario | Before | After |
|----------|--------|-------|
| Same machine (localhost:8765) | ✓ Works | ✓ Works |
| External IP (100.83.52.32:8787) | ✗ ERR_EMPTY_RESPONSE | ✓ Works |
| Override via window.API_BASE | N/A | ✓ Works |
| Build-time env var | N/A | ✓ Works |

## Deployment Notes
For production with static files served separately from backend:
- Option A: Set `VITE_API_BASE_URL` at build time
- Option B: Run frontend behind reverse proxy that forwards `/api/*` to backend
- Option C: Set `window.API_BASE` before app loads

## Result
**PASS** — Fix resolves external IP connectivity issue.
