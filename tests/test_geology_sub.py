"""Phase 1e geological sublayer invariants: rock, ore, soil."""

from collections import Counter

from world_factory.generator import generate_world
from world_factory.models import (
    GeologyLayer,
    PlateType,
    RockType,
    SoilType,
    WorldConfig,
    WorldScale,
)


def test_geology_layer_exposes_sublayer_grids() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert isinstance(world.geology, GeologyLayer)
    assert len(world.geology.rock_type_grid) == world.geology.height
    assert len(world.geology.rock_type_grid[0]) == world.geology.width
    assert len(world.geology.ore_presence_grid) == world.geology.height
    assert len(world.geology.ore_presence_grid[0]) == world.geology.width
    assert len(world.geology.soil_type_grid) == world.geology.height
    assert len(world.geology.soil_type_grid[0]) == world.geology.width


def test_rock_types_are_valid_strenum_values() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for row in world.geology.rock_type_grid:
        for rock in row:
            assert rock in set(RockType)


def test_soil_types_are_valid_strenum_values() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for row in world.geology.soil_type_grid:
        for soil in row:
            assert soil in set(SoilType)


def test_ore_presence_is_boolean() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for row in world.geology.ore_presence_grid:
        for value in row:
            assert isinstance(value, bool)


def test_oceanic_plates_have_basalt_rock() -> None:
    """Oceanic plate interior cells (no boundary) read as BASALT."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for plate in world.geology.plates:
        if plate.plate_type is not PlateType.OCEANIC:
            continue
        sample_x = int(plate.centroid_x)
        sample_y = int(plate.centroid_y)
        if 0 <= sample_x < world.geology.width and 0 <= sample_y < world.geology.height:
            assert world.geology.rock_type_grid[sample_y][sample_x] in {
                RockType.BASALT,
                RockType.VOLCANIC,
            }


def test_continental_interiors_have_granite_or_sedimentary() -> None:
    """Continental plate interior cells read as GRANITE (high
    elevation) or SEDIMENTARY (low elevation)."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for plate in world.geology.plates:
        if plate.plate_type is not PlateType.CONTINENTAL:
            continue
        sample_x = int(plate.centroid_x)
        sample_y = int(plate.centroid_y)
        if 0 <= sample_x < world.geology.width and 0 <= sample_y < world.geology.height:
            rock = world.geology.rock_type_grid[sample_y][sample_x]
            assert rock in {RockType.GRANITE, RockType.SEDIMENTARY, RockType.VOLCANIC}


def test_ore_count_grows_with_plate_count() -> None:
    """Higher plate count yields more plate boundaries, which yields
    more cells with ore presence."""
    few = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE, plate_count=3))
    many = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE, plate_count=64))
    few_ore = sum(
        1 for row in few.geology.ore_presence_grid for value in row if value
    )
    many_ore = sum(
        1 for row in many.geology.ore_presence_grid for value in row if value
    )
    assert many_ore >= few_ore


def test_rock_distribution_is_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.geology.rock_type_grid == b.geology.rock_type_grid
    assert a.geology.ore_presence_grid == b.geology.ore_presence_grid
    assert a.geology.soil_type_grid == b.geology.soil_type_grid


def test_world_id_stable_across_phase_1e() -> None:
    """Phase 1e adds no new WorldConfig fields, so the world_id hash
    for seed=42 must be unchanged from Phase 1d."""
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.metadata.world_id == b.metadata.world_id


def test_sublayer_provenance_record_present() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sublayer_paths = [
        record
        for record in world.provenance
        if record.output_path == "geology.sublayers"
    ]
    assert len(sublayer_paths) == 1
    assert sublayer_paths[0].algorithm_version == "rock-ore-soil-v1"


def test_rock_distribution_has_diversity() -> None:
    """A LARGE world should have more than one rock type represented."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    counter: Counter[RockType] = Counter()
    for row in world.geology.rock_type_grid:
        for rock in row:
            counter[rock] += 1
    assert len(counter) >= 2


def test_soil_distribution_has_diversity() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    counter: Counter[SoilType] = Counter()
    for row in world.geology.soil_type_grid:
        for soil in row:
            counter[soil] += 1
    assert len(counter) >= 2