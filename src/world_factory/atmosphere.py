"""Atmospheric circulation, moisture transport, and refined precipitation.

Phase 1c replaces the Phase 1a barometric pressure stub with a full
atmospheric model that derives:

1. **Per-cell atmospheric pressure** — barometric formula on elevation,
   with a small humidity buoyancy correction so moist air reads lighter
   than dry air at the same elevation.
2. **Prevailing surface winds** — three-cell circulation (Hadley at
   0–30°, Ferrel at 30–60°, polar easterlies at 60–90°). Coastal cells
   receive a sea-breeze modulation based on adjacent temperature
   contrast.
3. **Specific humidity** — evaporation from ocean cells (Magnus–Tetens
   saturation vapor pressure scaled by wind coefficient), then wind-
   driven transport over `TRANSPORT_ITERATIONS` iterations until
   convergence. Each cell emits `incoming × (1 − precipitation_loss)`
   to its downwind neighbour and loses a small amount to local
   precipitation along the way.
4. **Refined precipitation** — blends the Phase 1a noise field with
   transport-driven moisture; high-elevation cells receive an
   orographic boost.

All outputs are deterministic given (seed, geography, climate) and use
only the stdlib.
"""

import math

from world_factory.constants import (
    ATMOSPHERE_ALGORITHM_VERSION,
    ATMOSPHERIC_SCALE_HEIGHT_METERS,
    BASE_PRECIPITATION_LOSS,
    EVAPORATION_WIND_COEFFICIENT,
    MAXIMUM_ELEVATION_METERS,
    MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG,
    MINIMUM_ATMOSPHERIC_PRESSURE_KPA,
    MINIMUM_ELEVATION_METERS,
    OROGRAPHIC_BOOST_DIVISOR_METERS,
    PRECIPITATION_REFINEMENT_BLEND,
    PRESSURE_HUMIDITY_BUOYANCY,
    SEA_BREEZE_TEMPERATURE_DELTA_CELSIUS,
    STANDARD_ATMOSPHERIC_PRESSURE_KPA,
    TRANSPORT_ITERATIONS,
    WIND_BELT_FERREL_DEGREES,
    WIND_BELT_HADLEY_DEGREES,
)
from world_factory.models import ProvenanceRecord, WindDirection

FloatGrid = tuple[tuple[float, ...], ...]
WindGrid = tuple[tuple[WindDirection, ...], ...]


def _saturation_vapor_pressure_kpa(temperature_celsius: float) -> float:
    """Magnus–Tetens approximation for saturation vapor pressure in kPa."""
    return 0.6108 * math.exp((17.27 * temperature_celsius) / (temperature_celsius + 237.3))


def _wind_offset(direction: WindDirection) -> tuple[int, int]:
    """Return the (dx, dy) grid offset the wind blows TOWARD."""
    return {
        WindDirection.EAST: (1, 0),
        WindDirection.WEST: (-1, 0),
        WindDirection.NORTH: (0, -1),
        WindDirection.SOUTH: (0, 1),
        WindDirection.NORTH_EAST: (1, -1),
        WindDirection.NORTH_WEST: (-1, -1),
        WindDirection.SOUTH_EAST: (1, 1),
        WindDirection.SOUTH_WEST: (-1, 1),
        WindDirection.CALM: (0, 0),
    }[direction]


def _latitude_degrees(y: int, height: int) -> float:
    """Latitude from row index. y=0 is south pole (-90 deg), y=height-1
    is north pole (+90 deg)."""
    return -90.0 + (y + 0.5) / height * 180.0


def _prevailing_belt_wind(latitude_degrees: float) -> WindDirection:
    """Three-cell surface wind belt by latitude.

    Wind direction is the direction the wind blows TOWARD (so
    WindDirection.WEST = wind blows toward the west, i.e. an easterly
    trade wind blowing FROM east TO west). Hadley cell surface flows
    are easterly trades (blow west); Ferrel cell surface flows are
    westerlies (blow east); polar cell surface flows are polar
    easterlies (blow west).
    """
    absolute_latitude = abs(latitude_degrees)
    if absolute_latitude < WIND_BELT_HADLEY_DEGREES:
        return WindDirection.WEST
    if absolute_latitude < WIND_BELT_FERREL_DEGREES:
        return WindDirection.EAST
    return WindDirection.WEST


def _wind_direction_grid(
    elevation: FloatGrid,
    temperature: FloatGrid,
    sea_level: float,
) -> WindGrid:
    """Prevailing surface wind per cell. Belt assignment by latitude;
    coastal cells get a sea-breeze modulation when adjacent ocean is
    significantly cooler or warmer than the land."""
    height = len(elevation)
    width = len(elevation[0])
    grid: list[list[WindDirection]] = [
        [WindDirection.CALM] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            base = _prevailing_belt_wind(_latitude_degrees(y, height))
            elevation_self = elevation[y][x]
            temperature_self = temperature[y][x]
            sea_breeze: WindDirection | None = None
            if elevation_self > sea_level:
                warmest_ocean_delta = -math.inf
                coldest_ocean_delta = math.inf
                for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                    nx, ny = x + dx, y + dy
                    if not (0 <= nx < width and 0 <= ny < height):
                        continue
                    if elevation[ny][nx] <= sea_level:
                        delta = temperature_self - temperature[ny][nx]
                        warmest_ocean_delta = max(warmest_ocean_delta, delta)
                        coldest_ocean_delta = min(coldest_ocean_delta, delta)
                if warmest_ocean_delta > SEA_BREEZE_TEMPERATURE_DELTA_CELSIUS:
                    sea_breeze = (
                        WindDirection.SOUTH
                        if _latitude_degrees(y, height) < 0
                        else WindDirection.NORTH
                    )
                elif coldest_ocean_delta < -SEA_BREEZE_TEMPERATURE_DELTA_CELSIUS:
                    sea_breeze = (
                        WindDirection.NORTH
                        if _latitude_degrees(y, height) < 0
                        else WindDirection.SOUTH
                    )
            grid[y][x] = sea_breeze if sea_breeze is not None else base
    return tuple(tuple(row) for row in grid)


def _evaporation_grid(
    elevation: FloatGrid,
    temperature: FloatGrid,
    sea_level: float,
    wind: WindGrid,
) -> FloatGrid:
    """Per-cell evaporation in kg/kg units of specific humidity. Ocean
    cells contribute; land cells contribute zero."""
    height = len(elevation)
    width = len(elevation[0])
    grid: list[list[float]] = [[0.0] * width for _ in range(height)]
    for y in range(height):
        for x in range(width):
            if elevation[y][x] > sea_level:
                continue
            wind_dir = wind[y][x]
            wind_speed_factor = 0.5 if wind_dir is WindDirection.CALM else 1.0
            saturation = _saturation_vapor_pressure_kpa(temperature[y][x])
            evaporation = saturation * EVAPORATION_WIND_COEFFICIENT * wind_speed_factor
            grid[y][x] = min(
                MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG,
                evaporation / STANDARD_ATMOSPHERIC_PRESSURE_KPA,
            )
    return tuple(tuple(row) for row in grid)


def _transport_humidity(
    humidity: FloatGrid,
    elevation: FloatGrid,
    wind: WindGrid,
    sea_level: float,
) -> FloatGrid:
    """Steady-state humidity field via bounded iterative transport.

    Each iteration, each cell emits a fraction of its current humidity
    to its downwind neighbour and retains the remainder minus a small
    local precipitation loss. The emission fraction is bounded so the
    cell never ships more than it holds — this prevents exponential
    blow-up across many iterations.
    """
    height = len(elevation)
    width = len(elevation[0])
    humidity_grid: list[list[float]] = [list(row) for row in humidity]
    emission_fraction = 1.0 - BASE_PRECIPITATION_LOSS
    for _ in range(TRANSPORT_ITERATIONS):
        new_grid: list[list[float]] = [
            [BASE_PRECIPITATION_LOSS * value for value in row]
            for row in humidity_grid
        ]
        for y in range(height):
            for x in range(width):
                wind_dir = wind[y][x]
                if wind_dir is WindDirection.CALM:
                    continue
                dx, dy = _wind_offset(wind_dir)
                nx, ny = x + dx, y + dy
                if not (0 <= nx < width and 0 <= ny < height):
                    continue
                emitted = humidity_grid[y][x] * emission_fraction
                new_grid[ny][nx] += emitted
        humidity_grid = [
            [min(MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG, value) for value in row]
            for row in new_grid
        ]
    return tuple(tuple(row) for row in humidity_grid)


def _refined_precipitation(
    base_precipitation: FloatGrid,
    humidity: FloatGrid,
    elevation: FloatGrid,
    sea_level: float,
) -> FloatGrid:
    """Blend the Phase 1a noise field with transport-driven humidity and
    an orographic multiplier."""
    height = len(elevation)
    width = len(elevation[0])
    max_humidity = max(max(row) for row in humidity) or 1.0
    grid: list[list[float]] = []
    for y in range(height):
        row: list[float] = []
        for x in range(width):
            humidity_precip = (humidity[y][x] / max_humidity) * 2_500.0
            orographic_multiplier = 1.0 + math.tanh(
                max(elevation[y][x] - sea_level, 0.0) / OROGRAPHIC_BOOST_DIVISOR_METERS
            ) * 0.5
            base = base_precipitation[y][x]
            refined = (
                base * (1.0 - PRECIPITATION_REFINEMENT_BLEND)
                + humidity_precip * PRECIPITATION_REFINEMENT_BLEND
            ) * orographic_multiplier
            row.append(round(max(refined, 0.0), 6))
        grid.append(row)
    return tuple(tuple(row) for row in grid)


def atmospheric_pressure_grid(
    elevation_meters: FloatGrid, humidity: FloatGrid
) -> FloatGrid:
    """Per-cell atmospheric pressure via the barometric formula, with a
    small humidity buoyancy correction so moist air reads lighter than
    dry air at the same elevation."""
    height = len(elevation_meters)
    width = len(elevation_meters[0])
    max_humidity = max(max(row) for row in humidity) or MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG
    grid: list[list[float]] = []
    for y in range(height):
        row: list[float] = []
        for x in range(width):
            elevation = elevation_meters[y][x]
            elevation = max(MINIMUM_ELEVATION_METERS, min(MAXIMUM_ELEVATION_METERS, elevation))
            if elevation >= 0.0:
                pressure = STANDARD_ATMOSPHERIC_PRESSURE_KPA * math.exp(
                    -elevation / ATMOSPHERIC_SCALE_HEIGHT_METERS
                )
            else:
                pressure = STANDARD_ATMOSPHERIC_PRESSURE_KPA * (
                    1.0 + (-elevation) / ATMOSPHERIC_SCALE_HEIGHT_METERS
                )
            buoyancy = 1.0 - PRESSURE_HUMIDITY_BUOYANCY * (
                humidity[y][x] / max_humidity
            )
            pressure = max(MINIMUM_ATMOSPHERIC_PRESSURE_KPA, pressure * buoyancy)
            row.append(round(pressure, 6))
        grid.append(row)
    return tuple(tuple(row) for row in grid)


def refine_climate(
    elevation: FloatGrid,
    temperature: FloatGrid,
    base_precipitation: FloatGrid,
    sea_level: float,
) -> tuple[FloatGrid, WindGrid, FloatGrid, FloatGrid]:
    """Produce the Phase 1c climate fields. Returns
    (pressure, wind, humidity, refined_precipitation) in that order."""
    wind = _wind_direction_grid(elevation, temperature, sea_level)
    evaporation = _evaporation_grid(elevation, temperature, sea_level, wind)
    humidity = _transport_humidity(evaporation, elevation, wind, sea_level)
    refined_precip = _refined_precipitation(base_precipitation, humidity, elevation, sea_level)
    pressure = atmospheric_pressure_grid(elevation, humidity)
    return pressure, wind, humidity, refined_precip


def atmosphere_provenance() -> ProvenanceRecord:
    """Provenance record describing the atmospheric algorithm."""
    return ProvenanceRecord(
        output_path="climate",
        process="wind-belts-with-transport",
        input_paths=(
            "geography.elevation_meters",
            "climate.temperature_celsius",
            "climate.annual_precipitation_mm",
            "metadata.config.seed",
        ),
        algorithm_version=ATMOSPHERE_ALGORITHM_VERSION,
    )