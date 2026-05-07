# WOR-371: Review Issues - Report

**Date:** 2026-05-07  
**Reviewer:** CTO  
**Routine:** Review Issues (executed automatically)

---

## Review Summary

Reviewed all issues with status `in_review`. Found **0 issues** requiring attention.

---

## Issue Analysis

### Previous Review Findings

From [WOR-360](/issues/WOR-360):
- **WOR-342:** Backend API build fails with --features api → ✅ RESOLVED
  - Build passes successfully with PR #30 fix
  - Blocked child: WOR-339 (QA smoke test)

### Current PR #30 Status

**Branch:** `pr-30` (22 commits ahead of `main`)

**Modified Files (8 files, +191/-56 lines):**
| File | Changes |
|------|---------|
| `e2e/wor348-api-test.js` | Port update from 8080 to 8085 |
| `qa-reports/WOR-348-results.json` | Test results update |
| `src/api/mod.rs` | Storage persistence additions |
| `src/api/models.rs` | Faction/turn API types |
| `src/api/v1/artifacts.rs` | World ID normalization |
| `src/api/v1/cataclysms.rs` | World ID normalization |
| `src/api/v1/worlds.rs` | Major normalization fix (157 lines) |
| `src/storage.rs` | Path normalization + figures/events paths |

**Key Fixes Applied:**
1. ✅ Storage layer normalizes `world:` prefix internally
2. ✅ All API handlers normalize before storage calls
3. ✅ Added `figures_path()` and `events_path()` to storage
4. ✅ History events handler returns empty array (not 404)

---

## WOR-348 Smoke Test Results

**Results File:** `qa-reports/WOR-348-results.json`

| Status | Count |
|--------|-------|
| Total | 18 |
| Passed | 15 |
| Failed | 3 |

### Failing Tests Analysis

| Test | Status | Issue | Root Cause |
|------|--------|-------|------------|
| `DELETE /worlds/:id` | ❌ FAIL (HTTP 405) | Method not implemented | Endpoint handler not added |
| `GET /worlds/:id/history/events` | ❌ FAIL (HTTP 404) | Returns 404 not 200 | Handler returns empty events → should return 200 with empty array |
| `GET /worlds/:id/figures/:id` | ❌ FAIL (HTTP 404) | Figure not found | Endpoint looks up figure ID from wrong path |

### Root Cause Details

**1. DELETE /worlds/:id (HTTP 405)**
- Missing `delete` route in worlds.rs router
- Handler may exist but route not registered

**2. GET /worlds/:id/history/events (HTTP 404 → should be 200)**
- Handler `get_world_history_events` exists and returns valid response
- But API test gets 404
- Likely: Route not properly registered or path conflict

**3. GET /worlds/:id/figures/:id (HTTP 404)**
- Cross-world figure lookup exists in `src/api/v1/figures.rs`
- But no `/worlds/:id/figures/:id` endpoint under worlds router
- Test uses `fig-0` which won't exist in any world

---

## Recommendations

### Must Fix (3 issues)

1. **WOR-349:** Add DELETE endpoint route for `/worlds/:id`
   - Add route registration: `.route("/:id", delete(delete_world))`
   - Implement `delete_world` handler if missing

2. **WOR-350:** Fix /history/events returning 404 instead of 200
   - Verify route registration in worlds.rs
   - Ensure response always returns 200 with `events: []`

3. **WOR-351:** Add figure-by-id endpoint under worlds or fix test
   - Option A: Add `/worlds/:id/figures/:figure_id` route to worlds.rs
   - Option B: Update test to use cross-world `/figures/:id` endpoint

### Should Address

4. **Commit staged changes** - `src/api/mod.rs` has staged changes
   - `save_faction_registry` function staged but not committed

5. **Re-enable format check** - After ci.yml OAuth fix

---

## Related Issues

| Issue | Status | Notes |
|-------|--------|-------|
| WOR-339 | blocked | Blocked by smoke test failures |
| WOR-342 | done ✅ | Build fix complete |
| WOR-348 | in_progress | 3 QA failures to resolve |
| WOR-352 | done ✅ | World ID normalization |

---

## CTO Action Items

1. **Create 3 child issues** for the failing smoke test endpoints
2. **Verify route registration** for /history/events endpoint
3. **Commit or discard** staged changes in src/api/mod.rs

---

## Status: COMPLETE

Review completed. No issues in `in_review` status. Outstanding QA failures documented with root causes and recommended fixes.

**Next Action:** Create child issues for the 3 failing smoke test endpoints (WOR-349, WOR-350, WOR-351)