# WOR-67: Fix AppState Integration Tests - Progress

## Changes Made (API Tests)

1. **src/api/v1/species.rs** - Fixed routes function:
   - `pub fn routes(state)` → `pub fn routes()` (no state param)
   - State is now bound via `.with_state(state)` at the v1 router level

2. **src/api/v1/mod.rs** - Updated v1 router:
   - `.nest("/species", species::routes(state))` → `.nest("/species", species::routes())`
   - Added `.with_state(state)` on the v1 Router to bind state

3. **src/api/mod.rs** - Updated test helper:
   - Uses proper state cloning: `v1::routes(state.clone())`
   - Applies `.with_state(state)` to the top-level router

4. **WorldPackage initializers** - Fixed missing fields in multiple locations:
   - Lines 319, 453, 2470 in `src/api/v1/worlds.rs`

5. **Natural Wonders imports** - Fixed `WonderView`/`WonderBonusView` in:
   - `src/terrain/natural_wonders/mod.rs`

## Build Status: BLOCKED

The workspace has **pre-existing compilation errors** that prevent running tests:

### Unfixed Pre-existing Issues (Not in Scope for WOR-67)

1. `src/terrain/biome.rs` - `BiomeType` enum has new variants but match statements aren't exhaustive
2. `src/api/models.rs` - `WondersQueryParams` missing `category` field (appears in use)
3. `src/terrain/natural_wonders/mod.rs` - `WonderBonusType::to_api_string()` and `WonderType::to_api_name()` don't exist
4. `src/hydro/drainage_basin.rs` - `polygon.centroid` field doesn't exist on `elevation::Polygon`
5. `src/api/data_derivation.rs` - Type mismatches with `influence_radius: Option<f32>` vs `f32`

### Root Cause

The codebase appears to have been partially updated with new features (natural wonders, factions, etc.) but:
- The `WorldPackage` struct was not updated to include the new fields
- The `Polygon` struct in elevation.rs was partially updated with `centroid` field but implementation incomplete
- The `WondersQueryParams` struct was updated with `category` but not fully integrated

## Test Files Are Ready

Once the build is fixed, the tests in the following files should work:
- `src/api/mod.rs` - 3 integration tests (lines ~95-140)
- `src/api/v1/species.rs` - 3 integration tests (lines ~295-360)

## Recommendations

1. Create a separate issue to fix the build blockers
2. OR: Revert the `centroid` field addition to `elevation::Polygon` since it's incomplete
3. OR: Add the missing `to_api_string`/`to_api_name` methods to the enums