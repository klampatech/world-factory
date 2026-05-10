# WOR-860 WOR-855: QA REPORT - WorldApiClient sends world ID with prefix causing 400 errors

**Issue Type:** Bug - API Integration / World ID Normalization
**Priority:** High
**Status:** ❌ FAIL - Issue identified and documented

---

## Executive Summary

The `WorldApiClient` class correctly uses `normalizeWorldId()` to strip the `world:` prefix from world IDs before making API calls. However, **several other frontend API clients do NOT normalize the world ID**, causing HTTP 400 errors when world IDs with the `world:` prefix are passed.

---

## Root Cause Analysis

### 1. WorldApiClient (web/api-integration.js) - ✅ CORRECT

```javascript
// Line 157: Correctly normalizes worldId
async getWorld(worldId) {
    const normalizedId = normalizeWorldId(worldId);
    return this.request(`/worlds/${normalizedId}`);
}
```

All 12+ methods in `WorldApiClient` use `normalizeWorldId()` before building API endpoints.

---

### 2. TimelineApiClient (src/events/TimelineApiClient.ts) - ❌ MISSING NORMALIZATION

**Line 73:** `getTimeline()` method:
```typescript
const response = await fetch(
  `${this.baseUrl}/worlds/${worldId}/timeline?${params.toString()}`,
  // worldId is NOT normalized - if it has 'world:' prefix, causes 400
```

**Line 126:** `getEvents()` method:
```typescript
const response = await fetch(
  `${this.baseUrl}/worlds/${request.worldId}/events?${params.toString()}`,
  // request.worldId is NOT normalized
```

---

### 3. SocietyList (src/components/SocietyList.tsx) - ❌ MISSING NORMALIZATION

**Line 75:**
```typescript
const response = await fetch(`${apiBase}/worlds/${worldId}/societies`);
// worldId is NOT normalized
```

---

### 4. dashboardService (src/services/dashboardService.ts) - ❌ MISSING NORMALIZATION

**Line 65:**
```typescript
const societiesResponse = await fetch(`${API_BASE}/worlds/${worldId}/societies`);
```

**Line 68:**
```typescript
const mapResponse = await fetch(`${API_BASE}/worlds/${worldId}/map`);
```

**Line 71:**
```typescript
const planetResponse = await fetch(`${API_BASE}/worlds/${worldId}/planet`);
```

All three endpoints use un-normalized `worldId`.

---

## Verification Evidence

### World ID Format Issue

The backend `normalize_world_id()` function (src/api/mod.rs:104):
```rust
pub fn normalize_world_id(id: &str) -> String {
    if id.starts_with("world:") {
        id.strip_prefix("world:").unwrap_or(id).to_string()
    } else {
        id.to_string()
    }
}
```

The backend expects UUIDs WITHOUT the `world:` prefix. When the prefix is present:
- Backend UUID parsing (`uuid::Uuid::parse_str()`) fails
- Returns HTTP 400 Bad Request

### Where world IDs Get Prefixed

The backend returns world IDs with `world:` prefix in API responses:
- `POST /api/v1/worlds` returns `id: "world:a0286f51-..."`
- `GET /api/v1/worlds` returns `worlds[]` with `world:` prefixed IDs

Frontend code that extracts `worldId` from these responses and passes it directly to API calls (without stripping the prefix) will receive 400 errors.

---

## Affected Code Locations

| File | Method/Function | Line | Status |
|------|-----------------|------|--------|
| `src/events/TimelineApiClient.ts` | `getTimeline()` | 73 | ❌ Missing normalizeWorldId |
| `src/events/TimelineApiClient.ts` | `getEvents()` | 126 | ❌ Missing normalizeWorldId |
| `src/components/SocietyList.tsx` | `fetchSocieties()` | 75 | ❌ Missing normalizeWorldId |
| `src/services/dashboardService.ts` | `fetchWorldStats()` | 65, 68, 71 | ❌ Missing normalizeWorldId |

---

## Repro Steps

1. Create a new world via API
2. Backend returns world with `id: "world:a0286f51-..."`
3. Frontend extracts this ID and calls `TimelineApiClient.getTimeline(worldId)` 
4. Request goes to `/api/v1/worlds/world:a0286f51-.../timeline`
5. Backend UUID validation fails → Returns HTTP 400

**Expected:** Request to `/api/v1/worlds/a0286f51-.../timeline`
**Actual:** Request to `/api/v1/worlds/world:a0286f51-.../timeline` → 400 error

---

## Fix Instructions for Coder

### Step 1: Add normalizeWorldId utility function

Create or export a shared normalization function. Options:

**Option A:** Import from existing utility if available
**Option B:** Add to each file:
```typescript
function normalizeWorldId(worldId: string): string {
    return worldId.replace(/^world:/, '');
}
```

### Step 2: Fix TimelineApiClient (src/events/TimelineApiClient.ts)

**Line 73:** Change to:
```typescript
const normalizedWorldId = normalizeWorldId(worldId);
const response = await fetch(
  `${this.baseUrl}/worlds/${normalizedWorldId}/timeline?${params.toString()}`,
```

**Line 126:** Change to:
```typescript
const normalizedWorldId = normalizeWorldId(request.worldId);
const response = await fetch(
  `${this.baseUrl}/worlds/${normalizedWorldId}/events?${params.toString()}`,
```

### Step 3: Fix SocietyList (src/components/SocietyList.tsx)

**Line 75:** Change to:
```typescript
const normalizedWorldId = normalizeWorldId(worldId);
const response = await fetch(`${apiBase}/worlds/${normalizedWorldId}/societies`);
```

### Step 4: Fix dashboardService (src/services/dashboardService.ts)

**Lines 65, 68, 71:** Change to:
```typescript
const normalizedWorldId = normalizeWorldId(worldId);
const societiesResponse = await fetch(`${API_BASE}/worlds/${normalizedWorldId}/societies`);
const mapResponse = await fetch(`${API_BASE}/worlds/${normalizedWorldId}/map`);
const planetResponse = await fetch(`${API_BASE}/worlds/${normalizedWorldId}/planet`);
```

---

## QA Verification Plan

After fix is applied:

1. **Unit Test:** Verify `normalizeWorldId()` strips `world:` prefix
2. **Integration Test:** Create world → Extract ID → Call all 4 affected endpoints → Verify 200 responses
3. **Smoke Test:** Full e2e test with world ID prefix scenario

---

## Test Case IDs

- TC-WOR-860-01: TimelineApiClient.getTimeline() with world:-prefixed ID → 200 OK
- TC-WOR-860-02: TimelineApiClient.getEvents() with world:-prefixed ID → 200 OK
- TC-WOR-860-03: SocietyList.fetchSocieties() with world:-prefixed ID → 200 OK
- TC-WOR-860-04: dashboardService.fetchWorldStats() with world:-prefixed ID → 200 OK

---

## Report Generated

- **Date:** 2026-05-09
- **QA Agent:** d8323825-1f17-4949-9762-3f27cc831b68
- **Issue:** WOR-860 WOR-855

---

## Action Required

Coder to add `normalizeWorldId()` calls to the 4 affected locations before building API URLs. Return to QA for verification after fix applied.
