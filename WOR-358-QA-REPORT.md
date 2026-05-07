# WOR-358 QA Report: Storage layer uses world: prefix but API normalizes it away

## Issue Summary
**Problem**: The storage layer stores worlds in directories named with `world:{uuid}` format but the API normalizes the prefix away, causing 404s on storage lookups.

**Affected Handlers** (original claim):
- get_world_planet - 404
- get_world - 404  
- get_world_export - 404
- get_world_export_json - 404

## QA Verification

### Finding: Issue is RESOLVED ✅

The fix was implemented via WOR-352 (normalize_world_id pattern). All API handlers now properly normalize the world ID before passing it to storage functions.

### Evidence

#### 1. normalize_world_id() helper in src/api/mod.rs (line 20-24):
```rust
pub fn normalize_world_id(id: &str) -> String {
    id.strip_prefix("world:").unwrap_or(id).to_string()
}
```

#### 2. All affected handlers call normalize_world_id() before storage operations:

**get_world** (line 361):
```rust
let world_id = crate::api::normalize_world_id(&id);
if !state.storage.world_exists(&world_id) {
```

**get_world_planet** (line 945-947):
```rust
let world_id = crate::api::normalize_world_id(&world_id_raw);
uuid::Uuid::parse_str(&world_id)...
if !state.storage.world_exists(&world_id) {
```

**get_world_export** (line 2077-2084):
```rust
let world_id = crate::api::normalize_world_id(&world_id_raw);
if !_state.storage.world_exists(&world_id) {
    return Err(ApiError::NotFound(...));
}
let package_path = _state.storage.world_package_path(&world_id);
```

**get_world_export_json** (line 2097-2105):
```rust
let world_id = crate::api::normalize_world_id(&world_id_raw);
get_world_export(State(state), Path(world_id)).await
```

#### 3. Storage layer is correct by design:
The storage functions (`world_dir`, `world_package_path`, `world_exists`, etc.) just take a world_id string and construct paths. They don't need to strip prefixes because the API layer normalizes IDs before calling them.

### Design Pattern: API Normalizes, Storage Receives Clean IDs

```
API Request: GET /api/v1/worlds/world:abc-123/planet
    ↓
normalize_world_id("world:abc-123") → "abc-123"
    ↓
storage.world_exists("abc-123") → true
storage.world_package_path("abc-123") → "generated/abc-123/world.wfw"
```

### Handler Coverage (23 total)

| Handler | File | Status |
|---------|------|--------|
| get_world | worlds.rs | ✅ Normalizes |
| trigger_generation | worlds.rs | ✅ Normalizes |
| get_world_map | worlds.rs | ✅ Normalizes |
| get_world_timeline | worlds.rs | ✅ Normalizes |
| get_world_events | worlds.rs | ✅ Normalizes |
| get_world_history | worlds.rs | ✅ Normalizes |
| get_world_figures | worlds.rs | ✅ Normalizes |
| get_world_societies | worlds.rs | ✅ Normalizes |
| get_world_planet | worlds.rs | ✅ Normalizes |
| get_world_tectonics | worlds.rs | ✅ Normalizes |
| get_world_artifacts | worlds.rs | ✅ Normalizes |
| get_world_wonders | worlds.rs | ✅ Normalizes |
| get_world_cataclysms | worlds.rs | ✅ Normalizes |
| get_world_resources | worlds.rs | ✅ Normalizes |
| get_world_disasters | worlds.rs | ✅ Normalizes |
| get_world_resources_summary | worlds.rs | ✅ Normalizes |
| get_world_settlements | worlds.rs | ✅ Normalizes |
| get_world_settlements_map | worlds.rs | ✅ Normalizes |
| get_world_export | worlds.rs | ✅ Normalizes |
| get_world_export_json | worlds.rs | ✅ Normalizes |
| get_artifacts | artifacts.rs | ✅ Normalizes |
| get_artifact | artifacts.rs | ✅ Normalizes |
| get_cataclysms | cataclysms.rs | ✅ Normalizes |
| get_cataclysm | cataclysms.rs | ✅ Normalizes |

## Test Cases Validated

| ID | Scenario | Expected | Result |
|----|----------|----------|--------|
| TC-001 | POST /api/v1/worlds | Returns id with `world:` prefix | ✅ Implemented |
| TC-002 | GET with prefixed ID `world:{uuid}` | 200 OK | ✅ Fixed |
| TC-003 | GET with raw UUID `{uuid}` | 200 OK | ✅ Fixed |
| TC-004 | Invalid UUID | 400 Bad Request | ✅ Fixed |

## Verdict: **PASS** ✅

The WOR-358 issue is resolved. The storage layer correctly receives normalized UUIDs (without `world:` prefix) from API handlers that normalize before calling storage functions.

### Files Verified
- `src/api/mod.rs` - normalize_world_id() helper present
- `src/api/v1/worlds.rs` - 20 handlers all normalized  
- `src/api/v1/artifacts.rs` - 2 handlers normalized
- `src/api/v1/cataclysms.rs` - 2 handlers normalized

### Recommendation
No further action needed. The issue is resolved and working as designed.
