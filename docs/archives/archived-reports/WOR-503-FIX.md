# WOR-503 Fix Verification

## Issue: Rebuild and restart server binary to fix 404 errors

### Root Cause
The API handlers received world IDs with the `world:` prefix (e.g., `world:abc-123`) from the URL path, but the storage layer expects raw UUIDs (e.g., `abc-123`). This mismatch caused 404 errors on storage lookups.

### Fix Applied
Added `normalize_world_id()` calls to all 20 API handlers in `src/api/v1/worlds.rs` and 4 handlers in `src/api/v1/artifacts.rs` and `src/api/v1/cataclysms.rs`.

### Files Modified
1. `src/api/v1/worlds.rs` - 18 handlers updated
2. `src/api/v1/artifacts.rs` - 2 handlers updated
3. `src/api/v1/cataclysms.rs` - 2 handlers updated

### Pattern Applied
```rust
// Before:
Path(world_id): Path<String>
...
uuid::Uuid::parse_str(&world_id)...

// After:
Path(world_id_raw): Path<String>
...
let world_id = crate::api::normalize_world_id(&world_id_raw);
uuid::Uuid::parse_str(&world_id)...
```

### Handlers Fixed (23 total)

| Handler | File | Line |
|---------|------|------|
| get_world | worlds.rs | 361 |
| get_world_map | worlds.rs | 420 |
| get_world_timeline | worlds.rs | 570 |
| get_world_events | worlds.rs | 589 |
| get_world_history | worlds.rs | 622 |
| get_world_figures | worlds.rs | 685 |
| get_world_societies | worlds.rs | 717 |
| get_world_planet | worlds.rs | 945 |
| get_world_tectonics | worlds.rs | 1032 |
| get_world_artifacts | worlds.rs | 1119 |
| get_world_wonders | worlds.rs | 1240 |
| get_world_cataclysms | worlds.rs | 1522 |
| get_world_resources | worlds.rs | 1601 |
| get_world_disasters | worlds.rs | 1745 |
| get_world_resources_summary | worlds.rs | 2017 |
| get_world_settlements | worlds.rs | 2038 |
| get_world_settlements_map | worlds.rs | 2056 |
| get_world_export | worlds.rs | 2084 |
| get_world_export_json | worlds.rs | 2106 |
| get_artifacts | artifacts.rs | 59 |
| get_artifact | artifacts.rs | 238 |
| get_cataclysms | cataclysms.rs | 65 |
| get_cataclysm | cataclysms.rs | 295 |

### Next Step
Rebuild the server binary and restart to apply the changes. Then run smoke tests to verify 404 errors are resolved.