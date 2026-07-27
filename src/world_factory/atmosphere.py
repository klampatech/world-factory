"""Atmospheric pressure grid derived from elevation. Phase 1a first PR
ships the barometric-formula approximation; Phase 1c replaces it with
the full atmosphere recursion (composition, prevailing winds, storms)."""

import math

from world_factory.constants import (
    ATMOSPHERIC_SCALE_HEIGHT_METERS,
    MAXIMUM_ELEVATION_METERS,
    MINIMUM_ATMOSPHERIC_PRESSURE_KPA,
    MINIMUM_ELEVATION_METERS,
    STANDARD_ATMOSPHERIC_PRESSURE_KPA,
)

FloatGrid = tuple[tuple[float, ...], ...]


def atmospheric_pressure_grid(elevation_meters: FloatGrid) -> FloatGrid:
    """Per-cell atmospheric pressure via the barometric formula:
    P(h) = P_0 * exp(-h / H) for h >= 0, P_0 * (1 + |h|/H_max) for
    h < 0 (linear extrapolation into deep ocean basins)."""

    height = len(elevation_meters)
    width = len(elevation_meters[0])
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
            pressure = max(MINIMUM_ATMOSPHERIC_PRESSURE_KPA, pressure)
            row.append(round(pressure, 6))
        grid.append(row)
    return tuple(tuple(row) for row in grid)