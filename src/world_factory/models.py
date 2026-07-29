"""Typed, versioned contracts for generated worlds."""

from enum import StrEnum

import pydantic
from pydantic import BaseModel, ConfigDict, Field

from world_factory.constants import (
    EARTH_AXIAL_TILT_DEGREES,
    EARTH_ORBITAL_ECCENTRICITY,
    EARTH_ORBITAL_PERIOD_DAYS,
    EARTH_ROTATION_PERIOD_HOURS,
    KINSHIP_LINEAGE_DEPTH_MAX,
    KINSHIP_LINEAGE_DEPTH_MIN,
    MAXIMUM_PLATE_COUNT,
    MAXIMUM_SEED,
    MINIMUM_PLATE_COUNT,
    MINIMUM_SEED,
)


class StrictModel(BaseModel):
    """Reject unknown or coerced data and prevent model reassignment."""

    model_config = ConfigDict(extra="forbid", frozen=True, strict=True)


class WorldScale(StrEnum):
    """Supported grid sizes. Phase 1a adds LARGE for v1 demo worlds."""

    SMALL = "small"
    MEDIUM = "medium"
    LARGE = "large"


class ClimateClass(StrEnum):
    """Broad planetary climate controls implemented in Phase 0."""

    COLD = "cold"
    TEMPERATE = "temperate"
    HOT = "hot"


class WindDirection(StrEnum):
    """Prevailing surface wind direction per cell. Phase 1c."""

    EAST = "east"
    WEST = "west"
    NORTH = "north"
    SOUTH = "south"
    NORTH_EAST = "north-east"
    NORTH_WEST = "north-west"
    SOUTH_EAST = "south-east"
    SOUTH_WEST = "south-west"
    CALM = "calm"


class BiomeClass(StrEnum):
    """Per-cell biome classification derived from physical conditions."""

    OCEAN = "ocean"
    ICE = "ice"
    ALPINE = "alpine"
    DESERT = "desert"
    TROPICAL_FOREST = "tropical-forest"
    TEMPERATE_FOREST = "temperate-forest"
    GRASSLAND = "grassland"


class PlateType(StrEnum):
    """Lithospheric plate composition. Continental plates ride higher
    than oceanic plates; convergent boundaries between oceanic and
    continental plates are subduction zones."""

    CONTINENTAL = "continental"
    OCEANIC = "oceanic"


class BoundaryType(StrEnum):
    """Plate boundary classification derived from relative plate motion."""

    CONVERGENT = "convergent"
    DIVERGENT = "divergent"
    TRANSFORM = "transform"


class RockType(StrEnum):
    """Per-cell rock classification. Phase 1e sublayer."""

    BASALT = "basalt"
    GRANITE = "granite"
    SEDIMENTARY = "sedimentary"
    METAMORPHIC = "metamorphic"
    VOLCANIC = "volcanic"


class SoilType(StrEnum):
    """Per-cell soil classification. Phase 1e sublayer."""

    PERMAFROST = "permafrost"
    SAND = "sand"
    LOAM = "loam"
    CLAY = "clay"
    PEAT = "peat"


class FloraType(StrEnum):
    """Per-cell primary flora species or community. Phase 2."""

    CONIFER = "conifer"
    BROADLEAF = "broadleaf"
    SHRUB = "shrub"
    GRASS = "grass"
    MOSS = "moss"
    LICHEN = "lichen"
    ALGAE = "algae"
    SEAGRASS = "seagrass"
    CORAL = "coral"


class FaunaType(StrEnum):
    """Per-cell primary fauna species or community. Phase 2."""

    HERBIVORE_LARGE = "herbivore-large"
    HERBIVORE_SMALL = "herbivore-small"
    CARNIVORE_LARGE = "carnivore-large"
    CARNIVORE_SMALL = "carnivore-small"
    FISH = "fish"
    BIRD = "bird"
    INSECT = "insect"
    REPTILE = "reptile"


class WorldConfig(StrictModel):
    """Validated parameters that define a world's identity."""

    seed: int = Field(ge=MINIMUM_SEED, le=MAXIMUM_SEED)
    scale: WorldScale = WorldScale.SMALL
    climate_class: ClimateClass = ClimateClass.TEMPERATE
    sentience_enabled: bool = True
    magic_enabled: bool = False
    plate_count: int = Field(default=12, ge=MINIMUM_PLATE_COUNT, le=MAXIMUM_PLATE_COUNT)
    axial_tilt_degrees: float = Field(default=EARTH_AXIAL_TILT_DEGREES, ge=0.0, le=90.0)
    orbital_eccentricity: float = Field(default=EARTH_ORBITAL_ECCENTRICITY, ge=0.0, le=0.5)
    rotation_period_hours: float = Field(default=EARTH_ROTATION_PERIOD_HOURS, gt=0.0)
    orbital_period_days: float = Field(default=EARTH_ORBITAL_PERIOD_DAYS, gt=0.0)
    season_day: float = Field(default=0.0, ge=0.0, lt=1_000_000.0)


class WorldMetadata(StrictModel):
    """Stable identity and version information for a generated world."""

    world_id: str = Field(min_length=16, max_length=64)
    schema_version: str
    model_version: str
    config: WorldConfig


class PlateRecord(StrictModel):
    """A single tectonic plate. Plates own a contiguous Voronoi cell set
    on the world grid."""

    id: int = Field(ge=0)
    plate_type: PlateType
    centroid_x: float = Field(ge=0.0)
    centroid_y: float = Field(ge=0.0)
    motion_heading_radians: float = Field(ge=0.0)
    motion_speed: float = Field(ge=0.0)
    cell_count: int = Field(gt=0)


class BoundaryRecord(StrictModel):
    """A single plate-boundary cell classification."""

    x: int = Field(ge=0)
    y: int = Field(ge=0)
    boundary_type: BoundaryType
    plate_a: int = Field(ge=0)
    plate_b: int = Field(ge=0)


class GeologyLayer(StrictModel):
    """Tectonic state of the world. Phase 1a first PR ships the
    geometry; Phase 1e adds rock, ore, and soil sublayers."""

    width: int = Field(gt=0)
    height: int = Field(gt=0)
    plates: tuple[PlateRecord, ...]
    boundaries: tuple[BoundaryRecord, ...]
    plate_id_grid: tuple[tuple[int, ...], ...]
    boundary_type_grid: tuple[tuple[BoundaryType | None, ...], ...]
    rock_type_grid: tuple[tuple[RockType, ...], ...]
    ore_presence_grid: tuple[tuple[bool, ...], ...]
    soil_type_grid: tuple[tuple[SoilType, ...], ...]


class GeographyLayer(StrictModel):
    """Regular-grid topography produced by the active geography module."""

    width: int = Field(gt=0)
    height: int = Field(gt=0)
    sea_level_meters: float
    elevation_meters: tuple[tuple[float, ...], ...]


class AstronomyLayer(StrictModel):
    """Astronomical forcing and its per-cell consequences. Phase 1d
    adds axial tilt and seasonal cycles; the layer records the
    forcing parameters and the per-cell day length + insolation."""

    axial_tilt_degrees: float = Field(ge=0.0, le=90.0)
    orbital_eccentricity: float = Field(ge=0.0, le=0.5)
    season_day: float = Field(ge=0.0)
    solar_declination_degrees: float = Field(ge=-90.0, le=90.0)
    day_length_hours: tuple[tuple[float, ...], ...]
    insolation_factor: tuple[tuple[float, ...], ...]


class RiverSegment(StrictModel):
    """A traced river from headwater source to ocean mouth."""

    id: int = Field(ge=0)
    source: tuple[int, int]
    mouth: tuple[int, int]
    length_cells: int = Field(ge=1)
    mean_discharge: float = Field(ge=0.0)
    mean_slope: float = Field(ge=0.0)
    watershed_id: int = Field(ge=0)


class HydrologyLayer(StrictModel):
    """River network, per-cell discharge, and watershed delineation.
    Phase 0 emitted an aggregate stub; Phase 1b adds the actual network."""

    surface_water_fraction: float = Field(ge=0.0, le=1.0)
    headwater_candidate_count: int = Field(ge=0)
    river_segments: tuple[RiverSegment, ...]
    discharge_grid: tuple[tuple[float, ...], ...]
    watershed_id_grid: tuple[tuple[int | None, ...], ...]


class ClimateLayer(StrictModel):
    """Regular-grid climate state derived from topography, parameters,
    and the Phase 1c atmospheric circulation model."""

    atmospheric_pressure_kpa: tuple[tuple[float, ...], ...]
    temperature_celsius: tuple[tuple[float, ...], ...]
    annual_precipitation_mm: tuple[tuple[float, ...], ...]
    wind_direction_grid: tuple[tuple[WindDirection, ...], ...]
    specific_humidity_grid: tuple[tuple[float, ...], ...]


class BiomeLayer(StrictModel):
    """Biome classification grid derived from physical conditions."""

    classifications: tuple[tuple[BiomeClass, ...], ...]


class BiologyLayer(StrictModel):
    """Per-cell flora and fauna assignments. Phase 2.

    Each cell carries a primary flora species (or `None` for ocean
    cells that capture their biota via the ALGAE / FISH defaults)
    and a primary fauna species. The mapping is biome-driven;
    ocean cells override biome defaults with marine biota."""

    flora_grid: tuple[tuple[FloraType | None, ...], ...]
    fauna_grid: tuple[tuple[FaunaType | None, ...], ...]


class Settlement(StrictModel):
    """A single settlement. Phase 3a.

    Carries id, position, population, and the founding score that
    selected it from the candidate grid. v1 ballpark population
    derived from arable land + water access + mineral proximity."""

    id: int = Field(ge=0)
    x: int = Field(ge=0)
    y: int = Field(ge=0)
    population: int = Field(ge=0)
    founding_score: float = Field(ge=0.0, le=1.0)


class SettlementsLayer(StrictModel):
    """Per-world settlement placement. Phase 3a.

    Settlements are an unordered tuple; downstream phases may
    order or query by id."""

    settlements: tuple[Settlement, ...]


class AgricultureRecord(StrictModel):
    """Per-settlement caloric accounting. Phase 3a.2.

    Carries the carrying capacity (max sustainable population
    from local arable yield), the agricultural surplus in
    kilocalories per year (positive or negative relative to the
    settlement's current population), and a seasonal deficit
    flag set when the worst cell in the extraction radius falls
    below `AGRICULTURE_DEFICIT_YIELD_FRACTION` of the base yield
    or the settlement has zero arable neighbors."""

    settlement_id: int = Field(ge=0)
    carrying_capacity: int = Field(ge=0)
    agricultural_surplus_kcal_per_year: float
    seasonal_deficit: bool


class AgricultureLayer(StrictModel):
    """Per-world agricultural yield and carrying capacity. Phase 3a.2.

    Records are parallel to `SettlementsLayer.settlements` by id
    (same length, same order) so a reviewer can pair them by index
    without a separate lookup table."""

    agriculture: tuple[AgricultureRecord, ...]


class PortKind(StrEnum):
    """How a settlement qualifies as a port. Phase 3a.3."""

    RIVER = "river"
    COASTAL = "coastal"


class ProvenanceRecord(StrictModel):
    """Inspectable evidence linking an output path to its generating process."""

    output_path: str
    process: str
    input_paths: tuple[str, ...]
    algorithm_version: str


class RoadEdge(StrictModel):
    """A minimum-cost road edge between two settlements. Phase 3a.3.

    `from_settlement_id < to_settlement_id` (canonical direction
    so each undirected edge appears exactly once in the layer).
    `cost` is the sum of friction-grid cell costs along the
    Dijkstra-discovered path; `path_length` is the number of
    cell-steps."""

    id: int = Field(ge=0)
    from_settlement_id: int = Field(ge=0)
    to_settlement_id: int = Field(ge=0)
    cost: float = Field(ge=0.0)
    path_length: int = Field(ge=0)


class Port(StrictModel):
    """A settlement that qualifies as a port. Phase 3a.3.

    `port_kind` is RIVER if the settlement sits within
    INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS of any river path
    cell; COASTAL if it sits within
    INFRASTRUCTURE_COASTAL_RADIUS_CELLS of any ocean cell.
    `annual_tonnage` is a kcal-per-year proxy: the settlement's
    agricultural surplus summed with a per-population baseline,
    filtered by `INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD`."""

    id: int = Field(ge=0)
    settlement_id: int = Field(ge=0)
    port_kind: PortKind
    annual_tonnage: float = Field(ge=0.0)


class Canal(StrictModel):
    """An artificial waterway connecting two production zones. Phase 3a.3.

    Both endpoints must be settlements with positive agricultural
    surplus (production zones) and at least one river segment
    between them. `cost` is friction-weighted path cost; `mean_flow`
    is the river-segment mean discharge (m^3/year) supplying the
    canal; `mean_slope` is the segment's mean slope (rise/run).
    """

    id: int = Field(ge=0)
    from_settlement_id: int = Field(ge=0)
    to_settlement_id: int = Field(ge=0)
    cost: float = Field(ge=0.0)
    mean_flow: float = Field(ge=0.0)
    mean_slope: float = Field(ge=0.0)


class InfrastructureLayer(StrictModel):
    """Roads, ports, and canals. Phase 3a.3.

    Roads connect economic centers via a minimum-cost path over
    the friction layer. Ports mark settlements adjacent to a
    coastline or river with enough tonnage to qualify. Canals
    connect production zones along rivers where flow + slope
    permit."""

    roads: tuple[RoadEdge, ...]
    ports: tuple[Port, ...]
    canals: tuple[Canal, ...]


class EventType(StrEnum):
    """Phase 3a / 3b event taxonomy. Phase 4+ adds polity-scoped types.

    Phase 3a.4 demography emits BIRTH, DEATH, and MIGRATION.
    Phase 3b.1 cultures emits CULTURE_DRIFT (per-settlement per-step
    per-attribute drift events). Phase 3b.2 religion emits BELIEF (one
    per ritual add / remove per settlement per step). Other event
    types (settlement founding, yield computed, road built, port
    established, canal cut) are reserved for the follow-up phases
    per the PHASE_3A_TYPES.md adoption path.
    """

    BIRTH = "demography.birth"
    DEATH = "demography.death"
    MIGRATION = "demography.migration"
    CULTURE_DRIFT = "culture.drift"
    BELIEF = "religion.belief"
    LINEAGE_FOUNDED = "kinship.lineage_founded"


class EventLocation(StrictModel):
    """Where an event happened. Cell-coords for spatial, settlement_id
    for demographic events."""

    cell: tuple[int, int] | None = None
    settlement_id: int | None = Field(default=None, ge=0)


class EventActor(StrictModel):
    """Named participant in an event. Individuals, settlements, and
    (in Phase 4+) polities surface as actors with a `kind`
    discriminator and a stable identifier."""

    kind: str
    identifier: str
    display_name: str | None = None


class BirthPayload(StrictModel):
    """Discriminated payload for `EventType.BIRTH`.

    `individual_id` is the new string id born in this event; the
    same string appears as `EventActor.identifier` (with
    `kind="individual"`) in subsequent DEATH / MIGRATION events.
    No free-standing Individual registry; the event log IS the
    registry (per PHASE_3A_TYPES.md OQ-B).

    `parent_ids` is typed `list` rather than `tuple` because the
    payload round-trips through JSON (`load_world` re-parses with
    `strict=True`); Pydantic v2 strict mode rejects list-to-tuple
    coercion, so we accept the JSON-native list type."""

    settlement_id: int = Field(ge=0)
    individual_id: str
    parent_ids: list[str]
    cohort_year: int = Field(ge=0)


class DeathPayload(StrictModel):
    """Discriminated payload for `EventType.DEATH`.

    `age` semantics: years since the individual was born
    (`step - birth_step`). For birth-tracked ids this is the true
    lifetime. For synthetic initial-population ids (`birth_step = -1`),
    this is `step + 1` — they existed before the sim started; their
    age cannot be derived from the event log alone. The `birth_ledger`
    in `demography.py` records `birth_step` per individual id;
    consumers that need exact lifetime should join against the BIRTH
    event whose `individual_id` matches the DEATH event.
    """

    settlement_id: int = Field(ge=0)
    individual_id: str
    cause: str
    age: int = Field(ge=0)


class MigrationPayload(StrictModel):
    """Discriminated payload for `EventType.MIGRATION`.

    `from_settlement_id` and `to_settlement_id` reference
    `Settlement.id` values. Migration is only recorded along
    infrastructure road edges; the road graph is the only path
    between settlements in 3a.4.

    `individual_ids` is typed `list` rather than `tuple` because
    the payload round-trips through JSON; see BirthPayload."""

    from_settlement_id: int = Field(ge=0)
    to_settlement_id: int = Field(ge=0)
    individual_ids: list[str]
    road_cost: float = Field(ge=0.0)


class CultureDriftPayload(StrictModel):
    """Discriminated payload for `EventType.CULTURE_DRIFT`.

    One CULTURE_DRIFT event is emitted per changed attribute per
    (settlement, step) so the event log stays compact (rather than
    one event per (settlement, step) carrying the full 6-dim
    vector). `attribute` is one of `CULTURE_ATTRIBUTE_NAMES`
    (`values`, `norms`, `taboos`, `ritual_forms`, `cuisine`,
    `music_motifs`); `old_value` and `new_value` are the
    pre- and post-drift attribute values, clamped to `[0, 1]`.

    `step` mirrors `WorldEvent.t` and is included in the payload
    for downstream consumers that index by step (Phase 5 causal
    graph, v2 visual explorer).
    """

    settlement_id: int = Field(ge=0)
    attribute: str
    old_value: float = Field(ge=0.0, le=1.0)
    new_value: float = Field(ge=0.0, le=1.0)
    step: int = Field(ge=0)


class RitualType(StrEnum):
    """Categorical ritual practice. Phase 3b.2.

    The six ritual types are sampled per settlement from
    `RELIGION_BIOME_RITUAL_BIAS` (probabilities summing to 1.0 per
    biome). Spec note: arid biomes (DESERT) carry the strongest
    WATER weight (spec line 192-193: 'arid → water rituals')."""

    WATER = "water"
    HARVEST = "harvest"
    FIRE = "fire"
    ANCESTOR = "ancestor"
    SKY = "sky"
    EARTH = "earth"


class Cosmology(StrEnum):
    """Cosmological narrative axis. Phase 3b.2.

    CYCLE = cyclical renewal / eternal return; LINEAR = linear-time
    teleology (creation → eschaton). Sampled per settlement from
    `RELIGION_BIOME_COSMOLOGY_BIAS` and held stable across the
    simulation (structural element)."""

    CYCLE = "cycle"
    LINEAR = "linear"


class Eschatology(StrEnum):
    """End-state narrative. Phase 3b.2.

    APOCALYPTIC = catastrophe-driven ending; RENEWAL = restorative /
    salvific ending; CYCLICAL = eschaton-as-return-to-the-beginning.
    Sampled per settlement from `RELIGION_HISTORY_ESCHATOLOGY_BIAS`
    keyed on the recent-death-rate bucket (low / mid / high) and
    held stable across the simulation (structural element)."""

    APOCALYPTIC = "apocalyptic"
    RENEWAL = "renewal"
    CYCLICAL = "cyclical"


class BeliefPayload(StrictModel):
    """Discriminated payload for `EventType.BELIEF`.

    One BELIEF event is emitted per ritual addition / removal per
    (settlement, step). `ritual_added` and `ritual_removed` are
    `Ritual.id` references (NOT ritual types — a Ritual record has
    its own id so Phase 4 polities + Phase 5 causal graph can refer
    to per-ritual provenance). At most one of the two is non-None
    per event (a single step either adds one ritual or removes
    one; if both happen in the same step two events are emitted).

    `step` mirrors `WorldEvent.t`."""

    settlement_id: int = Field(ge=0)
    ritual_added: int | None = Field(default=None, ge=0)
    ritual_removed: int | None = Field(default=None, ge=0)
    step: int = Field(ge=0)

    @pydantic.model_validator(mode="after")
    def _validate_single_ritual_change(self) -> "BeliefPayload":
        if (self.ritual_added is None) == (self.ritual_removed is None):
            raise ValueError("exactly one of ritual_added or ritual_removed must be set")
        return self


class WorldEvent(StrictModel):
    """Atomic unit of world history. Phase 3a emits typed events;
    Phase 5 consumes them for the causal graph.

    `payload` is a dict at the model surface for cheap instantiation,
    but is re-validated against the typed `BirthPayload /
    DeathPayload / MigrationPayload` discriminated union via the
    `_validate_payload_shape` model_validator below, per
    `PHASE_3A_TYPES.md` OQ-A. The validator is invoked at construction
    AND at the `WorldModel.model_validate_json` trust boundary, so
    downstream agents that hand-construct a `WorldEvent` with the
    wrong payload shape for the declared `type` are caught
    immediately."""

    id: str = Field(min_length=16, max_length=64)
    type: EventType
    t: int
    location: EventLocation
    actors: tuple[EventActor, ...]
    payload: dict[str, object]
    causes: tuple[str, ...] = ()
    provenance: ProvenanceRecord

    @pydantic.model_validator(mode="after")
    def _validate_payload_shape(self) -> "WorldEvent":
        # Use model_validate (non-strict) for the re-validation so
        # that JSON round-trips — where tuples become lists — still
        # pass. The strict build-time construction still uses the
        # strict typed payload constructors in demography.py.
        if self.type == EventType.BIRTH:
            BirthPayload.model_validate(self.payload)
        elif self.type == EventType.DEATH:
            DeathPayload.model_validate(self.payload)
        elif self.type == EventType.MIGRATION:
            MigrationPayload.model_validate(self.payload)
        elif self.type == EventType.CULTURE_DRIFT:
            CultureDriftPayload.model_validate(self.payload)
        elif self.type == EventType.BELIEF:
            BeliefPayload.model_validate(self.payload)
        elif self.type == EventType.LINEAGE_FOUNDED:
            LineageFoundedPayload.model_validate(self.payload)
        return self


class PopulationPool(StrictModel):
    """Per-settlement population time series. Phase 3a.4.

    `populations` has length `time_steps + 1`: index 0 is the
    initial population (carried over from `Settlement.population`
    via `3a.1` placement); indices 1..time_steps are post-step
    populations after births, deaths, and migrations."""

    settlement_id: int = Field(ge=0)
    populations: tuple[int, ...]


class MigrationRecord(StrictModel):
    """A migration edge firing at a specific time step. Phase 3a.4.

    `count` is the number of individuals moved along the road
    from `from_settlement_id` to `to_settlement_id` during step
    `step` (year index)."""

    id: int = Field(ge=0)
    from_settlement_id: int = Field(ge=0)
    to_settlement_id: int = Field(ge=0)
    step: int = Field(ge=0)
    count: int = Field(ge=0)
    road_cost: float = Field(ge=0.0)


class DemographyLayer(StrictModel):
    """Per-world demographic simulation output. Phase 3a.4.

    `pools` are parallel to `SettlementsLayer.settlements` by id
    (same length, same order). `migrations` records the per-step
    flows along road edges. `events` holds the typed
    `BIRTH / DEATH / MIGRATION` events emitted by the simulation;
    they live here in 3a.4 and are promoted to a top-level
    `events: EventLog` on `WorldModel` by Phase 3a.5."""

    pools: tuple[PopulationPool, ...]
    migrations: tuple[MigrationRecord, ...]
    events: tuple[WorldEvent, ...]


class EventLog(StrictModel):
    """Append-only event history. Phase 3a.5.

    Per `PHASE_3A_TYPES.md` adoption path step 3: `events: EventLog`
    is the top-level field on `WorldModel` that promotes the
    demography-emitted `BIRTH / DEATH / MIGRATION` events (and any
    future layer-emitted events) into a single typed history.

    Order is causal-stable: monotonic by `(t, id)` (asserted in the
    validator). `algorithm_version` is a blake2b hash of the events
    tuple so any re-ordering breaks the version — ties the log's
    ordering to the generator and surfaces silent mutations."""

    events: tuple[WorldEvent, ...]
    algorithm_version: str


class Culture(StrictModel):
    """A single culture (one per settlement in 3b.1 v1 slice).
    Phase 3b.1.

    `attribute_history` is a time series of attribute vectors
    (length `time_steps + 1`): index 0 is the initial vector (at
    step 0, before any drift), indices 1..time_steps are the
    post-step vectors after neighbor-correlation pull + stochastic
    perturbation. Each attribute vector has 6 entries per
    `CULTURE_ATTRIBUTE_NAMES` (`values`, `norms`, `taboos`,
    `ritual_forms`, `cuisine`, `music_motifs`), all in `[0, 1]`.

    Mirrors `PopulationPool.populations` from 3a.4 demography:
    parallel to settlements by index, with per-step history for
    Phase 5 causal-graph consumers."""

    settlement_id: int = Field(ge=0)
    attribute_history: tuple[tuple[float, ...], ...]


class CultureLayer(StrictModel):
    """Per-world culture-layer output. Phase 3b.1.

    `cultures` is parallel to `SettlementsLayer.settlements` by id
    (same length, same order). `algorithm_version` is a blake2b hash
    of every (settlement, step, attribute) value so any
    mutation / re-ordering breaks the version — ties the layer's
    state to the generator and surfaces silent mutations at the
    trust boundary (`WorldModel.model_validate_json`)."""

    cultures: tuple[Culture, ...]
    algorithm_version: str


class Ritual(StrictModel):
    """A single ritual practice within a settlement's religion.
    Phase 3b.2.

    One Ritual is born when a religion's ritual set grows (a new
    practice is sampled / adopted) and dies when it falls out of
    practice (the ritual set shrinks). `attested_from_step` and
    `attested_until_step` give the practice window; rituals that
    remain in practice at end-of-sim carry `attested_until_step =
    None`. Phase 4 polities and Phase 5 causal graph can refer to
    rituals by `id` for per-ritual provenance (suppression,
    adoption, syncretism)."""

    id: int = Field(ge=0)
    settlement_id: int = Field(ge=0)
    ritual_type: RitualType
    attested_from_step: int = Field(ge=0)
    attested_until_step: int | None = Field(default=None, ge=0)

    @pydantic.model_validator(mode="after")
    def _validate_attestation_window(self) -> "Ritual":
        if (
            self.attested_until_step is not None
            and self.attested_until_step < self.attested_from_step
        ):
            raise ValueError("attested_until_step cannot precede attested_from_step")
        return self


class Religion(StrictModel):
    """A single religion (one per settlement in 3b.2 v1 slice).
    Phase 3b.2.

    The 4-element schema follows spec line 191-194: pantheon size
    (int), ritual practices (tuple of `Ritual.id` references —
    NOT raw `RitualType` values, so per-ritual provenance is
    preserved), cosmology (CYCLE / LINEAR), eschatology
    (APOCALYPTIC / RENEWAL / CYCLICAL). Pantheon size, cosmology,
    and eschatology are stable structural elements (held across
    all time steps); only `ritual_practices` drifts (add / remove
    one ritual per step at `RELIGION_RITUAL_DRIFT_RATE`).

    Parallel to `SettlementsLayer.settlements` by index (same
    length, same order), mirroring the `Culture` / `PopulationPool`
    parallel-by-index pattern from 3b.1 / 3a.4."""

    settlement_id: int = Field(ge=0)
    pantheon_size: int = Field(ge=1)
    ritual_practices: tuple[int, ...]
    cosmology: Cosmology
    eschatology: Eschatology


class ReligionLayer(StrictModel):
    """Per-world religion-layer output. Phase 3b.2.

    `religions` is parallel to `SettlementsLayer.settlements` by id
    (same length, same order). `rituals` holds the full set of
    Ritual records ever attested (sorted by id) so a Phase 5
    causal-graph consumer can answer "which rituals were ever
    practiced in settlement S" without re-deriving from the BELIEF
    event log. `algorithm_version` is a blake2b hash of religions
    + rituals so any mutation / re-ordering breaks the version —
    ties the layer's state to the generator and surfaces silent
    mutations at the trust boundary (`WorldModel.model_validate_json`)."""

    religions: tuple[Religion, ...]
    rituals: tuple[Ritual, ...]
    algorithm_version: str


class KinshipSystem(StrEnum):
    """Kinship-system typology. Phase 3b.3 v1 ships all five entries
    so the 3b.5 acceptance test (no single dominant system) has
    non-trivial variety to test against. Order matches
    `KINSHIP_TYPOGRAPHY` rows: matrilineal / patrilineal / bilateral
    are the most prevalent real-world systems; avunculate and cognatic
    are rarer (some Pacific + Amazonian cultures).

    `StrEnum` per the chain's type convention (matches `EventType`,
    `BiomeClass`, `PortKind`, `RitualType`)."""

    MATRILINEAL = "matrilineal"
    PATRILINEAL = "patrilineal"
    BILATERAL = "bilateral"
    AVUNCULATE = "avunculate"
    COGNATIC = "cognatic"


class Lineage(StrictModel):
    """A single kinship lineage (one per settlement in 3b.3 v1 slice).
    Phase 3b.3.

    Parallel to `SettlementsLayer.settlements` by index (same length,
    same order). The lineage is the structural unit Phase 4 polities
    will reference for "founder lineage" and Phase 5 causal graph
    consumers will use to trace lineage -> individual causal chains
    via `founder_actor_id` (when present) plus the demography event
    log.

    `system` is sampled at `build_kinship` time from the biome's
    `KINSHIP_TYPOGRAPHY` weights and held stable for the v1 slice
    (no per-step system drift — that lands in 3b.3.x if Phase 4
    needs it). `depth` = mean generations of continuous lineage
    claim; `founding_step` = `0` (lineages are initial at
    world-generation time, not per-step events); `founder_actor_id`
    = optional reference to one living demography individual at
    step 0, sampled when one exists. Pattern `^[0-9a-f]{16}$` matches
    the existing `individual_id` format (16-char hex blake2b)."""

    id: int = Field(ge=0)
    settlement_id: int = Field(ge=0)
    system: KinshipSystem
    depth: int = Field(ge=KINSHIP_LINEAGE_DEPTH_MIN, le=KINSHIP_LINEAGE_DEPTH_MAX)
    founding_step: int = Field(ge=0)
    founder_actor_id: str | None = Field(default=None, pattern=r"^[0-9a-f]{16}$")


class NamePool(StrictModel):
    """A culture's name-pool. Phase 3b.3.

    Parallel to `CultureLayer.cultures` by index (same length, same
    order) — `culture_id` is the culture index, NOT the settlement
    id. v1 ships phoneme-templated `given_names` only; full lexicon
    + grammar arrive in 3b.4 via additive-not-breaking extension of
    the same model.

    `surname_patterns` and `epithets` are templated strings (e.g.,
    `"{prefix}{root}{suffix}"`) reserved for the 3b.4 phonology pass;
    v1 ships them as empty tuples. `given_names` length is
    biome-conditioned within
    `KINSHIP_NAMES_PER_CULTURE_MIN..KINSHIP_NAMES_PER_CULTURE_MAX`
    per `KINSHIP_NAMES_PER_CULTURE_BIAS`."""

    culture_id: int = Field(ge=0)
    given_names: tuple[str, ...]
    surname_patterns: tuple[str, ...] = ()
    epithets: tuple[str, ...] = ()


class KinshipLayer(StrictModel):
    """Per-world kinship-layer output. Phase 3b.3.

    `lineages` is parallel to `SettlementsLayer.settlements` by id
    (one lineage per settlement; intra-settlement only per spec line
    201-202 — polity-wide kinship is a Phase 4 concern).

    `name_pools` is parallel to `CultureLayer.cultures` by index
    (one name-pool per culture). Spec line 205 calls for
    "`NamePool` per culture" — culture_id is the culture index.

    `algorithm_version` is a blake2b hash of lineages + name_pools
    so any mutation / re-ordering breaks the version — ties the
    layer's state to the generator and surfaces silent mutations
    at the trust boundary (`WorldModel.model_validate_json`)."""

    lineages: tuple[Lineage, ...]
    name_pools: tuple[NamePool, ...]
    algorithm_version: str


class LineageFoundedPayload(StrictModel):
    """Discriminated payload for `EventType.LINEAGE_FOUNDED`.

    Emitted at `build_kinship` time, one per Lineage (parallel to
    `KinshipLayer.lineages` by index). The event records the
    structural fact (a lineage was founded at step T in settlement S
    with system K); names live on `NamePool` directly so this
    payload stays minimal. Phase 5 causal graph can join on
    `lineage_id` to answer "founder of settlement S lineage"
    queries. `step` mirrors `WorldEvent.t`.

    `system` is stored as `str` (not `KinshipSystem`) so the
    payload survives JSON round-trip through `WorldModel.model_dump`
    / `WorldModel.model_validate_json` with `strict=False` (the
    Persistence trust-boundary contract). The `_validate_system`
    model_validator enforces that the value is one of the
    `KinshipSystem` enum members, preserving the type contract at
    construction time."""

    lineage_id: int = Field(ge=0)
    settlement_id: int = Field(ge=0)
    system: str
    founding_step: int = Field(ge=0)
    step: int = Field(ge=0)

    @pydantic.model_validator(mode="after")
    def _validate_system(self) -> "LineageFoundedPayload":
        valid_values = {member.value for member in KinshipSystem}
        if self.system not in valid_values:
            raise ValueError(
                f"system {self.system!r} is not one of {sorted(valid_values)}"
            )
        return self


class WorldModel(StrictModel):
    """Composable root contract shared by generation and simulation layers."""

    metadata: WorldMetadata
    geology: GeologyLayer
    geography: GeographyLayer
    hydrology: HydrologyLayer
    climate: ClimateLayer
    biomes: BiomeLayer
    astronomy: AstronomyLayer
    biology: BiologyLayer
    settlements: SettlementsLayer
    agriculture: AgricultureLayer
    infrastructure: InfrastructureLayer
    demography: DemographyLayer
    events: EventLog
    cultures: CultureLayer
    religions: ReligionLayer
    kinship: KinshipLayer
    provenance: tuple[ProvenanceRecord, ...]
