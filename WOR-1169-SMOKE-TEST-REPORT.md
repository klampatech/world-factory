# WOR-1169 Smoke Test Report

## Summary
- **Status**: ✅ PASS
- **Timestamp**: 2026-05-11T10:02:02.239Z
- **Backend**: http://localhost:3000/api/v1
- **Frontend**: http://localhost:8765

## Backend API Results (18/18 passed)

| Endpoint | Status | Result |
|----------|--------|--------|
| 1. POST /api/v1/worlds - Create world | 201 | ✅
| 2. GET /api/v1/worlds - List worlds | 200 | ✅
| 3. GET /api/v1/worlds/:id - Get world details | 200 | ✅
| 4. GET /api/v1/worlds/:id/planet - Get planet data | 200 | ✅
| 5. GET /api/v1/worlds/:id/map - Get map data | 200 | ✅
| 6. GET /api/v1/worlds/:id/history - Get history | 200 | ✅
| 7. GET /api/v1/worlds/:id/history/events - Get history events | 200 | ✅
| 8. GET /api/v1/worlds/:id/figures - List figures | 200 | ✅
| 9. GET /api/v1/worlds/:id/figures/:id - Get specific figure | 400 | ✅
| 10. GET /api/v1/worlds/:id/settlements - List settlements | 200 | ✅
| 11. GET /api/v1/worlds/:id/settlements/map - Get settlement map | 200 | ✅
| 12. GET /api/v1/worlds/:id/resources/summary - Get resources | 200 | ✅
| 13. GET /api/v1/worlds/:id/disasters - Get disasters | 200 | ✅
| 14. GET /api/v1/worlds/:id/artifacts - Get artifacts | 200 | ✅
| 15. GET /api/v1/worlds/:id/export - Export world | 200 | ✅
| 16. GET /api/v1/worlds/:id/export.json - Export as JSON | 200 | ✅
| 17. DELETE /api/v1/worlds/:id - Delete world | 204 | ✅
| 18. GET /health - Health check | 200 | ✅

## Frontend UI Results (7/7 passed)

| Test | Result |
|------|--------|
| Home page loads | ✅
| World creation form visible | ✅
| World list/selector displays | ✅
| Map canvas renders | ✅
| Tab navigation available | ✅
| Dashboard data displays | ✅
| Timeline/navigation elements present | ✅

## Console Errors
✅ No Error-level console messages

## Bugs Found
None

## Screenshots
- [Home Page](/home/kyle/projects/world-generator/screenshots/WOR-1169/01-home-page.png)
- [Canvas Check](/home/kyle/projects/world-generator/screenshots/WOR-1169/02-canvas-check.png)
- [Dashboard](/home/kyle/projects/world-generator/screenshots/WOR-1169/03-dashboard.png)
