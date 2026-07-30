"""Phase 5.1 causal graph — read-mostly post-pass over the EventLog.

`build_causal_graph(world)` derives a `CausalGraphLayer` from
`world.events.events` (3a.5 / 3b.x layers) and
`world.polities.events` (4.1 POLITY_FOUNDED). The layer is a stored
top-level field on `WorldModel` so consumers (v2 explorer, Phase 6
query surface, downstream analysis agents) get O(1) access without
re-deriving per query.

Edge types are pinned:

    CAUSAL_EDGE_TYPES = ("direct", "indirect", "contingent")

- DIRECT: declared in `WorldEvent.causes` by the event constructor
  at emission time. One edge per `(cause_id, event.id)` pair.
  Weight = 1.0.
- INDIRECT: derived post-pass over (spatial proximity, temporal
  window). Emitted when event B follows event A in time, A and B
  happen at settlements within `INDIRECT_PROXIMITY_KM` of each
  other, and the temporal gap is at most
  `INDIRECT_TEMPORAL_WINDOW_STEPS`. Weight = `1.0 / (1.0 +
  distance_km)` — closer in space = stronger edge.
- CONTINGENT: derived post-pass over shared upstream. Emitted when
  two events share at least one direct ancestor within depth
  `CONTINGENT_DEPTH`. Pure post-pass over the direct edges. Weight
  = 1.0.

Edge-type ordering is load-bearing for the transitive-reduction
post-pass and the algorithm-version blake2b hash.

The `algorithm_version` is computed from the sorted edge tuple so
any re-ordering or mutation breaks the version — same
trust-boundary contract as 3a.5 / 3b.x / 4.1. Uses the `b"causal"`
blake2b person namespace.

Validator order (`validate_causal_graph_layer`):
1. `_validate_algorithm_version` (FIRST; matches 3a.5 / 3b.x /
   4.1 pattern).
2. `_validate_edge_integrity` (per-`CausalEdge` field integrity,
   edge_type enum membership, weight range, non-empty reason).
3. `_validate_referential_integrity` (every `source_id` and
   `target_id` exists in `world.events.events ∪
   world.polities.events`).
4. `_validate_no_duplicate_edges` (no two edges share the same
   `(source_id, target_id, edge_type)` triple).
5. `_validate_reachability` (every event in the union is reachable
   from a Phase 0–2 trigger via the edge set).
"""

from __future__ import annotations

import hashlib
import json
import math
from collections import deque
from typing import TYPE_CHECKING

from world_factory.constants import (
    CAUSAL_GRAPH_ALGORITHM_VERSION,
    CONTINGENT_DEPTH,
    GRID_CELL_AREA_KILOMETERS_SQUARED,
    INDIRECT_PROXIMITY_KM,
    INDIRECT_TEMPORAL_WINDOW_STEPS,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    CausalEdge,
    CausalEdgeType,
    CausalGraphLayer,
    ProvenanceRecord,
    WorldEvent,
    WorldModel,
)

if TYPE_CHECKING:
    pass


_CAUSAL_BLAKE_PERSON = b"causal"
_CELL_SIZE_KM = math.sqrt(GRID_CELL_AREA_KILOMETERS_SQUARED)
# Phase 0–2 trigger event types: events emitted during geography /
# climate / biome generation (Phase 0–2) and the earliest demography
# events (Phase 3a.4). The causal-graph reachability check starts
# BFS from any event whose `t == 0` OR whose `type` is in this set.
_PHASE_0_TO_2_TRIGGER_TYPES: frozenset[str] = frozenset()


def _compute_algorithm_version(edges: tuple[CausalEdge, ...]) -> str:
    """blake2b hash of the causal-edge tuple. 16-char hex.

    Mirrors the 3a.5 / 3b.x / 4.1 pattern: mutations or re-ordering
    change the hash, allowing the trust boundary to detect silent
    corruption at `WorldModel.model_validate_json`.
    """
    digest = hashlib.blake2b(digest_size=8, person=_CAUSAL_BLAKE_PERSON)
    state = [edge.model_dump(mode="json") for edge in edges]
    encoded_state = json.dumps(
        state,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    digest.update(encoded_state)
    return digest.hexdigest()


def _settlement_position_lookup(
    world: WorldModel,
) -> dict[int, tuple[float, float]]:
    """Precompute `settlement_id -> (x_km, y_km)` lookup.

    Built once per `build_causal_graph` call so the per-pair
    distance computation is O(1) instead of O(N_settlements).
    """
    return {
        settlement.id: (
            float(settlement.x) * _CELL_SIZE_KM,
            float(settlement.y) * _CELL_SIZE_KM,
        )
        for settlement in world.settlements.settlements
    }


def _distance_km(
    pos_a: tuple[float, float] | None,
    pos_b: tuple[float, float] | None,
) -> float | None:
    """Compute distance between two precomputed positions in km.

    Returns `None` if either position is `None` (event without a
    settlement or settlement not found).
    """
    if pos_a is None or pos_b is None:
        return None
    dx = pos_a[0] - pos_b[0]
    dy = pos_a[1] - pos_b[1]
    return math.sqrt(dx * dx + dy * dy)


def _build_direct_edges(
    events: tuple[WorldEvent, ...],
) -> dict[tuple[str, str], CausalEdge]:
    """Build DIRECT edges from constructor-declared `event.causes`.

    Returns a dict keyed by `(source_id, target_id)` so callers can
    dedupe across the post-passes.
    """
    direct: dict[tuple[str, str], CausalEdge] = {}
    for event in events:
        for cause_id in event.causes:
            key = (cause_id, event.id)
            direct[key] = CausalEdge(
                source_id=cause_id,
                target_id=event.id,
                edge_type=CausalEdgeType.DIRECT,
                weight=1.0,
                reason="constructor-declared",
            )
    return direct


def _build_indirect_edges(
    events: tuple[WorldEvent, ...],
    existing: set[tuple[str, str, CausalEdgeType]],
    position_lookup: dict[int, tuple[float, float]],
) -> list[CausalEdge]:
    """Build INDIRECT edges from spatial-temporal proximity.

    Two flavors:

    1. Cross-step INDIRECT: from the first event at
       `(source_settlement, step_t)` to the first event at
       `(target_settlement, future_t)` when within
       `INDIRECT_PROXIMITY_KM`. Edge count bounded at
       `O(n_steps × window × proximate_settlements)`.
    2. Within-step INDIRECT: from the first event at
       `(settlement, step_t)` to every other event at the same
       `(settlement, step_t)`. Same-settlement same-step events
       share settlement state, so each is reachable from the
       proxy. Edge count bounded at
       `O(n_steps × settlements × events_per_step)`.

    Same-settlement chain edges (cross-step, same settlement) are
    emitted with `distance=0.0` and `weight=1.0`. Cross-settlement
    edges use the spatial-distance weight decay.

    Position lookup is precomputed once per call (O(N_settlements))
    so per-pair distance is O(1).
    """
    by_step_settlement: dict[int, dict[int, list[tuple[str, tuple[float, float]]]]] = {}
    for event in events:
        if event.location.settlement_id is None:
            continue
        pos = position_lookup.get(event.location.settlement_id)
        if pos is None:
            continue
        by_step_settlement.setdefault(event.t, {}).setdefault(
            event.location.settlement_id, []
        ).append((event.id, pos))

    steps = sorted(by_step_settlement.keys())
    indirect: list[CausalEdge] = []
    # Pass 1: cross-step INDIRECT (settlement-level proxy).
    for i, step_t in enumerate(steps):
        future_steps = steps[i + 1 : i + 1 + INDIRECT_TEMPORAL_WINDOW_STEPS]
        if not future_steps:
            break
        for source_settlement, source_events in by_step_settlement[step_t].items():
            source_event_id, pos_source = source_events[0]
            for future_t in future_steps:
                if future_t - step_t > INDIRECT_TEMPORAL_WINDOW_STEPS:
                    continue
                for target_settlement, target_events in by_step_settlement[future_t].items():
                    if target_settlement == source_settlement:
                        target_event_id, _ = target_events[0]
                        distance = 0.0
                    else:
                        pos_target = target_events[0][1]
                        dx = pos_source[0] - pos_target[0]
                        dy = pos_source[1] - pos_target[1]
                        distance = math.sqrt(dx * dx + dy * dy)
                        if distance > INDIRECT_PROXIMITY_KM:
                            continue
                        target_event_id = target_events[0][0]
                    key = (
                        source_event_id,
                        target_event_id,
                        CausalEdgeType.INDIRECT,
                    )
                    if key in existing:
                        continue
                    indirect.append(
                        CausalEdge(
                            source_id=source_event_id,
                            target_id=target_event_id,
                            edge_type=CausalEdgeType.INDIRECT,
                            weight=1.0 / (1.0 + distance) if distance > 0.0 else 1.0,
                            reason="spatial-temporal-proximity",
                        )
                    )
    # Pass 2: within-step INDIRECT (proxy -> other events at the
    # same settlement-step). Each edge has distance=0 and
    # weight=1.0; preserves the 5.4 per-event reachability gate.
    for by_settlement in by_step_settlement.values():
        for events_at_step in by_settlement.values():
            if len(events_at_step) < 2:
                continue
            proxy_id = events_at_step[0][0]
            for other_id, _ in events_at_step[1:]:
                key = (
                    proxy_id,
                    other_id,
                    CausalEdgeType.INDIRECT,
                )
                if key in existing:
                    continue
                indirect.append(
                    CausalEdge(
                        source_id=proxy_id,
                        target_id=other_id,
                        edge_type=CausalEdgeType.INDIRECT,
                        weight=1.0,
                        reason="within-step-settlement-state",
                    )
                )
    return indirect


def _ancestors_within_depth(
    event_id: str,
    direct_parents: dict[str, tuple[str, ...]],
    depth: int,
) -> set[str]:
    """Return the set of ancestor ids reachable from `event_id` via
    `direct_parents` within `depth` hops. Excludes `event_id`
    itself. Empty if the event has no causes.
    """
    ancestors: set[str] = set()
    queue: deque[tuple[str, int]] = deque([(event_id, 0)])
    while queue:
        current, current_depth = queue.popleft()
        if current_depth >= depth:
            continue
        for parent in direct_parents.get(current, ()):
            if parent == event_id or parent in ancestors:
                continue
            ancestors.add(parent)
            queue.append((parent, current_depth + 1))
    return ancestors


def _build_contingent_edges(
    events: tuple[WorldEvent, ...],
    direct_edges: dict[tuple[str, str], CausalEdge],
    existing: set[tuple[str, str, CausalEdgeType]],
) -> list[CausalEdge]:
    """Build CONTINGENT edges from shared-upstream detection.

    For every pair `(e_a, e_b)` where both have at least one direct
    ancestor in common (transitive ancestor within depth
    `CONTINGENT_DEPTH`), emit one CONTINGENT edge. Pure post-pass
    over the DIRECT edges — no new event semantics.

    Implementation: precompute per-event ancestor sets via BFS
    (O(n × depth × branching)), then index by ancestor_id and emit
    edges for all pairs sharing each ancestor. Total cost is
    O(n × ancestors_per_event + sum_over_ancestors_of_C(|descendants|,
    2)) — O(n) for typical fan-in distributions, vs. O(n²) for the
    naive pair-wise check.
    """
    direct_parents: dict[str, tuple[str, ...]] = {
        event.id: event.causes for event in events
    }
    # ancestor_id -> set of event ids that have this ancestor.
    ancestor_index: dict[str, set[str]] = {}
    for event in events:
        ancestors = _ancestors_within_depth(event.id, direct_parents, CONTINGENT_DEPTH)
        for ancestor_id in ancestors:
            ancestor_index.setdefault(ancestor_id, set()).add(event.id)

    contingent: list[CausalEdge] = []
    for descendant_set in ancestor_index.values():
        descendants = sorted(descendant_set)
        n = len(descendants)
        for i in range(n):
            for j in range(i + 1, n):
                key = (
                    descendants[i],
                    descendants[j],
                    CausalEdgeType.CONTINGENT,
                )
                if key in existing:
                    continue
                contingent.append(
                    CausalEdge(
                        source_id=descendants[i],
                        target_id=descendants[j],
                        edge_type=CausalEdgeType.CONTINGENT,
                        weight=1.0,
                        reason="shared-upstream",
                    )
                )
    return contingent


def _transitive_reduction(
    edges: list[CausalEdge],
) -> list[CausalEdge]:
    """Apply transitive reduction on the DIRECT + INDIRECT edge set.

    For each pair `(A, C)` where a path `A -> B -> C` exists via
    DIRECT or INDIRECT edges and the `(A, C)` edge is also DIRECT
    or INDIRECT, drop the `(A, C)` edge. CONTINGENT edges are
    preserved — they encode shared-upstream relationships that are
    not redundant with the directed causal chain.

    Determinism: edges are processed in lexicographic
    `(source_id, target_id, edge_type)` order.
    """
    # Index edges by type and build adjacency for reduction.
    sorted_edges = sorted(
        edges,
        key=lambda edge: (edge.source_id, edge.target_id, edge.edge_type.value),
    )
    reduction_targets: set[tuple[str, str]] = set()
    adjacency: dict[str, set[str]] = {}
    for edge in sorted_edges:
        if edge.edge_type == CausalEdgeType.CONTINGENT:
            continue
        adjacency.setdefault(edge.source_id, set()).add(edge.target_id)
    # For each (A, B) DIRECT/INDIRECT edge, find all C such that
    # (A, C) is also DIRECT/INDIRECT and B -> C is reachable.
    for edge in sorted_edges:
        if edge.edge_type == CausalEdgeType.CONTINGENT:
            continue
        a, b = edge.source_id, edge.target_id
        # BFS from b up to depth 2 (we only care about 1-step
        # reductions for the chain pattern).
        next_ids = adjacency.get(b, set())
        for c in next_ids:
            if (a, c) != (a, b):
                reduction_targets.add((a, c))
    # Drop edges in reduction_targets if they're DIRECT or INDIRECT.
    reduced: list[CausalEdge] = []
    for edge in sorted_edges:
        if edge.edge_type == CausalEdgeType.CONTINGENT:
            reduced.append(edge)
            continue
        if (edge.source_id, edge.target_id) in reduction_targets:
            # Verify the redundant edge was not the only path: skip
            # only if (source_id, target_id) is in reduction_targets
            # AND there exists a 2-step intermediate. The reduction
            # set already encodes that check.
            continue
        reduced.append(edge)
    return reduced


def build_causal_graph(world: WorldModel) -> CausalGraphLayer:
    """Build the causal graph layer from the world's events.

    Combines `world.events.events` (3a.5 / 3b.x — which already
    includes 4.1 POLITY_FOUNDED via the EventLog merge fix) with
    `world.polities.events` (4.1 — read for completeness). The
    union is deduplicated by `event.id` so polity events that
    appear in both streams are only counted once.
    """
    seen_ids: set[str] = set()
    events_list: list[WorldEvent] = []
    for event in world.events.events + world.polities.events:
        if event.id in seen_ids:
            continue
        seen_ids.add(event.id)
        events_list.append(event)
    events = tuple(events_list)

    direct_map = _build_direct_edges(events)
    direct_edges = list(direct_map.values())

    existing: set[tuple[str, str, CausalEdgeType]] = {
        (edge.source_id, edge.target_id, edge.edge_type)
        for edge in direct_edges
    }

    position_lookup = _settlement_position_lookup(world)
    indirect_edges = _build_indirect_edges(events, existing, position_lookup)
    for edge in indirect_edges:
        existing.add((edge.source_id, edge.target_id, edge.edge_type))

    contingent_edges = _build_contingent_edges(events, direct_map, existing)

    all_edges = direct_edges + indirect_edges + contingent_edges
    reduced = _transitive_reduction(all_edges)

    # Deterministic sort: lexicographic by (source_id, target_id,
    # edge_type). Pinned in module docstring.
    reduced.sort(
        key=lambda edge: (edge.source_id, edge.target_id, edge.edge_type.value)
    )
    edges_tuple = tuple(reduced)
    algorithm_version = _compute_algorithm_version(edges_tuple)
    return CausalGraphLayer(
        edges=edges_tuple,
        algorithm_version=algorithm_version,
    )


def _phase_0_to_2_seed_events(
    events: tuple[WorldEvent, ...],
) -> set[str]:
    """Return event ids that are seeds for the reachability check.

    Phase 0–2 trigger events are those at `t == 0` (initial world
    state) plus any events emitted by Phase 0–2 generators. v1
    simplifies: every event at `t == 0` is a seed because the
    initial world state has no upstream events.
    """
    return {event.id for event in events if event.t == 0}


def _reachable_from_seeds(
    seeds: set[str],
    adjacency: dict[str, set[str]],
) -> set[str]:
    """BFS over the directed edge set from `seeds`.

    Returns the set of event ids reachable from any seed via any
    number of edge traversals.
    """
    visited: set[str] = set(seeds)
    queue: deque[str] = deque(seeds)
    while queue:
        current = queue.popleft()
        for next_id in adjacency.get(current, ()):
            if next_id not in visited:
                visited.add(next_id)
                queue.append(next_id)
    return visited


def validate_causal_graph_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 5.1 causal-graph-layer invariants.

    Order:
    1. `_validate_algorithm_version` (FIRST; matches 3a.5 / 3b.x /
       4.1 pattern).
    2. `_validate_edge_integrity` (per-`CausalEdge` field
       integrity).
    3. `_validate_referential_integrity` (every source_id /
       target_id is in the union of events).
    4. `_validate_no_duplicate_edges` (no two edges share
       `(source_id, target_id, edge_type)`).
    5. `_validate_reachability` (every event in the union is
       reachable from a Phase 0–2 seed).
    """
    violations: list[InvariantViolation] = []
    layer = world.causal_graph
    all_event_ids: set[str] = {
        event.id for event in world.events.events
    } | {event.id for event in world.polities.events}

    expected_version = _compute_algorithm_version(layer.edges)
    if layer.algorithm_version != expected_version:
        violations.append(
            _violation(
                "causal-graph-algorithm-version-mismatch",
                "world.causal_graph.algorithm_version",
                (
                    f"causal graph algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; layer was "
                    f"mutated or re-ordered outside the generator"
                ),
            )
        )

    seen: set[tuple[str, str, CausalEdgeType]] = set()
    adjacency: dict[str, set[str]] = {}
    for index, edge in enumerate(layer.edges):
        # Field-integrity: edge_type enum membership, weight range,
        # non-empty reason. The CausalEdge StrictModel already
        # enforces these at construction; the validator re-checks
        # at the trust boundary.
        if not isinstance(edge.edge_type, CausalEdgeType):
            violations.append(
                _violation(
                    "causal-graph-invalid-edge-type",
                    f"world.causal_graph.edges.{index}.edge_type",
                    (
                        f"edge {edge.source_id}->{edge.target_id} has "
                        f"invalid edge_type {edge.edge_type!r}"
                    ),
                )
            )
        if edge.weight < 0.0:
            violations.append(
                _violation(
                    "causal-graph-negative-weight",
                    f"world.causal_graph.edges.{index}.weight",
                    (
                        f"edge {edge.source_id}->{edge.target_id} has "
                        f"negative weight {edge.weight}"
                    ),
                )
            )
        if not edge.reason:
            violations.append(
                _violation(
                    "causal-graph-empty-reason",
                    f"world.causal_graph.edges.{index}.reason",
                    (
                        f"edge {edge.source_id}->{edge.target_id} has "
                        f"empty reason"
                    ),
                )
            )
        if edge.source_id == edge.target_id:
            violations.append(
                _violation(
                    "causal-graph-self-loop",
                    f"world.causal_graph.edges.{index}",
                    (
                        f"self-loop edge {edge.source_id}->"
                        f"{edge.target_id}"
                    ),
                )
            )
        # Referential integrity: source_id and target_id must exist
        # in the union of events.
        if edge.source_id not in all_event_ids:
            violations.append(
                _violation(
                    "causal-graph-missing-source",
                    f"world.causal_graph.edges.{index}.source_id",
                    (
                        f"edge source {edge.source_id!r} not in "
                        f"world.events.events or world.polities.events"
                    ),
                )
            )
        if edge.target_id not in all_event_ids:
            violations.append(
                _violation(
                    "causal-graph-missing-target",
                    f"world.causal_graph.edges.{index}.target_id",
                    (
                        f"edge target {edge.target_id!r} not in "
                        f"world.events.events or world.polities.events"
                    ),
                )
            )
        # Duplicate detection.
        key = (edge.source_id, edge.target_id, edge.edge_type)
        if key in seen:
            violations.append(
                _violation(
                    "causal-graph-duplicate-edge",
                    f"world.causal_graph.edges.{index}",
                    (
                        f"duplicate edge {edge.source_id}->"
                        f"{edge.target_id} of type "
                        f"{edge.edge_type.value}"
                    ),
                )
            )
        seen.add(key)
        adjacency.setdefault(edge.source_id, set()).add(edge.target_id)

    # Reachability: every event must be reachable from a seed.
    seeds = _phase_0_to_2_seed_events(
        world.events.events + world.polities.events
    )
    reachable = _reachable_from_seeds(seeds, adjacency)
    unreachable = all_event_ids - reachable
    for event_id in sorted(unreachable):
        violations.append(
            _violation(
                "causal-graph-unreachable-event",
                "world.causal_graph",
                (
                    f"event {event_id} is not reachable from any "
                    f"Phase 0–2 seed via the causal graph"
                ),
            )
        )
    return violations


def causal_graph_provenance() -> ProvenanceRecord:
    """Provenance record describing the causal-graph builder."""
    return ProvenanceRecord(
        output_path="causal_graph",
        process=(
            "direct-from-causes + indirect-from-spatial-temporal-"
            "proximity + contingent-from-shared-upstream + "
            "transitive-reduction + blake2b-algorithm-version"
        ),
        input_paths=(
            "events.events",
            "polities.events",
            "settlements.settlements",
        ),
        algorithm_version=CAUSAL_GRAPH_ALGORITHM_VERSION,
    )