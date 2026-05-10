# WOR-952 Fix Report

## Problem

Index page timeline tab triggers double-slash API bug when `state.worldId` is null.

When navigating to the timeline tab (or other tabs) before a world ID is properly set, the frontend calls API methods like `loadTimeline()` which passes `state.worldId` to the API client. If `state.worldId` is null, the resulting API URL becomes malformed:

```
/api/v1/worlds//timeline  <-- double slash due to null worldId
```

The `normalizeWorldId()` function in `api-integration.js` returns empty string `''` when given null, but doesn't prevent the API call from being made.

## Root Cause

The `loadTimeline()`, `loadMapData()`, and `loadDashboard()` functions in both `web/index.html` and `web/world.html` lack guards against null/undefined `state.worldId` before making API calls.

## Fix Applied

Added null-check guards at the beginning of each affected function in both files:

### `web/index.html`
- `loadTimeline()` (line ~1832): Added guard before API call
- `loadMapData()` (line ~1728): Added guard before API call
- `loadDashboard()` (line ~2235): Added guard before API call

### `web/world.html`
- `loadTimeline()` (line ~1409): Added guard before API call
- `loadDashboard()` (line ~1776): Added guard before API call

## Guard Pattern

```javascript
async function loadTimeline() {
    // Guard: prevent API call with null/undefined worldId
    if (!state.worldId) {
        console.warn('Cannot load timeline: state.worldId is null');
        state.events = getDemoEvents();
        renderTimeline();
        return;
    }
    
    try {
        state.events = await api.getSimulationHistory(state.worldId);
        // ... rest of function
    }
}
```

## Verification

The guards will:
1. Log a warning when worldId is null (helps with debugging)
2. Load demo/fallback data instead of making a failed API call
3. Return early to prevent the malformed URL

## Files Changed

| File | Lines | Change |
|------|-------|--------|
| `web/index.html` | ~1832-1841 | Added guard to loadTimeline() |
| `web/index.html` | ~1728-1738 | Added guard to loadMapData() |
| `web/index.html` | ~2235-2245 | Added guard to loadDashboard() |
| `web/world.html` | ~1409-1418 | Added guard to loadTimeline() |
| `web/world.html` | ~1776-1786 | Added guard to loadDashboard() |
