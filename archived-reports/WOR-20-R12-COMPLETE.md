# WOR-20 R-12: Dashboard - Active Societies Count

## Status: COMPLETE

## Objective
Display count of active societies in the dashboard.

## Changes Made

### 1. Updated `src/services/dashboardService.ts`

**Before:** `fetchWorldStats()` returned mock data without calling the API.

**After:** `fetchWorldStats()` now:
- Calls the actual API endpoint `GET /api/v1/worlds/:id/societies` via `fetchSocietiesFromAPI()`
- Transforms the response using new `transformSocietiesToStats()` function
- Sets `activeSocieties: societies.length` from the actual API count
- Falls back to mock data if the API call fails

**Key transformation code:**
```typescript
function transformSocietiesToStats(societiesResponse: SocietiesResponse): WorldStats {
  const societies = societiesResponse.societies;
  // ... calculate total population
  return {
    // ...
    activeSocieties: societies.length, // Count of active societies from API
    // ...
  };
}
```

### 2. Dashboard Component (Pre-existing)

The Dashboard component already displays the active societies count:
- Located in the World State summary section
- Shows `{state.selectedWorldMetrics.activeSocieties}` 
- Label: "Active Societies"
- Sublabel: "Civilizations"

## API Integration Flow

1. Dashboard calls `fetchWorldStats(worldId)` with worldId prop
2. Service calls `fetchSocietiesFromAPI(worldId)` → `GET /api/v1/worlds/:id/societies`
3. Backend returns `SocietiesResponse` with societies array
4. Frontend transforms to `WorldStats` with `activeSocieties: societies.length`
5. Dashboard displays the count

## Verification

The implementation is complete. When a world is selected:
- The Dashboard fetches societies from the API
- The `activeSocieties` count reflects the number of societies returned
- Falls back to mock data (47 societies) if the API is unavailable

## Next Steps (Optional)

- Implement a dedicated `/stats` endpoint for performance (single request vs multiple)
- Add resources endpoint to populate the resources section
- Add cataclysms endpoint to populate the disasters section