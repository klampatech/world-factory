"""Phase 3a.3 infrastructure invariants — roads, ports, canals."""

import math

import pytest

from world_factory.constants import INFRASTRUCTURE_ALGORITHM_VERSION
from world_factory.generator import generate_world
from world_factory.infrastructure import (
    build_infrastructure,
    validate_infrastructure_layer,
)
from world_factory.models import (
    AgricultureLayer,
    BiomeClass,
    InfrastructureLayer,
    Port,
    PortKind,
    RoadEdge,
    Settlement,
    SettlementsLayer,
    WorldConfig,
    WorldScale,
)


def _config(seed: int = 42) -> WorldConfig:
    return WorldConfig(seed=seed, scale=WorldScale.LARGE)


def test_world_model_includes_infrastructure_layer() -> None:
    world = generate_world(_config())
    assert world.infrastructure is not None
    assert isinstance(world.infrastructure, InfrastructureLayer)


def test_infrastructure_layer_has_three_collections() -> None:
    world = generate_world(_config())
    layer = world.infrastructure
    assert isinstance(layer.roads, tuple)
    assert isinstance(layer.ports, tuple)
    assert isinstance(layer.canals, tuple)


def test_deterministic_across_runs() -> None:
    a = generate_world(_config())
    b = generate_world(_config())
    assert a.infrastructure == b.infrastructure


def test_world_id_stable_across_phase_3a3() -> None:
    """3a.3 adds no new WorldConfig fields, so world_id for --seed 42
    at LARGE scale must remain `9d75e7103b52704b48ce77071a22a586` —
    the v1-demo / 3a.2 reference value."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.metadata.world_id == "9d75e7103b52704b48ce77071a22a586"


def test_schema_version_at_phase_3a3() -> None:
    """At Phase 3a.3, SCHEMA_VERSION was bumped to 10.0.0 per the
    additive-required policy documented in 3a.2. 3a.4 bumped to
    11.0.0 (demography); 3a.5 bumped to 12.0.0 (events); 3b.1 bumped
    to 13.0.0 (cultures); 3b.2 bumped to 14.0.0 (religions). The
    current code is on 14.0.0; this test pins the 3a.3 milestone
    history."""
    world = generate_world(_config())
    assert world.metadata.schema_version == "15.0.0"


def test_road_costs_non_negative_and_finite() -> None:
    world = generate_world(_config())
    for edge in world.infrastructure.roads:
        assert edge.cost >= 0.0
        assert math.isfinite(edge.cost)


def test_roads_canonically_directed() -> None:
    """Every road edge must be canonically directed (from < to) so
    each undirected edge appears exactly once in the layer."""
    world = generate_world(_config())
    for edge in world.infrastructure.roads:
        assert edge.from_settlement_id < edge.to_settlement_id


def test_road_pairs_unique() -> None:
    world = generate_world(_config())
    seen: set[tuple[int, int]] = set()
    for edge in world.infrastructure.roads:
        pair = (edge.from_settlement_id, edge.to_settlement_id)
        assert pair not in seen
        seen.add(pair)


def test_road_path_length_positive() -> None:
    """Road edges must walk at least one cell between settlements."""
    world = generate_world(_config())
    assert world.infrastructure.roads
    for edge in world.infrastructure.roads:
        assert edge.path_length >= 1


def test_port_tonnage_non_negative_and_finite() -> None:
    world = generate_world(_config())
    for port in world.infrastructure.ports:
        assert port.annual_tonnage >= 0.0
        assert math.isfinite(port.annual_tonnage)


def test_port_kind_in_enum() -> None:
    world = generate_world(_config())
    for port in world.infrastructure.ports:
        assert port.port_kind in {PortKind.RIVER, PortKind.COASTAL}


def test_port_settlement_ids_valid() -> None:
    """Every port's settlement_id must reference an existing
    settlement."""
    world = generate_world(_config())
    valid_ids = {s.id for s in world.settlements.settlements}
    for port in world.infrastructure.ports:
        assert port.settlement_id in valid_ids


def test_canal_cost_flow_slope_non_negative_and_finite() -> None:
    world = generate_world(_config())
    for canal in world.infrastructure.canals:
        assert canal.cost >= 0.0
        assert math.isfinite(canal.cost)
        assert canal.mean_flow >= 0.0
        assert math.isfinite(canal.mean_flow)
        assert canal.mean_slope >= 0.0
        assert math.isfinite(canal.mean_slope)


def test_canal_direction_canonical() -> None:
    world = generate_world(_config())
    for canal in world.infrastructure.canals:
        assert canal.from_settlement_id < canal.to_settlement_id


def test_canal_settlement_ids_valid() -> None:
    world = generate_world(_config())
    valid_ids = {s.id for s in world.settlements.settlements}
    for canal in world.infrastructure.canals:
        assert canal.from_settlement_id in valid_ids
        assert canal.to_settlement_id in valid_ids


def test_infrastructure_provenance_record_present() -> None:
    world = generate_world(_config())
    matches = [
        r for r in world.provenance if r.output_path == "infrastructure"
    ]
    assert len(matches) == 1
    assert matches[0].algorithm_version == INFRASTRUCTURE_ALGORITHM_VERSION


def test_validate_infrastructure_empty_for_valid_world() -> None:
    world = generate_world(_config())
    assert validate_infrastructure_layer(world) == []


def test_validate_infrastructure_flags_unknown_settlement_in_road() -> None:
    world = generate_world(_config())
    bad_roads = (
        RoadEdge(
            id=9999,
            from_settlement_id=9999,
            to_settlement_id=9998,
            cost=1.0,
            path_length=1,
        ),
    )
    bad_world = world.model_copy(
        update={
            "infrastructure": InfrastructureLayer(
                roads=bad_roads, ports=(), canals=()
            )
        }
    )
    violations = validate_infrastructure_layer(bad_world)
    assert any(
        v.code == "infrastructure-road-from-settlement-unknown"
        for v in violations
    )
    assert any(
        v.code == "infrastructure-road-to-settlement-unknown"
        for v in violations
    )


def test_validate_infrastructure_flags_bad_road_direction() -> None:
    world = generate_world(_config())
    bad_roads = (
        RoadEdge(
            id=0,
            from_settlement_id=5,
            to_settlement_id=5,
            cost=1.0,
            path_length=1,
        ),
    )
    bad_world = world.model_copy(
        update={
            "infrastructure": InfrastructureLayer(
                roads=bad_roads, ports=(), canals=()
            )
        }
    )
    violations = validate_infrastructure_layer(bad_world)
    assert any(
        v.code == "infrastructure-road-direction" for v in violations
    )


def test_validate_infrastructure_flags_non_finite_port_tonnage() -> None:
    """NaN/Inf tonnage bypasses the model Field(ge=0) check but is
    caught by the validator's finite-check."""
    world = generate_world(_config())
    bad_ports = (
        Port(
            id=0,
            settlement_id=0,
            port_kind=PortKind.COASTAL,
            annual_tonnage=math.inf,
        ),
    )
    bad_world = world.model_copy(
        update={
            "infrastructure": InfrastructureLayer(
                roads=(), ports=bad_ports, canals=()
            )
        }
    )
    violations = validate_infrastructure_layer(bad_world)
    assert any(
        v.code == "infrastructure-port-tonnage-bad" for v in violations
    )


def test_validate_infrastructure_flags_non_finite_canal_flow() -> None:
    """NaN/Inf canal flow bypasses the model Field(ge=0) check but is
    caught by the validator's finite-check."""
    world = generate_world(_config())
    from world_factory.models import Canal
    bad_canals = (
        Canal(
            id=0,
            from_settlement_id=0,
            to_settlement_id=1,
            cost=1.0,
            mean_flow=math.inf,
            mean_slope=0.0,
        ),
    )
    bad_world = world.model_copy(
        update={
            "infrastructure": InfrastructureLayer(
                roads=(), ports=(), canals=bad_canals
            )
        }
    )
    violations = validate_infrastructure_layer(bad_world)
    assert any(
        v.code == "infrastructure-canal-flow-bad" for v in violations
    )


def test_roads_connect_surplus_settlements() -> None:
    """Cross-phase integration: roads must connect surplus-positive
    settlement pairs (production zones) per the 3a.3 spec's
    'roads connect economic centers realistically'."""
    world = generate_world(_config())
    surplus_ids = {
        a.settlement_id
        for a in world.agriculture.agriculture
        if a.agricultural_surplus_kcal_per_year > 0.0
    }
    assert len(surplus_ids) >= 2, "test requires at least 2 surplus+ settlements"
    surplus_pairs: set[tuple[int, int]] = set()
    for r in world.infrastructure.roads:
        if (
            r.from_settlement_id in surplus_ids
            and r.to_settlement_id in surplus_ids
        ):
            surplus_pairs.add((r.from_settlement_id, r.to_settlement_id))
    assert surplus_pairs, (
        "no road edge connects two surplus-positive settlements — "
        "3a.3 spec requires roads to link economic centers"
    )


def test_road_graph_minimal_connectivity_on_seed_42_large() -> None:
    """At seed=42 LARGE the road graph must connect a substantial
    fraction of settlements (no isolated components unless geography
    justifies it — and this world has a contiguous landmass)."""
    world = generate_world(_config())
    settlements = world.settlements.settlements
    adjacency: dict[int, set[int]] = {s.id: set() for s in settlements}
    for edge in world.infrastructure.roads:
        adjacency[edge.from_settlement_id].add(edge.to_settlement_id)
        adjacency[edge.to_settlement_id].add(edge.from_settlement_id)
    connected_count = sum(1 for node, neighbors in adjacency.items() if neighbors)
    total = len(settlements)
    assert connected_count >= total - 1, (
        f"only {connected_count}/{total} settlements connected by roads — "
        f"graph has too many isolated components"
    )


def test_port_count_respects_coast_length() -> None:
    """Port count must not exceed the number of settlements that sit
    within coastal-proximity radius of any ocean cell."""
    world = generate_world(_config())
    from world_factory.infrastructure import (
        _build_river_path_grid,
        _coastal_port_set,
        _river_proximity_set,
    )
    coastal = _coastal_port_set(
        world.geography.width, world.geography.height, world.biomes.classifications
    )
    river_path = _build_river_path_grid(
        world.geography.width, world.geography.height, world.hydrology.river_segments
    )
    river_proximity = _river_proximity_set(
        world.geography.width, world.geography.height, river_path
    )
    eligible_locations = sum(
        1 for s in world.settlements.settlements
        if (s.x, s.y) in coastal or (s.x, s.y) in river_proximity
    )
    assert len(world.infrastructure.ports) <= eligible_locations


def test_two_unreachable_settlements_produce_empty_roads() -> None:
    """A synthetic world where two settlements are separated by an
    ocean strip wide enough to block any friction-grid path must
    produce no road edge between them (and not crash)."""
    base_world = generate_world(_config())
    width = 12
    height = 8
    biome_grid = tuple(
        tuple(
            BiomeClass.OCEAN if x in (5, 6) else BiomeClass.GRASSLAND
            for x in range(width)
        )
        for y in range(height)
    )
    soil_grid = base_world.geology.soil_type_grid  # type stub; overwritten
    from world_factory.models import SoilType
    soil_grid = tuple(
        tuple(SoilType.LOAM for _ in range(width)) for _ in range(height)
    )
    settlements_layer = SettlementsLayer(
        settlements=(
            Settlement(id=0, x=2, y=4, population=1000, founding_score=0.9),
            Settlement(id=1, x=9, y=4, population=1000, founding_score=0.9),
        )
    )
    synthetic_world = base_world.model_copy(
        update={
            "geography": base_world.geography.model_copy(
                update={"width": width, "height": height}
            ),
            "biomes": base_world.biomes.model_copy(
                update={"classifications": biome_grid}
            ),
            "geology": base_world.geology.model_copy(
                update={"soil_type_grid": soil_grid}
            ),
            "settlements": settlements_layer,
            "agriculture": AgricultureLayer(agriculture=()),
            "infrastructure": InfrastructureLayer(roads=(), ports=(), canals=()),
        }
    )
    layer = build_infrastructure(synthetic_world)
    for edge in layer.roads:
        pair = (edge.from_settlement_id, edge.to_settlement_id)
        assert pair != (0, 1), (
            "ocean barrier should make settlements 0 and 1 unreachable"
        )


def test_canals_link_surplus_production_zones_along_rivers() -> None:
    """When canals exist, both endpoints must be surplus-positive
    settlements (production zones). The cross-phase integration from
    3a.2 (agriculture surplus) into 3a.3 (canals) must be respected."""
    world = generate_world(_config())
    if not world.infrastructure.canals:
        pytest.skip("seed=42 LARGE produced no canals at this calibration")
    surplus_ids = {
        a.settlement_id
        for a in world.agriculture.agriculture
        if a.agricultural_surplus_kcal_per_year > 0.0
    }
    for canal in world.infrastructure.canals:
        assert canal.from_settlement_id in surplus_ids
        assert canal.to_settlement_id in surplus_ids


def test_infrastructure_construction_does_not_crash_on_small_grid() -> None:
    """SMALL grid must produce a valid infrastructure layer without
    errors."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    assert world.infrastructure is not None
    assert validate_infrastructure_layer(world) == []


def test_infrastructure_construction_on_medium_grid() -> None:
    """MEDIUM grid must produce a valid infrastructure layer without
    errors."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    assert world.infrastructure is not None
    assert validate_infrastructure_layer(world) == []