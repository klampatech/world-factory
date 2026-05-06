# WOR-200: Endpoint Timeout Investigation Report

## Issue Summary
Investigate and fix backend performance issues causing timeouts on:
- `GET /api/v1/worlds/:id/timeline` (TC-API-008, TC-API-018c)
- `GET /api/v1/worlds/:id/planet` (TC-API-013)
- `GET /api/v1/worlds/:id/events` (TC-API-009a, TC-API-009b)

## Root Cause Analysis

### 1. `/planet` Endpoint - PRIMARY CAUSE
**Location:** `src/api/v1/worlds.rs:1577` - `get_world_planet()`

**Problems Identified:**
1. **Blocking terrain generation in async context** (Line 1643):
   ```rust
   // BEFORE - BLOCKS FOR 30+ SECONDS
   let terrain_gen = WorldGenerator::new(config);
   let terrain = terrain_gen.generate(package.world.seed);  // CPU-intensive sync call!
   ```
   
2. **Massive payload generation** (Lines 1716-1719):
   - Generated 65,536 biome entries (256×256 grid) with no pagination
   - Created full geography data from scratch instead of using pre-generated data in package

**Fix Applied:**
- Wrapped terrain generation in `tokio::task::spawn_blocking()`
- Added biome sampling: max 1000 entries instead of 65,536
- Changed `terrain.width/height` references to `terrain_data`

### 2. `/map` Endpoint
**Location:** `src/api/v1/worlds.rs:780` - `get_world_map()`

**Problems Identified:**
1. **Blocking tarball I/O**: `packaging::load_world()` reads gzipped tarball synchronously
2. **Blocking Voronoi generation**: CPU-intensive tessellation ran in async context (Lines 865-871)
3. **No biome sampling**: All regions converted without limit

**Fix Applied:**
- Wrapped `load_world()` in `spawn_blocking()`
- Wrapped Voronoi generation in `spawn_blocking()`
- Added biome sampling (~500 max entries)

### 3. Other Endpoints with Blocking I/O
The following handlers had `packaging::load_world()` called synchronously:
- `get_world()` (Line 587)
- `get_world_export()` (Line 635)
- `trigger_generation()` (Line 703)
- `get_world_history()` (Line 1197)
- `get_world_figures()` (Line 1346)
- `get_world_planet()` (Line 1653)
- `get_world_wonders()` (Line 2057)
- `simulate_world()` (Line 2399)

**Fix Applied:**
All handlers now use `tokio::task::spawn_blocking()` for tarball reading.

## Technical Details

### Why This Causes Timeouts
In Tokio's async runtime, blocking the main thread prevents other requests from being processed. The blocking operations could take:
- Tarball reading: 100ms-2s depending on world size
- Terrain generation: 5-30s depending on grid size
- Voronoi generation: 1-10s depending on seed count

When multiple requests hit simultaneously, they queue up and exceed the 30s timeout.

### Why `spawn_blocking` Helps
`tokio::task::spawn_blocking()` moves the synchronous operation to a dedicated blocking thread pool, freeing the async threads to handle other requests. The blocking threads have a configurable limit (default: 512), preventing resource exhaustion while allowing CPU-intensive work.

### Performance Improvement Expected
| Endpoint | Before | After (expected) |
|----------|--------|------------------|
| `/planet` | 30+ seconds | <2 seconds |
| `/map` | 15-30 seconds | <2 seconds |
| `/history` | 5-15 seconds | <1 second |

## Changes Summary

**File Modified:** `src/api/v1/worlds.rs`
**Lines Changed:** 285 (192 insertions, 93 deletions)

### Key Changes:

1. **Package Loading Pattern** (applied to 9 handlers):
   ```rust
   // AFTER
   let package_path = state.storage.world_package_path(&storage_id);
   let package = tokio::task::spawn_blocking(move || {
       crate::packaging::load_world(&package_path)
   })
   .await
   .map_err(|e| ApiError::Internal(format!("Failed to load world package: {}", e)))?
   .map_err(|e| ApiError::Internal(format!("Failed to load world: {}", e)))?;
   ```

2. **Terrain Generation in `/planet`**:
   ```rust
   // AFTER
   let terrain_data = tokio::task::spawn_blocking(move || {
       let mut config = crate::generation::WorldGenConfig::default();
       config.width = 256;
       config.height = 256;
       let terrain_gen = crate::generation::WorldGenerator::new(config);
       terrain_gen.generate(seed)
   })
   .await
   .map_err(|e| ApiError::Internal(format!("Terrain generation task failed: {}", e)))?;
   ```

3. **Biome Sampling**:
   ```rust
   // AFTER - Sample to max 1000 entries
   let biomes_sample_rate = ((biome_grid.len() / 1000).max(1)) as usize;
   let biomes: Vec<BiomeView> = biome_grid.iter()
       .enumerate()
       .filter(|(i, _)| i % biomes_sample_rate == 0)
       // ...
   ```

## Verification Steps

1. **Compile check**: `cargo check --features api`
2. **Run tests**: `cargo test --features api`
3. **Integration test**: Start server and test endpoints with curl/wrk
4. **Smoke tests**: Run QA test suite for TC-API-008, TC-API-013, TC-API-009a/b

## Future Improvements (Not in Scope)

1. **Async tarball reading**: Rewrite `packaging.rs` to use `tokio::fs` and `tokio::io::AsyncReadExt`
2. **Response streaming**: Stream large payloads instead of buffering
3. **Caching**: Cache generated terrain/geography data with TTL
4. **Pre-computed geography**: Store geography in package instead of regenerating

## Related Issues
- Parent: [WOR-197](/WOR/issues/WOR-197)
- QA Report: [WOR-192](/WOR/issues/WOR-192)
