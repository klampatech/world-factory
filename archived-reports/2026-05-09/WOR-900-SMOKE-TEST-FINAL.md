# WOR-900 Smoke Test Report - FINAL

**Date:** 2026-05-09T15:04:16Z  
**Duration:** 88s  
**Commit:** Latest main branch  
**Tester:** QA Agent  
**Status:** ❌ SMOKE TEST FAILED

---

## Summary

| Category | Passed | Failed |
|----------|--------|--------|
| Backend API (18 endpoints) | 16 | 2* |
| Frontend UI | 7 | 1** |
| Console Errors | 0 | 9 |
| **Overall** | 23 | 12 |

*\* One endpoint test failed due to no figure data available (see Bug #1 below)*  
*\*\* World creation form button not found (see Bug #2 below)*

---

## Bug #1: World Generation Stuck at "ready" Status

**Severity:** High  
**Issue:** Worlds created via POST /api/v1/worlds remain in "ready" status indefinitely instead of progressing to "complete"  
**Evidence:**
- World `world:04a36d22-290f-48d8-bcd4-ca86f804e653` created successfully but status stayed "ready" after 60 seconds of polling
- 20 worlds in database all show status "generating" - none have reached "complete" status
- GET /api/v1/worlds/:id/figures returns empty array because figures aren't being generated

**Impact:**
- Figure detail endpoint fails (no figures to fetch)
- Frontend shows 404 errors when trying to load world data
- Cannot complete end-to-end smoke test validation

**Reproduction Steps:**
1. POST /api/v1/worlds with valid payload
2. Poll GET /api/v1/worlds/:id every 2 seconds
3. Observe status remains "ready" indefinitely

**Console Errors (sample):**
```
Failed to load world: Error: HTTP 404
    at WorldApiClient.request (http://localhost:8765/api-integration.js:123:27)
Polling failed: Error: HTTP 404
```

---

## Bug #2: World Creation Form - Create Button Not Found

**Severity:** Medium  
**Issue:** Frontend index page does not display a "Create World" button that can be located via Playwright  
**Evidence:**
- Test script searched for: `button:has-text("Create"), button:has-text("New World"), #create-world-btn`
- None of these selectors found a visible button
- Screenshot `screenshots/WOR-900/01-index-page.png` shows the page renders but button not detected

**Impact:**
- Automated E2E test cannot validate the world creation flow
- Manual user flow cannot be verified programmatically

**Reproduction Steps:**
1. Navigate to http://localhost:8765/
2. Look for a clearly labeled "Create World" button
3. Button either missing or uses different selector/labeling

---

## Backend API Test Results

| # | Endpoint | Status | Result |
|---|----------|--------|--------|
| 1 | POST /api/v1/worlds | 201 | ✅ PASS |
| 2 | GET /api/v1/worlds | 200 | ✅ PASS |
| 3 | GET /api/v1/worlds/:id | 200 | ✅ PASS |
| 4 | GET /api/v1/worlds/:id/planet | 200 | ✅ PASS |
| 5 | GET /api/v1/worlds/:id/map | 200 | ✅ PASS |
| 6 | GET /api/v1/worlds/:id/history | 200 | ✅ PASS |
| 7 | GET /api/v1/worlds/:id/history/events | 200 | ✅ PASS |
| 8 | GET /api/v1/worlds/:id/figures | 200 | ✅ PASS |
| 9 | GET /api/v1/worlds/:id/figures/:figure_id | 200 | ❌ FAIL - No figures available |
| 10 | GET /api/v1/worlds/:id/settlements | 200 | ✅ PASS |
| 11 | GET /api/v1/worlds/:id/settlements/map | 200 | ✅ PASS |
| 12 | GET /api/v1/worlds/:id/resources/summary | 200 | ✅ PASS |
| 13 | GET /api/v1/worlds/:id/disasters | 200 | ✅ PASS |
| 14 | GET /api/v1/worlds/:id/artifacts | 200 | ✅ PASS |
| 15 | GET /api/v1/worlds/:id/export | 200 | ✅ PASS |
| 16 | GET /api/v1/worlds/:id/export.json | 200 | ✅ PASS |
| 17 | DELETE /api/v1/worlds/:id | 204 | ✅ PASS |

## Frontend UI Test Results

| Screen/Feature | Status | Evidence |
|----------------|--------|----------|
| World creation form | ❌ FAIL | Button not found |
| World list loads | ✅ PASS | 19 items displayed |
| Map canvas renders | ✅ PASS | Canvas visible |
| Map pan/zoom | ✅ PASS | Interaction works |
| Timeline loads | ✅ PASS | Tab renders |
| Dashboard loads | ✅ PASS | Tab renders |
| Figures tab loads | ✅ PASS | Tab renders |
| Tab navigation | ✅ PASS | All tabs switch |

---

## Screenshots Captured

| Screenshot | File |
|------------|------|
| Index page | `screenshots/WOR-900/01-index-page.png` |
| World list | `screenshots/WOR-900/04-world-list.png` |
| Map view | `screenshots/WOR-900/05-map-view.png` |
| Map zoomed | `screenshots/WOR-900/06-map-zoomed.png` |
| Timeline | `screenshots/WOR-900/07-timeline.png` |
| Dashboard | `screenshots/WOR-900/08-dashboard.png` |
| Figures | `screenshots/WOR-900/09-figures.png` |
| Tab dashboard | `screenshots/WOR-900/11-tab-dashboard.png` |
| Tab map | `screenshots/WOR-900/11-tab-map.png` |
| Tab timeline | `screenshots/WOR-900/11-tab-timeline.png` |
| Tab figures | `screenshots/WOR-900/11-tab-figures.png` |
| Tab settlements | `screenshots/WOR-900/11-tab-settlements.png` |

---

## Conclusion

The smoke test has identified **2 bugs** that need to be addressed before the application can be considered production-ready:

1. **World Generation Stuck** - Requires backend investigation into why worlds don't progress past "ready" status
2. **Create Button Missing** - Requires frontend investigation into world creation form

All API endpoints are functional (17/18 pass at the HTTP level). The single API "failure" is actually a consequence of Bug #1 - no figures exist because the world never fully generates.

---

**Recommendation:** File bug issues for both findings and assign to appropriate agents.
