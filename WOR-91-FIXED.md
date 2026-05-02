# WOR-91: Borrow Checker Errors - RESOLVED

**Status:** Complete (all 6 errors fixed)

## Errors Fixed

### E0382 (borrow of moved value: `time`) - 1 instance
**File:** `src/events/mod.rs:223`
**Fix:** Extracted `end_year = time.get_year() + duration_years` before `time` is moved to `Self::new()`

```rust
// Before: time.clone().get_year() - clones unnecessarily and uses after move
// After:
let end_year = time.get_year() + duration_years;  // Extract year first
let mut event = Self::new(..., time);  // time moved here
event.end_time = Some(HistoricalTime::year(end_year));  // end_year used instead
```

### E0502 (cannot borrow `*self` as immutable) - 3 instances

**File:** `src/terrain/elevation.rs:499`
**Fix:** Changed `&mut self.polygons` to `& self.polygons` to avoid nested borrow with `self.get()`

**File:** `src/simulation/population.rs:570`
**Fix:** Moved `self.seeded_random()` call before mutable borrow of `state`

**File:** `src/events/probability/engine.rs:615`
**Fix:** Collect `base_probabilities.keys()` into a `Vec` before iterating and calling `calculate_event_probability()`

### E0499 (cannot borrow `*self` as mutable more than once) - 2 instances

**File:** `src/simulation/population.rs:545,551`
**Fix:** Moved `simulate_disease_outbreaks()` and `simulate_disasters()` calls before the mutable borrow of `self.settlements.get_mut(id)?`

## Verification Results

```
E0499/E0502/E0382 (borrow errors): 0 remaining ✓
E0308 (type mismatches): 7 remaining (separate issue)
```

**Status: COMPLETE** — All 6 borrow checker errors fixed as of run 332127bb.
- `ElevationGrid` vs `TerrainGrid` in generation/mod.rs
- `Vec2<f32>` vs `Vec2<i32>` in hydro/drainage_basin.rs
- `BiomeType` vs `&BiomeType` in simulation/population.rs
- `EventType` dereferencing issue in probability/engine.rs