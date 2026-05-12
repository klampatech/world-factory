# WOR-1192: Dedicated /map route returns 404

## Summary
The dedicated `/worlds/:id/map` route was returning 404 or failing to properly load the world ID.

## Root Cause
The backend was injecting world ID as `let html = html.replace("demo-world-1", &world_id);` which:
1. Only replaced the literal string "demo-world-1" in the HTML
2. Did not properly set `window.WORLD_ID` which map.html's `parseParams()` function reads

## Fix Applied
Changed `src/api/static_pages.rs` to inject `window.WORLD_ID = '...';` before the closing `</script>` tag:

```rust
// Inject world_id as window.WORLD_ID for the page to use
// (window.WORLD_ID is accessed by map.html's parseParams function)
let world_id_js = format!("window.WORLD_ID = '{}';", world_id);
let html = html.replace(
    "</script>",
    &format!("{}\n</script>", world_id_js),
);
```

This fix was applied to both `serve_world_overview()` and `serve_map_page()` handlers.

## Also Updated
Dockerfile - Added COPY command for web/static directory:
```dockerfile
# Copy static web files for HTML page serving
COPY web/static /app/web/static
```

## Verification
The fix was verified with a smoke test that:
1. Creates a test world
2. Calls `/api/v1/worlds/:id/map` (returns 200)
3. Calls `/worlds/:id/map` static route (returns 200)
4. Verifies `window.WORLD_ID` is properly injected in the response

```
✅ TEST PASSED
```

## Files Changed
- `src/api/static_pages.rs` - Fixed world ID injection
- `Dockerfile` - Added web/static directory copy

## Status
**COMPLETED** - Fix committed to repository