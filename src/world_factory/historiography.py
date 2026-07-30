"""Phase 5.1 historiography — read-mostly self-awareness over polities.

`build_historiography(world)` derives a `HistoriographyLayer` from
the world's event log + polity records. v1 emits two record types:

- `SourceGap`: a gap in the polity-level event record. Emitted when
  the gap between consecutive events at a polity's primary
  settlement exceeds `SOURCE_GAP_THRESHOLD_STEPS` (default 20 steps
  ≈ 20 years per the plan-ack).
- `DisputedEvent`: a record-level conflict between rival polities.
  v1 emits zero disputes because polities emit only
  POLITY_FOUNDED events (no MERGED / SPLIT / EXPANDED /
  CONTRACTED). The slot is reserved for 5.3.x when 4.x surfaces
  per-step polity transitions that rival records can disagree
  about.

The layer is a stored top-level field on `WorldModel` so consumers
(v2 explorer, Phase 6 query surface) get O(1) access.

Algorithm version: blake2b hash of the source-gap + disputed-event
tuple, keyed by the `b"histori"` person namespace. Same
trust-boundary contract as 3a.5 / 3b.x / 4.1 / 5.1 causal-graph.

Validator order (`validate_historiography_layer`):
1. `_validate_algorithm_version` (FIRST; matches 3a.5 / 3b.x /
   4.1 / 5.1 pattern).
2. `_validate_source_gap_integrity` (per-`SourceGap` field
   integrity, polity_id references a real polity, length_steps >=
   SOURCE_GAP_THRESHOLD_STEPS, start_step < end_step).
3. `_validate_disputed_event_integrity` (per-`DisputedEvent` field
   integrity — empty in v1, but the slot is reserved).
4. `_validate_polity_referential_integrity` (every `polity_id` in
   gaps and disputes exists in `world.polities.polities`).
"""

from __future__ import annotations

import hashlib
import json

from world_factory.constants import (
    HISTORIOGRAPHY_ALGORITHM_VERSION,
    SOURCE_GAP_THRESHOLD_STEPS,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    DisputedEvent,
    HistoriographyLayer,
    ProvenanceRecord,
    SourceGap,
    WorldEvent,
    WorldModel,
)


_HISTORI_BLAKE_PERSON = b"histori"


def _compute_algorithm_version(
    source_gaps: tuple[SourceGap, ...],
    disputed_events: tuple[DisputedEvent, ...],
) -> str:
    """blake2b hash of the historiography record tuple. 16-char hex.

    Mirrors the 3a.5 / 3b.x / 4.1 / 5.1 pattern: mutations or
    re-ordering change the hash, allowing the trust boundary to
    detect silent corruption at `WorldModel.model_validate_json`.
    """
    digest = hashlib.blake2b(digest_size=8, person=_HISTORI_BLAKE_PERSON)
    state = {
        "source_gaps": [gap.model_dump(mode="json") for gap in source_gaps],
        "disputed_events": [
            dispute.model_dump(mode="json") for dispute in disputed_events
        ],
    }
    encoded_state = json.dumps(
        state,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    digest.update(encoded_state)
    return digest.hexdigest()


def _primary_settlement_id(world: WorldModel, polity_id: int) -> int | None:
    """Return the primary settlement for a polity, or None.

    v1 simplifies: the primary settlement is `memberships[0].
    settlement_id` for that polity (the first membership by
    polity_id + joined_step ordering). Every polity has exactly
    one membership per settlement in v1 (4.1 Q7).
    """
    for membership in world.polities.memberships:
        if membership.polity_id == polity_id:
            return membership.settlement_id
    return None


def _events_for_polity(
    world: WorldModel,
    polity_id: int,
    primary_settlement_id: int,
) -> tuple[WorldEvent, ...]:
    """Filter the union event stream to events touching a polity.

    v1 includes:
    - Polity events with `payload.polity_id == polity_id` (covers
      POLITY_FOUNDED).
    - World events with `location.settlement_id ==
      primary_settlement_id` (covers demography / culture /
      religion / kinship events at the polity's primary
      settlement).

    Sorted by `(t, id)` for stable gap detection.
    """
    filtered: list[WorldEvent] = []
    for event in world.polities.events:
        payload = event.payload
        if isinstance(payload, dict) and payload.get("polity_id") == polity_id:
            filtered.append(event)
    for event in world.events.events:
        if event.location.settlement_id == primary_settlement_id:
            filtered.append(event)
    filtered.sort(key=lambda event: (event.t, event.id))
    return tuple(filtered)


def _detect_source_gaps(
    polity_id: int,
    events: tuple[WorldEvent, ...],
    primary_settlement_id: int,
) -> list[SourceGap]:
    """Emit SourceGap records for gaps >= SOURCE_GAP_THRESHOLD_STEPS.

    A "gap" is the distance between consecutive event `t` values.
    If `events[i+1].t - events[i].t > SOURCE_GAP_THRESHOLD_STEPS`,
    emit one SourceGap covering `(events[i].t + 1,
    events[i+1].t - 1)`. Half-open: start inclusive, end exclusive.
    """
    gaps: list[SourceGap] = []
    for i in range(len(events) - 1):
        prev_t = events[i].t
        next_t = events[i + 1].t
        gap_length = next_t - prev_t - 1
        if gap_length >= SOURCE_GAP_THRESHOLD_STEPS:
            start_step = prev_t + 1
            end_step = next_t
            gaps.append(
                SourceGap(
                    polity_id=polity_id,
                    start_step=start_step,
                    end_step=end_step,
                    length_steps=gap_length,
                    primary_settlement_id=primary_settlement_id,
                )
            )
    return gaps


def build_historiography(world: WorldModel) -> HistoriographyLayer:
    """Build the historiography layer from the world's events.

    v1: source gaps from existing event walks only; disputed events
    slot is empty (vacuously satisfies the 5.4 acceptance gate
    because there are no rival records to disagree).
    """
    gaps: list[SourceGap] = []
    for polity in world.polities.polities:
        primary_settlement_id = _primary_settlement_id(world, polity.id)
        if primary_settlement_id is None:
            continue
        polity_events = _events_for_polity(world, polity.id, primary_settlement_id)
        gaps.extend(_detect_source_gaps(polity.id, polity_events, primary_settlement_id))

    gaps.sort(
        key=lambda gap: (gap.polity_id, gap.start_step, gap.primary_settlement_id)
    )
    gaps_tuple = tuple(gaps)
    # v1: zero disputes. DisputedEvent records require per-step
    # MERGED / SPLIT / EXPANDED / CONTRACTED events from 4.x —
    # those arrive in 4.2.x / 4.3.x, not v1.
    disputes_tuple: tuple[DisputedEvent, ...] = ()
    algorithm_version = _compute_algorithm_version(gaps_tuple, disputes_tuple)
    return HistoriographyLayer(
        source_gaps=gaps_tuple,
        disputed_events=disputes_tuple,
        algorithm_version=algorithm_version,
    )


def validate_historiography_layer(
    world: WorldModel,
) -> list[InvariantViolation]:
    """Phase 5.1 historiography-layer invariants.

    Order:
    1. `_validate_algorithm_version` (FIRST; matches 3a.5 / 3b.x /
       4.1 / 5.1 pattern).
    2. `_validate_source_gap_integrity` (per-`SourceGap` field
       integrity, polity_id references a real polity,
       length_steps >= SOURCE_GAP_THRESHOLD_STEPS,
       start_step < end_step).
    3. `_validate_disputed_event_integrity` (per-`DisputedEvent`
       field integrity — empty in v1, but the slot is reserved).
    4. `_validate_polity_referential_integrity` (every `polity_id`
       in gaps and disputes exists in `world.polities.polities`).
    """
    violations: list[InvariantViolation] = []
    layer = world.historiography
    real_polity_ids = {polity.id for polity in world.polities.polities}

    expected_version = _compute_algorithm_version(
        layer.source_gaps, layer.disputed_events
    )
    if layer.algorithm_version != expected_version:
        violations.append(
            _violation(
                "historiography-algorithm-version-mismatch",
                "world.historiography.algorithm_version",
                (
                    f"historiography algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; layer was "
                    f"mutated or re-ordered outside the generator"
                ),
            )
        )

    for index, gap in enumerate(layer.source_gaps):
        if gap.length_steps < SOURCE_GAP_THRESHOLD_STEPS:
            violations.append(
                _violation(
                    "historiography-source-gap-below-threshold",
                    f"world.historiography.source_gaps.{index}.length_steps",
                    (
                        f"SourceGap for polity {gap.polity_id} has "
                        f"length_steps={gap.length_steps} below "
                        f"SOURCE_GAP_THRESHOLD_STEPS="
                        f"{SOURCE_GAP_THRESHOLD_STEPS}"
                    ),
                )
            )
        if gap.start_step >= gap.end_step:
            violations.append(
                _violation(
                    "historiography-source-gap-invalid-range",
                    f"world.historiography.source_gaps.{index}",
                    (
                        f"SourceGap for polity {gap.polity_id} has "
                        f"start_step={gap.start_step} >= "
                        f"end_step={gap.end_step}"
                    ),
                )
            )
        if gap.polity_id not in real_polity_ids:
            violations.append(
                _violation(
                    "historiography-source-gap-missing-polity",
                    f"world.historiography.source_gaps.{index}.polity_id",
                    (
                        f"SourceGap polity_id {gap.polity_id} not in "
                        f"world.polities.polities"
                    ),
                )
            )

    for index, dispute in enumerate(layer.disputed_events):
        if not dispute.event_id:
            violations.append(
                _violation(
                    "historiography-disputed-event-empty-id",
                    f"world.historiography.disputed_events.{index}.event_id",
                    "DisputedEvent has empty event_id",
                )
            )
        if not dispute.polity_ids:
            violations.append(
                _violation(
                    "historiography-disputed-event-empty-polities",
                    f"world.historiography.disputed_events.{index}.polity_ids",
                    "DisputedEvent has empty polity_ids",
                )
            )
        if not dispute.reason:
            violations.append(
                _violation(
                    "historiography-disputed-event-empty-reason",
                    f"world.historiography.disputed_events.{index}.reason",
                    "DisputedEvent has empty reason",
                )
            )
        for polity_id in dispute.polity_ids:
            if polity_id not in real_polity_ids:
                violations.append(
                    _violation(
                        "historiography-disputed-event-missing-polity",
                        f"world.historiography.disputed_events.{index}.polity_ids",
                        (
                            f"DisputedEvent polity_id {polity_id} not "
                            f"in world.polities.polities"
                        ),
                    )
                )

    return violations


def historiography_provenance() -> ProvenanceRecord:
    """Provenance record describing the historiography builder."""
    return ProvenanceRecord(
        output_path="historiography",
        process=(
            "source-gaps-from-event-walks + disputed-events-vacuous-v1 "
            "+ blake2b-algorithm-version"
        ),
        input_paths=(
            "polities.polities",
            "polities.memberships",
            "polities.events",
            "events.events",
        ),
        algorithm_version=HISTORIOGRAPHY_ALGORITHM_VERSION,
    )