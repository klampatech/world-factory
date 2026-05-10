# WOR-944 Smoke Test Report

**Date:** 2026-05-09
**Commit:** 44f3a79 fix(WOR-921): Use preview server with API proxy for frontend
**Overall:** ❌ FAIL

## API Results (16/18 passed)

| # | Endpoint | Method | Status | Result | Details |
|---|----------|--------|--------|--------|---------|
| 1 | /api/v1/worlds | POST | 201 | ✅ PASS | worldId=world:ce581ec2-5580-43b6-9306-8ebe6a4c3b32 |
| 2 | /api/v1/worlds | GET | 200 | ✅ PASS | 9 worlds |
| 3 | /api/v1/worlds/:id | GET | 200 | ✅ PASS | name=undefined |
| 4 | /api/v1/worlds/:id | DELETE | 204 | ❌ FAIL |  |
| 5 | /api/v1/worlds/:id/planet | GET | 200 | ✅ PASS | hasData=true |
| 6 | /api/v1/worlds/:id/map | GET | 200 | ✅ PASS | polygons=132 |
| 7 | /api/v1/worlds/:id/history | GET | 200 | ✅ PASS | events=0 |
| 8 | /api/v1/worlds/:id/history/events | GET | 200 | ✅ PASS | events=0 |
| 9 | /api/v1/worlds/:id/figures | GET | 200 | ✅ PASS | count=0 |
| 10 | /api/v1/worlds/:id/figures/:figure_id | GET | 0 | ❌ FAIL | no figures |
| 11 | /api/v1/worlds/:id/settlements | GET | 200 | ✅ PASS | count=0 |
| 12 | /api/v1/worlds/:id/settlements/map | GET | 200 | ✅ PASS |  |
| 13 | /api/v1/worlds/:id/resources/summary | GET | 200 | ✅ PASS |  |
| 14 | /api/v1/worlds/:id/disasters | GET | 200 | ✅ PASS |  |
| 15 | /api/v1/worlds/:id/artifacts | GET | 200 | ✅ PASS |  |
| 16 | /api/v1/worlds/:id/export | GET | 200 | ✅ PASS |  |
| 17 | /api/v1/worlds/:id/export.json | GET | 200 | ✅ PASS |  |
| 18 | /api/v1/worlds/:id/figures | GET | 200 | ✅ PASS |  |

## Frontend UI Results (6/9 passed)

| # | Test | Result | Details |
|---|------|--------|---------|
| 1 | Frontend loads | ✅ PASS | title=World Selector | ProceduralWorld |
| 2 | World list renders | ✅ PASS | page loaded |
| 3 | World detail view loads (ready world) | ❌ FAIL | world status=generating, skipped |
| 4 | Map canvas renders | ✅ PASS |  |
| 5 | Map Voronoi polygons present | ✅ PASS | 132 polygons |
| 6 | Tab navigation works | ✅ PASS | 11 tabs found |
| 7 | Timeline/History tab | ✅ PASS |  |
| 8 | World creation form accessible | ❌ FAIL |  |
| 9 | Browser console errors | ❌ FAIL | 1 error(s) |

## Browser Console Errors: 1

1. Failed to load resource: net::ERR_CONNECTION_REFUSED

## Screenshots

- WOR-944-01-frontend-load.png
- WOR-944-02-world-list.png
- WOR-944-04-map-canvas.png
- WOR-944-05-timeline.png
- WOR-944-06-create-form.png
