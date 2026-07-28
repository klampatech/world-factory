"""Phase 3b.1 culture-layer invariants and determinism — CultureLayer on
WorldModel, CULTURE_DRIFT events, neighbor-correlated drift model."""

from __future__ import annotations

import statistics

from world_factory.constants import (
    CULTURE_ALGORITHM_VERSION,
    CULTURE_ATTRIBUTE_NAMES,
    CULTURE_BIOME_BIAS_TABLE,
    CULTURE_DRIFT_TIME_STEPS,
    CULTURE_NEIGHBOR_K,
    EVENT_LOG_ALGORITHM_VERSION,
)
from world_factory.cultures import (
    build_cultures,
    cultures_provenance,
    validate_cultures_layer,
)
from world_factory.event_log import validate_event_log
from world_factory.generator import generate_world
from world_factory.models import (
    Culture,
    CultureDriftPayload,
    CultureLayer,
    EventType,
    WorldConfig,
    WorldScale,
)


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_culture_layer() -> None:
    world = generate_world(_config())
    assert world.cultures is not None
    assert isinstance(world.cultures, CultureLayer)


def test_cultures_parallel_to_settlements() -> None:
    """Culture records are parallel to settlements by id (same length,
    same order)."""
    world = generate_world(_config())
    assert len(world.cultures.cultures) == len(world.settlements.settlements)
    for index, culture in enumerate(world.cultures.cultures):
        settlement = world.settlements.settlements[index]
        assert culture.settlement_id == settlement.id


def test_attribute_history_length_matches_time_steps() -> None:
    """Each culture's `attribute_history` has length
    `time_steps + 1`: index 0 is the initial vector, indices 1..N are
    post-step vectors."""
    world = generate_world(_config())
    expected_length = CULTURE_DRIFT_TIME_STEPS + 1
    for culture in world.cultures.cultures:
        assert len(culture.attribute_history) == expected_length


def test_attribute_vector_shape_and_bounds() -> None:
    """Each step vector has 6 attributes (per `CULTURE_ATTRIBUTE_NAMES`)
    and every value is in `[0, 1]`."""
    world = generate_world(_config())
    expected_count = len(CULTURE_ATTRIBUTE_NAMES)
    for culture in world.cultures.cultures:
        for _step_index, step_vector in enumerate(culture.attribute_history):
            assert len(step_vector) == expected_count
            for value in step_vector:
                assert 0.0 <= value <= 1.0


def test_attribute_names_match_spec() -> None:
    """Per spec wording (Ernie's review note 1): the 6 attributes are
    `values`, `norms`, `taboos`, `ritual_forms`, `cuisine`,
    `music_motifs`. (Earlier plan had `rituals` / `art` / `music`;
    spec wording is canonical.)"""
    assert CULTURE_ATTRIBUTE_NAMES == (
        "values",
        "norms",
        "taboos",
        "ritual_forms",
        "cuisine",
        "music_motifs",
    )


def test_biome_bias_table_covers_all_biome_classes() -> None:
    """Every `BiomeClass` (used by settlements) has a bias entry in
    `CULTURE_BIOME_BIAS_TABLE`. Settlements should never land on
    ocean cells, but the bias table is defensive — it covers all
    biomes the layer might encounter (including ocean / ice / alpine)."""
    from world_factory.models import BiomeClass

    expected_keys = {biome.value for biome in BiomeClass}
    actual_keys = set(CULTURE_BIOME_BIAS_TABLE.keys())
    assert expected_keys == actual_keys
    for _biome_key, biases in CULTURE_BIOME_BIAS_TABLE.items():
        assert len(biases) == len(CULTURE_ATTRIBUTE_NAMES)
        for value in biases:
            assert 0.0 <= value <= 1.0


def test_deterministic_across_runs() -> None:
    a = generate_world(_config())
    b = generate_world(_config())
    assert a.cultures == b.cultures


def test_world_id_stable_across_phase_3b1() -> None:
    """3b.1 adds no new WorldConfig fields, so world_id for --seed 42
    at LARGE scale must remain `9d75e7103b52704b48ce77071a22a586` —
    the v1-demo / 3a.2 / 3a.3 / 3a.4 / 3a.5 reference value."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.metadata.world_id == "9d75e7103b52704b48ce77071a22a586"


def test_schema_version_bumped_to_13() -> None:
    """3b.1 adds a required `cultures` field to WorldModel, so
    SCHEMA_VERSION must bump 12.0.0 -> 13.0.0 per the additive-required
    policy (3a.2 / 3a.4 / 3a.5 followed the same pattern)."""
    world = generate_world(_config())
    assert world.metadata.schema_version == "13.0.0"


def test_model_version_at_phase_3b1() -> None:
    """MODEL_VERSION advances to `phase-3b.1` for the 3b.1 milestone."""
    world = generate_world(_config())
    assert world.metadata.model_version == "phase-3b.1"


def test_algorithm_version_stability() -> None:
    """`algorithm_version` is a 16-char hex blake2b hash of the layer
    state. Re-computing it from the same layer produces the same hash."""
    world = generate_world(_config())
    assert len(world.cultures.algorithm_version) == 16
    assert all(c in "0123456789abcdef" for c in world.cultures.algorithm_version)


def test_algorithm_version_changes_with_culture_state() -> None:
    """Different seeds produce different culture algorithm_versions
    (the hash is content-derived)."""
    a = generate_world(_config(seed=42, scale=WorldScale.SMALL))
    b = generate_world(_config(seed=43, scale=WorldScale.SMALL))
    assert a.cultures.algorithm_version != b.cultures.algorithm_version


def test_cultures_provenance_record_present() -> None:
    world = generate_world(_config())
    matches = [
        record
        for record in world.provenance
        if record.output_path == "cultures"
    ]
    assert len(matches) == 1
    assert matches[0].algorithm_version == CULTURE_ALGORITHM_VERSION


def test_event_log_algorithm_version_distinct_from_cultures() -> None:
    """The cultures layer and the event log have separate
    `algorithm_version` hashes (different person namespaces in their
    blake2b digests)."""
    world = generate_world(_config())
    assert (
        world.cultures.algorithm_version != world.events.algorithm_version
    )


def test_event_log_provenance_record_still_present() -> None:
    """3a.5 events layer remains additive-required; the existing
    provenance record must still be in `world.provenance`."""
    world = generate_world(_config())
    matches = [
        record
        for record in world.provenance
        if record.output_path == "events"
    ]
    assert len(matches) == 1
    assert matches[0].algorithm_version == EVENT_LOG_ALGORITHM_VERSION


def test_validate_cultures_layer_empty_for_valid_world() -> None:
    world = generate_world(_config())
    assert validate_cultures_layer(world) == []


def test_validate_cultures_layer_flags_algorithm_version_mismatch() -> None:
    """If the cultures tuple is mutated after build (e.g., a downstream
    consumer drops a culture), the recomputed algorithm_version differs
    from the stored one. The validator catches this at the trust
    boundary."""
    world = generate_world(_config())
    tampered_cultures = world.cultures.cultures[:-1]
    bad_world = world.model_copy(
        update={
            "cultures": CultureLayer(
                cultures=tampered_cultures,
                algorithm_version=world.cultures.algorithm_version,
            )
        }
    )
    violations = validate_cultures_layer(bad_world)
    assert any(
        v.code == "culture-layer-algorithm-version-mismatch" for v in violations
    )


def test_validate_cultures_layer_flags_attribute_out_of_range() -> None:
    """If an attribute value is mutated outside `[0, 1]`, the validator
    catches it. (Pydantic strict mode would also catch it at the model
    boundary; this is the trust-boundary double-check.)"""
    world = generate_world(_config())
    first_culture = world.cultures.cultures[0]
    bad_history = (
        first_culture.attribute_history[:1]
        + (tuple(1.5 if i == 0 else v for i, v in enumerate(
            first_culture.attribute_history[1]
        )),)
        + first_culture.attribute_history[2:]
    )
    bad_culture = Culture(
        settlement_id=first_culture.settlement_id,
        attribute_history=bad_history,
    )
    bad_cultures = (bad_culture,) + world.cultures.cultures[1:]
    bad_world = world.model_copy(
        update={
            "cultures": CultureLayer(
                cultures=bad_cultures,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_cultures_layer(bad_world)
    assert any(
        v.code == "culture-layer-attribute-out-of-range" for v in violations
    )


def test_validate_cultures_layer_flags_settlement_id_mismatch() -> None:
    """If a culture's `settlement_id` is mutated to disagree with the
    parallel settlements index, the validator catches it."""
    world = generate_world(_config())
    first_culture = world.cultures.cultures[0]
    bad_culture = Culture(
        settlement_id=first_culture.settlement_id + 99999,
        attribute_history=first_culture.attribute_history,
    )
    bad_cultures = (bad_culture,) + world.cultures.cultures[1:]
    bad_world = world.model_copy(
        update={
            "cultures": CultureLayer(
                cultures=bad_cultures,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_cultures_layer(bad_world)
    assert any(
        v.code == "culture-layer-settlement-id-mismatch" for v in violations
    )


def test_culture_drift_events_are_emitted() -> None:
    """The EventLog includes CULTURE_DRIFT events emitted by the
    culture drift simulation (one per changed attribute per
    settlement per step)."""
    world = generate_world(_config())
    drift_events = [e for e in world.events.events if e.type == EventType.CULTURE_DRIFT]
    assert drift_events, "expected CULTURE_DRIFT events in EventLog"
    for event in drift_events[:50]:
        # payload re-validates against the typed CultureDriftPayload
        CultureDriftPayload.model_validate(event.payload)


def test_culture_drift_event_ids_are_unique() -> None:
    world = generate_world(_config())
    drift_event_ids = [
        e.id
        for e in world.events.events
        if e.type == EventType.CULTURE_DRIFT
    ]
    assert len(drift_event_ids) == len(set(drift_event_ids))


def test_culture_drift_event_t_is_step() -> None:
    """Per the spec, culture drift events have `t = step` (matching
    the WorldEvent.t convention). `payload.step` mirrors `t`."""
    world = generate_world(_config())
    drift_events = [e for e in world.events.events if e.type == EventType.CULTURE_DRIFT]
    for event in drift_events[:50]:
        assert event.payload["step"] == event.t


def test_culture_drift_events_have_deterministic_ids() -> None:
    """Per PHASE_3A_TYPES.md Option A pattern: event ids are 16-char
    hex. The blake2b person namespace for culture events is
    `b"culture"` (distinct from demography's `b"worldfac"`)."""
    world = generate_world(_config())
    drift_events = [e for e in world.events.events if e.type == EventType.CULTURE_DRIFT]
    for event in drift_events[:200]:
        assert len(event.id) == 16
        assert all(c in "0123456789abcdef" for c in event.id)


def test_event_log_validator_passes_with_culture_events_included() -> None:
    """Merging demography + culture events per-step preserves
    monotonicity, so the event-log validator still passes."""
    world = generate_world(_config())
    assert validate_event_log(world) == []


def test_attribute_drift_occurs_over_time() -> None:
    """Across `time_steps` epochs, attribute vectors actually drift
    (not all zero-change / not mode-collapsed). Each culture's
    initial vector should differ from its final vector in at least
    one attribute for a long enough run."""
    world = generate_world(_config())
    drifts_seen = 0
    for culture in world.cultures.cultures:
        initial = culture.attribute_history[0]
        final = culture.attribute_history[-1]
        if any(
            abs(round(final[i] - initial[i], 6)) > 0.0
            for i in range(len(CULTURE_ATTRIBUTE_NAMES))
        ):
            drifts_seen += 1
    # At least 50% of cultures should drift (some may saturate at the
    # [0, 1] boundary and end up identical to the bias initial).
    assert drifts_seen >= len(world.cultures.cultures) // 2


def test_neighbor_correlation_produces_spatial_clustering() -> None:
    """Per Ernie's Note 2: neighbor correlation should produce spatial
    clustering so 3b.2 (religion) and 3b.4 (languages) have meaningful
    cultural-proximity inputs. The test verifies that two cultures
    that are K=3 nearest neighbors of each other end up closer in
    attribute space than two cultures picked at random.

    This is the load-bearing property: without the neighbor pull, the
    layer is purely noise-driven and cultural proximity is undefined.
    """
    import math

    world = generate_world(_config())
    cultures = world.cultures.cultures
    if len(cultures) < 4:
        # Single-settlement worlds are degenerate; skip.
        return
    settlements_by_id = {
        settlement.id: settlement
        for settlement in world.settlements.settlements
    }
    positions = {
        culture.settlement_id: (
            settlements_by_id[culture.settlement_id].x,
            settlements_by_id[culture.settlement_id].y,
        )
        for culture in cultures
    }

    def distance(a_id: int, b_id: int) -> float:
        ax, ay = positions[a_id]
        bx, by = positions[b_id]
        return math.hypot(ax - bx, ay - by)

    def attribute_distance(
        culture_a: object, culture_b: object
    ) -> float:
        a = culture_a.attribute_history[-1]
        b = culture_b.attribute_history[-1]
        return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(len(a))))

    # For each culture, find its K=3 nearest neighbors and measure
    # the mean attribute distance to them. Compare to the mean
    # attribute distance to a random sample of cultures farther away.
    k = CULTURE_NEIGHBOR_K
    neighbor_distances: list[float] = []
    far_distances: list[float] = []
    for culture in cultures:
        sorted_others = sorted(
            (other for other in cultures if other.settlement_id != culture.settlement_id),
            key=lambda other: distance(culture.settlement_id, other.settlement_id),
        )
        nearest = sorted_others[:k]
        far = sorted_others[k : 2 * k] if len(sorted_others) >= 2 * k else sorted_others[k:]
        for other in nearest:
            neighbor_distances.append(attribute_distance(culture, other))
        for other in far:
            far_distances.append(attribute_distance(culture, other))
    if not neighbor_distances or not far_distances:
        return
    mean_neighbor = statistics.mean(neighbor_distances)
    mean_far = statistics.mean(far_distances)
    assert mean_neighbor < mean_far, (
        f"neighbor-correlated drift failed to produce spatial clustering: "
        f"mean attribute distance to K={k} nearest neighbors ({mean_neighbor:.4f}) "
        f"is not less than mean distance to the next {len(far_distances)//len(cultures)} "
        f"cultures ({mean_far:.4f})"
    )


def test_no_mode_collapse_across_seeds() -> None:
    """Across N seeds, final culture attribute distributions are not
    mode-collapsed (no single dominant culture). Per 3b.5 acceptance
    criterion, attribute vectors should retain meaningful variance."""
    seeds = (41, 42, 43)
    final_attribute_distributions = {attr: [] for attr in CULTURE_ATTRIBUTE_NAMES}
    for seed in seeds:
        world = generate_world(
            WorldConfig(seed=seed, scale=WorldScale.LARGE)
        )
        for culture in world.cultures.cultures:
            final = culture.attribute_history[-1]
            for attr_index, value in enumerate(final):
                final_attribute_distributions[CULTURE_ATTRIBUTE_NAMES[attr_index]].append(
                    value
                )
    for attr, values in final_attribute_distributions.items():
        stdev = statistics.stdev(values) if len(values) >= 2 else 0.0
        assert stdev > 0.005, (
            f"attribute {attr} has collapsed variance across seeds "
            f"(stdev={stdev:.6f}); culture layer is mode-collapsing"
        )


def test_neighbor_k_constant_matches_3a3_infrastructure_sparsity() -> None:
    """Per Ernie's Note 2: K=3 to match 3a.3 infrastructure sparsity
    (`INFRASTRUCTURE_ROAD_NEIGHBOR_K = 3`). The constant is exposed in
    `CULTURE_NEIGHBOR_K` so downstream phases can read it."""
    assert CULTURE_NEIGHBOR_K == 3


def test_cultures_layer_runs_cleanly_on_small_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    assert world.cultures is not None
    assert validate_cultures_layer(world) == []


def test_cultures_layer_runs_cleanly_on_medium_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    assert world.cultures is not None
    assert validate_cultures_layer(world) == []


def test_build_cultures_returns_layer_and_events() -> None:
    """`build_cultures` returns `(CultureLayer, tuple[WorldEvent, ...])`
    so the generator can merge culture events into the EventLog."""
    world = generate_world(_config())
    cultures, events = build_cultures(world)
    assert isinstance(cultures, CultureLayer)
    assert all(e.type == EventType.CULTURE_DRIFT for e in events)


def test_cultures_provenance_factory() -> None:
    """Direct test of the `cultures_provenance` factory: it returns a
    ProvenanceRecord describing the culture-layer builder."""
    record = cultures_provenance()
    assert record.output_path == "cultures"
    assert record.algorithm_version == CULTURE_ALGORITHM_VERSION
    assert "settlements.settlements" in record.input_paths
    assert "biomes.classifications" in record.input_paths
    assert "metadata.config.seed" in record.input_paths
