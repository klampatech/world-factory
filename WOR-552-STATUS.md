# WOR-552: Implement 202 Accepted + polling for world generation

## Status: ✅ Implementation Complete

## Summary

Implemented 202 Accepted + polling for world generation per API spec §7.2. All acceptance criteria met.

## Acceptance Criteria ✅

| # | Criterion | Status |
|---|-----------|--------|
| 1 | POST /api/v1/worlds returns 202 Accepted | ✅ |
| 2 | WorldMetadata includes status and progress fields | ✅ |
| 3 | GET /api/v1/worlds/:id supports polling | ✅ |
| 4 | Config validation: width ≤ 128, height ≤ 128 | ✅ |
| 5 | Validate pre_history_years | ✅ |
| 6 | Validate seed | ✅ |
| 7 | Validate species_templates | ✅ |
| 8 | Validate disaster_frequency | ✅ |
| 9 | Validate resource_richness | ✅ |

## Implementation Details

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/worlds` | POST | Create world → 202 Accepted + async generation |
| `/api/v1/worlds` | GET | List all worlds with status |
| `/api/v1/worlds/:id` | GET | Poll for generation status |
| `/api/v1/worlds/:id/simulate` | POST | Run simulation on existing world |
| `/health` | GET | Health check |

### New Types (src/api/v1/worlds.rs)

```rust
// Generation tracking
pub struct GenerationTask { ... }
pub enum GenerationStatus { Pending, Generating, Complete, Failed }

// API request/response
pub struct CreateWorldRequest { 
    name, seed, width, height, prehistory_years,
    resource_richness, disaster_frequency,
    species_templates, detailed_events, generate_figures, generate_artifacts
}
pub struct CreateWorldResponse { id, name, status, message, polling_url }
pub struct WorldMetadata { id, name, status, progress, message, created_at }
pub enum WorldPhase { Idle, Generating, Ready, Error }
pub struct SpeciesTemplate { id, name, initial_population, society_type }
```

### Config Validation (CreateWorldRequest::validate)

| Field | Validation |
|-------|------------|
| `width` | 1-128 |
| `height` | 1-128 |
| `prehistory_years` | 1-100000 |
| `resource_richness` | 0.0-1.0 |
| `disaster_frequency` | 0.0-1.0 |
| `seed` | None or non-zero if provided |
| `species_templates` | None, or non-empty with valid id/name/population/society_type |

### API Flow

```
POST /api/v1/worlds 
  Content: { name: "My World", width: 64, height: 64, ... }
  ↓
  202 Accepted
  {
    "id": "550e8400-...",
    "name": "My World",
    "status": "generating",
    "message": "World generation started",
    "polling_url": "/api/v1/worlds/550e8400-..."
  }

GET /api/v1/worlds/550e8400-... (poll every 1-2s)
  → { "status": "generating", "progress": 50, "message": "Generating..." }
  → { "status": "ready", "progress": 100, "message": "World ready" }
```

### Files Changed

| File | Change |
|------|--------|
| `web/Cargo.toml` | Added: tokio, axum, tower, tower-http, tracing |
| `web/src/api/v1/worlds.rs` | New types, validation, functions |
| `web/src/api/v1/mod.rs` | Updated exports |
| `web/src/main.rs` | Axum HTTP server with async endpoints |

### Unit Tests Added

- `test_create_world_creates_pending_task`
- `test_list_worlds_returns_metadata`
- `test_world_phase_serialization`
- `test_create_world_request_validation_width`
- `test_create_world_request_validation_prehistory_years`
- `test_create_world_request_validation_resource_params`
- `test_create_world_request_validation_species_templates`
- `test_create_world_request_validation_seed`

## Next Steps

- [ ] Web Front End Engineer: Update frontend to handle 202 + polling
- [ ] Replace serve.js with compiled main.rs binary
- [ ] Integration testing with actual API calls