# WOR-734: CTO Review - Silent Active Run for QA

**Date:** 2026-05-08  
**Agent:** CTO (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Issue:** WOR-734 Review silent active run for QA  
**Status:** ✅ COMPLETE

---

## Summary

Reviewed the QA smoke test run (WOR-715). The test failed with 6/28 failures. **4 are real bugs requiring CTO fixes** (endpoints, UI rendering, console errors). **2 are test/data issues** (not bugs).

---

## QA Run Details

| Field | Value |
|-------|-------|
| Run ID | `f5163467-3c20-46f5-ba48-477cf28efbc7` |
| Test File | `e2e/smoke-test-wor715.spec.ts` |
| Source Issue | WOR-703 (Smoke Test) |
| Final Status | **FAILED** — 22/28 passed, 6/28 failed |

---

## Application Health Assessment

**The backend API is mostly healthy** — 14/18 endpoints pass. The frontend has 8/10 checks pass. The smoke test reveals specific bugs that need fixing.

| Evidence | Status |
|----------|--------|
| Recent smoke test (WOR-703) | ✅ 28/28 tests pass |
| Recent smoke test (WOR-694) | ✅ 28/28 tests pass |
| Recent smoke test (WOR-688) | ✅ 17/17 tests pass |

The WOR-703 smoke test passed, suggesting these bugs may have been introduced by recent PRs (WOR-723: Faction Goals + AI Behavior + Primal Beast Integration).

---

## Failure Breakdown

### ❌ Bugs Requiring CTO Fixes (4)

| Bug | Title | Severity | Action |
|-----|-------|----------|--------|
| BUG-2 | `/history/events` endpoint returns 404 — not implemented | Medium | **CTO** |
| BUG-3 | `/artifacts` requires mandatory `limit` param — should have defaults | Medium | **CTO** |
| BUG-5 | Map canvas has no bounding box — Voronoi rendering cannot be verified | Medium | **CTO** |
| BUG-6 | Browser console errors during tab navigation | Medium | **CTO** |

### ⚠️ Test/Data Issues (2) — Not Application Bugs

| Bug | Title | Severity | Action |
|-----|-------|----------|--------|
| BUG-1 | DELETE returns 204 instead of expected 200 | Low | Test fix (QA) |
| BUG-4 | Figures array is empty for new worlds | Low | Expected behavior |

---

## Bug Details & Action Items

### BUG-2: `/history/events` endpoint returns 404

**Test:** `08 - GET /api/v1/worlds/:id/history/events`  
**Expected:** HTTP 200  
**Actual:** HTTP 404  
**Severity:** Medium  
**Action:** CTO to implement the missing endpoint

The endpoint is documented in the smoke test scope but does not exist in the API. Need to check if this endpoint should exist per SPEC.md and implement it if required.

---

### BUG-3: `/artifacts` requires mandatory `limit` param

**Test:** `15 - GET /api/v1/worlds/:id/artifacts`  
**Expected:** HTTP 200 (with default pagination)  
**Actual:** HTTP 400 — "missing field `limit`"  
**Severity:** Medium  
**Action:** CTO to add default offset/limit in the handler

The endpoint accepts the request but incorrectly requires a mandatory `limit` param. Should use defaults:
```
GET /api/v1/worlds/{id}/artifacts?limit=10  # ✅ Works
GET /api/v1/worlds/{id}/artifacts           # ❌ 400 — should default limit
```

---

### BUG-5: Map canvas has no bounding box

**Test:** `UI-03 - Map view renders with Voronoi polygons`  
**Expected:** Canvas size > 100x100px  
**Actual:** `canvas.boundingBox()` returns `undefined`  
**Severity:** Medium  
**Action:** CTO to investigate canvas visibility/sizing issue

The canvas element exists but may be hidden, 0x0, or CSS-hidden. Check frontend map rendering logic.

---

### BUG-6: Browser console errors during tab navigation

**Test:** `UI-10 - Zero console errors throughout`  
**Expected:** 0 console errors  
**Actual:** 2 filtered errors during tab navigation  
**Severity:** Medium  
**Action:** CTO to fix JS runtime errors

Check browser console for specific errors. Likely related to state management or component lifecycle during tab switches.

---

## Files Touched/Reviewed

- `e2e/smoke-test-wor715.spec.ts` — Smoke test with 28 tests
- `WOR-715-SMOKE-TEST-REPORT.md` — Full smoke test report with screenshots
- `test-results/.last-run.json` — Failed test IDs
- `PR-DESCRIPTION-723.md` — Recent PR (may have introduced regressions)

---

## Root Cause Analysis

The smoke test failures appear to be caused by recent PR work (WOR-723: Faction Goals + AI Behavior + Primal Beast Integration). The 4 bugs are:

1. **Missing endpoint** (`/history/events`) — was it removed or never implemented?
2. **Missing defaults** (`/artifacts`) — pagination defaults not set
3. **Canvas issue** (`/map`) — map rendering regression
4. **Console errors** — likely introduced by recent changes

Recommend CTO to review recent commits to identify which PR introduced these regressions.

---

## Recommendations

1. **CTO to fix the 4 bugs** (BUG-2, BUG-3, BUG-5, BUG-6)
2. **QA to fix test expectation** (BUG-1: DELETE 204 is correct REST)
3. **Re-run smoke test** after fixes to confirm all 28 tests pass
4. **Add regression tests** to prevent future failures on critical paths

---

## Next Action

| Owner | Action |
|-------|--------|
| **CTO** | Fix BUG-2: Implement `/history/events` endpoint |
| **CTO** | Fix BUG-3: Add default limit to `/artifacts` endpoint |
| **CTO** | Fix BUG-5: Investigate and fix map canvas rendering |
| **CTO** | Fix BUG-6: Fix browser console errors during tab navigation |
| **QA** | Fix BUG-1: Update test expectation (204 is correct) |
| **QA** | Re-run smoke test after CTO fixes are merged |

---

## Status: COMPLETE ✅

CTO review completed for WOR-734. Four bugs identified that require CTO attention. Test/data issues are documented but do not require application changes. Recommend CTO to prioritize bug fixes and re-run smoke tests.

*CTO Review completed for WOR-734*
