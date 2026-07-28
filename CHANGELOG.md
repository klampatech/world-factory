# Changelog

All notable changes to World Factory are documented here. The format is
loosely based on [Keep a Changelog](https://keepachangelog.com), and
the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
