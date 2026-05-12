# WOR-1213 Resolution: Docker Build Produces Broken Binary

## Issue Summary
Docker build was producing a ~426KB binary instead of the expected ~7.5MB binary.

## Root Cause Analysis
The Dockerfile's two-stage build process had a critical flaw in its layer caching strategy:

1. **Stage 1 (builder)**: Creates dummy `src/` directory to cache dependencies
2. **Builds dependencies**: `cargo build --release --features api && rm -rf src`
3. **Copies actual source**: `COPY src/ src/`
4. **Final build**: `cargo build --release --features api`

The problem: After building dependencies, only `src/` was deleted (`rm -rf src`), leaving the compiled target directory. When the second build runs with the dummy source, cargo detects the unchanged source (dummy) and skips recompilation. The binary gets linked against the cached object files, but since the dummy main.rs has no actual code logic, produces a near-empty binary.

## Fix Applied

**File**: `Dockerfile`

```diff
- RUN cargo build --release --features api && rm -rf src
+ RUN cargo build --release --features api && rm -rf src target
```

Deleting BOTH `src/` AND `target/` after the dependency build ensures a clean rebuild on the second pass. This matches the pattern already in `Dockerfile.clean`.

## Verification

Built corrected image and verified:
- Binary size: 7,522,352 bytes (~7.2MB) ✓
- Server starts: `Starting World Factory API server on http://0.0.0.0:8080` ✓
- Health endpoint: `/health` returns 200 OK ✓

## Smoke Test Log
```
node smoke-test-WOR-1213.js
Waiting for server health check...
PASS: Server health check passed
ls output: -rwxr-xr-x 1 root root 7522352 May 11 18:54 /app/world_generator
Binary size: 7522352 bytes (7.17MB)
PASS: Binary size acceptable
PASS: Health API returned 200

✅ All smoke tests passed!
```

## Files Modified
- `Dockerfile` - Added `target` to cleanup step

## Files Correct (Already Fixed)
- `Dockerfile.clean` - Had correct pattern all along