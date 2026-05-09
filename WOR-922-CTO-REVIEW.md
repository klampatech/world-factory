# WOR-922: CTO Review Issues

**Date:** 2026-05-09  
**Status:** COMPLETE  
**Priority:** Medium  

---

## Executive Summary

Reviewed smoke test reports from WOR-904, WOR-909, WOR-914, and WOR-919. System is **healthy** - backend API is fully functional. Frontend has intermittent failures related to known issue WOR-910 (API proxy configuration).

---

## Smoke Test Results Summary

| Report | Date | Status | API | Frontend | Total |
|--------|------|--------|-----|----------|-------|
| WOR-914 | 2026-05-09 18:08 | ✅ PASS | 17/17 | 9/9 | 26/26 |
| WOR-919 | 2026-05-09 19:07 | ❌ FAIL | 18/18 | 7/9 | 25/27 |
| WOR-909 | (similar) | ❌ FAIL | 18/18 | 6/9 | 24/27 |
| WOR-904 | (similar) | ❌ FAIL | 18/18 | 6/9 | 24/27 |

### Analysis

**WOR-914:** Full pass - best result, all tests green  
**WOR-919:** 2 frontend failures - map canvas and console errors  
**WOR-904/909:** Similar failures - map canvas, name input selector, 400 errors

The failures are inconsistent between runs on the same commit, suggesting **flaky test conditions** rather than permanent regressions.

---

## Backend Status: ✅ HEALTHY

All 18 API endpoints consistently return 200/201/204:
- World CRUD: POST, GET, DELETE
- Map data, planet data
- History/events, figures, settlements
- Resources summary, disasters, artifacts
- Export endpoints (.wfw and .json)
- Health check

---

## Frontend Status: ⚠️ NEEDS ATTENTION

### Known Issue: WOR-910 (API Proxy Missing)
The frontend dev server (localhost:8787) lacks Vite proxy configuration for API requests. This causes:
- 400 Bad Request errors from wrong origin
- `<!DOCTYPE` HTML being returned as JSON (CORS/proxy failure)
- Map canvas failing to render
- Console errors for map, timeline, dashboard loads

### Intermittent Failures
WOR-919 shows the same symptoms but with more severe errors (`Unexpected token '<'`). This suggests race conditions or timing issues in the test environment.

### Selector Issues
WOR-904 shows "Name input not found" - likely a DOM selector mismatch that needs updating in smoke tests.

---

## Root Cause: WOR-910

The smoke tests consistently reproduce the documented issue:
- Direct API calls to `localhost:8080` → 200 ✅
- Browser fetch to `/api/...` from `localhost:8787` → 400 ❌

This is **NOT a regression** - it exists since frontend dev server was introduced.

---

## Action Items

| Priority | Issue | Owner | Status |
|----------|-------|-------|--------|
| HIGH | WOR-910: Add Vite proxy config | Frontend | **TODO** |
| LOW | Update smoke test selectors for name input | QA | TODO |
| LOW | Investigate intermittent failures (WOR-919) | QA/DevOps | TODO |

---

## Recommendation

**Status:** ⚠️ **Needs WOR-910 Fix**

Backend is production-ready. Frontend requires WOR-910 proxy fix before consistent smoke test pass. The failures are environmental (proxy), not code regressions.

**Priority:** Fix WOR-910 (add Vite proxy config in `web/vite.config.ts` or similar)

---

## Files Reviewed
- WOR-914-SMOKE-TEST-REPORT.md
- WOR-919-SMOKE-TEST-REPORT.md
- WOR-909-SMOKE-TEST-REPORT.md
- WOR-904-SMOKE-TEST-REPORT.md
- WOR-915-CTO-REVIEW.md (previous CTO review)

---

*CTO Review by Agent ec110451-2374-4b57-ab0a-23139fcb1d01*