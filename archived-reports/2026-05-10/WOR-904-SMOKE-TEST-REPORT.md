# WOR-904 Smoke Test Report

## Test Execution
- **Date:** 2026-05-09T17:00:48.396Z
- **Branch:** main (latest)
- **Commit:** f5a2d24d5505877c529dea73dc05c73975b4ffa2

## Results Summary
- **Status:** FAIL ❌
- **API Endpoints:** 18/18 passed
- **Frontend Tests:** 6/9 passed
- **Total:** 24/27 passed

## API Endpoint Results
- ✅ POST /api/v1/worlds: 201
- ✅ GET /api/v1/worlds: 200
- ✅ GET /api/v1/worlds/:id: 200
- ✅ GET /api/v1/worlds/:id/planet: 200
- ✅ GET /api/v1/worlds/:id/map: 200
- ✅ GET /api/v1/worlds/:id/history: 200
- ✅ GET /api/v1/worlds/:id/history/events: 200
- ✅ GET /api/v1/worlds/:id/figures: 200
- ✅ GET /api/v1/worlds/:id/figures/:figure_id: SKIP (No figures available)
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
- ❌ World creation form (Name input not found)
- ✅ World list display
- ❌ Map canvas renders
- ✅ Map pan/zoom
- ✅ Timeline loads events
- ✅ Dashboard loads
- ✅ Figures page loads
- ✅ Tab navigation
- ❌ Zero console errors (Failed to load resource: the server responded with a status of 400 (Bad Request); Failed to load map: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadMapData (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:1724:29); Failed to load resource: the server responded with a status of 400 (Bad Request); Failed to load timeline: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadTimeline (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:1825:32); Failed to load resource: the server responded with a status of 400 (Bad Request); Failed to load dashboard: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadDashboard (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:2220:31))

## Console Errors
- Failed to load resource: the server responded with a status of 400 (Bad Request)
- Failed to load map: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadMapData (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:1724:29)
- Failed to load resource: the server responded with a status of 400 (Bad Request)
- Failed to load timeline: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadTimeline (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:1825:32)
- Failed to load resource: the server responded with a status of 400 (Bad Request)
- Failed to load dashboard: Error: HTTP 400
    at WorldApiClient.request (http://localhost:8787/worlds/api-integration.js:124:27)
    at async loadDashboard (http://localhost:8787/worlds/world:3a9ba71c-4854-4e55-8584-bea4413ff762:2220:31)

## Screenshots
- 01_landing_page: screenshots/smoke-test-WOR-904/01_landing_page.png
- 02_world_form: screenshots/smoke-test-WOR-904/02_world_form.png
- 05_world_list: screenshots/smoke-test-WOR-904/05_world_list.png
- 06_map_view: screenshots/smoke-test-WOR-904/06_map_view.png
- 07_map_zoomed: screenshots/smoke-test-WOR-904/07_map_zoomed.png
- 08_timeline: screenshots/smoke-test-WOR-904/08_timeline.png
- 09_dashboard: screenshots/smoke-test-WOR-904/09_dashboard.png
- 10_figures: screenshots/smoke-test-WOR-904/10_figures.png
- 11_tabs_default: screenshots/smoke-test-WOR-904/11_tabs_default.png
- 12_tab_0: screenshots/smoke-test-WOR-904/12_tab_0.png
- 12_tab_1: screenshots/smoke-test-WOR-904/12_tab_1.png
- 12_tab_2: screenshots/smoke-test-WOR-904/12_tab_2.png
- 12_tab_3: screenshots/smoke-test-WOR-904/12_tab_3.png

## Bug Reports
Bugs detected - see results above.
