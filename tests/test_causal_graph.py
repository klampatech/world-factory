"""Phase 5.1 causal-graph-layer invariants and acceptance gates.

CausalGraphLayer on WorldModel (additive top-level field per
plan-ack Q2), three edge types DIRECT + INDIRECT + CONTINGENT
(plan-ack Q1), algorithm_version blake2b hash with the
`b"causal"` person namespace, transitive-reduction post-pass
(spec R4), and the 5.4 acceptance gate "every event reachable from
a Phase 0-2 trigger".
"""

from __future__ import annotations

from world_factory.causal_graph import (
    _compute_algorithm_version,
    build_causal_graph,
    causal_graph_provenance,
    validate_causal_graph_layer,
)
from world_factory.constants import (
    CAUSAL_GRAPH_ALGORITHM_VERSION,
    CONTINGENT_DEPTH,
    INDIRECT_PROXIMITY_KM,
    INDIRECT_TEMPORAL_WINDOW_STEPS,
    MODEL_VERSION,
    SCHEMA_VERSION,
)
from world_factory.generator import generate_world
from world_factory.models import (
    CausalEdge,
    CausalEdgeType,
    CausalGraphLayer,
    WorldConfig,
    WorldScale,
)
from world_factory.validation import validate_world


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_causal_graph_layer() -> None:
    """`WorldModel.causal_graph` is a `CausalGraphLayer` aggregate
    (additive top-level field per plan-ack Q2)."""
    world = generate_world(_config())
    assert world.causal_graph is not None
    assert isinstance(world.causal_graph, CausalGraphLayer)


def test_causal_graph_algorithm_version_pinned() -> None:
    """`algorithm_version` matches the `b"causal"` blake2b hash
    of the edge tuple; pin is exposed via
    `CAUSAL_GRAPH_ALGORITHM_VERSION`."""
    world = generate_world(_config())
    expected = _compute_algorithm_version(world.causal_graph.edges)
    assert world.causal_graph.algorithm_version == expected, (
        f"causal_graph.algorithm_version "
        f"{world.causal_graph.algorithm_version!r} does not match "
        f"recomputed {expected!r}"
    )


def test_schema_version_bumped_to_18() -> None:
    """5.1 additive-required per 3a.2 policy: three new top-level
    fields on `WorldModel` (causal_graph, historiography;
    counterfactual is operation). `SCHEMA_VERSION` advances
    17.0.0 -> 18.0.0; `MODEL_VERSION` advances phase-4 -> phase-5.
    """
    assert SCHEMA_VERSION == "18.0.0", (
        f"SCHEMA_VERSION is {SCHEMA_VERSION!r}, expected '18.0.0'"
    )
    assert MODEL_VERSION == "phase-5", (
        f"MODEL_VERSION is {MODEL_VERSION!r}, expected 'phase-5'"
    )


def test_causal_graph_provenance_recorded() -> None:
    """Provenance record describes the causal-graph builder."""
    record = causal_graph_provenance()
    assert record.output_path == "causal_graph"
    assert record.algorithm_version == CAUSAL_GRAPH_ALGORITHM_VERSION


def test_causal_graph_edges_sorted_lexicographically() -> None:
    """Edges are sorted lexicographically by
    `(source_id, target_id, edge_type)` per the module docstring
    pin. Deterministic — same world + same seed produces the same
    order."""
    world = generate_world(_config())
    edges = world.causal_graph.edges
    for i in range(len(edges) - 1):
        cur = (edges[i].source_id, edges[i].target_id, edges[i].edge_type.value)
        nxt = (edges[i + 1].source_id, edges[i + 1].target_id, edges[i + 1].edge_type.value)
        assert cur < nxt, (
            f"edges not sorted at index {i}: {cur} >= {nxt}"
        )


def test_causal_graph_no_duplicate_edges() -> None:
    """No two edges share `(source_id, target_id, edge_type)`.
    Validator's `_validate_no_duplicate_edges` check passes."""
    world = generate_world(_config())
    seen: set[tuple[str, str, CausalEdgeType]] = set()
    for edge in world.causal_graph.edges:
        key = (edge.source_id, edge.target_id, edge.edge_type)
        assert key not in seen, (
            f"duplicate edge {edge.source_id} -> {edge.target_id} "
            f"of type {edge.edge_type.value}"
        )
        seen.add(key)


def test_causal_graph_edges_have_nonempty_reason() -> None:
    """Every edge has a non-empty `reason` string per the validator's
    `_validate_edge_integrity` check."""
    world = generate_world(_config())
    for index, edge in enumerate(world.causal_graph.edges):
        assert edge.reason, (
            f"edge {index} ({edge.source_id} -> {edge.target_id}) "
            f"has empty reason"
        )


def test_causal_graph_referential_integrity() -> None:
    """Every edge's `source_id` and `target_id` exist in the
    union of `world.events.events` and `world.polities.events`.
    Validator's `_validate_referential_integrity` check passes."""
    world = generate_world(_config())
    all_event_ids = {event.id for event in world.events.events} | {
        event.id for event in world.polities.events
    }
    for index, edge in enumerate(world.causal_graph.edges):
        assert edge.source_id in all_event_ids, (
            f"edge {index} source_id {edge.source_id!r} not in events"
        )
        assert edge.target_id in all_event_ids, (
            f"edge {index} target_id {edge.target_id!r} not in events"
        )


def test_5_4_acceptance_causal_reachability() -> None:
    """5.4 acceptance gate: every event in `world.events.events ∪
    world.polities.events` is reachable from a Phase 0-2 trigger
    via the `CausalGraphLayer.edges` graph. BFS from seed events,
    assert `len(reachable) == n_events`."""
    world = generate_world(_config())
    violations = validate_causal_graph_layer(world)
    reachability_violations = [
        v for v in violations if v.code == "causal-graph-unreachable-event"
    ]
    assert not reachability_violations, (
        f"{len(reachability_violations)} events unreachable; "
        f"first: {reachability_violations[0].message if reachability_violations else None}"
    )


def test_causal_graph_validator_clean_for_fresh_world() -> None:
    """`validate_causal_graph_layer` returns no violations for a
    freshly generated canonical world (small scale)."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    violations = validate_causal_graph_layer(world)
    codes = [v.code for v in violations]
    assert not violations, f"unexpected violations: {codes}"


def test_validate_world_includes_causal_graph() -> None:
    """`validate_world` calls `validate_causal_graph_layer` so the
    causal-graph invariants surface in the cross-layer report."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    report = validate_world(world)
    # `validate_world` may surface other layers' violations; we
    # just assert the causal-graph validator is wired by checking
    # that running it standalone matches the subset of the report.
    standalone = validate_causal_graph_layer(world)
    standalone_codes = sorted(v.code for v in standalone)
    report_codes = sorted(
        v.code
        for v in report.violations
        if v.code.startswith("causal-graph-")
    )
    assert standalone_codes == report_codes, (
        f"standalone {standalone_codes} != report {report_codes}"
    )


def test_causal_graph_edge_weights_nonnegative() -> None:
    """All edge weights are `>= 0.0` per `CausalEdge.weight: float
    = Field(ge=0.0)`."""
    world = generate_world(_config())
    for index, edge in enumerate(world.causal_graph.edges):
        assert edge.weight >= 0.0, (
            f"edge {index} has negative weight {edge.weight}"
        )


def test_causal_graph_indirect_within_window() -> None:
    """INDIRECT edges target events at most
    `INDIRECT_TEMPORAL_WINDOW_STEPS` steps after the source.

    Within-step INDIRECT edges (`reason="within-step-settlement-state"`)
    have `delta == 0` and represent shared settlement state at the
    same step. Cross-step INDIRECT edges have
    `1 <= delta <= INDIRECT_TEMPORAL_WINDOW_STEPS`."""
    world = generate_world(_config(scale=WorldScale.SMALL))
    source_t: dict[str, int] = {}
    target_t: dict[str, int] = {}
    for event in world.events.events + world.polities.events:
        source_t[event.id] = event.t
        target_t[event.id] = event.t
    for edge in world.causal_graph.edges:
        if edge.edge_type != CausalEdgeType.INDIRECT:
            continue
        delta = target_t[edge.target_id] - source_t[edge.source_id]
        if edge.reason == "within-step-settlement-state":
            assert delta == 0, (
                f"within-step INDIRECT edge has delta {delta}, "
                f"expected 0"
            )
        else:
            assert 0 < delta <= INDIRECT_TEMPORAL_WINDOW_STEPS, (
                f"INDIRECT edge has temporal delta {delta}, "
                f"expected 0 < delta <= {INDIRECT_TEMPORAL_WINDOW_STEPS}"
            )


def test_causal_graph_constants_match_plan_ack() -> None:
    """Constants pinned at plan-ack values per the module
    docstring."""
    assert INDIRECT_PROXIMITY_KM == 50.0, (
        f"INDIRECT_PROXIMITY_KM is {INDIRECT_PROXIMITY_KM}, expected 50.0"
    )
    assert INDIRECT_TEMPORAL_WINDOW_STEPS == 5, (
        f"INDIRECT_TEMPORAL_WINDOW_STEPS is "
        f"{INDIRECT_TEMPORAL_WINDOW_STEPS}, expected 5"
    )
    assert CONTINGENT_DEPTH == 2, (
        f"CONTINGENT_DEPTH is {CONTINGENT_DEPTH}, expected 2"
    )


def test_causal_graph_determinism_byte_equal() -> None:
    """Two runs with the same seed produce byte-identical
    `CausalGraphLayer`."""
    config = _config()
    world_a = generate_world(config)
    world_b = generate_world(config)
    assert (
        world_a.causal_graph.model_dump()
        == world_b.causal_graph.model_dump()
    ), "same seed produced different causal graph layers"


def test_causal_graph_handles_empty_union() -> None:
    """Build with no events yields an empty layer with stable
    algorithm_version."""
    from world_factory.models import WorldModel
    # Build a world via generate_world, then strip events before
    # calling build_causal_graph — verifies the empty path
    # doesn't crash and the algorithm_version is well-defined.
    world = generate_world(_config(scale=WorldScale.SMALL))
    empty_world = world.model_copy(
        update={
            "events": world.events.model_copy(
                update={"events": (), "algorithm_version": ""}
            ),
            "polities": world.polities.model_copy(
                update={"events": (), "algorithm_version": ""}
            ),
        }
    )
    layer = build_causal_graph(empty_world)
    assert layer.edges == ()
    assert layer.algorithm_version == _compute_algorithm_version(layer.edges)


def test_causal_edge_type_values_pinned() -> None:
    """Edge-type values are pinned at `direct` / `indirect` /
    `contingent` per the module docstring."""
    assert CausalEdgeType.DIRECT.value == "direct"
    assert CausalEdgeType.INDIRECT.value == "indirect"
    assert CausalEdgeType.CONTINGENT.value == "contingent"


def test_causal_edge_construction() -> None:
    """`CausalEdge` rejects negative weights (StrictModel field
    constraint) and accepts the three edge types."""
    import pydantic
    edge = CausalEdge(
        source_id="01f2c0f1abcdef01",
        target_id="b0681277abcdef01",
        edge_type=CausalEdgeType.DIRECT,
        weight=1.0,
        reason="test",
    )
    assert edge.edge_type == CausalEdgeType.DIRECT
    try:
        CausalEdge(
            source_id="01f2c0f1abcdef01",
            target_id="b0681277abcdef01",
            edge_type=CausalEdgeType.DIRECT,
            weight=-1.0,
            reason="test",
        )
    except pydantic.ValidationError:
        pass
    else:
        raise AssertionError("expected ValidationError for negative weight")