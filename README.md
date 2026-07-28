# World Factory

Deterministic, provenance-aware world simulation foundation. Ships a
typed `WorldModel` contract, a stateless seeded RNG, atomic JSON
persistence, a CLI, and cross-layer plausibility invariants.

This is the **Phase 1a release**. It introduces the tectonic plate
geometry, the barometric atmospheric pressure grid, and the versioned
RNG namespace bump (`tectonic-plates-v1`) that replaces the Phase 0
flat namespaces. It still does not simulate biology, society,
politics, or history; those land in later phases.

## What this version claims to model

- **Geology**: Voronoi-tessellated plate layout, plate metadata
  (type, centroid, motion heading, speed, cell count), and per-cell
  boundary classification (`convergent`, `divergent`, `transform`)
  derived from relative plate motion. Continental vs oceanic plate
  composition drives interior elevation; boundaries drive localised
  uplift or rifting.
- **Geography**: regular-grid elevation derived from plate interiors
  plus boundary uplift/rift plus deterministic per-cell noise.
- **Hydrology**: aggregate surface-water fraction and headwater
  candidate counts derived from elevation and precipitation. No river
  network, no watersheds, no discharge.
- **Atmosphere**: per-cell atmospheric pressure via the barometric
  formula, with linear extrapolation into ocean basins. Pressure
  varies with elevation, so the field is a grid, not a constant.
- **Climate**: regular-grid temperature and annual precipitation,
  derived from elevation, latitude, and a `climate_class` parameter.
- **Biomes**: a coarse per-cell classification (`ocean`, `ice`,
  `alpine`, `desert`, `tropical-forest`, `temperate-forest`,
  `grassland`) derived from the three physical layers.

## What this version explicitly does NOT model

Phase 1a adds the geology seam; it does not yet simulate:

- Real hydrology (river network, flow direction, discharge,
  watersheds, salinity, aquifers).
- Atmosphere beyond barometric pressure (no composition, prevailing
  winds, storms, or seasons).
- Geological sublayers (rock types, ore distribution, soil types,
  subduction chemistry).
- Astronomy (star, planets/moons, orbital elements, axial tilt,
  day length, year length, eclipses, tides, auroras).
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

## Breaking changes from Phase 0

- RNG namespaces are now layer-prefixed (`geography.elevation`,
  `geography.plate.*`, `climate.precipitation`, ...). The
  `DETERMINISTIC_ALGORITHM_VERSION` constant bumps to
  `tectonic-plates-v1`; persisted Phase 0 worlds do not round-trip.
  The canonical demo `world_id` for `--seed 42` therefore changes.
- `WorldModel` now includes a `geology: GeologyLayer` field and a
  `LARGE` scale (`256×128`).
- `ClimateLayer.atmospheric_pressure_kpa` is now a 2D grid, not a
  scalar constant.
- `WorldConfig` gains a `plate_count` knob (default `12`,
  range `[MINIMUM_PLATE_COUNT, MAXIMUM_PLATE_COUNT]`).

## Generation properties

- **Reproducibility**: same seed, scale, climate class, sentience,
  magic flag, and plate count → byte-identical serialized world.
- **Parametric control**: knobs (`--seed`, `--scale`, `--climate`,
  `--sentience`, `--magic`, `--plate-count`) all take effect;
  sentience and magic are carried through `WorldConfig` for
  downstream phases but do not influence Phase 1a outputs.
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
    --sentience --no-magic --plate-count 12 --out worlds/demo.json
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
├── atmosphere.py      # barometric pressure grid
├── geology.py         # Voronoi plate generation, boundary classification
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