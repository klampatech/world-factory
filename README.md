# World Factory

Deterministic, provenance-aware world simulation foundation. Ships a
typed `WorldModel` contract, a stateless seeded RNG, atomic JSON
persistence, a CLI, and cross-layer plausibility invariants.

This is the **Phase 0 seam**. It does not simulate biology, society,
politics, or history yet. It establishes the contract between the
geography core and the ABM / history layer so that later phases can
land without rewriting their neighbours.

## What this version claims to model

- **Geography**: regular-grid elevation generated from a stateless
  seeded sampler (latitude/longitude wave + deterministic per-cell
  noise).
- **Hydrology**: aggregate surface-water fraction and headwater
  candidate counts derived from elevation and precipitation. No river
  network, no watersheds, no discharge.
- **Climate**: regular-grid temperature and annual precipitation,
  derived from elevation, latitude, and a `climate_class` parameter.
- **Biomes**: a coarse per-cell classification (`ocean`, `ice`,
  `alpine`, `desert`, `tropical-forest`, `temperate-forest`,
  `grassland`) derived from the three physical layers.

## What this version explicitly does NOT model

Phase 0 is a seam, not a world. It does not yet claim to simulate:

- Geology (plate layout, tectonics, rock types, ore distribution,
  soil).
- Real hydrology (river network, flow direction, discharge,
  watersheds, salinity, aquifers).
- Astronomy (star, planets/moons, orbital elements, axial tilt,
  day length, year length, eclipses, tides, auroras).
- Atmosphere beyond a coarse climate class (no composition,
  pressure, prevailing winds, storms, or seasons).
- Biology (species, food webs, evolution, biogeography).
- Anthropogenic or social layers (settlements, agriculture,
  infrastructure, language, culture, religion, kinship).
- Politics or economics (polities, trade, conflict, governance).
- History (event log, causal graph, multi-perspective records).
- Exploration surface (no spatial / temporal / entity / event /
  topic queries yet; the CLI is generation-only).
- Living-world behavior (no drift, no surprise, no persistent state
  beyond what one generation produces).

A Definition of Done covering the full scope lives in the project's
internal planning docs. This README states limits because a definition
of done that hides its limits is not done.

## Generation properties

- **Reproducibility**: same seed, scale, climate class, sentience,
  and magic flag → byte-identical serialized world.
- **Parametric control**: knobs (`--seed`, `--scale`, `--climate`,
  `--sentience`, `--magic`) all take effect; sentience and magic are
  carried through `WorldConfig` for downstream phases but do not
  influence Phase 0 outputs.
- **Auditability**: every layer carries a `ProvenanceRecord` linking
  output paths to their generating process and algorithm version.
- **Versioning**: schema version and model version are recorded on
  every world; persisted worlds strictly reject unknown fields.

## Installation

```sh
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

Requires Python 3.12+.

## Usage

Generate a deterministic world:

```sh
world-factory generate --seed 42 --scale small --climate temperate \
    --sentience --no-magic --out worlds/demo.json
```

Validate a persisted world against the Phase 0 invariant set:

```sh
world-factory validate worlds/demo.json
```

The CLI exits non-zero on any invariant violation and prints a
machine-readable `ValidationReport` to stdout.

## Development

Run the quality gates locally:

```sh
ruff check src tests
mypy --strict src/world_factory
pytest
```

## Project structure

```
src/world_factory/
├── __init__.py        # public API: generate_world, save/load_world, validate_world
├── constants.py       # versioned constants and physical bounds
├── determinism.py     # stateless seeded sampler (blake2b grid)
├── models.py          # pydantic strict typed contracts
├── persistence.py     # atomic JSON load/save
├── generator.py       # deterministic generation pipeline
├── validation.py      # cross-layer plausibility invariants
└── cli.py             # argparse CLI entry point
```

## Provenance

Every serialized world carries:

- a deterministic `world_id` derived from its `WorldConfig`,
- a `schema_version` and `model_version`,
- per-layer `ProvenanceRecord` entries linking output paths to
  generating processes and algorithm versions.

The same input always produces the same world; changing the algorithm
version invalidates byte-identical comparison across versions.

## License

MIT. See `LICENSE`.