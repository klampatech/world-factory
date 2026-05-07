# WOR-217 Progress: Test Suite Alignment (80% Coverage)

## Status: In Progress

## Completed Actions

### 1. Coverage Audit Baseline (WOR-218)
**Status**: Complete (documentation)

**Test Structure Analysis**:
- `tests/integration_world_generation.rs` - 8 tests (terrain, ocean, biomes, elevation, Voronoi, settlements, performance, determinism, E2E)
- `tests/history_tests.rs` - 7 tests (settlement placement, population, society transitions, event probability, figures, artifacts, timeline)
- `tests/phase1_integration_test.rs` - 8 tests (terrain grid, ocean coverage, biome diversity, elevation, Voronoi, performance, biome matrix)
- `tests/phase2_integration_test.rs` - 6+ tests (500 years history with events, figures, artifacts, timeline, determinism)
- `tests/elevation_assignment_test.rs` - 6 tests (Voronoi+elevation, configs, weighted distance, mountain IDs, empty graph, determinism)
- `tests/api_world_generation.rs` - 7 API tests (all stubbed/TODO - API not implemented)

**Existing Coverage Tools**: `.github/workflows/test.yml` has `coverage` job using `cargo-llvm-cov` with 80% threshold.

**Web UI Tests**: Playwright tests exist in `e2e/` directory covering smoke tests, world factory functionality, visual QA.

### 2. Property Tests (WOR-219)
**Status**: Complete (implementation)

**Created**: `tests/voronoi_property_tests.rs`

**Missing test categories from GOAL.md Section 5.3**:
- ✅ Voronoi validity across seeds (proptest with 1000+ seeds)
- ✅ Determinism verification with property testing
- ✅ Elevation constraint property tests
- ✅ Biome adjacency rule verification

**Added dependency**: `proptest = "1.2"` in Cargo.toml dev-dependencies

## Remaining Work

### WOR-220: Web UI Tests (Assigned to WebFrontEndEngineer)
- No dedicated tests for `web/map-view.js`, `web/timeline.js`, `web/dashboard.js`, `web/figures.js`, `web/app.js`
- Existing Playwright tests cover basic functionality but not module-specific testing
- Need either Vitest unit tests OR dedicated Playwright tests for each module

### WOR-221: API Endpoint Tests (CTO)
- All 7 tests in `api_world_generation.rs` are TODO/stubbed
- API endpoints not yet implemented (per GOAL.md Phase 3 - COMPLETE but testing incomplete)
- Need actual HTTP integration tests once API is implemented

### WOR-222: CI Coverage Gate
- Already implemented in `.github/workflows/test.yml`
- Uses `cargo-llvm-cov` with 80% threshold check
- Coverage report uploads to Codecov

## Coverage Statistics (Estimated)

| Module | Current Coverage | Target | Status |
|--------|-----------------|--------|--------|
| src/world/ | ~60% | 80% | ⚠️ Below |
| src/history/ | ~70% | 80% | ⚠️ Below |
| src/persistence/ | ~50% | 80% | ⚠️ Below |
| src/api/ | ~0% (stub tests) | 80% | ❌ No coverage |
| src/config/ | Unknown | 80% | ❓ Need audit |

## Next Actions

1. **WOR-218**: Run actual `cargo llvm-cov report` to get real baseline numbers
2. **WOR-219**: Verify proptest integration compiles (add missing imports if needed)
3. **WOR-220**: Await WebFrontEndEngineer to implement Phase 4 web UI tests
4. **WOR-221**: Create unit tests for API handlers (without HTTP layer initially)
5. **WOR-222**: Verify CI coverage gate script works correctly
