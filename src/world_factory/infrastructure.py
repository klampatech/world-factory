"""Phase 3a.3 infrastructure — roads, ports, canals.

Algorithm:

1. **Friction grid** — per-cell traversal cost. Built from the
   biome, the slope between this cell and its lowest neighbor,
   and a per-cell penalty when the cell lies on a river-segment
   path (the "river crossing" cost). Ocean and ice cells carry
   `INFRASTRUCTURE_IMPASSABLE` so Dijkstra never routes over
   water.

2. **Roads** — for each settlement, run Dijkstra over the
   friction grid and pick the K nearest other settlements by
   path cost. Edges are deduplicated (canonicalised with
   `from < to`) so each undirected edge appears exactly once.
   This produces a sparse, MST-like graph connecting economic
   centers — the spec's "roads snap to a sparse graph".

3. **Ports** — settlements that sit within
   `INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS` of any river
   path cell (RIVER port) or within
   `INFRASTRUCTURE_COASTAL_RADIUS_CELLS` of any ocean cell
   (COASTAL port). The `annual_tonnage` field is a kcal/yr
   proxy derived from population + agricultural surplus,
   filtered by `INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD`.

4. **Canals** — pairs of settlements that BOTH have positive
   agricultural surplus (production zones) and are linked by
   at least one river segment whose mean discharge exceeds
   `INFRASTRUCTURE_CANAL_MIN_FLOW` and whose mean slope falls
   under `INFRASTRUCTURE_CANAL_SLOPE_LIMIT`. The cost is the
   friction-grid path cost between the two settlements, and
   `mean_flow` / `mean_slope` record the supporting river
   segment's hydrology.

The infrastructure layer consumes (settlements, agriculture,
hydrology, geography, biomes) and is additive on `WorldModel`.
`SCHEMA_VERSION` bumps to `10.0.0` to reflect the new
required `infrastructure` field; this follows the policy
established in Phase 3a.2 (any additive required-field change
bumps the major).

All outputs are deterministic given (seed, world state).

Forward-compat note for downstream consumers (3a.4 demography,
3b polities): the K-NN road graph is NOT guaranteed to be
single-component. With `INFRASTRUCTURE_ROAD_NEIGHBOR_K = 3`
each settlement connects only to its K nearest neighbors; on
worlds with disconnected landmasses (e.g., seed=42 LARGE
splits into a main island of 29 settlements and a smaller
island of 7), the resulting road graph is an archipelago with
multiple connected components. This is structurally realistic
— island settlements have no land bridge to the main
landmass within K reach — and downstream phases should expect
archipelago topology. If cross-island flows become necessary,
raise K or add a "near-reachability" fallback.
"""

from __future__ import annotations

import heapq
import math

from world_factory.constants import (
    INFRASTRUCTURE_ALGORITHM_VERSION,
    INFRASTRUCTURE_BASE_FRICTION_PER_BIOME,
    INFRASTRUCTURE_CANAL_MIN_FLOW,
    INFRASTRUCTURE_CANAL_SLOPE_LIMIT_M_PER_CELL,
    INFRASTRUCTURE_COASTAL_RADIUS_CELLS,
    INFRASTRUCTURE_DIAGONAL_COST,
    INFRASTRUCTURE_IMPASSABLE,
    INFRASTRUCTURE_MAX_CANALS,
    INFRASTRUCTURE_PORT_TONNAGE_PER_POPULATION,
    INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD,
    INFRASTRUCTURE_RIVER_CROSSING_PENALTY,
    INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS,
    INFRASTRUCTURE_ROAD_NEIGHBOR_K,
    INFRASTRUCTURE_SLOPE_PENALTY_PER_METER,
)
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    AgricultureRecord,
    BiomeClass,
    Canal,
    InfrastructureLayer,
    Port,
    PortKind,
    ProvenanceRecord,
    RiverSegment,
    RoadEdge,
    Settlement,
    WorldModel,
)

FloatGrid = tuple[tuple[float, ...], ...]
BoolGrid = tuple[tuple[bool, ...], ...]

_NEIGHBOR_OFFSETS: tuple[tuple[int, int, float], ...] = (
    (-1, -1, INFRASTRUCTURE_DIAGONAL_COST),  # NW
    (0, -1, 1.0),  # N
    (1, -1, INFRASTRUCTURE_DIAGONAL_COST),  # NE
    (-1, 0, 1.0),  # W
    (1, 0, 1.0),  # E
    (-1, 1, INFRASTRUCTURE_DIAGONAL_COST),  # SW
    (0, 1, 1.0),  # S
    (1, 1, INFRASTRUCTURE_DIAGONAL_COST),  # SE
)


def _biome_friction(biome: BiomeClass) -> float:
    return INFRASTRUCTURE_BASE_FRICTION_PER_BIOME[biome.value]


def _slope_between(
    elevation: FloatGrid, x: int, y: int, nx: int, ny: int
) -> float:
    """Return the absolute elevation drop per cell-step between (x,y)
    and (nx, ny)."""
    return abs(elevation[y][x] - elevation[ny][nx]) / math.sqrt(
        float((nx - x) ** 2 + (ny - y) ** 2)
    )


def _build_river_path_grid(
    width: int, height: int, river_segments: tuple[RiverSegment, ...]
) -> BoolGrid:
    """Per-cell flag set when the cell lies on any river-segment path."""
    grid: list[list[bool]] = [[False] * width for _ in range(height)]
    for segment in river_segments:
        for x, y in _interpolate_segment(segment):
            if 0 <= x < width and 0 <= y < height:
                grid[y][x] = True
    return tuple(tuple(row) for row in grid)


def _interpolate_segment(segment: RiverSegment) -> list[tuple[int, int]]:
    """Return every grid cell along a river segment's source-to-mouth
    path. The `RiverSegment` model stores source + mouth but not the
    intermediate cells; the hydrology layer traces them internally. For
    the infrastructure layer we approximate by walking from source to
    mouth one cell at a time along the dominant axis. River segments in
    Phase 1b are short (median ~3 cells), so this approximation is
    fine for the proximity check.
    """
    sx, sy = segment.source
    mx, my = segment.mouth
    cells: list[tuple[int, int]] = []
    if sx == mx:
        step = 1 if my > sy else -1
        for y in range(sy, my + step, step):
            cells.append((sx, y))
        return cells
    if sy == my:
        step = 1 if mx > sx else -1
        for x in range(sx, mx + step, step):
            cells.append((x, sy))
        return cells
    cells.append((sx, sy))
    # Diagonal: walk one axis at a time.
    step_x = 1 if mx > sx else -1
    step_y = 1 if my > sy else -1
    x, y = sx, sy
    while (x, y) != (mx, my):
        if abs(mx - x) > abs(my - y):
            x += step_x
        else:
            y += step_y
        cells.append((x, y))
    return cells


def _build_friction_grid(
    elevation: FloatGrid,
    biome_grid: tuple[tuple[BiomeClass, ...], ...],
    river_path: BoolGrid,
    height: int,
    width: int,
) -> FloatGrid:
    """Per-cell friction. Ocean / ice cells are impassable.

    Friction = biome_base × (1 + slope_penalty) + river_crossing_penalty
    where slope_penalty uses the maximum neighbor-to-neighbor elevation
    drop (steepest neighbor slope)."""
    grid: list[list[float]] = [[INFRASTRUCTURE_IMPASSABLE] * width for _ in range(height)]
    for y in range(height):
        for x in range(width):
            biome = biome_grid[y][x]
            base = _biome_friction(biome)
            if base >= INFRASTRUCTURE_IMPASSABLE:
                continue
            max_slope = 0.0
            for dx, dy, _ in _NEIGHBOR_OFFSETS:
                nx, ny = x + dx, y + dy
                if not (0 <= nx < width and 0 <= ny < height):
                    continue
                neighbor_biome = biome_grid[ny][nx]
                if _biome_friction(neighbor_biome) >= INFRASTRUCTURE_IMPASSABLE:
                    continue
                slope = _slope_between(elevation, x, y, nx, ny)
                if slope > max_slope:
                    max_slope = slope
            cost = base * (1.0 + INFRASTRUCTURE_SLOPE_PENALTY_PER_METER * max_slope)
            if river_path[y][x]:
                cost += INFRASTRUCTURE_RIVER_CROSSING_PENALTY
            grid[y][x] = cost
    return tuple(tuple(row) for row in grid)


def _dijkstra(
    friction: FloatGrid,
    start_x: int,
    start_y: int,
    width: int,
    height: int,
) -> tuple[FloatGrid, list[list[tuple[int, int] | None]]]:
    """Standard Dijkstra over the friction grid.

    Returns (cost_grid, parent_grid). parent_grid[y][x] is the
    predecessor cell on the cheapest path from (start_x, start_y),
    or None for the source and unreachable cells. Tiebreak on
    (cost, x, y) for determinism."""
    inf = math.inf
    cost_grid: list[list[float]] = [
        [inf] * width for _ in range(height)
    ]
    parent: list[list[tuple[int, int] | None]] = [
        [None] * width for _ in range(height)
    ]
    cost_grid[start_y][start_x] = 0.0
    heap: list[tuple[float, int, int]] = [(0.0, start_x, start_y)]
    while heap:
        cost, x, y = heapq.heappop(heap)
        if cost > cost_grid[y][x]:
            continue
        for dx, dy, step_cost in _NEIGHBOR_OFFSETS:
            nx, ny = x + dx, y + dy
            if not (0 <= nx < width and 0 <= ny < height):
                continue
            neighbor_friction = friction[ny][nx]
            if neighbor_friction >= INFRASTRUCTURE_IMPASSABLE:
                continue
            new_cost = cost + neighbor_friction * step_cost
            if new_cost < cost_grid[ny][nx] or (
                new_cost == cost_grid[ny][nx] and (nx, ny) < (parent[ny][nx] or (nx, ny))
            ):
                cost_grid[ny][nx] = new_cost
                parent[ny][nx] = (x, y)
                heapq.heappush(heap, (new_cost, nx, ny))
    return (
        tuple(tuple(row) for row in cost_grid),
        parent,
    )


def _path_length(
    parent: list[list[tuple[int, int] | None]],
    target_x: int,
    target_y: int,
) -> int:
    """Number of cell-steps from source to (target_x, target_y). 0 if
    unreachable."""
    if parent[target_y][target_x] is None:
        return 0
    length = 0
    current: tuple[int, int] | None = (target_x, target_y)
    while current is not None:
        prev = parent[current[1]][current[0]]
        if prev is None:
            break
        length += 1
        current = prev
    return length


def _compute_road_edges(
    settlements: tuple[Settlement, ...],
    friction: FloatGrid,
    width: int,
    height: int,
) -> tuple[RoadEdge, ...]:
    """K-NN road graph per settlement, deduplicated and canonically
    directed (from < to)."""
    settlement_by_id = {s.id: s for s in settlements}
    edges: dict[tuple[int, int], tuple[float, int]] = {}
    for start_settlement in settlements:
        cost_grid, parent_grid = _dijkstra(
            friction, start_settlement.x, start_settlement.y, width, height
        )
        candidates: list[tuple[float, int]] = []
        for other in settlements:
            if other.id == start_settlement.id:
                continue
            cost = cost_grid[other.y][other.x]
            if not math.isfinite(cost):
                continue
            candidates.append((cost, other.id))
        candidates.sort(key=lambda pair: (pair[0], pair[1]))
        keep = candidates[:INFRASTRUCTURE_ROAD_NEIGHBOR_K]
        for cost, other_id in keep:
            a_id = start_settlement.id
            b_id = other_id
            if a_id > b_id:
                a_id, b_id = b_id, a_id
            canonical: tuple[int, int] = (a_id, b_id)
            if canonical in edges:
                existing_cost, _ = edges[canonical]
                if cost >= existing_cost:
                    continue
            other_settlement = settlement_by_id[other_id]
            length = _path_length(
                parent_grid, other_settlement.x, other_settlement.y
            )
            edges[canonical] = (cost, length)
    return tuple(
        RoadEdge(
            id=index,
            from_settlement_id=a,
            to_settlement_id=b,
            cost=round(cost, 6),
            path_length=length,
        )
        for index, ((a, b), (cost, length)) in enumerate(
            sorted(
                edges.items(),
                key=lambda pair: (pair[1][0], pair[0][0], pair[0][1]),
            )
        )
    )


def _coastal_port_set(
    width: int,
    height: int,
    biome_grid: tuple[tuple[BiomeClass, ...], ...],
) -> set[tuple[int, int]]:
    """Cells within INFRASTRUCTURE_COASTAL_RADIUS_CELLS of any ocean cell."""
    ocean_mask = [
        [biome is BiomeClass.OCEAN for biome in row] for row in biome_grid
    ]
    coastal: set[tuple[int, int]] = set()
    radius = INFRASTRUCTURE_COASTAL_RADIUS_CELLS
    for y in range(height):
        for x in range(width):
            if ocean_mask[y][x]:
                for dy in range(-radius, radius + 1):
                    for dx in range(-radius, radius + 1):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < width and 0 <= ny < height and not ocean_mask[ny][nx]:
                            coastal.add((nx, ny))
    return coastal


def _river_proximity_set(
    width: int,
    height: int,
    river_path: BoolGrid,
) -> set[tuple[int, int]]:
    """Cells within INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS of any
    river path cell."""
    proximity: set[tuple[int, int]] = set()
    radius = INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS
    for y in range(height):
        for x in range(width):
            if river_path[y][x]:
                for dy in range(-radius, radius + 1):
                    for dx in range(-radius, radius + 1):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < width and 0 <= ny < height:
                            proximity.add((nx, ny))
    return proximity


def _compute_ports(
    settlements: tuple[Settlement, ...],
    agriculture: tuple[AgricultureRecord, ...],
    width: int,
    height: int,
    biome_grid: tuple[tuple[BiomeClass, ...], ...],
    river_path: BoolGrid,
) -> tuple[Port, ...]:
    coastal = _coastal_port_set(width, height, biome_grid)
    river_proximity = _river_proximity_set(width, height, river_path)
    agriculture_by_id = {record.settlement_id: record for record in agriculture}
    candidates: list[Port] = []
    next_port_id = 0
    for settlement in settlements:
        cell = (settlement.x, settlement.y)
        is_coastal = cell in coastal
        is_river = cell in river_proximity
        if not (is_coastal or is_river):
            continue
        agriculture_record = agriculture_by_id.get(settlement.id)
        surplus = (
            agriculture_record.agricultural_surplus_kcal_per_year
            if agriculture_record is not None
            else 0.0
        )
        tonnage = (
            surplus
            + INFRASTRUCTURE_PORT_TONNAGE_PER_POPULATION * settlement.population
        )
        if tonnage < INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD:
            continue
        port_kind = PortKind.COASTAL if is_coastal else PortKind.RIVER
        candidates.append(
            Port(
                id=next_port_id,
                settlement_id=settlement.id,
                port_kind=port_kind,
                annual_tonnage=round(tonnage, 6),
            )
        )
        next_port_id += 1
    candidates.sort(key=lambda port: (port.settlement_id, port.port_kind.value))
    return tuple(
        Port(
            id=index,
            settlement_id=port.settlement_id,
            port_kind=port.port_kind,
            annual_tonnage=port.annual_tonnage,
        )
        for index, port in enumerate(candidates)
    )


def _compute_canals(
    settlements: tuple[Settlement, ...],
    agriculture: tuple[AgricultureRecord, ...],
    river_segments: tuple[RiverSegment, ...],
    friction: FloatGrid,
    width: int,
    height: int,
) -> tuple[Canal, ...]:
    """Production-zone canals connecting surplus-positive settlement
    pairs that share a flow-feasible river segment."""
    agriculture_by_id = {record.settlement_id: record for record in agriculture}
    surplus_settlements = {
        settlement
        for settlement in settlements
        if agriculture_by_id.get(settlement.id) is not None
        and agriculture_by_id[settlement.id].agricultural_surplus_kcal_per_year > 0.0
    }
    if not surplus_settlements:
        return ()
    # Group river segments by source-cell + mouth-cell pair so a pair
    # of settlements can find the segment that links them.
    eligible_segments = [
        segment
        for segment in river_segments
        if segment.mean_discharge >= INFRASTRUCTURE_CANAL_MIN_FLOW
        and segment.mean_slope <= INFRASTRUCTURE_CANAL_SLOPE_LIMIT_M_PER_CELL
    ]
    if not eligible_segments:
        return ()
    canals: list[Canal] = []
    seen_pairs: set[tuple[int, int]] = set()
    for segment in eligible_segments:
        path_cells = _interpolate_segment(segment)
        for a in surplus_settlements:
            for b in surplus_settlements:
                if a.id >= b.id:
                    continue
                ax, ay = a.x, a.y
                bx, by = b.x, b.y
                # Segment must touch (or pass near) both settlements
                # within river-proximity radius.
                radius = INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS
                if not any(_within_radius((ax, ay), cell, radius) for cell in path_cells):
                    continue
                if not any(_within_radius((bx, by), cell, radius) for cell in path_cells):
                    continue
                pair = (a.id, b.id)
                if pair in seen_pairs:
                    continue
                cost_grid, _ = _dijkstra(friction, ax, ay, width, height)
                cost = cost_grid[by][bx]
                if not math.isfinite(cost):
                    continue
                seen_pairs.add(pair)
                canals.append(
                    Canal(
                        id=len(canals),
                        from_settlement_id=a.id,
                        to_settlement_id=b.id,
                        cost=round(cost, 6),
                        mean_flow=round(segment.mean_discharge, 6),
                        mean_slope=round(segment.mean_slope, 6),
                    )
                )
                if len(canals) >= INFRASTRUCTURE_MAX_CANALS:
                    canals.sort(
                        key=lambda canal: (
                            canal.cost,
                            canal.from_settlement_id,
                            canal.to_settlement_id,
                        )
                    )
                    return tuple(canals)
    canals.sort(key=lambda canal: (canal.cost, canal.from_settlement_id, canal.to_settlement_id))
    return tuple(canals)


def _within_radius(a: tuple[int, int], b: tuple[int, int], radius: int) -> bool:
    """Chebyshev distance <= radius."""
    return max(abs(a[0] - b[0]), abs(a[1] - b[1])) <= radius


def build_infrastructure(world: WorldModel) -> InfrastructureLayer:
    """Compute the roads / ports / canals layer.

    Reads settlements, agriculture, hydrology, geography, biomes.
    Returns an `InfrastructureLayer` whose roads connect economic
    centers via minimum-cost paths over a biome × slope × river
    friction grid; whose ports mark settlements on coastlines or
    rivers with sufficient tonnage; whose canals connect
    production-zone settlement pairs along flow- and slope-feasible
    river segments.
    """
    elevation = world.geography.elevation_meters
    biome_grid = world.biomes.classifications
    width = world.geography.width
    height = world.geography.height
    settlements = world.settlements.settlements
    agriculture = world.agriculture.agriculture
    river_segments = world.hydrology.river_segments
    river_path = _build_river_path_grid(width, height, river_segments)
    friction = _build_friction_grid(elevation, biome_grid, river_path, height, width)
    roads = _compute_road_edges(settlements, friction, width, height)
    ports = _compute_ports(
        settlements, agriculture, width, height, biome_grid, river_path
    )
    canals = _compute_canals(
        settlements, agriculture, river_segments, friction, width, height
    )
    return InfrastructureLayer(roads=roads, ports=ports, canals=canals)


def infrastructure_provenance() -> ProvenanceRecord:
    """Provenance record describing the infrastructure algorithm."""
    return ProvenanceRecord(
        output_path="infrastructure",
        process="min-cost-friction-with-knn-snap",
        input_paths=(
            "settlements.settlements",
            "agriculture.agriculture",
            "hydrology.river_segments",
            "geography.elevation_meters",
            "biomes.classifications",
        ),
        algorithm_version=INFRASTRUCTURE_ALGORITHM_VERSION,
    )


def validate_infrastructure_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 3a.3 infrastructure invariants.

    Checks:
      - roads / ports / canals reference valid settlement ids.
      - road edges are canonically directed (from < to) and unique.
      - road costs are finite and non-negative.
      - port tonnage is non-negative and finite.
      - canal mean_flow is finite and >= 0.
      - canal mean_slope is finite and >= 0.
      - infrastructure provenance record is present.
    """
    violations: list[InvariantViolation] = []
    settlements = world.settlements.settlements
    valid_ids = {settlement.id for settlement in settlements}
    layer = world.infrastructure
    seen_edge_pairs: set[tuple[int, int]] = set()
    for edge in layer.roads:
        if edge.from_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "infrastructure-road-from-settlement-unknown",
                    f"infrastructure.roads.{edge.id}.from_settlement_id",
                    f"road {edge.id} references unknown settlement id "
                    f"{edge.from_settlement_id}",
                )
            )
        if edge.to_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "infrastructure-road-to-settlement-unknown",
                    f"infrastructure.roads.{edge.id}.to_settlement_id",
                    f"road {edge.id} references unknown settlement id "
                    f"{edge.to_settlement_id}",
                )
            )
        if edge.from_settlement_id >= edge.to_settlement_id:
            violations.append(
                _violation(
                    "infrastructure-road-direction",
                    f"infrastructure.roads.{edge.id}",
                    (
                        f"road {edge.id} has from_settlement_id="
                        f"{edge.from_settlement_id} >= "
                        f"to_settlement_id={edge.to_settlement_id}"
                    ),
                )
            )
        pair = (edge.from_settlement_id, edge.to_settlement_id)
        if pair in seen_edge_pairs:
            violations.append(
                _violation(
                    "infrastructure-road-duplicate",
                    f"infrastructure.roads.{edge.id}",
                    f"road {edge.id} duplicates edge {pair}",
                )
            )
        seen_edge_pairs.add(pair)
        if not math.isfinite(edge.cost) or edge.cost < 0.0:
            violations.append(
                _violation(
                    "infrastructure-road-cost-bad",
                    f"infrastructure.roads.{edge.id}.cost",
                    f"road {edge.id} cost {edge.cost} is not finite or non-negative",
                )
            )
    for port in layer.ports:
        if port.settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "infrastructure-port-settlement-unknown",
                    f"infrastructure.ports.{port.id}.settlement_id",
                    f"port {port.id} references unknown settlement id "
                    f"{port.settlement_id}",
                )
            )
        if not math.isfinite(port.annual_tonnage) or port.annual_tonnage < 0.0:
            violations.append(
                _violation(
                    "infrastructure-port-tonnage-bad",
                    f"infrastructure.ports.{port.id}.annual_tonnage",
                    f"port {port.id} tonnage {port.annual_tonnage} not finite or non-negative",
                )
            )
    for canal in layer.canals:
        if canal.from_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "infrastructure-canal-from-settlement-unknown",
                    f"infrastructure.canals.{canal.id}.from_settlement_id",
                    f"canal {canal.id} references unknown settlement id "
                    f"{canal.from_settlement_id}",
                )
            )
        if canal.to_settlement_id not in valid_ids:
            violations.append(
                _violation(
                    "infrastructure-canal-to-settlement-unknown",
                    f"infrastructure.canals.{canal.id}.to_settlement_id",
                    f"canal {canal.id} references unknown settlement id "
                    f"{canal.to_settlement_id}",
                )
            )
        if canal.from_settlement_id >= canal.to_settlement_id:
            violations.append(
                _violation(
                    "infrastructure-canal-direction",
                    f"infrastructure.canals.{canal.id}",
                    (
                        f"canal {canal.id} has from_settlement_id="
                        f"{canal.from_settlement_id} >= "
                        f"to_settlement_id={canal.to_settlement_id}"
                    ),
                )
            )
        if not math.isfinite(canal.cost) or canal.cost < 0.0:
            violations.append(
                _violation(
                    "infrastructure-canal-cost-bad",
                    f"infrastructure.canals.{canal.id}.cost",
                    f"canal {canal.id} cost {canal.cost} not finite or non-negative",
                )
            )
        if not math.isfinite(canal.mean_flow) or canal.mean_flow < 0.0:
            violations.append(
                _violation(
                    "infrastructure-canal-flow-bad",
                    f"infrastructure.canals.{canal.id}.mean_flow",
                    f"canal {canal.id} flow {canal.mean_flow} not finite or non-negative",
                )
            )
        if not math.isfinite(canal.mean_slope) or canal.mean_slope < 0.0:
            violations.append(
                _violation(
                    "infrastructure-canal-slope-bad",
                    f"infrastructure.canals.{canal.id}.mean_slope",
                    f"canal {canal.id} slope {canal.mean_slope} not finite or non-negative",
                )
            )
    return violations