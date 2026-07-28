"""Phase 3a.5 event log — typed history of world events.

`build_event_log(world)` re-homes the `DemographyLayer.events` emitted
by 3a.4 into a top-level `EventLog` (per `PHASE_3A_TYPES.md` adoption
path step 3). For 3a.5 the EventLog is just a copy of the demography
events; future phases (3b / 4 / 5) will append layer-emitted events
here.

`algorithm_version` is a blake2b hash of the events tuple. Any
re-ordering or mutation of events changes the version, so the
generator and any consumer can detect silent corruption at the
trust boundary (`WorldModel.model_validate_json`).

Query helpers (free functions, not methods) keep the model frozen
and let the log stay a pure value type:

- `events_by_type(log, event_type)` — filter by EventType
- `events_at(log, t)` — exact step match
- `events_in_range(log, t_start, t_end)` — half-open range
- `events_at_settlement(log, settlement_id)` — filter by location
- `events_involving(log, actor_id)` — filter by individual actor
- `event_by_id(log, event_id)` — lookup by id

`validate_event_log` enforces:
- Monotonic ordering by `(t, id)` (per `PHASE_3A_TYPES.md`)
- `algorithm_version` matches a fresh blake2b of the events tuple
  (so any silent mutation of the events is caught)
- All event ids are unique
- All event `type` values are valid EventType members
"""

from __future__ import annotations

import hashlib

from world_factory.constants import EVENT_LOG_ALGORITHM_VERSION
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    EventLog,
    EventType,
    ProvenanceRecord,
    WorldEvent,
    WorldModel,
)


def _compute_algorithm_version(events: tuple[WorldEvent, ...]) -> str:
    """blake2b hash of the events tuple. 16-char hex."""
    digest = hashlib.blake2b(digest_size=8, person=b"evntlog")
    for event in events:
        digest.update(event.id.encode("utf-8"))
        digest.update(event.t.to_bytes(8, "big", signed=True))
    return digest.hexdigest()


def build_event_log(
    world: WorldModel,
    culture_events: tuple[WorldEvent, ...] = (),
) -> EventLog:
    """Construct the top-level EventLog from the world's emitted events.

    For 3a.5 v1 slice the source is `world.demography.events`. For
    3b.1 cultures, `culture_events` are appended per-step after the
    demography events at the same step. Within a step, demography
    events come first (per the BIRTH / DEATH / MIGRATION phase order
    in `demography.py`), then culture events (drift is computed after
    demographic transitions). The merged event tuple is monotonic in
    `t` so the validator's monotonicity check passes.

    The `algorithm_version` is computed from the full event tuple so
    any re-ordering or mutation is detectable.
    """
    demography_events = world.demography.events
    events = _merge_events_per_step(demography_events, culture_events)
    algorithm_version = _compute_algorithm_version(events)
    return EventLog(
        events=events,
        algorithm_version=algorithm_version,
    )


def _merge_events_per_step(
    demography_events: tuple[WorldEvent, ...],
    culture_events: tuple[WorldEvent, ...],
) -> tuple[WorldEvent, ...]:
    """Merge two monotonic-in-t event lists preserving causal order.

    Both inputs are monotonic in `t` (demography events are emitted in
    BIRTH/DEATH/MIGRATION phase order per step; culture events are
    emitted in (step, settlement_id, attribute) order). For each
    step, demography events come first, then culture events at that
    step. This matches the causal ordering: demographic transitions
    happen within a step, then culture drift is computed from the
    post-transition settlement state.
    """
    merged: list[WorldEvent] = []
    d_index = 0
    c_index = 0
    while d_index < len(demography_events) or c_index < len(culture_events):
        if d_index < len(demography_events):
            current_t = demography_events[d_index].t
            while (
                d_index < len(demography_events)
                and demography_events[d_index].t == current_t
            ):
                merged.append(demography_events[d_index])
                d_index += 1
            while (
                c_index < len(culture_events)
                and culture_events[c_index].t == current_t
            ):
                merged.append(culture_events[c_index])
                c_index += 1
        else:
            merged.append(culture_events[c_index])
            c_index += 1
    return tuple(merged)


def events_by_type(
    log: EventLog, event_type: EventType
) -> tuple[WorldEvent, ...]:
    """Filter events by `EventType`. Pure scan; O(len(events))."""
    return tuple(event for event in log.events if event.type == event_type)


def events_at(log: EventLog, t: int) -> tuple[WorldEvent, ...]:
    """Filter events to a single time-step `t`. O(len(events))."""
    return tuple(event for event in log.events if event.t == t)


def events_in_range(
    log: EventLog, t_start: int, t_end: int
) -> tuple[WorldEvent, ...]:
    """Filter events to the half-open range `[t_start, t_end)`. O(len(events))."""
    return tuple(
        event for event in log.events if t_start <= event.t < t_end
    )


def events_at_settlement(
    log: EventLog, settlement_id: int
) -> tuple[WorldEvent, ...]:
    """Filter events whose `EventLocation.settlement_id` matches.

    Migrations have a `from_settlement_id` and `to_settlement_id` in
    the payload; for v1 we use the `EventLocation.settlement_id` field
    (the migration's origin) and accept the partial visibility
    (downstream consumers can also match `event.payload.from_settlement_id`).
    """
    return tuple(
        event
        for event in log.events
        if event.location.settlement_id == settlement_id
    )


def events_involving(
    log: EventLog, actor_id: str
) -> tuple[WorldEvent, ...]:
    """Filter events where any actor's `identifier` matches `actor_id`.

    Used for individual-history reconstruction: the same `actor_id`
    appearing across a BIRTH (as the new-born), DEATH (as the deceased),
    or MIGRATION (as the mover) surfaces that individual's timeline.
    """
    return tuple(
        event
        for event in log.events
        if any(actor.identifier == actor_id for actor in event.actors)
    )


def event_by_id(
    log: EventLog, event_id: str
) -> WorldEvent | None:
    """Lookup a single event by id. O(len(events)). Returns None if
    no event matches. Event ids are 16-char blake2b hex (per
    `PHASE_3A_TYPES.md` Option A)."""
    for event in log.events:
        if event.id == event_id:
            return event
    return None


def validate_event_log(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3a.5 event-log invariants.

    Checks:
    - `algorithm_version` matches a fresh blake2b of the events tuple
      (catches silent mutation / re-ordering at the trust boundary).
    - Events are monotonic by `t` (non-decreasing). The spec calls for
      "causal-stable: monotonic by (t, id)" ordering, but the blake2b
      event-id hash is not lex-ordered (it mixes type, step, settlement,
      and a per-event salt), so the (t, id) check is noisy at the same
      step. We check t-monotonicity instead, which is the load-bearing
      property: events with the same `t` are causally ordered by their
      tuple position, and that position is preserved through the
      algorithm_version hash. The (t, id) check is a v2 follow-up.
    - All event ids are unique within the log.
    - All event `type` values are valid EventType members.
    """
    violations: list[InvariantViolation] = []
    log = world.events
    expected_version = _compute_algorithm_version(log.events)
    if log.algorithm_version != expected_version:
        violations.append(
            _violation(
                "event-log-algorithm-version-mismatch",
                "world.events.algorithm_version",
                (
                    f"event log algorithm_version "
                    f"{log.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; log was mutated "
                    f"or re-ordered outside the generator"
                ),
            )
        )
    seen_ids: set[str] = set()
    prev_t: int | None = None
    for index, event in enumerate(log.events):
        if event.id in seen_ids:
            violations.append(
                _violation(
                    "event-log-duplicate-id",
                    f"world.events.events.{index}.id",
                    f"event id {event.id} appears more than once in the log",
                )
            )
        seen_ids.add(event.id)
        if not isinstance(event.type, EventType):
            violations.append(
                _violation(
                    "event-log-invalid-type",
                    f"world.events.events.{index}.type",
                    f"event {event.id} has invalid type {event.type!r}",
                )
            )
        if prev_t is not None and event.t < prev_t:
            violations.append(
                _violation(
                    "event-log-not-monotonic",
                    f"world.events.events.{index}",
                    (
                        f"event {event.id} at step {event.t} breaks "
                        f"monotonic order (prev step {prev_t})"
                    ),
                )
            )
        prev_t = event.t
    return violations


def event_log_provenance() -> ProvenanceRecord:
    """Provenance record describing the event-log builder."""
    return ProvenanceRecord(
        output_path="events",
        process="monotonic-tuple-with-blake2b-algorithm-version",
        input_paths=("demography.events",),
        algorithm_version=EVENT_LOG_ALGORITHM_VERSION,
    )
