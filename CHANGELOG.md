# Changelog

All notable changes to World Factory are documented here. The format is
loosely based on [Keep a Changelog](https://keepachangelog.com), and
the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added — Phase 3a.4 demography / population pools, migrations, events

- `world_factory.demography` module: in-tree aggregate population
  model (no new pyproject deps). Birth rate from climate optimum +
  capacity headroom; death rate from climate stress + over-capacity
  + per-step conflict tension; migration along infrastructure road
  edges with pressure × cost-factors. Emits `BIRTH / DEATH /
  MIGRATION` typed events per `PHASE_3A_TYPES.md` discriminated
  payload contract.
- New types on `models.py` (steps 1-2 of `PHASE_3A_TYPES.md`
  adoption path):
  - `EventType` StrEnum (BIRTH, DEATH, MIGRATION for 3a.4; others
    reserved for follow-up phases).
  - `EventLocation` (cell + settlement_id).
  - `EventActor` (kind + identifier + display_name).
  - `BirthPayload`, `DeathPayload`, `MigrationPayload`
    (discriminated payloads).
  - `WorldEvent` (id, type, t, location, actors, payload, causes,
    provenance). Event ids are 16-char blake2b hex per
    `PHASE_3A_TYPES.md` Option A recommendation with per-type +
    (t, settlement_id) salt.
  - `PopulationPool` (settlement_id, populations tuple of length
    `time_steps + 1`).
  - `MigrationRecord` (from_settlement_id, to_settlement_id, step,
    count, road_cost).
  - `DemographyLayer` (pools, migrations, events).
- `WorldModel` gains the `demography: DemographyLayer` field.
- New constants: `DEMOGRAPHY_ALGORITHM_VERSION =
  "aggregate-pools-v1"`, `DEMOGRAPHY_DEFAULT_TIME_STEPS = 50`,
  `DEMOGRAPHY_BASE_BIRTH_RATE = 0.04`,
  `DEMOGRAPHY_BASE_DEATH_RATE = 0.03`,
  `DEMOGRAPHY_CAPACITY_HEADROOM_BIRTH_BOOST = 0.02`,
  `DEMOGRAPHY_OVER_CAPACITY_DEATH_PENALTY = 0.02`,
  `DEMOGRAPHY_CLIMATE_OPTIMUM_CELSIUS = 18.0`,
  `DEMOGRAPHY_CLIMATE_RANGE_CELSIUS = 18.0`,
  `DEMOGRAPHY_CONFLICT_THRESHOLD = 0.6`,
  `DEMOGRAPHY_CONFLICT_DEATH_MULTIPLIER = 1.5`,
  `DEMOGRAPHY_MIGRATION_PRESSURE_FACTOR = 0.10`,
  `DEMOGRAPHY_MIGRATION_PULL_FACTOR = 0.05`,
  `DEMOGRAPHY_MIGRATION_COST_DIVISOR = 50.0`.
- New demography `ProvenanceRecord`
  (`output_path="demography"`,
  `process="aggregate-pools-with-climate-capacity-conflict-migration"`,
  `algorithm_version="aggregate-pools-v1"`,
  `input_paths=(settlements, agriculture, infrastructure.roads,
  climate.temperature_celsius, metadata.config.seed)`).
- New validator `validate_demography_layer(world)`: pools parallel
  to settlements by id, populations non-negative, migrations
  reference valid settlement ids, events reference valid
  settlement ids.
- Cross-phase integration: 3a.4 consumes 3a.2 agriculture carrying
  capacity (Malthusian ceiling for birth boost / over-capacity
  death penalty), 3a.3 infrastructure roads (migration paths),
  3a.1 climate (temperature drives birth/death climate factors).
  Downstream 3a.5 EventLog will promote `DemographyLayer.events`
  to a top-level `events: EventLog` on `WorldModel`.
- Decisions on the three open questions (per the channel thread):
  - Conflict seed derived deterministically from
    `metadata.config.seed` via the existing namespace pattern
    (`sample_unit_interval(seed, "demography.conflict",
    settlement_id, step)`); no new `WorldConfig` field. Avoids a
    world_id re-hash across 3a.1 / 3a.2 / 3a.3 stability tests.
  - In-tree aggregate model (no `mesa` dep); ~150 LOC keeps the
    dep surface at `pydantic==2.11.7` alone. Can migrate to actual
    Mesa later if agent-style scheduling is needed.
  - DoD scope: birth/death from climate + capacity, migration
    along infrastructure edges, `BIRTH/DEATH/MIGRATION` event
    emission. Language spread and urbanization deferred to a
    follow-up slice (they need the EventLog foundation first).
- New tests: `tests/test_demography.py` (24 tests) covering layer
  presence, three-collection shape, deterministic reproducibility,
  world_id stability (`9d75e7...` for `--seed 42 --scale large`
  unchanged across 3a.2 → 3a.4), schema_version bump
  (`10.0.0` → `11.0.0`), pools parallel to settlements by id,
  populations non-negative / finite, time-series length
  (time_steps + 1), provenance record presence, validator empty on
  valid worlds, validator flags length / settlement-id
  mismatches, validator flags unknown settlement ids in migrations,
  event types restricted to BIRTH / DEATH / MIGRATION,
  event ids 16-char hex, event actors present, migrations only
  along road edges (archipelago-aware), migration count
  non-negative, settlement with pop=0 doesn't crash, runs cleanly
  on SMALL / MEDIUM, seed variation produces different outputs,
  births ≤ deaths under over-capacity regime, populations
  decline under over-capacity.

### Changed — Phase 3a.4 schema bump

- `SCHEMA_VERSION` bumps `10.0.0` → `11.0.0` (breaking:
  `WorldModel` gains required `demography` field).
  `MODEL_VERSION` unchanged at `phase-3.1`. No new
  `WorldConfig` fields, so `world_id` for `--seed 42` is
  unchanged from Phase 3a.3 / 3a.2 / v1-demo /
  `9d75e7103b52704b48ce77071a22a586`.
- Schema-version policy (continued from 3a.2): `SCHEMA_VERSION`
  is bumped on every additive-required-field change to
  `WorldModel`. Future required-field additions will go
  `11.0.0` → `12.0.0`.
- `V1DemoReport` does not surface the `demography` layer; this
  remains deferred to the explorer integration PR (Finding B from
  the 3a.2 review), matching the same deferral for agriculture
  and infrastructure.

### Added — Phase 3a.3 infrastructure / roads, ports, canals

- `world_factory.infrastructure` module: roads, ports, and canals
  derived from the friction layer (biome × slope × river crossing).
  Roads are a K-NN sparse graph over Dijkstra-minimum-cost paths
  between settlements (canonical direction `from < to`, deduped).
  Ports mark settlements within coastal-proximity radius of any
  ocean cell (COASTAL) or river-proximity radius of any river path
  cell (RIVER), filtered by tonnage threshold. Canals connect
  surplus-positive settlement pairs sharing a flow- and
  slope-feasible river segment.
- `RoadEdge` model: `id`, `from_settlement_id`,
  `to_settlement_id`, `cost`, `path_length`.
- `Port` model: `id`, `settlement_id`, `port_kind` (RIVER or
  COASTAL), `annual_tonnage`.
- `Canal` model: `id`, `from_settlement_id`, `to_settlement_id`,
  `cost`, `mean_flow`, `mean_slope`.
- `InfrastructureLayer` carries a tuple each of roads, ports,
  canals. `WorldModel` gains the `infrastructure:
  InfrastructureLayer` field.
- New constants: `INFRASTRUCTURE_ALGORITHM_VERSION =
  "min-cost-friction-v1"`, `INFRASTRUCTURE_BASE_FRICTION_PER_BIOME`
  (biome-keyed friction coefficients), `INFRASTRUCTURE_IMPASSABLE =
  1e9`, `INFRASTRUCTURE_SLOPE_PENALTY_PER_METER = 0.0015`,
  `INFRASTRUCTURE_RIVER_CROSSING_PENALTY = 6.0`,
  `INFRASTRUCTURE_DIAGONAL_COST = sqrt(2)`,
  `INFRASTRUCTURE_ROAD_NEIGHBOR_K = 3`,
  `INFRASTRUCTURE_COASTAL_RADIUS_CELLS = 1`,
  `INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS = 2`,
  `INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD = 1.0`,
  `INFRASTRUCTURE_PORT_TONNAGE_PER_POPULATION = 1.0`,
  `INFRASTRUCTURE_MAX_CANALS = 8`,
  `INFRASTRUCTURE_CANAL_SLOPE_LIMIT_M_PER_CELL = 5000.0`,
  `INFRASTRUCTURE_CANAL_MIN_FLOW = 50_000_000.0`.
- New infrastructure `ProvenanceRecord`
  (`output_path="infrastructure"`,
  `process="min-cost-friction-with-knn-snap"`,
  `algorithm_version="min-cost-friction-v1"`,
  `input_paths=(settlements, agriculture, hydrology, geography,
  biomes)`).
- New validator `validate_infrastructure_layer(world)`: settlement
  ids resolve, road edges canonical, port tonnage finite / non-neg,
  canal cost / flow / slope finite / non-neg.
- Cross-phase integration: 3a.3 consumes 3a.2 agriculture surplus
  for "roads connect economic centers" and canal production-zone
  gating. Downstream 3a.4 demography will consume road graph edges
  for migration.
- New tests: `tests/test_infrastructure.py` (28 tests) covering
  layer presence, three-collection shape, deterministic
  reproducibility across runs, world_id stability (`9d75e7...` for
  `--seed 42 --scale large` unchanged), schema_version bump
  (`10.0.0`), road cost / direction / pair uniqueness / positive
  path length, port tonnage / kind / settlement-id validity, canal
  cost / flow / slope / direction / settlement-id validity,
  provenance record presence, validator empty on valid worlds,
  validator flags unknown settlement / bad road direction /
  non-finite tonnage / non-finite flow, roads connect
  surplus-positive settlement pairs (cross-phase), graph
  connectivity on seed=42 LARGE (>= 35/36 connected), port count
  respects coastal-proximity geography, ocean barrier blocks road
  path between two settlements (no crash), canals link surplus
  production zones along rivers, infrastructure runs cleanly on
  SMALL / MEDIUM grids.

### Changed — Phase 3a.3 schema bump

- `SCHEMA_VERSION` bumps `9.0.0` → `10.0.0` (breaking:
  `WorldModel` gains required `infrastructure` field).
  `MODEL_VERSION` unchanged at `phase-3.1`. No new
  `WorldConfig` fields, so `world_id` for `--seed 42` is
  unchanged from Phase 3a.2 / v1-demo /
  `9d75e7103b52704b48ce77071a22a586`.
- Schema-version policy (continued from 3a.2): `SCHEMA_VERSION`
  is bumped on every additive-required-field change to
  `WorldModel`. Future required-field additions will go
  `10.0.0` → `11.0.0`.
- `V1DemoReport` does not surface the `infrastructure` layer; this
  remains deferred to the explorer integration PR (Finding B from
  the 3a.2 review), matching the same deferral for agriculture.

### Added — v2 visual explorer (first slice, no schema bump)

- `world_factory.explorer` package: static HTML + vanilla JS visual
  explorer for the v1 demo JSON. Renders per-cell biomes with four
  overlay toggles (biome / elevation / rivers / settlements) and
  a click-to-`CellSummary` side panel. Zero new runtime
  dependencies (no Pillow, no JS framework); the page uses native
  `fetch` + 2D canvas + DOM.
- `ExplorerServer` class: threaded `http.server` rooted at a
  chosen directory, with `find_free_port()` and a context-manager
  interface for tests. CLI subcommand `world-factory serve
  [--directory DIR] [--port PORT] [--host HOST]` runs the same
  server bound to `127.0.0.1` (default port `8765`).
- v1 demo JSON envelope extended with per-cell grids the explorer
  needs: `grid_width`, `grid_height`, `biome_grid` (flat row-major
  tuple of biome names, length = width × height), `river_cells`
  (every river-segment mouth), `settlement_cells` (every
  settlement's (x, y)). Pure additive; existing fields unchanged,
  byte-identical for unchanged inputs.
- Explorer runtime guard: `validateShape()` in the page JS
  fails loudly if `demo.json` is missing required fields or if
  `biome_grid.length` does not match `grid_width * grid_height`.
- Accessibility basics: `role="toolbar"`, `aria-label`,
  `aria-pressed` on overlay buttons, `:focus-visible` outline,
  `data-marker` letter prefix on each toggle (non-color layer
  identification per release gate), and a `role="status"`
  `aria-live="polite"` status line. World data is rendered via a
  local `escapeText()` helper rather than raw `innerHTML`.
- New tests in `tests/test_explorer.py`: HTML well-formedness and
  DOM hooks; vanilla-API-only check; biome color table embedded;
  per-cell grids present in the demo JSON and byte-stable across
  runs; grids agree with `world.biomes.classifications`; HTTP
  serve flow (start server, GET `/index.html`, GET `/demo.json`,
  assert 200 + parseable body); package-data path resolves;
  `find_free_port()` returns distinct ports.
- No new runtime or dev dependencies in `pyproject.toml`. The
  explorer ships as package data; the HTTP server is stdlib
  `http.server`.

### Known limitations (this slice is geo-only)

- The v2 explorer slice is geography-only. It does not yet expose
  polity selection / boundaries, a temporal / event view, or
  causal / provenance drilldown — those land when Phase 3a.2-5,
  Phase 3b, Phase 4, and Phase 5 ship, per
  `RESEARCH/WORLD_FACTORY_V2_RELEASE_GATES.md`. The slice is
  intentionally a first cut, not the final v2 surface.
- The elevation overlay derives a proxy band from biome name
  because `elevation_grid` is not yet in the demo JSON. v2.1
  adds the real grid once Phase 3a lands.
- Click-to-summary returns full `CellSummary` data only for the
  sample polity cell and its 3×3 bioregion (the cells covered by
  the existing v1 demo walkthrough). Other cells show biome + ocean
  flag only; full per-cell summaries land with the persisted-world
  endpoint in v2.1.
### Added — Phase 3a.2 agriculture / caloric accounting

- `world_factory.agriculture` module: per-settlement caloric
  accounting. Each settlement walks its extraction radius
  (`AGRICULTURE_EXTRACTION_RADIUS_CELLS = 2`, Chebyshev
  distance ≤ 2) and accumulates yield from each cell as a
  product of base yield × precipitation response × temperature
  response × soil quality × biome quality. Per-cell yield
  converts to kcal via `AGRICULTURE_CALORIC_KCAL_PER_TONNE`
  (3,000,000 kcal / tonne of cereal-equivalent).
- `carrying_capacity = floor(total_kcal /
  AGRICULTURE_KCAL_PER_PERSON_PER_YEAR)` — the Malthusian
  ceiling: population cannot exceed what the land can feed.
- `agricultural_surplus_kcal_per_year` is signed (positive or
  negative) and represents the kcal delta between current
  population and carrying capacity.
- `seasonal_deficit` flag is set when the settlement has zero
  arable neighbors in its radius OR the worst per-cell yield
  falls below `AGRICULTURE_DEFICIT_YIELD_FRACTION ×
  AGRICULTURE_BASE_YIELD_TONNES_PER_CELL`.
- `AgricultureRecord` model: `settlement_id`,
  `carrying_capacity`, `agricultural_surplus_kcal_per_year`,
  `seasonal_deficit`. `AgricultureLayer` carries a tuple of
  records parallel to `SettlementsLayer.settlements` (same
  length, same order, id-keyed by index).
- `WorldModel` gains the `agriculture: AgricultureLayer`
  field.
- New constants: `AGRICULTURE_ALGORITHM_VERSION =
  "caloric-accounting-v1"`, `AGRICULTURE_EXTRACTION_RADIUS_CELLS
  = 2`, `AGRICULTURE_BASE_YIELD_TONNES_PER_CELL = 1.0`,
  `AGRICULTURE_PRECIPITATION_OPTIMUM_MM = 1000.0`,
  `AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS = 18.0`,
  `AGRICULTURE_TEMPERATURE_RANGE_CELSIUS = 18.0`,
  `AGRICULTURE_SOIL_QUALITY`,
  `AGRICULTURE_BIOME_QUALITY`,
  `AGRICULTURE_CALORIC_KCAL_PER_TONNE = 3_000_000.0`,
  `AGRICULTURE_KCAL_PER_PERSON_PER_YEAR = 800_000.0`,
  `AGRICULTURE_DEFICIT_YIELD_FRACTION = 0.5`,
  `AGRICULTURE_MINIMUM_ARABLE_CELLS = 1`.
- New agriculture `ProvenanceRecord`
  (`output_path="agriculture"`,
  `process="caloric-accounting-with-extraction-radius"`,
  `algorithm_version="caloric-accounting-v1"`).
- New validator `validate_agriculture_layer(world)`: parallel
  records, settlement id matching, non-negative capacity,
  finite surplus, finite precipitation / temperature in every
  cell of every settlement's extraction radius.
- New tests: `tests/test_agriculture.py` (17 tests) covering
  layer presence, parallel-by-id invariants, non-negative
  capacity, finite surplus, deterministic reproducibility,
  world_id stability (`9d75e7103b52704b48ce77071a22a586` for
  `--seed 42 --scale large` unchanged), provenance record
  presence, validator empty on valid worlds, validator flags
  length / settlement-id mismatches, NaN temperature and
  Infinity precipitation fail loudly with cell-coordinate
  paths, zero-arable-neighbor settlements yield `capacity=0`
  and `seasonal_deficit=True`, Malthusian ceiling applied,
  statistical-realism caps, extraction radius = 2,
  temperate-forest / loam / optimum climate yields non-zero
  capacity with no deficit.

### Changed — Phase 3a.2 schema bump

- `SCHEMA_VERSION` bumps `8.0.0` → `9.0.0` (breaking:
  `WorldModel` gains required `agriculture` field).
  `MODEL_VERSION` unchanged at `phase-3.1`. No new
  `WorldConfig` fields, so `world_id` for `--seed 42` is
  unchanged from Phase 3a / v1-demo /
  `9d75e7103b52704b48ce77071a22a586`.
- Schema-version policy: `SCHEMA_VERSION` is bumped on every
  additive-required-field change to `WorldModel`. Today
  that means: any new required field on the root contract.
  Backwards compat: `load_world` uses
  `WorldModel.model_validate_json(..., strict=True)`, so
  historical JSON without the new field fails to parse
  loudly. Persisted-world migrations are out of scope for
  this phase; the next pre-`1.0` release should add a
  migration loader if needed.

### Changed — Phase 3a.2 calibration fix

- `AGRICULTURE_EXTRACTION_RADIUS_CELLS` 2 → 10 (21×21
  extraction window, ~50 km medieval-city hinterland).
- `AGRICULTURE_BASE_YIELD_TONNES_PER_CELL` 1.0 → 4.0
  (acknowledges the abstract unit is not meant to be 1
  tonne of literal wheat per 80 km² cell).
- At these values, seed=42 LARGE produces a power-law-ish
  distribution: 22/36 deficit, 14/36 surplus, mean capacity
  925 vs. mean population 1348. Phase 3a.2 DoD hook 2.x
  (population vs. arable land follows a power-law-ish
  pattern across settlements) is satisfied.
- Calibration is documented in `agriculture.py` module
  docstring.

### Changed — Phase 3a.2 seasonal_deficit semantic

- `seasonal_deficit` now reads the **median** per-cell yield
  in the extraction radius (was: worst per-cell yield).
  Rationale: at radius=10 the 441-cell window guarantees at
  least one cell where the yield factor dips below 0.5 from
  climate variability alone, so the worst-cell flag fired
  near-universally and did not differentiate well-fed from
  starving settlements.
- `AGRICULTURE_DEFICIT_YIELD_FRACTION` 0.5 → 0.25. Empirical
  sweet spot for seed=42 LARGE: 24/36 deficit-True, 12/36
  deficit-False. Flag now tracks "structural food risk" —
  the typical cell in the radius is marginal — distinct from
  pop/cap ratio, which tracks current sustainability.
- Small-scale (SMALL, MEDIUM) grids still flag uniformly
  True because the 21×21 radius exceeds the grid dimensions;
  expected limitation, not a bug.
- New test `test_seasonal_deficit_mixed_distribution` pins
  the threshold behavior so future calibration changes are
  intentional, not silent.

### Added — v1 demo walkthrough

- `world_factory.demo` module: end-to-end world exploration
  walkthrough. `run_v1_demo(seed=42, scale=WorldScale.LARGE)`
  generates a world, validates it, computes summary statistics
  (total cells, ocean / land split, biome counts, settlement
  count, total population, river segment count), picks the
  highest-scoring settlement as the sample polity, walks a 3x3
  bioregion around it, and runs `validate_query_surface` to
  confirm Phase 6 round-trips agree with the underlying data.
- New `V1DemoReport` dataclass: structured output with
  `to_dict()` for JSON serialization.
- New CLI subcommand: `world-factory demo --seed 42 --scale large
  --out demo.json`. Emits the report to stdout AND writes the
  same JSON to `--out`.
- New tests: `tests/test_v1_demo.py` covering happy path,
  biome-count sum invariant, settlement population, 3x3
  bioregion, query-surface round-trip, JSON serialization,
  determinism, different seeds, CLI subprocess, SMALL scale.
- Closes the DoD v1 bar: "Phase 0..2 + Phase 6 query surface
  + a polity demo walkthrough." Phase 3b (Mesa ABM polity
  agency), Phase 4 (ABM scale-up), and Phase 5 (causal graph +
  history) remain deferred to v2 per the product-scope flag raised
  in Phase 6 review.

### Added — Phase 6 query surface (no schema bump)

- `world_factory.queries` module: programmatic world exploration
  API.
- `CellSummary` pydantic model: composite view of a single cell
  (elevation, biome, climate, geology sublayer, biology, river
  segments, settlements).
- `summary_at(world, x, y) -> CellSummary`: single-cell view.
- `settlements_within(world, radius, x, y)`: settlements within
  Chebyshev distance `radius` (matches Phase 1b D8 flow topology).
- `summary_in_bounding_box(world, x_min, y_min, x_max, y_max)`:
  inclusive box walk; clamps to grid bounds.
- `validate_query_surface(world)`: round-trip check that
  `summary_at` on each settlement's cell includes that settlement.
- New tests: `tests/test_queries.py` covering single-cell summary,
  ocean / land detection, settlement radius, bounding-box
  clamping, settlement round-trip, deterministic reproducibility,
  world_id stability across Phase 6.

### Changed — Phase 6 no schema bump

- Pure surface addition. No `WorldConfig` or `WorldModel` shape
  change. `world_id` for `--seed 42` unchanged.

### Added — Phase 3a settlement placement

- `settlements` module: deterministic candidate-scoring placement
  reads Phase 0..2 fields and produces settlements via a coarse
  candidate grid + weighted score + rejection sampling.
- Score weights: 0.30 water_access (proximity to river mouths),
  0.30 arable_land (biome in {TEMPERATE_FOREST, GRASSLAND,
  TROPICAL_FOREST}), 0.10 defensibility (elevation in
  [200m, 1500m]), 0.20 climate_suitability (temperature in
  [5°C, 25°C]), 0.10 mineral_proximity (ore within 3 cells).
- Top-K = max(20, plate_count × 3) candidates with rejection
  sampling on SETTLEMENT_MIN_SPACING_CELLS = 4.
- Settlement population = arable_land × 1000 + water × 500 +
  mineral × 200 (v1 ballpark).
- `Settlement` model: id, x, y, population, founding_score.
- `SettlementsLayer` on `WorldModel`: settlements.
- New constants: SETTLEMENT_CANDIDATE_GRID_DIVISOR=16,
  SETTLEMENT_MIN_COUNT=20, SETTLEMENT_PER_PLATE_COUNT=3,
  SETTLEMENT_MIN_SPACING_CELLS=4, defensibility + climate +
  population constants, SETTLEMENTS_ALGORITHM_VERSION =
  "candidate-scoring-v1".
- New settlements `ProvenanceRecord`
  (`output_path="settlements"`,
  `process="candidate-scoring-with-rejection-sampling"`,
  `algorithm_version="candidate-scoring-v1"`).
- New validation: settlement positions within grid bounds;
  population non-negative; founding_score in [0, 1].
- New tests: `tests/test_settlements.py` covering settlement
  layer presence, grid bounds, positive population, founding
  score range, min count, deterministic reproducibility, scaling
  with plate_count, ocean-cell avoidance, unique IDs, provenance
  record presence, and a direct synthetic-input test for
  `build_settlements`.

### Changed — Phase 3a schema bump

- `SCHEMA_VERSION` bumps `7.0.0` → `8.0.0` (breaking:
  `WorldModel` gains required `settlements` field). `MODEL_VERSION`
  bumps `phase-2.1` → `phase-3.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); new
  `SETTLEMENTS_ALGORITHM_VERSION = "candidate-scoring-v1"` on the
  settlements `ProvenanceRecord`. `world_id` for `--seed 42`
  unchanged (no new `WorldConfig` fields).

### Added — Phase 2 biology

- `biology` module: per-cell flora and fauna assignment by biome.
  Each `BiomeClass` maps to a characteristic `FloraType` and
  `FaunaType`. Ocean cells (elevation ≤ sea_level) capture their
  marine biota via the `ALGAE` flora + `FISH` fauna defaults.
- `FloraType` StrEnum: CONIFER, BROADLEAF, SHRUB, GRASS, MOSS,
  LICHEN, ALGAE, SEAGRASS, CORAL.
- `FaunaType` StrEnum: HERBIVORE_LARGE, HERBIVORE_SMALL,
  CARNIVORE_LARGE, CARNIVORE_SMALL, FISH, BIRD, INSECT, REPTILE.
- `BiologyLayer` model on `WorldModel`: `flora_grid` and
  `fauna_grid`.
- New constants: `BIOLOGY_ALGORITHM_VERSION = "biome-biota-v1"`.
- New biology `ProvenanceRecord` (`output_path="biology"`,
  `process="biome-driven-biota"`,
  `algorithm_version="biome-biota-v1"`).
- New validation: flora / fauna grid shapes match geography;
  flora / fauna values are valid StrEnum members.
- New tests: `tests/test_biology.py` covering grid shapes, StrEnum
  validity, ocean-cell marine biota, deterministic reproducibility,
  world_id stability across Phase 2, flora / fauna diversity, and
  `validate_biology_layer` returning empty on a valid world.

### Changed — Phase 2 schema bump

- `SCHEMA_VERSION` bumps `6.0.0` → `7.0.0` (breaking:
  `WorldModel` gains required `biology` field). `MODEL_VERSION`
  bumps `phase-1e.1` → `phase-2.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); new
  `BIOLOGY_ALGORITHM_VERSION = "biome-biota-v1"` on the biology
  `ProvenanceRecord`. `world_id` for `--seed 42` unchanged (no
  new `WorldConfig` fields).
- Biomes now derive from `temperature_base` (pre-seasonal) and
  pre-refinement `precipitation` (was: post-seasonal temperature +
  refined precipitation). Stable ecological properties, not
  instantaneous weather; biology then derives from biomes.
  Borderline cells may classify slightly differently across the
  Phase 1f → 2 boundary; `world_id` itself is unchanged.

### Changed — Biome inputs shifted from instantaneous to long-term averages

### Changed — Phase 1f validator consolidation (no schema bump)

- `validation.py` is split into per-layer validator modules. Each
  layer module exports its own `validate_<layer>_layer(world)`
  function: `world_factory.geology.validate_geology_sublayer_shapes`,
  `world_factory.hydrology.validate_hydrology_layer` (P1),
  `world_factory.atmosphere.validate_atmosphere_layer`,
  `world_factory.astronomy.validate_astronomy_layer`.
- New `world_factory.invariants` module holds the shared
  `InvariantViolation`, `ValidationReport`, and `violation()`
  helper. Lifted out of `validation.py` so per-layer validators
  can import without circular dependency on `validation.py`,
  which itself imports the per-layer validators to orchestrate.
- `validation.py` is now a thin orchestrator holding the
  cross-cutting grid-shape and provenance invariants. The
  orchestrating `validate_world(world)` delegates to each
  per-layer validator.
- Back-compat preserved: `InvariantViolation` and
  `ValidationReport` remain exported from `world_factory.validation`
  via `__all__`.
- Pure refactor; no `WorldConfig` or `WorldModel` shape change,
  no schema bump, `world_id` for `--seed 42` unchanged.
- New tests: `tests/test_validation.py` covers back-compat
  re-exports, per-layer validators are callable,
  `validate_world` returns a valid `ValidationReport`, per-layer
  validators return empty on a valid world, watershed validator
  flags an injected ocean-cell label, `validate_world` is valid
  at all three scales.

### Added — Phase 1e geological sublayers

- Rock-type, ore-presence, and soil-type grids under the Phase 1a
  geology core.
- `RockType` StrEnum: BASALT, GRANITE, SEDIMENTARY, METAMORPHIC,
  VOLCANIC.
- `SoilType` StrEnum: PERMAFROST, SAND, LOAM, CLAY, PEAT.
- `GeologyLayer` gains `rock_type_grid`,
  `ore_presence_grid: tuple[tuple[bool, ...], ...]`,
  `soil_type_grid`.
- Rock type per cell is a deterministic function of (plate type,
  boundary type, elevation): oceanic interiors → BASALT;
  continental interiors → GRANITE (high) or SEDIMENTARY (low);
  convergent boundaries → VOLCANIC; divergent → BASALT;
  transform → METAMORPHIC.
- Ore presence per cell: probability scales with rock type
  (volcanic=0.4, granite=0.2, basalt=0.1, sedimentary=0.15,
  metamorphic=0.25) and proximity to plate boundaries (computed via
  BFS in `_boundary_distance_grid`); cells crossing
  `MINIMUM_ORE_PROBABILITY = 0.10 × ORE_PROBABILITY_SCALE = 0.4`
  are marked.
- Soil type per cell: PERMAFROST if temperature < -10°C; SAND if
  precipitation < 350mm; PEAT if precipitation ≥ 1400mm; CLAY on
  basalt; LOAM otherwise.
- New constants: `MINIMUM_ORE_PROBABILITY`, `ORE_PROBABILITY_SCALE`,
  `SEDIMENTARY_ELEVATION_CAP_METERS = 500`,
  `PEAT_PRECIPITATION_THRESHOLD_MM = 1400`,
  `LOAM_PRECIPITATION_THRESHOLD_MM = 350`,
  `PERMAFROST_TEMPERATURE_CELSIUS = -10`,
  `GEOLOGY_SUBLEYER_ALGORITHM_VERSION = "rock-ore-soil-v1"`.
- New RNG namespace `geology.ore_presence` for per-cell ore draws.
- New sublayer `ProvenanceRecord`
  (`output_path="geology.sublayers"`,
  `process="rock-ore-soil-tagging"`,
  `algorithm_version="rock-ore-soil-v1"`).
- New tests: `tests/test_geology_sub.py` covering rock-type /
  soil-type StrEnum validity, ore-presence booleanity, oceanic
  plate samples reading as BASALT, continental interior samples
  reading as GRANITE / SEDIMENTARY / VOLCANIC, ore count grows
  with `plate_count`, deterministic reproducibility, world_id
  stability across Phase 1e (no new WorldConfig fields), and
  sublayer `ProvenanceRecord` presence.

### Changed — Phase 1e schema bump

- `SCHEMA_VERSION` bumps `5.0.0` → `6.0.0` (breaking:
  `GeologyLayer` adds required fields). `MODEL_VERSION` bumps
  `phase-1d.1` → `phase-1e.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); the new sublayer algorithm
  version (`rock-ore-soil-v1`) lives on the sublayer
  `ProvenanceRecord`. `world_id` for `--seed 42` is unchanged
  (no new `WorldConfig` fields).

### Added — Phase 1d astronomy

- `astronomy` module: axial-tilt-driven solar declination
  (`δ = T × sin(2π × season_day / orbital_period_days)`),
  per-cell day-length via the standard formula `cos(ω) = −tan(φ) ·
  tan(δ)` with explicit clamping for polar night (argument > 1) and
  midnight sun (argument < −1), per-cell insolation factor
  `max(0, cos(latitude − declination))`.
- `AstronomyLayer` model on `WorldModel`: `axial_tilt_degrees`,
  `orbital_eccentricity`, `season_day`, `solar_declination_degrees`,
  `day_length_hours: tuple[tuple[float, ...], ...]`,
  `insolation_factor: tuple[tuple[float, ...], ...]`.
- `WorldConfig` gains `axial_tilt_degrees`, `orbital_eccentricity`,
  `rotation_period_hours`, `orbital_period_days`, `season_day`
  with Earth-analog defaults (`23.5°`, `0.0167`, `24h`, `365.25d`,
  `0`).
- Generator applies a `SEASONAL_TEMPERATURE_AMPLITUDE = 0.10`
  correction: `T_corrected = T_base × (1 + 0.10 ×
  (insolation_factor − 0.5))`. Equatorial sub-solar cells read
  slightly hotter; antisolar poles slightly cooler.
- New constants: `SEASONAL_TEMPERATURE_AMPLITUDE`,
  `EARTH_AXIAL_TILT_DEGREES`, `EARTH_ORBITAL_ECCENTRICITY`,
  `EARTH_ROTATION_PERIOD_HOURS`, `EARTH_ORBITAL_PERIOD_DAYS`,
  `ASTRONOMY_ALGORITHM_VERSION = "axial-tilt-v1"`.
- New astronomy `ProvenanceRecord` (`output_path="astronomy"`,
  `process="axial-tilt-with-seasonal-forcing"`,
  `algorithm_version="axial-tilt-v1"`).
- New validation: day-length in `[0, 24]`, insolation in `[0, 1]`,
  solar declination in `[−axial_tilt, +axial_tilt]`, grid shapes
  match geography.
- New tests: `tests/test_astronomy.py` covering declination at
  equinox / solstice, day-length at polar regions (midnight sun and
  polar night), insolation at sub-solar / antisolar points,
  seasonal correction propagation, deterministic reproducibility,
  shape parity with geography.

### Changed — Phase 1d schema bump

- `SCHEMA_VERSION` bumps `4.0.0` → `5.0.0` (breaking:
  `WorldModel` gains required `astronomy` field; `WorldConfig` gains
  five new fields with defaults). `MODEL_VERSION` bumps
  `phase-1c.1` → `phase-1d.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); the new astronomy algorithm
  version (`axial-tilt-v1`) lives on the astronomy
  `ProvenanceRecord`. `world_id` for `--seed 42` changes (because
  the config hash now includes the new fields); recorded as
  breaking in CHANGELOG.

### Added — Phase 1c atmosphere recursion

- `atmosphere` module: three-cell circulation model (Hadley at 0-30,
  Ferrel at 30-60, polar easterlies at 60-90), per-cell prevailing
  surface wind (direction is the direction the wind blows TOWARD:
  Hadley trades blow west, Ferrel westerlies blow east, polar
  easterlies blow west), sea-breeze modulation for coastal cells
  based on adjacent temperature contrast, ocean evaporation via
  Magnus-Tetens saturation vapor pressure, wind-driven humidity
  transport over 32 iterations (bounded emission so humidity cannot
  accumulate without limit), orographic precipitation boost, refined
  precipitation grid blending the Phase 1a noise field with
  transport-driven moisture.
- `WindDirection` StrEnum: EAST, WEST, NORTH, SOUTH, NORTH_EAST,
  NORTH_WEST, SOUTH_EAST, SOUTH_WEST, CALM.
- `ClimateLayer` extended with `wind_direction_grid: tuple[tuple[WindDirection, ...], ...]`
  and `specific_humidity_grid: tuple[tuple[float, ...], ...]`. Existing
  `atmospheric_pressure_kpa`, `temperature_celsius`,
  `annual_precipitation_mm` retained; pressure now includes a small
  humidity buoyancy correction so moist air reads lighter than dry air
  at the same elevation.
- New constants: `MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG = 0.030`,
  `WIND_BELT_HADLEY_DEGREES`, `WIND_BELT_FERREL_DEGREES`,
  `SEA_BREEZE_TEMPERATURE_DELTA_CELSIUS`,
  `EVAPORATION_WIND_COEFFICIENT`, `TRANSPORT_ITERATIONS = 32`,
  `BASE_PRECIPITATION_LOSS`, `OROGRAPHIC_BOOST_DIVISOR_METERS`,
  `PRESSURE_HUMIDITY_BUOYANCY`, `PRECIPITATION_REFINEMENT_BLEND`,
  `ATMOSPHERE_ALGORITHM_VERSION = "wind-belts-v1"`.
- New climate `ProvenanceRecord` (`output_path="climate"`,
  `process="wind-belts-with-transport"`, `algorithm_version="wind-belts-v1"`).
- New validation: wind directions are valid StrEnum values;
  specific humidity is in `[0, MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG]`.
- New tests: `tests/test_atmosphere.py` covering belt assignment by
  latitude, evaporation on ocean cells only, transport boundedness,
  refined precipitation non-negativity, deterministic reproducibility,
  provenance record presence.

### Changed — Phase 1c schema bump

- `SCHEMA_VERSION` bumps `3.0.0` -> `4.0.0` (breaking:
  `ClimateLayer` adds required fields). `MODEL_VERSION` bumps
  `phase-1b.1` -> `phase-1c.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); the new atmospheric algorithm
  version (`wind-belts-v1`) is recorded per-process on the climate
  `ProvenanceRecord`. `world_id` for `--seed 42` is unchanged from
  Phase 1b.

### Added — Phase 1b hydrology

- `hydrology` module: D8 flow direction with sink-routing, flow
  accumulation via descending-elevation topological sort, multi-source
  BFS for nearest-ocean fallback (handles flow-direction cycles),
  per-cell discharge in m³/year, headwater identification, river
  segmentation, watershed delineation.
- `RiverSegment` model: `id`, `source`, `mouth`, `length_cells`,
  `mean_discharge`, `mean_slope`, `watershed_id`.
- `HydrologyLayer` extended with `river_segments: tuple[RiverSegment, ...]`,
  `discharge_grid: tuple[tuple[float, ...], ...]`, and
  `watershed_id_grid: tuple[tuple[int | None, ...], ...]`. Existing
  `surface_water_fraction` and `headwater_candidate_count` retained
  for backwards compatibility.
- New constants: `MINIMUM_HEADWATER_BASIN_CELLS`,
  `MINIMUM_RUNOFF_PRECIPITATION_MM`, `MINIMUM_HEADWATER_ELEVATION_METERS`,
  `RUNOFF_COEFFICIENT`, `GRID_CELL_AREA_KILOMETERS_SQUARED`,
  `HYDROLOGY_ALGORITHM_VERSION = "flow-routing-v1"`.
- New RNG namespace: `hydrology.flow_direction`.
- New hydrology `ProvenanceRecord` (`output_path="hydrology"`,
  `algorithm_version="flow-routing-v1"`).
- New validation rules: P1 hydrographic consistency (river mouths at
  sea level; positive river length and discharge; ocean cells carry no
  watershed id; land cells carry a watershed id).
- New tests: `tests/test_hydrology.py` covering river network structure,
  discharge monotonicity, watershed integrity, deterministic
  reproducibility, and P1 invariant.

### Changed — Phase 1b schema bump

- `SCHEMA_VERSION` bumped `2.0.0` → `3.0.0` (breaking:
  `HydrologyLayer` adds required fields). `MODEL_VERSION` bumped
  `phase-1a.1` → `phase-1b.1`. `DETERMINISTIC_ALGORITHM_VERSION`
  unchanged (`tectonic-plates-v1`); the new hydrology algorithm
  version is recorded per-process in `ProvenanceRecord`.

### Added — Phase 1a geography core

- `geology` module: Voronoi-tessellated tectonic plate generation
  with plate metadata (`PlateRecord`) and per-cell boundary
  classification (`BoundaryRecord`, `BoundaryType`).
- `GeologyLayer` model field exposed on `WorldModel`; new
  `WorldScale.LARGE` (256×128) for v1 demo worlds.
- `atmosphere` module: barometric-formula atmospheric pressure grid
  (`atmospheric_pressure_grid`).
- `ClimateLayer.atmospheric_pressure_kpa` promoted to a 2D grid.
- `WorldConfig.plate_count` knob (default `12`, bounded by
  `MINIMUM_PLATE_COUNT` / `MAXIMUM_PLATE_COUNT`).
- CLI `--plate-count` flag on `world-factory generate`.
- RNG namespaces migrated to layer-prefixed form
  (`geography.elevation`, `geography.plate.*`, `climate.precipitation`).
- Deterministic-algorithm-version bumped to `tectonic-plates-v1`.
- Plausibility invariant: atmospheric pressure ≥
  `MINIMUM_ATMOSPHERIC_PRESSURE_KPA` per cell.
- New tests: `tests/test_geology.py` covering plate count bounds,
  boundary grid, geology/GeographyLayer shape parity, atmospheric
  pressure bounds, and minimum plate interior cell count.
- Parametric smoke test now sweeps `plate_count` in
  `{MINIMUM_PLATE_COUNT, default, MAXIMUM_PLATE_COUNT}` so the new
  knob is verified across its range.

### Changed

- `SCHEMA_VERSION` bumped to `2.0.0` — Phase 1a changes the
  on-disk shape (`ClimateLayer.atmospheric_pressure_kpa` becomes
  a grid; new `GeologyLayer` field), so persisted Phase 1a worlds
  do not round-trip back to Phase 0. Aligns with PHASE_3A_TYPES
  §schema-versioning.
- Continental/oceanic plate draw threshold tuned to `0.45` to keep
  ocean fraction in Earth-analog range across `plate_count` from
  `MINIMUM_PLATE_COUNT` to `MAXIMUM_PLATE_COUNT` at all scales.
- `generator._generate_elevation` applies a stronger
  `CONVERGENT_BOUNDARY_UPLIFT_METERS` when both adjacent plates
  are continental (mountain belt) and a reduced uplift on
  oceanic-convergent (island arc) boundaries.
- `generator.py` rewritten to derive elevation from plate interiors
  and boundaries, replacing the Phase 0 latitude-wave + per-cell
  noise placeholder.
- README expanded with Phase 1a coverage, breaking-change notice,
  and updated project structure.

### Breaking

- Phase 0 persisted worlds do not round-trip. The canonical demo
  `--seed 42` `world_id` is now derived from the new
  `tectonic-plates-v1` algorithm; treat prior `world_id` references
  as stale.

## [0.1.0] — Phase 0 seam

### Added

- Typed `WorldModel` pydantic contract (geography, hydrology,
  climate, biomes, metadata, provenance).
- Stateless `blake2b` deterministic RNG (`sample_unit_interval`).
- Atomic JSON persistence with strict re-validation on load.
- CLI: `world-factory generate`, `world-factory validate`.
- Plausibility invariants: grid shape, elevation bounds, climate
  bounds, ocean fraction 10–90%, provenance coverage.
- Parametric composition test (scale × climate × sentience × magic).
- Biome `StrEnum` (`BiomeClass`) replacing raw strings.
