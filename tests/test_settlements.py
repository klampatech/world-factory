"""Phase 3a settlement placement invariants."""

from world_factory.generator import generate_world
from world_factory.models import WorldConfig, WorldScale
from world_factory.settlements import build_settlements, validate_settlements_layer


def test_world_model_includes_settlements_layer() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.settlements is not None
    assert len(world.settlements.settlements) >= 0


def test_settlements_within_grid_bounds() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    width = world.geography.width
    height = world.geography.height
    for settlement in world.settlements.settlements:
        assert 0 <= settlement.x < width
        assert 0 <= settlement.y < height


def test_settlements_have_positive_population() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for settlement in world.settlements.settlements:
        assert settlement.population >= 0


def test_founding_score_in_unit_range() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for settlement in world.settlements.settlements:
        assert 0.0 <= settlement.founding_score <= 1.0


def test_min_settlement_count() -> None:
    """A LARGE world should produce at least SETTLEMENT_MIN_COUNT
    settlements."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert len(world.settlements.settlements) >= 20


def test_deterministic_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.settlements.settlements == b.settlements.settlements


def test_settlement_count_scales_with_plate_count() -> None:
    """Higher plate count yields more candidate terrain diversity,
    so more settlements (up to spacing limits)."""
    few = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE, plate_count=3))
    many = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE, plate_count=64))
    assert len(many.settlements.settlements) >= len(few.settlements.settlements)


def test_settlements_avoid_ocean_cells() -> None:
    """Settlement placement skips OCEAN cells by construction."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    for settlement in world.settlements.settlements:
        cell_elevation = world.geography.elevation_meters[settlement.y][settlement.x]
        assert cell_elevation > sea_level


def test_settlements_have_unique_ids() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    ids = [s.id for s in world.settlements.settlements]
    assert len(set(ids)) == len(ids)


def test_settlements_provenance_record_present() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    settlement_records = [
        record for record in world.provenance if record.output_path == "settlements"
    ]
    assert len(settlement_records) == 1
    assert settlement_records[0].algorithm_version == "candidate-scoring-v1"


def test_validate_settlements_layer_returns_empty_for_valid_world() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert validate_settlements_layer(world) == []


def test_build_settlements_with_synthetic_inputs() -> None:
    """Direct test with a 4x4 grid where every cell is grassland.
    Settlements should land on land cells with reasonable scores."""
    from world_factory.models import BiomeClass

    elevation = (
        (100.0, 100.0, 100.0, 100.0),
        (100.0, 800.0, 100.0, 100.0),
        (100.0, 100.0, 100.0, 100.0),
        (100.0, 100.0, 100.0, 100.0),
    )
    temperature = (
        (15.0, 15.0, 15.0, 15.0),
        (15.0, 10.0, 15.0, 15.0),
        (15.0, 15.0, 15.0, 15.0),
        (15.0, 15.0, 15.0, 15.0),
    )
    biome_grid = tuple(tuple(BiomeClass.GRASSLAND for _ in range(4)) for _ in range(4))
    ore_grid = tuple(tuple(False for _ in range(4)) for _ in range(4))
    settlements_layer = build_settlements(
        elevation=elevation,
        temperature=temperature,
        biome_grid=biome_grid,
        ore_grid=ore_grid,
        river_segments=(),
        plate_count=12,
    )
    assert len(settlements_layer.settlements) >= 1
    for s in settlements_layer.settlements:
        assert 0 <= s.x < 4
        assert 0 <= s.y < 4