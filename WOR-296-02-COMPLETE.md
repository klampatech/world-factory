# WOR-296-02: API Route Spec Mismatch - COMPLETE

## Status: ✅ IMPLEMENTATION COMPLETE

## Problem

The smoke test specification (WOR-296) listed endpoints that did not match actual implementation:
- `DELETE /api/v1/worlds/:id` → 405 Not Implemented
- `GET /api/v1/worlds/:id/figures/:figure_id` → 404 Not Implemented

Additionally, documentation showed `/api/` prefix instead of `/api/v1/`.

## Solution

### Phase 1: Documentation Fix (Previous Heartbeat)
Updated all API endpoint documentation to use `/api/v1/` prefix:
- docs/SPEC.md
- docs/API_CONTRACT.md
- docs/CURRENT_STATUS.md
- src/ comments (terrain/MapData.ts, events/TimelineApiClient.ts, components/Dashboard.tsx)

### Phase 2: Implemented Missing Routes (This Heartbeat)

#### 1. DELETE /api/v1/worlds/:id
**File:** `src/api/v1/worlds.rs`

```rust
/// DELETE /api/v1/worlds/:id - Delete a world
async fn delete_world(
    State(state): State<crate::api::AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Validate UUID format
    let uuid_part = if id.starts_with("world:") { &id[6..] } else { &id };
    if !is_valid_uuid(uuid_part) {
        return Err(ApiError::BadRequest(...));
    }
    let storage_id = normalize_world_id(&id);
    if !state.storage.world_exists(&storage_id) {
        return Err(ApiError::NotFound(...));
    }
    state.storage.delete_world(&storage_id)
        .map_err(|e| ApiError::Internal(...))?;
    Ok(StatusCode::NO_CONTENT)
}
```

Route registration updated:
```rust
.route("/:id", get(get_world).delete(delete_world))
```

#### 2. GET /api/v1/worlds/:id/figures/:figure_id
**File:** `src/api/v1/worlds.rs`

```rust
/// GET /api/v1/worlds/:id/figures/:figure_id - Get a specific figure by ID
async fn get_world_figure(
    State(state): State<crate::api::AppState>,
    Path((world_id, figure_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<HistoricalFigure>>, ApiError> {
    // Validates world ID, checks existence, loads package
    let figure = package.notable_figures
        .iter()
        .find(|f| f.id.to_string() == figure_id)
        .map(HistoricalFigure::from)
        .ok_or_else(|| ApiError::NotFound(...))?;
    Ok(Json(ApiResponse::new(figure)))
}
```

Route added:
```rust
.route("/:id/figures/:figure_id", get(get_world_figure))
```

## Documentation Updates (Additional)

### docs/SPEC.md Section 7.1
Added new endpoints to endpoint list:
- `DELETE /api/v1/worlds/:id` 
- `GET /api/v1/worlds/:id/figures/:figure_id`

### docs/CURRENT_STATUS.md
Updated endpoint table:
- `DELETE /api/v1/worlds/:id` → `delete_world` | Done
- `GET /api/v1/worlds/:id/figures/:figure_id` → `get_world_figure` | Done

## Route Status Summary

| Route | Previous | Now |
|-------|----------|-----|
| `DELETE /api/v1/worlds/:id` | 405 | ✅ Implemented |
| `GET /api/v1/worlds/:id/figures/:figure_id` | 404 | ✅ Implemented |

## Route Path Differences (Documented as Aliases)

| Spec Expects | Actual Route | Status |
|--------------|--------------|--------|
| `/settlements` | `/societies` | ✅ 200 OK (semantic alias) |
| `/disasters` | `/cataclysms` | ✅ 200 OK (semantic alias) |
| `/history/events` | `/events` | ✅ 200 OK (semantic alias) |
| `/export.json` | `/export` | ✅ 200 OK (returns .wfw) |

*Completed: 2026-05-06*
