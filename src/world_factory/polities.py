"""Phase 4.1 — polity formation.

`build_polities(world) -> PolityLayer`
samples one `Polity` per culture (per plan-ack Q1: the 3b.4 1:1
culture-to-root-language ratio collapses the joint `(culture,
language_root)` cluster key to culture alone), then draws defensible
borders between polities from `hydrology.river_segments` and
`geography.elevation_meters` (per plan-ack Q2: river + elevation; no
biome-as-resource in v1 — that lands in 4.2). Governance type is
pinned at founding via `len(members)` per plan-ack Q3 (1-2 BAND,
3-6 CHIEFDOM, 7-15 KINGDOM, 16+ EMPIRE; `REPUBLIC` is in the enum
but unused at v1). Founder follows the 3b.3 `Lineage.founder_actor_
id` pattern: sample one living demography individual at step 0 from
the polity's primary settlement. v1 emits one `FOUNDED` event per
polity at step 0 (per plan-ack Q5).

`validate_polities_layer(world)` enforces the standard 3b.x validator
order (algorithm-version-mismatch FIRST, then parallel_structure,
per-record integrity, field ranges, no-surplus, no-orphans).
`polities_provenance()` describes the generator's input / process /
output paths.

Public surface (per the chain convention):

- `build_polities(world) -> PolityLayer`
- `validate_polities_layer(world) -> list[InvariantViolation]`
- `polities_provenance() -> ProvenanceRecord`

Spec fidelity:
- `Polity(id, name, founding_step, founder_actor_id, governance_type,
  algorithm_version)` per plan-ack.
- `PolityMember(polity_id, settlement_id, joined_step, joined_reason)`
  edge-list per plan-ack Q7 (separate from `Polity`).
- `Border(polity_a_id, polity_b_id, length_km, defense_strength,
  segments)` per-pair; `segments` is a tuple of `(x, y)` geography
  cells where the boundary runs (river segments + mountain cells
  above `ELEVATION_BORDER_THRESHOLD_M = 800`).
- `PolityEvent` discriminated payload pattern; `PolityFoundedPayload`
  minimal event for v1.
- `PolityLayer(polities, memberships, borders, events,
  algorithm_version)` aggregate on `WorldModel.polities`
  (additive-required).
- `WorldModel.polities` is additive-required per the 3a.2 policy.
  Schema bump 16.0.0 -> 17.0.0; Model-version bump phase-3b.4 ->
  phase-4.
"""

from __future__ import annotations

import hashlib
import json
import struct

from world_factory.constants import (
    ELEVATION_BORDER_THRESHOLD_M,
    GOVERNANCE_BAND_MAX,
    GOVERNANCE_CHIEFDOM_MAX,
    GOVERNANCE_KINGDOM_MAX,
    POLITY_ALGORITHM_VERSION,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    Border,
    EventLocation,
    EventType,
    GovernanceType,
    JoinReason,
    Polity,
    PolityFoundedPayload,
    PolityLayer,
    PolityMember,
    ProvenanceRecord,
    WorldEvent,
    WorldModel,
)

_MAXIMUM_UNSIGNED_64_BIT_VALUE = (1 << 64) - 1
_POLITY_BLAKE_PERSON = b"polities"

# Reserved blake2b person for the POLITY_FOUNDED event-id namespace
# (distinct from `b"worldfac"` / `b"culture"` / `b"religion"` /
# `b"kinship"` / `b"languages"` event-id namespaces).
_POLITY_EVENTID_PERSON = b"polyev"


def _derive_governance_type(n_members: int) -> GovernanceType:
    """Pin governance at founding via polity size (per plan-ack Q3).

    Size buckets:
    - 1-2: BAND
    - 3-6: CHIEFDOM
    - 7-15: KINGDOM
    - 16+: EMPIRE

    `REPUBLIC` is in the enum but unused at v1 — slot for 4.2
    political events. Drift (KINGDOM -> EMPIRE) deferred to 4.2.
    """
    if n_members <= GOVERNANCE_BAND_MAX:
        return GovernanceType.BAND
    if n_members <= GOVERNANCE_CHIEFDOM_MAX:
        return GovernanceType.CHIEFDOM
    if n_members <= GOVERNANCE_KINGDOM_MAX:
        return GovernanceType.KINGDOM
    return GovernanceType.EMPIRE


def _sample_founder_actor_id(
    world: WorldModel,
    polity_primary_settlement_id: int,
) -> str | None:
    """Sample one living demography individual at step 0 from
    `polity_primary_settlement_id` (per plan-ack Q4).

    Returns the `individual_id` from the first BIRTH event at
    `step == 0` referencing the primary settlement. Returns None
    if no such event exists (synthetic initial population or
    a demography layer with no step-0 births).
    """
    for event in world.demography.events:
        if (
            event.type == EventType.BIRTH
            and event.t == 0
            and event.location.settlement_id == polity_primary_settlement_id
        ):
            individual_id = event.payload.get("individual_id")
            if isinstance(individual_id, str):
                return individual_id
    return None


def _compute_polity_algorithm_version(
    polity_id: int,
    culture_id: int,
    governance_type: GovernanceType,
    founder_actor_id: str | None,
    founding_step: int,
) -> str:
    """Per-polity algorithm version blake2b hash."""
    digest = hashlib.blake2b(digest_size=8, person=b"polyver")
    digest.update(struct.pack(">q", polity_id))
    digest.update(struct.pack(">q", culture_id))
    digest.update(governance_type.value.encode("utf-8"))
    if founder_actor_id is not None:
        digest.update(founder_actor_id.encode("utf-8"))
    else:
        digest.update(b"-")
    digest.update(struct.pack(">q", founding_step))
    return digest.hexdigest()


def _compute_algorithm_version(
    polities: tuple[Polity, ...],
    memberships: tuple[PolityMember, ...],
    borders: tuple[Border, ...],
    events: tuple[WorldEvent, ...],
) -> str:
    """blake2b hash of polity + membership + border + event state.

    Mirrors the 3a.5 / 3b.x algorithm-version pattern: mutations or
    reordering change the hash, allowing the trust boundary to detect
    silent corruption."""
    digest = hashlib.blake2b(digest_size=8, person=_POLITY_BLAKE_PERSON)
    state = {
        "polities": [polity.model_dump(mode="json") for polity in polities],
        "memberships": [member.model_dump(mode="json") for member in memberships],
        "borders": [border.model_dump(mode="json") for border in borders],
        "events": [event.model_dump(mode="json") for event in events],
    }
    encoded_state = json.dumps(
        state,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    digest.update(encoded_state)
    return digest.hexdigest()


def _make_event_id(
    seed: int,
    event_type_str: str,
    step: int,
    polity_id: int,
    salt: str,
) -> str:
    """Deterministic 16-char hex POLITY_FOUNDED event id via blake2b.

    Distinct `b"polyev"` person namespace keeps POLITY_FOUNDED ids
    from colliding with demography / culture / religion / kinship /
    language event ids."""
    digest = hashlib.blake2b(digest_size=8, person=_POLITY_EVENTID_PERSON)
    digest.update(struct.pack(">Q", seed & _MAXIMUM_UNSIGNED_64_BIT_VALUE))
    digest.update(event_type_str.encode("utf-8"))
    digest.update(struct.pack(">q", step))
    digest.update(struct.pack(">q", polity_id))
    digest.update(salt.encode("utf-8"))
    return digest.hexdigest()


def _primary_settlement_for_culture(
    world: WorldModel,
    culture_settlement_id: int,
) -> int:
    """Pick the primary settlement for a culture.

    Per plan-ack Q4: primary settlement = the polity's largest
    settlement (or first by id if ties). v1 simplifies: just return
    the culture's own settlement_id (the one culture_id maps to
    directly via `world.cultures.cultures[i].settlement_id`).
    """
    return culture_settlement_id


def _derive_borders(
    world: WorldModel,
    polities: list[Polity],
    memberships: tuple[PolityMember, ...],
) -> tuple[Border, ...]:
    """Derive defensible borders between every polity pair.

    Per plan-ack Q2: river segments + `geography.elevation_meters
    >= ELEVATION_BORDER_THRESHOLD_M = 800m` (mountain cells). Each
    border carries the boundary cell list + length_km + defense
    strength.

    For v1, we walk the geography grid once and segment cells by
    their polity membership. A border is the set of cells where
    two adjacent cells belong to different polities. We emit one
    `Border` per unordered polity pair; `segments` lists the cell
    coordinates where the boundary runs, `length_km` is the count
    of boundary cells (proxy for boundary length; true length would
    use the geography's `GRID_CELL_AREA_KILOMETERS_SQUARED = 80.0`
    for `cell_size_km = sqrt(80) ~ 8.94km`).
    """
    if len(polities) < 2:
        return ()

    # Build settlement_id -> polity_id map.
    settlement_to_polity: dict[int, int] = {}
    for member in memberships:
        settlement_to_polity[member.settlement_id] = member.polity_id

    settlement_by_id = {s.id: s for s in world.settlements.settlements}
    sorted_settlement_ids = sorted(settlement_to_polity.keys())
    if not sorted_settlement_ids:
        return ()

    # Group settlements by polity for boundary walking.
    polity_to_settlements: dict[int, list[int]] = {}
    for sid in sorted_settlement_ids:
        polity_to_settlements.setdefault(settlement_to_polity[sid], []).append(sid)

    # Walk geography grid to find boundary cells. Two adjacent
    # cells belong to different polities -> boundary cell. We use
    # the cell's biome row/col to look up its nearest settlement
    # (or use the cell's biome directly as the boundary cell).
    biomes_grid = world.biomes.classifications
    elevation_grid = world.geography.elevation_meters
    height = len(biomes_grid)
    width = len(biomes_grid[0]) if biomes_grid else 0
    if height == 0 or width == 0:
        return ()

    # Assign every geography cell to its nearest member settlement. This
    # produces a deterministic Voronoi territory map; limiting the map to
    # settlement cells would miss almost every shared boundary.
    member_settlements = [
        settlement_by_id[settlement_id]
        for settlement_id in sorted_settlement_ids
        if settlement_id in settlement_by_id
    ]
    cell_polity: dict[tuple[int, int], int] = {}
    for y in range(height):
        for x in range(width):
            nearest = min(
                member_settlements,
                key=lambda settlement: (
                    (settlement.x - x) ** 2 + (settlement.y - y) ** 2,
                    settlement.id,
                ),
            )
            cell_polity[(x, y)] = settlement_to_polity[nearest.id]

    # Walk adjacent cell pairs. For each adjacent (a, b) where
    # cell_polity[a] != cell_polity[b], record a boundary cell.
    boundary_cells_per_pair: dict[tuple[int, int], list[tuple[int, int]]] = {}
    for y in range(height):
        for x in range(width):
            polity_a = cell_polity.get((x, y))
            if polity_a is None:
                continue
            # 4-connectivity: right + down neighbors.
            for dx, dy in ((1, 0), (0, 1)):
                nx, ny = x + dx, y + dy
                polity_b = cell_polity.get((nx, ny))
                if polity_b is None or polity_b == polity_a:
                    continue
                # Mountain boundary: cell at (x, y) or (nx, ny) is
                # at/above ELEVATION_BORDER_THRESHOLD_M.
                cell_a_high = (
                    elevation_grid[y][x] >= ELEVATION_BORDER_THRESHOLD_M
                    if y < height and x < width
                    else False
                )
                cell_b_high = (
                    elevation_grid[ny][nx] >= ELEVATION_BORDER_THRESHOLD_M
                    if ny < height and nx < width
                    else False
                )
                # For v1, every polity pair gets a border cell
                # entry; mountain threshold filters out
                # low-elevation pairs (per the 4.1 v1 simplification
                # note in the spec).
                if not (cell_a_high or cell_b_high):
                    continue
                # Normalize pair ordering so (a, b) and (b, a)
                # hash the same.
                pair = (min(polity_a, polity_b), max(polity_a, polity_b))
                boundary_cells_per_pair.setdefault(pair, []).append((x, y))

    # Build river-segment border contributions. For each river
    # segment that borders two polities, add the segment cells
    # to the boundary cell list.
    river_segments = world.hydrology.river_segments
    for segment in river_segments:
        for cell in (segment.source, segment.mouth):
            x, y = cell[0], cell[1]
            polity_a = cell_polity.get(cell)
            if polity_a is None:
                continue
            # Look at 4-adjacent cells for the other polity.
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                polity_b = cell_polity.get((nx, ny))
                if polity_b is None or polity_b == polity_a:
                    continue
                pair = (min(polity_a, polity_b), max(polity_a, polity_b))
                boundary_cells_per_pair.setdefault(pair, []).append(cell)

    # Build Border records. defense_strength = len(members) sum
    # per plan-ack minor note.
    membership_count: dict[int, int] = {}
    for member in memberships:
        membership_count[member.polity_id] = membership_count.get(member.polity_id, 0) + 1
    borders: list[Border] = []
    for (polity_a_id, polity_b_id), cells in boundary_cells_per_pair.items():
        defense = float(membership_count.get(polity_a_id, 0) + membership_count.get(polity_b_id, 0))
        # 1 cell unit = `cell_size_km = sqrt(80) ~ 8.944` per
        # `GRID_CELL_AREA_KILOMETERS_SQUARED = 80`.
        cell_size_km = (80.0) ** 0.5
        length_km = float(len(cells)) * cell_size_km
        borders.append(
            Border(
                polity_a_id=polity_a_id,
                polity_b_id=polity_b_id,
                length_km=length_km,
                defense_strength=defense,
                segments=tuple(cells),
            )
        )
    return tuple(borders)


def _build_polity_founded_event(
    seed: int,
    polity: Polity,
    primary_settlement_id: int,
    culture_id: int,
) -> WorldEvent:
    """Build the `POLITY_FOUNDED` event for a single polity at step 0."""
    payload = PolityFoundedPayload(
        polity_id=polity.id,
        culture_id=culture_id,
        governance_type=polity.governance_type,
        founding_step=polity.founding_step,
        step=polity.founding_step,
    )
    event_id = _make_event_id(
        seed,
        EventType.POLITY_FOUNDED.value,
        polity.founding_step,
        polity.id,
        f"polity:{polity.id}",
    )
    return WorldEvent(
        id=event_id,
        type=EventType.POLITY_FOUNDED,
        t=polity.founding_step,
        location=EventLocation(cell=None, settlement_id=primary_settlement_id),
        actors=(),
        payload=payload.model_dump(mode="python"),
        causes=(),
        provenance=ProvenanceRecord(
            output_path="polities.founded",
            process="polity-founding-step-0",
            input_paths=(
                "metadata.config.seed",
                "cultures.cultures",
                "demography.events",
            ),
            algorithm_version=POLITY_ALGORITHM_VERSION,
        ),
    )


def build_polities(
    world: WorldModel,
) -> PolityLayer:
    """Construct `PolityLayer` — one polity per culture, defensible
    borders between polities, governance pinned at founding, founder
    sampled from demography, one FOUNDED event per polity at step 0.

    Deterministic per `world.metadata.config.seed`: same seed ->
    same polities, same borders, same events, same algorithm
    versions.
    """
    seed = world.metadata.config.seed
    sorted_cultures = sorted(world.cultures.cultures, key=lambda c: c.settlement_id)

    polities: list[Polity] = []
    memberships: list[PolityMember] = []
    events: list[WorldEvent] = []
    for index, culture in enumerate(sorted_cultures):
        polity_id = index
        primary_settlement = _primary_settlement_for_culture(world, culture.settlement_id)
        # v1: every initial member joins at founding with
        # `joined_reason: CULTURE`. Multi-member polities for
        # 1 culture per settlement is the v1 case (1 culture per
        # settlement in the chain).
        memberships.append(
            PolityMember(
                polity_id=polity_id,
                settlement_id=culture.settlement_id,
                joined_step=0,
                joined_reason=JoinReason.CULTURE,
            )
        )
        founder_actor_id = _sample_founder_actor_id(world, primary_settlement)
        governance_type = _derive_governance_type(n_members=1)
        polity_name = f"Polity-{polity_id}-{culture.settlement_id}"
        founding_step = 0
        polity_version = _compute_polity_algorithm_version(
            polity_id=polity_id,
            culture_id=culture.settlement_id,
            governance_type=governance_type,
            founder_actor_id=founder_actor_id,
            founding_step=founding_step,
        )
        polities.append(
            Polity(
                id=polity_id,
                name=polity_name,
                founding_step=founding_step,
                founder_actor_id=founder_actor_id,
                governance_type=governance_type,
                algorithm_version=polity_version,
            )
        )
        events.append(
            _build_polity_founded_event(
                seed=seed,
                polity=polities[-1],
                primary_settlement_id=primary_settlement,
                culture_id=culture.settlement_id,
            )
        )

    # Build defensible borders between polities via geography
    # (river segments + elevation).
    borders = _derive_borders(
        world,
        polities=polities,
        memberships=tuple(memberships),
    )

    algorithm_version = _compute_algorithm_version(
        tuple(polities),
        tuple(memberships),
        borders,
        tuple(events),
    )
    return PolityLayer(
        polities=tuple(polities),
        memberships=tuple(memberships),
        borders=borders,
        events=tuple(events),
        algorithm_version=algorithm_version,
    )


def validate_polities_layer(world: WorldModel) -> list[InvariantViolation]:
    """Standard 3b.x validator order for the polities layer."""
    violations: list[InvariantViolation] = []
    layer = world.polities
    expected = _compute_algorithm_version(
        layer.polities, layer.memberships, layer.borders, layer.events
    )
    if layer.algorithm_version != expected:
        violations.append(
            _violation(
                "polities-algorithm-version-mismatch",
                "world.polities.algorithm_version",
                (
                    f"polities algorithm_version "
                    f"{layer.algorithm_version!r} does not match "
                    f"recomputed {expected!r}; layer was mutated "
                    f"or re-ordered outside the generator"
                ),
            )
        )

    cultures = world.cultures.cultures
    n_cultures = len(cultures)
    n_polities = len(layer.polities)
    if n_polities != n_cultures:
        violations.append(
            _violation(
                "polities-roots-parallel-structure",
                "world.polities.polities",
                (
                    f"polities count {n_polities} does not match "
                    f"cultures length {n_cultures} (expected one "
                    f"polity per culture per plan-ack Q1)"
                ),
            )
        )

    seen_polity_ids: set[int] = set()
    seen_membership_pairs: set[tuple[int, int]] = set()
    seen_member_settlement_ids: set[int] = set()
    seen_border_pairs: set[tuple[int, int]] = set()
    seen_event_ids: set[str] = set()
    for index, polity in enumerate(layer.polities):
        if polity.id in seen_polity_ids:
            violations.append(
                _violation(
                    "polities-duplicate-polity-id",
                    f"world.polities.polities.{index}.id",
                    f"polity id {polity.id} appears more than once",
                )
            )
        seen_polity_ids.add(polity.id)
        if polity.id != index:
            violations.append(
                _violation(
                    "polities-id-not-parallel-index",
                    f"world.polities.polities.{index}.id",
                    (
                        f"polity id {polity.id} does not match index "
                        f"{index} (polities must be parallel-to-cultures "
                        f"by index)"
                    ),
                )
            )
    for index, member in enumerate(layer.memberships):
        if (member.polity_id, member.settlement_id) in seen_membership_pairs:
            violations.append(
                _violation(
                    "polities-duplicate-membership",
                    f"world.polities.memberships.{index}",
                    f"({member.polity_id}, {member.settlement_id}) appears more than once",
                )
            )
        seen_membership_pairs.add((member.polity_id, member.settlement_id))
        if member.settlement_id in seen_member_settlement_ids:
            violations.append(
                _violation(
                    "polities-settlement-multiple-memberships",
                    f"world.polities.memberships.{index}.settlement_id",
                    f"settlement {member.settlement_id} belongs to multiple polities",
                )
            )
        seen_member_settlement_ids.add(member.settlement_id)
        if member.polity_id not in seen_polity_ids:
            violations.append(
                _violation(
                    "polities-orphaned-membership",
                    f"world.polities.memberships.{index}",
                    (f"membership references unknown polity {member.polity_id}"),
                )
            )

    expected_settlement_ids = {
        settlement.id for settlement in world.settlements.settlements
    }
    missing_memberships = expected_settlement_ids - seen_member_settlement_ids
    if missing_memberships:
        violations.append(
            _violation(
                "polities-settlement-membership-missing",
                "world.polities.memberships",
                f"settlements missing polity memberships: {sorted(missing_memberships)}",
            )
        )

    for index, border in enumerate(layer.borders):
        pair = (border.polity_a_id, border.polity_b_id)
        if border.polity_a_id >= border.polity_b_id:
            violations.append(
                _violation(
                    "polities-border-order",
                    f"world.polities.borders.{index}",
                    f"border pair {pair} must be ordered with polity_a_id < polity_b_id",
                )
            )
        if pair in seen_border_pairs:
            violations.append(
                _violation(
                    "polities-duplicate-border",
                    f"world.polities.borders.{index}",
                    f"border pair {pair} appears more than once",
                )
            )
        seen_border_pairs.add(pair)
        if pair[0] not in seen_polity_ids or pair[1] not in seen_polity_ids:
            violations.append(
                _violation(
                    "polities-orphaned-border",
                    f"world.polities.borders.{index}",
                    f"border pair {pair} references an unknown polity",
                )
            )

    for index, event in enumerate(layer.events):
        if event.id in seen_event_ids:
            violations.append(
                _violation(
                    "polities-duplicate-event-id",
                    f"world.polities.events.{index}",
                    f"event id {event.id} appears more than once",
                )
            )
        seen_event_ids.add(event.id)
        if event.type != EventType.POLITY_FOUNDED:
            violations.append(
                _violation(
                    "polities-invalid-event-type",
                    f"world.polities.events.{index}",
                    (f"event {event.id} has type {event.type!r}; v1 expects only POLITY_FOUNDED"),
                )
            )

    return violations


def polities_provenance() -> ProvenanceRecord:
    """Provenance record describing the polity-layer builder."""
    return ProvenanceRecord(
        output_path="polities",
        process="polity-formation-via-culture-cluster-and-geography-borders",
        input_paths=(
            "metadata.config.seed",
            "cultures.cultures",
            "demography.events",
            "hydrology.river_segments",
            "geography.elevation_meters",
        ),
        algorithm_version=POLITY_ALGORITHM_VERSION,
    )
