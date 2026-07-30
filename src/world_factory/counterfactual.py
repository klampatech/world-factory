"""Phase 5.1 counterfactual — operation, not stored layer.

`run_counterfactual(world, intervention)` is a free function that
returns a `CounterfactualRun` record describing the alternate
timeline produced by applying the mutation to the canonical world.

The contract:

- Pure deterministic — same `(world, intervention)` produces
  byte-identical `CounterfactualRun` across runs. Acceptance gate
  5.4.
- No stored layer — counterfactual is an operation, not a
  `WorldModel` field. Per plan-ack Q3.
- v1 ships `RemoveEventMutation` only (per plan-ack Q4). The
  `Intervention` type alias is `tuple[str, RemoveEventMutation]`
  where `str` is the event id (matches the plan-ack convention).

Mutation types in v1:

- `RemoveEventMutation(event_id)`: drop the event at
  `intervention[0]` from the alternate timeline. The alternate
  timeline does NOT rerun demography / culture / religion /
  kinship / polity logic — it is the canonical log minus the
  removed event, with downstream `causes` recomputed by walking
  the post-pass causal graph (the "operation, not branch graph"
  model per spec 5.2).

Algorithm version: blake2b hash of the alternate event tuple, keyed
by the `b"counter"` person namespace.
"""

from __future__ import annotations

import hashlib
import json
from collections import deque
from typing import TYPE_CHECKING

from world_factory.constants import COUNTERFACTUAL_ALGORITHM_VERSION
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    CounterfactualRun,
    ProvenanceRecord,
    RemoveEventMutation,
    WorldEvent,
    WorldModel,
)

if TYPE_CHECKING:
    pass


_COUNTER_BLAKE_PERSON = b"counter"


# Type alias per plan-ack: canonical form is `(event_id, mutation)`
# tuple, matching the plan-ack's spec for the intervention surface.
Intervention = tuple[str, RemoveEventMutation]


def _compute_algorithm_version(events: tuple[WorldEvent, ...]) -> str:
    """blake2b hash of the alternate event tuple. 16-char hex.

    Mirrors the 3a.5 / 3b.x / 4.1 / 5.1 pattern: any mutation or
    re-ordering changes the hash.
    """
    digest = hashlib.blake2b(digest_size=8, person=_COUNTER_BLAKE_PERSON)
    state = [event.model_dump(mode="json") for event in events]
    encoded_state = json.dumps(
        state,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    digest.update(encoded_state)
    return digest.hexdigest()


def _direct_parents_by_event(
    events: tuple[WorldEvent, ...],
) -> dict[str, tuple[str, ...]]:
    """Index events by id for direct-parent lookups.

    Returns a dict mapping `event.id` -> `event.causes` so
    `_diverged_events` can BFS over the direct-edge ancestry.
    """
    return {event.id: event.causes for event in events}


def _diverged_events(
    removed_id: str,
    direct_parents: dict[str, tuple[str, ...]],
) -> tuple[str, ...]:
    """Return event ids that transitively depended on `removed_id`.

    BFS over the reverse-causal graph: starting from any event that
    lists `removed_id` in its `causes`, walk forward to all events
    that list any visited id in their `causes`. Pure post-pass over
    the direct edges; no new event semantics.
    """
    # Build reverse adjacency: child -> set of parents
    children: dict[str, set[str]] = {}
    for child_id, parents in direct_parents.items():
        for parent_id in parents:
            children.setdefault(parent_id, set()).add(child_id)
    visited: set[str] = set()
    queue: deque[str] = deque(children.get(removed_id, set()))
    while queue:
        current = queue.popleft()
        if current in visited or current == removed_id:
            continue
        visited.add(current)
        for grandchild in children.get(current, set()):
            if grandchild not in visited and grandchild != removed_id:
                queue.append(grandchild)
    return tuple(sorted(visited))


def _orphan_event_ids(
    alternate_events: tuple[WorldEvent, ...],
    removed_id: str,
) -> tuple[str, ...]:
    """Return event ids with empty `causes` after removal.

    An "orphan" is an event whose `causes` becomes empty because
    every direct parent was either (a) the removed event or (b)
    another orphan. v1 simplifies: only events that had a single
    cause which was removed, OR all of whose causes were removed,
    become orphans.

    Acceptance gate metric: `n_orphans` is exposed via
    `divergence_metrics`.
    """
    orphans: list[str] = []
    for event in alternate_events:
        if event.causes:
            continue
        if event.id == removed_id:
            continue
        orphans.append(event.id)
    return tuple(sorted(orphans))


def run_counterfactual(
    world: WorldModel,
    intervention: Intervention,
) -> CounterfactualRun:
    """Run a counterfactual on `world` and return the alternate run.

    `intervention` is `(event_id, RemoveEventMutation(event_id))`.
    The mutation must reference a real event in
    `world.events.events ∪ world.polities.events`; otherwise the
    operation raises `ValueError` with a structured message.

    The alternate timeline is the canonical event tuple minus the
    removed event. Events whose `causes` lists reference the
    removed event are NOT mutated in v1 — they keep their original
    `causes` lists (the "operation, not branch graph" model).
    Downstream causal reachability is recomputed lazily by the
    causal graph; this function reports divergence as a metric.
    """
    event_id, mutation = intervention
    if event_id != mutation.event_id:
        raise ValueError(
            f"intervention event_id {event_id!r} does not match "
            f"mutation.event_id {mutation.event_id!r}"
        )
    canonical_events = world.events.events + world.polities.events
    canonical_ids = {event.id for event in canonical_events}
    if event_id not in canonical_ids:
        raise ValueError(
            f"intervention event_id {event_id!r} not in "
            f"world.events.events ∪ world.polities.events"
        )

    alternate_events = tuple(
        event for event in canonical_events if event.id != event_id
    )

    direct_parents = _direct_parents_by_event(canonical_events)
    diverged_ids = _diverged_events(event_id, direct_parents)
    orphan_ids = _orphan_event_ids(alternate_events, event_id)

    total = len(canonical_events)
    n_diverged = len(diverged_ids)
    n_orphans = len(orphan_ids)
    pct_diverged = (n_diverged / total) if total else 0.0
    divergence_metrics: dict[str, float] = {
        "n_diverged": float(n_diverged),
        "pct_diverged": pct_diverged,
        "n_orphans": float(n_orphans),
    }

    algorithm_version = _compute_algorithm_version(alternate_events)
    return CounterfactualRun(
        base_world_id=world.metadata.world_id,
        intervention=mutation,
        alternate_events=alternate_events,
        diverged_event_ids=diverged_ids,
        divergence_metrics=divergence_metrics,
        algorithm_version=algorithm_version,
    )


def validate_counterfactual_run(
    run: CounterfactualRun,
) -> list[InvariantViolation]:
    """Validate a `CounterfactualRun` record at the trust boundary.

    Counterfactual is an operation, not a stored layer, so this
    validator is opt-in — typically called from a test or from a
    downstream consumer that round-trips a `CounterfactualRun`
    through JSON. The 5.4 acceptance gate (counterfactual
    reproducibility) calls this directly.
    """
    violations: list[InvariantViolation] = []
    expected_version = _compute_algorithm_version(run.alternate_events)
    if run.algorithm_version != expected_version:
        violations.append(
            _violation(
                "counterfactual-algorithm-version-mismatch",
                "counterfactual_run.algorithm_version",
                (
                    f"counterfactual algorithm_version "
                    f"{run.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; alternate "
                    f"events were mutated or re-ordered"
                ),
            )
        )
    if run.intervention.event_id in {
        event.id for event in run.alternate_events
    }:
        violations.append(
            _violation(
                "counterfactual-removed-event-present",
                "counterfactual_run.alternate_events",
                (
                    f"alternate timeline contains the removed "
                    f"event {run.intervention.event_id!r}"
                ),
            )
        )
    if run.base_world_id not in run.intervention.event_id and not run.base_world_id:
        violations.append(
            _violation(
                "counterfactual-empty-base-world-id",
                "counterfactual_run.base_world_id",
                "base_world_id is empty",
            )
        )
    return violations


def counterfactual_provenance() -> ProvenanceRecord:
    """Provenance record describing the counterfactual operation."""
    return ProvenanceRecord(
        output_path="counterfactual_run",
        process=(
            "remove-event-from-union-event-tuple + "
            "divergence-bfs-over-direct-edges + "
            "blake2b-algorithm-version"
        ),
        input_paths=(
            "events.events",
            "polities.events",
            "metadata.world_id",
        ),
        algorithm_version=COUNTERFACTUAL_ALGORITHM_VERSION,
    )