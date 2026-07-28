"""Phase 6 query-surface tests."""

from world_factory.generator import generate_world
from world_factory.models import WorldConfig, WorldScale
from world_factory.queries import (
    settlements_within,
    summary_at,
    summary_in_bounding_box,
    validate_query_surface,
)


def test_summary_at_returns_cell_summary() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    summary = summary_at(world, 0, 0)
    assert summary.x == 0
    assert summary.y == 0
    assert summary.elevation_meters == world.geography.elevation_meters[0][0]


def test_summary_at_ocean_detection() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    for y in range(world.geography.height):
        for x in range(world.geography.width):
            if world.geography.elevation_meters[y][x] <= sea_level:
                summary = summary_at(world, x, y)
                assert summary.is_ocean is True
                return


def test_summary_at_land_detection() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    sea_level = world.geography.sea_level_meters
    for y in range(world.geography.height):
        for x in range(world.geography.width):
            if world.geography.elevation_meters[y][x] > sea_level:
                summary = summary_at(world, x, y)
                assert summary.is_ocean is False
                return


def test_settlements_within_radius() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    if not world.settlements.settlements:
        return
    origin = world.settlements.settlements[0]
    near = settlements_within(world, 0, origin.x, origin.y)
    assert origin in near
    far = settlements_within(world, 10000, origin.x, origin.y)
    assert far == tuple(world.settlements.settlements)


def test_summary_in_bounding_box_returns_correct_count() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    cells = summary_in_bounding_box(world, 0, 0, 4, 4)
    assert len(cells) == 25  # 5x5


def test_summary_in_bounding_box_clamps_to_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    width = world.geography.width
    height = world.geography.height
    cells = summary_in_bounding_box(world, 0, 0, width * 2, height * 2)
    assert len(cells) == width * height


def test_summary_at_returns_settlement_on_its_cell() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    for settlement in world.settlements.settlements:
        summary = summary_at(world, settlement.x, settlement.y)
        assert settlement in summary.settlements


def test_validate_query_surface_returns_empty_for_valid_world() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert validate_query_surface(world) == []


def test_deterministic_summary_across_runs() -> None:
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    if a.settlements.settlements and b.settlements.settlements:
        s_a = summary_at(a, a.settlements.settlements[0].x, a.settlements.settlements[0].y)
        s_b = summary_at(b, b.settlements.settlements[0].x, b.settlements.settlements[0].y)
        assert s_a.model_dump() == s_b.model_dump()


def test_world_id_stable_across_phase_6() -> None:
    """Phase 6 is a no-schema-bump addition; world_id is stable."""
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert a.metadata.world_id == b.metadata.world_id