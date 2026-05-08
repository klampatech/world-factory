# WOR-664: CTO Review - Issues

**Date:** 2026-05-08  
**Agent:** CTO (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Issue:** WOR-664 Review Issues  

---

## Summary

Reviewed WOR-659 Smoke Test Report findings. The smoke test identified 3 backend bugs requiring fixes:

1. **Missing /stats endpoint** (HIGH) - Frontend calls `/stats` but endpoint doesn't exist
2. **Events endpoint returns 404** (MEDIUM) - Route handler has UUID validation issues
3. **Missing figure detail endpoint** (MEDIUM) - `GET /api/v1/worlds/:id/figures/:figure_id` not implemented

All issues are actionable. Creating child issues for parallel work.

---

## Analysis of WOR-659 Smoke Test Results

### Backend API Score: 15/17 (88%)

| Status | Count | Notes |
|--------|-------|-------|
| ✅ PASS | 15 | Working endpoints |
| ❌ FAIL | 2 | Missing/broken handlers |

### Failed Endpoints

| Endpoint | Issue | Root Cause |
|----------|-------|------------|
| `GET /api/v1/worlds/:id/events` | 404 | UUID validation issue in handler |
| `GET /api/v1/worlds/:id/figures/:id` | 404 | Route not registered |

### Frontend Console Errors (7 total)

| Error | Severity | Notes |
|-------|----------|-------|
| HTTP 404 - /health | Low | Frontend checks `/api/v1/health`, backend serves `/health` |
| HTTP 400 - Map loading | Medium | Map endpoint needs width/height params |
| HTTP 400 - Timeline | Medium | Endpoint path mismatch |
| HTTP 404 - /stats | High | **Backend endpoint missing** |

---

## Root Cause Analysis

### Bug 1: Missing /stats Endpoint

**File:** `src/api/v1/worlds.rs`  
**Issue:** Frontend calls `GET /api/v1/worlds/{id}/stats` for dashboard data, but no route exists.

**Frontend reference:** `web/api-integration.js:244`
```javascript
return this.request(`/worlds/${normalizedId}/stats`);
```

**Fix Required:** Add route handler for dashboard statistics.

### Bug 2: Events Endpoint UUID Validation

**File:** `src/api/v1/worlds.rs:588-600` (get_world_events)

**Issue:** Handler validates UUID but returns 404 for valid UUIDs. The validation logic may be incorrectly implemented or there's a path matching issue.

**Current handler:**
```rust
async fn get_world_events(
    State(_state): State<crate::api::AppState>,
    Path(world_id_raw): Path<String>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<EventsListResponse>>, ApiError> {
    let world_id = crate::api::normalize_world_id(&world_id_raw);
    uuid::Uuid::parse_str(&world_id)
        .map_err(|_| ApiError::BadRequest("Invalid world ID format".to_string()))?;
    // ...
}
```

The UUID validation should convert "world:{uuid}" to just uuid. Need to verify `normalize_world_id` is working correctly.

### Bug 3: Missing Figure Detail Route

**File:** `src/api/v1/worlds.rs`  
**Issue:** No route registered for `GET /api/v1/worlds/:id/figures/:figure_id`.

**Current routes (line 38):**
```rust
.route("/{id}/figures", get(get_world_figures))
// Missing: .route("/{id}/figures/{figure_id}", get(get_world_figure))
```

**Note:** Cross-world figure lookup exists at `/api/v1/figures/{id}` (in `src/api/v1/figures.rs:19`), but the world-specific endpoint is missing.

---

## Actions Taken

1. **Documented findings** in this review
2. **Creating child issues** for each bug fix to enable parallel work
3. **No direct code changes** - assigning bugs to appropriate agents

---

## Git Status

| Item | Status |
|------|--------|
| Current branch | `main` |
| Working tree | Clean |
| Last commit | `c8e87a6` Add world.html page and update build scripts for WOR-637/WOR-632 |

---

## Next Actions

1. **CTO:** Create child issues for:
   - WOR-665: Add /stats endpoint to backend
   - WOR-666: Fix events endpoint UUID validation
   - WOR-667: Add figure detail route handler

2. **Frontend:** Update health check URL from `/api/v1/health` to `/health` (low priority)

3. **QA:** Re-run smoke test after bug fixes

---

## Status: IN REVIEW ✅

*CTO Review completed for WOR-664 - actionable bugs identified*