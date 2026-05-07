# WOR-308 WOR-305 BUG: API build fails with faction module not found

## Status: RESOLVED ✅

## Root Cause

The faction API endpoints (`src/api/v1/factions.rs`) were using type names that didn't exist in the models module:
- `FactionSummaryView` → should be `FactionView`
- `FactionTurnStateView` → created new `TurnStateView` 
- `TurnAdvanceResponse` → created new response type
- `FactionAssetView` → created new `AssetView`
- `ApiResponse::success()` → doesn't exist, use `ApiResponse::new()`
- `save_faction_registry()` → method didn't exist in `AppState`

Additionally, `DiplomaticRelationView` had different field names than expected.

## Changes Made

### 1. `src/api/v1/factions.rs`
- Fixed imports: removed unused `FactionRegistry` from imports
- Changed `FactionSummaryView::from` to `FactionView::from_faction`
- Removed `.with_world_id()` call (doesn't exist on `FactionsListView`)
- Fixed `FactionDetailView::from(faction)` → `FactionDetailView::from_faction(faction)`
- Fixed `DiplomaticRelationView` construction to match actual struct fields
- Changed `ApiResponse::success(relations)` → `ApiResponse::new(relations)`
- Fixed `FactionTurnStateView` → `TurnStateView`
- Fixed `FactionAssetView` → `AssetView`
- Fixed `save_faction_registry(world_id, registry)` → `save_faction_registry(&registry, world_id)` (correct parameter order)

### 2. `src/api/models.rs`
Added missing types:
- `TurnStateView` - for faction turn state API responses
- `AssetView` - for faction asset API responses
- `TurnAdvanceResponse` - for turn advance endpoint response

### 3. `src/api/mod.rs`
Added missing method `save_faction_registry()` to `AppState`

### 4. `src/api/v1/worlds.rs`
Fixed borrow-after-move error in disasters endpoint by cloning filtered vector before iteration.

## Verification

```bash
docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo build --features api
# → Finished successfully (45 warnings)

docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo check --features api
# → Finished successfully
```

## Remaining Warnings

The build has 45 warnings (mostly pre-existing unreachable patterns and unused code in other modules). These are not related to the faction module fix.

## Note on Test Failures

Pre-existing test failures in `tests/voronoi_property_tests.rs` use outdated `BiomeType` names (e.g., `Savanna`, `Forest`). These are unrelated to WOR-308/305 and should be addressed separately.

## Resolved By
SeniorRustEngineer - 2026-05-06