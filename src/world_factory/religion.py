"""Phase 3b.2 religion — per-settlement religion records with per-ritual
provenance, plus belief events emitted on ritual add / remove.

Per `PLANS/PHASE_3_TO_5_PLAN.md:186-198`: each religion carries a
4-element schema (pantheon size, ritual practices, cosmology,
eschatology) plus a per-ritual record set. Initial religion is sampled
from biome + history bias tables (`RELIGION_BIOME_RITUAL_BIAS`,
`RELIGION_BIOME_PANTHEON_RANGE`, `RELIGION_BIOME_COSMOLOGY_BIAS`,
`RELIGION_HISTORY_ESCHATOLOGY_BIAS`). Per-step ritual drift is a low-
probability add / remove driven by the settlement's recent-death-rate
window (`RELIGION_PRESSURE_WINDOW_STEPS`).

`Religion.ritual_practices: tuple[int, ...]` references `Ritual.id`
rather than raw `RitualType` values, so Phase 4 polities and the
Phase 5 causal graph can refer to per-ritual provenance (e.g.,
'polity P suppressed ritual R in settlement S'). `Ritual` records
carry their `attested_from_step` / `attested_until_step` window so
lapsed rituals remain queryable.

Spec fidelity note: the canonical spec calls for separate `Ritual`
records (line 195). This module implements that faithfully rather
than collapsing rituals into an enum on `Religion` — Ernie's plan-ack
Note 1 selected path (b).

`algorithm_version` is a blake2b hash over `(religions, rituals)` so
any mutation / re-ordering breaks the version and is detected at the
trust boundary. `validate_religion_layer` enforces:

- `religions` is parallel to `SettlementsLayer.settlements` by id
  (same length, same order).
- `rituals` is sorted by `(settlement_id, attested_from_step, id)`.
- Each `Religion.ritual_practices` is a tuple of `Ritual.id` values
  where every id points to a Ritual record carried in the layer.
- Pantheon size is in the biome's range (`RELIGION_BIOME_PANTHEON_RANGE`).
- `algorithm_version` matches a fresh blake2b of the layer.

The drift model is deterministic given (seed, world state): ritual
add / remove decisions use
`sample_unit_interval(seed, "religion.drift", settlement_id, step, ...)`
so identical seeds produce byte-equivalent output.
"""

from __future__ import annotations

import hashlib
import struct

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
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BeliefPayload,
    BiomeClass,
    Cosmology,
    Eschatology,
    EventActor,
    EventLocation,
    EventType,
    ProvenanceRecord,
    Religion,
    ReligionLayer,
    Ritual,
    RitualType,
    WorldEvent,
    WorldModel,
)

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1
_HISTORY_BUCKET_LOW = "low"
_HISTORY_BUCKET_MID = "mid"
_HISTORY_BUCKET_HIGH = "high"


def _compute_algorithm_version(
    religions: tuple[Religion, ...],
    rituals: tuple[Ritual, ...],
) -> str:
    """blake2b hash of religion + ritual state. 16-char hex.

    Hashes each `Religion` (settlement_id, pantheon_size, sorted
    ritual_practices, cosmology, eschatology) and each `Ritual`
    (settlement_id, id, ritual_type, from, until). Sorted ordering
    ensures the hash is stable across re-orderings of the rituals
    tuple (the generator sorts before persisting). Mirrors the
    culture / event-log algorithm-version pattern."""
    digest = hashlib.blake2b(digest_size=8, person=b"religion")
    for religion in religions:
        digest.update(struct.pack(">q", religion.settlement_id))
        digest.update(struct.pack(">q", religion.pantheon_size))
        for ritual_id in sorted(religion.ritual_practices):
            digest.update(struct.pack(">q", ritual_id))
        digest.update(religion.cosmology.value.encode("utf-8"))
        digest.update(religion.eschatology.value.encode("utf-8"))
    for ritual in rituals:
        digest.update(struct.pack(">q", ritual.settlement_id))
        digest.update(struct.pack(">q", ritual.id))
        digest.update(ritual.ritual_type.value.encode("utf-8"))
        digest.update(struct.pack(">q", ritual.attested_from_step))
        until = ritual.attested_until_step if ritual.attested_until_step is not None else -1
        digest.update(struct.pack(">q", until))
    return digest.hexdigest()


def _make_event_id(
    seed: int,
    event_type: EventType,
    step: int,
    settlement_id: int,
    salt: str,
) -> str:
    """Deterministic 16-char hex event id via blake2b.

    Uses a distinct blake2b person namespace (`b"religion"`) so
    belief event ids cannot collide with demography (`b"worldfac"`)
    or culture (`b"culture"`) event ids."""
    digest = hashlib.blake2b(digest_size=8, person=b"religion")
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(event_type.value.encode("utf-8"))
    digest.update(struct.pack(">q", step))
    digest.update(struct.pack(">q", settlement_id))
    digest.update(salt.encode("utf-8"))
    return digest.hexdigest()


def _lookup_biome(
    biome_grid: tuple[tuple[BiomeClass, ...], ...],
    x: int,
    y: int,
) -> BiomeClass:
    """Look up the biome at (x, y). Defensive: returns GRASSLAND if
    the cell is out of bounds (settlements should always be
    in-bounds; this guards against future scale changes)."""
    if 0 <= y < len(biome_grid) and 0 <= x < len(biome_grid[y]):
        return biome_grid[y][x]
    return BiomeClass.GRASSLAND


def _compute_history_bucket(death_rate: float) -> str:
    """Bucket the recent-death-rate into low / mid / high per
    `RELIGION_DEATH_RATE_LOW_THRESHOLD` / `RELIGION_DEATH_RATE_HIGH_THRESHOLD`.

    Bucket thresholds are pinned absolutely so the 3b.5 chi-square
    acceptance test (arid vs tropical water-ritual frequency) is
    deterministic across seeds."""
    if death_rate < RELIGION_DEATH_RATE_LOW_THRESHOLD:
        return _HISTORY_BUCKET_LOW
    if death_rate < RELIGION_DEATH_RATE_HIGH_THRESHOLD:
        return _HISTORY_BUCKET_MID
    return _HISTORY_BUCKET_HIGH


def _compute_recent_death_rate(
    world: WorldModel,
    settlement_id: int,
    current_step: int,
    window: int,
) -> float:
    """Mean per-step death rate over the last `window` steps for
    `settlement_id`. Returns 0.0 if there is no death / population
    data (defensive — fresh worlds with no events)."""
    if window <= 0 or current_step <= 0:
        return 0.0
    populations: tuple[int, ...] | None = None
    for pool in world.demography.pools:
        if pool.settlement_id == settlement_id:
            populations = pool.populations
            break
    if populations is None:
        return 0.0
    step_start = max(0, current_step - window)
    death_counts: dict[int, int] = {}
    for event in world.demography.events:
        if event.type is not EventType.DEATH:
            continue
        if event.payload.get("settlement_id") != settlement_id:
            continue
        event_step = event.t
        if step_start < event_step <= current_step:
            death_counts[event_step] = death_counts.get(event_step, 0) + 1
    rates: list[float] = []
    for step in range(step_start + 1, current_step + 1):
        if step >= len(populations):
            break
        population = populations[step]
        if population <= 0:
            continue
        deaths = death_counts.get(step, 0)
        rates.append(deaths / population)
    if not rates:
        return 0.0
    return sum(rates) / len(rates)


def _initial_pantheon_size(
    seed: int,
    settlement_id: int,
    biome: BiomeClass,
) -> int:
    """Sample pantheon size from `RELIGION_BIOME_PANTHEON_RANGE[biome]`
    deterministically. The biome range is inclusive on both ends."""
    minimum, maximum = RELIGION_BIOME_PANTHEON_RANGE[biome.value]
    span = maximum - minimum + 1
    sample = sample_unit_interval(seed, "religion.pantheon", settlement_id)
    return minimum + int(sample * span)


def _sample_ritual_type(
    bias: tuple[float, ...],
    sample: float,
    excluded: set[RitualType],
) -> RitualType | None:
    """Sample a ritual from the remaining weighted ritual types."""
    available = [
        (ritual_type, weight)
        for ritual_type, weight in zip(RitualType, bias, strict=True)
        if ritual_type not in excluded
    ]
    total_weight = sum(weight for _, weight in available)
    if total_weight <= 0.0:
        return None
    target = sample * total_weight
    cumulative = 0.0
    for ritual_type, weight in available:
        cumulative += weight
        if target < cumulative:
            return ritual_type
    return available[-1][0]


def _initial_ritual_types(
    seed: int,
    settlement_id: int,
    biome: BiomeClass,
    count: int,
) -> tuple[RitualType, ...]:
    """Sample `count` ritual types from `RELIGION_BIOME_RITUAL_BIAS[biome]`
    without replacement (no duplicate rituals in the initial set)."""
    bias = RELIGION_BIOME_RITUAL_BIAS[biome.value]
    types: list[RitualType] = []
    used: set[RitualType] = set()
    for index in range(count):
        sample = sample_unit_interval(seed, "religion.init", settlement_id, index)
        chosen = _sample_ritual_type(bias, sample, used)
        if chosen is None:
            break
        types.append(chosen)
        used.add(chosen)
    return tuple(types)


def _initial_cosmology(
    seed: int,
    settlement_id: int,
    biome: BiomeClass,
) -> Cosmology:
    """Sample cosmology from `RELIGION_BIOME_COSMOLOGY_BIAS[biome]`.
    Stable across the simulation (structural element)."""
    bias = RELIGION_BIOME_COSMOLOGY_BIAS[biome.value]
    sample = sample_unit_interval(seed, "religion.cosmology", settlement_id)
    cumulative = 0.0
    for cosmology in Cosmology:
        cumulative += bias[cosmology.value]
        if sample < cumulative:
            return cosmology
    return Cosmology.CYCLE


def _initial_eschatology(
    seed: int,
    settlement_id: int,
    history_bucket: str,
) -> Eschatology:
    """Sample eschatology from `RELIGION_HISTORY_ESCHATOLOGY_BIAS[bucket]`.
    Stable across the simulation (structural element)."""
    bias = RELIGION_HISTORY_ESCHATOLOGY_BIAS[history_bucket]
    sample = sample_unit_interval(seed, "religion.eschatology", settlement_id)
    cumulative = 0.0
    for eschatology in Eschatology:
        cumulative += bias[eschatology.value]
        if sample < cumulative:
            return eschatology
    return Eschatology.RENEWAL


def _sample_add_ritual_type(
    seed: int,
    settlement_id: int,
    step: int,
    biome: BiomeClass,
    excluded: set[RitualType],
) -> RitualType | None:
    """Sample a ritual type from `RELIGION_BIOME_RITUAL_BIAS[biome]` for
    an add step, excluding any types already in `excluded`. Returns
    None when every biome-allowed type is already present (caller
    treats this as 'no add possible')."""
    bias = RELIGION_BIOME_RITUAL_BIAS[biome.value]
    sample = sample_unit_interval(seed, "religion.add_type", settlement_id, step)
    return _sample_ritual_type(bias, sample, excluded)


def _emit_belief_event(
    seed: int,
    settlement_id: int,
    settlement_x: int,
    settlement_y: int,
    ritual_added: int | None,
    ritual_removed: int | None,
    step: int,
    provenance: ProvenanceRecord,
) -> WorldEvent:
    """Build a single BELIEF event for a ritual add or remove."""
    salt = f"add:{ritual_added}" if ritual_added is not None else f"remove:{ritual_removed}"
    return WorldEvent(
        id=_make_event_id(seed, EventType.BELIEF, step, settlement_id, salt),
        type=EventType.BELIEF,
        t=step,
        location=EventLocation(
            cell=(settlement_x, settlement_y),
            settlement_id=settlement_id,
        ),
        actors=(
            EventActor(
                kind="religion",
                identifier=f"religion:{settlement_id}",
                display_name=None,
            ),
        ),
        payload=BeliefPayload(
            settlement_id=settlement_id,
            ritual_added=ritual_added,
            ritual_removed=ritual_removed,
            step=step,
        ).model_dump(mode="python"),
        causes=(),
        provenance=provenance,
    )


def build_religion(
    world: WorldModel,
    time_steps: int = RELIGION_DRIFT_TIME_STEPS,
) -> tuple[ReligionLayer, tuple[WorldEvent, ...]]:
    """Build the top-level ReligionLayer from the world's settlements.

    One religion per settlement (parallel to 3b.1 cultures). Initial
    pantheon size is sampled from the biome range; initial ritual
    set is `RELIGION_INITIAL_RITUAL_COUNT_MIN..MAX` distinct ritual
    types drawn from `RELIGION_BIOME_RITUAL_BIAS[biome]`; cosmology
    and eschatology are sampled from biome / history-bucket bias
    tables and held stable. Per-step drift may add / remove one
    ritual at probability `RELIGION_RITUAL_DRIFT_RATE`; each change
    emits one BELIEF event with the Ritual id.

    Returns `(ReligionLayer, tuple[WorldEvent, ...])`: the layer
    and the emitted belief events. The generator merges the belief
    events with demography + culture events per-step and re-runs
    `build_event_log` so the top-level `EventLog` carries the
    full history.

    Determinism: identical seeds produce byte-equivalent output.
    RNG calls are namespaced as `"religion.init"` (initial ritual
    draws), `"religion.pantheon"` (initial pantheon size),
    `"religion.cosmology"` (initial cosmology),
    `"religion.eschatology"` (initial eschatology),
    `"religion.drift"` (per-step drift decisions),
    `"religion.add_type"` (per-step ritual-type selection)."""
    seed = world.metadata.config.seed
    settlements = world.settlements.settlements
    biome_grid = world.biomes.classifications
    provenance = religion_provenance()
    sorted_settlements = sorted(settlements, key=lambda s: s.id)

    religions: list[Religion] = []
    rituals: dict[int, Ritual] = {}
    next_ritual_id = 0
    events: list[WorldEvent] = []

    for settlement in sorted_settlements:
        biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
        initial_count = RELIGION_INITIAL_RITUAL_COUNT_MIN + int(
            sample_unit_interval(seed, "religion.init_count", settlement.id)
            * (RELIGION_INITIAL_RITUAL_COUNT_MAX - RELIGION_INITIAL_RITUAL_COUNT_MIN + 1)
        )
        initial_types = _initial_ritual_types(seed, settlement.id, biome, initial_count)
        initial_ritual_ids: list[int] = []
        for ritual_type in initial_types:
            ritual = Ritual(
                id=next_ritual_id,
                settlement_id=settlement.id,
                ritual_type=ritual_type,
                attested_from_step=0,
                attested_until_step=None,
            )
            rituals[next_ritual_id] = ritual
            initial_ritual_ids.append(next_ritual_id)
            next_ritual_id += 1
        pantheon_size = _initial_pantheon_size(seed, settlement.id, biome)
        initial_death_rate = _compute_recent_death_rate(
            world, settlement.id, time_steps, RELIGION_PRESSURE_WINDOW_STEPS
        )
        initial_history_bucket = _compute_history_bucket(initial_death_rate)
        cosmology = _initial_cosmology(seed, settlement.id, biome)
        eschatology = _initial_eschatology(seed, settlement.id, initial_history_bucket)
        religions.append(
            Religion(
                settlement_id=settlement.id,
                pantheon_size=pantheon_size,
                ritual_practices=tuple(sorted(initial_ritual_ids)),
                cosmology=cosmology,
                eschatology=eschatology,
            )
        )

    for step in range(1, time_steps + 1):
        next_religion_practices: list[tuple[int, ...]] = [
            religion.ritual_practices for religion in religions
        ]
        for index, religion in enumerate(religions):
            settlement = sorted_settlements[index]
            current_practices = religion.ritual_practices
            drift_sample = sample_unit_interval(
                seed, "religion.drift", religion.settlement_id, step
            )
            if drift_sample >= RELIGION_RITUAL_DRIFT_RATE:
                continue
            decision_sample = sample_unit_interval(
                seed,
                "religion.drift",
                religion.settlement_id,
                step,
                1,
            )
            should_remove = decision_sample < 0.5 and len(current_practices) > 0
            if should_remove:
                removal_index = int(
                    sample_unit_interval(
                        seed,
                        "religion.drift",
                        religion.settlement_id,
                        step,
                        2,
                    )
                    * len(current_practices)
                )
                removal_index = min(removal_index, len(current_practices) - 1)
                removed_id = current_practices[removal_index]
                old_ritual = rituals[removed_id]
                rituals[removed_id] = old_ritual.model_copy(update={"attested_until_step": step})
                next_religion_practices[index] = tuple(
                    ritual_id for ritual_id in current_practices if ritual_id != removed_id
                )
                events.append(
                    _emit_belief_event(
                        seed,
                        religion.settlement_id,
                        settlement.x,
                        settlement.y,
                        ritual_added=None,
                        ritual_removed=removed_id,
                        step=step,
                        provenance=provenance,
                    )
                )
            else:
                current_types = {rituals[ritual_id].ritual_type for ritual_id in current_practices}
                biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
                added_type = _sample_add_ritual_type(
                    seed,
                    religion.settlement_id,
                    step,
                    biome,
                    current_types,
                )
                if added_type is None:
                    continue
                ritual = Ritual(
                    id=next_ritual_id,
                    settlement_id=religion.settlement_id,
                    ritual_type=added_type,
                    attested_from_step=step,
                    attested_until_step=None,
                )
                rituals[next_ritual_id] = ritual
                next_religion_practices[index] = tuple(
                    sorted(current_practices + (next_ritual_id,))
                )
                events.append(
                    _emit_belief_event(
                        seed,
                        religion.settlement_id,
                        settlement.x,
                        settlement.y,
                        ritual_added=next_ritual_id,
                        ritual_removed=None,
                        step=step,
                        provenance=provenance,
                    )
                )
                next_ritual_id += 1
        religions = [
            Religion(
                settlement_id=religion.settlement_id,
                pantheon_size=religion.pantheon_size,
                ritual_practices=next_religion_practices[index],
                cosmology=religion.cosmology,
                eschatology=religion.eschatology,
            )
            for index, religion in enumerate(religions)
        ]

    sorted_rituals = tuple(
        sorted(
            rituals.values(),
            key=lambda ritual: (
                ritual.settlement_id,
                ritual.attested_from_step,
                ritual.id,
            ),
        )
    )
    algorithm_version = _compute_algorithm_version(tuple(religions), sorted_rituals)
    return (
        ReligionLayer(
            religions=tuple(religions),
            rituals=sorted_rituals,
            algorithm_version=algorithm_version,
        ),
        tuple(events),
    )


def validate_religion_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3b.2 religion-layer invariants.

    Checks:
    - `algorithm_version` matches a fresh blake2b of `(religions,
      rituals)` (catches silent mutation / re-ordering at the trust
      boundary).
    - `religions` is parallel to `SettlementsLayer.settlements` by
      id (same length, same order).
    - Every `Religion.ritual_practices` references a Ritual.id that
      exists in `world.religions.rituals` and that Ritual belongs to
      the same settlement.
    - Every ritual in `world.religions.rituals` is currently attested
      (i.e. its id appears in some `Religion.ritual_practices`) OR
      has a non-None `attested_until_step`.
    - Pantheon size is in the biome's range
      (`RELIGION_BIOME_PANTHEON_RANGE`).
    - `rituals` is sorted by `(settlement_id, attested_from_step, id)`.

    The algorithm-version check runs first so a length-mutated layer
    still surfaces the mutation. Per-religion invariants accumulate
    without bailing on the first length mismatch."""
    violations: list[InvariantViolation] = []
    settlements = world.settlements.settlements
    layer = world.religions
    biome_grid = world.biomes.classifications

    expected_version = _compute_algorithm_version(layer.religions, layer.rituals)
    if layer.algorithm_version != expected_version:
        violations.append(
            _violation(
                "religion-layer-algorithm-version-mismatch",
                "religions.algorithm_version",
                (
                    f"religion algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected_version!r}; layer was "
                    f"mutated or re-ordered outside the generator"
                ),
            )
        )

    if len(layer.religions) != len(settlements):
        violations.append(
            _violation(
                "religion-layer-length-mismatch",
                "religions.religions",
                (
                    f"religion layer ({len(layer.religions)}) does not "
                    f"match settlements ({len(settlements)})"
                ),
            )
        )
    last_valid_index = min(len(layer.religions), len(settlements))
    rituals_by_id: dict[int, Ritual] = {}
    for ritual_index, ritual in enumerate(layer.rituals):
        if ritual.id in rituals_by_id:
            violations.append(
                _violation(
                    "religion-ritual-id-duplicate",
                    f"religions.rituals.{ritual_index}.id",
                    f"ritual id {ritual.id} appears more than once",
                )
            )
        rituals_by_id[ritual.id] = ritual

    referenced_ritual_ids: set[int] = set()
    for index in range(last_valid_index):
        religion = layer.religions[index]
        settlement = settlements[index]
        if religion.settlement_id != settlement.id:
            violations.append(
                _violation(
                    "religion-layer-settlement-id-mismatch",
                    f"religions.religions.{index}.settlement_id",
                    (
                        f"religion {index} references "
                        f"settlement_id={religion.settlement_id} but "
                        f"settlements.{index}.id={settlement.id}"
                    ),
                )
            )
        biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
        minimum, maximum = RELIGION_BIOME_PANTHEON_RANGE[biome.value]
        if not (minimum <= religion.pantheon_size <= maximum):
            violations.append(
                _violation(
                    "religion-pantheon-size-out-of-range",
                    f"religions.religions.{index}.pantheon_size",
                    (
                        f"religion {index} pantheon_size "
                        f"{religion.pantheon_size} is outside "
                        f"biome {biome.value} range "
                        f"[{minimum}, {maximum}]"
                    ),
                )
            )
        for ritual_index, ritual_id in enumerate(religion.ritual_practices):
            path = f"religions.religions.{index}.ritual_practices.{ritual_index}"
            current_ritual = rituals_by_id.get(ritual_id)
            if current_ritual is None:
                violations.append(
                    _violation(
                        "religion-ritual-id-missing",
                        path,
                        (
                            f"religion {index} references ritual_id "
                            f"{ritual_id} not present in religions.rituals"
                        ),
                    )
                )
            elif current_ritual.settlement_id != religion.settlement_id:
                violations.append(
                    _violation(
                        "religion-ritual-settlement-mismatch",
                        path,
                        (
                            f"ritual {ritual_id} belongs to settlement "
                            f"{current_ritual.settlement_id}, not religion settlement "
                            f"{religion.settlement_id}"
                        ),
                    )
                )
            elif current_ritual.attested_until_step is not None:
                violations.append(
                    _violation(
                        "religion-retired-ritual-referenced",
                        path,
                        (
                            f"ritual {ritual_id} retired at step "
                            f"{current_ritual.attested_until_step} but remains active"
                        ),
                    )
                )
            referenced_ritual_ids.add(ritual_id)
    for index in range(last_valid_index, len(layer.religions)):
        violations.append(
            _violation(
                "religion-layer-surplus",
                f"religions.religions.{index}",
                (
                    f"religion {index} has no parallel settlement "
                    f"(settlements has {len(settlements)} entries; "
                    f"religions has {len(layer.religions)})"
                ),
            )
        )
    for ritual in layer.rituals:
        if ritual.id not in referenced_ritual_ids and ritual.attested_until_step is None:
            violations.append(
                _violation(
                    "religion-ritual-orphaned",
                    f"religions.rituals.{ritual.id}",
                    (
                        f"ritual {ritual.id} (settlement "
                        f"{ritual.settlement_id}, type "
                        f"{ritual.ritual_type.value}) is not "
                        f"referenced by any religion's "
                        f"ritual_practices and has no "
                        f"attested_until_step — should have "
                        f"been retired"
                    ),
                )
            )
    rituals_sorted = tuple(
        sorted(
            layer.rituals,
            key=lambda ritual: (
                ritual.settlement_id,
                ritual.attested_from_step,
                ritual.id,
            ),
        )
    )
    if rituals_sorted != layer.rituals:
        violations.append(
            _violation(
                "religion-rituals-not-sorted",
                "religions.rituals",
                (
                    "rituals tuple is not sorted by "
                    "(settlement_id, attested_from_step, id); "
                    "generator must sort before persisting"
                ),
            )
        )
    return violations


def religion_provenance() -> ProvenanceRecord:
    """Provenance record describing the religion-layer builder."""
    return ProvenanceRecord(
        output_path="religions",
        process=("biome-and-history-biased-pantheon-with-low-probability-ritual-drift"),
        input_paths=(
            "settlements.settlements",
            "biomes.classifications",
            "demography.events",
            "demography.pools",
            "metadata.config.seed",
        ),
        algorithm_version=RELIGION_ALGORITHM_VERSION,
    )


__all__ = [
    "build_religion",
    "religion_provenance",
    "validate_religion_layer",
]
