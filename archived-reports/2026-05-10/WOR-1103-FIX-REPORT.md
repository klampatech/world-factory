# WOR-1103: CLI World Persistence Fix

## Status: ✅ COMPLETE

## Summary

Implemented full world persistence for the CLI, including automatic `.wfw` saving, custom export directory, deterministic world IDs, and verified API integration.

## Changes Made

### `src/types.rs` - Deterministic World IDs

Added `EntityId::from_seed()` using UUID v5:

```rust
pub fn from_seed(seed: u64, entity_type: EntityType) -> Self {
    let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
    let input = format!("world-factory:{}", seed);
    let derived_uuid = Uuid::new_v5(&namespace, input.as_bytes());
    Self { id: derived_uuid, entity_type }
}
```

Updated `World::new()` to use deterministic IDs.

### `src/main.rs` - CLI Commands

1. **`generate`** - Extended with `--export-to <path>` flag
2. **`list`** - Lists all saved worlds
3. **`load <path>`** - Loads and displays a saved world
4. **`inspect <path>`** - Shows package metadata

### `Cargo.toml` - Dependency Update

Updated `uuid` crate to `1.11` for v5 feature support.

### `src/api/static_pages.rs` - API Fixes

- Added missing `routing::get` import
- Changed route syntax from `:id` to `{id}` (axum 0.8 format)
- Added `AppState` type parameter to `Router`

### `src/api/mod.rs` - API Fixes

- Changed `.nest("/", ...)` to `.merge(...)` for static pages

## Acceptance Criteria Verification

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1 | `generate` saves `.wfw` file | ✅ PASS | `world:e0970182.../world.wfw` exists |
| 2 | `.wfw` contains valid `world.json` | ✅ PASS | `inspect` shows valid JSON |
| 3 | `--export-to` saves to custom dir | ✅ PASS | Saves to `/tmp/test-worlds/` |
| 4 | Deterministic IDs (same seed) | ✅ PASS | Seed 42 → same ID twice |
| 5 | Server lists CLI world | ✅ PASS | 408 worlds, includes World-42 ID |
| 6 | Round-trip load works | ✅ PASS | `load` displays world data |
| 7 | `cargo test --lib` passes | ✅ PASS | 443 tests passed |

## Test Evidence

### Criterion #5 - API Listing Verified
```
$ curl localhost:3008/api/v1/worlds
{"success":true,"data":{
  "totalWorlds": 408,
  "worlds": [
    {"id": "world:e0970182-c74d-5045-bbf4-af5259ca988d", ...},
    ...
  ]
}}
```
World-42 (seed 42) produces ID `world:e0970182-c74d-5045-bbf4-af5259ca988d` and is listed in API.

### Deterministic IDs
```
$ world_generator generate --seed 42 --width 16 --height 16
World saved to: .../world:e0970182-c74d-5045-bbf4-af5259ca988d/world.wfw

$ world_generator generate --seed 42 --width 16 --height 16
World saved to: .../world:e0970182-c74d-5045-bbf4-af5259ca988d/world.wfw
```

### Round-trip Load
```
$ world_generator load world:e0970182-c74d-5045-bbf4-af5259ca988d
Name: World-42
ID: world:e0970182-c74d-5045-bbf4-af5259ca988d
Seed: 42
Created: 2026-05-10T20:51:40.871066024+00:00
```

## API Compilation Fixes (Prerequisite for Criterion #5)

Fixed pre-existing API compilation errors:
1. Missing `routing::get` import in `static_pages.rs`
2. Route syntax updated from `:id` to `{id}` (axum 0.8 format)
3. Router type mismatch - `static_pages::routes()` now returns `Router<AppState>`
4. Changed `.nest("/", ...)` to `.merge(...)` for compatible router types

These fixes are separate from the CLI persistence feature but required to verify criterion #5.
