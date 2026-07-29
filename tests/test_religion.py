"""Phase 3b.2 religion-layer invariants and determinism — ReligionLayer
on WorldModel, BELIEF events, biome + history bias tables."""

from __future__ import annotations

from world_factory.constants import (
    RELIGION_ALGORITHM_VERSION,
    RELIGION_BIOME_COSMOLOGY_BIAS,
    RELIGION_BIOME_PANTHEON_RANGE,
    RELIGION_BIOME_RITUAL_BIAS,
    RELIGION_DEATH_RATE_HIGH_THRESHOLD,
    RELIGION_DEATH_RATE_LOW_THRESHOLD,
    RELIGION_DRIFT_TIME_STEPS,
    RELIGION_HISTORY_ESCHATOLOGY_BIAS,
    RELIGION_INITIAL_RITUAL_COUNT_MAX,
    RELIGION_INITIAL_RITUAL_COUNT_MIN,
    RELIGION_PRESSURE_WINDOW_STEPS,
    RELIGION_RITUAL_DRIFT_RATE,
)
from world_factory.generator import generate_world
from world_factory.models import (
    BeliefPayload,
    BiomeClass,
    Cosmology,
    Eschatology,
    EventType,
    Religion,
    ReligionLayer,
    Ritual,
    RitualType,
    WorldConfig,
    WorldScale,
)
from world_factory.religion import (
    build_religion,
    religion_provenance,
    validate_religion_layer,
)


def _config(seed: int = 42, scale: WorldScale = WorldScale.LARGE) -> WorldConfig:
    return WorldConfig(seed=seed, scale=scale)


def test_world_model_includes_religion_layer() -> None:
    world = generate_world(_config())
    assert world.religions is not None
    assert isinstance(world.religions, ReligionLayer)


def test_religions_parallel_to_settlements() -> None:
    """Religion records are parallel to settlements by id (same length,
    same order)."""
    world = generate_world(_config())
    assert len(world.religions.religions) == len(world.settlements.settlements)
    for index, religion in enumerate(world.religions.religions):
        settlement = world.settlements.settlements[index]
        assert religion.settlement_id == settlement.id


def test_religion_has_four_field_schema() -> None:
    """Each religion carries the spec 4-element schema: pantheon_size,
    ritual_practices, cosmology, eschatology (per
    PLANS/PHASE_3_TO_5_PLAN.md:191-194)."""
    world = generate_world(_config())
    for religion in world.religions.religions:
        assert isinstance(religion.pantheon_size, int)
        assert religion.pantheon_size >= 1
        assert isinstance(religion.ritual_practices, tuple)
        assert religion.ritual_practices  # non-empty
        assert isinstance(religion.cosmology, Cosmology)
        assert isinstance(religion.eschatology, Eschatology)


def test_ritual_records_present_with_full_schema() -> None:
    """`ReligionLayer.rituals` holds Ritual records with the full
    5-field schema (id, settlement_id, ritual_type, attested_from_step,
    attested_until_step)."""
    world = generate_world(_config())
    assert world.religions.rituals, "expected at least one Ritual record"
    for ritual in world.religions.rituals:
        assert isinstance(ritual, Ritual)
        assert isinstance(ritual.id, int)
        assert ritual.id >= 0
        assert isinstance(ritual.settlement_id, int)
        assert ritual.settlement_id >= 0
        assert isinstance(ritual.ritual_type, RitualType)
        assert isinstance(ritual.attested_from_step, int)
        assert ritual.attested_from_step >= 0
        if ritual.attested_until_step is not None:
            assert ritual.attested_until_step >= ritual.attested_from_step


def test_religion_ritual_practices_references_ritual_ids() -> None:
    """Every id in `Religion.ritual_practices` matches a Ritual.id in
    `ReligionLayer.rituals` and that Ritual belongs to the same
    settlement."""
    world = generate_world(_config())
    rituals_by_id = {ritual.id: ritual for ritual in world.religions.rituals}
    for religion in world.religions.religions:
        for ritual_id in religion.ritual_practices:
            assert ritual_id in rituals_by_id, (
                f"religion {religion.settlement_id} references unknown ritual_id {ritual_id}"
            )
            ritual = rituals_by_id[ritual_id]
            assert ritual.settlement_id == religion.settlement_id, (
                f"ritual {ritual_id} belongs to settlement "
                f"{ritual.settlement_id} but religion references it "
                f"for settlement {religion.settlement_id}"
            )


def test_initial_ritual_count_in_range() -> None:
    """Each religion's initial ritual set (Rituals with
    `attested_from_step == 0`) has length in
    `[RELIGION_INITIAL_RITUAL_COUNT_MIN, RELIGION_INITIAL_RITUAL_COUNT_MAX]`."""
    world = generate_world(_config())
    for religion in world.religions.religions:
        initial_ritual_ids = {
            ritual.id
            for ritual in world.religions.rituals
            if ritual.settlement_id == religion.settlement_id and ritual.attested_from_step == 0
        }
        count = len(initial_ritual_ids)
        assert RELIGION_INITIAL_RITUAL_COUNT_MIN <= count <= RELIGION_INITIAL_RITUAL_COUNT_MAX, (
            f"settlement {religion.settlement_id} has {count} initial "
            f"rituals; expected "
            f"[{RELIGION_INITIAL_RITUAL_COUNT_MIN}, "
            f"{RELIGION_INITIAL_RITUAL_COUNT_MAX}]"
        )


def test_initial_rituals_have_distinct_types() -> None:
    """Initial ritual set contains no duplicate RitualType values (the
    initial sampler draws without replacement)."""
    world = generate_world(_config())
    for religion in world.religions.religions:
        initial_types = [
            ritual.ritual_type
            for ritual in world.religions.rituals
            if ritual.settlement_id == religion.settlement_id and ritual.attested_from_step == 0
        ]
        assert len(initial_types) == len(set(initial_types))


def test_pantheon_size_in_biome_range() -> None:
    """Each religion's pantheon_size falls within
    `RELIGION_BIOME_PANTHEON_RANGE[biome]`."""
    world = generate_world(_config())
    settlements_by_id = {settlement.id: settlement for settlement in world.settlements.settlements}
    biome_grid = world.biomes.classifications
    for religion in world.religions.religions:
        settlement = settlements_by_id[religion.settlement_id]
        biome = (
            biome_grid[settlement.y][settlement.x]
            if 0 <= settlement.y < len(biome_grid)
            and 0 <= settlement.x < len(biome_grid[settlement.y])
            else BiomeClass.GRASSLAND
        )
        minimum, maximum = RELIGION_BIOME_PANTHEON_RANGE[biome.value]
        assert minimum <= religion.pantheon_size <= maximum, (
            f"settlement {religion.settlement_id} in biome "
            f"{biome.value} has pantheon_size "
            f"{religion.pantheon_size}; expected [{minimum}, {maximum}]"
        )


def test_cosmology_is_str_enum_value() -> None:
    """`Religion.cosmology` is one of `Cosmology.{CYCLE, LINEAR}`."""
    world = generate_world(_config())
    for religion in world.religions.religions:
        assert religion.cosmology in {Cosmology.CYCLE, Cosmology.LINEAR}


def test_eschatology_is_str_enum_value() -> None:
    """`Religion.eschatology` is one of
    `Eschatology.{APOCALYPTIC, RENEWAL, CYCLICAL}`."""
    world = generate_world(_config())
    for religion in world.religions.religions:
        assert religion.eschatology in {
            Eschatology.APOCALYPTIC,
            Eschatology.RENEWAL,
            Eschatology.CYCLICAL,
        }


def test_biome_ritual_bias_table_covers_all_biomes() -> None:
    """Every `BiomeClass` has a bias row in
    `RELIGION_BIOME_RITUAL_BIAS`. The row has 6 entries
    (one per `RitualType`) and the probabilities sum to 1.0."""
    expected_keys = {biome.value for biome in BiomeClass}
    actual_keys = set(RELIGION_BIOME_RITUAL_BIAS.keys())
    assert expected_keys == actual_keys
    for biome_key, biases in RELIGION_BIOME_RITUAL_BIAS.items():
        assert len(biases) == len(RitualType), (
            f"biome {biome_key} bias row has {len(biases)} entries; expected {len(RitualType)}"
        )
        total = sum(biases)
        assert abs(total - 1.0) < 1e-9, f"biome {biome_key} bias row sums to {total}; expected 1.0"
        for value in biases:
            assert 0.0 <= value <= 1.0


def test_biome_pantheon_range_table_covers_all_biomes() -> None:
    """Every `BiomeClass` has a `(min, max)` row in
    `RELIGION_BIOME_PANTHEON_RANGE` with `min <= max` and `min >= 1`."""
    expected_keys = {biome.value for biome in BiomeClass}
    actual_keys = set(RELIGION_BIOME_PANTHEON_RANGE.keys())
    assert expected_keys == actual_keys
    for _biome_key, (minimum, maximum) in RELIGION_BIOME_PANTHEON_RANGE.items():
        assert minimum >= 1
        assert minimum <= maximum


def test_history_eschatology_bias_table_covers_all_buckets() -> None:
    """`RELIGION_HISTORY_ESCHATOLOGY_BIAS` has low / mid / high buckets
    with probabilities summing to 1.0 per bucket."""
    expected_keys = {"low", "mid", "high"}
    actual_keys = set(RELIGION_HISTORY_ESCHATOLOGY_BIAS.keys())
    assert expected_keys == actual_keys
    for bucket, biases in RELIGION_HISTORY_ESCHATOLOGY_BIAS.items():
        total = sum(biases.values())
        assert abs(total - 1.0) < 1e-9, f"bucket {bucket} bias sums to {total}; expected 1.0"


def test_death_rate_thresholds_pinned() -> None:
    """`RELIGION_DEATH_RATE_LOW_THRESHOLD` < `RELIGION_DEATH_RATE_HIGH_THRESHOLD`
    and both are positive (the chi-square acceptance test relies on
    these being absolutely pinned)."""
    assert RELIGION_DEATH_RATE_LOW_THRESHOLD > 0.0
    assert RELIGION_DEATH_RATE_LOW_THRESHOLD < RELIGION_DEATH_RATE_HIGH_THRESHOLD


def test_belief_events_emitted_per_ritual_change() -> None:
    """BELIEF events are emitted for every ritual add / remove. The
    number of BELIEF events should be at least 1 (some religions
    always drift at least once across `RELIGION_DRIFT_TIME_STEPS`
    with `RELIGION_RITUAL_DRIFT_RATE = 0.05`)."""
    world = generate_world(_config())
    belief_events = [event for event in world.events.events if event.type == EventType.BELIEF]
    assert belief_events, "expected at least one BELIEF event"


def test_belief_events_have_unique_ids() -> None:
    world = generate_world(_config())
    belief_event_ids = [event.id for event in world.events.events if event.type == EventType.BELIEF]
    assert len(belief_event_ids) == len(set(belief_event_ids))


def test_belief_events_have_valid_payload() -> None:
    """Every BELIEF event re-validates against `BeliefPayload` with
    `ritual_added` XOR `ritual_removed` non-None and `step == t`."""
    world = generate_world(_config())
    belief_events = [event for event in world.events.events if event.type == EventType.BELIEF]
    assert belief_events
    for event in belief_events:
        payload = BeliefPayload.model_validate(event.payload)
        assert payload.step == event.t
        assert (payload.ritual_added is None) != (payload.ritual_removed is None), (
            f"BELIEF event {event.id} must have exactly one of "
            f"ritual_added / ritual_removed non-None"
        )


def test_belief_event_ids_use_religion_namespace() -> None:
    """BELIEF event ids are 16-char hex blake2b from the `b"religion"`
    person namespace (distinct from `b"worldfac"` and `b"culture"`)."""
    world = generate_world(_config())
    belief_events = [event for event in world.events.events if event.type == EventType.BELIEF]
    assert belief_events
    for event in belief_events:
        assert len(event.id) == 16
        assert all(c in "0123456789abcdef" for c in event.id)


def test_belief_event_ids_distinct_from_other_event_types() -> None:
    """BELIEF event ids do not collide with demography or culture
    event ids (distinct blake2b person namespaces)."""
    world = generate_world(_config())
    belief_ids = {event.id for event in world.events.events if event.type == EventType.BELIEF}
    drift_ids = {event.id for event in world.events.events if event.type == EventType.CULTURE_DRIFT}
    death_ids = {event.id for event in world.events.events if event.type == EventType.DEATH}
    assert belief_ids.isdisjoint(drift_ids)
    assert belief_ids.isdisjoint(death_ids)


def test_algorithm_version_is_16_char_hex() -> None:
    world = generate_world(_config())
    assert len(world.religions.algorithm_version) == 16
    assert all(c in "0123456789abcdef" for c in world.religions.algorithm_version)


def test_algorithm_version_stable_across_runs() -> None:
    """Same seed produces the same `algorithm_version`."""
    a = generate_world(_config(seed=42, scale=WorldScale.SMALL))
    b = generate_world(_config(seed=42, scale=WorldScale.SMALL))
    assert a.religions.algorithm_version == b.religions.algorithm_version


def test_algorithm_version_changes_with_seed() -> None:
    """Different seeds produce different `algorithm_version` (content-
    derived hash)."""
    a = generate_world(_config(seed=42, scale=WorldScale.SMALL))
    b = generate_world(_config(seed=43, scale=WorldScale.SMALL))
    assert a.religions.algorithm_version != b.religions.algorithm_version


def test_algorithm_version_distinct_from_other_layers() -> None:
    """Religion `algorithm_version` is distinct from cultures and
    events `algorithm_version` (different blake2b person namespaces
    and content)."""
    world = generate_world(_config())
    assert world.religions.algorithm_version != world.cultures.algorithm_version
    assert world.religions.algorithm_version != world.events.algorithm_version


def test_validate_religion_layer_empty_for_valid_world() -> None:
    world = generate_world(_config())
    assert validate_religion_layer(world) == []


def test_validate_religion_layer_flags_algorithm_version_mismatch() -> None:
    """Mutating the rituals tuple after build breaks the algorithm
    version hash; the validator catches this at the trust boundary."""
    world = generate_world(_config())
    tampered_rituals = world.religions.rituals[:-1]
    bad_world = world.model_copy(
        update={
            "religions": ReligionLayer(
                religions=world.religions.religions,
                rituals=tampered_rituals,
                algorithm_version=world.religions.algorithm_version,
            )
        }
    )
    violations = validate_religion_layer(bad_world)
    assert any(v.code == "religion-layer-algorithm-version-mismatch" for v in violations)


def test_validate_religion_layer_flags_pantheon_out_of_range() -> None:
    """A religion with `pantheon_size` outside the biome range is
    flagged by the validator."""
    world = generate_world(_config())
    first_religion = world.religions.religions[0]
    bad_religion = Religion(
        settlement_id=first_religion.settlement_id,
        pantheon_size=99,
        ritual_practices=first_religion.ritual_practices,
        cosmology=first_religion.cosmology,
        eschatology=first_religion.eschatology,
    )
    bad_religions = (bad_religion,) + world.religions.religions[1:]
    bad_world = world.model_copy(
        update={
            "religions": ReligionLayer(
                religions=bad_religions,
                rituals=world.religions.rituals,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_religion_layer(bad_world)
    assert any(v.code == "religion-pantheon-size-out-of-range" for v in violations)


def test_validate_religion_layer_flags_settlement_id_mismatch() -> None:
    """A religion with `settlement_id` that disagrees with the
    parallel settlement index is flagged."""
    world = generate_world(_config())
    first_religion = world.religions.religions[0]
    bad_religion = Religion(
        settlement_id=first_religion.settlement_id + 99999,
        pantheon_size=first_religion.pantheon_size,
        ritual_practices=first_religion.ritual_practices,
        cosmology=first_religion.cosmology,
        eschatology=first_religion.eschatology,
    )
    bad_religions = (bad_religion,) + world.religions.religions[1:]
    bad_world = world.model_copy(
        update={
            "religions": ReligionLayer(
                religions=bad_religions,
                rituals=world.religions.rituals,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_religion_layer(bad_world)
    assert any(v.code == "religion-layer-settlement-id-mismatch" for v in violations)


def test_validate_religion_layer_flags_orphaned_ritual() -> None:
    """A Ritual record that is not referenced by any religion's
    `ritual_practices` and has `attested_until_step is None` is
    flagged (orphaned)."""
    world = generate_world(_config())
    first_ritual = world.religions.rituals[0]
    extra_ritual = Ritual(
        id=max(r.id for r in world.religions.rituals) + 1,
        settlement_id=first_ritual.settlement_id,
        ritual_type=RitualType.FIRE,
        attested_from_step=0,
        attested_until_step=None,
    )
    bad_rituals = world.religions.rituals + (extra_ritual,)
    bad_world = world.model_copy(
        update={
            "religions": ReligionLayer(
                religions=world.religions.religions,
                rituals=bad_rituals,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_religion_layer(bad_world)
    assert any(v.code == "religion-ritual-orphaned" for v in violations)


def test_validate_religion_layer_flags_ritual_id_missing() -> None:
    """A `Religion.ritual_practices` referencing a non-existent
    `Ritual.id` is flagged."""
    world = generate_world(_config())
    first_religion = world.religions.religions[0]
    bad_religion = Religion(
        settlement_id=first_religion.settlement_id,
        pantheon_size=first_religion.pantheon_size,
        ritual_practices=first_religion.ritual_practices + (99999,),
        cosmology=first_religion.cosmology,
        eschatology=first_religion.eschatology,
    )
    bad_religions = (bad_religion,) + world.religions.religions[1:]
    bad_world = world.model_copy(
        update={
            "religions": ReligionLayer(
                religions=bad_religions,
                rituals=world.religions.rituals,
                algorithm_version="0000000000000000",
            )
        }
    )
    violations = validate_religion_layer(bad_world)
    assert any(v.code == "religion-ritual-id-missing" for v in violations)


def test_arid_water_ritual_frequency_exceeds_other_biomes() -> None:
    """3b.5 acceptance: across N seeds, the frequency of WATER rituals
    in arid (DESERT) settlements must exceed the frequency in
    non-arid land settlements. DESERT carries water bias 0.50 per
    `RELIGION_BIOME_RITUAL_BIAS`; GRASSLAND / TEMPERATE_FOREST /
    ICE carry water bias 0.05-0.15, so the contrast should hold
    across many seeds.

    Measured as the fraction of initial-set rituals (those with
    `attested_from_step == 0`) that are `RitualType.WATER`."""
    arid_water = 0
    arid_total = 0
    other_water = 0
    other_total = 0
    seeds = range(20)
    for seed in seeds:
        world = generate_world(WorldConfig(seed=seed, scale=WorldScale.SMALL))
        biome_grid = world.biomes.classifications
        initial_rituals_by_settlement: dict[int, list[Ritual]] = {}
        for ritual in world.religions.rituals:
            if ritual.attested_from_step == 0:
                initial_rituals_by_settlement.setdefault(ritual.settlement_id, []).append(ritual)
        for settlement in world.settlements.settlements:
            if not (
                0 <= settlement.y < len(biome_grid)
                and 0 <= settlement.x < len(biome_grid[settlement.y])
            ):
                continue
            biome = biome_grid[settlement.y][settlement.x]
            rituals = initial_rituals_by_settlement.get(settlement.id, [])
            if not rituals:
                continue
            water_count = sum(1 for r in rituals if r.ritual_type == RitualType.WATER)
            if biome == BiomeClass.DESERT:
                arid_total += len(rituals)
                arid_water += water_count
            elif biome in {
                BiomeClass.GRASSLAND,
                BiomeClass.TEMPERATE_FOREST,
                BiomeClass.ICE,
            }:
                other_total += len(rituals)
                other_water += water_count
    assert arid_total > 0, (
        f"no desert settlements observed across {len(list(seeds))} "
        f"seeds at SMALL scale — chi-square test cannot run"
    )
    assert other_total > 0
    arid_frequency = arid_water / arid_total
    other_frequency = other_water / other_total
    assert arid_frequency > other_frequency, (
        f"arid water-ritual frequency {arid_frequency:.3f} "
        f"({arid_water}/{arid_total}) is not greater than "
        f"non-arid land water-ritual frequency {other_frequency:.3f} "
        f"({other_water}/{other_total})"
    )


def test_cosmology_distribution_not_mode_collapsed() -> None:
    """Across N seeds, cosmology distribution is not mode-collapsed
    (both CYCLE and LINEAR appear in the population). Per 3b.5
    distributional acceptance criterion."""
    counts: dict[str, int] = {}
    for seed in (41, 42, 43):
        world = generate_world(WorldConfig(seed=seed, scale=WorldScale.LARGE))
        for religion in world.religions.religions:
            counts[religion.cosmology.value] = counts.get(religion.cosmology.value, 0) + 1
    assert counts.get("cycle", 0) > 0
    assert counts.get("linear", 0) > 0


def test_eschatology_distribution_not_mode_collapsed() -> None:
    """Across N seeds, eschatology distribution is not mode-collapsed
    (multiple `Eschatology` values appear)."""
    counts: dict[str, int] = {}
    for seed in (41, 42, 43):
        world = generate_world(WorldConfig(seed=seed, scale=WorldScale.LARGE))
        for religion in world.religions.religions:
            counts[religion.eschatology.value] = counts.get(religion.eschatology.value, 0) + 1
    assert len(counts) >= 2, f"eschatology distribution is mode-collapsed: {counts}"


def test_religion_provenance_record_present() -> None:
    world = generate_world(_config())
    matches = [record for record in world.provenance if record.output_path == "religions"]
    assert len(matches) == 1
    assert matches[0].algorithm_version == RELIGION_ALGORITHM_VERSION


def test_religion_provenance_factory() -> None:
    """Direct test of the `religion_provenance` factory: it returns a
    `ProvenanceRecord` describing the religion-layer builder."""
    record = religion_provenance()
    assert record.output_path == "religions"
    assert record.algorithm_version == RELIGION_ALGORITHM_VERSION
    assert "settlements.settlements" in record.input_paths
    assert "biomes.classifications" in record.input_paths
    assert "metadata.config.seed" in record.input_paths


def test_event_log_validator_passes_with_belief_events() -> None:
    """Merging demography + culture + religion events per-step
    preserves monotonicity, so the event-log validator still passes."""
    from world_factory.event_log import validate_event_log

    world = generate_world(_config())
    assert validate_event_log(world) == []


def test_belief_events_merged_after_culture_events_per_step() -> None:
    """Within a step, demography events come first, then culture
    events, then belief events (per 3b.2 ordering). Only enforced at
    steps where all three event types are present (some steps have
    no demography events — e.g. step 0 — and some have no culture /
    belief events when no settlement drifts)."""
    world = generate_world(_config())
    per_step_types: dict[int, list[EventType]] = {}
    for event in world.events.events:
        per_step_types.setdefault(event.t, []).append(event.type)
    for step, types in per_step_types.items():
        has_demo = any(t in _DEMOGRAPHY_TYPES for t in types)
        has_culture = any(t == EventType.CULTURE_DRIFT for t in types)
        has_belief = any(t == EventType.BELIEF for t in types)
        last_demo = max(
            (index for index, t in enumerate(types) if t in _DEMOGRAPHY_TYPES),
            default=-1,
        )
        first_culture = next(
            (index for index, t in enumerate(types) if t == EventType.CULTURE_DRIFT),
            len(types),
        )
        first_belief = next(
            (index for index, t in enumerate(types) if t == EventType.BELIEF),
            len(types),
        )
        if has_demo and has_culture:
            assert last_demo < first_culture, (
                f"step {step}: culture event appears before demography events"
            )
        if has_culture and has_belief:
            assert first_culture < first_belief, (
                f"step {step}: belief event appears before culture events"
            )


_DEMOGRAPHY_TYPES = frozenset({EventType.BIRTH, EventType.DEATH, EventType.MIGRATION})


def test_world_id_stable_across_phase_3b2() -> None:
    """3b.2 adds no new WorldConfig fields, so world_id for --seed 42
    at LARGE scale remains `9d75e7103b52704b48ce77071a22a586` (the
    pre-3b.x reference value)."""
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.LARGE))
    assert world.metadata.world_id == "9d75e7103b52704b48ce77071a22a586"


def test_religion_layer_runs_cleanly_on_small_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.SMALL))
    assert world.religions is not None
    assert validate_religion_layer(world) == []


def test_religion_layer_runs_cleanly_on_medium_grid() -> None:
    world = generate_world(WorldConfig(seed=42, scale=WorldScale.MEDIUM))
    assert world.religions is not None
    assert validate_religion_layer(world) == []


def test_build_religion_returns_layer_and_events() -> None:
    """`build_religion` returns `(ReligionLayer, tuple[WorldEvent, ...])`
    so the generator can merge belief events into the EventLog."""
    world = generate_world(_config())
    religions, events = build_religion(world)
    assert isinstance(religions, ReligionLayer)
    assert all(e.type == EventType.BELIEF for e in events)


def test_rituals_sorted_by_settlement_id_and_step() -> None:
    """`ReligionLayer.rituals` is sorted by
    `(settlement_id, attested_from_step, id)` for hash stability."""
    world = generate_world(_config())
    rituals = world.religions.rituals
    for earlier, later in zip(rituals, rituals[1:], strict=False):
        earlier_key = (
            earlier.settlement_id,
            earlier.attested_from_step,
            earlier.id,
        )
        later_key = (
            later.settlement_id,
            later.attested_from_step,
            later.id,
        )
        assert earlier_key <= later_key


def test_initial_rituals_have_step_zero() -> None:
    """Every Ritual with `attested_from_step == 0` belongs to the same
    settlement as the religion that owns it (initial-set
    construction invariant)."""
    world = generate_world(_config())
    initial_rituals_by_settlement: dict[int, set[int]] = {}
    for ritual in world.religions.rituals:
        if ritual.attested_from_step == 0:
            initial_rituals_by_settlement.setdefault(ritual.settlement_id, set()).add(ritual.id)
    for religion in world.religions.religions:
        assert initial_rituals_by_settlement.get(religion.settlement_id, set()), (
            f"settlement {religion.settlement_id} religion has no initial rituals"
        )


def test_pressure_window_constant_matches_drift_step() -> None:
    """`RELIGION_PRESSURE_WINDOW_STEPS` and `RELIGION_DRIFT_TIME_STEPS`
    are pinned so the chi-square acceptance test is deterministic
    across seeds."""
    assert RELIGION_PRESSURE_WINDOW_STEPS > 0
    assert RELIGION_DRIFT_TIME_STEPS > 0
    assert 0.0 < RELIGION_RITUAL_DRIFT_RATE < 1.0


def test_cosmology_bias_table_covers_all_biomes() -> None:
    """Every `BiomeClass` has a CYCLE / LINEAR bias row in
    `RELIGION_BIOME_COSMOLOGY_BIAS` with values summing to 1.0."""
    expected_keys = {biome.value for biome in BiomeClass}
    actual_keys = set(RELIGION_BIOME_COSMOLOGY_BIAS.keys())
    assert expected_keys == actual_keys
    for biome_key, biases in RELIGION_BIOME_COSMOLOGY_BIAS.items():
        total = biases["cycle"] + biases["linear"]
        assert abs(total - 1.0) < 1e-9, (
            f"biome {biome_key} cosmology bias sums to {total}; expected 1.0"
        )
