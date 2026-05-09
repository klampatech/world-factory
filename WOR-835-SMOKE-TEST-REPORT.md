# WOR-835 Smoke Test Report

## Test Execution
- **Date:** 2026-05-09T02:06:29.053Z
- **Branch:** main (latest)
- **Commit:** 010733c807ad5189c94510d7895614e975fb1062

## Results Summary
- **Status:** FAIL ❌
- **API Endpoints:** 17/18 passed
- **Frontend Tests:** 4/9 passed
- **Total:** 21/27 passed

## API Endpoint Results
- ✅ POST /api/v1/worlds: 201
- ✅ GET /api/v1/worlds: 200
- ✅ GET /api/v1/worlds/:id: 200
- ✅ GET /api/v1/worlds/:id/planet: 200
- ✅ GET /api/v1/worlds/:id/map: 200
- ✅ GET /api/v1/worlds/:id/history: 200
- ❌ GET /api/v1/worlds/:id/history/events: 404
- ✅ GET /api/v1/worlds/:id/figures: 200
- ✅ GET /api/v1/worlds/:id/figures/:figure_id: SKIP (No figures to test)
- ✅ GET /api/v1/worlds/:id/settlements: 200
- ✅ GET /api/v1/worlds/:id/settlements/map: 200
- ✅ GET /api/v1/worlds/:id/resources/summary: 200
- ✅ GET /api/v1/worlds/:id/disasters: 200
- ✅ GET /api/v1/worlds/:id/artifacts: 200
- ✅ GET /api/v1/worlds/:id/export: 200
- ✅ GET /api/v1/worlds/:id/export.json: 200
- ✅ DELETE /api/v1/worlds/:id: 204
- ✅ GET /health: 200

## Frontend UI Results
- ❌ World creation form (Create button not found)
- ❌ World list display
- ❌ Map canvas renders
- ✅ Map pan/zoom
- ✅ Timeline loads events
- ✅ Dashboard loads
- ✅ Figures page loads
- ❌ Tab navigation
- ❌ Zero console errors (Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found); Failed to load resource: the server responded with a status of 404 (Not Found))

## Console Errors
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)
- Failed to load resource: the server responded with a status of 404 (Not Found)

## Screenshots
- 01_landing_page: screenshots/smoke-test-WOR-835/01_landing_page.png
- 05_world_list: screenshots/smoke-test-WOR-835/05_world_list.png
- 06_map_view: screenshots/smoke-test-WOR-835/06_map_view.png
- 07_map_zoomed: screenshots/smoke-test-WOR-835/07_map_zoomed.png
- 08_timeline: screenshots/smoke-test-WOR-835/08_timeline.png
- 09_dashboard: screenshots/smoke-test-WOR-835/09_dashboard.png
- 10_figures: screenshots/smoke-test-WOR-835/10_figures.png
- 11_tabs_default: screenshots/smoke-test-WOR-835/11_tabs_default.png

## Bug Reports
Bugs detected - see results above.
