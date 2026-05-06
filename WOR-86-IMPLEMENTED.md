# WOR-86: GET /api/v1/worlds/:id/wonders endpoint

## Status: in_progress

## Summary

Fixed the natural wonders endpoint to load actual wonder data from world packages instead of always returning mock data.

## Changes Made

### 1. packaging.rs
Added `wonders` field to `WorldPackage` struct to store generated wonders:
```rust
pub wonders: Vec<crate::terrain::natural_wonders::NaturalWonder>,
```

### 2. natural_wonders/wonder_types.rs
Added `to_api_name()` method to `WonderType`:
- Converts enum variants to PascalCase API strings (e.g., `SacredMountain`)

Added `to_api_string()` method to `WonderCategory`:
- Converts to lowercase strings (e.g., `geological`)

### 3. natural_wonders/mod.rs
Added `to_api_string()` method to `WonderIconType`:
- Converts to camelCase strings (e.g., `ancientTree`)

Added `From<&NaturalWonder>` for `WonderView`:
- Converts domain `NaturalWonder` to API `WonderView`
- Uses the new conversion methods for proper enum-to-string mapping

### 4. natural_wonders/wonder_effects.rs
Added `to_api_string()` method to `WonderBonusType`:
- Converts to lowercase strings (e.g., `food`, `production`)

### 5. api/v1/worlds.rs
Updated `get_world_wonders` endpoint:
- Loads wonders from `package.wonders` (proper field)
- Uses `apply_wonder_filters()` from data_derivation module
- Computes real `derive_wonder_stats()` from actual wonder data
- Falls back to mock data only for non-existent worlds

Simplified `generate_mock_wonders()`:
- Removed unused filter parameters (filters now applied via data_derivation module)

## Notes

- The endpoint now properly loads actual generated wonders from world packages
- Filter functionality is delegated to the existing `apply_wonder_filters()` in data_derivation
- Stats computation is now done via `derive_wonder_stats()` instead of hardcoded values
- Mock data is only used as fallback for non-existent worlds (demo/dev mode)

## Testing Needed

- Verify endpoint returns actual wonders for existing worlds
- Verify filtering works correctly with category and wonder_type params
- Verify stats are computed correctly from actual data
