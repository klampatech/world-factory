# WOR-847 Smoke Test Report

## Test Execution
- **Date:** 2026-05-09T04:01:39.839Z
- **Branch:** main (latest)
- **Commit:** 68cb0b22e1affc1fee52829990315af238c0ad61

## Results Summary
- **Status:** FAIL ❌
- **API Endpoints:** 17/18 passed
- **Frontend Tests:** 6/9 passed
- **Total:** 23/27 passed

## API Endpoint Results
- ✅ POST /api/v1/worlds: 201
- ✅ GET /api/v1/worlds: 200
- ✅ GET /api/v1/worlds/:id: 200
- ✅ GET /api/v1/worlds/:id/planet: 200
- ✅ GET /api/v1/worlds/:id/map: 200
- ✅ GET /api/v1/worlds/:id/history: 200
- ❌ GET /api/v1/worlds/:id/history/events: 404
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
- ❌ Zero console errors (Failed to load map: ReferenceError: api is not defined
    at loadMapData (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1724:35)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1634:37)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21); Failed to load timeline: ReferenceError: api is not defined
    at loadTimeline (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1825:38)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1637:52)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21); Failed to load dashboard: ReferenceError: api is not defined
    at loadDashboard (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:2220:37)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1640:39)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21))

## Console Errors
- Failed to load map: ReferenceError: api is not defined
    at loadMapData (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1724:35)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1634:37)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21)
- Failed to load timeline: ReferenceError: api is not defined
    at loadTimeline (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1825:38)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1637:52)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21)
- Failed to load dashboard: ReferenceError: api is not defined
    at loadDashboard (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:2220:37)
    at loadTabContent (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1640:39)
    at HTMLButtonElement.<anonymous> (http://localhost:8787/worlds/world:3f8accef-c8d7-4253-b5bd-499136ff6d48:1626:21)

## Screenshots
- 01_landing_page: screenshots/smoke-test-WOR-847/01_landing_page.png
- 02_world_form: screenshots/smoke-test-WOR-847/02_world_form.png
- 05_world_list: screenshots/smoke-test-WOR-847/05_world_list.png
- 06_map_view: screenshots/smoke-test-WOR-847/06_map_view.png
- 07_map_zoomed: screenshots/smoke-test-WOR-847/07_map_zoomed.png
- 08_timeline: screenshots/smoke-test-WOR-847/08_timeline.png
- 09_dashboard: screenshots/smoke-test-WOR-847/09_dashboard.png
- 10_figures: screenshots/smoke-test-WOR-847/10_figures.png
- 11_tabs_default: screenshots/smoke-test-WOR-847/11_tabs_default.png
- 12_tab_0: screenshots/smoke-test-WOR-847/12_tab_0.png
- 12_tab_1: screenshots/smoke-test-WOR-847/12_tab_1.png
- 12_tab_2: screenshots/smoke-test-WOR-847/12_tab_2.png
- 12_tab_3: screenshots/smoke-test-WOR-847/12_tab_3.png

## Bug Reports
Bugs detected - see results above.
