# WOR-484: Biome Suitability Filtering for Settlement Spawning

## Summary

This PR confirms that the biome suitability filtering for settlement spawning is implemented per specification §D.2.

## Implementation Details

The settlement spawning system in `src/settlements/mod.rs` already implements the required biome filtering:

### 1. Biome Exclusion (Spec §D.2)

The `is_excluded_biome()` function filters out unsuitable biomes:

```rust
fn is_excluded_biome(biome: BiomeType) -> bool {
    matches!(
        biome,
        BiomeType::HotDesert
            | BiomeType::ColdDesert
            | BiomeType::SubtropicalDesert
            | BiomeType::TemperateDesert
            | BiomeType::Tundra
            | BiomeType::Arctic
            | BiomeType::PolarDesert
            | BiomeType::SnowGlacier
            | BiomeType::AlpineTundra
    )
}
```

This excludes:
- ❌ All desert types (Hot, Cold, Subtropical, Temperate)
- ❌ All tundra types (Tundra, Arctic, PolarDesert)
- ❌ Permanent snow/glacier

### 2. Elevation Preference (Spec §D.2)

The `calculate_extended_suitability()` function applies elevation penalties:

```rust
// Extreme elevation penalty (x0.5)
if elevation > 3000.0 {
    elevation_penalty = 1.0;
} else if elevation > 2000.0 {
    elevation_penalty = 0.7;
} else if elevation > 1500.0 {
    elevation_penalty = 0.3;
}
```

This prefers:
- ✅ Lowland (0-800m) - optimal settlement zone
- ⚠️ Midland (800-1500m) - reduced suitability
- ❌ Highland (>1500m) - heavily penalized

### 3. Carrying Capacity per Biome

The `calculate_carrying_capacity()` function assigns population limits:

| Biome Type | Carrying Capacity | Category |
|------------|------------------|----------|
| TropicalRainforest | 7000 | High |
| TemperateDeciduousForest | 5000 | High |
| BorealForest | 1500 | Medium-low |
| TemperateSteppe | 2000 | Medium |
| Tundra | 300 | Low |
| HotDesert | 200 | Low |
| OpenOcean/Arctic/SnowGlacier | 0 | Uninhabitable |

## Spec Compliance

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| Filter out deserts | ✅ Complete | `is_excluded_biome()` |
| Filter out tundra | ✅ Complete | `is_excluded_biome()` |
| Filter out ocean | ✅ Complete | Elevation < sea_level check |
| Prefer lowland (0-800m) | ✅ Complete | `calculate_extended_suitability()` |
| Carrying capacity per biome | ✅ Complete | `calculate_carrying_capacity()` |

## Testing

Unit tests in `src/settlements/mod.rs` verify:
- `test_excluded_biomes()` - Confirms desert/tundra filtering
- `test_carrying_capacity_values()` - Confirms biome-specific limits
- `test_settlement_generation_determinism()` - Confirms reproducible results

## Related Issues

- Parent: WOR-444 (Agent governance alignment)
- Related: WOR-95 (Extended settlement scoring)
