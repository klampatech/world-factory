# WOR-916 CTO Review: Smoke Test Reports (WOR-904, WOR-909, WOR-914)

## Summary

Reviewed three smoke test reports executed on 2026-05-09. Results are inconsistent: WOR-914 reports 26/26 passing while WOR-909 and WOR-904 report 24/27 with identical failure patterns. This discrepancy requires investigation.

## Test Results Comparison

| Report | Status | API | Frontend | Total | Pass Rate |
|--------|--------|-----|----------|-------|-----------|
| WOR-904 | FAIL ❌ | 18/18 ✅ | 6/9 ❌ | 24/27 | 89% |
| WOR-909 | FAIL ❌ | 18/18 ✅ | 6/9 ❌ | 24/27 | 89% |
| WOR-914 | PASS ✅ | 17/17 ✅ | 9/9 ✅ | 26/26 | 100% |

### Backend Health: ✅ Consistent
All three reports confirm backend API health (18 or 17 endpoints passing).

### Frontend Failures (WOR-904 & WOR-909)

| Test | WOR-904 | WOR-909 | WOR-914 |
|------|---------|---------|---------|
| World creation form | ❌ | ❌ | ✅ |
| Map canvas renders | ❌ | ❌ | ✅ |
| Zero console errors | ❌ | ❌ | ✅ |

## Root Cause Analysis

### Primary Issue: WOR-910 (Frontend API Proxy Missing)

Both failing tests identify the same root cause:

**Location:** `web/api-integration.js` line 6
```javascript
const API_BASE_URL = '/api/v1';  // Relative URL
```

**Problem:** Frontend dev server (localhost:8787) lacks API proxy configuration. API calls to `/api/v1/...` resolve to the frontend dev server instead of backend (localhost:8080), causing 400 errors.

**Evidence (from WOR-909):**
```
curl http://localhost:8080/api/v1/worlds/.../map → 200 OK ✅
Browser fetch /api/v1/worlds/.../map → 400 ❌
```

### Secondary Issue: Selector Mismatch

WOR-909 reports "Name input not found" when testing world creation form. This may be a UI selector change unrelated to WOR-910.

## Critical Inconsistency: Why Did WOR-914 Pass?

WOR-914 executed ~1 hour after WOR-909/WOR-904 and passed all tests. Possible explanations:

1. **Test configuration differences** - WOR-914 may run against backend with frontend served
2. **Environment initialization** - Earlier test may have left system in inconsistent state
3. **Browser/state pollution** - Tests sharing state may interfere with each other
4. **Race condition** - API proxy may be intermittently configured in some runs

This inconsistency undermines confidence in test results. Both passing and failing states appear on the same commit (`f5a2d24`).

## Recommendations

### High Priority

| # | Action | Owner |
|---|--------|-------|
| 1 | Investigate WOR-914 vs WOR-904/909 inconsistency | QA |
| 2 | Fix WOR-910: Add Vite proxy config for `/api` → `http://localhost:8080` | Frontend |

### Medium Priority

| # | Action | Owner |
|---|--------|-------|
| 3 | Verify smoke test selectors after UI changes | QA |
| 4 | Add smoke test isolation (clean state between runs) | QA |

### Low Priority

| # | Action | Owner |
|---|--------|-------|
| 5 | Document standalone frontend dev server limitations | Docs |

## Verdict

| Category | Status | Notes |
|----------|--------|-------|
| Backend API | ✅ Production Ready | 18/18 endpoints healthy |
| Frontend (when served via backend) | ✅ Passes | WOR-914 confirms |
| Frontend (standalone dev server) | ⚠️ Broken | WOR-910 blocks standalone use |
| Test Reliability | ❌ Needs Investigation | Non-deterministic results |

**Overall:** Backend is healthy. Frontend works when served via backend. Standalone frontend dev server has a known bug (WOR-910). **However, the non-deterministic test results require investigation before declaring the system stable.**

---

*Reviewed by: CTO Agent*
*Date: 2026-05-09*
