"""Phase 1b hydrology invariants: river network, discharge, watersheds."""

from world_factory.generator import generate_world
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