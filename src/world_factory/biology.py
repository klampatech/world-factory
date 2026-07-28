"""Phase 2 biology: per-cell flora and fauna assignments by biome.

The mapping is biome-driven: each biome in the existing `BiomeLayer`
has a characteristic flora and fauna that drive the
`BiologyLayer.grids`. Ocean cells (elevation ≤ sea_level) capture
their marine biota via the `ALGAE` flora + `FISH` fauna defaults.
"""

from world_factory.constants import BIOLOGY_ALGORITHM_VERSION
from world_factory.invariants import InvariantViolation
from world_factory.invariants import violation as _violation
from world_factory.models import (
    BiologyLayer,
    BiomeClass,
    FaunaType,
    FloraType,
    ProvenanceRecord,
    WorldModel,
)

_BIOME_FLORA: dict[BiomeClass, FloraType] = {
    BiomeClass.OCEAN: FloraType.ALGAE,
    BiomeClass.ICE: FloraType.LICHEN,
    BiomeClass.ALPINE: FloraType.MOSS,
    BiomeClass.DESERT: FloraType.SHRUB,
    BiomeClass.TROPICAL_FOREST: FloraType.BROADLEAF,
    BiomeClass.TEMPERATE_FOREST: FloraType.BROADLEAF,
    BiomeClass.GRASSLAND: FloraType.GRASS,
}

_BIOME_FAUNA: dict[BiomeClass, FaunaType] = {
    BiomeClass.OCEAN: FaunaType.FISH,
    BiomeClass.ICE: FaunaType.BIRD,
    BiomeClass.ALPINE: FaunaType.HERBIVORE_SMALL,
    BiomeClass.DESERT: FaunaType.REPTILE,
    BiomeClass.TROPICAL_FOREST: FaunaType.BIRD,
    BiomeClass.TEMPERATE_FOREST: FaunaType.HERBIVORE_LARGE,
    BiomeClass.GRASSLAND: FaunaType.HERBIVORE_LARGE,
}


def _grid_height_width(
    classifications: tuple[tuple[BiomeClass, ...], ...],
) -> tuple[int, int]:
    height = len(classifications)
    width = len(classifications[0]) if classifications else 0
    return height, width


def build_biology(
    classifications: tuple[tuple[BiomeClass, ...], ...],
    elevation: "FloatGrid",
    sea_level: float,
) -> BiologyLayer:
    """Build the Phase 2 biology layer from the biome grid.

    Each cell carries a primary flora species (or `None` for ocean
    cells that capture their biota via the ALGAE / FISH defaults)
    and a primary fauna species. Ocean cells override biome
    defaults with marine biota.
    """
    height, width = _grid_height_width(classifications)
    flora_grid: list[list[FloraType | None]] = [
        [None] * width for _ in range(height)
    ]
    fauna_grid: list[list[FaunaType | None]] = [
        [None] * width for _ in range(height)
    ]
    for y in range(height):
        for x in range(width):
            biome = classifications[y][x]
            if elevation[y][x] <= sea_level:
                flora_grid[y][x] = _BIOME_FLORA.get(biome, FloraType.ALGAE)
                fauna_grid[y][x] = _BIOME_FAUNA.get(biome, FaunaType.FISH)
                continue
            flora_grid[y][x] = _BIOME_FLORA.get(biome, FloraType.GRASS)
            fauna_grid[y][x] = _BIOME_FAUNA.get(biome, FaunaType.HERBIVORE_SMALL)
    return BiologyLayer(
        flora_grid=tuple(tuple(row) for row in flora_grid),
        fauna_grid=tuple(tuple(row) for row in fauna_grid),
    )


def biology_provenance() -> ProvenanceRecord:
    """Provenance record describing the biology algorithm."""
    return ProvenanceRecord(
        output_path="biology",
        process="biome-driven-biota",
        input_paths=(
            "biomes.classifications",
            "geography.elevation_meters",
            "geography.sea_level_meters",
        ),
        algorithm_version=BIOLOGY_ALGORITHM_VERSION,
    )


FloatGrid = tuple[tuple[float, ...], ...]


def validate_biology_layer(world: WorldModel) -> list[InvariantViolation]:
    """Phase 2 biology grid-shape and StrEnum-validity checks."""
    violations: list[InvariantViolation] = []
    height = world.geography.height
    valid_flora = set(FloraType)
    valid_fauna = set(FaunaType)
    if len(world.biology.flora_grid) != height:
        violations.append(
            _violation(
                "flora-grid-shape",
                "biology.flora_grid",
                f"expected {height} rows, found {len(world.biology.flora_grid)}",
            )
        )
    if len(world.biology.fauna_grid) != height:
        violations.append(
            _violation(
                "fauna-grid-shape",
                "biology.fauna_grid",
                f"expected {height} rows, found {len(world.biology.fauna_grid)}",
            )
        )
    for y, flora_row in enumerate(world.biology.flora_grid):
        for x, flora_value in enumerate(flora_row):
            if flora_value is not None and flora_value not in valid_flora:
                violations.append(
                    _violation(
                        "flora-invalid",
                        f"biology.flora_grid[{y}][{x}]",
                        f"flora {flora_value!r} is not a valid FloraType",
                    )
                )
    for y, fauna_row in enumerate(world.biology.fauna_grid):
        for x, fauna_value in enumerate(fauna_row):
            if fauna_value is not None and fauna_value not in valid_fauna:
                violations.append(
                    _violation(
                        "fauna-invalid",
                        f"biology.fauna_grid[{y}][{x}]",
                        f"fauna {fauna_value!r} is not a valid FaunaType",
                    )
                )
    return violations