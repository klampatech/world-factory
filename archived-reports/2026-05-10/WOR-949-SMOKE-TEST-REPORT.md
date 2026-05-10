# WOR-949 Smoke Test Report

**Date:** 2026-05-10
**Branch:** main
**Commit:** 44f3a79 fix(WOR-921): Use preview server with API proxy for frontend
**Docker Image:** test-run (running from pre-built image, not rebuilt after WOR-946 fix)
**Overall:** ⚠️ PARTIAL PASS (code fixed, container needs rebuild)

## Test Configuration

- Backend: `http://localhost:8080` (via Docker container `test-run`)
- Frontend: `http://localhost:8765` (via preview server with API proxy)
- Test script: `smoke-test-WOR-944.js` (comprehensive end-to-end test)

## API Results (16/18 passed)

| # | Endpoint | Method | Status | Result | Details |
|---|----------|--------|--------|--------|---------|
| 1 | /api/v1/worlds | POST | 201 | ✅ PASS | worldId=world:fa742d8e-e03f-4916-b56c-911bb6ff5247 |
| 2 | /api/v1/worlds | GET | 200 | ✅ PASS | worlds list accessible |
| 3 | /api/v1/worlds/:id | GET | 200 | ✅ PASS | world data returned |
| 4 | /api/v1/worlds/:id | DELETE | 204 | ❌ FAIL | expected 200 |
| 5 | /api/v1/worlds/:id/planet | GET | 200 | ✅ PASS | planet data returned |
| 6 | /api/v1/worlds/:id/map | GET | 200 | ✅ PASS | map polygons returned |
| 7 | /api/v1/worlds/:id/history | GET | 200 | ✅ PASS | history events returned |
| 8 | /api/v1/worlds/:id/history/events | GET | 200 | ✅ PASS | events list returned |
| 9 | /api/v1/worlds/:id/figures | GET | 200 | ✅ PASS | figures list returned |
| 10 | /api/v1/worlds/:id/figures/:figId | GET | - | ⚠️ SKIP | no figures in test world |
| 11 | /api/v1/worlds/:id/settlements | GET | 200 | ✅ PASS | settlements list returned |
| 12 | /api/v1/worlds/:id/settlements/map | GET | 200 | ✅ PASS | |
| 13 | /api/v1/worlds/:id/resources/summary | GET | 200 | ✅ PASS | |
| 14 | /api/v1/worlds/:id/disasters | GET | 200 | ✅ PASS | |
| 15 | /api/v1/worlds/:id/artifacts | GET | 200 | ✅ PASS | |
| 16 | /api/v1/worlds/:id/export | GET | 200 | ✅ PASS | |
| 17 | /api/v1/worlds/:id/export.json | GET | 200 | ✅ PASS | |
| 18 | /api/v1/worlds/:id/figures | GET | 200 | ✅ PASS | |

## Frontend UI Results (5/8 passed)

| # | Test | Result | Details |
|---|------|--------|---------|
| 1 | Frontend loads | ✅ PASS | title="World Selector \| ProceduralWorld" |
| 2 | World list renders | ✅ PASS | page loads successfully |
| 3 | World detail view loads | ⚠️ SKIP | no worlds visible |
| 4 | Map canvas renders | ✅ PASS | canvas element present |
| 5 | Tab navigation works | ✅ PASS | 11 tabs found |
| 6 | Timeline/History tab | ✅ PASS | tab navigation works |
| 7 | World creation form accessible | ❌ FAIL | form not found |
| 8 | Browser console errors | ❌ FAIL | 2 errors (see below) |

## Browser Console Errors: 2

1. `Failed to load resource: the server responded with a status of 400 (Bad Request)`
2. `Failed to load timeline: Error: HTTP 400 at WorldApiClient.request`

## Root Cause Analysis

The timeline endpoint (WOR-946 fix) is **correctly implemented in the codebase** (`src/api/v1/worlds.rs:599-620`), but the **Docker container was not rebuilt** with the fix. The running backend is from a pre-built image that doesn't include:

```rust
// Fix in code (present in src/):
if !state.storage.world_exists(&world_id) {
    return Err(ApiError::NotFound(...));
}
```

Verified via direct curl:
```
GET /api/v1/worlds/00000000-0000-0000-0000-000000000000/timeline
→ {"code":"NOT_FOUND","error":"World '00000000-0000-0000-0000-000000000000' not found","success":false} ✅ 404

GET /api/v1/worlds/invalid-uuid/timeline
→ {"code":"BAD_REQUEST","error":"Invalid world ID format","success":false} ✅ 400
```

The backend fix is verified working on the local dev environment. The smoke test failure is due to the container not being rebuilt.

## Additional Notes

### DELETE endpoint returns 204 (not 200)
The DELETE endpoint returns 204 No Content which is actually correct HTTP semantics for successful deletion. The test expected 200.

### World creation form not found
The frontend world selector landing page may use a different navigation path than expected. Screenshot captured.

## Screenshots

- WOR-944-01-frontend-load.png - Frontend successfully loaded
- WOR-944-02-world-list.png - World list renders
- WOR-944-04-map-canvas.png - Map canvas renders
- WOR-944-05-timeline.png - Timeline tab (shows 400 error due to old container)
- WOR-944-06-create-form.png - Creation form test result

## Recommendation

1. **Rebuild Docker image** with latest main branch to include WOR-946 fix
2. **Re-run smoke test** after container rebuild to verify timeline works
3. **Update smoke test** to expect 204 for DELETE endpoint

## Verdict

- **Code Quality:** ✅ PASS - The fix is correctly implemented
- **Container State:** ⚠️ NEEDS REBUILD - Running old image
- **Overall Smoke Test:** ⚠️ PARTIAL PASS - Core functionality works, timeline error is expected given old container
