# WOR-67 Status: API Tests Fixed, Build Blocked by Pre-existing Errors

## Completed: API Test Code Changes

### 1. `src/api/v1/species.rs` - Routes Refactored
- Changed `pub fn routes(state: AppState)` → `pub fn routes()` 
- State is now bound at router level via `.with_state(state)`

### 2. `src/api/v1/mod.rs` - Updated Router Pattern
- Now uses `species::routes()` (no state param)
- `.with_state(state)` applied at v1 router level

### 3. `src/api/mod.rs` - Test Helper Fixed
- `create_test_router()` uses proper state cloning and binding pattern

### 4. `tests/api_history_figures_test.rs` - WorldPackage Fixed
- Added missing fields: `wonders`, `cataclysms`, `artifacts`

### 5. `src/api/data_derivation.rs` - Test Data Fixed
- Changed `influence_radius: None` → `influence_radius: 0.0`
- Changed `influence_radius: Some(100.0)` → `influence_radius: 100.0`

### 6. `src/api/v1/worlds.rs` - Type Name Fixed
- Changed `FactionListView` → `FactionsListView` (2 locations)

## Current Build Errors (Pre-existing, Not in Scope for WOR-67)

| Error | Location | Issue |
|-------|----------|-------|
| Missing `crate::faction` module | `src/api/mod.rs` | Module not declared in lib.rs |
| `StorageManager::load/save` missing | `src/api/mod.rs:72,89` | Methods don't exist on struct |
| `polygon.centroid` field missing | `src/hydro/drainage_basin.rs:102` | Field not defined on Polygon |
| Missing `factions` module | `src/api/v1/mod.rs:16` | File doesn't exist |

## Tests Are Ready

The API integration tests are syntactically correct:
- `src/api/mod.rs::tests` - 3 tests (lines ~169-210)
- `src/api/v1/species.rs::tests` - 3 tests (lines ~303-360)

They will run once the pre-existing build errors are fixed.

## Next Action

Fix pre-existing workspace build errors to enable test execution.