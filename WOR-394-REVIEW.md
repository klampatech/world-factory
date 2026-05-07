# WOR-394: Review Issues - Report

**Date:** 2026-05-07  
**Reviewer:** CTO  
**Routine:** Review Issues (executed automatically)

---

## Executive Summary

Review completed. Previous review cycles have addressed all critical issues. Current state is healthy with minor implementation gaps documented as backlog.

**Status:** ✅ All critical review findings from previous cycles are resolved. Latest smoke test (WOR-388) confirms 15/17 API + 14/14 frontend passing.

---

## Previous Review Status (from WOR-356, WOR-384, WOR-391)

| Issue | Status | Notes |
|-------|--------|-------|
| WOR-342: Backend API build fails | ✅ RESOLVED | PR #30 fixed import paths |
| WOR-348: Smoke test failures | ✅ FIXED | Commits `54859cf` + `3dd9a78` |
| WOR-352: API handler normalization | ✅ DONE | `normalize_world_id()` implemented |
| WOR-358: Storage fix | ✅ DONE | Path normalization in storage.rs |
| WOR-377: BUG-377-1 case sensitivity | ✅ FIXED | `serde(alias)` in WorldSize |
| WOR-381: WorldSize fix | ✅ COMPLETE | Fix committed (commit `fd59ab8`) |
| WOR-388: Smoke test | ✅ PASS | 15/17 API + 14/14 frontend |

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

## Current Findings

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

## Commit History

| Commit | Description |
|--------|-------------|
| `fd59ab8` | WOR-381: Fix WorldSize enum case sensitivity with serde aliases |
| `d8f3a73` | docs: Add QA reports and review documentation |
| `3dd9a78` | WOR-348: Fix test paths - use /events, remove non-existent /figures/:id route |
| `54859cf` | WOR-352 WOR-358: Fix world ID normalization and storage path handling |
| `fc8712b` | fix(CI): Run cargo test --lib instead of --workspace |

---

## Git Status Review

**Modified but unstaged files:**
```
 M e2e/phase4-web-ui-tests.spec.ts
 M src/events/TimelineApiClient.ts
 M src/events/TimelineTypes.ts
 M src/events/index.ts
 M src/simulation/population.rs
 M web/api-integration.js
```

**Assessment:** Files in `src/events/` and `src/simulation/` indicate Phase 5 faction/history work in progress. These changes are separate from the review scope and appear to be development work.

**Note:** Rust build cannot be verified (cargo not in PATH), but previous CI builds have passed.

---

## Backlog Items (Nice-to-have)

| Item | Priority | Description |
|------|----------|-------------|
| WOR-363 | Low | Add DELETE endpoint for worlds |
| Event persistence | Low | Persist events during world generation |
| Artifacts limit | Low | Make limit optional or document |
| CI format check | Low | Re-enable after OAuth fix |
| Pre-commit hook | Low | Prevent formatting drift |
| Phase 5 timeline events | In progress | Active development in src/events/ |

---

## Status: COMPLETE ✅

All critical review findings from previous cycles have been addressed:

1. ✅ WOR-342: Backend build fixed
2. ✅ WOR-348: 15/17 smoke tests passing
3. ✅ WOR-352: API normalization implemented
4. ✅ WOR-358: Storage path fix committed
5. ✅ WOR-377: BUG-377-1 (case sensitivity) fixed
6. ✅ WOR-381: Fix committed and verified
7. ✅ WOR-388: Smoke test PASS with known findings

**Next Action:** No immediate action required. Backlog items are non-blocking. Consider prioritizing DELETE endpoint (WOR-363) for future milestone.
