"""Phase 2 biology invariants: flora and fauna per biome."""

from collections import Counter

from world_factory.biology import build_biology
from world_factory.generator import generate_world
from world_factory.models import (
    BiomeClass,
    FaunaType,
    FloraType,
    WorldConfig,
    WorldScale,
)


def test_world_model_includes_biology_layer() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.biology is not None
    assert len(world.biology.flora_grid) == world.geography.height
    assert len(world.biology.flora_grid[0]) == world.geography.width
    assert len(world.biology.fauna_grid) == world.geography.height
    assert len(world.biology.fauna_grid[0]) == world.geography.width


def test_flora_types_are_valid_strenum_values() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    valid = set(FloraType)
    for row in world.biology.flora_grid:
        for value in row:
            if value is not None:
                assert value in valid


def test_fauna_types_are_valid_strenum_values() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    valid = set(FaunaType)
    for row in world.biology.fauna_grid:
        for value in row:
            if value is not None:
                assert value in valid


def test_biology_provenance_record_present() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    biology_records = [
        record for record in world.provenance if record.output_path == "biology"
    ]
    assert len(biology_records) == 1
    assert biology_records[0].algorithm_version == "biome-biota-v1"


def test_ocean_cells_carry_marine_biota() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    found_marine = False
    for y in range(world.geography.height):
        for x in range(world.geography.width):
            if world.geography.elevation_meters[y][x] > sea_level:
                continue
            flora = world.biology.flora_grid[y][x]
            fauna = world.biology.fauna_grid[y][x]
            if flora == FloraType.ALGAE and fauna == FaunaType.FISH:
                found_marine = True
                break
        if found_marine:
            break
    assert found_marine


def test_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.biology.flora_grid == b.biology.flora_grid
    assert a.biology.fauna_grid == b.biology.fauna_grid


def test_world_id_stable_across_phase_2() -> None:
    """Phase 2 adds no new WorldConfig fields, so the world_id hash
    for seed=42 must be unchanged from Phase 1f."""
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.metadata.world_id == b.metadata.world_id


def test_flora_distribution_has_diversity() -> None:
    """A LARGE world should have more than one flora type represented."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    counter: Counter[FloraType] = Counter()
    for row in world.biology.flora_grid:
        for value in row:
            if value is not None:
                counter[value] += 1
    assert len(counter) >= 2


def test_fauna_distribution_has_diversity() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    counter: Counter[FaunaType] = Counter()
    for row in world.biology.fauna_grid:
        for value in row:
            if value is not None:
                counter[value] += 1
    assert len(counter) >= 2


def test_build_biology_assigns_algae_to_ocean() -> None:
    """Direct test of build_biology with a synthetic biome grid:
    every ocean cell should carry ALGAE flora + FISH fauna."""
    classifications = (
        (BiomeClass.OCEAN, BiomeClass.GRASSLAND, BiomeClass.OCEAN),
        (BiomeClass.OCEAN, BiomeClass.GRASSLAND, BiomeClass.OCEAN),
    )
    elevation = (
        (-50.0, 100.0, -50.0),
        (-50.0, 100.0, -50.0),
    )
    biology = build_biology(classifications, elevation, sea_level=0.0)
    assert biology.flora_grid[0][0] == FloraType.ALGAE
    assert biology.fauna_grid[0][0] == FaunaType.FISH
    assert biology.flora_grid[0][1] == FloraType.GRASS
    assert biology.fauna_grid[0][1] == FaunaType.HERBIVORE_LARGE


def test_validate_biology_layer_returns_empty_for_valid_world() -> None:
    from world_factory.biology import validate_biology_layer

    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert validate_biology_layer(world) == []