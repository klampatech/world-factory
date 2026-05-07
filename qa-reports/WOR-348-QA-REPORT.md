# WOR-348 Smoke Test QA Report

**Date:** 2026-05-07  
**Tester:** QA Agent  
**Status:** PARTIAL PASS (blocking issues found)

---

## Executive Summary

Executed end-to-end smoke test of World Factory application stack. **11 of 18 API endpoints pass**, **7 fail** due to ID format inconsistencies. Frontend loads and renders map, but has minor visual issues. **Critical blocking issue:** world generation is extremely slow (hours), preventing complete validation of dependent endpoints.

---

## Test Configuration

- **Backend:** Cargo-built binary on `pr-30` branch  
- **Frontend:** Node static file server on port 8765  
- **API Base:** http://localhost:8080/api/v1  
- **Test Duration:** ~15 minutes  

---

## Backend API Results (18 endpoints)

| # | Endpoint | Method | Status | Notes |
|---|----------|--------|--------|-------|
| 1 | Health | GET | ✅ PASS | HTTP 200 |
| 2 | POST /worlds | POST | ✅ PASS | Creates world with `world:UUID` format |
| 3 | GET /worlds | GET | ✅ PASS | Returns 177 worlds |
| 4 | GET /worlds/:id | GET | ❌ FAIL | `world:` prefix rejected by some endpoints |
| 5 | DELETE /worlds/:id | DELETE | ❌ FAIL | 405 Method Not Allowed |
| 6 | GET /worlds/:id/planet | GET | ❌ FAIL | Same ID format issue |
| 7 | GET /worlds/:id/map | GET | ✅ PASS | Works with UUID format |
| 8 | GET /worlds/:id/history | GET | ✅ PASS | Works with UUID format |
| 9 | GET /worlds/:id/history/events | GET | ❌ FAIL | 404 Not Found |
| 10 | GET /worlds/:id/figures | GET | ✅ PASS | Works with UUID format |
| 11 | GET /worlds/:id/figures/:id | GET | ❌ FAIL | 404 Not Found |
| 12 | GET /worlds/:id/settlements | GET | ✅ PASS | Works with UUID format |
| 13 | GET /worlds/:id/settlements/map | GET | ✅ PASS | Works with UUID format |
| 14 | GET /worlds/:id/resources/summary | GET | ✅ PASS | Works with UUID format |
| 15 | GET /worlds/:id/disasters | GET | ✅ PASS | Works with UUID format |
| 16 | GET /worlds/:id/artifacts | GET | ✅ PASS | Works with UUID format |
| 17 | GET /worlds/:id/export | GET | ❌ FAIL | 404 Not Found |
| 18 | GET /worlds/:id/export.json | GET | ❌ FAIL | 404 Not Found |

**Result: 11/18 PASS (61%)**

---

## Bug Report: API ID Format Inconsistency

### Severity: HIGH

### Description
The API has inconsistent ID format handling:

- `POST /worlds` returns IDs with `world:` prefix (e.g., `world:7b4c0889-...`)
- GET list endpoints return same format
- **Some endpoints** reject `world:` prefix (e.g., `/planet`, `/figures/:id`)
- **Other endpoints** only work with raw UUID (e.g., `/map`, `/history`)
- **No endpoint** works with both formats

### Repros Steps
1. Create world via `POST /worlds`
2. Receive ID: `world:7b4c0889-9990-440b-8dd1-d114696a4de0`
3. Try `GET /api/v1/worlds/world:7b4c0889-.../planet` → 400 "Invalid world ID format"
4. Try `GET /api/v1/worlds/7b4c0889-.../planet` → 404 "World not found"
5. Only `/map`, `/history`, `/figures` accept raw UUID

### Evidence
```
GET /worlds/:id/planet with world: prefix:
{"code":"BAD_REQUEST","error":"Invalid world ID format","success":false}

GET /worlds/:id/planet with raw UUID:
{"code":"NOT_FOUND","error":"World '7b4c0889-...' not found","success":false}
```

### Expected Behavior
All world-specific endpoints should accept the same ID format returned by the API.

### Suggested Fix
Standardize on raw UUID (no prefix) for all internal routing, strip prefix at API boundary.

---

## Frontend UI Results

| Component | Status | Notes |
|-----------|--------|-------|
| World List | ⚠️ PARTIAL | Empty state shown, need to expand panel |
| Map View | ✅ PASS | Renders with Voronoi polygons, shows generation progress |
| Timeline | ⚠️ PARTIAL | UI renders but no data visible (world not ready) |
| Navigation | ✅ PASS | Tab switching works |
| Console Errors | ⚠️ 2 errors | "Failed to load resource: net::ERR_CONNECTION_REFUSED" (non-critical) |

### Visual Issues Observed

1. **Generating status text cut off** in map view (see screenshot)
2. **Dark header gradient** may need contrast review
3. **Empty state message** for world list requires user to expand left panel

### Screenshots Captured
- `WOR-348-world-list.png` - World list empty state
- `WOR-348-map-view.png` - Map with generation progress  
- `WOR-348-timeline.png` - Timeline view

---

## Critical Issue: World Generation Too Slow

### Severity: BLOCKING

### Description
Created "WOR-348 Smoke Test" world at 03:06:12. After 15+ minutes, status still shows "generating" with only 0.2% progress. Cannot complete smoke test validation of endpoints that require ready state.

### Timeline
- 03:06:12 - World created
- 03:06:15 - Still at 0.2%
- 03:21:00 - Still at 0.2% (screenshot taken)

### Impact
Cannot test:
- `/planet` endpoint fully
- `/figures/:id` specific figure data
- `/export` endpoints
- Frontend dashboard with complete data

### Recommendation
Investigate generation performance. Consider:
1. Adding generation timeout
2. Creating mock/small worlds for testing
3. Pre-generating test data

---

## Console Errors

| Error | Count | Severity |
|-------|-------|----------|
| Failed to load resource: net::ERR_CONNECTION_REFUSED | 2 | Low (backend fetch attempt) |

These are non-critical - likely frontend polling for data before backend responds.

---

## Recommendations

### P0 (Must Fix Before Next Smoke Test)
1. **Fix API ID format inconsistency** - Create one consistent format
2. **Add DELETE endpoint support** - 405 indicates missing route
3. **Create pre-generated test data** - For smoke test validation

### P1 (Should Fix)
4. **Improve generation speed** - Current speed makes testing impractical
5. **Fix visual clipping** - Status text cut off in map view

### P2 (Nice to Have)
6. Add `/history/events` route
7. Add `/figures/:id` route

---

## Test Artifacts

- Screenshots: `/screenshots/WOR-348-*.png`
- API results: `/qa-reports/WOR-348-results.json`
- Test scripts: `/e2e/smoke-test-wor348.spec.ts`, `/e2e/wor348-api-test.js`

---

**Test Result: PARTIAL PASS** - Core functionality works, but blocking issues prevent full validation.
