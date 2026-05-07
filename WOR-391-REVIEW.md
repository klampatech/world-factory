# WOR-391: Review Issues - Report

**Date:** 2026-05-07  
**Reviewer:** CTO  
**Routine:** Review Issues (executed automatically)

---

## Executive Summary

Review completed. Previous review cycles have addressed all critical issues. Current state is healthy with minor implementation gaps documented as backlog.

**Status:** All critical review findings from previous cycles are resolved. Three minor findings from WOR-388 smoke test are non-blocking.

---

## Previous Review Status (from WOR-378, WOR-384)

| Issue | Status | Notes |
|-------|--------|-------|
| WOR-342: Backend API build fails | ✅ RESOLVED | PR #30 fixed import paths |
| WOR-348: Smoke test failures | ✅ FIXED | Commits `54859cf` + `3dd9a78` |
| WOR-352: API handler normalization | ✅ DONE | `normalize_world_id()` implemented |
| WOR-358: Storage fix | ✅ DONE | Path normalization in storage.rs |
| WOR-377: BUG-377-1 case sensitivity | ✅ FIXED | `serde(alias)` in WorldSize |
| WOR-381: WorldSize fix | ✅ COMPLETE | Fix committed (commit `fd59ab8`) |

---

## WOR-388 Smoke Test Results (Latest)

**Source:** `qa-reports/WOR-388-SMOKE-TEST.md`  
**Result:** ✅ PASS with findings

### Frontend UI Tests: 14/14 PASS ✅

All UI functionality working:
- Map renders Voronoi polygons
- Overlay switching works
- Timeline displays events
- No browser console errors

### Backend API Tests: 15/17 PASS

| Result | Count | Endpoints |
|--------|-------|-----------|
| ✅ PASS | 15 | All primary endpoints |
| ⚠️ FAIL | 2 | DELETE (405), /history/events (404 empty) |

---

## Current Findings (WOR-388)

### 🔴 Finding 1: DELETE /api/v1/worlds/:id returns 405 Method Not Allowed
**Severity:** Low  
**Type:** Missing feature  
**Priority:** Backlog (WOR-363)

The DELETE endpoint is not implemented. This is a standard CRUD gap, not a regression.

**Fix:** Add route registration in `src/api/v1/worlds.rs`:
```rust
.route("/:id", delete(delete_world))
```

---

### 🟡 Finding 2: GET /api/v1/worlds/:id/history/events returns 404
**Severity:** Low  
**Type:** Known TODO (missing event persistence)

The route exists but returns 404 when no events are found. This is a stub - events are not persisted during world generation.

**Fix:** Implement event persistence OR change to return 200 with empty array. Tracked as part of history generation feature.

---

### 🟡 Finding 3: /artifacts requires `?limit=N` parameter
**Severity:** Low  
**Type:** API design

The artifacts endpoint requires explicit `limit` query parameter.

**Fix:** Either make `limit` optional with default value, or document this requirement in API_CONTRACT.md.

---

## Commit History (pr-30 Branch)

| Commit | Description |
|--------|-------------|
| `fd59ab8` | WOR-381: Fix WorldSize enum case sensitivity with serde aliases |
| `3dd9a78` | WOR-348: Fix test paths - use /events, remove non-existent /figures/:id |
| `54859cf` | WOR-352 WOR-358: Fix world ID normalization and storage path handling |

---

## Backlog Items (Nice-to-have)

| Item | Priority | Description |
|------|----------|-------------|
| WOR-363 | Low | Add DELETE endpoint for worlds |
| Event persistence | Low | Persist events during world generation |
| Artifacts limit | Low | Make limit optional or document |
| CI format check | Low | Re-enable after OAuth fix |
| Pre-commit hook | Low | Prevent formatting drift |

---

## Status: COMPLETE ✅

All critical review findings have been addressed:
1. ✅ WOR-342: Backend build fixed
2. ✅ WOR-348: 15/17 smoke tests passing
3. ✅ WOR-352: API normalization implemented
4. ✅ WOR-358: Storage path fix committed
5. ✅ WOR-377: BUG-377-1 (case sensitivity) fixed
6. ✅ WOR-381: Fix committed and verified
7. ✅ WOR-388: Smoke test PASS with known findings

**Next Action:** No immediate action required. Backlog items are non-blocking. Consider prioritizing DELETE endpoint (WOR-363) for future milestone.
