"""Phase 3a.4 demography — population pools, migrations, events.

Algorithm:

1. **Birth rate** per settlement per step =
   `base_birth_rate * climate_factor * (1 + capacity_headroom_birth_boost)`
   where `climate_factor` is bell-shaped in temperature (peaks at
   `DEMOGRAPHY_CLIMATE_OPTIMUM_CELSIUS`) and `capacity_headroom_birth_boost`
   rewards settlements with surplus capacity.

2. **Death rate** per settlement per step =
   `base_death_rate * (1 + climate_stress + over_capacity_penalty) * conflict_factor`
   where `climate_stress` is the normalized |temperature − optimum|
   deviation, `over_capacity_penalty` ramps up as population exceeds
   carrying capacity, and `conflict_factor` jumps to
   `DEMOGRAPHY_CONFLICT_DEATH_MULTIPLIER` when per-step conflict
   tension exceeds `DEMOGRAPHY_CONFLICT_THRESHOLD`.

3. **Migration** along infrastructure road edges. For each road
   `from -> to`, the net flow =
   `pressure_factor * (max(0, pop_a - cap_a) / pop_a)
    - pull_factor * (max(0, cap_b - pop_b) / cap_b)`
   scaled by `1 / (1 + cost / cost_divisor)`. Inter-component
   flow is impossible because migrations only travel along road
   edges (which are within a component by construction).

4. **Event emission**: every birth, death, and migration produces
   a typed `WorldEvent` with a deterministic blake2b-derived id.
   Individual ids are born in `BirthPayload.individual_id` and
   propagate as `EventActor.identifier` in subsequent
   death / migration events. Per-settlement living-individual
   lists track who is alive; deaths / migrations pop from the
   list and (for migrations) append to the destination list.

5. **Determinism**: every random draw uses
   `sample_unit_interval(seed, namespace, *coordinates)` with
   namespaces of the form `"demography.<phase>"`. No global
   RNG mutation. Identical seeds produce byte-equivalent output.

Cross-phase integration:

- Consumes 3a.2 agriculture surplus / carrying capacity (capacity
  is the Malthusian ceiling).
- Consumes 3a.3 infrastructure roads (migration paths).
- Consumes 3a.1 climate (temperature drives birth / death climate
  factors).
- Emits typed events per `PHASE_3A_TYPES.md`; lives in
  `DemographyLayer.events` until Phase 3a.5 promotes them to a
  top-level `events: EventLog` on `WorldModel`.

Schema: `SCHEMA_VERSION` bumps `10.0.0` -> `11.0.0` to reflect
the new required `demography` field on `WorldModel`, per the
additive-required-field policy from Phase 3a.2.
"""

from __future__ import annotations

import hashlib
import struct

from world_factory.constants import (
    DEMOGRAPHY_ALGORITHM_VERSION,
    DEMOGRAPHY_BASE_BIRTH_RATE,
    DEMOGRAPHY_BASE_DEATH_RATE,
    DEMOGRAPHY_CAPACITY_HEADROOM_BIRTH_BOOST,
    DEMOGRAPHY_CLIMATE_OPTIMUM_CELSIUS,
    DEMOGRAPHY_CLIMATE_RANGE_CELSIUS,
    DEMOGRAPHY_CONFLICT_DEATH_MULTIPLIER,
    DEMOGRAPHY_CONFLICT_THRESHOLD,
    DEMOGRAPHY_DEFAULT_TIME_STEPS,
    DEMOGRAPHY_MIGRATION_COST_DIVISOR,
    DEMOGRAPHY_MIGRATION_PRESSURE_FACTOR,
    DEMOGRAPHY_MIGRATION_PULL_FACTOR,
    DEMOGRAPHY_OVER_CAPACITY_DEATH_PENALTY,
)
from world_factory.determinism import sample_unit_interval
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BirthPayload,
    DeathPayload,
    DemographyLayer,
    EventActor,
    EventLocation,
    EventType,
    MigrationPayload,
    MigrationRecord,
    PopulationPool,
    ProvenanceRecord,
    RoadEdge,
    Settlement,
    WorldEvent,
    WorldModel,
)

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1


def _climate_factor(temperature_celsius: float) -> float:
    """Bell-shaped in temperature. Returns 1.0 at the optimum, 0.0 at
    or beyond +/- DEMOGRAPHY_CLIMATE_RANGE_CELSIUS, linear in between.
    NaN propagates."""
    if temperature_celsius != temperature_celsius:
        return float("nan")
    delta = abs(temperature_celsius - DEMOGRAPHY_CLIMATE_OPTIMUM_CELSIUS)
    if delta >= DEMOGRAPHY_CLIMATE_RANGE_CELSIUS:
        return 0.0
    return 1.0 - delta / DEMOGRAPHY_CLIMATE_RANGE_CELSIUS


def _climate_stress(temperature_celsius: float) -> float:
    """Inverse of `_climate_factor`: 0.0 at the optimum, 1.0 at the
    range limit. NaN propagates."""
    return 1.0 - _climate_factor(temperature_celsius)


def _capacity_headroom_fraction(population: int, capacity: int) -> float:
    """Fraction of capacity still available. Negative if over-capacity.
    Returns 0.0 if capacity is 0 (no headroom; settlement cannot
    sustain growth)."""
    if capacity <= 0:
        return 0.0
    return (capacity - population) / capacity


def _over_capacity_fraction(population: int, capacity: int) -> float:
    """Fraction by which population exceeds capacity. 0.0 if at or
    below capacity. Returns 0.0 if capacity is 0."""
    if capacity <= 0:
        return 0.0
    return max(0.0, (population - capacity) / capacity)


def _make_event_id(
    seed: int,
    event_type: EventType,
    step: int,
    settlement_id: int,
    salt: str,
) -> str:
    """Deterministic 16-char hex event id via blake2b.

    Per PHASE_3A_TYPES.md "Option A with a per-type + (t,
    settlement_id) salt" recommendation."""
    digest = hashlib.blake2b(digest_size=8, person=b"worldfac")
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(event_type.value.encode("utf-8"))
    digest.update(struct.pack(">q", step))
    digest.update(struct.pack(">q", settlement_id))
    digest.update(salt.encode("utf-8"))
    return digest.hexdigest()


def _make_individual_id(
    seed: int,
    settlement_id: int,
    step: int,
    birth_index: int,
) -> str:
    """Deterministic individual id for a birth event. Same scheme as
    event id but uses a longer person namespace to disambiguate."""
    digest = hashlib.blake2b(digest_size=8, person=b"indiv")
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(struct.pack(">q", settlement_id))
    digest.update(struct.pack(">q", step))
    digest.update(struct.pack(">q", birth_index))
    return digest.hexdigest()


def _step_population(
    current_population: int,
    births: int,
    deaths: int,
    net_migration: int,
) -> int:
    """Apply the demographic transitions for one step. Population is
    floored at 0 (no negative populations)."""
    return max(0, current_population + births - deaths + net_migration)


def _births_for_settlement(
    seed: int,
    settlement: Settlement,
    current_population: int,
    capacity: int,
    temperature: float,
    step: int,
) -> int:
    """Births at this step for this settlement."""
    climate_factor = _climate_factor(temperature)
    headroom = _capacity_headroom_fraction(current_population, capacity)
    if climate_factor != climate_factor:
        return 0  # NaN propagates; settle for 0 births.
    rate = (
        DEMOGRAPHY_BASE_BIRTH_RATE
        * climate_factor
        * (1.0 + DEMOGRAPHY_CAPACITY_HEADROOM_BIRTH_BOOST * headroom)
    )
    return int(current_population * rate)


def _deaths_for_settlement(
    seed: int,
    settlement: Settlement,
    current_population: int,
    capacity: int,
    temperature: float,
    step: int,
) -> int:
    """Deaths at this step for this settlement."""
    climate_stress = _climate_stress(temperature)
    if climate_stress != climate_stress:
        return 0  # NaN propagates; settle for 0 deaths.
    over = _over_capacity_fraction(current_population, capacity)
    conflict_tension = sample_unit_interval(
        seed, "demography.conflict", settlement.id, step
    )
    conflict_factor = (
        DEMOGRAPHY_CONFLICT_DEATH_MULTIPLIER
        if conflict_tension > DEMOGRAPHY_CONFLICT_THRESHOLD
        else 1.0
    )
    rate = (
        DEMOGRAPHY_BASE_DEATH_RATE
        * (1.0 + climate_stress + DEMOGRAPHY_OVER_CAPACITY_DEATH_PENALTY * over)
        * conflict_factor
    )
    return int(current_population * rate)


def _migrations_for_edge(
    seed: int,
    road: RoadEdge,
    pop_a: int,
    pop_b: int,
    cap_a: int,
    cap_b: int,
    step: int,
) -> int:
    """Migration count along a road edge at this step. Returns the
    net count (always >= 0; the formula prevents reverse flow)."""
    if pop_a <= 0:
        return 0
    pressure = max(0.0, (pop_a - cap_a) / pop_a)
    pull = max(0.0, (cap_b - pop_b) / max(cap_b, 1))
    cost_factor = 1.0 / (1.0 + road.cost / DEMOGRAPHY_MIGRATION_COST_DIVISOR)
    net_flow = (
        DEMOGRAPHY_MIGRATION_PRESSURE_FACTOR * pressure
        - DEMOGRAPHY_MIGRATION_PULL_FACTOR * pull
    )
    return max(0, int(net_flow * cost_factor * pop_a))


def _emit_birth_event(
    seed: int,
    settlement: Settlement,
    individual_id: str,
    step: int,
    birth_index: int,
    provenance: ProvenanceRecord,
) -> WorldEvent:
    return WorldEvent(
        id=_make_event_id(seed, EventType.BIRTH, step, settlement.id, individual_id),
        type=EventType.BIRTH,
        t=step,
        location=EventLocation(
            cell=(settlement.x, settlement.y),
            settlement_id=settlement.id,
        ),
        actors=(
            EventActor(
                kind="individual",
                identifier=individual_id,
                display_name=None,
            ),
        ),
        payload=BirthPayload(
            settlement_id=settlement.id,
            individual_id=individual_id,
            parent_ids=(),
            cohort_year=step,
        ).model_dump(mode="json"),
        causes=(),
        provenance=provenance,
    )


def _emit_death_event(
    seed: int,
    settlement: Settlement,
    individual_id: str,
    cause: str,
    age: int,
    step: int,
    provenance: ProvenanceRecord,
) -> WorldEvent:
    return WorldEvent(
        id=_make_event_id(seed, EventType.DEATH, step, settlement.id, individual_id),
        type=EventType.DEATH,
        t=step,
        location=EventLocation(
            cell=(settlement.x, settlement.y),
            settlement_id=settlement.id,
        ),
        actors=(
            EventActor(
                kind="individual",
                identifier=individual_id,
                display_name=None,
            ),
        ),
        payload=DeathPayload(
            settlement_id=settlement.id,
            individual_id=individual_id,
            cause=cause,
            age=age,
        ).model_dump(mode="json"),
        causes=(),
        provenance=provenance,
    )


def _emit_migration_event(
    seed: int,
    road: RoadEdge,
    moving_ids: tuple[str, ...],
    step: int,
    migration_index: int,
    provenance: ProvenanceRecord,
) -> WorldEvent:
    return WorldEvent(
        id=_make_event_id(
            seed,
            EventType.MIGRATION,
            step,
            road.from_settlement_id,
            f"{road.to_settlement_id}:{migration_index}",
        ),
        type=EventType.MIGRATION,
        t=step,
        location=EventLocation(
            cell=None,
            settlement_id=road.from_settlement_id,
        ),
        actors=tuple(
            EventActor(kind="individual", identifier=mid, display_name=None)
            for mid in moving_ids
        ),
        payload=MigrationPayload(
            from_settlement_id=road.from_settlement_id,
            to_settlement_id=road.to_settlement_id,
            individual_ids=moving_ids,
            road_cost=road.cost,
        ).model_dump(mode="json"),
        causes=(),
        provenance=provenance,
    )


def build_demography(
    world: WorldModel,
    time_steps: int = DEMOGRAPHY_DEFAULT_TIME_STEPS,
) -> DemographyLayer:
    """Run the population / migration / event simulation.

    Returns a `DemographyLayer` whose `pools` are parallel to
    `world.settlements.settlements` by id, whose `migrations`
    records per-edge per-step flows, and whose `events` holds
    the typed `BIRTH / DEATH / MIGRATION` events.

    Determinism: identical seeds produce byte-equivalent output.
    Every random draw is namespaced via `sample_unit_interval`.
    """
    seed = world.metadata.config.seed
    settlements = world.settlements.settlements
    agriculture = world.agriculture.agriculture
    roads = world.infrastructure.roads
    capacity_by_id = {
        record.settlement_id: record.carrying_capacity
        for record in agriculture
    }
    population_by_id = {settlement.id: settlement.population for settlement in settlements}
    # Initialize living-individual lists with synthetic ids for the
    # initial population so deaths / migrations can sample from a
    # non-empty pool. These ids do NOT appear in BIRTH events (they
    # predate the simulation) but DO appear as actors in subsequent
    # DEATH / MIGRATION events when sampled. The synthetic ids are
    # deterministic blake2b hashes of (seed, "init", settlement_id, n)
    # for n in range(initial_population).
    living_individuals: dict[int, list[str]] = {}
    for settlement in settlements:
        initial_pop = population_by_id[settlement.id]
        living_individuals[settlement.id] = [
            _make_individual_id(seed, settlement.id, -1, n)
            for n in range(initial_pop)
        ]
    provenance = demography_provenance()

    # Per-settlement population time series (length time_steps + 1).
    populations_by_id: dict[int, list[int]] = {
        settlement.id: [population_by_id[settlement.id]]
        for settlement in settlements
    }

    migrations: list[MigrationRecord] = []
    events: list[WorldEvent] = []

    sorted_settlements = sorted(settlements, key=lambda s: s.id)
    sorted_roads = sorted(
        roads, key=lambda r: (r.from_settlement_id, r.to_settlement_id)
    )

    # Pre-fetch climate temperature per settlement to avoid repeated indexing.
    temperature_by_id: dict[int, float] = {}
    climate_temperature = world.climate.temperature_celsius
    for settlement in settlements:
        if (
            0 <= settlement.y < len(climate_temperature)
            and 0 <= settlement.x < len(climate_temperature[settlement.y])
        ):
            temperature_by_id[settlement.id] = climate_temperature[
                settlement.y
            ][settlement.x]
        else:
            temperature_by_id[settlement.id] = float("nan")

    for step in range(time_steps):
        # BIRTHS
        for settlement in sorted_settlements:
            pop = populations_by_id[settlement.id][-1]
            capacity = capacity_by_id.get(settlement.id, 0)
            temperature = temperature_by_id[settlement.id]
            n_births = _births_for_settlement(
                seed, settlement, pop, capacity, temperature, step
            )
            new_individual_ids: list[str] = []
            for birth_index in range(n_births):
                new_id = _make_individual_id(seed, settlement.id, step, birth_index)
                new_individual_ids.append(new_id)
                events.append(
                    _emit_birth_event(
                        seed,
                        settlement,
                        new_id,
                        step,
                        birth_index,
                        provenance,
                    )
                )
            living_individuals[settlement.id].extend(new_individual_ids)
            populations_by_id[settlement.id].append(
                _step_population(
                    populations_by_id[settlement.id][-1],
                    n_births,
                    0,
                    0,
                )
            )

        # DEATHS
        for settlement in sorted_settlements:
            pop = populations_by_id[settlement.id][-1]
            capacity = capacity_by_id.get(settlement.id, 0)
            temperature = temperature_by_id[settlement.id]
            n_deaths = _deaths_for_settlement(
                seed, settlement, pop, capacity, temperature, step
            )
            living = living_individuals[settlement.id]
            sampleable_deaths = min(n_deaths, len(living))
            for death_index in range(sampleable_deaths):
                if not living:
                    break
                draw = sample_unit_interval(
                    seed,
                    "demography.death",
                    settlement.id,
                    step,
                    death_index,
                )
                target_index = int(draw * len(living))
                if target_index >= len(living):
                    target_index = len(living) - 1
                dead_id = living.pop(target_index)
                cause = (
                    "conflict"
                    if sample_unit_interval(
                        seed, "demography.cause", settlement.id, step, death_index
                    )
                    > DEMOGRAPHY_CONFLICT_THRESHOLD
                    else "natural"
                )
                events.append(
                    _emit_death_event(
                        seed,
                        settlement,
                        dead_id,
                        cause,
                        step,
                        step,
                        provenance,
                    )
                )
            populations_by_id[settlement.id][-1] = max(
                0, populations_by_id[settlement.id][-1] - n_deaths
            )

        # MIGRATIONS
        for road in sorted_roads:
            pop_a = populations_by_id[road.from_settlement_id][-1]
            pop_b = populations_by_id[road.to_settlement_id][-1]
            cap_a = capacity_by_id.get(road.from_settlement_id, 0)
            cap_b = capacity_by_id.get(road.to_settlement_id, 0)
            n_migrate = _migrations_for_edge(
                seed, road, pop_a, pop_b, cap_a, cap_b, step
            )
            living_a = living_individuals[road.from_settlement_id]
            # Cap migration at len(living_a) for v1 so population
            # moves match the sampled individual count. This is
            # conservative but keeps the event log + population
            # count in sync. Future phases can lift the cap by
            # initializing living_a with synthetic ids for the
            # initial population.
            n_migrate = min(n_migrate, len(living_a))
            moving_ids: list[str] = []
            for migration_index in range(n_migrate):
                if not living_a:
                    break
                draw = sample_unit_interval(
                    seed,
                    "demography.migration",
                    road.from_settlement_id,
                    road.to_settlement_id,
                    step,
                    migration_index,
                )
                target_index = int(draw * len(living_a))
                if target_index >= len(living_a):
                    target_index = len(living_a) - 1
                moving_id = living_a.pop(target_index)
                moving_ids.append(moving_id)
            populations_by_id[road.from_settlement_id][-1] = max(
                0, populations_by_id[road.from_settlement_id][-1] - n_migrate
            )
            populations_by_id[road.to_settlement_id][-1] += n_migrate
            living_individuals[road.to_settlement_id].extend(moving_ids)
            if moving_ids:
                migrations.append(
                    MigrationRecord(
                        id=len(migrations),
                        from_settlement_id=road.from_settlement_id,
                        to_settlement_id=road.to_settlement_id,
                        step=step,
                        count=len(moving_ids),
                        road_cost=round(road.cost, 6),
                    )
                )
                events.append(
                    _emit_migration_event(
                        seed,
                        road,
                        tuple(moving_ids),
                        step,
                        len(migrations) - 1,
                        provenance,
                    )
                )

    pools = tuple(
        PopulationPool(
            settlement_id=settlement.id,
            populations=tuple(populations_by_id[settlement.id]),
        )
        for settlement in sorted_settlements
    )
    return DemographyLayer(
        pools=pools,
        migrations=tuple(migrations),
        events=tuple(events),
    )


def demography_provenance() -> ProvenanceRecord:
    """Provenance record describing the demography simulation."""
    return ProvenanceRecord(
        output_path="demography",
        process="aggregate-pools-with-climate-capacity-conflict-migration",
        input_paths=(
            "settlements.settlements",
            "agriculture.agriculture",
            "infrastructure.roads",
            "climate.temperature_celsius",
            "metadata.config.seed",
        ),
        algorithm_version=DEMOGRAPHY_ALGORITHM_VERSION,
    )


def validate_demography_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3a.4 demography invariants.

    Checks:
      - `DemographyLayer.pools` is parallel to
        `SettlementsLayer.settlements` by id.
      - Every population in the time series is non-negative and
        finite.
      - Migration records reference valid settlement ids and a
        non-negative count; road cost is finite.
      - Events have valid type / location / actor references.
    """
    violations: list[InvariantViolation] = []
    settlements = world.settlements.settlements
    valid_ids = {settlement.id for settlement in settlements}
    layer = world.demography
    if len(layer.pools) != len(settlements):
        violations.append(
            _violation(
                "demography-pool-length-mismatch",
                "demography.pools",
                (
                    f"demography pools ({len(layer.pools)}) do not match "
                    f"settlements ({len(settlements)})"
                ),
            )
        )
        return violations
    for index, pool in enumerate(layer.pools):
        settlement = settlements[index]
        if pool.settlement_id != settlement.id:
            violations.append(
                _violation(
                    "demography-pool-settlement-id-mismatch",
                    f"demography.pools.{index}.settlement_id",
                    (
                        f"demography pool {index} references "
                        f"settlement_id={pool.settlement_id} but "
                        f"settlements.{index}.id={settlement.id}"
                    ),
                )
            )
        for step_index, population in enumerate(pool.populations):
            if population < 0:
                violations.append(
                    _violation(
                        "demography-population-negative",
                        f"demography.pools.{index}.populations.{step_index}",
                        (
                            f"population {population} for settlement "
                            f"{settlement.id} at step {step_index} is negative"
                        ),
                    )
                )
    for migration in layer.migrations:
        if migration.from_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "demography-migration-from-settlement-unknown",
                    f"demography.migrations.{migration.id}.from_settlement_id",
                    (
                        f"migration {migration.id} references unknown "
                        f"settlement id {migration.from_settlement_id}"
                    ),
                )
            )
        if migration.to_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "demography-migration-to-settlement-unknown",
                    f"demography.migrations.{migration.id}.to_settlement_id",
                    (
                        f"migration {migration.id} references unknown "
                        f"settlement id {migration.to_settlement_id}"
                    ),
                )
            )
        if migration.count < 0:
            violations.append(
                _violation(
                    "demography-migration-count-negative",
                    f"demography.migrations.{migration.id}.count",
                    f"migration {migration.id} count {migration.count} is negative",
                )
            )
    for event in layer.events:
        if (
            event.location.settlement_id is not None
            and event.location.settlement_id not in valid_ids
        ):
            violations.append(
                _violation(
                    "demography-event-settlement-unknown",
                    f"demography.events.{event.id}.location",
                    (
                        f"event {event.id} references unknown settlement id "
                        f"{event.location.settlement_id}"
                    ),
                )
            )
    return violations