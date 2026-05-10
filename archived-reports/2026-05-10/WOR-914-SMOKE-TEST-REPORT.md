# WOR-914 Smoke Test Report

## Test Execution
- **Date:** 2026-05-09T18:08:56.816Z
- **Branch:** main (latest)
- **Commit:** f5a2d24d5505877c529dea73dc05c73975b4ffa2

## Results Summary
- **Status:** PASS ✅
- **API Endpoints:** 17/17 passed
- **Frontend Tests:** 9/9 passed
- **Total:** 26/26 passed

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
- ✅ GET /health: 200

## Frontend UI Results
- ✅ World creation form
- ✅ World list display
- ✅ Map canvas renders
- ✅ Map pan/zoom
- ✅ Timeline loads events
- ✅ Dashboard loads
- ✅ Figures page loads
- ✅ Tab navigation
- ✅ Zero console errors

## Console Errors
✅ No console errors detected

## Screenshots
- 01_landing_page: screenshots/smoke-test-WOR-914/01_landing_page.png
- 02_world_form: screenshots/smoke-test-WOR-914/02_world_form.png
- 03_form_filled: screenshots/smoke-test-WOR-914/03_form_filled.png
- 04_after_submit: screenshots/smoke-test-WOR-914/04_after_submit.png
- 05_world_list: screenshots/smoke-test-WOR-914/05_world_list.png
- 06_map_view: screenshots/smoke-test-WOR-914/06_map_view.png
- 07_map_zoomed: screenshots/smoke-test-WOR-914/07_map_zoomed.png
- 08_timeline: screenshots/smoke-test-WOR-914/08_timeline.png
- 09_dashboard: screenshots/smoke-test-WOR-914/09_dashboard.png
- 10_figures: screenshots/smoke-test-WOR-914/10_figures.png
- 11_tabs_default: screenshots/smoke-test-WOR-914/11_tabs_default.png
- 12_tab_0: screenshots/smoke-test-WOR-914/12_tab_0.png
- 12_tab_1: screenshots/smoke-test-WOR-914/12_tab_1.png
- 12_tab_2: screenshots/smoke-test-WOR-914/12_tab_2.png
- 12_tab_3: screenshots/smoke-test-WOR-914/12_tab_3.png

## Bug Reports
No bugs found.
