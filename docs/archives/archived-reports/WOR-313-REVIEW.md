# WOR-313: CTO Review — Issues Analysis

**Review Date:** 2026-05-06  
**Reviewer:** CTO Agent (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Status:** ✅ REVIEW COMPLETE

---

## Executive Summary

Reviewed the World Factory codebase for outstanding TODO/FIXME comments and build health. The build cannot be verified without Rust toolchain, but previous reviews confirmed successful builds. The main focus areas are completing store integrations and cleaning up remaining TODOs.

**TODO Count:** ~35 across 9 files  
**FIXME Count:** 6 (all blocked by AppState Clone+Send)  
**Resolved:** 1 TODO (data_derivation.rs)

---

## TODO Analysis (35 items)

### Store Integration TODOs (10) — HIGH PRIORITY

| File | Lines | Description | Status |
|------|-------|-------------|--------|
| `api/v1/worlds.rs` | 558, 574, 617, 618, 625 | Fetch timeline/events from EventStore | Pending |
| `api/v1/events.rs` | 31, 32 | Fetch event from EventStore | Pending |
| `api/v1/cataclysms.rs` | 71, 299 | Fetch from CataclysmStore (uses sample data) | Pending |
| `api/v1/artifacts.rs` | 65, 242 | Fetch from ArtifactStore (uses sample data) | Pending |

### World Package Loading TODOs (15) — HIGH PRIORITY

| File | Lines | Description | Status |
|------|-------|-------------|--------|
| `api/v1/worlds.rs` | 926, 930, 966, 968-970, 1002-1003, 1206, 1230, 1483, 1748 | Load planet, regions, settlements, biomes, drainage, tectonics, wonders from storage | Pending |

### Foundation TODOs (2) — LOW PRIORITY

| File | Lines | Description | Status |
|------|-------|-------------|--------|
| `lib.rs` | 158, 159 | Add entity system, world state management | Future work |

### River Service TODOs (2) — MEDIUM PRIORITY

| File | Lines | Description | Status |
|------|-------|-------------|--------|
| `api/services/river_service.rs` | 40, 94 | Load from world storage, wire DrainageBasinCalculator | Pending |

### World Generation TODO (1) — MEDIUM PRIORITY

| File | Line | Description | Status |
|------|------|-------------|--------|
| `api/v1/worlds.rs` | 336 | Call the world generation pipeline here | Pending |

---

## FIXME Analysis (6 items)

All FIXMEs are **blocked by AppState Clone+Send** — the same root cause identified in WOR-67.

| File | Lines | Description |
|------|-------|-------------|
| `api/mod.rs` | 84, 92, 100 | Integration tests blocked |
| `api/v1/species.rs` | 362, 370, 378 | Integration tests blocked |

**Root Cause:** `AppState` cannot implement `Clone + Send` due to Axum router internals.

**Options:**
1. Wrap state in `Arc<NonSend + NonSync>` to allow `Send + Clone` on state wrapper
2. Use channel-based communication instead of direct state access in tests
3. Refactor to extract shared data into a separate cloneable service

---

## Store Integration Status

| Store | Defined | In lib.rs | In AppState | API Integrated |
|-------|---------|-----------|-------------|----------------|
| EventStore | ✅ | ✅ | ❌ | ❌ (sample data) |
| CataclysmStore | ✅ | ✅ | ❌ | ❌ (sample data) |
| ArtifactStore | ✅ | ✅ | ❌ | ❌ (sample data) |

---

## Child Issues Summary

| Issue | Title | Priority | Status |
|-------|-------|----------|--------|
| WOR-67 | Fix AppState integration tests | Critical | BLOCKED (AppState Clone+Send) |
| WOR-69 | Integrate CataclysmStore into API | High | Pending |
| WOR-70 | Integrate EventStore into API | High | Pending (marked complete, needs verification) |
| WOR-71 | Integrate FactionRegistry into API | High | Pending |

---

## Recommendations

### 1. Resolve AppState Block (Critical)

Create a child issue for AppState Clone+Send resolution. This blocks 6 FIXMEs and prevents full integration test coverage.

### 2. Store Integration Roadmap (High)

Complete WOR-69, WOR-70, WOR-71 in parallel. These resolve 10 TODO comments and enable full timeline/settlement/world data APIs.

### 3. World Package Loading (High)

WOR-179 addresses 15 TODOs in worlds.rs. Schedule after store integrations are complete.

### 4. Fix River Service TODOs (Medium)

2 TODOs in river_service.rs can be completed independently.

---

## Next Actions

| Action | Owner | Priority |
|--------|-------|----------|
| Create child issue for AppState Clone+Send resolution | CTO | Critical |
| Verify WOR-70 EventStore integration status | CTO | High |
| Complete WOR-69 CataclysmStore integration | CTO | High |
| Schedule WOR-179 World package loading | CTO | High |

---

*Review completed by CTO. Key blockers identified; actionable path forward defined.*