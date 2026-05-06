# World Factory — Procedural World & History Generation

A Rust-based procedural world generation engine with HTTP API, history simulation, and HTML/Canvas visualization.

## Features

- **Terrain Generation** — Voronoi-based geography with elevation, tectonics, rivers, and erosion
- **Biome Assignment** — Temperature + precipitation matrix for realistic biomes
- **Resource Spawning** — Minerals, energy, materials, and organic resources
- **History Simulation** — Configurable pre-history years with events, figures, and artifacts
- **Species Templates** — Human, Elf, Dwarf, Orc, Halfling with configurable behaviors
- **HTTP API** — Full REST API for world CRUD and data retrieval
- **Docker Support** — Run as a persistent development server

## Quick Start

### 1. CLI Mode

Generate a world directly in the terminal:

```bash
cargo run --features api -- generate --width 32 --height 32 --seed 42
```

### 2. API Server Mode

Start the HTTP server for full API access:

```bash
cargo run --features api -- --server --port 8080
```

In another terminal:

```bash
# Health check
curl http://localhost:8080/health

# Create a world
curl -X POST http://localhost:8080/api/v1/worlds \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test World",
    "config": {
      "width": 32,
      "height": 32,
      "pre_history_years": 50,
      "seed": 42
    }
  }'

# List worlds
curl http://localhost:8080/api/v1/worlds

# Get world details (replace <id> with UUID from creation response)
curl http://localhost:8080/api/v1/worlds/<id>

# Get map data
curl http://localhost:8080/api/v1/worlds/<id>/map

# Get history timeline
curl http://localhost:8080/api/v1/worlds/<id>/timeline

# Get notable figures
curl http://localhost:8080/api/v1/worlds/<id>/figures

# Get species list
curl http://localhost:8080/api/v1/species

# Export world as .wfw tarball
curl http://localhost:8080/api/v1/worlds/<id>/export -o world.wfw
```

### 3. Docker Mode

```bash
# Build and start persistent server
docker compose up -d world-factory

# Watch logs
docker compose logs -f

# Stop server
docker compose down
```

The server runs on `http://localhost:8080` with the same API endpoints as above.

### 4. Web Visualization

The server serves HTML visualization pages:

| URL | Description |
|-----|-------------|
| `GET /` | Landing page - world selector with list of all worlds |
| `GET /worlds/:id` | World overview with tabs |
| `GET /worlds/:id/map` | Map visualization with zoom/pan |
| `GET /worlds/:id/timeline` | History timeline view |
| `GET /worlds/:id/dashboard` | World stats dashboard |

Open `http://localhost:8080` in your browser to see the world selector and navigate to specific worlds.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      World Factory                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ World Gen   │  │ History     │  │ Faction System      │ │
│  │ Engine      │  │ Simulator   │  │ (Phase 5 - Future)  │ │
│  │ (Rust)      │  │ (Rust)      │  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              HTTP API Server (Axum)                      ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │              HTML/Canvas Visualization                  ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
world-factory/
├── src/
│   ├── api/v1/         # HTTP API handlers
│   ├── terrain/         # Geography generation
│   ├── hydro/           # Rivers and water
│   ├── history/         # History generation
│   ├── events/         # Event system
│   ├── figures.rs       # Notable figures
│   ├── artifacts.rs     # Historical artifacts
│   ├── species/         # Species templates
│   └── storage.rs       # World persistence
├── web/
│   └── index.html      # Browser visualization
├── tests/              # Integration tests
├── Dockerfile
├── docker-compose.yml
└── SPEC.md            # Full specification
```

## Development

```bash
# Build
cargo build --features api

# Run tests (406 lib tests)
cargo test --lib

# Run specific test
cargo test --lib test_terrain_generation

# Run with logging
RUST_LOG=debug cargo run --features api -- --server --port 8080
```

## Configuration

World generation is configurable via the API:

```json
{
  "name": "My World",
  "config": {
    "width": 64,
    "height": 64,
    "pre_history_years": 100,
    "seed": 12345,
    "sea_level": 0.4,
    "terrain": {
      "noise_scale": 0.01,
      "octaves": 6,
      "persistence": 0.5,
      "lacunarity": 2.0
    },
    "rivers": {
      "river_density": 0.3,
      "min_length": 10,
      "max_length": 500,
      "erosion_intensity": 0.5
    }
  }
}
```

## License

MIT
