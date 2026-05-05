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

### Prerequisites

- **Rust** (recommended): Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
  ```

- **Docker** (alternative): Install [Docker Desktop](https://docs.docker.com/get-docker/) if Rust is not available

### Building and Testing

```bash
# With Rust installed:
cargo build
cargo test -- --nocapture
cargo clippy --all-targets --all-features -- -D warnings

# Without Rust (using Docker):
docker build -f Dockerfile.test -t world-factory:test .
docker run --rm -v $(pwd):/workspace -w /workspace world-factory:test

# With just (recommended for either case):
# Install: cargo install just
just test          # Run all tests
just test-unit     # Unit tests only
just test-integration  # Integration tests
just lint         # Run clippy
just build        # Build the project
just fmt          # Format code
```

### Local Test Workflow

The project provides two ways to run tests locally:

1. **Docker (recommended when no Rust installed)**
   ```bash
   docker build -f Dockerfile.test -t world-factory:test .
   docker run --rm -v $(pwd):/workspace -w /workspace world-factory:test
   ```

2. **just task runner (for either Rust or Docker)**
   ```bash
   # Install just: cargo install just
   just test
   ```
   The justfile automatically detects whether `cargo` or `docker` is available and runs tests accordingly.

### Benchmarking

```bash
cargo bench
```

## License

MIT
