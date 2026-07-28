"""Phase 3b.1 cultures — per-settlement culture attribute vectors with
neighbor-correlated drift.

Per `PLANS/PHASE_3_TO_5_PLAN.md:175-184`: each culture carries a 6-dim
attribute vector (values, norms, taboos, ritual forms, cuisine, music
motifs) in `[0, 1]`. Initial values are biome-biased (per
`CULTURE_BIOME_BIAS_TABLE`); per-epoch drift = small stochastic
perturbation + neighbor-correlation pull toward the mean of the K
nearest cultures (by spatial proximity). Drift events are emitted per
changed attribute per (settlement, step).

`algorithm_version` is a blake2b hash of the culture layer so any
mutation / re-ordering breaks the version and is detected at the trust
boundary. `validate_cultures_layer` enforces:

- `cultures` is parallel to `SettlementsLayer.settlements` by id
  (same length, same order)
- `attribute_history` length is `time_steps + 1`
- all attribute values are in `[0, 1]`
- `algorithm_version` matches a fresh blake2b of the layer
- (no mode-collapse / cross-seed variance check — that lives in
  `tests/test_cultures.py` as a distributional assertion)

The drift model is deterministic given (seed, world state): neighbor
pull is a function of state; perturbation uses
`sample_unit_interval(seed, "culture.<attr>", settlement.id, step)`.
"""

from __future__ import annotations

import hashlib
import math
import struct
from collections.abc import Sequence

from world_factory.constants import (
    CULTURE_ALGORITHM_VERSION,
    CULTURE_ATTRIBUTE_NAMES,
    CULTURE_BIOME_BIAS_TABLE,
    CULTURE_DRIFT_PULL,
    CULTURE_DRIFT_RATE,
    CULTURE_DRIFT_TIME_STEPS,
    CULTURE_NEIGHBOR_K,
    CULTURE_PER_ATTR_NOISE,
)
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiomeClass,
    Culture,
    CultureDriftPayload,
    CultureLayer,
    EventActor,
    EventLocation,
    EventType,
    ProvenanceRecord,
    WorldEvent,
    WorldModel,
)

_ATTRIBUTE_COUNT = len(CULTURE_ATTRIBUTE_NAMES)
_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1


def _compute_algorithm_version(cultures: tuple[Culture, ...]) -> str:
    """blake2b hash of the culture layer state. 16-char hex.

    Hashes settlement id + every (step, attribute) value so any
    mutation / re-ordering breaks the version. Mirrors the event-log
    algorithm-version pattern at `event_log._compute_algorithm_version`.
    """
    digest = hashlib.blake2b(digest_size=8, person=b"culture")
    for culture in cultures:
        digest.update(struct.pack(">q", culture.settlement_id))
        for step_vector in culture.attribute_history:
            for attr_value in step_vector:
                digest.update(struct.pack(">d", attr_value))
    return digest.hexdigest()


def _make_event_id(
    seed: int,
    event_type: EventType,
    step: int,
    settlement_id: int,
    salt: str,
) -> str:
    """Deterministic 16-char hex event id via blake2b.

    Uses a distinct blake2b person namespace (`b"culture"`) so culture
    event ids cannot collide with demography event ids.
    """
    digest = hashlib.blake2b(digest_size=8, person=b"culture")
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(event_type.value.encode("utf-8"))
    digest.update(struct.pack(">q", step))
    digest.update(struct.pack(">q", settlement_id))
    digest.update(salt.encode("utf-8"))
    return digest.hexdigest()


def _clamp_vector(
    vector: tuple[float, ...],
) -> tuple[float, ...]:
    """Clamp each attribute to [0, 1]. Round to 6 dp for byte-equal
    determinism (mirrors `_generate_grid` rounding in generator.py).
    """
    return tuple(round(max(0.0, min(1.0, value)), 6) for value in vector)


def _initial_attribute_vector(
    seed: int,
    settlement_id: int,
    biome: BiomeClass,
) -> tuple[float, ...]:
    """Biome-biased initial vector + small per-culture RNG noise.

    Noise is namespaced per `(settlement, attribute)` so the same seed
    produces the same initial vector across runs.
    """
    bias = CULTURE_BIOME_BIAS_TABLE[biome]
    values: list[float] = []
    for attr_index in range(_ATTRIBUTE_COUNT):
        noise = (
            sample_unit_interval(seed, "culture.init", settlement_id, attr_index)
            - 0.5
        ) * CULTURE_PER_ATTR_NOISE
        values.append(bias[attr_index] + noise)
    return _clamp_vector(tuple(values))


def _neighbor_mean_vector(
    self_index: int,
    cultures: Sequence[Culture],
    settlement_positions: dict[int, tuple[int, int]],
    step_index: int,
    k: int,
) -> tuple[float, ...]:
    """Mean attribute vector of the K nearest cultures (by settlement
    Euclidean distance) at `step_index`. Returns the zero vector when
    fewer than K other cultures exist (single-settlement worlds) — the
    drift step then has no neighbor pull but still applies the
    stochastic perturbation."""
    self_settlement_id = cultures[self_index].settlement_id
    self_x, self_y = settlement_positions[self_settlement_id]
    distances: list[tuple[float, int]] = []
    for other_index, other_culture in enumerate(cultures):
        if other_index == self_index:
            continue
        other_x, other_y = settlement_positions[other_culture.settlement_id]
        distance = math.hypot(other_x - self_x, other_y - self_y)
        distances.append((distance, other_index))
    distances.sort(key=lambda pair: (pair[0], pair[1]))
    keep = distances[:k]
    if not keep:
        return tuple(0.0 for _ in range(_ATTRIBUTE_COUNT))
    sums = [0.0] * _ATTRIBUTE_COUNT
    for _, other_index in keep:
        other_vector = cultures[other_index].attribute_history[step_index]
        for attr_index in range(_ATTRIBUTE_COUNT):
            sums[attr_index] += other_vector[attr_index]
    return tuple(sums[attr_index] / len(keep) for attr_index in range(_ATTRIBUTE_COUNT))


def _drift_vector(
    seed: int,
    settlement_id: int,
    current: tuple[float, ...],
    neighbor_mean: tuple[float, ...],
    step: int,
) -> tuple[float, ...]:
    """Per-step drift: pull toward neighbor mean + per-attribute
    stochastic perturbation. Pull is `CULTURE_DRIFT_PULL * (mean -
    current)`; perturbation is `+/- CULTURE_DRIFT_RATE` per attribute
    (uniformly sampled). Output clamped to [0, 1]."""
    perturbed: list[float] = []
    for attr_index, current_value in enumerate(current):
        pull = neighbor_mean[attr_index] - current_value
        perturbation = (
            sample_unit_interval(
                seed, "culture.drift", settlement_id, step, attr_index
            )
            - 0.5
        ) * 2.0 * CULTURE_DRIFT_RATE
        perturbed.append(current_value + CULTURE_DRIFT_PULL * pull + perturbation)
    return _clamp_vector(tuple(perturbed))


def _emit_drift_events(
    seed: int,
    settlement_id: int,
    settlement_x: int,
    settlement_y: int,
    previous: tuple[float, ...],
    current: tuple[float, ...],
    step: int,
    provenance: ProvenanceRecord,
) -> tuple[WorldEvent, ...]:
    """Emit one CULTURE_DRIFT event per attribute that changed in this
    step. Per-attribute events keep the event log compact (one event
    per `(settlement, step, attribute)` rather than one event per
    `(settlement, step)` carrying the full vector)."""
    events: list[WorldEvent] = []
    for attr_index, (old_value, new_value) in enumerate(
        zip(previous, current, strict=True)
    ):
        if old_value == new_value:
            continue
        attribute_name = CULTURE_ATTRIBUTE_NAMES[attr_index]
        events.append(
            WorldEvent(
                id=_make_event_id(
                    seed,
                    EventType.CULTURE_DRIFT,
                    step,
                    settlement_id,
                    f"{attribute_name}",
                ),
                type=EventType.CULTURE_DRIFT,
                t=step,
                location=EventLocation(
                    cell=(settlement_x, settlement_y),
                    settlement_id=settlement_id,
                ),
                actors=(
                    EventActor(
                        kind="culture",
                        identifier=f"culture:{settlement_id}",
                        display_name=None,
                    ),
                ),
                payload=CultureDriftPayload(
                    settlement_id=settlement_id,
                    attribute=attribute_name,
                    old_value=old_value,
                    new_value=new_value,
                    step=step,
                ).model_dump(mode="python"),
                causes=(),
                provenance=provenance,
            )
        )
    return tuple(events)


def _lookup_biome(
    biome_grid: tuple[tuple[BiomeClass, ...], ...],
    x: int,
    y: int,
) -> BiomeClass:
    """Look up the biome at (x, y). Defensive: returns GRASSLAND if
    the cell is out of bounds (settlements should always be in-bounds;
    this guards against future scale changes)."""
    if 0 <= y < len(biome_grid) and 0 <= x < len(biome_grid[y]):
        return biome_grid[y][x]
    return BiomeClass.GRASSLAND


def build_cultures(
    world: WorldModel,
    time_steps: int = CULTURE_DRIFT_TIME_STEPS,
) -> tuple[CultureLayer, tuple[WorldEvent, ...]]:
    """Build the top-level CultureLayer from the world's settlements.

    One culture per settlement (v1 simplification; multi-culture per
    settlement deferred). Initial attribute vectors are biome-biased;
    per-step drift = neighbor-correlation pull toward the K=3 nearest
    cultures' mean + small stochastic perturbation. Emits one
    CULTURE_DRIFT event per changed attribute per step.

    Returns `(CultureLayer, tuple[WorldEvent, ...])`: the layer and
    the emitted drift events. The generator concatenates the drift
    events with the demography events and re-runs `build_event_log`
    so the top-level `EventLog` carries the full history.

    Determinism: identical seeds produce byte-equivalent output. RNG
    calls are namespaced as `"culture.init"` (initial noise) and
    `"culture.drift"` (per-step perturbation).
    """
    seed = world.metadata.config.seed
    settlements = world.settlements.settlements
    biome_grid = world.biomes.classifications
    provenance = cultures_provenance()
    sorted_settlements = sorted(settlements, key=lambda s: s.id)
    settlement_positions: dict[int, tuple[int, int]] = {
        settlement.id: (settlement.x, settlement.y)
        for settlement in sorted_settlements
    }

    # Initialize attribute history: index 0 is initial vector,
    # indices 1..time_steps are post-step vectors.
    cultures: list[Culture] = []
    for settlement in sorted_settlements:
        biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
        initial_vector = _initial_attribute_vector(seed, settlement.id, biome)
        cultures.append(
            Culture(
                settlement_id=settlement.id,
                attribute_history=(initial_vector,),
            )
        )

    events: list[WorldEvent] = []

    for step in range(1, time_steps + 1):
        # Compute next-step vectors in parallel: every culture pulls
        # toward the prior-step mean of its K nearest neighbors. This
        # keeps the step a pure function of (cultures[step-1],
        # sorted_settlements, seed, step) so order-of-update cannot
        # leak in.
        next_history_per_culture: list[tuple[tuple[float, ...], ...]] = [
            culture.attribute_history for culture in cultures
        ]
        for self_index, culture in enumerate(cultures):
            previous = culture.attribute_history[step - 1]
            neighbor_mean = _neighbor_mean_vector(
                self_index,
                cultures,
                settlement_positions,
                step - 1,
                k=CULTURE_NEIGHBOR_K,
            )
            next_vector = _drift_vector(
                seed,
                culture.settlement_id,
                previous,
                neighbor_mean,
                step,
            )
            next_history_per_culture[self_index] = culture.attribute_history + (
                next_vector,
            )
            settlement_x, settlement_y = settlement_positions[culture.settlement_id]
            events.extend(
                _emit_drift_events(
                    seed,
                    culture.settlement_id,
                    settlement_x,
                    settlement_y,
                    previous,
                    next_vector,
                    step,
                    provenance,
                )
            )
        cultures = [
            Culture(
                settlement_id=culture.settlement_id,
                attribute_history=next_history_per_culture[index],
            )
            for index, culture in enumerate(cultures)
        ]

    algorithm_version = _compute_algorithm_version(tuple(cultures))
    return (
        CultureLayer(
            cultures=tuple(cultures),
            algorithm_version=algorithm_version,
        ),
        tuple(events),
    )


def validate_cultures_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3b.1 culture-layer invariants.

    Checks:
    - `cultures` is parallel to `SettlementsLayer.settlements` by id
      (same length, same order).
    - `attribute_history` length is `time_steps + 1` (length matches
      `DEMOGRAPHY_DEFAULT_TIME_STEPS` per settlement; we don't pin
      this in stone — the check is that all cultures agree).
    - All attribute values are in `[0, 1]`.
    - `algorithm_version` matches a fresh blake2b of the layer
      (catches silent mutation / re-ordering at the trust boundary).

    The algorithm-version check runs first (so a length-mutated
    layer still surfaces the mutation) and per-culture invariants
    accumulate without bailing early on the first length mismatch.
    """
    violations: list[InvariantViolation] = []
    settlements = world.settlements.settlements
    layer = world.cultures

    # Algorithm-version check first: catches any mutation / reorder
    # of the cultures tuple regardless of length agreement.
    expected_version = _compute_algorithm_version(layer.cultures)
    if layer.algorithm_version != expected_version:
        violations.append(
            _violation(
                "culture-layer-algorithm-version-mismatch",
                "cultures.algorithm_version",
                (
                    f"culture algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; layer was mutated "
                    f"or re-ordered outside the generator"
                ),
            )
        )

    if len(layer.cultures) != len(settlements):
        violations.append(
            _violation(
                "culture-layer-length-mismatch",
                "cultures.cultures",
                (
                    f"culture layer ({len(layer.cultures)}) does not match "
                    f"settlements ({len(settlements)})"
                ),
            )
        )
        # Continue with per-culture invariants below; the length
        # mismatch is recorded but doesn't excuse further checks
        # from running on the (mis-aligned) cultures.
    expected_history_length: int | None = None
    last_valid_index = min(len(layer.cultures), len(settlements))
    for index in range(last_valid_index):
        culture = layer.cultures[index]
        settlement = settlements[index]
        if culture.settlement_id != settlement.id:
            violations.append(
                _violation(
                    "culture-layer-settlement-id-mismatch",
                    f"cultures.cultures.{index}.settlement_id",
                    (
                        f"culture {index} references "
                        f"settlement_id={culture.settlement_id} but "
                        f"settlements.{index}.id={settlement.id}"
                    ),
                )
            )
        if expected_history_length is None:
            expected_history_length = len(culture.attribute_history)
        elif len(culture.attribute_history) != expected_history_length:
            violations.append(
                _violation(
                    "culture-layer-history-length-mismatch",
                    f"cultures.cultures.{index}.attribute_history",
                    (
                        f"culture {index} has attribute_history length "
                        f"{len(culture.attribute_history)}; expected "
                        f"{expected_history_length}"
                    ),
                )
            )
        for step_index, step_vector in enumerate(culture.attribute_history):
            if len(step_vector) != _ATTRIBUTE_COUNT:
                violations.append(
                    _violation(
                        "culture-layer-attribute-count-mismatch",
                        (
                            f"cultures.cultures.{index}."
                            f"attribute_history.{step_index}"
                        ),
                        (
                            f"step {step_index} has "
                            f"{len(step_vector)} attributes; expected "
                            f"{_ATTRIBUTE_COUNT}"
                        ),
                    )
                )
                continue
            for attr_index, value in enumerate(step_vector):
                if value < 0.0 or value > 1.0:
                    violations.append(
                        _violation(
                            "culture-layer-attribute-out-of-range",
                            (
                                f"cultures.cultures.{index}."
                                f"attribute_history.{step_index}.{attr_index}"
                            ),
                            (
                                f"attribute value {value} is outside [0, 1]"
                            ),
                        )
                    )
    # Flag any extra cultures beyond the settlements length so a
    # length-mutated layer surfaces both the surplus and the
    # algorithm-version mismatch.
    for index in range(last_valid_index, len(layer.cultures)):
        violations.append(
            _violation(
                "culture-layer-surplus",
                f"cultures.cultures.{index}",
                (
                    f"culture {index} has no parallel settlement "
                    f"(settlements has {len(settlements)} entries; "
                    f"cultures has {len(layer.cultures)})"
                ),
            )
        )
    return violations


def cultures_provenance() -> ProvenanceRecord:
    """Provenance record describing the culture-layer builder."""
    return ProvenanceRecord(
        output_path="cultures",
        process=(
            "biome-biased-initial-attribute-vector-with-neighbor-correlated-drift"
        ),
        input_paths=(
            "settlements.settlements",
            "biomes.classifications",
            "metadata.config.seed",
        ),
        algorithm_version=CULTURE_ALGORITHM_VERSION,
    )


__all__ = [
    "build_cultures",
    "cultures_provenance",
    "validate_cultures_layer",
]
