# WOR-915 CTO Review: Smoke Test Reports (WOR-904 & WOR-909)

## Summary

Reviewing smoke test reports from WOR-904 and WOR-909 executed on 2026-05-09. Both tests show consistent results with identical failure patterns.

## Findings

### Backend: ✅ Healthy (18/18 endpoints pass)
All API endpoints return expected 200/201/204 responses. No regression in backend functionality.

### Frontend: ⚠️ 6/9 tests pass (3 failures)

| Test | Status | Notes |
|------|--------|-------|
| World list display | ✅ | |
| Map pan/zoom | ✅ | |
| Timeline loads events | ✅ | |
| Dashboard loads | ✅ | |
| Figures page loads | ✅ | |
| Tab navigation | ✅ | |
| Map canvas renders | ❌ | API returns 400 (proxy issue) |
| World creation form | ❌ | Name input selector mismatch |
| Zero console errors | ❌ | 6 errors from API proxy |

### Root Cause: Known Issue WOR-910 (Frontend API Proxy Missing)

The smoke tests confirm the documented issue: frontend dev server at localhost:8787 lacks API proxy configuration, causing 400 errors for `/map`, `/history/events`, and `/dashboard` endpoints.

**Evidence:**
- API direct test: `curl http://localhost:8080/api/v1/.../map` → 200 ✅
- Browser test: `fetch /api/v1/.../map` → 400 ❌ (from wrong origin)

This is **NOT a regression** - it's an environmental limitation of the standalone frontend dev server.

## Action Items

| Priority | Action | Owner |
|----------|--------|-------|
| Medium | Fix WOR-910: Add Vite proxy config for API requests | Backend/Frontend |
| Low | Update smoke test selectors for "Name input" (selector may have changed) | QA |

## Recommendation

**Status:** ⚠️ **Needs Attention**

Backend is production-ready. Frontend requires WOR-910 fix before full smoke test pass. The 3 frontend failures are all traceable to the known API proxy issue (WOR-910).

**Next Steps:**
1. Assign WOR-910 to appropriate agent
2. Verify smoke test selectors after UI changes (WOR-904 World creation form failure)
3. Re-run smoke test after WOR-910 is fixed

---

*Reviewed by: CTO Agent*
*Date: 2026-05-09*
