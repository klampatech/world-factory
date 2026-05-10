# Priority Fix 3: CLI World Persistence

> **Issue:** `cargo run -- generate ...` does not save a `.wfw` file to storage
> **Severity:** MEDIUM — generated worlds are ephemeral, not visible to the API server
> **Reference:** SPEC.md §7.4 (CLI-to-Server World Persistence)

---

## Problem Description

When running `cargo run -- generate --width 64 --height 64 --seed 42`, the world is generated
and output is printed to stdout, but **no `.wfw` tarball is saved** to `WORLD_FACTORY_DIR`.

This means:
1. CLI-generated worlds are lost when the process exits
2. The API server cannot see CLI-generated worlds (they use independent storage)
3. The SPEC.md §7.4 acceptance criteria are not met

---

## Required Implementation (per SPEC.md §7.4)

### 1. Automatic Persistence on Generation

After successful world generation, the CLI should call `packaging::save_world_package()` to
save the world as a `.wfw` tarball in `<WORLD_FACTORY_DIR>/generated/<world_id>/world.wfw`.

```
~/.local/share/world-factory/generated/<world_id>/world.wfw
```

The tarball must contain:
```
<world_id>.wfw/
├── world.json          # Top-level metadata (id, name, created_at, config)
├── planet/
│   ├── geography.json  # Polygon mesh, elevation, hydrology
│   ├── biomes.json     # Biome assignments
│   └── resources.json  # Resource locations
├── history/
│   ├── events.json     # Chronological events
│   ├── figures.json    # Notable figures
│   └── artifacts.json  # Historical artifacts
├── societies/
│   ├── factions.json   # Current factions
│   └── settlements.json # Cities, towns, villages
└── time/
    └── state.json      # Current year, time scale
```

### 2. `--export-to <path>` Flag

```bash
world-factory generate --width 64 --height 64 --export-to /tmp/worlds
```

Saves to the specified directory instead of the default `WORLD_FACTORY_DIR`.

### 3. Shared Storage with API Server

When CLI and server share the same `WORLD_FACTORY_DIR`:
- CLI generates world → saves to shared dir
- Server startup reads all worlds from shared dir
- `GET /api/v1/worlds` lists both CLI-generated and server-generated worlds

```bash
# Generate with shared storage
WORLD_FACTORY_DIR=/tmp/worlds cargo run -- generate --width 32 --height 32 --seed 42

# Start server with same directory
WORLD_FACTORY_DIR=/tmp/worlds cargo run --features api -- --server --port 8080

# World visible at GET /api/v1/worlds
```

### 4. Deterministic World IDs

Running `generate` with the same seed must produce the same world ID, enabling:
- Reproducible world exports
- Server can detect and skip re-generating known worlds

---

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| 1 | `cargo run -- generate --width 32 --height 32 --seed 42` saves a `.wfw` file | Check `<WORLD_FACTORY_DIR>/generated/<id>/world.wfw` exists |
| 2 | The `.wfw` tarball contains valid `world.json` with id, name, created_at, config | Inspect tarball contents |
| 3 | `--export-to /tmp/custom-path` saves to the specified directory | Compare storage paths |
| 4 | Same seed produces same world ID (deterministic) | Run twice, compare IDs |
| 5 | Server with shared `WORLD_FACTORY_DIR` lists the CLI world at `GET /api/v1/worlds` | Start server, curl endpoint |
| 6 | World in tarball can be loaded back (round-trip) | Server reads and serves the world |
| 7 | `cargo test --lib` still passes | No regression |

---

## Key Files

- `src/main.rs` — CLI `generate` command (add persistence call after generation)
- `src/packaging.rs` — `save_world_package()` function (should already exist)
- `src/storage.rs` — `StorageManager` for world save/load
- `src/lib.rs` — public exports of storage types

---

## Implementation Hint

In `src/main.rs`, after the generation pipeline succeeds, add:

```rust
// Save to storage
let storage = StorageManager::new()?;
let world_path = storage.save_world(&world)?;
println!("World saved to: {:?}", world_path);
```

The `packaging::save_world_package()` function should already exist and create the tarball.
If it doesn't exist, implement it using the existing JSON serialization in `src/`.

---

## Notes

- The API server already has `GET /api/v1/worlds` and `POST /api/v1/worlds` working
- The issue is purely on the CLI side — generation works, persistence doesn't
- Do NOT change the API endpoints or storage format — just wire up the existing pieces