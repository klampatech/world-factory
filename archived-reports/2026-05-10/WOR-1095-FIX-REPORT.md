# WOR-1095 Fix Report: Hardcoded Non-Existent World ID in Frontend

## Issue Summary

**Severity:** High  
**Component:** Frontend  
**Status:** Fixed

### Problem
Frontend polling loop continuously failed with HTTP 404 when world ID didn't exist, generating 37+ console errors.

### Root Cause Analysis

The polling mechanism in `startPolling()` would:
1. Make API call `api.getWorld(state.worldId)`
2. On error, log to console and continue
3. Retry every 2 seconds indefinitely

This caused:
- 37+ console errors over ~74 seconds
- Degraded user experience
- Unnecessary server load

### Solution Implemented

Added error handling to stop polling when API calls fail:

**web/index.html** (lines ~2288-2309):
```javascript
function startPolling() {
    if (state.pollingInterval) return;
    if (!state.worldId) return;
    
    state.pollingInterval = setInterval(async () => {
        try {
            state.world = await api.getWorld(state.worldId);
            renderWorldMetadata();
            
            if (['ready', 'error'].includes(state.world.status.phase)) {
                stopPolling();
            }
        } catch (error) {
            console.error('Polling failed:', error);
            // Stop polling on error to prevent repeated failed API calls
            stopPolling();  // <-- NEW
        }
    }, 2000);
}
```

**web/world.html** (lines ~1831-1851):
```javascript
function startPolling() {
    if (state.pollingInterval) return;
    if (!state.worldId) return;  // <-- NEW guard
    
    state.pollingInterval = setInterval(async () => {
        try {
            state.world = await api.getWorld(state.worldId);
            renderWorldMetadata();
            
            if (['ready', 'error'].includes(state.world.status?.phase)) {
                stopPolling();
            }
        } catch (error) {
            console.error('Polling failed:', error);
            // Stop polling on error to prevent repeated failed API calls
            stopPolling();  // <-- NEW
        }
    }, 2000);
}
```

## Files Modified

| File | Changes |
|------|---------|
| `web/index.html` | +2 lines (stop polling on catch) |
| `web/world.html` | +4 lines (guard + stop on catch) |

## Impact

- **Before:** 37+ console errors per polling cycle
- **After:** Maximum 1 error on first failure, then polling stops

## Commit

```
c45ae4d WOR-1095: Stop polling on API errors to prevent console spam
```

Branch: `wor-1085-ctoreview-20260510`

## Note on Demo UUIDs

The demo world UUIDs (`b9aea887-f2de-4c2d-800d-be9f25362caa`, etc.) in `getDemoWorlds()` and `getDemoWorld()` are **intentional** - they're fallback data for offline/demo mode, not the root cause of the 404 errors. The actual issue was the polling loop not stopping on failure.