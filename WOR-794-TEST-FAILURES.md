# WOR-794 Test Failures Report

## Status: CI BLOCKED by 13 Test Failures

Build passes (PR #59), but tests fail. These 13 test failures must be fixed for CI checks to pass.

---

## Failing Tests Summary

### 1. artifacts::tests (5 failures)
- `test_causal_chain_validator_weapon`
- `test_causal_chain_validator_sacred_relic`
- `test_causal_chain_validator_crown_jewel`
- `test_causal_chain_validator_magical`
- `test_causal_chain_validator_document`

**Location:** `src/artifacts.rs:1546-1600` (approx)

### 2. beasts::slaying::tests (5 failures)
- `test_all_beasts_create_remnants`
- `test_insufficient_factions_fails`
- `test_insufficient_power_fails`
- `test_slaying_creates_remnant`
- Likely `test_remnant_decay` (line 314)

**Location:** `src/beasts/slaying.rs` tests

### 3. beasts::remnants::tests (1 failure)
- `test_remnant_decay`

### 4. faction::faction_stats_tests (3 failures)
- `test_wealth_calculation` (line 1058: assertion failed: left=36, right=41)
- `test_recalculate_stats` (line 1137: assertion failed: left=53, right=21)
- `test_is_critical` (line 1197)

**Location:** `src/faction.rs:1046-1197`

---

## Evidence

CI Run: `25580768902` (WOR-804 Branch Protection PR - also fails)
```
test result: FAILED. 426 passed; 13 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Root Cause Analysis

The test failures appear to be related to:
1. **RemnantSystem integration** - Beast slaying and remnant tests failing
2. **Faction stats calculations** - HP and wealth calculations have incorrect expected values

---

## Required Fix

A Coder agent must fix all 13 failing tests. The tests are correctly written - the implementation code needs to match the expected values.

---

## Related Issues

- WOR-712: Wire primal beast death to create Remnant artifact per SPEC.md §D.4.3
- WOR-729: Integrate RemnantSystem into World/Simulation state (WOR-712 integration)
- WOR-707: CLI generate command does not save .wfw files to storage

---

*Report prepared by QA Agent*
