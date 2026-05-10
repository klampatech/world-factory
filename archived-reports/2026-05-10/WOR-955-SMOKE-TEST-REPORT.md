# WOR-955 Smoke Test Report

**Date:** 2026-05-10
**Commit:** 88a31e6 WOR-953: CTO review - Smoke test cycle verification (WOR-944, WOR-946, WOR-952)
**Overall:** ❌ FAIL

## API Results (17/18 passed)

| # | Endpoint | Method | Status | Result | Details |
|---|----------|--------|--------|--------|---------|
| 1 | /api/v1/worlds | POST | 201 | ✅ PASS | worldId=world:3036e8ee-082e-4f36-8588-3895eefb9cb8 |
| 2 | /api/v1/worlds | GET | 200 | ✅ PASS | 12 worlds |
| 3 | /api/v1/worlds/:id | GET | 200 | ✅ PASS | name=undefined |
| 4 | /api/v1/worlds/:id | DELETE | 204 | ✅ PASS |  |
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

## Frontend UI Results (9/9 passed)

| # | Test | Result | Details |
|---|------|--------|---------|
| 1 | Frontend loads | ✅ PASS | title=World Selector | ProceduralWorld |
| 2 | World list renders | ✅ PASS | page loaded |
| 3 | World detail view loads | ✅ PASS | status=generating |
| 4 | Map canvas renders | ✅ PASS |  |
| 5 | Map Voronoi polygons present | ✅ PASS | 132 polygons |
| 6 | Tab navigation works | ✅ PASS | 11 tabs found |
| 7 | Timeline/History tab | ✅ PASS |  |
| 8 | World creation form accessible | ✅ PASS | modal opened with form inputs |
| 9 | Browser console errors | ✅ PASS | 0 error(s) |

## Browser Console Errors: 0

None ✅

## Screenshots

- WOR-955-01-frontend-load.png
- WOR-955-02-world-list.png
- WOR-955-03-world-detail-ready.png
- WOR-955-04-map-canvas.png
- WOR-955-05-timeline.png
- WOR-955-06-create-form.png
