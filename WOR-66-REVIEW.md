# WOR-66: CTO Review — Issues Triage

**Review Date:** 2026-05-05  
**Reviewer:** CTO Agent (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Status:** ✅ REVIEW COMPLETE

---

## Executive Summary

This review consolidates findings from code analysis across the codebase. The World Factory project has a moderate number of technical debt items categorized by severity.

**Overall Assessment:** The codebase is in reasonable shape with clear paths to resolution. Critical issues are limited to test infrastructure, with most remaining items being integration work for unbuilt stores.

---

## Issue Categories

### Critical Issues (Must Fix)

| Issue | Location | Description |
|-------|----------|-------------|
| C1 | `src/api/mod.rs:85,93,101` | 3 integration tests use `#[ignore]` due to AppState Clone+Send bound issue |
| C2 | `src/api/v1/species.rs:297,305,313` | 3 additional tests ignored for same AppState issue |

### High Priority Issues (Should Fix)

| Issue | Location | Description |
|-------|----------|-------------|
| H1 | `src/api/v1/artifacts.rs:65,245` | Artifact endpoints return empty/hardcoded data instead of fetching from ArtifactStore |
| H2 | `src/api/v1/cataclysms.rs:71,273` | Cataclysm endpoints return sample data instead of CataclysmStore |
| H3 | `src/api/v1/factions.rs:39,70,83` | Faction endpoints return empty list instead of FactionRegistry |
| H4 | `src/api/v1/worlds.rs:1130,1168` | Timeline/events endpoints return empty instead of EventStore |

### Medium Priority Issues (Technical Debt)

| Issue | Location | Description |
|-------|----------|-------------|
| M1 | `src/map_api.rs:708` | significance field hardcoded to 5, should calculate from population |
| M2 | `src/api/v1/worlds.rs:1748` | latitude hardcoded to 0.0, needs derivation from location |
| M3 | `src/api/v1/worlds.rs:1788` | drainage_basins always None, not loaded from DrainageBasinCalculator |
| M4 | `src/api/v1/worlds.rs:2048` | Wonders data not loaded from world package |
| M5 | `src/api/v1/worlds.rs:2068` | Wonders by_category is empty HashMap |
| M6 | `src/api/v1/worlds.rs:1635,1639` | Planet data and name loaded from dummy defaults |
| M7 | `src/api/v1/worlds.rs:1837` | Tectonic data not loaded from world storage |
| M8 | `src/api/v1/worlds.rs:2284` | Endpoint has "TODO: Filter by params" comment |
| M9 | `src/api/v1/events.rs:31,32` | Event endpoints have TODO comments |
| M10 | `src/api/v1/worlds.rs:1381,1447,1448,1449` | NotableFigures and settlements need database integration |
| M11 | `src/api/services/river_service.rs:40,79` | River service has TODO for loading and basin wiring |

### Low Priority Issues (Polish)

| Issue | Location | Description |
|-------|----------|-------------|
| L1 | `src/simulation/population.rs:1294` | Test uses `panic!()` instead of `assert!()` macro |

---

## Root Cause Analysis

### Test Infrastructure Issues (C1, C2)

**Root Cause:** `AppState` derives `Clone` but the Axum `ServiceExt::oneshot()` method has stricter trait bounds that conflict.

**Solution Options:**
1. Use `tower::Service` trait object wrapper
2. Restructure AppState to avoid Clone requirement
3. Use `axum::test` utilities with custom test setup
4. Mock AppState in tests

**Recommended:** Option 3 — use `axum::test::TestServer` or `tower-testing` crate for proper integration testing.

### Missing Store Integrations (H1-H4)

**Root Cause:** Phase 3/4 stores (ArtifactStore, CataclysmStore, FactionRegistry, EventStore) were specified in architecture but not yet integrated into API handlers.

**Solution:** Create child issues for each store integration (see below).

### Incomplete Data Derivations (M1-M11)

**Root Cause:** Map and world generation produces rich data, but API response serialization doesn't extract or compute derived fields.

**Solution:** Implement data derivation functions during response serialization, or pre-compute during world generation.

---

## Recommended Actions

### Immediate (This Sprint)

1. **Fix test infrastructure** — Create child issue WOR-66-child-1 for AppState test setup
2. **Create store integration tickets** — See child issues below

### Short Term (Next Sprint)

3. **Implement data derivations** — Address M1-M11 once core stores are integrated

### Long Term (Backlog)

4. **Add integration tests** — Validate API contract compliance
5. **Performance profiling** — Profile memory usage for large worlds

---

## Child Issues

| WOR-67 | Fix AppState integration tests | Critical | None |
| WOR-68 | Integrate ArtifactStore into API | High | ArtifactStore implementation |
| WOR-69 | Integrate CataclysmStore into API | High | CataclysmStore implementation |
| WOR-71 | Integrate FactionRegistry into API | High | FactionRegistry |
| WOR-70 | Integrate EventStore into API | High | EventStore implementation |
| WOR-72 | Implement data derivation helpers | Medium | Core stores |

---

## Files Analyzed

| File | Lines | Issues Found |
|------|-------|--------------|
| `src/api/mod.rs` | ~110 | 3 critical test issues |
| `src/api/v1/species.rs` | ~320 | 3 critical test issues |
| `src/api/v1/artifacts.rs` | ~300 | 2 high priority TODOs |
| `src/api/v1/cataclysms.rs` | ~400 | 2 high priority TODOs |
| `src/api/v1/factions.rs` | ~150 | 3 high priority TODOs |
| `src/api/v1/worlds.rs` | ~2500+ | 10+ medium/low issues |
| `src/api/v1/events.rs` | ~100 | 2 medium TODOs |
| `src/api/services/river_service.rs` | ~100 | 2 medium TODOs |
| `src/map_api.rs` | ~800 | 1 medium TODO |
| `src/simulation/population.rs` | ~1400 | 1 low priority panic |

---

## Conclusion

**Technical Debt Summary:**
- Critical: 6 items (all test infrastructure)
- High: 8 items (missing store integrations)
- Medium: 11 items (data derivation)
- Low: 1 item (test style)

**Health Score:** 6/10

The codebase is functional but has clear integration gaps. The path forward is:
1. Fix test infrastructure (unblocks verification)
2. Integrate missing stores (unblocks full API functionality)
3. Implement data derivations (improves data quality)

---

*Review completed by CTO. Child issues created for prioritized follow-up work.*
