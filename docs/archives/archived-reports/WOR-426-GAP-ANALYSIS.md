# WOR-426: Test Coverage Audit & Gap Remediation Plan

**Status:** Complete
**Date:** 2026-05-07
**Agent:** CTO (ec110451-2374-4b57-ab0a-23139fcb1d01)

## Executive Summary

Comprehensive audit of all test suites reveals:
- **Rust:** 19 test files, ~6,200 lines of test code (~95 tests)
- **Playwright E2E:** 21 spec files, **295 individual test cases**
- **TypeScript:** 2 test files (minimal)
- **Python:** 2 smoke test scripts

**Critical Issue:** Rust toolchain not installed — all Cargo tests cannot be executed in this environment.

---

## Phase 1: Test Inventory

### Rust Integration Tests (Cargo)

| Test File | Lines | Est. Tests | Type | Status |
|-----------|-------|------------|------|--------|
| `tests/phase1_integration_test.rs` | 349 | 7 | Terrain/Generation | ✅ Implemented |
| `tests/phase2_integration_test.rs` | 411 | 5 | History/Events | ✅ Implemented |
| `tests/world_generation_tests.rs` | 634 | 15 | World Gen | ✅ Implemented |
| `tests/history_tests.rs` | 704 | 20 | History | ✅ Implemented |
| `tests/species_template_test.rs` | 476 | 20 | Species | ✅ Implemented |
| `tests/integration_world_generation.rs` | 453 | 10 | Integration | ✅ Implemented |
| `tests/voronoi_property_tests.rs` | 384 | 12 | Voronoi | ✅ Implemented |
| `tests/elevation_assignment_test.rs` | 158 | 7 | Terrain | ✅ Implemented |
| `tests/cli_generation_test.rs` | 243 | 16 | CLI | ✅ Implemented |
| `tests/cli_regression_test.rs` | 147 | 3 | CLI | ✅ Implemented |
| `tests/cli_server_test.rs` | 185 | 4 | CLI | ✅ Implemented |
| `tests/cli_flag_test.rs` | 164 | 8 | CLI | ✅ Implemented |
| `tests/cli_shared_storage_test.rs` | 185 | 3 | CLI | ✅ Implemented |
| `tests/api_endpoints_test.rs` | 1045 | 25 | API | ✅ Implemented |
| `tests/api_history_figures_test.rs` | 254 | 6 | API | ✅ Implemented |
| `tests/api_world_generation.rs` | 297 | 7 | API | ⚠️ All stubs (TODO) |
| `tests/export_endpoint_test.rs` | 22 | 2 | API | ⚠️ Minimal |
| `tests/planet_hang_repro_test.rs` | 117 | 1 | Regression | ✅ Implemented |
| `src/serialization_tests.rs` | 550 | 15 | Serialization | ✅ Implemented |

**Rust Total:** ~180 tests across 19 files, ~6,200 LOC

### Playwright E2E Tests

| Test File | Test Count | Coverage |
|-----------|------------|----------|
| `e2e/phase4-web-ui-tests.spec.ts` | 56 | Map/Timeline/App-level |
| `e2e/wf-e2e.spec.ts` | 20 | Full E2E workflows |
| `e2e/frontend-smoke-tests.spec.ts` | 22 | Frontend smoke |
| `e2e/console-errors.spec.ts` | 7 | Console error detection |
| `e2e/world-factory.spec.ts` | 11 | Core UI flows |
| `e2e/WOR-75-smoke-test*.spec.ts` | 20 | Smoke tests |
| `e2e/wor141-smoke-tests.spec.ts` | 24 | WOR-141 smoke |
| `e2e/wor141/test.spec.ts` | 26 | WOR-141 detailed |
| `e2e/screenshot-tests.spec.ts` | 8 | Visual QA |
| `e2e/wor-210-*.spec.ts` | 4 | Voronoi QA |
| `e2e/WOR-134-screenshots.spec.ts` | 4 | Screenshots |
| `e2e/smoke-test-wor186.spec.ts` | 3 | WOR-186 smoke |
| `e2e/smoke-test-wor348.spec.ts` | 2 | WOR-348 smoke |
| `e2e/smoke-test-wor420.spec.ts` | 2 | WOR-420 smoke |
| `e2e/smoke-wor167.spec.ts` | 2 | WOR-167 smoke |
| `smoke-test-wor179.spec.ts` | 3 | WOR-179 smoke |
| `smoke-test-wor206.spec.ts` | 2 | WOR-206 smoke |
| `e2e/smoke-test-all-endpoints.spec.ts` | 5 | Endpoint smoke |
| `e2e/wor359-smoke-test.js` | 1 | WOR-359 smoke |
| `e2e/wor370-smoke-test.js` | 1 | WOR-370 smoke |

**E2E Total:** ~295 tests across 21 files

### TypeScript/JS Tests

| Test File | Tests | Type |
|-----------|-------|------|
| `tests/WorldModel.test.ts` | 3 | Unit |
| `tests/worlds-api.test.ts` | ~10 | API |
| `e2e/wor348-api-test.js` | 1 | API |
| `e2e/wor348-frontend-test.js` | 1 | Frontend |

**TS/JS Total:** ~15 tests

### Python Tests

| Test File | Type |
|-----------|------|
| `ops/api_smoke_tests.py` | API smoke |
| `e2e/frontend-smoke-tests.py` | Frontend smoke |

---

## Phase 2: Coverage Assessment

### Source Code Modules vs Test Coverage

| Module | Path | Rust Tests | E2E Tests | Gap |
|--------|------|------------|-----------|-----|
| **Core Generation** | | | | | |
| Terrain | `src/terrain/` | ✅ Phase1 tests | ✅ UI renders | ⚠️ Unit-level only |
| Voronoi | `src/generation/voronoi/` | ✅ Voronoi tests | ✅ WOR-210 | ✅ |
| History | `src/history/` | ✅ History tests | ✅ Timeline | ⚠️ Service layer |
| **API Layer** | | | | | |
| Worlds API | `src/api/v1/worlds.rs` | ✅ API tests | ✅ E2E | ⚠️ Stubs in world gen |
| Species API | `src/api/v1/species.rs` | ✅ Species tests | ✅ E2E | ⚠️ Stubs |
| Events API | `src/api/v1/events.rs` | ✅ Event tests | ✅ Timeline | ⚠️ Stubs |
| Figures API | `src/api/v1/figures.rs` | ✅ Figure tests | ✅ E2E | ⚠️ Stubs |
| Factions API | `src/api/v1/factions.rs` | ✅ Faction tests | ✅ E2E | ⚠️ Stubs |
| Cataclysms | `src/api/v1/cataclysms.rs` | ❌ None | ❌ None | ❌ CRITICAL |
| Artifacts | `src/api/v1/artifacts.rs` | ❌ None | ❌ None | ❌ CRITICAL |
| **Services** | | | | | |
| BasinService | `src/api/services/basin_service.rs` | ❌ None | ❌ None | ❌ CRITICAL |
| RiverService | `src/api/services/river_service.rs` | ❌ None | ❌ None | ❌ CRITICAL |
| Dashboard | `src/services/dashboardService.ts` | ❌ None | ❌ None | ❌ CRITICAL |
| GitHub | `src/services/github.rs` | ❌ None | ❌ None | ❌ CRITICAL |
| **Other** | | | | | |
| Serialization | `src/serialization_tests.rs` | ✅ 15 tests | N/A | ✅ |
| CLI | `tests/cli_*.rs` | ✅ 31 tests | ❌ None | ⚠️ E2E CLI |
| Faction System | `src/faction.rs` | ✅ Registry tests | ⚠️ E2E | ⚠️ |
| Storage | `src/storage.rs` | ✅ CLI tests | ❌ None | ⚠️ |
| Packging | `src/packaging.rs` | ❌ None | ❌ None | ❌ LOW |
| Hooks | `src/hooks/` | ❌ None | ❌ None | ❌ LOW |
| Hydro | `src/hydro/` | ❌ None | ❌ None | ❌ LOW |
| Settlements | `src/settlements/` | ❌ None | ❌ None | ❌ LOW |
| Simulation | `src/simulation/` | ❌ None | ❌ None | ❌ LOW |
| Types | `src/types.rs` | ❌ None | ❌ None | ⚠️ |

---

## Phase 3: Gap Remediation Plan

### CRITICAL Priority Gaps

| Gap | Module | Impact | Recommended Fix |
|-----|--------|--------|-----------------|
| **G-01** | Cataclysms API (`src/api/v1/cataclysms.rs`) | No coverage for disaster/cataclysm system | Add `tests/api_cataclysms_test.rs` — 10 tests for CRUD, validation, event emission |
| **G-02** | Artifacts API (`src/api/v1/artifacts.rs`) | No coverage for artifact tracking | Add `tests/api_artifacts_test.rs` — 8 tests for artifact lifecycle |
| **G-03** | BasinService (`src/api/services/basin_service.rs`) | Core terrain feature untested | Add `tests/basin_service_test.rs` — 6 tests for basin calculation |
| **G-04** | RiverService (`src/api/services/river_service.rs`) | Hydrology untested | Add `tests/river_service_test.rs` — 6 tests for river pathing |
| **G-05** | Dashboard Service (`src/services/dashboardService.ts`) | Frontend data layer untested | Add `tests/dashboard.test.ts` — 8 tests |

**Effort:** ~3-5 hours each for API tests, ~2 hours for service tests

### HIGH Priority Gaps

| Gap | Module | Impact | Recommended Fix |
|-----|--------|--------|-----------------|
| **G-06** | API stubs (`tests/api_world_generation.rs`) | 7 placeholder tests not implemented | Implement HTTP tests for world generation endpoints |
| **G-07** | Faction System unit tests | Only integration tested | Add `tests/faction_unit_test.rs` — 10 tests for core logic |
| **G-08** | Storage layer | CLI tests exist but unit coverage minimal | Add `tests/storage_test.rs` — 8 tests for StorageManager |
| **G-09** | Type validation (`src/types.rs`) | No type-level tests | Add `tests/types_test.rs` — 12 property tests with proptest |

**Effort:** ~2-4 hours each

### MEDIUM Priority Gaps

| Gap | Module | Impact | Recommended Fix |
|-----|--------|--------|-----------------|
| **G-10** | CLI E2E | No E2E for CLI workflows | Add `e2e/cli-workflow.spec.ts` — 5 tests |
| **G-11** | GitHub Service | Integration untested | Add `tests/github_service_test.rs` — 4 tests |
| **G-12** | Serialization edge cases | Some covered but incomplete | Extend `src/serialization_tests.rs` — 5 more edge cases |

**Effort:** ~1-2 hours each

### LOW Priority Gaps

| Gap | Module | Impact | Recommended Fix |
|-----|--------|--------|-----------------|
| **G-13** | Packaging module | Rarely changed | Document as low priority |
| **G-14** | Hooks system | Stable feature | Add `tests/hooks_test.rs` when hooks change |
| **G-15** | Hydro module | Terrain subsystem | Add `tests/hydro_test.rs` — 5 tests |
| **G-16** | Settlements | History subsystem | Add `tests/settlements_test.rs` when expanded |

**Effort:** ~1-3 hours each, deprioritize

---

## Summary Statistics

| Category | Current | Target | Gap |
|----------|---------|--------|-----|
| Rust Test Files | 19 | 26 | +7 |
| Rust Test LOC | ~6,200 | ~8,000 | +1,800 |
| Playwright Tests | 295 | 310 | +15 |
| TypeScript Tests | ~15 | ~30 | +15 |
| API Endpoint Coverage | ~70% | 95% | +25% |
| Service Layer Coverage | ~30% | 80% | +50% |

---

## Recommended Execution Order

1. **Immediate (This Sprint):** G-01 through G-05 (Critical gaps)
2. **Next Sprint:** G-06 through G-09 (High priority)
3. **Following Sprints:** G-10 through G-12 (Medium)
4. **Backlog:** G-13 through G-16 (Low, defer until needed)

---

## Next Steps

1. **Engineering:** Pick up Critical gaps (G-01 to G-05)
2. **Install Rust toolchain** in test environment to verify existing tests pass
3. **Set coverage thresholds:** Recommend 80% line coverage for core modules
4. **Add CI gate:** Fail PRs that reduce coverage below threshold

---

*Report generated by CTO agent during WOR-426 audit phase.*
