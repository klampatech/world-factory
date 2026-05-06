# WOR-59: Full System Review — Architecture Report

**Status:** Draft for CTO Review  
**Date:** 2026-05-05  
**Reviewer:** SystemsArchitect Agent

---

## Executive Summary

**World Factory** is a Rust-based procedural world and history generation engine. It generates deterministic Earth-like planets with geological, climatic, ecological, and hydrological systems, simulates pre-history, tracks historical events and notable figures, and supports configurable species with templated behaviors.

**Overall Assessment:** The system is well-architected with clear separation of concerns. However, there are **critical gaps** between the API contract specification and current implementation status, plus **32 pre-existing test failures** that need resolution.

---

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         World Factory Engine                            │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
│  │  WorldGenerator │  │   World Config   │  │  HistoryGenerator (Phase2)│ │
│  │     (lib.rs)     │  │  (config.rs)    │  │    (history/generator)   │ │
│  └────────┬────────┘  └────────┬────────┘  └────────────┬──────────────┘ │
│           │                   │                        │                  │
│  ┌────────▼───────────────────▼────────────────────────▼──────────────┐ │
│  │                         Core Domain Layer                           │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────────┐  │ │
│  │  │ terrain/ │  │   hydro/ │  │ species/ │  │    settlements/     │  │ │
│  │  │ Elevation│  │  Rivers  │  │   Data   │  │ SettlementGenerator │  │ │
│  │  │ Biomes   │  │ Drainage │  │ Traits   │  │ Species Assignment  │  │ │
│  │  │Climate   │  │  Basins │  │ Templates│  │   Name Generation   │  │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────────┘  │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────────┐  │ │
│  │  │ events/ │  │ figures/ │  │history/  │  │   simulation/       │  │ │
│  │  │ EventType│  │  Dynasties│  │  Society │  │   Population        │  │ │
│  │  │ Probability│ │ Relationships│ │ Population │  │ Food/Disease      │ │ │
│  │  │ Timeline │  │ FigureGen │  │Artifacts │  │                     │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │                         API Layer (api/)                            │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐  │ │
│  │  │   Axum Router  │  │   Models.rs    │  │    v1/ Routes       │  │ │
│  │  │  CORS Config   │  │  ApiError      │  │ worlds, events,     │  │ │
│  │  │  /health      │  │  StorageMgr    │  │ species             │  │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────────────────┐ │
│  │                     Persistence Layer                                │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐  │ │
│  │  │   packaging/   │  │   storage/     │  │    serialization   │  │ │
│  │  │  .wfw format  │  │ FileLock/      │  │    serde_json      │  │ │
│  │  │  Tar+gzip     │  │ StorageManager │  │                     │  │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────┘  │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Module Inventory & Status

### ✅ Implemented & Functional

| Module | Files | Purpose | Status |
|--------|-------|---------|--------|
| `terrain` | 15+ | Elevation grids, biomes, climate, LOD | ✅ Core complete |
| `hydro` | 5 | Rivers, drainage basins, erosion | ✅ Core complete |
| `events` | 6 | Historical events, timeline, probability engine | ✅ Core complete |
| `figures` | 1 | Notable figures, dynasties, relationships | ✅ Core complete |
| `settlements` | 1 | Settlement generation, species-aware placement | ✅ Core complete |
| `species` | 2 | 5 playable species with traits and name templates | ✅ Core complete |
| `history` | 6 | Society, population simulation, history generator | ✅ Phase 2 complete |
| `api` | 10+ | Axum REST API with v1 routes | ⚠️ Partial |
| `packaging` | 1 | .wfw save/load format | ✅ Complete |
| `storage` | 1 | StorageManager with file locking | ✅ Complete |

### ⚠️ API Contract vs Implementation Gap

**Contract Specified in `docs/API_CONTRACT.md`:**
- `GET /api/worlds/:id/map` — Map data with polygons, biomes, resources
- `GET /api/worlds/:id/events` — Event timeline with filtering
- `GET /api/worlds/:id/figures` — Historical figures
- `GET /api/v1/species` — Species definitions
- `GET /api/v1/worlds/:id/societies` — Societies with settlements
- `GET /api/events/:id` — Single event
- Cache headers (ETag) support

**Currently Implemented in `src/api/v1/`:**
- ✅ `worlds.rs` (95KB - substantial implementation)
- ✅ `species.rs` (12KB)
- ✅ `events.rs` (basic routing)
- ✅ `artifacts.rs`, `cataclysms.rs`, `factions.rs`
- ❌ **Missing:** `/worlds/:id/map` endpoint
- ❌ **Missing:** Cache headers (ETag)
- ❌ **Missing:** Societies endpoint detail validation

---

## Data Flow & API Contracts

### Primary API Flow

```
Client Request
     │
     ▼
┌─────────────┐     ┌─────────────────┐     ┌──────────────────┐
│   Router    │────▶│  AppState       │────▶│  StorageManager  │
│ (Axum)      │     │ (Clone, Shared) │     │ (World lookup)   │
└─────────────┘     └─────────────────┘     └──────────────────┘
                          │
                          ▼
                   ┌─────────────────┐
                   │  Domain Objects │
                   │  (serde_json)   │
                   └─────────────────┘
                          │
                          ▼
                    JSON Response
```

### Key API Endpoints

| Endpoint | Method | Status | Notes |
|----------|--------|--------|-------|
| `/health` | GET | ✅ | Basic health check |
| `/api/v1/worlds` | GET | ✅ | List worlds |
| `/api/v1/worlds/:id` | GET | ✅ | World metadata |
| `/api/v1/worlds/:id/map` | GET | ❌ | **NOT IMPLEMENTED** |
| `/api/v1/worlds/:id/timeline` | GET | ⚠️ | Needs validation |
| `/api/v1/worlds/:id/events` | GET | ⚠️ | Alias for timeline |
| `/api/v1/worlds/:id/figures` | GET | ⚠️ | Needs validation |
| `/api/v1/v1/species` | GET | ✅ | Species definitions |
| `/api/v1/v1/worlds/:id/societies` | GET | ⚠️ | Needs validation |

---

## Technology Choices

### Stack Summary

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Core Engine | **Rust** | Memory safety, performance, determinism |
| Serialization | **serde + serde_json** | Standard Rust serialization |
| API Server | **Axum 0.7** | Type-safe, async, tower integration |
| Async Runtime | **Tokio** | Multi-threaded async |
| CORS | **tower-http CORS** | Frontend integration |
| CLI | **clap** | Argument parsing |
| RNG | Custom **Mulberry32** | Seeded determinism |

### Alternatives Considered

- **API Framework:** Actix-web → Axum (better type safety, tower ecosystem)
- **Serialization:** bincode → serde_json (interoperability over speed for API)
- **Async:** async-std → Tokio (larger ecosystem, runtime compatibility)

---

## Performance Budgets

| Metric | Target | Current | Notes |
|--------|--------|---------|-------|
| World gen (256×256) | < 5s | Unknown | Needs benchmarking |
| Memory/million entities | < 500MB | Unknown | Needs measurement |
| API `/timeline` | < 300ms | Unknown | Needs profiling |
| API `/map` | < 500ms | N/A | Endpoint missing |

---

## Concurrency Model

### Thread Strategy

```
Main Thread (CLI)           Tokio Threads (API)
      │                           │
      ▼                           ▼
┌──────────┐               ┌──────────┐
│ Sync RNG │               │ AppState  │
│ Generation│              │ (Clone)   │
│ (blocking)│              │           │
└──────────┘               │ Handlers  │
                          │ (async)   │
                          └──────────┘
```

- **CLI Mode:** Synchronous, single-threaded generation
- **API Mode:** Async handlers with `AppState` shared via `Arc`/`Clone`
- **Determinism:** All RNG seeded via `Mulberry32` — same seed = same output

---

## Data Ownership

| Data | Owner Module | Access Pattern |
|------|-------------|----------------|
| World metadata | `storage/` | Single writer, multiple readers |
| Terrain grid | `terrain/` | Generated fresh per session |
| Events | `events/` | Append-only timeline |
| Figures | `figures/` | Generated from events |
| Settlements | `settlements/` | Generated once, stored |
| Species | `species/` | Immutable after init |

**No shared mutable state** between modules — each generator owns its output.

---

## Risk Register

| ID | Risk | Severity | Probability | Mitigation |
|----|------|----------|-------------|------------|
| R1 | API endpoints don't match contract | High | High | Validate against `API_CONTRACT.md`, implement missing endpoints |
| R2 | 32 library test failures | High | Active | Fix or document as known issues |
| R3 | Missing `/worlds/:id/map` endpoint | Medium | High | Implement map data generation API |
| R4 | No caching strategy (ETag) | Medium | Medium | Add cache headers per contract |
| R5 | Phase 2 integration tests pass but lib tests fail | Medium | Medium | Investigate test setup issues |

---

## Fault Isolation

- **API crashes:** Do not affect generation engine (separate binaries possible)
- **Generation failures:** Return error JSON, don't crash API
- **Storage errors:** Graceful degradation, error reporting

---

## Critical Gaps & Recommendations

### 1. Missing Map Endpoint (R3)

**Issue:** `GET /api/worlds/:id/map` per `API_CONTRACT.md` is not implemented.

**Required:**
- Polygon data for terrain rendering
- Biome colors and types
- Resource positions
- Entity locations (settlements, landmarks)
- Elevation grid data
- ETag caching headers

**Recommendation:** Create child issue to implement map endpoint.

### 2. Test Failures (R2)

**Issue:** 32 library tests failing across multiple modules:
- events::tests
- figures::tests
- hydro::drainage_basin::tests
- terrain::biome_assignment::tests
- And more...

**Recommendation:** Investigate test setup/fixture issues. Create child issue to fix tests.

### 3. API Validation Against Contract

**Issue:** No automated validation that API responses match contract.

**Recommendation:** Add integration tests that validate response shapes against `API_CONTRACT.md`.

---

## Out of Scope for This Review

- Frontend visualization code (`demo.html`, `demo-society-dashboard.html`)
- E2E test suite (`e2e/`)
- Smoke tests (`api_smoke_tests.py`)
- Deployment/infrastructure configuration

---

## Conclusion

**System Health:** 7/10

The World Factory engine is well-architected with clean separation of concerns. The domain model is comprehensive, covering terrain, hydrology, settlements, species, history, events, and figures. The Phase 2 history integration is complete.

**Critical Next Steps:**
1. Implement missing `/api/worlds/:id/map` endpoint
2. Fix or document the 32 failing library tests
3. Add ETag caching per API contract
4. Validate API responses against contract with integration tests

---

*Report generated for CTO review. See [WOR-59](/WOR/issues/WOR-59) for tracking.*
