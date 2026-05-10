# WOR-1091 QA Report: Failing Tests in CI

## Issue
Find root cause and fix failing CI tests.

## Status: **FAIL** — Compilation errors prevent any tests from running

---

## Summary

CI cannot run any tests because **two test files fail to compile**. The test code expects API methods and struct fields that don't exist in the implementation.

---

## Failing Test Files

### 1. `tests/history_tests.rs` — 7 compilation errors

**Errors:**
- `activate()` method not found on `Artifact`
- `can_activate()` method not found on `Artifact`
- `activations_used` field not found on `Artifact`

**Lines affected:** 514-529 (test_history_artifacts_creation_and_usage)

**Root cause:** The `Artifact` struct (src/artifacts.rs:951) lacks:
- `activate()` method
- `can_activate()` method
- `activations_used` field

The tests expect artifact activation functionality (MAX = 3 activations per artifact) that was never implemented.

---

### 2. `tests/api_history_figures_test.rs` — 28 compilation errors

**Errors:**
- `WorldPackage` has no field `geographies`
- `WorldPackage` has no field `event_store_events`
- `WorldPackage` has no field `notable_figures`

**Root cause:** The `WorldPackage` struct (src/packaging.rs:92) only has these fields:
- `world`, `regions`, `settlements`, `persons`, `events`, `timelines`, `terrain`

The tests expect the package to include additional data structures (`geographies`, `event_store_events`, `notable_figures`) that were never added to the struct.

---

## Evidence

```
$ docker run --rm -v $(pwd):/workspace -w /workspace world-factory:test cargo test

error[E0599]: no method named `activate` found for struct `world_factory::Artifact`
error[E0599]: no method named `can_activate` found for struct `world_factory::Artifact`
error[E0609]: no field `activations_used` on type `world_factory::Artifact`

error[E0560]: struct `WorldPackage` has no field named `geographies`
error[E0560]: struct `WorldPackage` has no field named `event_store_events`
error[E0560]: struct `WorldPackage` has no field named `notable_figures`
```

---

## Verdict

| Test File | Result | Cause |
|-----------|--------|-------|
| `history_tests.rs` | **FAIL (compile error)** | Missing `activate()`/`can_activate()` methods and `activations_used` field on `Artifact` |
| `api_history_figures_test.rs` | **FAIL (compile error)** | `WorldPackage` missing `geographies`, `event_store_events`, `notable_figures` fields |

**No tests can execute until the above compilation errors are resolved.**

---

## Required Fix (Coder Responsibility)

Two possible approaches:

### Option A: Implement the missing code
1. Add `activate()`, `can_activate()` methods and `activations_used` field to `Artifact` struct
2. Add `geographies`, `event_store_events`, `notable_figures` fields to `WorldPackage`

### Option B: Fix the tests to match implementation
1. Remove/modify tests that reference non-existent `Artifact` activation methods
2. Remove/modify tests that reference non-existent `WorldPackage` fields

**Recommendation:** Option B (test alignment) is likely faster unless these features are planned. The test file names (`api_history_figures_test`) suggest the tests were written for a future API design that never landed.

---

## Owner Assignment

This is **Coder territory** — I cannot fix implementation gaps. Please assign to a Coder agent with concrete instructions:

1. Review `src/artifacts.rs` and `src/packaging.rs`
2. Either implement missing code or update tests to match current API
3. Verify all tests compile and pass

---

*QA Report prepared by Agent d8323825-1f17-4949-9762-3f27cc831b68*
