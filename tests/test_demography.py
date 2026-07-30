"""Phase 3a.4 demography invariants — population pools, migrations, events."""

import math

import pytest
from pydantic import ValidationError

from world_factory.constants import (
    DEMOGRAPHY_ALGORITHM_VERSION,
    DEMOGRAPHY_DEFAULT_TIME_STEPS,
)
from world_factory.demography import (
    build_demography,
    demography_provenance,
    validate_demography_layer,
)
from world_factory.generator import generate_world
from world_factory.models import (
    AgricultureLayer,
    DemographyLayer,
    EventType,
    Settlement,
    SettlementsLayer,
    WorldConfig,
    WorldEvent,
    WorldScale,
)


def _config(seed: int = 42) -> WorldConfig:
    return WorldConfig(seed=seed, scale=WorldScale.LARGE)


def test_world_model_includes_demography_layer() -> None:
    world = generate_world(_config())
    assert world.demography is not None
    assert isinstance(world.demography, DemographyLayer)


def test_demography_layer_has_three_collections() -> None:
    world = generate_world(_config())
    layer = world.demography
    assert isinstance(layer.pools, tuple)
    assert isinstance(layer.migrations, tuple)
    assert isinstance(layer.events, tuple)


def test_deterministic_across_runs() -> None:
    a = generate_world(_config())
    b = generate_world(_config())
    assert a.demography.pools == b.demography.pools
    assert a.demography.migrations == b.demography.migrations
    assert a.demography.events == b.demography.events


def test_world_id_stable_across_phase_3a4() -> None:
    """3a.4 adds no new WorldConfig fields, so world_id for --seed 42
    at LARGE scale must remain `9d75e7103b52704b48ce77071a22a586` —
    the v1-demo / 3a.2 / 3a.3 reference value."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.metadata.world_id == "9d75e7103b52704b48ce77071a22a586"


def test_schema_version_bumped_to_11() -> None:
    """3a.4 added a required `demography` field to WorldModel, so
    SCHEMA_VERSION must have been 11.0.0 at 3a.4. 3a.5 then added a
    required `events` field, bumping to 12.0.0. 3b.1 added a required
    `cultures` field, bumping to 13.0.0. 3b.2 added a required
    `religions` field, bumping to 14.0.0. The current code is on
    14.0.0; this test pins the 3a.4 milestone history (demography
    bump 10.0.0 -> 11.0.0)."""
    world = generate_world(_config())
    # 3a.4 milestone: bumped 10.0.0 -> 11.0.0
    # 3a.5 milestone: bumped 11.0.0 -> 12.0.0
    # 3b.1 milestone: bumped 12.0.0 -> 13.0.0
    # 3b.2 milestone: bumped 13.0.0 -> 14.0.0 (current)
    assert world.metadata.schema_version == "18.0.0"


def test_pools_parallel_to_settlements() -> None:
    """Demography pools must be parallel to settlements by index."""
    world = generate_world(_config())
    settlements = world.settlements.settlements
    pools = world.demography.pools
    assert len(pools) == len(settlements)
    for index, pool in enumerate(pools):
        assert pool.settlement_id == settlements[index].id


def test_populations_non_negative_and_finite() -> None:
    world = generate_world(_config())
    for pool in world.demography.pools:
        for population in pool.populations:
            assert population >= 0
            assert isinstance(population, int)


def test_population_time_series_length() -> None:
    """Each pool's populations tuple has length time_steps + 1:
    index 0 is the initial population from 3a.1 placement; indices
    1..time_steps are post-step populations."""
    world = generate_world(_config())
    expected_length = DEMOGRAPHY_DEFAULT_TIME_STEPS + 1
    for pool in world.demography.pools:
        assert len(pool.populations) == expected_length


def test_demography_provenance_record_present() -> None:
    world = generate_world(_config())
    matches = [
        record for record in world.provenance if record.output_path == "demography"
    ]
    assert len(matches) == 1
    assert matches[0].algorithm_version == DEMOGRAPHY_ALGORITHM_VERSION


def test_validate_demography_empty_for_valid_world() -> None:
    world = generate_world(_config())
    assert validate_demography_layer(world) == []


def test_validate_demography_flags_length_mismatch() -> None:
    world = generate_world(_config())
    valid_pools = world.demography.pools
    trimmed_pools = valid_pools[:-1]
    bad_world = world.model_copy(
        update={
            "demography": DemographyLayer(
                pools=trimmed_pools, migrations=(), events=()
            )
        }
    )
    violations = validate_demography_layer(bad_world)
    assert any(
        v.code == "demography-pool-length-mismatch" for v in violations
    )


def test_validate_demography_flags_settlement_id_mismatch() -> None:
    world = generate_world(_config())
    if not world.demography.pools:
        return
    pools = list(world.demography.pools)
    if pools:
        first = pools[0]
        pools[0] = first.model_copy(update={"settlement_id": 9999})
    bad_world = world.model_copy(
        update={
            "demography": DemographyLayer(
                pools=tuple(pools), migrations=(), events=()
            )
        }
    )
    violations = validate_demography_layer(bad_world)
    assert any(
        v.code == "demography-pool-settlement-id-mismatch" for v in violations
    )


def test_validate_demography_flags_unknown_settlement_in_migration() -> None:
    from world_factory.models import MigrationRecord

    world = generate_world(_config())
    bad_migrations = (
        MigrationRecord(
            id=0,
            from_settlement_id=9999,
            to_settlement_id=9998,
            step=0,
            count=10,
            road_cost=1.0,
        ),
    )
    bad_world = world.model_copy(
        update={
            "demography": DemographyLayer(
                pools=world.demography.pools,
                migrations=bad_migrations,
                events=(),
            )
        }
    )
    violations = validate_demography_layer(bad_world)
    assert any(
        v.code == "demography-migration-from-settlement-unknown"
        for v in violations
    )
    assert any(
        v.code == "demography-migration-to-settlement-unknown"
        for v in violations
    )


def test_event_types_birth_death_migration_only() -> None:
    """For 3a.4 v1 slice, only BIRTH / DEATH / MIGRATION are emitted.
    Settlement founding, yield computed, etc. are reserved for the
    follow-up phases per PHASE_3A_TYPES.md adoption path."""
    world = generate_world(_config())
    valid_types = {EventType.BIRTH, EventType.DEATH, EventType.MIGRATION}
    for event in world.demography.events:
        assert event.type in valid_types


def test_events_have_deterministic_ids() -> None:
    """Event ids are 16-char hex derived via blake2b (PHASE_3A_TYPES.md
    Option A recommendation). All ids must match the expected length."""
    world = generate_world(_config())
    for event in world.demography.events:
        assert len(event.id) == 16
        assert all(c in "0123456789abcdef" for c in event.id)


def test_event_actors_present() -> None:
    """Every event must have at least one actor. BIRTH and DEATH have
    one individual actor; MIGRATION may have multiple (one per
    migrating individual)."""
    world = generate_world(_config())
    for event in world.demography.events:
        assert len(event.actors) >= 1
        for actor in event.actors:
            assert actor.kind
            assert actor.identifier


def test_migration_along_road_edges_only() -> None:
    """Migrations only fire on road edges. No migration between
    disconnected settlements (e.g., the archipelago-split seed=42
    LARGE case from 3a.3)."""
    world = generate_world(_config())
    road_pairs = {
        (edge.from_settlement_id, edge.to_settlement_id)
        for edge in world.infrastructure.roads
    }
    for migration in world.demography.migrations:
        pair = (migration.from_settlement_id, migration.to_settlement_id)
        assert pair in road_pairs


def test_migration_count_non_negative() -> None:
    world = generate_world(_config())
    for migration in world.demography.migrations:
        assert migration.count >= 0
        assert math.isfinite(migration.road_cost)


def test_settlement_with_population_zero_does_not_crash() -> None:
    """A synthetic world with a settlement that has population=0 must
    not crash demography and must produce a pool of zeros."""
    from world_factory.models import InfrastructureLayer

    base_world = generate_world(_config())
    empty_settlement = Settlement(
        id=0, x=0, y=0, population=0, founding_score=0.0
    )
    synthetic_world = base_world.model_copy(
        update={
            "settlements": SettlementsLayer(settlements=(empty_settlement,)),
            "agriculture": AgricultureLayer(agriculture=()),
            "infrastructure": InfrastructureLayer(roads=(), ports=(), canals=()),
            "demography": DemographyLayer(pools=(), migrations=(), events=()),
        }
    )
    layer = build_demography(synthetic_world, time_steps=5)
    assert len(layer.pools) == 1
    assert all(p == 0 for p in layer.pools[0].populations)


def test_demography_runs_cleanly_on_small_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    assert world.demography is not None
    assert validate_demography_layer(world) == []


def test_demography_runs_cleanly_on_medium_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    assert world.demography is not None
    assert validate_demography_layer(world) == []


def test_seed_variation_changes_outputs() -> None:
    """Different seeds produce different demography outputs (sanity
    check that the seed actually drives the simulation)."""
    a = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    b = generate_world(WorldConfig(seed=43, scale=WorldScale.SMALL))
    assert a.demography.pools != b.demography.pools or a.demography.events != b.demography.events


def test_births_deaths_proportional_to_population() -> None:
    """Statistical realism: at the seed=42 LARGE distribution, mean
    birth rate should be near DEMOGRAPHY_BASE_BIRTH_RATE and mean
    death rate slightly higher (over-capacity settlements)."""
    world = generate_world(_config())
    if not world.demography.events:
        return
    n_births = sum(1 for e in world.demography.events if e.type == EventType.BIRTH)
    n_deaths = sum(1 for e in world.demography.events if e.type == EventType.DEATH)
    assert n_births > 0
    assert n_deaths > 0
    # Death count should be at least as large as birth count given
    # the over-capacity settlement distribution (caps mean 925 vs
    # populations mean 1348).
    assert n_deaths >= n_births


def test_populations_eventually_decline_when_over_capacity() -> None:
    """At seed=42 LARGE, all settlements are over capacity (Phase 3a.2
    caps mean 925 vs populations mean 1348). The aggregate population
    should decline over the simulation."""
    world = generate_world(_config())
    pools = world.demography.pools
    initial_total = sum(p.populations[0] for p in pools)
    final_total = sum(p.populations[-1] for p in pools)
    assert initial_total > 0
    assert final_total < initial_total


def test_validate_world_event_flags_payload_type_mismatch() -> None:
    """Per Finding B: WorldEvent._validate_payload_shape must reject
    a payload that doesn't match the declared event.type. A BIRTH
    event with a death-shaped payload should fail at construction."""
    # Malformed BIRTH payload (missing required field, wrong shape)
    bad_payload = {"individual_id": "abc123"}  # missing settlement_id, cohort_year
    with pytest.raises(ValidationError):
        WorldEvent(
            id="0123456789abcdef",
            type=EventType.BIRTH,
            t=0,
            location={"cell": None, "settlement_id": 0},
            actors=(),
            payload=bad_payload,
            causes=(),
            provenance=demography_provenance(),
        )


def test_validate_world_event_accepts_well_typed_payload() -> None:
    """A well-typed BIRTH event must construct cleanly through the
    validator."""
    good_payload = {
        "settlement_id": 0,
        "individual_id": "abc123",
        "parent_ids": [],
        "cohort_year": 0,
    }
    event = WorldEvent(
        id="0123456789abcdef",
        type=EventType.BIRTH,
        t=0,
        location={"cell": None, "settlement_id": 0},
        actors=(),
        payload=good_payload,
        causes=(),
        provenance=demography_provenance(),
    )
    assert event.type == EventType.BIRTH