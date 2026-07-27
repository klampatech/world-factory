# Changelog

All notable changes to World Factory are documented here. The format is
loosely based on [Keep a Changelog](https://keepachangelog.com), and
the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
