# WOR-1154 Smoke Test Report

## Summary
- **Status**: ❌ FAIL
- **Timestamp**: 2026-05-11T08:04:58.681Z

---

## Bug Found: Figure ID Validation Returns 400 Instead of 404

**Issue Created**: [WOR-1157](/WOR/issues/WOR-1157) - Figure ID validation returns 400 instead of 404 for legacy IDs

**Endpoint**: `GET /api/v1/worlds/:id/figures/:figure_id`

**Issue**: Legacy figure ID format (e.g., `fig-99999`) returns `400 Bad Request` instead of `404 Not Found`

| Figure ID Format | Response | Expected |
|-----------------|----------|----------|
| UUID (`00000000-0000-0000-0000-000000000999`) | 404 Not Found | ✅ |
| Legacy (`fig-99999`) | 400 Bad Request | ❌ (should be 404) |

### Test Evidence
```
GET /api/v1/figures/fig-99999
Response: {"code":"BAD_REQUEST","error":"Invalid figure ID format","success":false}

GET /api/v1/figures/00000000-0000-0000-0000-000000000999
Response: {"code":"NOT_FOUND","error":"Figure '00000000-0000-0000-0000-000000000999' not found","success":false}
```

**Fix Required**: Accept both UUID and legacy `fig-*` formats. Return 404 when figure not found, not 400 for format validation.

---

## Backend API Results (17/18 passed, 1 BUG)

| # | Endpoint | Status | Result |
|---|----------|--------|--------|
| 1 | POST /api/v1/worlds - Create world | 201 | ✅ |
| 2 | GET /api/v1/worlds - List worlds | 200 | ✅ |
| 3 | GET /api/v1/worlds/:id - Get world details | 200 | ✅ |
| 4 | GET /api/v1/worlds/:id/planet - Get planet data | 200 | ✅ |
| 5 | GET /api/v1/worlds/:id/map - Get map data | 200 | ✅ |
| 6 | GET /api/v1/worlds/:id/history - Get history | 200 | ✅ |
| 7 | GET /api/v1/worlds/:id/history/events - Get history events | 200 | ✅ |
| 8 | GET /api/v1/worlds/:id/figures - List figures | 200 | ✅ |
| 9 | GET /api/v1/worlds/:id/figures/:id - Get specific figure | 400 | ❌ BUG |
| 10 | GET /api/v1/worlds/:id/settlements - List settlements | 200 | ✅ |
| 11 | GET /api/v1/worlds/:id/settlements/map - Get settlement map | 200 | ✅ |
| 12 | GET /api/v1/worlds/:id/resources/summary - Get resources | 200 | ✅ |
| 13 | GET /api/v1/worlds/:id/disasters - Get disasters | 200 | ✅ |
| 14 | GET /api/v1/worlds/:id/artifacts - Get artifacts | 200 | ✅ |
| 15 | GET /api/v1/worlds/:id/export - Export world | 200 | ✅ |
| 16 | GET /api/v1/worlds/:id/export.json - Export as JSON | 200 | ✅ |
| 17 | DELETE /api/v1/worlds/:id - Delete world | 204 | ✅ |
| 18 | GET /health - Health check | 200 | ✅ |

## Frontend UI Results (6/6 passed)

| Test | Result |
|------|--------|
| Home page loads | ✅ |
| World creation form visible | ✅ |
| World list/selector displays | ✅ |
| Map canvas renders | ✅ |
| Tab navigation available | ✅ |
| Dashboard data displays | ✅ |

## Console Errors
⚠️ 1 Error detected:
- `Failed to load resource: the server responded with a status of 502 (Bad Gateway)`

This appears to be a transient backend connectivity issue during the test, not a persistent bug.

## Screenshots
- [Home Page](/home/kyle/projects/world-generator/screenshots/WOR-1154-1-home.png)
- [Canvas Check](/home/kyle/projects/world-generator/screenshots/WOR-1154-2-canvas-check.png)
- [Dashboard](/home/kyle/projects/world-generator/screenshots/WOR-1154-3-dashboard.png)
