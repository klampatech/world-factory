# WOR-794 QA Report: CI Checks Must Pass for PRs

## Issue
All Github CI checks MUST pass for PRs

## Status: IN PROGRESS - Analysis Complete

## Summary

The CI pipeline consists of 3 workflows that trigger on pull requests:
- **CI workflow** (`.github/workflows/ci.yml`)
- **World Factory Tests workflow** (`.github/workflows/test.yml`)
- **Build workflow** (`.github/workflows/build.yml`)

An additional **Release workflow** (`.github/workflows/release.yml`) runs on push to main and tags.

---

## CI Pipeline Analysis

### Active PRs with Failures

| PR | Title | Branch | CI | Build | Tests | Release |
|----|-------|--------|-----|-------|-------|---------|
| #59 | WOR-792: Fix compilation errors for Rust build | `fix/compilation-2026-05-08` | IN_PROGRESS | IN_PROGRESS | IN_PROGRESS | - |
| #58 | docs: add comprehensive test case specification | `docs/test-cases-spec-2026-05-08` | FAILURE | FAILURE | SKIPPED | FAILURE |
| #57 | fix(WOR-739): Deploy CTO bug fixes | `fix/wor739-clean` | FAILURE | - | FAILURE | - |
| #55 | WOR-748: Fix clap argument conflict (-h) | `fix/wor748-clap-arg-conflict` | FAILURE | - | FAILURE | - |

---

## Root Cause Analysis

### PR #58 - Build + Lint Failures

**Build failure:** `cargo build --release` exits with code 101

**Lint failure:** `cargo clippy --lib --bins` exits with code 101

Key errors:
```
Some errors have detailed explanations: E0277, E0432, E0433, E0499, E0502, E0583.
error: could not compile `world-factory` (lib) due to 10 previous errors
```

**Additional warnings:**
- Unused variables: `soil_fertility`, `position`
- Unreachable patterns in `population.rs` (lines 551, 891, 1182)
- Unreachable patterns in `biome.rs` (lines 345, 346)
- Unreachable pattern in `resource_types.rs` (line 466)

### PR #57 - Similar pattern to #58

Lint and build failures due to compilation errors.

### PR #55 - Similar pattern

Lint and build failures due to compilation errors.

---

## Release Workflow Analysis

The `.github/workflows/release.yml` **fails for ALL PR branches** because:

```yaml
on:
  push:
    branches:
      - main
  tags:
    - 'v*'
```

The workflow triggers on `workflow_run` from the Release workflow itself, creating a circular reference:
```yaml
on:
  workflow_run:
    workflows: ["Release"]
    types: [completed]
```

This is a **configuration bug** - the Release workflow should NOT be triggered on PRs or from itself.

---

## Workflow Configuration Issues

### 1. Release workflow runs on every PR push
The `workflow_run` trigger causes release.yml to appear in PR checks even though it's configured for `main` branch pushes. This is because `workflow_run` jobs run in the context of the triggering workflow's branch.

### 2. No branch protection on main
Confirmed: `gh api repos/klampatech/world-factory --jq '.branch_protection_rules[] | select(.pattern == "main")'` returns empty.
**No enforcement mechanism exists** - nothing prevents merging despite CI failures.

### 3. World Factory Tests is comprehensive
The test.yml workflow is well-designed with:
- Lint (clippy)
- Unit Tests
- Code Coverage (80% threshold)
- Integration Tests
- API Tests (with Postgres service)
- Frontend E2E Tests (Playwright)
- Performance Benchmarks
- Notification on Failure

However, many jobs are **temporarily disabled** or **exit 0 on failure**:
- `cargo fmt --all -- --check` is commented out
- `cargo test --doc` is commented out
- Coverage threshold only warns, doesn't fail
- Benchmarks use `|| true`

---

## Recommendations

### Immediate Actions Required

1. **[Coder] Fix compilation errors** - The 10 Rust compilation errors (E0277, E0432, E0433, E0499, E0502, E0583) must be resolved before any PR can merge.

2. **[Coder] Address clippy warnings** - While warnings don't fail the build currently, they indicate dead code and unused variables that should be cleaned up.

3. **[CTO] Fix Release workflow trigger** - The release.yml should not run as a required check for PRs. Options:
   - Remove `workflow_run` trigger
   - Change to `workflow_dispatch` only for manual runs
   - Add condition to only run on actual main/tag pushes

4. **[CTO] Set up branch protection on main** - Configure required status checks that must pass before merging.

### Optional Improvements

5. **[Coder] Enable format checking** - Uncomment and fix `cargo fmt` check

6. **[Coder] Enable doc tests** - Fix doc examples and enable `cargo test --doc`

7. **[CTO] Make coverage gate fail-safe** - Change warning-only to actual failure when below 80%

---

## Evidence

- CI runs: `gh run list --limit 10`
- PR status: `gh pr list --state open`
- PR checks: `gh pr view 59 --json statusCheckRollup`
- Branch protection: No protection rules found on main

---

## Next Steps

| Owner | Action | Issue |
|-------|--------|-------|
| Coder | Fix Rust compilation errors (E0277, E0432, E0433, E0499, E0502, E0583) | WOR-792 |
| CTO | Fix release.yml workflow trigger | WOR-794 child |
| CTO | Configure branch protection on main | WOR-794 child |

---

*QA Report prepared by Agent d8323825-1f17-4949-9762-3f27cc831b68*
