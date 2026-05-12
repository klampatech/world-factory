# Task: Wire ResourceSpawner into get_world_map Response ✅ DONE

(Previously stashed work picked up and implemented)
The `get_world_map` handler creates a `ResourceSpawner` but never calls it. The response's `biomes` and `resources` arrays are always empty because the spawned data is discarded.

## Current State
- `ResourceSpawner::with_config()` is called at line 569
- Each polygon computes a `biome` locally but only uses it for internal branching, never surfaces it
- `WorldMap` struct has `biomes: Vec<Biome>` and `resources: Vec<Resource>` fields
- Both are hardcoded to `Vec::new()`

## What Needs to Change

### 1. Collect biomes as a deduped list
The handler assigns a biome to every polygon but never collects them. Need to:
- Track unique biomes assigned to polygons
- Create `Biome` objects with id, name, type, colors, properties
- Populate `WorldMap.biomes`

### 2. Spawn and collect resources per polygon
`ResourceSpawner::spawn_region(region_id, biome, elevation, x, y)` returns `RegionResourceSpawn` which contains deposits. Need to:
- Extract centroids from polygon vertices (average x, average y) to get x, y position
- Call `spawn_region` for each non-ocean polygon
- Map `RegionResourceSpawn` deposits to `Resource` API model
- Populate `WorldMap.resources`

### 3. Add biome/terrain resource data to polygons
Each polygon may have resource deposits associated with it. Need to:
- Optionally link resource IDs to polygon response via `resource_ids: Vec<String>` on `Polygon`
- Or keep resources as a flat list on WorldMap (current design)

## Files Affected
- `src/api/v1/worlds.rs` — main handler logic
- `src/api/models.rs` — `Biome` struct if it needs new fields
- `src/terrain/resource_spawner.rs` — `RegionResourceSpawn` return type

## Verification
- After wiring, `GET /api/v1/worlds/{id}/map` should return non-empty `biomes` and `resources` arrays
- Resources should be biome-appropriate (iron near mountains, forests have lumber, etc.)
- Biomes should be unique per type, not repeated per polygon

## Status
- [x] Design: Decide if Polygon should reference resource IDs or if flat list is sufficient
- [x] Implement biome collection
- [x] Implement resource spawning loop
- [x] Test with known world ID