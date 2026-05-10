# WOR-1093 Smoke Test Report

**Date:** 2026-05-10T19:03:43.650Z

**Summary:** 22/25 tests passed

**Status:** ❌ TESTS FAILED

## Backend API Endpoints (18 Total)

- [✓] POST /api/v1/worlds (Create World)
- [✓] GET /api/v1/worlds (List Worlds)
- [✓] GET /api/v1/worlds/:id (Get World)
- [✗] DELETE /api/v1/worlds/:id (Delete World) — Unexpected end of JSON input
- [✓] GET /api/v1/worlds/:id/planet (Get Planet)
- [✓] GET /api/v1/worlds/:id/map (Get Map)
- [✓] GET /api/v1/worlds/:id/history (Get History)
- [✓] GET /api/v1/worlds/:id/history/events (Get History Events)
- [✓] GET /api/v1/worlds/:id/figures (Get Figures)
- [✓] GET /api/v1/worlds/:id/figures/:figure_id (Get Figure)
- [✓] GET /api/v1/worlds/:id/settlements (Get Settlements)
- [✓] GET /api/v1/worlds/:id/settlements/map (Get Settlements Map)
- [✓] GET /api/v1/worlds/:id/resources/summary (Get Resources Summary)
- [✓] GET /api/v1/worlds/:id/disasters (Get Disasters)
- [✓] GET /api/v1/worlds/:id/artifacts (Get Artifacts)
- [✓] GET /api/v1/worlds/:id/export (Get Export)
- [✓] GET /api/v1/worlds/:id/export.json (Get Export JSON)

## Frontend UI Tests

- [✓] Frontend: World list page loads
- [✓] Frontend: World creation form exists
- [✓] Frontend: World detail page loads
- [✓] Frontend: Map view renders correctly
- [✗] Frontend: Tab navigation works — locator.click: Timeout 30000ms exceeded.
Call log:
  - waiting for locator('[role="tab"], .tab-button, button:has-text("Map"), button:has-text("Timeline"), button:has-text("History")').nth(4)

- [✓] Frontend: Timeline/History loads
- [✓] Frontend: Dashboard/Stats loads
- [✗] Frontend: No console errors

## Console Errors

- [2026-05-10T19:03:52.868Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:03:52.869Z] Console Error: Failed to load world: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async loadWorld (http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1174:35)
    at async HTMLDocument.<anonymous> (http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1076:13)
- [2026-05-10T19:03:52.870Z] Console Error: Failed to load world data
- [2026-05-10T19:03:54.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:03:54.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:03:56.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:03:56.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:03:58.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:03:58.876Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:00.877Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:00.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:02.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:02.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:04.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:04.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:06.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:06.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:08.875Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:08.876Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:10.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:10.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:12.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:12.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:14.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:14.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:16.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:16.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:18.877Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:18.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:20.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:20.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:22.877Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:22.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:24.877Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:24.878Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35
- [2026-05-10T19:04:26.876Z] Console Error: Failed to load resource: the server responded with a status of 404 (Not Found)
- [2026-05-10T19:04:26.877Z] Console Error: Polling failed: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:124:27)
    at async http://localhost:8765/world.html?id=b9aea887-f2de-4c2d-800d-be9f25362caa&tab=map:1835:35

## Screenshots

Screenshots saved to `screenshots/smoke-test-WOR-1093/`:
- 01-*.png
- 02-*.png
- 03-*.png
- 04-*.png
- 05-*.png
- 06-*.png
