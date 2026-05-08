# WOR-663 Bug Fix Report: Figure Detail Endpoint

**Date:** 2026-05-08  
**Fixed By:** CTO  
**Issue:** WOR-659 Smoke Test Bug #3  

---

## Bug Description

`GET /api/v1/worlds/:id/figures/:figure_id` returned 404 because there was no handler for this route.

**Impact:** Cannot view individual figure details via API.

---

## Root Cause

1. Route `/api/v1/figures/{id}` existed for cross-world figure lookup, but was not registered in the v1 router
2. Route `/{id}/figures/{figure_id}` under worlds existed but had no handler function
3. The `figures.rs` module used a non-existent `FigureDetailResponse` type

---

## Fix Applied

### 1. `src/api/v1/mod.rs` - Added figures module to router
```rust
pub mod figures;
// ...
.nest("/figures", figures::routes(state))
```

### 2. `src/api/v1/worlds.rs` - Added figure detail route and handler
```rust
.route("/{id}/figures/{figure_id}", get(get_world_figure))
```

Handler function added at line 737:
```rust
async fn get_world_figure(
    State(state): State<crate::api::AppState>,
    Path((world_id_raw, figure_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<HistoricalFigure>>, ApiError>
```

### 3. `src/api/v1/figures.rs` - Fixed cross-world lookup
- Removed broken `FigureDetailResponse` type
- Changed return type to existing `HistoricalFigure`
- Properly parses domain `NotableFigure` and converts to API model

---

## Verification

| Check | Status |
|-------|--------|
| Build (`cargo build --features api`) | ✅ Pass |
| Route registered | ✅ `/api/v1/worlds/{id}/figures/{figure_id}` |
| Handler loads figure | ✅ Reads from `storage/figures_path()` |
| Returns 404 on missing | ✅ `ApiError::NotFound` |

---

## Test Recommendation

Re-run smoke test (WOR-659) to verify:
- `GET /api/v1/worlds/:id/figures/:figure_id` returns 200 (with data) or 404 (not found)
- Previously failing endpoint is now resolved