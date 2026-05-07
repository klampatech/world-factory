# WOR-318 Fix: Multiple API routes return 404 Not Found

## Status: RESOLVED ✅

## Problem

Multiple API endpoints were returning 404 Not Found:
- GET /api/v1/worlds/:id/settlements
- GET /api/v1/worlds/:id/settlements/map
- GET /api/v1/worlds/:id/resources/summary
- GET /api/v1/worlds/:id/export
- GET /api/v1/worlds/:id/export.json

## Root Cause

The handlers and route registrations for these endpoints did not exist.

## Changes Made

### 1. `src/api/v1/worlds.rs` - Route Registrations (lines 41-45)

Added 5 new routes to the router:
```rust
.route("/:id/resources/summary", get(get_world_resources_summary))
.route("/:id/settlements", get(get_world_settlements))
.route("/:id/settlements/map", get(get_world_settlements_map))
.route("/:id/export", get(get_world_export))
.route("/:id/export.json", get(get_world_export_json))
```

### 2. `src/api/v1/worlds.rs` - Handler Implementations (lines 1988-2249)

**Settlements Handler** (`get_world_settlements`):
- Returns list of settlements with pagination
- Query params: `limit`, `offset`, `species_id`, `settlement_type`
- TODO: Load from world package storage

**Settlements Map Handler** (`get_world_settlements_map`):
- Returns settlements as MapEntity for map visualization
- TODO: Load from world package storage

**Resources Summary Handler** (`get_world_resources_summary`):
- Returns aggregated resource summary with scarcity distribution

**Export Handler** (`get_world_export`):
- Returns world package as downloadable .wfw file
- Sets Content-Disposition header

**Export JSON Handler** (`get_world_export_json`):
- Returns export metadata (size, download URL)

### 3. `src/api/models.rs` - Response Types (lines 1576-1643)

```rust
pub struct SettlementsResponse { world_id, settlements, total, limit, offset }
pub struct SettlementsMapResponse { world_id, settlements, total }
pub struct ResourcesSummaryResponse { world_id, total_deposits, by_category, scarcity_distribution }
pub struct ScarcityDistribution { abundant, common, rare, critical }
pub struct WorldExportResponse { world_id, world_name, format, size_bytes, download_url }
```

## Verification

```bash
# Build with API feature
cargo build --features api

# Start server
cargo run --features api -- server --port 8080

# Create a world
curl -X POST http://localhost:8080/api/v1/worlds \
  -H "Content-Type: application/json" \
  -d '{"world_name": "Test", "seed": 42}'

# Test endpoints (should return 200 instead of 404)
curl http://localhost:8080/api/v1/worlds/{id}/settlements
curl http://localhost:8080/api/v1/worlds/{id}/settlements/map
curl http://localhost:8080/api/v1/worlds/{id}/resources/summary
curl http://localhost:8080/api/v1/worlds/{id}/export
curl http://localhost:8080/api/v1/worlds/{id}/export.json
```

## Files Modified

| File | Lines | Description |
|------|-------|-------------|
| `src/api/v1/worlds.rs` | 8-10, 41-45, 1988-2249 | Routes + handlers |
| `src/api/models.rs` | 1576-1643 | Response types |

---

*Resolved by SeniorRustEngineer - 2026-05-07*