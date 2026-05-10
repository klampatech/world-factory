# World Factory - Implementation Status

> Generated: May 10, 2026
> Branch: `phase-1` (9d00917)
> Test Status: 435 lib tests passing, **8 FAILED** (regression)
> Smoke Test Status: Previously 26/26 PASS (WOR-934, WOR-925)

---

## Executive Summary

World Factory is a procedural world & history generation system written in Rust (~36K LOC).
Implementation is **substantially complete** through Phase 3 (Persistence & API), Phase 4
(Visualization) partially done, and **Phase 5 spec'd but implementation has regressions**.

**Critical:** 8 test failures in `beasts::slaying` and `faction::faction_stats_tests` indicate
a recent regression in the faction/beasts systems. These must be fixed before Phase 5 can be
considered functional.

---

## Phase 1: Core World Generation - COMPLETE (with regression)

|| Feature | Module | Status | Notes ||
|---------|--------|--------|-------|-------|
| Voronoi + Lloyd relaxation | `src/world/generation/lloyd_relaxation.rs` | Done | |
| Elevation & tectonics | `src/terrain/elevation.rs`, `src/terrain/tectonic/` | Done | |
| River generation | `src/hydro/`, `src/terrain/erosion.rs` | Done | |
| Erosion simulation | `src/terrain/erosion.rs` | Done | |
| Climate zones | `src/terrain/climate_calculator.rs` | Done | |
| Biome assignment | `src/terrain/biome_assignment.rs` | Done | |
| Resource spawning | `src/terrain/resource_spawner.rs` (817 LOC) | Done | |
| Natural wonders | `src/terrain/natural_wonders/` | Done | |
| Primal beasts | `src/beasts/` (profiles, movement, effects, slaying, remnants) | **Done but regressed** | 4 beasts specced (Pyraxes, Tidarth, Terros, Lumina); all implemented but 5 slaying/remnant tests failing |
| CLI world persistence | `src/main.rs` | **NOT DONE** - `generate` command does not save `.wfw` to storage | |

---

## Phase 2: History Generation - COMPLETE

|| Feature | Module | Status | Notes ||
|---------|--------|--------|-------|-------|
| Species templates (YAML/JSON) | `src/species/`, `src/history/` | Done | |
| Civilization emergence | `src/history/` | Done | |
| Settlement spawning | `src/settlements/mod.rs` | Done | |
| Population growth model | `src/simulation/population.rs` | Done | |
| Event generation engine | `src/events/` | Done | |
| Notable figures | `src/figures.rs` (71KB) | Done | |
| Historical artifacts | `src/artifacts.rs` (48KB) | Done | |
| Cataclysmic events | `src/cataclysms.rs` (20KB) | Done | |

**Spec additions since last review (SPEC.md D.3-D.6):**
- Artifact placement rules: causal chains, prerequisites per artifact type
- Faction territory rules: clustered centers, ocean exclusion, age scaling
- Primal beasts & spirits: four elemental beasts with world effects, faction interactions, death consequences

---

## Phase 3: Persistence & API - MOSTLY COMPLETE

|| Endpoint | Handler | Status ||
|----------|---------|--------|
| `POST /api/v1/worlds` | `create_world` | Done |
| `GET /api/v1/worlds` | `list_worlds` | Done |
| `GET /api/v1/worlds/:id` | `get_world` | Done |
| `GET /api/v1/worlds/:id/planet` | `get_world_planet` | Done |
| `GET /api/v1/worlds/:id/map` | `get_world_map` | Done |
| `GET /api/v1/worlds/:id/timeline` | `get_world_timeline` | Done |
| `GET /api/v1/worlds/:id/events` | `get_world_events` | Done |
| `GET /api/v1/worlds/:id/history` | `get_world_history` | Done |
| `GET /api/v1/worlds/:id/figures` | `get_world_figures` | Done |
| `GET /api/v1/worlds/:id/societies` | `get_world_societies` | Done |
| `GET /api/v1/worlds/:id/artifacts` | `get_world_artifacts` | Done |
| `GET /api/v1/worlds/:id/cataclysms` | `get_world_cataclysms` | Done |
| `GET /api/v1/worlds/:id/wonders` | `get_world_wonders` | Done |
| `GET /api/v1/worlds/:id/tectonics` | `get_world_tectonics` | Done |
| `POST /api/v1/worlds/:id/simulate` | `simulate_world` | Done |
| `GET /api/v1/worlds/:id/export` | `get_world_export` | Done |
| `DELETE /api/v1/worlds/:id` | via `create_world` | Done |
| `GET /api/v1/species` | `list_species` | Done |
| `GET /api/v1/species/:id` | `get_species` | Done |
| `GET /api/v1/artifacts` | `get_artifacts` | Done |
| `GET /api/v1/cataclysms` | `get_cataclysms` | Done |

**Storage Layer:**
- Tarball packaging (`.wfw` files): Done (`src/packaging.rs`)
- JSON serialization: Done
- World save/load/delete: Done (`src/storage.rs`)

### Phase 3 Issues

1. **Integration test fails to compile** - `tests/export_endpoint_test.rs` imports
   `world_factory::storage::{StorageManager, StorageConfig}` which don't exist at that path
2. **Dead code warnings** - `start()` function in `src/main.rs` unused; `get_cell()` in
   `tests/integration_world_generation.rs` unused

---

## Phase 4: Visualization - PARTIALLY COMPLETE

### Current State

The existing `web/index.html` (94KB) and `web/world.html` (72KB) are single-page viewers that:
- Render the world map via Canvas
- Have basic zoom/pan
- Display a timeline view
- Connect to API at `http://localhost:8080` (API base URL hardcoded)

### What's Missing (per SPEC.md Section 6)

|| Feature | Status | Notes ||
|---------|--------|-------|
| **Landing page (`GET /`)** | NOT DONE | No world selector with list of all worlds |
| **World overview page (`GET /worlds/:id`)** | NOT DONE | Metadata, tabs for Map/Timeline/Dashboard |
| **Map view (`GET /worlds/:id/map`)** | PARTIAL | Works but embedded in single page |
| **Timeline view (`GET /worlds/:id/timeline`)** | PARTIAL | Works but embedded in single page |
| **Dashboard view (`GET /worlds/:id/dashboard`)** | NOT DONE | No population charts, stats display |
| **Faction view** | NOT DONE | Replaced by Dashboard in spec |
| **Multi-world navigation** | NOT DONE | No way to switch between worlds |
| **PNG export** | NOT DONE | Canvas export button not implemented |
| **Create world from UI** | NOT DONE | No generation form on landing page |
| **Server-side HTML serving** | NOT DONE | No Axum routes for `GET /`, `GET /worlds/:id/*` |

### Required Implementation

The visualization needs to be refactored from a single-page app into a **multi-page routing system**:

```
GET  /                           -> Landing page (world list)
GET  /worlds/:id                 -> World overview
GET  /worlds/:id/map             -> Map view
GET  /worlds/:id/timeline        -> Timeline view
GET  /worlds/:id/dashboard       -> Dashboard
```

---

## Phase 5: Faction System - IMPLEMENTED BUT REGRESSED

> Reference: MichaelBlackwell/SWN3 implementation - `turnSlice.ts`, `turnManager.ts`, `faction.ts`

**Faction Turn System (`src/faction.rs`, `src/faction_turn.rs`, `src/faction_integration.rs`):**
- Turn structure (Income → Maintenance → Action → News)
- Faction attributes (Force/Cunning/Wealth/HP)
- Faction tags and goal types
- Attack/Move/Purchase/Diplomacy/Expand actions
- Asset system (categories, limits, upgrades)
- Multi-turn campaigns (homeworld/seizure/binding)
- Primal beast integration
- Victory conditions (epoch end, soft failure)
- AI faction behavior
- Data model and API endpoints

**CRITICAL: 8 tests failing** - Recent changes caused regressions in:
- `beasts::remnants::tests::test_remnant_decay`
- `beasts::slaying::tests::test_slaying_creates_remnant`
- `beasts::slaying::tests::test_insufficient_factions_fails`
- `beasts::slaying::tests::test_insufficient_power_fails`
- `beasts::slaying::tests::test_all_beasts_create_remnants`
- `faction::faction_stats_tests::hp_mechanics::test_recalculate_stats`
- `faction::faction_stats_tests::hp_mechanics::test_is_critical`
- `faction::faction_stats_tests::stat_calculations::test_wealth_calculation`

---

## Codebase Structure

```
src/
├── api/v1/           # HTTP API handlers
│   └── worlds.rs     # World CRUD + generation
├── terrain/          # Geography generation (23 files)
│   ├── elevation.rs, biome.rs, terrain_generator.rs
│   ├── tectonic/, resource_spawner.rs, erosion.rs
│   └── natural_wonders/
├── history/          # History generation
├── events/           # Event system (probability, effects)
├── figures.rs        # Notable figures
├── artifacts.rs      # Historical artifacts
├── cataclysms.rs     # Cataclysm system
├── settlements/       # Settlement generation
├── simulation/        # Population growth
├── species/           # Species templates
├── hydro/             # River/hydrology
├── faction.rs         # Faction system (1209 LOC)
├── faction_turn.rs   # Faction turn logic
├── faction_integration.rs
├── beasts/           # Primal beasts system
│   ├── profiles.rs, movement.rs, effects.rs
│   ├── slaying.rs, remnants.rs
├── storage.rs        # Storage manager
├── packaging.rs      # .wfw tarball handling
└── world/            # Planet/geography domain types
    └── generation/   # Voronoi, Lloyd relaxation
```

**Total: ~36,000+ LOC across all Rust source**

---

## Test Status

```
cargo test --lib
-> 435 tests passed, 8 FAILED, 0 ignored
-> Fails: beasts::slaying (4), beasts::remnants (1), faction::faction_stats_tests (3)
```

**Test regression summary:**
- All 8 failures are in `beasts` and `faction` modules
- Appears to be caused by stat calculation changes (wealth calculation off by 5: got 36, expected 41)
- HP mechanics tests also failing (critical threshold, recalculate)
- Slaying tests failing (remnant creation, faction requirements, power requirements)

---

## What Works End-to-End

1. Generate a world via `POST /api/v1/worlds` with config
2. Server generates terrain, biomes, rivers, settlements
3. History simulation runs, creating events, figures, artifacts
4. Fetch map data via `GET /api/v1/worlds/:id/map`
5. Fetch timeline via `GET /api/v1/worlds/:id/timeline`
6. Fetch figures via `GET /api/v1/worlds/:id/figures`
7. Export world as `.wfw` tarball
8. Primal beasts exist in the data model (profiles, movement, effects, slaying, remnants)

---

## Priority Fixes

1. **Fix 8 failing tests** in `beasts` and `faction` modules - regression from recent changes
2. **Fix `tests/export_endpoint_test.rs`** - broken import path for `StorageManager` and `StorageConfig`
3. **Address dead code warnings** - `start()` in main.rs, `get_cell` in tests
4. **CLI world persistence** - `generate` command should save `.wfw` to storage (per SPEC.md §7.4)
5. **Phase 4 visualization routing** - implement landing page and multi-page routing

---

## Spec Coverage vs Implementation

### Fully Implemented
- Phase 1 core geography (Voronoi, elevation, tectonics, rivers, biomes, resources, natural wonders)
- Phase 2 history (species, settlements, events, figures, artifacts, cataclysms)
- Phase 3 persistence & API (all endpoints, tarball storage)
- Phase 5 faction system types and turn structure (but with test regressions)

### Partially Implemented
- Phase 4 visualization: canvas map renders, but no multi-page routing or landing page
- CLI persistence: generate command works but doesn't save to `.wfw` storage

### Not Started
- Phase 4: landing page (`GET /`), world overview, dashboard, PNG export
- Phase 4: server-side HTML routes for `/worlds/:id/*`
- Phase 5: AI faction behavior (fully specced but not implemented)
- Phase 5: full campaign system with homeworld transition

---

## Docker Support

Dockerfile and docker-compose.yml are present and functional.

```bash
docker compose up -d world-factory   # Start persistent server on :8080
docker compose logs -f              # Watch logs
docker compose down                 # Stop server
```

---

## Files

- [SPEC.md](./SPEC.md) - Full specification (1782 lines, comprehensive)
- [API_CONTRACT.md](./API_CONTRACT.md) - API endpoint documentation
- [WOR-143-completion-summary.md](./WOR-143-completion-summary.md) - Phase completion notes