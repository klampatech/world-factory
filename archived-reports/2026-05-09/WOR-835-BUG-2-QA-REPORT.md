# WOR-835-BUG-2 QA Report: ReferenceError - api is not defined

## Issue Summary
- **Ticket:** WOR-835-BUG-2
- **Severity:** HIGH
- **Status:** Under Investigation

## Reported Error
```
ReferenceError: api is not defined
  - loadMapData (map tab)
  - loadTimeline (timeline tab)
  - loadDashboard (dashboard tab)
```

## Investigation

### 1. API Initialization Analysis

**Finding:** The `api` global variable IS correctly defined in `web/api-integration.js`:
```javascript
// Line 377-378 in web/api-integration.js
const api = new WorldApiClient();
window.api = api; // Make globally accessible for HTML script inclusion
```

**Script loading order in `web/world.html` (line 1035):**
```html
<script src="api-integration.js"></script>
```
✅ API integration script loads before the inline world detail script

### 2. Missing API Methods Analysis

The smoke test calls these methods that are **NOT defined** in `WorldApiClient`:

| Method Called | Defined in WorldApiClient? | Location |
|--------------|---------------------------|----------|
| `api.getSocieties()` | ❌ NO | Used in dashboard.js:37 |
| `api.getFigures()` | ❌ NO | Used in dashboard.js:38 |
| `api.getHistoryEvents()` | ❌ NO | Used in api.js:17 |
| `api.exportWorld()` | ❌ NO | Used in api.js:21 |

**Defined methods:**
- `getWorld()` ✅
- `getWorldMap()` ✅
- `getSimulationHistory()` ✅
- `getDashboardStats()` ✅
- `simulate()` ✅

### 3. Error Message Discrepancy

The reported error "api is not defined" suggests a **ReferenceError** (variable doesn't exist). However, if `api` exists but `api.getSocieties()` doesn't, the error would be "api.getSocieties is not a function".

**Possible scenarios:**
1. **Race condition**: JavaScript executes before `api-integration.js` fully loads (unlikely given script tag)
2. **Different code version**: Smoke test ran against older/different code
3. **Minification issue**: `api` variable was stripped during minification
4. **Module scope**: Code running in a different scope than expected

### 4. Code Quality Issues Found

#### Issue A: Missing API Methods in WorldApiClient
The class is missing methods that other parts of the codebase expect:
```javascript
// Expected but missing:
api.getSocieties(worldId)
api.getFigures(worldId)
api.getHistoryEvents(worldId, page)
api.exportWorld(worldId)
```

#### Issue B: api.js Wrapper Uses Missing Functions
In `web/js/api.js`, methods reference non-existent API methods:
```javascript
const getSocieties = (worldId) => api.getSocieties(worldId); // api.getSocieties doesn't exist!
const getFigures = (worldId) => api.getFigures(worldId);     // api.getFigures doesn't exist!
```

## Root Cause Assessment

**Primary Issue:** Missing method implementations in `WorldApiClient` class.

**Secondary Issue:** The "api is not defined" error may indicate:
- Either a timing/loading issue with script execution
- Or the smoke test captured an earlier version of the bug where the global wasn't properly initialized

## Resolution Path

### Option 1: Add Missing Methods to WorldApiClient
Add these methods to `web/api-integration.js`:

```javascript
async getSocieties(worldId) {
    const normalizedId = normalizeWorldId(worldId);
    return this.request(`/worlds/${normalizedId}/societies`);
}

async getFigures(worldId) {
    const normalizedId = normalizeWorldId(worldId);
    return this.request(`/worlds/${normalizedId}/figures`);
}

async getHistoryEvents(worldId, page = 1) {
    const normalizedId = normalizeWorldId(worldId);
    return this.request(`/worlds/${normalizedId}/history/events?page=${page}`);
}

async exportWorld(worldId) {
    const normalizedId = normalizeWorldId(worldId);
    return this.request(`/worlds/${normalizedId}/export`);
}
```

### Option 2: Verify Script Loading Order
Ensure `api-integration.js` is loaded before any code that references `window.api`.

### Option 3: Add Defensive Checks
Wrap API calls with existence checks:
```javascript
const api = window.api || { getSocieties: () => Promise.resolve([]) };
```

## Verification Plan

1. **Static Analysis:** Check if all API methods called elsewhere are defined
2. **Runtime Test:** Load world.html and check console for errors
3. **Method Test:** Call each missing method and verify response

## Impact Assessment

- **Map Tab:** ❌ BROKEN - uses missing `api` reference
- **Timeline Tab:** ✅ WORKS - `api.getSimulationHistory()` is defined
- **Dashboard Tab:** ❌ BROKEN - uses missing `api.getSocieties()` and `api.getFigures()`

## QA Recommendation

1. Add missing methods to WorldApiClient
2. Add defensive null checks for missing methods
3. Verify script loading order in production builds
4. Re-run smoke test to confirm fix

---
**Reported:** 2026-05-09  
**Investigated by:** QA Agent  
**Status:** Awaiting Fix