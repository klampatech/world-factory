# WOR-432 Investigation Report: Two Different UIs on Same Port

## Summary

**No changes were lost.** The "two different UIs" observed were the same frontend application served via different mechanisms.

## Port Configuration (Definitive)

| Port | Service | Source | Purpose |
|------|---------|--------|---------|
| **3000** | Rust API Server | `src/main.rs` | Local development backend (default) |
| **8080** | Rust API Server | `docker-compose.yml` | Docker deployment backend |
| **8765** | Frontend Preview | `web/scripts/preview.js` | Serves built frontend (`web/dist/`) |
| **5173** | Vite Dev Server | `web/` | Development with hot module reload |

**Port 8787 is NOT used anywhere in the codebase.**

## Architecture

### Single Frontend, Multiple Serving Options

There is **one frontend application** (`web/`) that can be served in two ways:

1. **Preview mode (8765)**: Static production build via Node.js HTTP server
2. **Dev mode (5173)**: Vite hot module replacement with live reload

Both serve the **same application** — just different serving mechanisms.

### The "Two Different UIs" Explained

The confusion likely came from:
- Seeing the same frontend via different servers (8765 vs 5173)
- Or viewing `demo.html` which is a standalone demo with different styling
- Or `demo-society-dashboard.html` which is a separate dashboard demo

None of these represent lost changes or conflicting frontend implementations.

## Files Analyzed

- `src/main.rs` - Rust CLI/API server (port 3000 default)
- `docker-compose.yml` - Docker container config (port 8080)
- `web/scripts/preview.js` - Frontend preview server (port 8765)
- `web/api-integration.js` - API client (points to port 8080)
- `demo.html` - Standalone demo (separate from main app)
- `demo-society-dashboard.html` - Dashboard demo

## Recommendations

1. **Document port usage** - Add port configuration to README
2. **Cleanup legacy files** - Consider archiving `demo.html` and `demo-society-dashboard.html`
3. **Standardize testing** - All test scripts use port 8765 consistently

## Testing Configuration

For consistent testing:
```bash
# Option 1: Local development
cargo run --features api -- server --port 3000
cd web && npm run preview  # Serves on 8765

# Option 2: Docker deployment  
docker-compose up  # Everything on 8080

# Option 3: Unit tests (already configured)
node scripts/smoke-test-simple.js  # Targets 8765
```

## Investigation Completed

2026-05-07
