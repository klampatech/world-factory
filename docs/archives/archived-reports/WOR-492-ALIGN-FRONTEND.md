# WOR-492: Align Front End Apps

## Status
- **Created:** May 7, 2026
- **Status:** IN PROGRESS
- **Priority:** Critical

## Summary
Catalog and fix inconsistencies between frontend, backend API, tests, and documentation.

## Issues Found

### 1. API Base URL Misalignment

**File:** `web/api-integration.js`
- **Current:** `'http://localhost:8080/api/v1'` (line 32)
- **Issue:** Hardcoded to port 8080

**File:** `src/api/mod.rs`
- **Expected:** Backend runs on configurable port (default 8080)

**Current Test Config:** `e2e/frontend-smoke.config.ts`
- Points to `http://localhost:8765` (frontend preview)

### 2. Missing API Endpoints in Frontend

The frontend `api-integration.js` calls these endpoints:
- `/worlds` - POST create
- `/worlds/:id/generate` - POST 
- `/worlds/:id/planet` - GET
- `/worlds/:id/map` - GET
- `/worlds/:id/timeline` - GET
- `/worlds/:id/history` - GET
- `/worlds/:id/wonders` - GET
- `/worlds/:id/disasters` - GET

Backend implements (from `src/api/v1/worlds.rs`):
- All the above ✓
- Additional: `/worlds/:id/figures`, `/worlds/:id/societies`, `/worlds/:id/tectonics`, etc.

### 3. CORS Configuration

**File:** `src/api/mod.rs`
```rust
.allow_origin(AllowOrigin::list([
    "http://localhost:8765".parse().unwrap(),
    "http://localhost:8080".parse().unwrap(),
]))
```

**Issue:** Only `localhost` is allowed. QA tests access via `127.0.0.1` which is a different origin.

### 4. Smoke Test DOM Selectors

**File:** `e2e/frontend-smoke-tests.spec.ts`

| Selector | Expected Element | Status |
|----------|-----------------|--------|
| `#map-canvas` | Canvas element (line 964) | ✓ Verified |
| `#overlay-controls` | Overlay container (line 1004) | ✓ Verified |
| `[data-overlay="resources"]` | Resources button (line 1005) | ✓ Verified |
| `[data-overlay="elevation"]` | Elevation button (line 1011) | ✓ Verified |
| `[data-overlay="political"]` | Political button (line 1017) | ✓ Verified |
| `[data-overlay="wonders"]` | Wonders button (line 1023) | ✓ Verified |
| `#overlay-legend` | Legend panel (line 1031) | ✓ Verified |
| `#zoom-level` | Zoom indicator (line 982) | ✓ Verified |
| `#timeline-container` | Timeline container (line 1102) | ✓ Verified |
| `.view-tab` | View tabs (line 85-87 in HTML) | ✓ Verified |
| `#timeline-view` | Timeline view wrapper (line 1074) | ✓ Verified |

### 5. API Response Format

**Backend Response:**
```json
{
  "success": true,
  "data": { ... }
}
```

**Frontend Handling (api-integration.js):**
```javascript
const worldData = result.data.data || result.data;
```

Handles both wrapped (`result.data.data`) and unwrapped (`result.data`) responses.

## Actions Taken

### 2026-05-07

1. **CORS Fix**: Updated `src/api/mod.rs` to add `127.0.0.1:8765` and `127.0.0.1:8080` to CORS allowed origins
   - This fixes the CORS preflight failures when frontend is accessed via `127.0.0.1` instead of `localhost`
2. **API Contract Update**: Verified all endpoints documented in `docs/API_CONTRACT.md` are implemented in backend
3. **DOM Selector Verification**: Confirmed all smoke test selectors match actual HTML elements in `web/index.html`
4. **Documentation Fix**: Updated `web/api-integration.js` header comment to reflect correct default port (8080)
5. **Cleanup**: Removed stale `web/index.html.bak` backup file
6. **API Base URL Config**: The API_BASE can be overridden via `window.API_BASE` (already implemented, documented)

## Remaining Work

1. [x] Update `src/api/mod.rs` CORS for 127.0.0.1 (DONE)
2. [x] Verify smoke test selectors (DONE)
3. [x] Update `web/api-integration.js` documentation (DONE)
4. [x] Remove stale backup file `web/index.html.bak` (DONE)
5. [x] Environment-based API_BASE configuration (ALREADY IMPLEMENTED - use `window.API_BASE`)

## Actions Addressed from CTO Review

1. ✅ **index.html.bak removed** - File deleted from web/ directory
2. ✅ **Documentation fixed** - api-integration.js now correctly documents default port 8080
3. ✅ **Environment config verified** - `window.API_BASE` override was already implemented

## Demo Files Analysis

The `web/` directory contains both the main application AND QA tools:

| File | Purpose | Status |
|------|---------|--------|
| `web/index.html` | Main SPA (89KB) | ✓ ACTUAL APP |
| `web/api-integration.js` | API client module | ✓ Part of main app |
| `web/hex-test.html` | Hex tiling QA tool (WOR-436) | QA tool |
| `web/hex-tiling-verification.html` | Hex verification | QA tool |
| `web/wor205-qa-test.html` | Polygon QA test (WOR-205) | QA tool |

**The main frontend application IS unified:**
- `web/index.html` (and built copy in `web/dist/index.html`)
- Single-page application with map viewer, timeline, controls
- Serves on port 8765 via `npm run preview`
- Connects to backend API at port 8080

**Note:** QA tools remain in `web/` for easy browser testing.

## Verification

Run smoke tests:
```bash
npm run preview &  # Start frontend on 8765
cargo run -- --server &  # Start backend on 8080
npx playwright test e2e/frontend-smoke-tests.spec.ts
```