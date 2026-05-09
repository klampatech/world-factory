# WOR-835 Smoke Test - Final Report

## Test Execution Summary

| Category | Passed | Total | Status |
|----------|--------|-------|--------|
| **API Endpoints** | 17 | 18 | ⚠️ 1 FAIL |
| **Frontend UI** | 4 | 9 | ❌ 5 FAIL |
| **Total** | 21 | 27 | ❌ FAIL |

---

## 🔴 Critical Issues Found

### Issue 1: `/history/events` endpoint returns 404
- **Endpoint:** `GET /api/v1/worlds/:id/history/events`
- **Expected:** 200 with events array
- **Actual:** 404 Not Found
- **Severity:** HIGH
- **Repro Steps:**
  1. Create a world
  2. Wait for generation to complete
  3. Call `GET /api/v1/worlds/{id}/history/events`
  4. Response is 404 instead of 200

### Issue 2: Frontend World List Elements Not Detected
- **Test:** World list display
- **Expected:** World list items visible with `.world-item` or similar selector
- **Actual:** Selector not found, test fails
- **Severity:** MEDIUM
- **Note:** This may be a test selector issue rather than UI bug

### Issue 3: Create Button Not Found in Frontend
- **Test:** World creation form
- **Expected:** Button with "Create" text visible
- **Actual:** Button not detected
- **Severity:** MEDIUM
- **Note:** May be a selector issue in the test

### Issue 4: Map Canvas Not Detected
- **Test:** Map canvas renders
- **Expected:** `<canvas>` element visible on map view
- **Actual:** Canvas not detected
- **Severity:** MEDIUM
- **Note:** May be a timing issue - map view page loads

### Issue 5: Tab Navigation Failed
- **Test:** Tab navigation
- **Expected:** Tab elements found and clickable
- **Actual:** No tabs detected with test selectors
- **Severity:** MEDIUM
- **Note:** May be a selector issue

### Issue 6: 7 Console Errors (404 Resource Loading)
- **Test:** Zero console errors
- **Expected:** No console errors
- **Actual:** 7 "Failed to load resource: 404" errors
- **Severity:** HIGH
- **Note:** Multiple resources fail to load (likely images, fonts, or API calls)

---

## ✅ Passing Tests

### API (17/18)
- ✅ POST /api/v1/worlds - World creation
- ✅ GET /api/v1/worlds - List worlds  
- ✅ GET /api/v1/worlds/:id - Get world details
- ✅ GET /api/v1/worlds/:id/planet - Planet data
- ✅ GET /api/v1/worlds/:id/map - Map data with Voronoi polygons
- ✅ GET /api/v1/worlds/:id/history - History timeline
- ✅ GET /api/v1/worlds/:id/figures - Figures list
- ✅ GET /api/v1/worlds/:id/settlements - Settlements with 5 species
- ✅ GET /api/v1/worlds/:id/settlements/map - Settlement map
- ✅ GET /api/v1/worlds/:id/resources/summary - Resources (8 types)
- ✅ GET /api/v1/worlds/:id/disasters - Disasters (3 ongoing)
- ✅ GET /api/v1/worlds/:id/artifacts - Artifacts (3 items)
- ✅ GET /api/v1/worlds/:id/export - Export tarball
- ✅ GET /api/v1/worlds/:id/export.json - Export JSON
- ✅ DELETE /api/v1/worlds/:id - World deletion
- ✅ GET /health - Health check
- ⚠️ GET /api/v1/worlds/:id/history/events - **404 ERROR**

### Frontend UI (4/9)
- ✅ Map pan/zoom - Interactive map works
- ✅ Timeline loads - Timeline page renders
- ✅ Dashboard loads - Dashboard page renders
- ✅ Figures page loads - Figures page renders
- ❌ World creation form - Button not detected
- ❌ World list display - Elements not found
- ❌ Map canvas renders - Canvas not detected
- ❌ Tab navigation - Tabs not found
- ❌ Zero console errors - 7 errors found

---

## Screenshots Captured

| Screenshot | Description | Status |
|------------|--------------|--------|
| 01_landing_page.png | Landing page | ✅ Captured |
| 05_world_list.png | World list view | ✅ Captured |
| 06_map_view.png | Map view | ✅ Captured |
| 07_map_zoomed.png | Map after zoom | ✅ Captured |
| 08_timeline.png | Timeline view | ✅ Captured |
| 09_dashboard.png | Dashboard view | ✅ Captured |
| 10_figures.png | Figures page | ✅ Captured |
| 11_tabs_default.png | Default tabs view | ✅ Captured |

---

## Test Environment

- **Branch:** main
- **Commit:** 010733c807ad5189c94510d7895614e975fb1062
- **Backend:** Running on http://localhost:8080
- **Frontend:** Running on http://localhost:5173
- **Date:** 2026-05-09T02:04:13Z

---

## Next Steps

1. **WOR-835-BUG-1**: Fix `/history/events` endpoint returning 404
   - Assign to: Backend Developer
   - Priority: HIGH

2. **WOR-835-BUG-2**: Investigate console 404 errors
   - Assign to: Frontend Developer
   - Priority: HIGH

3. **WOR-835-BUG-3**: Update smoke test selectors for frontend tests
   - Assign to: QA Engineer
   - Priority: MEDIUM
   - Note: Some failures may be test selector issues, not actual UI bugs
