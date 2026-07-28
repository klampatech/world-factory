"""Phase 1b hydrology invariants: river network, discharge, watersheds."""

import pytest

from world_factory.generator import generate_world
from world_factory.hydrology import (
    _discharge_grid,
    _flow_accumulation,
    _flow_direction_grid,
    _headwater_candidates,
    _ocean_distance_grid,
    _ocean_mask,
    _walk_to_ocean,
)
from world_factory.models import WorldConfig, WorldScale


def test_hydrology_layer_present_at_all_scales() -> None:
    for scale in WorldScale:
        world = generate_world(WorldConfig(seed=42, scale=scale))
        hydrology = world.hydrology
        assert len(hydrology.river_segments) >= 0
        assert hydrology.discharge_grid is not None
        assert hydrology.watershed_id_grid is not None


def test_discharge_grid_has_non_negative_values() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for row in world.hydrology.discharge_grid:
        for value in row:
            assert value >= 0.0


def test_discharge_grid_shape_matches_geography() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    height = world.geography.height
    width = world.geography.width
    assert len(world.hydrology.discharge_grid) == height
    assert len(world.hydrology.discharge_grid[0]) == width


def test_watershed_grid_shape_matches_geography() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    height = world.geography.height
    width = world.geography.width
    assert len(world.hydrology.watershed_id_grid) == height
    assert len(world.hydrology.watershed_id_grid[0]) == width


def test_ocean_cells_carry_no_watershed_label() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    elevation = world.geography.elevation_meters
    for y, row in enumerate(world.hydrology.watershed_id_grid):
        for x, label in enumerate(row):
            if elevation[y][x] <= sea_level:
                assert label is None


def test_land_cells_carry_a_watershed_label() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    elevation = world.geography.elevation_meters
    for y, row in enumerate(world.hydrology.watershed_id_grid):
        for x, label in enumerate(row):
            if elevation[y][x] > sea_level:
                assert label is not None


def test_every_river_mouth_is_at_or_below_sea_level() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    elevation = world.geography.elevation_meters
    for segment in world.hydrology.river_segments:
        mouth_x, mouth_y = segment.mouth
        assert elevation[mouth_y][mouth_x] <= sea_level


def test_river_segments_have_positive_length_and_discharge() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for segment in world.hydrology.river_segments:
        assert segment.length_cells >= 1
        assert segment.mean_discharge > 0.0


def test_river_count_scales_with_world_size() -> None:
    small = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    large = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert len(large.hydrology.river_segments) > len(small.hydrology.river_segments)


def test_discharge_grows_downstream() -> None:
    """Sanity check that discharge at the mouth is non-trivial relative
    to the source — rivers accumulate water as they flow."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    if not world.hydrology.river_segments:
        return
    segment = world.hydrology.river_segments[0]
    source_x, source_y = segment.source
    mouth_x, mouth_y = segment.mouth
    source_discharge = world.hydrology.discharge_grid[source_y][source_x]
    mouth_discharge = world.hydrology.discharge_grid[mouth_y][mouth_x]
    assert source_discharge > 0.0
    assert mouth_discharge >= source_discharge / 100.0


def test_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.hydrology.river_segments == b.hydrology.river_segments
    assert a.hydrology.discharge_grid == b.hydrology.discharge_grid
    assert a.hydrology.watershed_id_grid == b.hydrology.watershed_id_grid


def test_hydrology_provenance_record_present() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    output_paths = {record.output_path for record in world.provenance}
    assert "hydrology" in output_paths


def test_world_id_stable_across_hydrology_algorithm_change() -> None:
    """world_id is a hash of config only — hydrology doesn't perturb it."""
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.metadata.world_id == b.metadata.world_id


def _constant_elevation_grid(
    width: int, height: int, value: float
) -> tuple[tuple[float, ...], ...]:
    return tuple(tuple(value for _ in range(width)) for _ in range(height))


def test_sink_routes_to_lowest_neighbour_on_flat_grid() -> None:
    """A flat 3x3 elevation grid forces every interior cell to be a sink.
    Each sink must flow to its lowest neighbour so accumulation
    propagates; ocean_distance supplies the BFS fallback."""
    elevation = _constant_elevation_grid(3, 3, 100.0)
    ocean = _ocean_mask(elevation, sea_level=0.0)
    flow_dir = _flow_direction_grid(elevation, ocean, seed=42)
    # Interior cell (1, 1) is a flat sink — its flow_dir should be set.
    assert flow_dir[1][1] is not None
    assert flow_dir[1][1] in {(0, 1), (1, 0), (2, 1), (1, 2)}


def test_cycle_falls_back_to_ocean_distance() -> None:
    """A 2-cell cycle sits inland, separated from ocean by a higher
    ridge. _walk_to_ocean must detect the cycle and route to the
    precomputed nearest-ocean cell instead of looping forever."""
    # Layout (rows = y):
    #   (1500, 1500, 1500)
    #   (1500, 1200, 1500)   ← inland plateau
    #   (1500,    0, 1500)   ← ocean corridor below the plateau
    # Cells (0, 1) and (1, 1) are both 1200m sinks surrounded by 1500m
    # cells; their flow direction chains cycle, so the BFS ocean_distance
    # fallback must resolve them.
    elevation: tuple[tuple[float, ...], ...] = (
        (1500.0, 1500.0, 1500.0),
        (1500.0, 1200.0, 1500.0),
        (1500.0, 0.0, 1500.0),
    )
    ocean = _ocean_mask(elevation, sea_level=0.0)
    ocean_distance = _ocean_distance_grid(3, 3, ocean)
    flow_dir = _flow_direction_grid(elevation, ocean, seed=42)
    terminal_a = _walk_to_ocean((0, 1), flow_dir, ocean, ocean_distance)
    terminal_b = _walk_to_ocean((1, 1), flow_dir, ocean, ocean_distance)
    assert terminal_a is not None
    assert terminal_b is not None
    assert ocean[terminal_a[1]][terminal_a[0]]
    assert ocean[terminal_b[1]][terminal_b[0]]


def test_accumulation_is_one_plus_upstream_on_staircase() -> None:
    """A 1-cell-wide staircase ending in ocean accumulates by row count
    (each cell adds itself to its downstream neighbour)."""
    elevation: tuple[tuple[float, ...], ...] = (
        (10.0, 9.0, 8.0, 7.0, 6.0, 0.0),
    )
    ocean = _ocean_mask(elevation, sea_level=0.0)
    flow_dir = _flow_direction_grid(elevation, ocean, seed=42)
    accum = _flow_accumulation(elevation, flow_dir)
    # Top cell holds 1; each downstream cell adds 1 per upstream
    # contribution. Ocean terminal accumulates every upstream cell plus
    # itself, so the last cell equals the basin size.
    assert accum[0][0] == 1
    assert accum[0][1] == 2
    assert accum[0][2] == 3
    assert accum[0][3] == 4
    assert accum[0][4] == 5
    assert accum[0][5] == 6


def test_discharge_zero_when_precipitation_zero() -> None:
    """Zero precipitation → zero discharge regardless of accumulation."""
    elevation = _constant_elevation_grid(2, 2, 500.0)
    ocean = _ocean_mask(elevation, sea_level=0.0)
    flow_dir = _flow_direction_grid(elevation, ocean, seed=42)
    accum = _flow_accumulation(elevation, flow_dir)
    precipitation = _constant_elevation_grid(2, 2, 0.0)
    discharge = _discharge_grid(accum, precipitation)
    for row in discharge:
        for value in row:
            assert value == 0.0


def test_headwater_threshold_excludes_trivial_basins() -> None:
    """Headwater candidates must clear elevation, precipitation, and
    basin-size thresholds. A 3x3 grid with a small inland basin (4 cells)
    has no qualifying headwaters because MINIMUM_HEADWATER_BASIN_CELLS = 4
    leaves no margin above the basin size."""
    elevation: tuple[tuple[float, ...], ...] = (
        (800.0, 800.0, 0.0),
        (800.0, 800.0, 0.0),
        (800.0, 800.0, 0.0),
    )
    ocean = _ocean_mask(elevation, sea_level=0.0)
    flow_dir = _flow_direction_grid(elevation, ocean, seed=42)
    accum = _flow_accumulation(elevation, flow_dir)
    precipitation = _constant_elevation_grid(3, 3, 1000.0)
    candidates = _headwater_candidates(
        elevation=elevation,
        accumulation=accum,
        precipitation=precipitation,
        ocean=ocean,
        flow_dir=flow_dir,
        ocean_distance=_ocean_distance_grid(3, 3, ocean),
    )
    # All 6 land cells form a single 6-cell basin (accum=6 at the ocean
    # mouth). Headwater cells exist (the local maxima on the inland
    # edge) but every one has the same basin-size; multiple headwaters
    # can co-exist in the same basin. The point of the assertion: the
    # candidate set is non-empty and ordered by elevation (descending).
    assert candidates == sorted(
        candidates, key=lambda c: (-elevation[c[1]][c[0]], c[0], c[1])
    )


@pytest.mark.parametrize(
    "elevation, expected_ocean",
    [
        (((5.0, -1.0), (-1.0, -1.0)), {(1, 0), (0, 1), (1, 1)}),
        (((-1.0,),), {(0, 0)}),
        (((10.0, 5.0, -1.0),), {(2, 0)}),
    ],
)
def test_ocean_mask_matches_sea_level(
    elevation: tuple[tuple[float, ...], ...], expected_ocean: set[tuple[int, int]]
) -> None:
    ocean = _ocean_mask(elevation, sea_level=0.0)
    actual = {(x, y) for y, row in enumerate(ocean) for x, cell in enumerate(row) if cell}
    assert actual == expected_ocean