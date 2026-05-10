# Priority Fix 1: Regression in Faction/Beasts Tests

> **Issue:** 8 tests failing in `beasts::slaying` and `faction::faction_stats_tests`
> **Severity:** HIGH - blocks Phase 5 progress
> **Test output:** `cargo test --lib` → 435 passed, **8 FAILED**

---

## Failing Tests

### beasts::slaying (4 failures)
```
test beasts::slaying::tests::test_slaying_creates_remnant ... FAILED
test beasts::slaying::tests::test_insufficient_factions_fails ... FAILED
test beasts::slaying::tests::test_insufficient_power_fails ... FAILED
test beasts::slaying::tests::test_all_beasts_create_remnants ... FAILED
```

### beasts::remnants (1 failure)
```
test beasts::remnants::tests::test_remnant_decay ... FAILED
```

### faction::faction_stats_tests (3 failures)
```
test faction::faction_stats_tests::hp_mechanics::test_recalculate_stats ... FAILED
test faction::faction_stats_tests::hp_mechanics::test_is_critical ... FAILED
test faction::faction_stats_tests::stat_calculations::test_wealth_calculation ... FAILED
  left: 36, right: 41
```

---

## Root Cause Analysis

The `test_wealth_calculation` failure shows a **stat calculation mismatch**: expected 41, got 36.
This indicates a recent change affected how `Wealth` (or related `f`, `c`, `w` stats) are computed,
which cascades into `HP` calculations (`MaxHP = 10 + (f + c + w) / 3`).

The slaying and remnant tests depend on:
1. Correct faction stat calculations
2. Remnant drop mechanics
3. Slaying requirement checks (3+ factions, sufficient power, legendary artifacts)

---

## Required Fixes

### Fix A: `src/faction.rs` - Stat Calculations

Check `faction.rs:1063` and the `wealth_calculation` test. The test expects wealth = 41 but gets 36.
Review the wealth formula and all callers — likely a recent change to `calculate_wealth()` or
an upstream dependency changed the stat inputs.

Also review:
- `recalculate_stats()` — may have changed HP calculation formula
- `test_is_critical` — likely uses `is_critical` threshold that's affected by stat changes

**Files to examine:**
- `src/faction.rs` (line ~1063, `stat_calculations` test module, `hp_mechanics` test module)

### Fix B: `src/beasts/slaying.rs` - Slaying Requirements

Check that `BeastSlayingRequirements` struct correctly enforces:
- Minimum 3 cooperating factions required
- Minimum combined power threshold per beast
- Legendary artifact requirement per beast element

The tests `test_insufficient_factions_fails`, `test_insufficient_power_fails`, and
`test_all_beasts_create_remnants` are failing, suggesting the requirements check logic
or the test setup changed.

**Files to examine:**
- `src/beasts/slaying.rs` - `BeastSlayingRequirements`, `SlayingParticipant`
- `src/beasts/remnants.rs` - `test_remnant_decay`

### Fix C: `src/beasts/remnants.rs` - Remnant Decay

`test_remnant_decay` is failing. Check the decay rate formula and ensure:
- Decay is proportional to world age
- Remnants don't decay below a minimum threshold
- The test's expected values match the actual decay calculation

**Files to examine:**
- `src/beasts/remnants.rs` - `RemnantSystem`, `RemnantArtifact`, `EffectIntensity`

---

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| 1 | `cargo test --lib` → 435 passed, 0 failed | Run full test suite |
| 2 | `test_wealth_calculation` passes with exact expected values | Verify wealth = 41 |
| 3 | `test_recalculate_stats` passes | HP recalculates correctly after stat changes |
| 4 | `test_is_critical` passes | Critical threshold logic correct |
| 5 | `test_slaying_creates_remnant` passes | Slaying produces RemnantArtifact |
| 6 | `test_insufficient_factions_fails` passes | 2 factions rejected, 3+ accepted |
| 7 | `test_insufficient_power_fails` passes | Below-threshold power rejected |
| 8 | `test_all_beasts_create_remnants` passes | Each of 4 beasts drops correct remnant type |
| 9 | `test_remnant_decay` passes | Decay formula produces expected values |
| 10 | No new warnings introduced | `cargo clippy` clean |

---

## Notes

- Do NOT simplify tests to make them pass — fix the implementation
- The stat calculation changes may be intentional improvements that need the test expectations updated, OR accidental regressions. Verify by checking `git log` on the relevant files
- Run `cargo test --lib beasts --lib faction --lib` to isolate these modules