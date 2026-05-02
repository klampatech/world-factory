# Settlement Spawning Module

## Overview

Generates settlement locations and properties for World Factory procedural worlds.

## Quick Start

```rust
use world_factory::{SettlementGenerator, SettlementConfig, WorldGenConfig, WorldGenerator};

let config = WorldGenConfig::default();
let settlement_config = SettlementConfig::default();

let world = WorldGenerator::new(config.clone()).generate(42);

// Generate biomes first (see BiomeAssignmentMatrix)
let (biome_grid, climate_grid) = generate_world_biomes(&world, 42);

// Generate settlements
let river_cells: Vec<(i32, i32)> = world.river_cells().iter()
    .map(|v| (v.x, v.y))
    .collect();

let mut settlement_gen = SettlementGenerator::new(settlement_config, 42);
let result = settlement_gen.generate(
    &world.elevation.data,
    &biome_grid,
    &climate_grid,
    config.sea_level,
    config.width,
    config.height,
    Some(&river_cells),
);

println!("Generated {} settlements", result.stats.total);
```

## Algorithm Phases

### 1. Suitability Analysis
Each terrain cell scored 0.0–1.0:
- ✅ Grassland, Forest = best (0.7–0.9)
- ⚠️ Taiga, Wetland = acceptable (0.5)
- ❌ Desert, Tundra, Ocean = excluded

Bonuses: River (+0.15), Coastal (+0.10)
Penalties: High elevation (>1000m), Polar climate

### 2. Population Density Map
Multi-scale noise (1/64 + 1/16 octaves) for natural clustering.

### 3. Site Selection
Greedy algorithm with `min_spacing` constraint (default 8 cells).
Target count: `density_target × (width × height) / 1000`

### 4. Settlement Creation
- Type: Hamlet/Village/Town/City/Metropolis
- Population: Scaled by local density
- Location: Geo coordinates from cell position

### 5. Name Generation
Procedural names (placeholder for species/culture module).

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `density_target` | 0.5 | Settlements per 1000 cells |
| `min_spacing` | 8 | Min cells between settlements |
| `max_attempts` | 100 | Site search attempts |
| `coastal_max_elevation` | 2.0m | Max elevation for coastal |

## Settlement Types

```rust
pub enum SettlementType {
    Hamlet,      // < 100 people
    Village,     // 100-1000 people
    Town,        // 1000-10000 people
    City,        // 10000-100000 people
    Metropolis,  // 100000+ people
    Capital,     // Political center
    Fortress,    // Military installation
    Port,        // Maritime trade
    SacredSite,  // Religious significance
}
```

## Result

```rust
pub struct SettlementResult {
    pub settlements: Vec<Settlement>,
    pub stats: SettlementStats,
}

pub struct SettlementStats {
    pub total: usize,
    pub by_type: HashMap<SettlementType, usize>,
    pub by_biome: HashMap<BiomeType, usize>,
    pub coastal_count: usize,
    pub river_count: usize,
    pub average_population: f64,
}
```

## Tests

```bash
cargo test settlements::tests
```

**Test coverage:**
- `test_excluded_biomes` — Verify desert/tundra are blocked
- `test_settlement_config_default` — Config defaults
- `test_density_map_access` — Grid boundary handling
- `test_settlement_generation_determinism` — Same seed = same output
- `test_species_suitability_by_biome` — Species/biome mapping
- `test_settlement_species_assignment_placeholder` — Assignment logic
- `test_species_name_templates_placeholder` — Name generation

## Species Integration (WOR-43)

See `WOR-43-SPECIES-MODULE.md` for species assignment spec.

```rust
// When species module is available:
impl SettlementGenerator {
    pub fn generate_with_species(
        &mut self,
        elevation_grid: &[f32],
        biome_grid: &[BiomeType],
        climate_grid: &[ClimateZone],
        species_data: &SpeciesData,
        sea_level: f32,
        width: usize,
        height: usize,
        river_cells: Option<&[(i32, i32)]>,
    ) -> SettlementResult {
        // Species-aware settlement creation
    }
}
```

## Files

- `src/settlements/mod.rs` — Core algorithm
- `src/generation/world_with_settlements.rs` — Pipeline integration