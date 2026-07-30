"""Phase 5.1 historiography — read-mostly self-awareness layer.

HistoriographyLayer on WorldModel (additive top-level field),
`SourceGap` derived from event-walk gaps at each polity's primary
settlement, `DisputedEvent` slot reserved but empty in v1
(vacuous acceptance per plan-ack Q6 / 5.4 disputed-events
acceptance), and the 5.4 acceptance gate "≥1 source gap per
polity on average across N seeds".
"""

from __future__ import annotations

from world_factory.constants import (
    HISTORIOGRAPHY_ALGORITHM_VERSION,
    SOURCE_GAP_THRESHOLD_STEPS,
)
from world_factory.generator import generate_world
from world_factory.historiography import (
    _compute_algorithm_version,
    build_historiography,
    historiography_provenance,
    validate_historiography_layer,
)
from world_factory.models import (
    DisputedEvent,
    HistoriographyLayer,
    SourceGap,
    WorldConfig,
    WorldScale,
)


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_historiography_layer() -> None:
    """`WorldModel.historiography` is a `HistoriographyLayer`
    aggregate (additive top-level field per plan-ack Q2)."""
    world = generate_world(_config())
    assert world.historiography is not None
    assert isinstance(world.historiography, HistoriographyLayer)


def test_historiography_algorithm_version_pinned() -> None:
    """`algorithm_version` matches the `b"histori"` blake2b hash
    of the gap + dispute tuple."""
    world = generate_world(_config())
    expected = _compute_algorithm_version(
        world.historiography.source_gaps,
        world.historiography.disputed_events,
    )
    assert world.historiography.algorithm_version == expected, (
        f"historiography.algorithm_version "
        f"{world.historiography.algorithm_version!r} does not match "
        f"recomputed {expected!r}"
    )


def test_historiography_disputed_events_vacuous_v1() -> None:
    """5.4 acceptance gate — disputed events: vacuous in v1
    (zero disputes because polities emit only POLITY_FOUNDED at
    step 0; no MERGED / SPLIT / EXPANDED / CONTRACTED until
    4.2.x / 4.3.x)."""
    world = generate_world(_config())
    assert world.historiography.disputed_events == (), (
        f"v1 emits 0 disputed events; got "
        f"{len(world.historiography.disputed_events)}"
    )


def test_historiography_provenance_recorded() -> None:
    """Provenance record describes the historiography builder."""
    record = historiography_provenance()
    assert record.output_path == "historiography"
    assert record.algorithm_version == HISTORIOGRAPHY_ALGORITHM_VERSION


def test_historiography_validator_clean_for_fresh_world() -> None:
    """`validate_historiography_layer` returns no violations for a
    freshly generated canonical world (small scale)."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    violations = validate_historiography_layer(world)
    codes = [v.code for v in violations]
    assert not violations, f"unexpected violations: {codes}"


def test_source_gap_threshold_respected() -> None:
    """Every `SourceGap.length_steps >= SOURCE_GAP_THRESHOLD_STEPS`."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    for gap in world.historiography.source_gaps:
        assert gap.length_steps >= SOURCE_GAP_THRESHOLD_STEPS, (
            f"gap {gap.polity_id} has length_steps={gap.length_steps}, "
            f"below threshold {SOURCE_GAP_THRESHOLD_STEPS}"
        )


def test_source_gap_polity_referential_integrity() -> None:
    """Every `SourceGap.polity_id` references a real polity in
    `world.polities.polities`."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    real_polity_ids = {polity.id for polity in world.polities.polities}
    for gap in world.historiography.source_gaps:
        assert gap.polity_id in real_polity_ids, (
            f"SourceGap polity_id {gap.polity_id} not in world.polities.polities"
        )


def test_source_gap_start_before_end() -> None:
    """Every `SourceGap.start_step < end_step` (half-open range)."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    for gap in world.historiography.source_gaps:
        assert gap.start_step < gap.end_step, (
            f"gap {gap.polity_id} has start_step={gap.start_step} "
            f">= end_step={gap.end_step}"
        )


def test_historiography_determinism_byte_equal() -> None:
    """Two runs with the same seed produce byte-identical
    `HistoriographyLayer`."""
    config = _config()
    world_a = generate_world(config)
    world_b = generate_world(config)
    assert (
        world_a.historiography.model_dump()
        == world_b.historiography.model_dump()
    ), "same seed produced different historiography layers"


def test_historiography_handles_empty_polities() -> None:
    """Build with no polities yields an empty historiography layer
    with stable algorithm_version."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    empty_world = world.model_copy(
        update={
            "polities": world.polities.model_copy(
                update={
                    "polities": (),
                    "memberships": (),
                    "borders": (),
                    "events": (),
                    "algorithm_version": "",
                }
            ),
        }
    )
    layer = build_historiography(empty_world)
    assert layer.source_gaps == ()
    assert layer.disputed_events == ()
    assert layer.algorithm_version == _compute_algorithm_version(
        layer.source_gaps, layer.disputed_events
    )


def test_historiography_layer_aggregate_fields() -> None:
    """`HistoriographyLayer` exposes the three required fields."""
    world = generate_world(_config())
    layer = world.historiography
    assert isinstance(layer.source_gaps, tuple)
    assert isinstance(layer.disputed_events, tuple)
    assert isinstance(layer.algorithm_version, str)


def test_source_gap_aggregate_fields() -> None:
    """`SourceGap` exposes the five required fields with the
    pinned types."""
    gap = SourceGap(
        polity_id=0,
        start_step=1,
        end_step=20,
        length_steps=19,
        primary_settlement_id=0,
    )
    assert gap.polity_id == 0
    assert gap.start_step == 1
    assert gap.end_step == 20
    assert gap.length_steps == 19
    assert gap.primary_settlement_id == 0


def test_disputed_event_aggregate_fields() -> None:
    """`DisputedEvent` exposes the three required fields (slot
    reserved for v1 — zero disputes until 4.x surfaces per-step
    polity transitions)."""
    dispute = DisputedEvent(
        event_id="01f2c0f1abcdef01",
        polity_ids=(0, 1),
        reason="rival-records-disagree",
    )
    assert dispute.event_id == "01f2c0f1abcdef01"
    assert dispute.polity_ids == (0, 1)
    assert dispute.reason == "rival-records-disagree"