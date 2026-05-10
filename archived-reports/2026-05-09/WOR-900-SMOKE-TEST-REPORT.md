# WOR-900 Smoke Test Report

**Date:** 2026-05-09T15:04:16.760Z
**Duration:** 88s
**World ID:** world:04a36d22-290f-48d8-bcd4-ca86f804e653

## Summary

- **Backend API:** 16 passed, 1 failed
- **Frontend UI:** 7 passed, 1 failed
- **Console Errors:** 9 found

## Backend API Tests (18 endpoints)

| Endpoint | Status | Notes |
|----------|--------|-------|
| POST /api/v1/worlds - Create world | ✅ PASS |  |
| GET /api/v1/worlds - List worlds | ✅ PASS | Found 29 worlds |
| GET /api/v1/worlds/:id - Get world details | ✅ PASS |  |
| GET /api/v1/worlds/:id/planet | ✅ PASS |  |
| GET /api/v1/worlds/:id/map | ✅ PASS |  |
| GET /api/v1/worlds/:id/history | ✅ PASS |  |
| GET /api/v1/worlds/:id/history/events | ✅ PASS |  |
| GET /api/v1/worlds/:id/figures | ✅ PASS |  |
| GET /api/v1/worlds/:id/figures/:figure_id | ❌ FAIL | No figures available |
| GET /api/v1/worlds/:id/settlements | ✅ PASS |  |
| GET /api/v1/worlds/:id/settlements/map | ✅ PASS |  |
| GET /api/v1/worlds/:id/resources/summary | ✅ PASS |  |
| GET /api/v1/worlds/:id/disasters | ✅ PASS |  |
| GET /api/v1/worlds/:id/artifacts | ✅ PASS |  |
| GET /api/v1/worlds/:id/export | ✅ PASS | Status: 200 |
| GET /api/v1/worlds/:id/export.json | ✅ PASS |  |
| DELETE /api/v1/worlds/:id - Delete world | ✅ PASS | Status: 204 |

## Frontend UI Tests

| Screen/Feature | Status | Notes |
|----------------|--------|-------|
| World creation form | ❌ FAIL | No create button found |
| World list loads | ✅ PASS | Found 19 items |
| Map canvas renders | ✅ PASS | Canvas count: 1 |
| Map pan/zoom | ✅ PASS |  |
| Timeline loads | ✅ PASS |  |
| Dashboard loads | ✅ PASS |  |
| Figures tab loads | ✅ PASS |  |
| Tab navigation | ✅ PASS |  |

## Console Errors

1. `Failed to load resource: the server responded with a status of 404 (Not Found)`

2. `Failed to load world: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:123:27)
    at async loadWorld (http://localhost:8765/world.html?id=world:04a36d22-290f-48d8-bcd4-ca86f804e653:1174:35)
    at async HTMLDocument.<anonymous> (http://localhost:8765/world.html?id=world:04a36d22-290f-48d8-bcd4-ca86f804e653:1076:13)`

3. `Failed to load world data`

4. `Failed to load resource: the server responded with a status of 404 (Not Found)`

5. `Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:123:27)
    at async http://localhost:8765/world.html?id=world:04a36d22-290f-48d8-bcd4-ca86f804e653:1819:35`

6. `Failed to load resource: the server responded with a status of 404 (Not Found)`

7. `Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:123:27)
    at async http://localhost:8765/world.html?id=world:04a36d22-290f-48d8-bcd4-ca86f804e653:1819:35`

8. `Failed to load resource: the server responded with a status of 404 (Not Found)`

9. `Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:123:27)
    at async http://localhost:8765/world.html?id=world:04a36d22-290f-48d8-bcd4-ca86f804e653:1819:35`


## Screenshots

- 01-index-page: `screenshots/WOR-900/01-index-page.png`
- 04-world-list: `screenshots/WOR-900/04-world-list.png`
- 05-map-view: `screenshots/WOR-900/05-map-view.png`
- 06-map-zoomed: `screenshots/WOR-900/06-map-zoomed.png`
- 07-timeline: `screenshots/WOR-900/07-timeline.png`
- 08-dashboard: `screenshots/WOR-900/08-dashboard.png`
- 09-figures: `screenshots/WOR-900/09-figures.png`
- 11-tab-dashboard: `screenshots/WOR-900/11-tab-dashboard.png`
- 11-tab-map: `screenshots/WOR-900/11-tab-map.png`
- 11-tab-timeline: `screenshots/WOR-900/11-tab-timeline.png`
- 11-tab-figures: `screenshots/WOR-900/11-tab-figures.png`
- 11-tab-settlements: `screenshots/WOR-900/11-tab-settlements.png`

---

**Overall Status: ❌ FAILED**
