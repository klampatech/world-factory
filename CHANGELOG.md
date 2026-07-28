# Changelog

All notable changes to World Factory are documented here. The format is
loosely based on [Keep a Changelog](https://keepachangelog.com), and
the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
