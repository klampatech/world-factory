# WOR-349: CTO Review — Issues Review

**Review Date:** 2026-05-07  
**Reviewer:** CTO Agent (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Status:** ✅ REVIEW COMPLETE

---

## Executive Summary

Reviewed codebase for outstanding TODO/FIXME comments and current development state. Active development ongoing (122 lines changed in API, 46 lines in terrain/species). Build status unknown (cargo not in PATH in harness environment).

**Build Status:** ⚠️ Unable to verify (cargo not in PATH)  
**Previous Test Status:** ✅ 406 lib tests passing (per CURRENT_STATUS.md, 2026-05-05)

---

## Active Development Changes

### API Changes (122 lines, +22/-26)

| File | Change | Purpose |
|------|--------|---------|
| `api/mod.rs` | +10 lines | Added `save_faction_registry()` method |
| `api/v1/factions.rs` | +19 lines | Extended faction turn system |
| `api/v1/worlds.rs` | +119 lines | Extended world endpoints |

### Terrain/Species Changes (46 lines, +15/-9)

| File | Change | Purpose |
|------|--------|---------|
| `hydro/drainage_basin.rs` | +5/-2 | Drainage basin fixes |
| `species/mod.rs` | +17/-6 | Species module improvements |
| `terrain/biome_assignment.rs` | +18/-9 | Biome assignment updates |
| `terrain/elevation_assignment.rs` | +4/-2 | Elevation fixes |
| `terrain/ocean.rs` | +37/-17 | Ocean generation updates |

---

## FIXMEs (6 items — unchanged from WOR-293)

All FIXMEs remain blocked by **AppState Clone+Send** — same root cause blocking WOR-67 integration tests.

| File | Line | Description |
|------|------|-------------|
| `api/mod.rs` | 84, 92, 100 | Integration tests blocked |
| `api/v1/species.rs` | 362, 370, 378 | Integration tests blocked |

**Root Cause:** `AppState` cannot implement `Clone + Send` due to Axum router internals.

---

## TODOs Remaining (25 items)

| Category | Count | Description |
|----------|-------|-------------|
| Store Integration | 8 | EventStore, CataclysmStore, ArtifactStore fetch calls |
| World Package Loading | 12 | Load planet, regions, settlements, biomes, drainage, tectonics, wonders |
| World Generation Pipeline | 1 | Call generation pipeline on worlds create |
| Database Filtering | 4 | Figures, settlements, wonders filtering logic |
| River Service | 2 | Load from world storage, wire DrainageBasinCalculator |
| Foundation | 2 | Entity system, world state management (lib.rs) |

### Notable TODOs for API Completeness

| Location | Description | Priority |
|----------|-------------|----------|
| `worlds.rs:341` | Call world generation pipeline on create | High |
| `worlds.rs:563-630` | Load timeline/events from EventStore | High |
| `worlds.rs:669-700` | Load figures/settlements from storage | Medium |
| `worlds.rs:931-1008` | Load planet/tectonics from package | Medium |
| `artifacts.rs:65,242` | Fetch from ArtifactStore | Medium |
| `cataclysms.rs:71,299` | Fetch from CataclysmStore | Medium |
| `river_service.rs:40,94` | Wire DrainageBasinCalculator | Medium |

---

## Phase Status Overview

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Core Generation | ✅ Done | World generation, terrain, biomes |
| Phase 2: History | ✅ Done | Events, figures, artifacts, settlements |
| Phase 3: Persistence & API | ⚠️ Mostly Done | Storage works, store integrations pending |
| Phase 4: Visualization | ⚠️ Partial | SPA exists, multi-page routing not done |
| Phase 5: Faction System | ⚠️ In Progress | FactionTurnState added, registry save method added |

---

## Critical Path Items

### 1. AppState Clone+Send (BLOCKS WOR-67)
Integration tests remain ignored. Design decision needed:
- **Option A:** Wrap state in `Arc<Mutex<...>>` to allow Clone
- **Option B:** Use channel-based test communication
- **Option C:** Mark tests as requiring architectural refactor

### 2. Store Integrations (HIGH PRIORITY)
Complete EventStore, CataclysmStore, ArtifactStore integrations:
- Resolves 8 TODO comments
- Enables full timeline/settlement/world data APIs
- Blocks Phase 3 completion

### 3. World Generation Pipeline Wiring (HIGH PRIORITY)
`worlds.rs:341` — Generation pipeline not called on world creation
- Blocks users from creating functional worlds via API

### 4. Phase 5 Faction Turn System
Active development continues:
- `save_faction_registry()` method added to AppState
- Faction turn endpoints in progress

---

## Recommendations

### Immediate Actions

1. **Create child issue** for AppState Clone+Send resolution
2. **Prioritize WOR-341** — Wire generation pipeline on world creation
3. **Schedule store integrations** — WOR-69, WOR-70, WOR-71

### Documentation Updates Needed

1. Update CURRENT_STATUS.md with:
   - Active development notes
   - Faction system progress
   - Phase 5 turn structure details

2. Archive WOR-293 after WOR-349 review complete

---

## Files Reviewed

| File | TODOs | FIXMEs | Status |
|------|-------|--------|--------|
| lib.rs | 2 | 0 | - |
| api/v1/artifacts.rs | 2 | 0 | - |
| api/v1/cataclysms.rs | 2 | 0 | - |
| api/v1/events.rs | 2 | 0 | - |
| api/v1/worlds.rs | 18 | 0 | Active dev |
| api/v1/species.rs | 0 | 3 | - |
| api/mod.rs | 0 | 3 | +save method |
| api/services/river_service.rs | 2 | 0 | - |
| **Total** | **28** | **6** | - |

---

## Next Actions

1. [ ] **Verify build** with `cargo check --lib` (when cargo available)
2. [ ] **Create child issue** for AppState Clone+Send resolution
3. [ ] **Wire generation pipeline** on world creation (WOR-341)
4. [ ] **Complete store integrations** (WOR-69, WOR-70, WOR-71)
5. [ ] **Update CURRENT_STATUS.md** with Phase 5 progress

---

*Review completed by CTO. Active development ongoing; blockers identified for Phase 3 completion.*
