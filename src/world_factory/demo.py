"""Phase v1 demo walkthrough — end-to-end reproducible world exploration.

`run_v1_demo(seed=42, scale=WorldScale.LARGE)` generates a world,
validates it, and returns a structured `V1DemoReport` with summary
statistics, a sample polity CellSummary, a sample bioregion
walkthrough, and the per-cell grids the v2 visual explorer needs.
Designed for the DoD v1 bar: "Phase 0..2 + Phase 6 query surface
+ a polity demo walkthrough."

The CLI exposes this as `world-factory demo --seed 42 --scale large
--out demo.json`. The JSON output is the v1 demo walkthrough. The
v2 visual explorer reads the same JSON over HTTP.
"""

from dataclasses import dataclass, field
from typing import TypedDict

from world_factory.generator import generate_world
from world_factory.models import (
    WorldConfig,
    WorldModel,
    WorldScale,
)
from world_factory.queries import (
    CellSummary,
    summary_at,
    summary_in_bounding_box,
    validate_query_surface,
)
from world_factory.validation import validate_world


class _BiomeCount(TypedDict):
    biome: str
    cells: int


@dataclass(frozen=True)
class V1DemoReport:
    """Structured output of the v1 demo walkthrough."""

    seed: int
    scale: str
    world_id: str
    schema_version: str
    model_version: str
    is_valid: bool
    total_cells: int
    ocean_cells: int
    land_cells: int
    surface_water_fraction: float
    biome_counts: tuple[_BiomeCount, ...]
    settlement_count: int
    total_population: int
    river_segment_count: int
    sample_polity_summary: CellSummary
    sample_bioregion_summaries: tuple[CellSummary, ...] = field(default_factory=tuple)
    query_surface_validates: bool = True
    # v2 visual-explorer payload. Per-cell biome grid (flat row-major)
    # plus the cell sets the explorer needs for river / settlement
    # overlays. Size at LARGE (256x128) is ~330 KB; well under the
    # SQLite threshold in `RESEARCH/EXPLORATION_SURFACE.md`.
    grid_width: int = 0
    grid_height: int = 0
    biome_grid: tuple[str, ...] = ()
    river_cells: tuple[tuple[int, int], ...] = ()
    settlement_cells: tuple[tuple[int, int], ...] = ()

    def to_dict(self) -> dict[str, object]:
        return {
            "seed": self.seed,
            "scale": self.scale,
            "world_id": self.world_id,
            "schema_version": self.schema_version,
            "model_version": self.model_version,
            "is_valid": self.is_valid,
            "total_cells": self.total_cells,
            "ocean_cells": self.ocean_cells,
            "land_cells": self.land_cells,
            "surface_water_fraction": self.surface_water_fraction,
            "biome_counts": [dict(entry) for entry in self.biome_counts],
            "settlement_count": self.settlement_count,
            "total_population": self.total_population,
            "river_segment_count": self.river_segment_count,
            "sample_polity_summary": self.sample_polity_summary.model_dump(mode="json"),
            "sample_bioregion_summaries": [
                s.model_dump(mode="json") for s in self.sample_bioregion_summaries
            ],
            "query_surface_validates": self.query_surface_validates,
            "grid_width": self.grid_width,
            "grid_height": self.grid_height,
            "biome_grid": list(self.biome_grid),
            "river_cells": [list(cell) for cell in self.river_cells],
            "settlement_cells": [list(cell) for cell in self.settlement_cells],
        }


def _count_biomes(world: WorldModel) -> tuple[_BiomeCount, ...]:
    counter: dict[str, int] = {}
    for row in world.biomes.classifications:
        for biome in row:
            counter[biome.value] = counter.get(biome.value, 0) + 1
    return tuple(
        _BiomeCount(biome=name, cells=count)
        for name, count in sorted(counter.items(), key=lambda pair: -pair[1])
    )


def _summary_statistics(world: WorldModel) -> tuple[int, int, int, float]:
    width = world.geography.width
    height = world.geography.height
    sea_level = world.geography.sea_level_meters
    total = width * height
    ocean = sum(
        1
        for y in range(height)
        for x in range(width)
        if world.geography.elevation_meters[y][x] <= sea_level
    )
    return total, ocean, total - ocean, ocean / total if total else 0.0


def _flatten_biome_grid(world: WorldModel) -> tuple[str, ...]:
    """Return the per-cell biome grid as a flat row-major tuple of
    biome-name strings. Order: row 0 col 0, row 0 col 1, ...,
    row (height-1) col (width-1). Total length = width * height."""
    return tuple(
        cell.value
        for row in world.biomes.classifications
        for cell in row
    )


def _collect_river_cells(world: WorldModel) -> tuple[tuple[int, int], ...]:
    """Return every river-segment mouth as (x, y). The current
    hydrology model routes rivers to ocean mouths, so this is the
    set of coastal cells with rivers. Ordered deterministically by
    segment id so the JSON output is byte-stable."""
    return tuple(
        (segment.mouth[0], segment.mouth[1])
        for segment in sorted(world.hydrology.river_segments, key=lambda s: s.id)
    )


def _collect_settlement_cells(world: WorldModel) -> tuple[tuple[int, int], ...]:
    """Return every settlement's (x, y), ordered by settlement id
    for deterministic byte-stable output."""
    return tuple(
        (settlement.x, settlement.y)
        for settlement in sorted(world.settlements.settlements, key=lambda s: s.id)
    )


def run_v1_demo(
    seed: int = 42, scale: WorldScale = WorldScale.LARGE
) -> V1DemoReport:
    """Run the v1 demo walkthrough on a generated world.

    Steps:
    1. Generate the world from `WorldConfig(seed, scale)`.
    2. Validate the world (cross-cutting + per-layer invariants).
    3. Compute summary statistics (cells, ocean / land split,
       biome counts, settlement count, total population, river
       segment count).
    4. Pick the highest-scoring settlement as the sample polity
       and emit its `CellSummary`.
    5. Walk a 3x3 bioregion around the sample polity and emit
       each cell's `CellSummary`.
    6. Run `validate_query_surface` to confirm Phase 6 round-trips
       agree with the underlying data.
    7. Flatten the per-cell biome grid and collect river /
       settlement cell sets for the v2 visual explorer.
    """
    config = WorldConfig(seed=seed, scale=scale)
    world = generate_world(config)
    report = validate_world(world)
    total, ocean, land, surface_water = _summary_statistics(world)
    biome_counts = _count_biomes(world)
    settlements = world.settlements.settlements
    total_population = sum(s.population for s in settlements)
    if settlements:
        sample = max(settlements, key=lambda s: s.founding_score)
        sample_summary = summary_at(world, sample.x, sample.y)
        half_box = 1
        bioregion = tuple(
            summary_in_bounding_box(
                world,
                sample.x - half_box,
                sample.y - half_box,
                sample.x + half_box,
                sample.y + half_box,
            )
        )
    else:
        sample_summary = summary_at(world, 0, 0)
        bioregion = (sample_summary,)
    query_violations = validate_query_surface(world)
    return V1DemoReport(
        seed=seed,
        scale=scale.value,
        world_id=world.metadata.world_id,
        schema_version=world.metadata.schema_version,
        model_version=world.metadata.model_version,
        is_valid=report.is_valid,
        total_cells=total,
        ocean_cells=ocean,
        land_cells=land,
        surface_water_fraction=round(surface_water, 6),
        biome_counts=biome_counts,
        settlement_count=len(settlements),
        total_population=total_population,
        river_segment_count=len(world.hydrology.river_segments),
        sample_polity_summary=sample_summary,
        sample_bioregion_summaries=bioregion,
        query_surface_validates=not query_violations,
        grid_width=world.geography.width,
        grid_height=world.geography.height,
        biome_grid=_flatten_biome_grid(world),
        river_cells=_collect_river_cells(world),
        settlement_cells=_collect_settlement_cells(world),
        # Document the settlements_within call used by the demo
        # so a future v1.5 can use it for a 3-ring walk.
        # settlements_within(world, half_box + 1, sample.x, sample.y)
    )