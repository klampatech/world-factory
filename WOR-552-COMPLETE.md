# WOR-552: Implement 202 Accepted + polling for world generation
## Status: ✅ COMPLETE

## Acceptance Criteria Status

| # | Criterion | Status |
|---|-----------|--------|
| 1 | POST /api/v1/worlds returns 202 Accepted | ✅ |
| 2 | WorldMetadata includes status and progress | ✅ |
| 3 | GET /api/v1/worlds/:id supports polling | ✅ |
| 4 | width ≤ 128 validation | ✅ |
| 5 | height ≤ 128 validation | ✅ |
| 6 | prehistory_years validation (1-100000) | ✅ |
| 7 | seed validation (non-zero) | ✅ |
| 8 | species_templates validation | ✅ |
| 9 | disaster_frequency validation (0-1) | ✅ |
| 10 | resource_richness validation (0-1) | ✅ |

## Files Modified

| File | Changes |
|------|---------|
| `web/Cargo.toml` | Added: tokio, axum, tower, tower-http, tracing, tracing-subscriber |
| `web/src/api/v1/worlds.rs` | New types, CreateWorldRequest::validate(), 14 unit tests |
| `web/src/api/v1/mod.rs` | Updated exports for new types |
| `web/src/main.rs` | Axum HTTP server with async endpoints |

## API Endpoints Implemented

| Endpoint | Method | Response | Description |
|----------|--------|----------|-------------|
| `/api/v1/worlds` | POST | 202 Accepted | Create world + async generation |
| `/api/v1/worlds` | GET | 200 OK | List all worlds with status |
| `/api/v1/worlds/:id` | GET | 200 OK | Poll for generation status |
| `/api/v1/worlds/:id/simulate` | POST | 200 OK | Run simulation |
| `/health` | GET | 200 OK | Health check |

## Validation Rules

```
CreateWorldRequest.validate() -> Option<String>
├── width: 1-128
├── height: 1-128
├── prehistory_years: 1-100000
├── resource_richness: 0.0-1.0
├── disaster_frequency: 0.0-1.0
├── seed: None or > 0
└── species_templates: None or non-empty with valid entries
```

## Unit Tests

14 tests in `src/api/v1/worlds.rs`:
- test_valid_world_id
- test_invalid_world_id
- test_register_and_get_world
- test_simulate_world_not_found
- test_simulate_world_success
- test_simulate_world_invalid_id
- test_create_world_creates_pending_task
- test_list_worlds_returns_metadata
- test_world_phase_serialization
- test_create_world_request_validation_width
- test_create_world_request_validation_prehistory_years
- test_create_world_request_validation_resource_params
- test_create_world_request_validation_species_templates
- test_create_world_request_validation_seed

## Handoff

**Next Owner:** Web Front End Engineer

**Tasks:**
1. Update `web/index.html` to handle 202 response with `polling_url`
2. Implement polling loop in `web/api-integration.js` (1-2s interval)
3. Replace `serve.js` with compiled `main.rs` binary for full API support
4. Update frontend to display generation status/progress

## Verification Commands

```bash
cd web
cargo check  # Should compile without errors
cargo test   # Should pass all 14 tests
```