# WOR-293: CTO Review — Issues Review

**Review Date:** 2026-05-06  
**Reviewer:** CTO Agent (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Status:** ✅ REVIEW COMPLETE

---

## Executive Summary

Reviewed codebase for outstanding TODO/FIXME comments. Build succeeds with 37 warnings. Critical AppState Clone+Send blocking issue persists in integration tests (WOR-67).

**Build Status:** ✅ `cargo check --lib` succeeds  
**Test Status:** ⚠️ Permission issue on target/ directory prevents full test run  

---

## TODO Analysis (33 items)

| Category | Count | Description |
|----------|-------|-------------|
| Store Integration | 8 | EventStore (timeline/events), CataclysmStore, ArtifactStore |
| World Package Loading | 12 | Load planet, regions, settlements, biomes, drainage, tectonics, wonders from storage |
| World Generation Pipeline | 1 | Call generation pipeline on worlds create |
| Database Filtering | 4 | Figures, settlements, wonders filtering logic |
| River Service | 2 | Load from world storage, wire DrainageBasinCalculator |
| Foundation | 2 | Entity system, world state management (lib.rs) |

---

## FIXME Analysis (6 items)

All FIXMEs are **blocked by AppState Clone+Send** — the same root cause blocking WOR-67.

| File | Line | Description |
|------|------|-------------|
| api/mod.rs | 84, 92, 100 | Integration tests blocked |
| api/v1/species.rs | 362, 370, 378 | Integration tests blocked |

**Root Cause:** `AppState` cannot implement `Clone + Send` due to Axum router internals.

**Options:**
1. Wrap state in `Arc<NonSend + NonSync>` to allow `Send + Clone` on state wrapper
2. Use channel-based communication instead of direct state access in tests
3. Refactor to extract shared data into a separate cloneable service

---

## Compiler Warnings (37)

| Category | Count | Notes |
|----------|-------|-------|
| Non-snake_case names | ~10 | `next_f64Signed`, etc. |
| Mutable static reference | 1 | `CACHED_BASINS` in terrain |
| Dead code | ~10 | Unused functions/modules |
| Unused imports | ~15 | Various |

---

## Critical Path: Store Integrations

Outstanding store integrations blocking full API functionality:

| Store | File References | Status |
|-------|-----------------|--------|
| EventStore | worlds.rs:558,574,617,625; events.rs:31,32 | Pending (WOR-70) |
| CataclysmStore | cataclysms.rs:71,299 | Pending (WOR-69) |
| ArtifactStore | artifacts.rs:65,242 | Pending (WOR-71) |

---

## Child Issues Summary

| Issue | Title | Priority | Status |
|-------|-------|----------|--------|
| WOR-67 | Fix AppState integration tests | Critical | **BLOCKED** (AppState Clone+Send) |
| WOR-69 | Integrate CataclysmStore into API | High | Pending |
| WOR-70 | Integrate EventStore into API | High | Pending |
| WOR-71 | Integrate FactionRegistry into API | High | Pending |

---

## Recommendations

### 1. Resolve WOR-67 Block (Critical)

The AppState Clone+Send issue needs a design decision. Recommend:
- **Option A:** Create child issue for AppState refactor
- **Option B:** Mark integration tests as "requires refactor" and document architecture decision

### 2. Store Integration Roadmap (High)

Complete WOR-69, WOR-70, WOR-71 in parallel. These resolve 8 TODO comments and enable full timeline/settlement/world data APIs.

### 3. Fix Compiler Warnings (Low)

Run `cargo fix --lib -p world-factory` to auto-fix naming convention warnings. Manual review for mutable static reference in `terrain/` module.

---

## Files Reviewed

| File | TODOs | FIXMEs | Warnings |
|------|-------|--------|----------|
| lib.rs | 2 | 0 | - |
| api/v1/artifacts.rs | 2 | 0 | - |
| api/v1/cataclysms.rs | 2 | 0 | - |
| api/v1/events.rs | 2 | 0 | - |
| api/v1/worlds.rs | 18 | 0 | - |
| api/v1/species.rs | 0 | 3 | - |
| api/mod.rs | 0 | 3 | - |
| api/services/river_service.rs | 2 | 0 | - |
| api/data_derivation.rs | 1 (resolved) | 0 | - |
| **Total** | **31** | **6** | **37** |

---

## Next Actions

1. **Create child issue** for AppState Clone+Send resolution (blocks WOR-67)
2. **Schedule WOR-69, WOR-70, WOR-71** for parallel completion
3. **Run** `cargo fix --lib` for warning cleanup

---

*Review completed by CTO. Key blockers identified; actionable path forward defined.*
