# WOR-49: Implement true river merging with flow accumulation

## Status: COMPLETE ✅

## Implementation Summary

Implemented true river merging with D8 flow accumulation in `src/hydro/rivers.rs`.

### Key Features

1. **D8 Flow Direction**: Each cell flows to its single steepest downslope neighbor (all 8 directions)
2. **Flow Accumulation**: `u32` counts representing total cells draining into each location
3. **River Merging**: Confluences detected when rivers join; flow combines downstream
4. **Confluence Tracking**: `Confluence` struct records merge points with position, IDs, flow increase

### Acceptance Criteria Met

- ✅ When tributaries join, flow accumulation is combined correctly
- ✅ River geometry reflects accurate flow volume (flow_rate increases at confluences)
- ✅ Downstream routing is consistent with accumulated flow (D8 + recalculated accumulation)

### Files Modified

- `src/hydro/rivers.rs` - Core river generation with merging

### Files Fixed (pre-existing issues)

- `src/storage.rs` - Orphaned function definition
- `src/events/probability/engine.rs` - Missing Season import

### Tests Added

- `test_river_merging_accumulation` - Verifies river merging with proper accumulation
- `test_flow_accumulation_increases_downstream` - Validates downstream flow increase

### Test Results

All 9 river tests pass:
- test_flow_direction_d8 ✓
- test_river_id ✓
- test_flow_accumulation ✓
- test_river_generation ✓
- test_confluence_tracking ✓
- test_tributary_threshold ✓
- test_deterministic_rivers ✓
- test_river_merging_accumulation ✓
- test_flow_accumulation_increases_downstream ✓