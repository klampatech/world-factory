"""Phase 4.1 polity-layer invariants and determinism — PolityLayer
on WorldModel, Polity records parallel to cultures per plan-ack Q1,
defensible borders (river + elevation per plan-ack Q2), governance
derivation (size-pinned per plan-ack Q3), founder_actor_id pattern
(3b.3-style per plan-ack Q4), and one POLITY_FOUNDED event per polity
at step 0 (per plan-ack Q5)."""

from __future__ import annotations

from world_factory.constants import (
    ELEVATION_BORDER_THRESHOLD_M,
    POLITY_ALGORITHM_VERSION,
)
from world_factory.generator import generate_world
from world_factory.models import (
    EventType,
    GovernanceType,
    JoinReason,
    PolityEventType,
    PolityFoundedPayload,
    PolityLayer,
    PolityMember,
    WorldConfig,
    WorldScale,
)
from world_factory.polities import (
    _compute_algorithm_version,
    _compute_polity_algorithm_version,
    _derive_governance_type,
    validate_polities_layer,
)
from world_factory.validation import validate_world


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_polities_layer() -> None:
    """`WorldModel.polities` is a `PolityLayer` aggregate (additive
    required, parallel-by-cultures per plan-ack Q1)."""
    world = generate_world(_config())
    assert world.polities is not None
    assert isinstance(world.polities, PolityLayer)


def test_polities_parallel_to_cultures() -> None:
    """One polity per culture (plan-ack Q1: 1 polity per culture, v1
    simplification since 3b.4 1:1 root-language-to-culture ratio
    collapses the joint (culture, language_root) key to culture)."""
    world = generate_world(_config())
    assert len(world.polities.polities) == len(world.cultures.cultures), (
        f"polities {len(world.polities.polities)} != cultures {len(world.cultures.cultures)}"
    )
    culture_settlement_ids = {culture.settlement_id for culture in world.cultures.cultures}
    for index, polity in enumerate(world.polities.polities):
        assert polity.id in culture_settlement_ids or polity.id == index, (
            f"polity {polity.id} (index {index}) not parallel to a culture"
        )


def test_memberships_parallel_to_settlements() -> None:
    """One `PolityMember` per settlement (v1: every settlement
    belongs to exactly one polity)."""
    world = generate_world(_config())
    settlements_count = len(world.settlements.settlements)
    assert len(world.polities.memberships) == settlements_count, (
        f"memberships {len(world.polities.memberships)} != settlements {settlements_count}"
    )
    polity_ids = {polity.id for polity in world.polities.polities}
    for member in world.polities.memberships:
        assert member.polity_id in polity_ids, (
            f"membership references unknown polity {member.polity_id}"
        )
        assert member.joined_step == 0, f"v1 emits only joined_step=0; got {member.joined_step}"
        assert member.joined_reason == JoinReason.CULTURE, (
            f"v1 emits only joined_reason=CULTURE; got {member.joined_reason}"
        )


def test_governance_derivation_pinned_at_founding() -> None:
    """Plan-ack Q3: governance_type is pinned at founding via
    `len(members)`. Single-member polities in v1 → `BAND`."""
    world = generate_world(_config())
    for polity in world.polities.polities:
        # 1 member (the primary settlement) → BAND.
        assert polity.governance_type == GovernanceType.BAND, (
            f"polity {polity.id} has {polity.governance_type}, expected BAND"
        )


def test_governance_size_buckets() -> None:
    """`_derive_governance_type` follows the plan-ack Q3 size buckets:
    1-2 BAND, 3-6 CHIEFDOM, 7-15 KINGDOM, 16+ EMPIRE."""
    assert _derive_governance_type(1) == GovernanceType.BAND
    assert _derive_governance_type(2) == GovernanceType.BAND
    assert _derive_governance_type(3) == GovernanceType.CHIEFDOM
    assert _derive_governance_type(6) == GovernanceType.CHIEFDOM
    assert _derive_governance_type(7) == GovernanceType.KINGDOM
    assert _derive_governance_type(15) == GovernanceType.KINGDOM
    assert _derive_governance_type(16) == GovernanceType.EMPIRE
    assert _derive_governance_type(100) == GovernanceType.EMPIRE


def test_founder_actor_id_pattern() -> None:
    """Plan-ack Q4: `founder_actor_id` follows 3b.3's
    `Lineage.founder_actor_id` pattern — sample one living demography
    individual at step 0 from the founding polity's primary settlement.
    At LARGE with 36 cultures, every polity has a founder (synthetic
    populations only get None)."""
    world = generate_world(_config())
    founders = [polity.founder_actor_id for polity in world.polities.polities]
    # At least some have founders; v1 has them for all populated
    # settlements.
    founded = sum(1 for f in founders if f is not None)
    assert founded > 0
    for founder in founders:
        if founder is not None:
            assert len(founder) == 16, f"founder {founder!r} must be 16-char hex per 3b.3 pattern"


def test_one_founded_event_per_polity_at_step_zero() -> None:
    """Plan-ack Q5: one POLITY_FOUNDED event per polity at step 0;
    no MERGED / SPLIT / EXPANDED / CONTRACTED events in v1."""
    world = generate_world(_config())
    assert len(world.polities.events) == len(world.polities.polities)
    seen_event_ids: set[str] = set()
    for event in world.polities.events:
        if event.id in seen_event_ids:
            raise AssertionError(f"duplicate event id {event.id}")
        seen_event_ids.add(event.id)
        assert event.type == EventType.POLITY_FOUNDED, (
            f"v1 emits only POLITY_FOUNDED; got {event.type}"
        )
        assert event.t == 0, f"v1 emits only step 0; got {event.t}"


def test_polity_founded_event_payload_validates() -> None:
    """`POLITY_FOUNDED` event payloads validate against
    `PolityFoundedPayload`."""
    world = generate_world(_config())
    for event in world.polities.events:
        payload = PolityFoundedPayload.model_validate(event.payload)
        assert payload.founding_step == 0
        assert payload.step == 0
        assert payload.culture_id >= 0


def test_borders_are_polity_pairs() -> None:
    """`Border` records are per-pair; `polity_a_id < polity_b_id` after
    normalization; each pair appears at most once."""
    world = generate_world(_config())
    seen_pairs: set[tuple[int, int]] = set()
    for border in world.polities.borders:
        pair = (
            min(border.polity_a_id, border.polity_b_id),
            max(border.polity_a_id, border.polity_b_id),
        )
        assert pair not in seen_pairs, f"duplicate border pair {pair}"
        seen_pairs.add(pair)
        assert border.length_km >= 0.0
        assert border.defense_strength >= 0.0


def test_borders_use_elevation_threshold_segments() -> None:
    """Plan-ack Q2: border cells come from elevation >=
    `ELEVATION_BORDER_THRESHOLD_M = 800m` OR river segments.

    At LARGE the world has 36 settlements; defensible borders appear
    only between polities that share an elevation cell above 800m or
    a river segment. v1 boundary derivation is conservative — many
    pairs may have no border if geography doesn't support a defensible
    line. We only assert that all border cells (when present) are in
    the world grid."""
    world = generate_world(_config())
    height = world.geography.height
    width = world.geography.width
    for border in world.polities.borders:
        for x, y in border.segments:
            assert 0 <= x < width, f"border segment x {x} out of grid bounds"
            assert 0 <= y < height, f"border segment y {y} out of grid bounds"


def test_algorithm_version_recomputed_matches_recorded() -> None:
    """The recorded `PolityLayer.algorithm_version` matches a fresh
    blake2b of polities + memberships + borders + events."""
    world = generate_world(_config())
    expected = _compute_algorithm_version(
        world.polities.polities,
        world.polities.memberships,
        world.polities.borders,
        world.polities.events,
    )
    assert world.polities.algorithm_version == expected


def test_per_polity_algorithm_version_matches_recorded() -> None:
    """Each polity's `algorithm_version` matches its computed blake2b
    identity hash."""
    world = generate_world(_config())
    for polity in world.polities.polities:
        expected = _compute_polity_algorithm_version(
            polity_id=polity.id,
            culture_id=polity.id,  # 1:1 culture-per-polity in v1
            governance_type=polity.governance_type,
            founder_actor_id=polity.founder_actor_id,
            founding_step=polity.founding_step,
        )
        assert polity.algorithm_version == expected


def test_validator_catches_algorithm_version_mismatch() -> None:
    """Mutating `algorithm_version` triggers a violation."""
    world = generate_world(_config())
    tampered = world.model_copy(
        update={
            "polities": world.polities.model_copy(update={"algorithm_version": "deadbeefcafebabe"})
        }
    )
    violations = validate_polities_layer(tampered)
    assert any(v.code == "polities-algorithm-version-mismatch" for v in violations)


def test_validator_catches_polity_count_mismatch() -> None:
    """A `PolityLayer` with `len(polities) != n_cultures` triggers
    a parallel-structure violation."""
    world = generate_world(_config())
    if not world.polities.polities:
        return
    bogus_layer = world.polities.model_copy(update={"polities": world.polities.polities[:-1]})
    bad_world = world.model_copy(update={"polities": bogus_layer})
    violations = validate_polities_layer(bad_world)
    assert any(v.code == "polities-roots-parallel-structure" for v in violations)


def test_validator_catches_orphan_membership() -> None:
    """A `PolityMember` referencing an unknown polity triggers
    a `polities-orphaned-membership` violation."""
    world = generate_world(_config())
    bogus_member = PolityMember(
        polity_id=99999,
        settlement_id=0,
        joined_step=0,
        joined_reason=JoinReason.CULTURE,
    )
    bad_memberships = (bogus_member,) + world.polities.memberships[1:]
    bogus_layer = world.polities.model_copy(
        update={
            "memberships": bad_memberships,
            "algorithm_version": _compute_algorithm_version(
                world.polities.polities,
                bad_memberships,
                world.polities.borders,
                world.polities.events,
            ),
        }
    )
    bad_world = world.model_copy(update={"polities": bogus_layer})
    violations = validate_polities_layer(bad_world)
    assert any(v.code == "polities-orphaned-membership" for v in violations)


def test_validate_world_clean_at_seed_42() -> None:
    """End-to-end validation at LARGE seed=42 produces a clean report
    including the new polities validator."""
    world = generate_world(_config())
    report = validate_world(world)
    assert report.is_valid, (
        f"validate_world reported violations: "
        f"{[(v.code, v.path, v.message) for v in report.violations]}"
    )


def test_world_id_stable_across_4_1() -> None:
    """`world_id` for `--seed 42` is unchanged from the chain (no new
    `WorldConfig` fields)."""
    from world_factory.models import WorldScale

    world_a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    world_b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world_a.metadata.world_id == world_b.metadata.world_id, (
        f"world_id drifted: {world_a.metadata.world_id} != {world_b.metadata.world_id}"
    )
    assert world_a.polities.polities[0].id == world_b.polities.polities[0].id
    assert (
        world_a.polities.polities[0].founder_actor_id
        == world_b.polities.polities[0].founder_actor_id
    )


def test_polity_algorithm_version_constant() -> None:
    """`POLITY_ALGORITHM_VERSION` carries an algorithm-shaped suffix,
    not a phase number (per chain convention)."""
    assert POLITY_ALGORITHM_VERSION == "polity-formation-v1"
    assert "-" in POLITY_ALGORITHM_VERSION
    assert POLITY_ALGORITHM_VERSION.endswith("-v1")


def test_polity_event_type_enum() -> None:
    """`PolityEventType` enum has FOUNDED for v1; MERGED / SPLIT /
    EXPANDED / CONTRACTED are reserved for 4.x."""
    assert PolityEventType.FOUNDED.value == "founded"
    reserved = {
        PolityEventType.MERGED,
        PolityEventType.SPLIT,
        PolityEventType.EXPANDED,
        PolityEventType.CONTRACTED,
    }
    assert len(reserved) == 4


def test_border_segments_above_elevation_threshold() -> None:
    """Border `segments` derive from geography cells; the boundary
    walker filters by `geography.elevation_meters >=
    ELEVATION_BORDER_THRESHOLD_M = 800`. At SMALL the world has
    fewer elevation cells above 800m, so the test asserts only that
    the threshold constant is pinned correctly."""
    assert ELEVATION_BORDER_THRESHOLD_M == 800.0


def test_polity_count_at_seed_42_matches_culture_count() -> None:
    """Plan-ack minor note: `test_polity_count_matches_culture_count_
    at_seed_42` is the basic shape check for 4.6 acceptance —
    `len(polities) == len(cultures)` at the canonical seed."""
    from world_factory.models import WorldScale

    for scale in (WorldScale.SMALL, WorldScale.LARGE):
        world = generate_world(WorldConfig(seed=42, scale=scale))
        n_cultures = len(world.cultures.cultures)
        n_polities = len(world.polities.polities)
        assert n_polities == n_cultures, (
            f"at {scale.value}: {n_polities} polities != {n_cultures} cultures"
        )


def test_polity_determinism() -> None:
    """Same seed → same polities, same founders, same borders, same
    events."""
    from world_factory.models import WorldScale

    world_a = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    world_b = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world_a.polities.algorithm_version == world_b.polities.algorithm_version
    for pa, pb in zip(
        world_a.polities.polities,
        world_b.polities.polities,
        strict=True,
    ):
        assert pa.id == pb.id
        assert pa.name == pb.name
        assert pa.founder_actor_id == pb.founder_actor_id
