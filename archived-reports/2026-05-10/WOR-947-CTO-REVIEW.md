# WOR-947 CTO Review: Timeline Endpoint World Existence Check

## Review Summary

**Date:** 2026-05-09  
**Issue:** WOR-946 - Timeline endpoint returns HTTP 400 for 'generating' status worlds  
**Review Type:** Smoke Test Verification  
**Result:** ✅ APPROVED - All Tests Passed

## Issue Background

WOR-946 reported that the timeline endpoint (`GET /api/v1/worlds/:id/timeline`) was missing a world existence check, causing:
1. Non-existent worlds to return 200 OK with empty timeline (should be 404)
2. Inconsistent behavior compared to other endpoints like `get_world_events`, `get_world_figures`, etc.

## Fix Applied

**File:** `src/api/v1/worlds.rs`

The fix changed `get_world_timeline` handler to:
1. Use `State(state)` instead of `State(_state)` (was ignoring state)
2. Added world existence check using `state.storage.world_exists(&world_id)`

```rust
// Check if world exists in storage
if !state.storage.world_exists(&world_id) {
    return Err(ApiError::NotFound(format!(
        "World '{}' not found",
        world_id
    )));
}
```

## Test Coverage

**File:** `tests/api_endpoints_test.rs`  
New test: `test_get_world_timeline_not_found_returns_404()`

**Smoke Test:** `smoke-test-WOR-946.js`
| Test | Description | Expected | Actual | Status |
|------|-------------|----------|--------|--------|
| 1 | Non-existent world timeline | 404 NOT_FOUND | 404 NOT_FOUND | ✅ PASS |
| 2 | Invalid UUID format | 400 BAD_REQUEST | 400 BAD_REQUEST | ✅ PASS |
| 3 | Created world timeline | 200 OK | 200 OK | ✅ PASS |

## Verification Steps Performed

1. **Reviewed code fix** - Confirmed existence check was added in `src/api/v1/worlds.rs` lines 605-611
2. **Built fixed container** - `world-factory:fixed` from updated Dockerfile
3. **Deployed container** - Restarted `test-run` with new image
4. **Ran smoke test** - All 3 tests passed

## Consistency Check

Confirmed the fix follows the same pattern used by other handlers:
- `get_world` (line 374)
- `update_world` (line 410)
- `get_world_events` (line 626)
- `get_world_figures` (line 729)
- `get_world_history` (line 1083)
- `get_world_artifacts` (line 2214)
- `get_world_resources` (line 2408)
- `get_world_disasters` (line 2449)

## Notes

- Backend restart required to apply fix (documented in WOR-946-FIX.md)
- Smoke test confirms fix works correctly
- Unit test added to prevent regression
- Code follows established patterns in codebase
