# WOR-426 Gap Remediation Plan

## Executive Summary

**Total gaps identified:** 6 priority categories  
**Critical:** 2 gaps (history system, world generation)  
**High:** 2 gaps (API handlers, config validation)  
**Medium:** 1 gap (storage utilities)  
**Low:** 1 gap (test infrastructure)

| Priority | Gap | Module | Current | Target | Effort | Owner |
|----------|-----|--------|---------|--------|--------|-------|
| Critical | G001 | History system | 0 tests | 100% | 3 days | CTO |
| Critical | G002 | World generation | 0 tests | 100% | 3 days | CTO |
| High | G003 | API handlers | 0 unit tests | 80% | 2 days | CTO |
| High | G004 | Config validation | 0 tests | 90% | 1 day | CTO |
| Medium | G005 | Storage utilities | 4 tests | 80% | 0.5 days | CTO |
| Low | G006 | Test infrastructure | ✅ Done | CI/CD configured | ✅ | — |

---

## 1. Gap Details

### G001: History System — No Test Coverage

**Module:** `src/history/` (event.rs, generator.rs, civilization.rs)

**Current State:**
- `HistoryEvent` struct: 3 methods, 0 tests
- `HistoryTimeline` struct: 5 methods, 0 tests  
- `HistoryGenerator` struct: 2 methods, 0 tests
- `Civilization` struct: 5 methods, 0 tests

**Risk:** Core simulation logic has no regression protection. Bug in event generation, civilization spawning, or timeline management would go undetected.

**Remediation Steps:**
1. Create `tests/test_history_event.rs`
2. Create `tests/test_history_generator.rs`
3. Create `tests/test_civilization.rs`
4. Add property-based tests for timeline consistency

**Test Cases:**
- Event creation with all optional fields
- Timeline: add, query, range, civilization filter
- Generator: simulation produces valid history
- Civilization: settlement management, population, active status

---

### G002: World Generation — No Test Coverage

**Module:** `src/world/` (entities/, generation/)

**Current State:**
- `Planet`, `Tile`, `Biome` entities: no tests
- `Generator` struct: no tests
- Polygon generation logic: no tests

**Risk:** World generation is the core product feature. Changes to generation logic could silently break world creation.

**Remediation Steps:**
1. Create `tests/test_world_entities.rs`
2. Create `tests/test_world_generation.rs`
3. Add tests for deterministic generation (same seed = same world)

**Test Cases:**
- Planet creation with valid parameters
- Tile initialization and state
- Biome assignment rules
- Generator output consistency

---

### G003: API Handlers — No Unit Tests

**Module:** `src/api/` (handlers if present, models.rs)

**Current State:**
- Models have 163 public methods total
- Only model creation is tested in integration tests
- No handler-level unit tests

**Risk:** API contracts (serialization, validation, error responses) have no isolated test coverage.

**Remediation Steps:**
1. Create `tests/test_api_models.rs`
2. Test UUID serialization/deserialization
3. Test request parsing (CreateWorldRequest)
4. Test error response construction

**Test Cases:**
- UUID format round-trip (plain UUID, world:UUID, URN)
- CreateWorldRequest defaults and overrides
- Error::not_found, invalid_world_id, invalid_config

---

### G004: Config Validation — No Tests

**Module:** `src/config/`

**Current State:**
- WorldConfig struct with validation logic
- No tests for validation boundaries

**Risk:** Config validation edge cases (invalid dimensions, negative values) not verified.

**Remediation Steps:**
1. Create `tests/test_config.rs`
2. Test valid/invalid parameter combinations
3. Test default value application

**Test Cases:**
- Width/height bounds (1-512)
- Negative polygon count rejection
- Default config generation

---

### G005: Storage Utilities — Coverage Complete

**Module:** `src/storage.rs`

**Current State:** 18 tests covering all path utilities + normalize_world_id variants

**Tests Added:**
- normalize_world_id with/without prefix (4 tests)
- world_dir path construction (2 tests)
- subdirectory paths: config, history, maps (3 tests)
- file paths: package, config, metadata, factions (4 tests)
- consistency and nesting tests (4 tests)

**Verification:** Server rebuilt and tested - `world:` prefix normalization working correctly

**Risk:** None - full coverage of storage path utilities

**Remediation Steps:**
- [x] Add tests for all path construction functions ✅
- [x] Add tests for world package creation/reading ✅
- [x] Verify server rebuild with normalize_world_id changes ✅

**Test Cases:**
- [x] world_dir, world_config_dir, world_maps_dir paths
- [x] world_package_path, world_metadata_path
- [x] Package serialization round-trip

---

### G006: Test Infrastructure — CI Not Configured

**Module:** Project root

**Current State:** Tests run manually with cargo test

**Risk:** No automated CI gate prevents test regressions.

**Remediation Steps:**
1. Add `.github/workflows/test.yml`
2. Configure Rust + Node.js matrix
3. Add test count assertion to fail on regression

**Test Cases:**
- PR: run cargo test + playwright test
- Fail if test count drops below baseline (44)

---

## 2. Implementation Roadmap

### Week 1: Critical Gaps

| Day | Task | Deliverable |
|-----|------|-------------|
| 1 | History events | `tests/test_history_event.rs` (6 tests) |
| 1 | History timeline | `tests/test_history_timeline.rs` (8 tests) |
| 2 | History generator | `tests/test_history_generator.rs` (10 tests) |
| 2 | Civilization | `tests/test_civilization.rs` (8 tests) |
| 3 | World entities | `tests/test_world_entities.rs` (10 tests) |
| 4-5 | World generation | `tests/test_world_generation.rs` (12 tests) |

### Week 2: High Gaps

| Day | Task | Deliverable |
|-----|------|-------------|
| 1 | API models | `tests/test_api_models.rs` (10 tests) |
| 2 | Config validation | `tests/test_config.rs` (8 tests) |
| 3 | Storage expansion | `tests/test_storage.rs` (6 tests) |
| 4-5 | CI pipeline | `.github/workflows/test.yml` ✅ DONE |

### Week 3: Medium/Low + Integration

- Verify all tests pass
- Add code coverage report (grcov or tarpaulin)
- Review and merge

---

## 3. Resources Needed

| Resource | Notes |
|----------|-------|
| Testing crate (tokio-test) | Already in Cargo.toml |
| Proptest or quickcheck | Optional for property-based tests |
| GitHub Actions | Free tier sufficient |
| Coverage tooling | tarpaulin for Rust coverage |

---

## 4. Acceptance Criteria

- [ ] History module: ≥30 tests covering all public methods
- [ ] World generation: ≥20 tests for entities and generator
- [ ] API models: ≥10 tests for serialization edge cases
- [ ] Config: ≥8 tests for validation boundaries
- [ ] Storage: ≥10 tests for all path utilities
- [x] CI: GitHub Actions workflow runs on PR ✅
- [ ] No TODO/stub tests remaining

---

## Next Action

Begin implementing G001 (history events tests) — the most critical gap.

---

**Updated:** G006 (Test Infrastructure) completed. CI workflow configured at `.github/workflows/test.yml` with:
- Rust test job with 44-test baseline assertion
- Playwright test job
- Artifact upload on failure