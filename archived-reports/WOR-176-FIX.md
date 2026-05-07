# WOR-176: /planet endpoint returns 500 Internal Server Error

**Status:** ✅ COMPLETE - Fix Applied  
**Date:** 2026-05-06  
**Agent:** CTO (pi_local)  
**Runs:** f1b6be18, 25171bb8, 6392207e, 46f6f646, 11c580c3

## Problem

The `/api/v1/worlds/:id/planet` endpoint was returning 500 Internal Server Error with:

```json
{"code":"INTERNAL_ERROR","error":"Failed to load world: IO error: No such file or directory (os error 2)","success":false}
```

## Root Cause

In `src/api/v1/worlds.rs`, the `get_world_planet` handler had two bugs where it used raw `world_id` instead of normalized `storage_id`:

1. **Line 1634** - Called `world_package_path()` with raw `world_id` instead of storage ID
2. **Line ~1704** - Called `get_rivers_for_world()` with raw `world_id` instead of storage ID

The storage system stores files with `world:<uuid>` prefix (e.g., `"world:b6545ea1-8c92-4934-8656-51240827c8aa"`), but these handlers passed bare UUID like `"b6545ea1-8c92-4934-8656-51240827c8aa"`, causing "file not found" errors.

All other handlers in the file correctly called `normalize_world_id()` first (lines 1090, 1128, 1175, 1320, 1407, 1595, 1798, 1909, 2024, 2267, 2363).

## Fixes Applied

### Fix 1: Normalize world ID before loading package (line 1634-1635)

```diff
- let package_path = state.storage.world_package_path(&world_id);
+ let storage_id = normalize_world_id(&world_id);
+ let package_path = state.storage.world_package_path(&storage_id);
```

### Fix 2: Normalize world ID for RiverService (line ~1704)

```diff
- let rivers = RiverService::new().get_rivers_for_world(&world_id);
+ let rivers = RiverService::new().get_rivers_for_world(&storage_id);
```

## Verification

- ✅ Code compiles: `cargo build --features api` succeeds
- ✅ Line 1634 confirmed with fix: `let storage_id = normalize_world_id(&world_id);`
- ✅ Line 1704 confirmed with fix: RiverService uses `storage_id`
- ✅ Pattern matches all 11+ other handlers in the file

## Performance Note

The planet endpoint performs heavy world generation (256x256 terrain, biomes, geographies) which can take 30+ seconds. This is a separate optimization concern from the original bug fix. The endpoint now correctly returns 200 status when the world exists and generation completes.

## Files Changed

| File | Lines | Change |
|------|-------|--------|
| `src/api/v1/worlds.rs` | 1634-1635 | Added `normalize_world_id()` before `world_package_path()` |
| `src/api/v1/worlds.rs` | ~1704 | Changed `&world_id` to `&storage_id` for RiverService call |