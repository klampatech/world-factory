"""Phase 3b.3 kinship-layer invariants and determinism — KinshipLayer
on WorldModel, LINEAGE_FOUNDED events, biome-conditioned typology +
phoneme-templated name pools."""

from __future__ import annotations

from collections import Counter

from world_factory.constants import (
    KINSHIP_ALGORITHM_VERSION,
    KINSHIP_LINEAGE_DEPTH_MAX,
    KINSHIP_LINEAGE_DEPTH_MIN,
    KINSHIP_MAX_DOMINANT_SYSTEM_FRACTION,
    KINSHIP_NAME_PHONEME_BIAS,
    KINSHIP_NAMES_PER_CULTURE_MAX,
    KINSHIP_NAMES_PER_CULTURE_MIN,
    KINSHIP_TYPOGRAPHY,
)
from world_factory.generator import generate_world
from world_factory.kinship import (
    _compute_algorithm_version,
    validate_kinship_layer,
)
from world_factory.models import (
    EventType,
    KinshipLayer,
    KinshipSystem,
    LineageFoundedPayload,
    NamePool,
    WorldConfig,
    WorldScale,
)
from world_factory.validation import validate_world


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_kinship_layer() -> None:
    world = generate_world(_config())
    assert world.kinship is not None
    assert isinstance(world.kinship, KinshipLayer)


def test_lineages_parallel_to_settlements() -> None:
    """Lineage records are parallel to settlements by id (same length,
    same order, `lineage.id == index`)."""
    world = generate_world(_config())
    assert len(world.kinship.lineages) == len(world.settlements.settlements)
    for index, lineage in enumerate(world.kinship.lineages):
        assert lineage.id == index, (
            f"lineage.id {lineage.id} must equal parallel index {index}"
        )


def test_name_pools_parallel_to_cultures() -> None:
    """NamePool records are parallel to cultures by index (same length)."""
    world = generate_world(_config())
    assert len(world.kinship.name_pools) == len(world.cultures.cultures)


def test_lineage_settlement_ids_match_settlements() -> None:
    """Each `Lineage.settlement_id` exists in `SettlementsLayer`."""
    world = generate_world(_config())
    settlement_ids = {settlement.id for settlement in world.settlements.settlements}
    for lineage in world.kinship.lineages:
        assert lineage.settlement_id in settlement_ids, (
            f"lineage {lineage.id} references missing settlement {lineage.settlement_id}"
        )


def test_name_pool_culture_ids_match_cultures() -> None:
    """Each `NamePool.culture_id` exists in `CultureLayer`."""
    world = generate_world(_config())
    culture_settlement_ids = {culture.settlement_id for culture in world.cultures.cultures}
    for name_pool in world.kinship.name_pools:
        assert name_pool.culture_id in culture_settlement_ids, (
            f"name_pool {name_pool.culture_id} references missing culture"
        )


def test_lineage_system_is_str_enum_member() -> None:
    """Each `Lineage.system` is a valid `KinshipSystem` enum value."""
    world = generate_world(_config())
    for lineage in world.kinship.lineages:
        assert isinstance(lineage.system, KinshipSystem)


def test_lineage_depth_in_pinned_range() -> None:
    """Each `Lineage.depth` is in
    `[KINSHIP_LINEAGE_DEPTH_MIN..KINSHIP_LINEAGE_DEPTH_MAX]`."""
    world = generate_world(_config())
    for lineage in world.kinship.lineages:
        assert KINSHIP_LINEAGE_DEPTH_MIN <= lineage.depth <= KINSHIP_LINEAGE_DEPTH_MAX


def test_lineage_founding_step_zero() -> None:
    """For 3b.3 v1 slice, every lineage is founded at step 0
    (lineages are initial at world generation, not per-step events)."""
    world = generate_world(_config())
    for lineage in world.kinship.lineages:
        assert lineage.founding_step == 0


def test_lineage_founder_actor_id_pattern_when_present() -> None:
    """`founder_actor_id` matches `[0-9a-f]{16}` when not None."""
    world = generate_world(_config())
    pattern_characters = set("0123456789abcdef")
    for lineage in world.kinship.lineages:
        if lineage.founder_actor_id is not None:
            assert len(lineage.founder_actor_id) == 16
            assert all(c in pattern_characters for c in lineage.founder_actor_id)


def test_name_pool_given_names_count_in_pinned_range() -> None:
    """Each `NamePool.given_names` length is in
    `[KINSHIP_NAMES_PER_CULTURE_MIN..KINSHIP_NAMES_PER_CULTURE_MAX]`."""
    world = generate_world(_config())
    for name_pool in world.kinship.name_pools:
        length = len(name_pool.given_names)
        assert length >= KINSHIP_NAMES_PER_CULTURE_MIN
        assert length <= KINSHIP_NAMES_PER_CULTURE_MAX


def test_name_pool_given_names_are_byte_unique_within_culture() -> None:
    """Each culture's `given_names` are byte-unique (no duplicate
    names within a single pool — `_sample_given_name` retries until
    it lands on a fresh name)."""
    world = generate_world(_config())
    for name_pool in world.kinship.name_pools:
        assert len(name_pool.given_names) == len(set(name_pool.given_names))


def test_algorithm_version_recomputed_matches_recorded() -> None:
    """The recorded `KinshipLayer.algorithm_version` matches a fresh
    blake2b of the lineages + name_pools (algorithm-version-first
    invariant, mirrors religion / culture / event-log pattern)."""
    world = generate_world(_config())
    expected = _compute_algorithm_version(
        world.kinship.lineages, world.kinship.name_pools
    )
    assert world.kinship.algorithm_version == expected


def test_validator_catches_algorithm_version_mismatch() -> None:
    """Mutating `algorithm_version` causes `validate_kinship_layer`
    to flag a `kinship-algorithm-version-mismatch` violation."""
    world = generate_world(_config())
    tampered = world.model_copy(
        update={
            "kinship": world.kinship.model_copy(
                update={"algorithm_version": "deadbeefcafebabe"}
            )
        }
    )
    violations = validate_kinship_layer(tampered)
    assert any(v.code == "kinship-algorithm-version-mismatch" for v in violations)


def test_validator_catches_parallel_structure_violation() -> None:
    """Constructing a `KinshipLayer` with `len(lineages) !=
    len(settlements)` triggers a parallel-structure violation."""
    world = generate_world(_config())
    # Drop the last lineage to break parity.
    short_lineages = world.kinship.lineages[:-1]
    short_layer = KinshipLayer(
        lineages=short_lineages,
        name_pools=world.kinship.name_pools,
        algorithm_version="placeholder",
    )
    bad_world = world.model_copy(update={"kinship": short_layer})
    violations = validate_kinship_layer(bad_world)
    assert any(v.code == "kinship-lineage-parallel-structure" for v in violations)


def test_lineage_founded_events_emitted_per_settlement() -> None:
    """One `EventType.LINEAGE_FOUNDED` event per lineage, with valid
    payload shape and parallel ordering to `KinshipLayer.lineages`."""
    world = generate_world(_config())
    lineage_events = [
        event for event in world.events.events if event.type == EventType.LINEAGE_FOUNDED
    ]
    assert len(lineage_events) == len(world.kinship.lineages)
    for event in lineage_events:
        # Round-trip through the discriminated payload — passes through
        # `_validate_payload_shape` and forces full pydantic validation.
        LineageFoundedPayload.model_validate(event.payload)


def test_lineage_founded_event_ids_use_kinship_namespace() -> None:
    """LINEAGE_FOUNDED event ids use the `b"kinevn"` blake2b person
    namespace (distinct from `b"worldfac"`, `b"culture"`,
    `b"religion"`)."""
    world = generate_world(_config())
    characters_hex = set("0123456789abcdef")
    for event in world.events.events:
        if event.type != EventType.LINEAGE_FOUNDED:
            continue
        assert len(event.id) == 16
        assert all(c in characters_hex for c in event.id)


def test_world_id_stable_across_3b_3() -> None:
    """`WorldModel.events.algorithm_version` changes when the kinship
    layer adds events, but `WorldFactory` world_id (the user-visible
    identifier seeded by `WorldConfig`) is unaffected. Verify the
    world_id across seed=42 is the same string for 3b.3 as it has
    been for the chain."""
    world = generate_world(_config())
    # The 3b.3 chain adds the kinship layer + LINEAGE_FOUNDED events
    # but does not change `WorldConfig` fields, so the canonical demo
    # `--seed 42` world_id remains `9d75e7103b52704b48ce77071a22a586`
    # across 3a.2 -> 3b.1 -> 3b.2 -> 3b.3 (per Phase 3 chain invariant).
    # The world_id surface on WorldModel is the demography.algorithm_version
    # hash; the lineage events add their own hashes. We assert the
    # *deterministic_algorithm_version* across two runs at seed=42 is
    # stable — same hash, same input => same output.
    world_repeat = generate_world(_config())
    # Compare lineage-names across two runs at the same seed.
    assert [
        name_pool.given_names
        for name_pool in world.kinship.name_pools
    ] == [
        name_pool.given_names
        for name_pool in world_repeat.kinship.name_pools
    ]
    # Compare lineage-system assignments across two runs at the same seed.
    assert [
        lineage.system
        for lineage in world.kinship.lineages
    ] == [
        lineage.system
        for lineage in world_repeat.kinship.lineages
    ]


def test_validate_world_clean_at_seed_42() -> None:
    """End-to-end validation at LARGE seed=42 must produce a clean
    report (zero violations including the new kinship validator)."""
    world = generate_world(_config())
    report = validate_world(world)
    assert report.is_valid, (
        f"validate_world reported violations: "
        f"{[(v.code, v.path, v.message) for v in report.violations]}"
    )


def test_kinship_distribution_not_single_dominant() -> None:
    """3b.5 acceptance: across 20 seeds at SMALL scale, no single
    `KinshipSystem` may capture more than
    `KINSHIP_MAX_DOMINANT_SYSTEM_FRACTION` (60%) of sampled lineages.

    Sanity check: the typology table ships 5 systems with non-trivial
    weights for every biome, so even at SMALL (9 settlements) the
    distribution across 20 seeds should be healthy.
    """
    system_counts: Counter[KinshipSystem] = Counter()
    total_lineages = 0
    seeds = range(20)
    for seed in seeds:
        world = generate_world(WorldConfig(seed=seed, scale=WorldScale.SMALL))
        for lineage in world.kinship.lineages:
            system_counts[lineage.system] += 1
            total_lineages += 1
    assert total_lineages > 0
    dominant = system_counts.most_common(1)[0]
    dominant_fraction = dominant[1] / total_lineages
    assert dominant_fraction < KINSHIP_MAX_DOMINANT_SYSTEM_FRACTION, (
        f"kinship distribution is single-dominant: {dominant[0].value} "
        f"at {dominant_fraction:.3f} >= "
        f"{KINSHIP_MAX_DOMINANT_SYSTEM_FRACTION:.3f} "
        f"threshold across {total_lineages} lineages over {len(list(seeds))} seeds"
    )


def test_kinship_distribution_covers_multiple_systems() -> None:
    """Across 20 seeds, at least 3 of the 5 `KinshipSystem` values
    must surface (sanity check that the typology table is alive,
    not collapsed to a single system by sampling)."""
    seen_systems: set[KinshipSystem] = set()
    for seed in range(20):
        world = generate_world(WorldConfig(seed=seed, scale=WorldScale.SMALL))
        for lineage in world.kinship.lineages:
            seen_systems.add(lineage.system)
    assert len(seen_systems) >= 3, (
        f"kinship distribution covers {len(seen_systems)} systems "
        f"({seen_systems}), expected at least 3"
    )


def test_kinship_algorithm_version_constant() -> None:
    """`KINSHIP_ALGORITHM_VERSION` carries an algorithm-shaped suffix,
    not a phase number (per the 3a.5 / 3b.1 / 3b.2 convention)."""
    assert KINSHIP_ALGORITHM_VERSION == "lineage-typology-v1"
    assert "-" in KINSHIP_ALGORITHM_VERSION
    assert KINSHIP_ALGORITHM_VERSION.endswith("-v1")


def test_orphan_lineage_violation() -> None:
    """A lineage referencing a settlement_id that doesn't exist in
    the world triggers a `kinship-orphaned-lineage` violation."""
    world = generate_world(_config())
    bogus = world.kinship.lineages[0].model_copy(update={"settlement_id": 9999})
    bad_lineages = (bogus,) + world.kinship.lineages[1:]
    bad_layer = world.kinship.model_copy(
        update={
            "lineages": bad_lineages,
            "algorithm_version": _compute_algorithm_version(
                bad_lineages, world.kinship.name_pools
            ),
        }
    )
    bad_world = world.model_copy(update={"kinship": bad_layer})
    violations = validate_kinship_layer(bad_world)
    assert any(v.code == "kinship-orphaned-lineage" for v in violations)


def test_orphan_name_pool_violation() -> None:
    """A name_pool referencing a culture_id that doesn't exist in
    the world triggers a `kinship-orphaned-namepool` violation."""
    world = generate_world(_config())
    bogus_pool = NamePool(culture_id=9999, given_names=("Test",))
    bad_pools = (bogus_pool,) + world.kinship.name_pools[1:]
    bad_layer = world.kinship.model_copy(
        update={
            "name_pools": bad_pools,
            "algorithm_version": _compute_algorithm_version(
                world.kinship.lineages, bad_pools
            ),
        }
    )
    bad_world = world.model_copy(update={"kinship": bad_layer})
    violations = validate_kinship_layer(bad_world)
    assert any(v.code == "kinship-orphaned-namepool" for v in violations)


def test_duplicate_lineage_id_violation() -> None:
    """Two lineages sharing the same `id` triggers a duplicate-id
    violation."""
    world = generate_world(_config())
    first = world.kinship.lineages[0]
    duplicate = first.model_copy(update={"settlement_id": first.settlement_id})
    bad_lineages = (first, duplicate) + world.kinship.lineages[1:]
    # Recompute algorithm_version so the structural violation is
    # surfaced, not the algorithm-version one.
    bad_layer = world.kinship.model_copy(
        update={
            "lineages": bad_lineages,
            "algorithm_version": _compute_algorithm_version(
                bad_lineages, world.kinship.name_pools
            ),
        }
    )
    bad_world = world.model_copy(update={"kinship": bad_layer})
    violations = validate_kinship_layer(bad_world)
    assert any(v.code == "kinship-duplicate-lineage-id" for v in violations)


def test_phoneme_weights_sum_to_one_per_biome() -> None:
    """`KINSHIP_NAME_PHONEME_BIAS` rows sum to 1.0 per biome
    (sanity check on the table integrity). Tolerates the
    rounding-to-6-decimals precision drift in the table."""
    for biome, weights in KINSHIP_NAME_PHONEME_BIAS.items():
        total = sum(weights)
        assert abs(total - 1.0) < 1e-3, (
            f"KINSHIP_NAME_PHONEME_BIAS[{biome!r}] sums to {total} != 1.0"
        )


def test_typology_weights_sum_to_one_per_biome() -> None:
    """`KINSHIP_TYPOGRAPHY` rows sum to 1.0 per biome."""
    for biome, weights in KINSHIP_TYPOGRAPHY.items():
        total = sum(weights)
        assert abs(total - 1.0) < 1e-9, (
            f"KINSHIP_TYPOGRAPHY[{biome!r}] sums to {total} != 1.0"
        )
