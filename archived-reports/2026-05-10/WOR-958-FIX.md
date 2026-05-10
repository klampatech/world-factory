# WOR-958: Timeline `state.events.sort is not a function` Fix

## Issue
Timeline JS crash when world is still `generating` - `state.events.sort is not a function`

## Root Cause
`api.getSimulationHistory()` returns an `ApiResponse` wrapper object:
```json
{ "success": true, "data": { "events": [...], ... } }
```

But `loadTimeline()` was assigning the entire wrapper to `state.events`:
```javascript
state.events = await api.getSimulationHistory(state.worldId);
```

Then calling `.sort()` on this object throws `TypeError` because objects don't have a `.sort()` method.

## Fix Applied

### web/world.html (line ~1417)
```diff
- state.events = await api.getSimulationHistory(state.worldId);
+ const response = await api.getSimulationHistory(state.worldId);
+ state.events = response?.data?.events || [];
```

### web/index.html (line ~1841)
Same fix applied.

### web/js/timeline.js (TimelineComponent.load)
```diff
- this.state.events = await api.getSimulationHistory(this.worldId);
+ const response = await api.getSimulationHistory(this.worldId);
+ this.state.events = response?.data?.events || [];
```

Also corrected `.sort()` from `b.tick - a.tick` to `b.year - a.year` to match `HistoryEventView.year` field from API.

## Branch
`fix/WOR-958-timeline-sort-error` - pushed to origin

## PR
https://github.com/klampatech/world-factory/pull/new/fix/WOR-958-timeline-sort-error

## Status
Fix applied, committed, and pushed. Ready for review.
