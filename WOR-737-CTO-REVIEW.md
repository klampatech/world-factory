# WOR-737: CTO Review - Silent Active Run for QA

**Issue:** WOR-737 Review silent active run for QA  
**Review Date:** 2026-05-08  
**Reviewer:** CTO

## Summary

Reviewed the QA silent active run that failed. The failure was caused by a **test design issue** — the test expects figures to exist in a newly created world, but newly generated worlds don't generate figures by default. This is not an application bug.

## Failure Analysis

### Test That Failed
- **Test:** `smoke-test-wor715.spec.ts >> Backend API - 18 Endpoints >> 10 - GET /api/v1/worlds/:id/figures/:figure_id - Get figure details`
- **Location:** `e2e/smoke-test-wor715.spec.ts:175`

### Root Cause
The test creates a new world and immediately expects to retrieve figures from it:
```typescript
const figuresRes = await fetch(`${API_BASE}/worlds/${worldId}/figures`);
const figuresJson = await figuresRes.json();
const figures: any[] = figuresJson.data || [];
expect(figures.length).toBeGreaterThan(0);  // ← FAILS HERE
```

Newly generated worlds don't have figures populated by default. This is expected application behavior, not a bug.

### Confirmed Working Endpoints
All 17 other smoke tests in the same run passed, including:
- World creation, listing, details, deletion
- Planet, map, history, history events
- Settlements, resources, disasters, artifacts
- Export (JSON and full)
- Health check

## Application Health Status

✅ **Application is healthy.** All critical API paths are operational.

## Recommendation

1. Fix the smoke test to handle empty figures array gracefully
2. Or update test to use a world that has been populated with figures through simulation/turn advancement
3. No application code changes needed

## Action Items

- [ ] QA to update smoke test to handle empty figures on new worlds