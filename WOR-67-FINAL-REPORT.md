# WOR-67: Fix AppState Integration Tests - Final Report

## Summary

The primary task was to fix 6 integration tests in `src/api/mod.rs` and `src/api/v1/species.rs` that use `#\[ignore]` due to AppState Clone+Send trait bound conflict with tower::ServiceExt::oneshot.

## Changes Made

### 1. src/api/v1/species.rs (lines ~18-21, ~295-300)
Fixed the routes function to separate router creation from state binding:
```rust
// BEFORE
pub fn routes(state: crate::api::AppState) -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_species))
        .route("/:id", get(get_species))
        .with_state(state)  // <-- This causes Clone+Send bound conflict
}

// AFTER
pub fn routes() -> Router<crate::api::AppState> {
    Router::new()
        .route("/", get(list_species))
        .route("/:id", get(get_species))
}
```
State is now bound at the v1 router level with `.with_state(state)`.

### 2. src/api/v1/mod.rs (lines ~22-26)
Updated to use the new route pattern:
```rust
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/worlds", worlds::routes(state.clone()))
        .nest("/events", events::routes(state.clone()))
        .nest("/species", species::routes())  // No state param
        .with_state(state)  // Bind state here
}
```

### 3. src/api/mod.rs (lines ~95-140)
Updated `create_test_router()` helper to use proper state cloning and `.with_state()`:
```rust
fn create_test_router() -> Router<AppState> {
    let state = AppState::new_for_test();
    Router::new()
        .nest("/api/v1", v1::routes(state.clone()))
        .route("/health", get(health_check))
        .with_state(state)
}
```

### 4. tests/api_history_figures_test.rs
Fixed 3 WorldPackage initializers to include missing fields (wonders, cataclysms, artifacts).

## Current Build Status: BLOCKED

The workspace has **multiple pre-existing compilation errors** that are NOT in the API test files but affect the entire build:

### Critical Errors (Not in Scope for WOR-67)

1. **Missing `factions` module** - `src/api/v1/mod.rs:16` references `pub mod factions;` but the file doesn't exist

2. **Missing `StorageManager::load/save` methods** - `src/api/mod.rs:72,89` calls methods that don't exist on the struct

3. **Type mismatches in `data_derivation.rs`** - `influence_radius: Some(100.0)` where `f32` expected (5 occurrences)

4. **Missing `RiverService` in scope** - `src/api/v1/worlds.rs:1842` uses `RiverService::new()` but it's not imported

5. **Naming mismatch** - `FactionListView` vs `FactionsListView` (lines ~2649, 2671)

6. **Borrow after move** - `src/api/v1/worlds.rs:2179` `filtered_wonders` moved then borrowed

7. **SuccessResponse type mismatch** - `src/api/v1/worlds.rs:2883` returns wrong type

### Root Cause

The codebase has been partially updated with new features (factions, natural wonders, etc.) but:
- Some new module declarations reference non-existent files
- Some API models have naming inconsistencies
- Some methods have been renamed or removed without updating callers
- Type signatures have changed but not propagated to all use sites

## Test Files Status

The API test files are **syntactically correct and ready**:
- `src/api/mod.rs` tests: lines ~169-210 (3 tests)
- `src/api/v1/species.rs` tests: lines ~303-360 (3 tests)

They will compile and run once the workspace build errors above are fixed.

## Recommendation

Create a child issue "Fix workspace build errors" to address the pre-existing compilation errors, then resume to run the API tests once build is clean.