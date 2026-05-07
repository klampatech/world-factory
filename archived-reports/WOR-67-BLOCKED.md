## WOR-67 Status: BLOCKED - Build Has Pre-existing Errors

### Summary of API Test Changes Made

1. **src/api/v1/species.rs** (lines ~18-21, ~295-300):
   - Changed `pub fn routes(state)` → `pub fn routes()` (no state param)
   - State is now bound at the router level with `.with_state(state)`
   - This fixes the `ServiceExt::oneshot` trait bound issue

2. **src/api/v1/mod.rs** (lines ~22-26):
   - Updated to use `species::routes()` and added `.with_state(state)` to Router

3. **src/api/mod.rs** (lines ~95-140):
   - Updated `create_test_router()` helper for proper state cloning

4. **src/api/v1/worlds.rs** (lines 319, 453, 2470):
   - Fixed WorldPackage initializers to match current struct definition

5. **src/terrain/natural_wonders/mod.rs** (lines ~53-93):
   - Fixed `WonderView`/`WonderBonusView` imports from `crate::api::models`

### Build Status: BLOCKED BY PRE-EXISTING ERRORS

The workspace has **unrelated compilation errors** that prevent running tests:

1. `BiomeType` enum has new variants but match statements not exhaustive (multiple files)
2. `FactionTurnState` missing `beast_bonds` field (src/faction.rs)
3. Type mismatches in data_derivation.rs
4. `elevation::Polygon` has incomplete `centroid` field

### Test Files Status

The API test files are ready to compile/run once build is fixed:
- `src/api/mod.rs` tests: lines ~169-210 (3 tests)
- `src/api/v1/species.rs` tests: lines ~303-360 (3 tests)

### Recommendation

Create a child issue "Fix build errors for WOR-67 tests" to address the pre-existing compilation errors, then resume WOR-67 once build is clean.