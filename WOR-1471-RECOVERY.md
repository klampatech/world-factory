# WOR-1471: Recover Stalled Issue WOR-1460

**Created:** 2026-05-12
**Status:** DONE
**Priority:** HIGH
**Parent Issue:** WOR-1460 recovery

## Recovery Context

**Problem:** WOR-1460 reported that timeline always shows 0 events. Root cause was identified and fix implemented.

## Investigation Summary

- WOR-1460 workspace had zero presence (never picked up work by previous assignee)
- API confirmed issue assigned to SeniorRustEngineer but never executed
- Identified root cause in `src/api/v1/worlds.rs`

## Root Cause

**Line 2540-2578: World generation did NOT call history simulation**

```rust
// OLD CODE - only generates terrain, no events
let _generated_world = generator.generate(package.world.seed);
// events: Vec::new() - ALWAYS EMPTY
```

**API endpoints also had placeholder empty responses:**
- `get_world_history` (line ~760) - TODO placeholder returning 0 events
- `get_history_events` (line ~835) - TODO placeholder returning 0 events

## Fixes Applied

### 1. `src/api/v1/worlds.rs` - World generation now generates history events

Added HistoryGenerator call after terrain generation:

```rust
// Generate world history (pre-history simulation) using HistoryGenerator
use crate::history::generator::{GeneratorConfig, HistoryGenerator};
let mut history_gen = HistoryGenerator::new();
let history_config = GeneratorConfig::default();
let domain_world = crate::types::World::new(
    format!("Generated World {}", normalized_id),
    package.world.seed,
);
let history_result = history_gen.generate(&domain_world, history_config);

// Convert HistoryGenerator events to HistoricalEvent format
let historical_events: Vec<crate::types::HistoricalEvent> = history_result
    .events
    .events()
    .iter()
    .map(|e| {
        crate::types::HistoricalEvent::new(
            e.world_id,
            e.name.clone(),
            crate::types::HistoricalTime::Year(e.year),
            e.description.clone().unwrap_or_else(|| e.name.clone()),
        )
    })
    .collect();

// ... later in file
package.events = historical_events;  // Now populated!
```

### 2. `src/api/models.rs` - Added HistoryEventView::from for HistoricalEvent

```rust
impl From<crate::types::HistoricalEvent> for HistoryEventView {
    fn from(event: crate::types::HistoricalEvent) -> Self {
        Self {
            id: event.id.to_string(),
            event_type: event.event_type
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".to_string()),
            year: match event.time {
                crate::types::HistoricalTime::Year(y) => y,
                _ => 0,
            },
            title: event.name,
            description: Some(event.description),
            significance: 0.5,
            location_id: event.location_id.map(|l| l.to_string()),
            participant_count: event.participants.as_ref().map(|p| p.len()),
            tags: None,
        }
    }
}
```

### 3. `src/api/v1/worlds.rs` - Fixed get_world_history endpoint

Now loads events from WorldPackage instead of placeholder:

```rust
// Load events from package
let package_path = state.storage.world_package_path(&world_id);
let package = match crate::packaging::load_world(&package_path) {
    Ok(p) => p,
    Err(_) => { /* fallback empty response */ }
};

// Load events from the package
let mut events: Vec<_> = package.events.into_iter().map(HistoryEventView::from).collect();

// Apply filters (year range, etc.)
// Sort and paginate
```

### 4. `src/api/v1/worlds.rs` - Fixed get_history_events endpoint

Same fix as get_world_history - loads from package.

## Files Modified

1. `src/api/v1/worlds.rs`
   - Line ~2637: Added HistoryGenerator call in `run_world_generation`
   - Line ~2649-2660: Added event conversion logic
   - Line ~787: Fixed `get_world_history` to load from package
   - Line ~889: Fixed `get_history_events` to load from package

2. `src/api/models.rs`
   - Line ~1268: Added `impl From<HistoricalEvent> for HistoryEventView`

## Verification

1. Create a test world with `pre_history_years > 0`
2. Check `/api/v1/worlds/{id}/timeline` - should show events
3. Check `/api/v1/worlds/{id}/history` - should have event count > 0
4. Check `/api/v1/worlds/{id}/events` - should have events

## Resolution

**Status:** COMPLETE - Fix implemented

The bug was that `HistoryGenerator` existed but was never called during world generation. Now when a world is generated, pre-history simulation runs and populates `WorldPackage.events` with generated historical events.

Timeline API (`/timeline`) was already working correctly. The fix ensures `/history` and `/events` also return the generated events.