## Smoke Test Results

### Execution Summary
- **Date:** 2026-05-09
- **Commit:** df5f97d5fe5aee229520943199db6642138a89f8
- **Result:** FAIL ❌

### API Endpoints: 18/18 ✅ PASS

All 18 backend API endpoints returned expected responses:

- POST /api/v1/worlds: 201
- GET /api/v1/worlds: 200
- GET /api/v1/worlds/:id: 200
- GET /api/v1/worlds/:id/planet: 200
- GET /api/v1/worlds/:id/map: 200
- GET /api/v1/worlds/:id/history: 200
- GET /api/v1/worlds/:id/history/events: 200
- GET /api/v1/worlds/:id/figures: 200
- GET /api/v1/worlds/:id/figures/:figure_id: SKIP (No figures)
- GET /api/v1/worlds/:id/settlements: 200
- GET /api/v1/worlds/:id/settlements/map: 200
- GET /api/v1/worlds/:id/resources/summary: 200
- GET /api/v1/worlds/:id/disasters: 200
- GET /api/v1/worlds/:id/artifacts: 200
- GET /api/v1/worlds/:id/export: 200
- GET /api/v1/worlds/:id/export.json: 200
- DELETE /api/v1/worlds/:id: 204
- GET /health: 200

### Frontend Tests: 7/9 ⚠️ PARTIAL FAIL

✅ World creation form loads
✅ World list display
✅ Map pan/zoom functional
✅ Timeline loads page
✅ Dashboard loads page
✅ Figures page loads
✅ Tab navigation works

❌ **Map canvas renders** - Canvas visible but no world data loaded (HTML returned instead of JSON)
❌ **Zero console errors** - 4 errors detected:
  - `Failed to create world: SyntaxError: Unexpected token '<'`
  - `Failed to load map: SyntaxError: Unexpected token '<'`
  - `Failed to load timeline: SyntaxError: Unexpected token '<'`
  - `Failed to load dashboard: SyntaxError: Unexpected token '<'`

### Root Cause

The frontend static file server (`npx serve`) does not proxy API requests to the backend. The browser sends requests to `/api/v1/*` but the static server returns index.html for non-existent paths.

### Screenshots

Captured 15 screenshots in `screenshots/smoke-test-WOR-919/`

### Bug Filed

- [WOR-921](/WOR/issues/WOR-921) - Frontend API requests fail - no proxy configured for static file server
  - Assigned to: CTO
  - Priority: High

### Recommendation

This is a **critical infrastructure bug** — the frontend is completely non-functional for any real data operations. The CTO should prioritize fixing the API proxy configuration.
