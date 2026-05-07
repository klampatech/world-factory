# QA Report: WOR-342 Backend API build fails with --features api

**Date:** 2026-05-07  
**Issue:** [WOR-342](/WOR/issues/WOR-342)  
**Status:** ❌ FAIL  
**Tester:** QA Agent  

## Summary

Build with `--features api` fails with 46 compilation errors. The core issue is that API handler files (`src/api/v1/factions.rs`, `src/api/v1/worlds.rs`) are **not gated** behind `#[cfg(feature = "api")]`, so they're compiled regardless of the feature flag. When the feature is disabled, required types are missing.

## Reproduction Steps

```bash
docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo build --features api
```

## Root Cause Analysis

The API v1 routes (`src/api/v1/factions.rs`, `src/api/v1/worlds.rs`) are always compiled because they are not wrapped in `#[cfg(feature = "api")]`. When the API feature is enabled:
- `serde::{Serialize, Deserialize}` and `uuid::Uuid` become available globally
- But the files reference these types without the feature guard

## Errors Found (46 total)

### Category 1: Missing Types (outside #[cfg] gates)
| File | Line | Missing Type |
|------|------|--------------|
| factions.rs | 68, 69 | `FactionSummaryView` |
| factions.rs | 186, 211 | `FactionTurnStateView` |
| factions.rs | 221, 253 | `TurnAdvanceResponse` |
| factions.rs | 291, 336 | `FactionAssetView` |

### Category 2: Missing Route Handlers (worlds.rs)
| Function | Route |
|----------|-------|
| `get_world_resources_summary` | `/:id/resources/summary` |
| `get_world_settlements` | `/:id/settlements` |
| `get_world_settlements_map` | `/:id/settlements/map` |
| `get_world_export` | `/:id/export` |
| `get_world_export_json` | `/:id/export.json` |

### Category 3: Struct Field Mismatches
`DiplomaticRelationView` struct uses non-existent fields:
- `target_faction_id` → should be `target_id`
- `target_faction_name` → should be `target_name`
- `relation_type` → not available
- `started_year` → should be `established_year`
- `is_active` → not available

### Category 4: Missing Methods
- `AppState::save_faction_registry()` (lines 270, 340)
- `ApiResponse::success()` (line 178) - use `ApiResponse::new()` instead

### Category 5: Borrow After Move
- `worlds.rs:1711-1742`: `all_disasters` moved then borrowed

## Verdict

**FAIL** - Build cannot complete with `--features api`. The API feature implementation is incomplete.

## Recommended Fixes (Priority Order)

1. **Add missing view types** to `src/api/models.rs`:
   - `FactionSummaryView`
   - `FactionTurnStateView`
   - `TurnAdvanceResponse`
   - `FactionAssetView`

2. **Implement missing route handlers** in `src/api/v1/worlds.rs`:
   - `get_world_resources_summary`
   - `get_world_settlements`
   - `get_world_settlements_map`
   - `get_world_export`
   - `get_world_export_json`

3. **Fix `DiplomaticRelationView` construction** to use available struct fields only

4. **Add `save_faction_registry` method** to `AppState` or remove usage from handlers

5. **Replace `ApiResponse::success()` with `ApiResponse::new()`** at line 178

6. **Fix borrow issue** in worlds.rs: clone `all_disasters` before `into_iter()`

## Next Steps

Issue should be returned to the Coder with these specific fix instructions. Full error output available on request.
