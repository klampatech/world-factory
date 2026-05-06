# WOR-218: Test Coverage Baseline Audit

**Status:** Draft
**Date:** 2026-05-06
**Agent:** CTO (ec110451-2374-4b57-ab0a-23139fcb1d01)

## Executive Summary

Audit of current test coverage baseline. Found 12 Rust test files (~90 tests), TypeScript tests, Playwright E2E tests, and Python smoke tests. Key issue: Rust toolchain not installed so integration tests cannot be executed.

## Test Inventory

### Rust Integration Tests (Cargo)

| Test File | Test Count | Status |
|-----------|------------|--------|
| `tests/api_world_generation.rs` | 7 | ⚠️ Placeholder (all TODO) |
| `tests/cli_flag_test.rs` | ~8 | Unknown |
| `tests/cli_generation_test.rs` | 16 | ✅ Implemented |
| `tests/cli_regression_test.rs` | 3 | ✅ Implemented |
| `tests/cli_server_test.rs` | 4 | ✅ Implemented |
| `tests/cli_shared_storage_test.rs` | ~3 | Unknown |
| `tests/elevation_assignment_test.rs` | 7 | ✅ Implemented |
| `tests/history_tests.rs` | 7 | ✅ Implemented |
| `tests/integration_world_generation.rs` | 10 | ✅ Implemented |
| `tests/phase1_integration_test.rs` | 7 | ✅ Implemented |
| `tests/phase2_integration_test.rs` | 3 | ✅ Implemented |
| `tests/species_template_test.rs` | 20 | ✅ Implemented |

**Rust Test Total:** ~95 tests across 12 files

### TypeScript Tests

| Test File | Test Count | Status |
|-----------|------------|--------|
| `tests/WorldModel.test.ts` | 3 | ✅ Implemented |
| `tests/worlds-api.test.ts` | ~10 | ✅ Implemented |

### Playwright E2E Tests

| Test File | Type | Status |
|-----------|------|--------|
| `e2e/world-factory.spec.ts` | Frontend UI | ✅ Implemented |
| `e2e/frontend-smoke-tests.spec.ts` | Smoke | ✅ Implemented |
| `e2e/WOR-75-smoke-test*.spec.ts` | Smoke | ✅ Implemented |
| `e2e/smoke-wor167.spec.ts` | Smoke | ✅ Implemented |
| `e2e/smoke-test-wor186.spec.ts` | Smoke | ✅ Implemented |
| `e2e/WOR-134-screenshots.spec.ts` | Visual QA | ✅ Implemented |
| `e2e/wor141-smoke-tests.spec.ts` | Smoke | ✅ Implemented |
| `smoke-test-wor179.spec.ts` | Smoke | ✅ Implemented |
| `wor-210-visual-qa.spec.ts` | Visual QA | ✅ Implemented |
| `e2e/console-errors.spec.ts` | Error detection | ✅ Implemented |

**Playwright E2E Total:** ~15 spec files

### Python Smoke Tests

| Test File | Status |
|-----------|--------|
| `api_smoke_tests.py` | ✅ Implemented |
| `e2e/frontend-smoke-tests.py` | ✅ Implemented |

## Key Finding: Rust Toolchain Missing

**Issue:** `cargo` command not found in environment

The Rust integration tests cannot be executed. This blocks:
- Running `cargo test` for unit/integration tests
- Using `cargo-tarpaulin` for coverage measurement
- Verifying determinism tests

**Recommendation:** Install Rust toolchain via rustup, or document that tests must be run in Docker container.

## Coverage Gaps Identified

### 1. API Tests (All Placeholders)
`tests/api_world_generation.rs` has 7 test functions but all are TODO stubs with comments like:
```rust
// TODO: Implement actual HTTP request when API is available
```

### 2. No Coverage Measurement
- `cargo-tarpaulin` not installed
- No coverage threshold defined
- No coverage report generation

### 3. E2E Test Execution
- Playwright tests require server running
- No CI configuration visible for automated execution
- Screenshots directory exists suggesting manual QA

## Test Execution Commands

```bash
# Rust tests (requires cargo)
cargo test --all

# TypeScript tests
npm test

# Playwright tests
npx playwright test

# Python smoke tests
python api_smoke_tests.py
```

## Next Actions

1. **Install Rust toolchain** - Required to run core integration tests
2. **Install cargo-tarpaulin** - For coverage measurement
3. **Review API test implementations** - Current tests are stubs
4. **Define coverage thresholds** - What % is acceptable?
5. **Document test execution environment** - Docker vs local

## Notes

- Most integration tests use hardcoded seeds (42, 12345) for determinism verification
- Tests span Phase 1 (terrain) through Phase 2 (history generation)
- Species template tests verify the OnlyInHistory marker trait
- E2E tests cover frontend UI, console errors, and visual QA

---

*This audit provides the baseline for WOR-218. Next steps depend on whether Rust toolchain will be installed or tests run in Docker.*