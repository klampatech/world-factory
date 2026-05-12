# WOR-1192 Fix Summary

## Issue
Direct navigation to `/worlds/:id/map` returns 404 Not Found when running in Docker container.

## Root Cause
The `static_file_path()` function in `src/api/static_pages.rs` was using `std::env::current_dir()` to locate static HTML files. When running from Docker:
- The binary runs from `/app/`
- But `current_dir()` returns `/` or the working directory set by Docker
- The path `web/static/map.html` doesn't exist relative to that directory

## Fix Applied
Changed `static_file_path()` to use `std::env::current_exe()` to find files relative to the binary location:

```rust
/// Get the static HTML file path for a page
/// Uses the executable's directory as the base, not current working directory
fn static_file_path(page: &str) -> PathBuf {
    // Get the directory containing the executable, not the current working directory
    // This ensures static files are found correctly when running from any location
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("web").join("static").join(page)
}
```

## Files Changed
- `src/api/static_pages.rs` - Fixed static file path resolution

## Commit
- Cherry-picked to `feat/WOR-1196-cleanup-one-off-tests` (commit: `7b3665d`)

## Verification
The fix is correct - when `current_exe()` returns `/app/world_generator`:
- `parent()` → `/app`
- `join("web").join("static")` → `/app/web/static`
- `join("map.html")` → `/app/web/static/map.html`

This path matches where `web/static/` is copied in the Dockerfile.

## Verification Script
See `smoke-test-WOR-1192-v2.js` - ready for testing once Docker build issue is resolved.

## Docker Build Issue (BLOCKER)
Docker build produces a 426KB non-functional binary instead of the expected ~7.6MB binary.
- Binary exits immediately with code 0
- This is a separate infrastructure issue
- Working containers (e.g., `check-port`) have the correct binary size
