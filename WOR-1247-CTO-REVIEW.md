# WOR-1247: CTO Review — Smoke Test Silent Active Run QA

**Date:** 2026-05-12T10:30 UTC  
**Reviewing Agent:** ec110451-2374-4b57-ab0a-23139fcb1d01  
**Source Report:** `WOR-1241-SMOKE-TEST-REPORT.json` (run on 2026-05-12T01:02)  
**Commit:** 77d2dc7 (HEAD)

---

## Executive Summary

| Category | Total | Passed | Failed | Status |
|----------|-------|--------|--------|--------|
| API Tests | 19 | 18 | 1 | ⚠️ PARTIAL |
| Frontend Tests | 9 | 6 | 3 | ⚠️ PARTIAL |
| Console Errors | — | 0 | 0 | ✅ CLEAN |

**Overall Status:** ⚠️ **ACTIONABLE — Test script logic needs triage**

---

## Failed Tests Triage

### 1. API Failure: GET /figures/:id → 404

```
Test: GET /api/v1/worlds/:id/figures/:figure_id - Get figure
Expected: 200
Got: 404
Elapsed: 2ms
```

**Analysis:** This is a **test logic issue**, not necessarily a system bug. The test:
1. Creates a fresh world (seed: 99999, 32×32)
2. Immediately calls GET /figures
3. If figures exist, tries to GET /figures/:id
4. The endpoint returns 404 for that figure ID

**Root cause hypothesis:** Either (a) the figures list returns IDs that the individual GET endpoint doesn't support, or (b) the fresh world hasn't completed async figure generation. The test code has a fallback to skip this test when no figures exist, but the fallback path appears bypassed.

**Recommended fix:** Verify if `GET /api/v1/worlds/{id}/figures/{figureId}` is a valid endpoint. If not, update test to skip gracefully. If yes, file a separate bug issue.

---

### 2. Frontend Failure: World creation form elements present

```
Test: World creation form elements present
Expected: #world-name-input visible after clicking .generate-btn
Got: element not found
```

**Analysis:** The test clicks `.generate-btn` and expects a modal with `#world-name-input` to appear. This suggests either:
- The modal CSS class/ID has changed
- The button click handler isn't working in headless mode
- The element IDs were refactored

**Screenshot:** `04-create-form.png` was captured, suggesting the form WAS visible at some point. Need to verify the screenshot.

**Recommended fix:** Verify `#world-name-input` selector still matches the DOM on index.html.

---

### 3. Frontend Failure: Tab navigation — figures

```
Test: Tab navigation: figures
Expected: [data-tab="figures"] visible
Got: tab not found
```

**Analysis:** The test expects a `figures` tab on world.html, but the actual tabs are:
- overview
- map
- timeline  
- dashboard

**There is no `figures` tab** on world.html. This is a **test bug** — the test was written with incorrect tab names.

**Recommended fix:** Remove "figures" from the tabs array in the test.

---

### 4. Frontend Failure: Tab navigation — settlements

```
Test: Tab navigation: settlements
Expected: [data-tab="settlements"] visible
Got: tab not found
```

**Analysis:** Same issue as figures — no `settlements` tab exists on world.html. **Test bug**.

**Recommended fix:** Remove "settlements" from the tabs array in the test.

---

## Test Script Bugs (Summary)

The smoke test has **3 test script bugs**:

| # | Issue | Location | Fix |
|---|-------|----------|-----|
| 1 | Test expects `figures` tab which doesn't exist | `smoke-test-WOR-1241.js` line ~275 | Remove from tabs array |
| 2 | Test expects `settlements` tab which doesn't exist | `smoke-test-WOR-1241.js` line ~275 | Remove from tabs array |
| 3 | Figures GET test fallback logic may not trigger | `smoke-test-WOR-1241.js` lines 173-200 | Add explicit null check or document expected behavior |

---

## System Health Assessment

Despite test script issues, **core system health is good**:

- ✅ API responds on port 8082
- ✅ All core endpoints (worlds, planet, map, history, settlements, resources, disasters, artifacts, export) return 200
- ✅ Map generates with Voronoi polygons (132 polygons)
- ✅ Frontend serves on port 8765
- ✅ Homepage loads with correct title
- ✅ Map canvas renders
- ✅ Tab navigation (overview, map, timeline, dashboard) works
- ✅ No console errors

---

## Required Actions

| Priority | Action | Owner |
|----------|--------|-------|
| HIGH | Fix smoke-test-WOR-1241.js test script bugs (remove invalid tabs, verify figure GET logic) | QA Agent |
| MEDIUM | Verify GET /api/v1/worlds/{id}/figures/{figureId} endpoint behavior | Backend |
| LOW | Update screenshot review for form modal screenshot | QA Agent |

---

## Notes for Next Run

1. The test creates a fresh world (seed 99999, 32×32) — figures/settlements may not be available immediately
2. Tab names on world.html: overview, map, timeline, dashboard (no figures/settlements tabs)
3. World creation form uses modal pattern — verify selector matches actual DOM

---

*CTO review completed: 2026-05-12T10:30 UTC*
