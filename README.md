# World Factory — Core Engine

Procedural world and history generation system in Rust.

## Quick Start

```rust
use world_factory::{WorldGenerator, WorldGenConfig};

let config = WorldGenConfig::default();
let generator = WorldGenerator::new(config);
let world = generator.generate(42); // Seed 42

println!("Generated {} rivers", world.rivers.len());
println!("Land coverage: {:.1}%", world.land_percentage() * 100.0);
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      World Factory                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   World     │    │   Entity    │    │  Timeline   │     │
│  │ Generation  │───▶│   System    │───▶│  Generator  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                  │                  │              │
│         ▼                  ▼                  ▼              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Terrain   │    │   Culture  │    │   History  │     │
│  │  Generator │    │  Generator │    │  Generator │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Modules

### `terrain` — Terrain Generation
- Elevation grids with noise functions
- Multi-octave Perlin-style noise
- Sea level and biome assignment

### `hydro` — Hydrology
- River generation from elevation
- Flow accumulation modeling
- Erosion simulation

### `entity` — Entity System (TODO)
- Hierarchical entity storage
- Relationship graphs
- Attribute querying

### `util` — Utilities
- Seeded RNG (Mulberry32)
- Geometric primitives
- Common algorithms

## Configuration

```rust
let config = WorldGenConfig {
    width: 512,
    height: 512,
    sea_level: 0.4,
    terrain: TerrainConfig {
        noise_scale: 0.01,
        octaves: 6,
        persistence: 0.5,
        lacunarity: 2.0,
        ..Default::default()
    },
    rivers: RiverConfig {
        river_density: 0.3,
        min_length: 10,
        max_length: 500,
        erosion_intensity: 0.5,
        ..Default::default()
    },
};
```

## Performance Targets

| Metric | Target |
|--------|--------|
| World generation (256×256) | < 5 seconds |
| Memory per million entities | < 500MB |
| Deterministic output | Same seed = same world |

## Development

```bash
# Build
cargo build

# Test
cargo test

# Benchmarks
cargo bench
```

## License

MIT
