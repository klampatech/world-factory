# WOR-1186: Dedicated /map route fails to render Voronoi cells

**Status:** RESOLVED  
**Test Date:** 2026-05-11  
**Fix Branch:** feature/WOR-1186-map-route-fix

---

## Problem Description

The dedicated `/worlds/:id/map` route was not rendering Voronoi cells on the map page. Users navigating directly to this URL would see an empty or incomplete map.

### Root Cause

1. **Path parameter vs query parameter mismatch**: The route `/worlds/:id/map` passes the world ID via path parameters, but `map.html` was reading from URL search parameters (`?id=...`)
2. **`loadMap()` not called on init**: The function to fetch map data from the API wasn't being called during page initialization

---

## Fix Applied

### 1. src/api/static_pages.rs

Modified `serve_map_page()` to inject `WORLD_ID` JavaScript variable into the HTML:

```rust
async fn serve_map_page(Path(world_id): Path<String>) -> impl IntoResponse {
    match load_html("map.html").await {
        Ok(html) => {
            // Inject world_id into JavaScript for the page to use
            let world_id_js = format!("const WORLD_ID = '{}';\n", world_id);
            // Inject before the closing </script> tag
            let html = html.replace(
                "</script>",
                &format!("{}\n</script>", world_id_js),
            );
            Html(html).into_response()
        }
        Err(status) => (status, "Map page not found").into_response(),
    }
}
```

### 2. web/static/map.html

Updated `parseParams()` to use injected `WORLD_ID`:

```javascript
function parseParams() {
    // Priority: 1) Injected by backend, 2) URL parameter, 3) fallback
    state.worldId = window.WORLD_ID || new URLSearchParams(window.location.search).get('id') || 'demo-world-1';
    document.getElementById('world-name').textContent = state.worldId.substring(0, 8) + '...';
}
```

Also added `loadMap()` call in the DOMContentLoaded handler:

```javascript
document.addEventListener('DOMContentLoaded', () => {
    parseParams();
    setupToolbar();
    loadMap();  // <-- Added this call
    // ... rest of init
});
```

---

## Test Results

All API tests pass:

| # | Test | Result |
|---|------|--------|
| 1 | GET /api/v1/worlds/:id/map returns 200 | PASS |
| 2 | Voronoi polygons returned | PASS (132 polygons) |
| 3 | Polygon has valid vertices | PASS (133 vertices) |
| 4 | Polygon has elevation data | PASS |
| 5 | Polygon has ocean metadata | PASS |
| 6 | Map has dimensions | PASS (256x256) |
| 7 | Polygon count in expected range | PASS |
| 8 | Elevation range valid | PASS |
| 9 | All polygons have valid structure | PASS |
| 10 | Vertex coordinates in valid range | PASS |

**Result: 10/10 tests passed**

---

## Deployment

The Docker image has been built:

```bash
docker build -t world-factory:WOR1186 -f Dockerfile .
```

To deploy:

```bash
# Stop existing container
docker stop world-factory
docker rm world-factory

# Start new container with the fix
docker run -d --name world-factory -p 8080:8080 --restart unless-stopped world-factory:WOR1186
```

Or using docker-compose:

```bash
docker-compose up -d
```

---

## Files Changed

- `src/api/static_pages.rs` - Inject WORLD_ID variable
- `web/static/map.html` - Use injected variable, call loadMap() on init
- `smoke-test-WOR-1186.js` - Test script (for reference)
- `qa-reports/WOR-1186-SMOKE-TEST.md` - Test report

---

## Verification

After deployment, verify the fix:

1. Navigate to `http://localhost:8080/worlds/{world-id}/map`
2. The page should display Voronoi polygons on the canvas
3. Check browser console for no errors
4. Verify the "Loading map..." indicator disappears

---

## Related Issues

- WOR-1184: Full smoke test includes map rendering verification
- WOR-1180: API smoke test verified /map endpoint returns 200