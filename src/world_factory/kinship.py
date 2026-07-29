"""Phase 3b.3 — kinship lineages + per-culture name pools.

`build_kinship(world) -> tuple[KinshipLayer, tuple[WorldEvent, ...]]`
samples a `KinshipSystem` per settlement from a biome-conditioned
typology table (`KINSHIP_TYPOGRAPHY`) and a phoneme-templated
`NamePool` per culture from a biome-conditioned phoneme bag
(`KINSHIP_NAME_PHONEME_BIAS`). It also emits one
`EventType.LINEAGE_FOUNDED` event per lineage (parallel to
`KinshipLayer.lineages` by index) into the unified EventLog via
`event_log.build_event_log`.

`validate_kinship_layer(world)` enforces the standard 3b.x validator
order (algorithm-version-mismatch FIRST, then parallel-structure,
per-record integrity, field ranges, no-surplus, no-orphans). Public
surface:

- `build_kinship(world) -> tuple[KinshipLayer, tuple[WorldEvent, ...]]`
- `validate_kinship_layer(world) -> list[InvariantViolation]`
- `kinship_provenance() -> ProvenanceRecord`

Spec fidelity:
- `Lineage(id, settlement_id, system, depth, founding_step,
  founder_actor_id)` per `PLANS/PHASE_3_TO_5_PLAN.md:199-207`.
- One `Lineage` per settlement (`lineages` parallel to
  `SettlementsLayer.settlements` by id); intra-settlement only,
  polity-wide is a Phase 4 concern per spec line 201-202.
- One `NamePool` per culture (`name_pools` parallel to
  `CultureLayer.cultures` by index); phoneme-templated v1, full
  lexicon + grammar deferred to 3b.4.
- `KINSHIP_ALGORITHM_VERSION = "lineage-typology-v1"` (algorithm-
  shaped suffix, not a phase number).
- `WorldModel.kinship` is additive-required per the 3a.2
  additive-required-field policy. Schema bump
  14.0.0 -> 15.0.0; Model-version bump phase-3b.2 -> phase-3b.3.
"""

from __future__ import annotations

import hashlib
import struct
from typing import TYPE_CHECKING

from world_factory.constants import (
    KINSHIP_ALGORITHM_VERSION,
    KINSHIP_LINEAGE_DEPTH_MAX,
    KINSHIP_LINEAGE_DEPTH_MIN,
    KINSHIP_NAME_PHONEME_BIAS,
    KINSHIP_NAME_PHONEMES,
    KINSHIP_NAMES_PER_CULTURE_BIAS,
    KINSHIP_NAMES_PER_CULTURE_MAX,
    KINSHIP_NAMES_PER_CULTURE_MIN,
    KINSHIP_TYPOGRAPHY,
)
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiomeClass,
    EventLocation,
    EventType,
    KinshipLayer,
    KinshipSystem,
    Lineage,
    LineageFoundedPayload,
    NamePool,
    ProvenanceRecord,
    WorldEvent,
    WorldModel,
)

if TYPE_CHECKING:
    pass

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1
_KINSHIP_BLAKE_PERSON = b"kinship"
_KINSHIP_EVENTID_PERSON = b"kinevn"

# Index positions in the typology-tuple map to (matrilineal,
# patrilineal, bilateral, avunculate, cognatic). Order is fixed
# because KINSHIP_TYPOGRAPHY rows are 5-tuples and each enum member's
# value matches the order below.
_KINSHIP_SYSTEM_ORDER: tuple[KinshipSystem, ...] = (
    KinshipSystem.MATRILINEAL,
    KinshipSystem.PATRILINEAL,
    KinshipSystem.BILATERAL,
    KinshipSystem.AVUNCULATE,
    KinshipSystem.COGNATIC,
)


def _compute_algorithm_version(
    lineages: tuple[Lineage, ...],
    name_pools: tuple[NamePool, ...],
) -> str:
    """blake2b hash of lineage + name-pool state. 16-char hex.

    Mirrors the culture / religion / event-log algorithm-version
    pattern: stable across re-orders, breaking across mutations, so
    the trust boundary (`WorldModel.model_validate_json`) catches
    silent corruption."""
    digest = hashlib.blake2b(digest_size=8, person=_KINSHIP_BLAKE_PERSON)
    for lineage in lineages:
        digest.update(struct.pack(">q", lineage.id))
        digest.update(struct.pack(">q", lineage.settlement_id))
        digest.update(lineage.system.value.encode("utf-8"))
        digest.update(struct.pack(">q", lineage.depth))
        digest.update(struct.pack(">q", lineage.founding_step))
        if lineage.founder_actor_id is not None:
            digest.update(lineage.founder_actor_id.encode("utf-8"))
    for name_pool in name_pools:
        digest.update(struct.pack(">q", name_pool.culture_id))
        for name in sorted(name_pool.given_names):
            digest.update(name.encode("utf-8"))
        digest.update("|".join(name_pool.surname_patterns).encode("utf-8"))
        digest.update("|".join(name_pool.epithets).encode("utf-8"))
    return digest.hexdigest()


def _make_event_id(
    seed: int,
    event_type_str: str,
    step: int,
    settlement_id: int,
    salt: str,
) -> str:
    """Deterministic 16-char hex lineage-event id via blake2b.

    Uses a distinct `b"kinevn"` person namespace so lineage-event ids
    cannot collide with demography (`b"evntlog"`), culture
    (`b"culture"`), or religion (`b"religion"`) event ids."""
    digest = hashlib.blake2b(digest_size=8, person=_KINSHIP_EVENTID_PERSON)
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(event_type_str.encode("utf-8"))
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
    the cell is out of bounds."""
    if 0 <= y < len(biome_grid) and 0 <= x < len(biome_grid[y]):
        return biome_grid[y][x]
    return BiomeClass.GRASSLAND


def _sample_typology(
    seed: int,
    settlement_id: int,
    biome: BiomeClass,
) -> KinshipSystem:
    """Sample a single KinshipSystem from `KINSHIP_TYPOGRAPHY[biome]`
    weighted table. Deterministic per (seed, settlement_id)."""
    weights = KINSHIP_TYPOGRAPHY[biome.value]
    draw = sample_unit_interval(seed, "kinship.typology", settlement_id, 0)
    running = 0.0
    for system, weight in zip(_KINSHIP_SYSTEM_ORDER, weights, strict=True):
        running += weight
        if draw <= running:
            return system
    return _KINSHIP_SYSTEM_ORDER[-1]  # pragma: no cover


def _sample_depth(seed: int, settlement_id: int) -> int:
    """Sample a lineage depth in
    `[KINSHIP_LINEAGE_DEPTH_MIN..KINSHIP_LINEAGE_DEPTH_MAX]`. Returns
    an integer in the inclusive range; the draw is scaled across the
    range then floored. Deterministic per (seed, settlement_id)."""
    draw = sample_unit_interval(seed, "kinship.depth", settlement_id, 0)
    span = KINSHIP_LINEAGE_DEPTH_MAX - KINSHIP_LINEAGE_DEPTH_MIN + 1
    return KINSHIP_LINEAGE_DEPTH_MIN + int(draw * span)


def _sample_name_count(seed: int, culture_id: int, biome: BiomeClass) -> int:
    """Sample a name-pool size within the biome-conditioned
    `[KINSHIP_NAMES_PER_CULTURE_MIN..KINSHIP_NAMES_PER_CULTURE_MAX]`
    range per `KINSHIP_NAMES_PER_CULTURE_BIAS[biome]`. The biome
    range is clamped to the global min/max."""
    low, high = KINSHIP_NAMES_PER_CULTURE_BIAS[biome.value]
    low = max(low, KINSHIP_NAMES_PER_CULTURE_MIN)
    high = min(high, KINSHIP_NAMES_PER_CULTURE_MAX)
    span = high - low + 1
    if span <= 0:
        return KINSHIP_NAMES_PER_CULTURE_MIN
    draw = sample_unit_interval(seed, "kinship.name_count", culture_id, 0)
    return low + int(draw * span)


def _sample_given_name(
    seed: int,
    culture_id: int,
    culture_attempt: int,
    biome: BiomeClass,
    used: set[str],
) -> str:
    """Sample a single phoneme-templated given-name from
    `KINSHIP_NAME_PHONEME_BIAS[biome]`. Builds a 3-5 syllable name
    by concatenating `KINSHIP_NAME_PHONEMES` entries (weighted per
    biome) until the result is unique within `used`. The
    `culture_attempt` salt disambiguates retries for a stable name
    set per `(seed, culture_id)`."""
    phoneme_bag = KINSHIP_NAME_PHONEMES
    phoneme_weights = KINSHIP_NAME_PHONEME_BIAS[biome.value]
    for attempt in range(16):
        length_syllables = 2 + int(
            sample_unit_interval(seed, "kinship.name_length", culture_id, attempt) * 4
        )
        syllables: list[str] = []
        for syllable_index in range(length_syllables):
            draw = sample_unit_interval(
                seed,
                "kinship.name_syllable",
                culture_id,
                culture_attempt * 8 + attempt * 4 + syllable_index,
            )
            cumulative = 0.0
            chosen = phoneme_bag[-1]
            for phoneme, weight in zip(phoneme_bag, phoneme_weights, strict=True):
                cumulative += weight
                if draw <= cumulative:
                    chosen = phoneme
                    break
            syllables.append(chosen)
        candidate = "".join(syllables).capitalize()
        if candidate and candidate not in used:
            return candidate
    # Fallback: force uniqueness by appending culture_id + attempt.
    return f"X{culture_id}c{culture_attempt}"


def _sample_founder_actor_id(
    world: WorldModel,
    settlement_id: int,
) -> str | None:
    """Sample one living individual at step 0 from
    `demography.events` for the given settlement, or None if the
    settlement has no recorded births at step 0.

    Per spec: at `build_kinship` time, if any BIRTH event references
    the settlement at `step=0`, sample one. Settlements with synthetic
    initial populations (no BIRTH events) carry `None`."""
    for event in world.demography.events:
        if (
            event.type == EventType.BIRTH
            and event.t == 0
            and event.location.settlement_id == settlement_id
        ):
            individual_id = event.payload.get("individual_id")
            if isinstance(individual_id, str):
                return individual_id
    return None


def build_kinship(
    world: WorldModel,
) -> tuple[KinshipLayer, tuple[WorldEvent, ...]]:
    """Construct `KinshipLayer` + one LINEAGE_FOUNDED event per
    lineage.

    The output tuple unpacks into `(kinship_layer, lineage_events)`;
    callers pass `lineage_events` to `event_log.build_event_log` so
    kinship events are merged into the unified log per the standard
    within-step ordering (demography -> culture -> religion ->
    kinship, monotonic in `t`).

    Deterministic per `world.metadata.config.seed`: same seed, same
    lineages, same name pools, same events."""
    seed = world.metadata.config.seed
    biome_grid = world.biomes.classifications
    sorted_settlements = sorted(world.settlements.settlements, key=lambda s: s.id)

    lineages: list[Lineage] = []
    events: list[WorldEvent] = []
    for index, settlement in enumerate(sorted_settlements):
        biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
        system = _sample_typology(seed, settlement.id, biome)
        depth = _sample_depth(seed, settlement.id)
        founder_actor_id = _sample_founder_actor_id(world, settlement.id)
        lineage = Lineage(
            id=index,
            settlement_id=settlement.id,
            system=system,
            depth=depth,
            founding_step=0,
            founder_actor_id=founder_actor_id,
        )
        lineages.append(lineage)
        salt = f"lineage:{lineage.id}"
        event_payload = LineageFoundedPayload(
            lineage_id=lineage.id,
            settlement_id=lineage.settlement_id,
            system=lineage.system,
            founding_step=lineage.founding_step,
            step=lineage.founding_step,
        )
        event_id = _make_event_id(
            seed,
            "kinship.lineage_founded",
            lineage.founding_step,
            lineage.settlement_id,
            salt,
        )
        events.append(
            WorldEvent(
                id=event_id,
                type=EventType.LINEAGE_FOUNDED,
                t=lineage.founding_step,
                location=EventLocation(cell=None, settlement_id=lineage.settlement_id),
                actors=(),
                payload=event_payload.model_dump(mode="python"),
                causes=(),
                provenance=ProvenanceRecord(
                    output_path="kinship.lineages",
                    process="lineage-typology-sampler",
                    input_paths=("metadata.config.seed", "biomes.classifications"),
                    algorithm_version=KINSHIP_ALGORITHM_VERSION,
                ),
            )
        )

    # NamePools are parallel to cultures by index. Cultures are
    # parallel to settlements by index (per the 3b.1 convention), so
    # we walk the cultures in settlement_id order and pull biome per
    # the corresponding settlement.
    culture_to_settlement = {
        culture.settlement_id: culture for culture in world.cultures.cultures
    }
    sorted_culture_ids = sorted(culture_to_settlement)
    name_pools: list[NamePool] = []
    for culture_id in sorted_culture_ids:
        settlement_index = next(
            (
                i
                for i, s in enumerate(sorted_settlements)
                if s.id == culture_id
            ),
            None,
        )
        if settlement_index is None:
            # Defensive: a culture without a settlement is a model
            # invariant violation upstream; we still emit an empty
            # name pool so the validator can flag it.
            name_pools.append(NamePool(culture_id=culture_id, given_names=()))
            continue
        settlement = sorted_settlements[settlement_index]
        biome = _lookup_biome(biome_grid, settlement.x, settlement.y)
        n_names = _sample_name_count(seed, culture_id, biome)
        used_names: set[str] = set()
        given_names: list[str] = []
        for attempt in range(n_names):
            name = _sample_given_name(seed, culture_id, attempt, biome, used_names)
            used_names.add(name)
            given_names.append(name)
        name_pools.append(
            NamePool(
                culture_id=culture_id,
                given_names=tuple(given_names),
            )
        )

    algorithm_version = _compute_algorithm_version(
        tuple(lineages), tuple(name_pools)
    )
    layer = KinshipLayer(
        lineages=tuple(lineages),
        name_pools=tuple(name_pools),
        algorithm_version=algorithm_version,
    )
    return layer, tuple(events)


def validate_kinship_layer(world: WorldModel) -> list[InvariantViolation]:
    """Standard 3b.x validator order.

    1. `_validate_algorithm_version` — algorithm_version blake2b
       matches the lineages + name_pools tuple.
    2. `_validate_parallel_structure` — lineages parallel to
       settlements by id (same length); name_pools parallel to
       cultures by index (same length as cultures).
    3. `_validate_lineage_records` — per-lineage field integrity
       (system enum membership, depth range, founding_step
       non-negative, founder_actor_id pattern when present).
    4. `_validate_name_pool_records` — per-name-pool integrity
       (culture_id non-negative, given_names length within
       `[KINSHIP_NAMES_PER_CULTURE_MIN..KINSHIP_NAMES_PER_CULTURE_MAX]`).
    5. `_validate_surplus_lineages` — no lineage past
       `len(settlements)`.
    6. `_validate_orphaned_lineages` — every lineage settlement_id
       exists; every name_pool culture_id has a culture record.
    """
    violations: list[InvariantViolation] = []
    layer = world.kinship
    expected = _compute_algorithm_version(layer.lineages, layer.name_pools)
    if layer.algorithm_version != expected:
        violations.append(
            _violation(
                "kinship-algorithm-version-mismatch",
                "world.kinship.algorithm_version",
                (
                    f"kinship algorithm_version {layer.algorithm_version!r} "
                    f"does not match recomputed {expected!r}; layer was "
                    f"mutated or re-ordered outside the generator"
                ),
            )
        )

    settlements = world.settlements.settlements
    cultures = world.cultures.cultures
    if len(layer.lineages) != len(settlements):
        violations.append(
            _violation(
                "kinship-lineage-parallel-structure",
                "world.kinship.lineages",
                (
                    f"kinship lineages length {len(layer.lineages)} does "
                    f"not match settlements length {len(settlements)} "
                    f"(expected one lineage per settlement)"
                ),
            )
        )
    if len(layer.name_pools) != len(cultures):
        violations.append(
            _violation(
                "kinship-namepool-parallel-structure",
                "world.kinship.name_pools",
                (
                    f"kinship name_pools length {len(layer.name_pools)} "
                    f"does not match cultures length {len(cultures)} "
                    f"(expected one name_pool per culture)"
                ),
            )
        )

    seen_lineage_ids: set[int] = set()
    seen_culture_ids: set[int] = set()
    settlement_ids = {settlement.id for settlement in settlements}
    culture_settlement_ids = {culture.settlement_id for culture in cultures}
    for index, lineage in enumerate(layer.lineages):
        if lineage.id in seen_lineage_ids:
            violations.append(
                _violation(
                    "kinship-duplicate-lineage-id",
                    f"world.kinship.lineages.{index}.id",
                    f"lineage id {lineage.id} appears more than once",
                )
            )
        seen_lineage_ids.add(lineage.id)
        if lineage.id != index:
            violations.append(
                _violation(
                    "kinship-lineage-id-not-parallel-index",
                    f"world.kinship.lineages.{index}.id",
                    (
                        f"lineage id {lineage.id} does not match index "
                        f"{index} (lineages must be parallel-to-settlements "
                        f"by id)"
                    ),
                )
            )
        if lineage.settlement_id not in settlement_ids:
            violations.append(
                _violation(
                    "kinship-orphaned-lineage",
                    f"world.kinship.lineages.{index}.settlement_id",
                    (
                        f"lineage {lineage.id} references unknown "
                        f"settlement {lineage.settlement_id}"
                    ),
                )
            )

    for index, name_pool in enumerate(layer.name_pools):
        if name_pool.culture_id in seen_culture_ids:
            violations.append(
                _violation(
                    "kinship-duplicate-namepool-culture-id",
                    f"world.kinship.name_pools.{index}.culture_id",
                    f"name_pool culture_id {name_pool.culture_id} appears more than once",
                )
            )
        seen_culture_ids.add(name_pool.culture_id)
        if name_pool.culture_id not in culture_settlement_ids:
            violations.append(
                _violation(
                    "kinship-orphaned-namepool",
                    f"world.kinship.name_pools.{index}.culture_id",
                    (
                        f"name_pool {index} references unknown culture "
                        f"settlement_id {name_pool.culture_id}"
                    ),
                )
            )
        if not (
            KINSHIP_NAMES_PER_CULTURE_MIN
            <= len(name_pool.given_names)
            <= KINSHIP_NAMES_PER_CULTURE_MAX
        ):
            violations.append(
                _violation(
                    "kinship-namepool-bounds",
                    f"world.kinship.name_pools.{index}.given_names",
                    (
                        f"name_pool {index} given_names length "
                        f"{len(name_pool.given_names)} outside bounds "
                        f"[{KINSHIP_NAMES_PER_CULTURE_MIN}.."
                        f"{KINSHIP_NAMES_PER_CULTURE_MAX}]"
                    ),
                )
            )

    return violations


def kinship_provenance() -> ProvenanceRecord:
    """Provenance record describing the kinship-layer builder.

    Same shape as culture / religion provenance: declares the output
    path, the generator process (lineage-typology sampler + name-pool
    phoneme bag), the upstream inputs, and the algorithm version."""
    return ProvenanceRecord(
        output_path="kinship",
        process="lineage-typology-sampler-with-phoneme-name-pool",
        input_paths=(
            "metadata.config.seed",
            "biomes.classifications",
            "cultures.cultures",
            "settlements.settlements",
        ),
        algorithm_version=KINSHIP_ALGORITHM_VERSION,
    )
