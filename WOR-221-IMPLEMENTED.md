# WOR-221: API Endpoint Tests Implementation

## Status: Implementation Complete

Tests created for 18 handlers covering both happy paths and error paths.

## Changes Made

### 1. `src/api/mod.rs` - AppState Fix
- Wrapped `StorageManager` in `Arc<>` to enable `Clone` for Axum router
- Added `get_faction_registry()` method for faction endpoints

### 2. `src/storage.rs` - Storage Manager Enhancement
- Added `factions_path()` method to return path for faction registry TOML files

### 3. `src/faction.rs` - FactionRegistry Persistence
- Added `load()` static method to load registry from TOML file
- Added `save()` method to persist registry to TOML file
- Returns empty registry if file doesn't exist (graceful fallback)

### 4. `tests/api_endpoints_test.rs` - Test Suite (New File)
Comprehensive tests covering:

| # | Endpoint | Tests |
|---|----------|-------|
| 1 | GET /health | Returns 200 with correct JSON structure |
| 2 | GET /api/v1/worlds | List worlds, pagination, invalid sort params (400) |
| 3 | POST /api/v1/worlds | Create world (201), empty name (400), name too long (400) |
| 4 | GET /api/v1/worlds/:id | Non-existent (404), invalid UUID (400) |
| 5 | POST /api/v1/worlds/:id/generate | Invalid UUID (400), not found (404) |
| 6 | GET /api/v1/worlds/:id/map | Invalid UUID (400), with params (404) |
| 7 | GET /api/v1/worlds/:id/timeline | Invalid UUID (400) |
| 8 | GET /api/v1/worlds/:id/events | Invalid UUID (400), with pagination (404) |
| 9 | GET /api/v1/worlds/:id/history | Invalid UUID (400) |
| 10 | GET /api/v1/worlds/:id/figures | Invalid UUID (400), with filters (404) |
| 11 | GET /api/v1/worlds/:id/societies | Invalid UUID (400) |
| 12 | GET /api/v1/worlds/:id/planet | Invalid UUID (400) |
| 13 | GET /api/v1/worlds/:id/artifacts | Invalid UUID (400), with filters (404) |
| 14 | GET /api/v1/worlds/:id/cataclysms | Invalid UUID (400), with filters (404) |
| 15 | GET /api/v1/worlds/:id/wonders | Invalid UUID (400), with type filter (404) |
| 16 | GET /api/v1/worlds/:id/resources | Invalid UUID (400), with type filter (404) |
| 17 | GET /api/v1/worlds/:id/disasters | Invalid UUID (400), with status filter (404) |
| 18 | GET /api/v1/species | Returns 200, with habitat filter, with trait filter |

**Additional Tests:**
- Response structure verification (success/error format)
- Error path tests for species, events, factions endpoints
- Concurrency test (10 simultaneous requests)

## Test Execution

Run with Docker:
```bash
# Build test image
docker build -f Dockerfile.test -t world-factory:test .

# Run tests
docker run --rm -v $(pwd):/workspace -w /workspace world-factory:test \
  cargo test --features api tests/api_endpoints_test.rs -- --nocapture
```

Or locally (if Rust is installed):
```bash
cargo test --features api tests/api_endpoints_test.rs -- --nocapture
```

## Notes

- All handlers that take UUID parameters validate the format and return 400 for invalid UUIDs
- Handlers that reference non-existent resources return 404
- The `#[cfg(feature = "api")]` conditional compilation ensures tests only run when API feature is enabled
- AppState uses Arc<StorageManager> to enable concurrent request handling (Axum router can be cloned)