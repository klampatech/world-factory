# World Factory

Deterministic, provenance-aware world simulation foundation. Ships a
typed `WorldModel` contract, a stateless seeded RNG, atomic JSON
persistence, a CLI, and cross-layer plausibility invariants.

This is the **Phase 1b release**. It introduces the river network,
per-cell discharge, and watershed delineation on top of the Phase 1a
geology core. RNG namespaces and `DETERMINISTIC_ALGORITHM_VERSION`
remain `tectonic-plates-v1` (geology is unchanged); the new hydrology
algorithm is recorded per-process in `ProvenanceRecord` as
`flow-routing-v1`. The `SCHEMA_VERSION` bumps to `3.0.0` because
`HydrologyLayer` adds required fields. It still does not simulate
biology, society, politics, or history; those land in later phases.

## What this version claims to model

- **Geology**: Voronoi-tessellated plate layout, plate metadata
  (type, centroid, motion heading, speed, cell count), and per-cell
  boundary classification (`convergent`, `divergent`, `transform`)
  derived from relative plate motion. Continental vs oceanic plate
  composition drives interior elevation; boundaries drive localised
  uplift or rifting.
- **Geography**: regular-grid elevation derived from plate interiors
  plus boundary uplift/rift plus deterministic per-cell noise.
- **Hydrology**: D8 flow direction with sink-routing to lowest
  neighbour, flow accumulation via descending-elevation topological
  sort, per-cell discharge in m³/year, headwater identification, river
  segmentation traced from headwater to ocean mouth, watershed
  delineation by ocean-distance BFS, and the legacy Phase 0
  surface-water-fraction + headwater-candidate aggregates.
- **Atmosphere**: per-cell atmospheric pressure via the barometric
  formula, with linear extrapolation into ocean basins. Pressure
  varies with elevation, so the field is a grid, not a constant.
- **Climate**: regular-grid temperature and annual precipitation,
  derived from elevation, latitude, and a `climate_class` parameter.
- **Biomes**: a coarse per-cell classification (`ocean`, `ice`,
  `alpine`, `desert`, `tropical-forest`, `temperate-forest`,
  `grassland`) derived from the three physical layers.

## What this version explicitly does NOT model

Phase 1b ships the river network; it does not yet simulate:

- Tributary splitting (each headwater is one segment to its terminal
  ocean; confluence cells are not split into per-source tributaries).
- Lakes, groundwater, irrigation, soil moisture (Phase 1e
  geological sublayers will add soil; hydrology extension
  follows).
- Atmosphere beyond barometric pressure (no composition, prevailing
  winds, storms, or seasons).
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

## Breaking changes from Phase 1a

- `SCHEMA_VERSION` bumps `2.0.0` → `3.0.0`. `HydrologyLayer` adds
  required fields (`river_segments`, `discharge_grid`,
  `watershed_id_grid`); Phase 1a persisted worlds do not
  round-trip.
- `MODEL_VERSION` bumps `phase-1a.1` → `phase-1b.1`.
- `DETERMINISTIC_ALGORITHM_VERSION` stays `tectonic-plates-v1`;
  the new hydrology algorithm version (`flow-routing-v1`) lives on
  the hydrology `ProvenanceRecord`. The canonical demo
  `world_id` for `--seed 42` is unchanged from Phase 1a.

## Generation properties

- **Reproducibility**: same seed, scale, climate class, sentience,
  magic flag, and plate count → byte-identical serialized world.
- **Parametric control**: knobs (`--seed`, `--scale`, `--climate`,
  `--sentience`, `--magic`, `--plate-count`) all take effect;
  sentience and magic are carried through `WorldConfig` for
  downstream phases but do not influence Phase 1b outputs.
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

Validate a persisted world against the Phase 1b invariant set (P1
hydrographic consistency + Phase 0 physical bounds):

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