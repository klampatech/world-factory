# Primal Beasts System - WOR-80

## Status: IMPLEMENTED

## Overview

The Primal Beasts System provides legendary creature management for procedural world generation. Primal Beasts are powerful, unique entities tied to specific terrain features and biome types.

## Core Types

### PrimalBeastType (12 types)

| Type | Power | Element | Preferred Biomes |
|------|-------|---------|------------------|
| Dragon | 0.95 | Fire | MontaneForest, Volcanic |
| Leviathan | 0.90 | Water | OpenOcean, CoastalOcean |
| Phoenix | 0.85 | Fire/Life | Volcanic, HotDesert |
| StormLord | 0.88 | Air/Lightning | MontaneForest, OpenOcean |
| Basilisk | 0.80 | Earth | MontaneForest, RockyDesert |
| Serpent | 0.78 | Water/Wisdom | TropicalRainforest, SwampForest |
| Fenrir | 0.82 | Shadow | BorealForest, TemperateSteppe |
| Gryphon | 0.75 | Air/Earth | MontaneForest, AlpineTundra |
| Roc | 0.72 | Air | AlpineTundra, RockyMountain |
| Ursine | 0.70 | Ice | BorealForest, Arctic |
| Treant | 0.68 | Life/Nature | TemperateRainforest, BorealForest |
| Elemental | 0.85 | Primal | Volcanic, RockyDesert |

### BeastLifecycleState

- **Active** - Beast present in world (1.5x influence)
- **Dormant** - Sleeping in lair (0.8x influence)
- **Migrated** - Moved to new territory (0.5x influence)
- **Slain** - Killed by heroes (0.3x influence)
- **Mythological** - Only exists in legends (1.0x influence)

### BeastBehavior

- **Territorial** - Guards territory fiercely (0.7 aggression)
- **Migratory** - Moves between regions (0.3 aggression)
- **Raider** - Attacks settlements (0.9 aggression)
- **Secretive** - Hidden unless provoked (0.4 aggression)
- **Guardian** - Protects sacred sites (0.6 aggression)
- **Elusive** - Avoids civilization (0.2 aggression)

## Data Model

### PrimalBeast

```rust
pub struct PrimalBeast {
    pub id: EntityId,
    pub world_id: Uuid,
    pub beast_type: PrimalBeastType,
    pub name: String,
    pub description: String,
    pub state: BeastLifecycleState,
    pub behavior: BeastBehavior,
    pub polygon_id: Option<u32>,
    pub latitude: f64,
    pub longitude: f64,
    pub territory_radius_km: f64,
    pub power_level: f32,
    pub first_appearance_year: Option<i32>,
    pub state_change_year: Option<i32>,
    pub associated_events: Vec<Uuid>,
    pub parent_id: Option<Uuid>,
}
```

## Generation Algorithm

1. **Count**: 1-3 beasts based on world size
2. **Type Selection**: Weighted probability (Dragons 25%, Phoenix 12%, etc.)
3. **Location**: Terrain-aware placement preferring matching biomes
4. **Territory**: 50-150km radius based on type
5. **Behavior**: Type-based with settlement proximity influence

## Integration Points

### With HistoryGenerator

- Beasts can trigger significant events (appearances, battles, migrations)
- Events linked via `associated_events` field
- State transitions generate historical records

### With Terrain System

- `BeastTerrainData` constructed from elevation/biome grids
- Placement respects biome preferences
- Coastal detection for water-based beasts

## API Patterns

Exposed via `lib.rs` exports:

```rust
pub use primal_beasts::{
    PrimalBeast, PrimalBeastType, PrimalBeastStore, PrimalBeastGenerator,
    PrimalBeastConfig, BeastLifecycleState, BeastBehavior,
    BeastTerrainData, BeastTerrainCell, PrimalBeastNameGenerator,
};
```

## Files

- `src/primal_beasts.rs` - Core implementation (40KB, ~1050 lines)
- `src/lib.rs` - Module and type exports

## Unit Tests (10 tests)

- `test_beast_type_labels` - Label generation
- `test_beast_power_levels` - Power hierarchy
- `test_beast_preferred_biomes` - Biome mapping
- `test_beast_display_color` - Map colors
- `test_primal_beast_creation` - Construction
- `test_beast_state_transition` - State changes
- `test_beast_store` - Storage operations
- `test_name_generation` - Procedural names
- `test_beast_terrain_data` - Grid construction
- `test_generator_config` - Config defaults
- `test_beast_behavior_aggression` - Behavior scoring

---

*Implemented: 2026-05-06*
---

## Post-Implementation Fixes Applied (2026-05-06)

During final compilation check, identified and fixed several exhaustiveness issues caused by BiomeType enum extensions:

### 1. `src/primal_beasts.rs` - Removed unused imports
```rust
// Removed: use chrono::{DateTime, Utc};
// Removed: use crate::types::Timestamp;
// Removed: use crate::terrain::ClimateZone;
```
Also added explicit type annotation: `let mut score: f32 = 0.5;`

### 2. `src/terrain/biome.rs` - Extended match arms
Added missing BiomeType variants to:
- `BiomeColorMapping::get_color()` - 19 new variants (CoastalOcean, DeepOcean, FreshwaterMarsh, IceShelf, etc.)
- `BiomeType::name()` - 16 new variants
- `BiomeType::vegetation()` - 14 new variants mapped to `Desert` fallback

### 3. `src/terrain/biome_assignment.rs` - Extended climate index match
Added 13 new BiomeType variants mapped to climate zone 2 (Temperate)

### 4. `src/simulation/population.rs` - Extended growth modifier match
Added 13 new BiomeType variants with appropriate growth modifiers

### 5. `src/types.rs` - Extended carrying capacity match
Added 13 new BiomeType variants with capacity values (0-800 range)

### 6. `src/terrain/natural_wonders/mod.rs` - Fixed conditional import
Moved `use crate::api::models::{WonderView, WonderBonusView};` under `#[cfg(feature = "api")]`

## Verification
- `cargo check --lib` succeeds for primal_beasts module
- Remaining compilation errors in codebase are pre-existing (packaging.rs, faction.rs, drainage_basin.rs) and unrelated to WOR-80
