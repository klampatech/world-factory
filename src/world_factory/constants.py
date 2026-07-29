"""Versioned constants shared across World Factory modules."""

SCHEMA_VERSION = "16.0.0"
MODEL_VERSION = "phase-3b.4"
DETERMINISTIC_ALGORITHM_VERSION = "tectonic-plates-v1"
HYDROLOGY_ALGORITHM_VERSION = "flow-routing-v1"
ATMOSPHERE_ALGORITHM_VERSION = "wind-belts-v1"
ASTRONOMY_ALGORITHM_VERSION = "axial-tilt-v1"
GEOLOGY_SUBLEYER_ALGORITHM_VERSION = "rock-ore-soil-v1"
BIOLOGY_ALGORITHM_VERSION = "biome-biota-v1"
SETTLEMENTS_ALGORITHM_VERSION = "candidate-scoring-v1"
CANONICAL_DEMO_SEED = 42
MINIMUM_SEED = 0
MAXIMUM_SEED = (1 << 64) - 1
MINIMUM_ELEVATION_METERS = -11_000.0
MAXIMUM_ELEVATION_METERS = 9_000.0
MINIMUM_SURFACE_TEMPERATURE_CELSIUS = -120.0
MAXIMUM_SURFACE_TEMPERATURE_CELSIUS = 80.0
MINIMUM_OCEAN_FRACTION = 0.1
MAXIMUM_OCEAN_FRACTION = 0.9
STANDARD_ATMOSPHERIC_PRESSURE_KPA = 101.325
MINIMUM_ATMOSPHERIC_PRESSURE_KPA = 1.0
ATMOSPHERIC_SCALE_HEIGHT_METERS = 8_400.0
MINIMUM_PLATE_COUNT = 3
MAXIMUM_PLATE_COUNT = 64
MINIMUM_PLATE_INTERIOR_CELL_COUNT = 4
CONVERGENT_BOUNDARY_UPLIFT_METERS = 4_800.0
DIVERGENT_BOUNDARY_RIFT_METERS = -2_400.0
CONTINENTAL_INTERIOR_BASE_ELEVATION_METERS = 900.0
OCEANIC_INTERIOR_BASE_ELEVATION_METERS = -1_200.0
ELEVATION_NOISE_RANGE_METERS = 1_000.0
MINIMUM_HEADWATER_BASIN_CELLS = 4
MINIMUM_RUNOFF_PRECIPITATION_MM = 250.0
MINIMUM_HEADWATER_ELEVATION_METERS = 750.0
RUNOFF_COEFFICIENT = 0.35
GRID_CELL_AREA_KILOMETERS_SQUARED = 80.0
MILLIMETERS_PER_YEAR_PER_CELL_KILOMETER_SQUARED = 1.0
HYDROLOGY_DISCHARGE_SCALE = 1.0e-6
MAXIMUM_SPECIFIC_HUMIDITY_KG_PER_KG = 0.030
WIND_BELT_HADLEY_DEGREES = 30.0
WIND_BELT_FERREL_DEGREES = 60.0
SEA_BREEZE_TEMPERATURE_DELTA_CELSIUS = 4.0
EVAPORATION_WIND_COEFFICIENT = 0.06
TRANSPORT_ITERATIONS = 32
BASE_PRECIPITATION_LOSS = 0.04
OROGRAPHIC_BOOST_DIVISOR_METERS = 1_000.0
PRESSURE_HUMIDITY_BUOYANCY = 0.01
PRECIPITATION_REFINEMENT_BLEND = 0.5
SEASONAL_TEMPERATURE_AMPLITUDE = 0.10
EARTH_AXIAL_TILT_DEGREES = 23.5
EARTH_ORBITAL_ECCENTRICITY = 0.0167
EARTH_ROTATION_PERIOD_HOURS = 24.0
EARTH_ORBITAL_PERIOD_DAYS = 365.25
MINIMUM_ORE_PROBABILITY = 0.10
ORE_PROBABILITY_SCALE = 0.4
SEDIMENTARY_ELEVATION_CAP_METERS = 500.0
PEAT_PRECIPITATION_THRESHOLD_MM = 1_400.0
LOAM_PRECIPITATION_THRESHOLD_MM = 350.0
PERMAFROST_TEMPERATURE_CELSIUS = -10.0
SETTLEMENT_CANDIDATE_GRID_DIVISOR = 16
SETTLEMENT_MIN_COUNT = 20
SETTLEMENT_PER_PLATE_COUNT = 3
SETTLEMENT_MIN_SPACING_CELLS = 4
SETTLEMENT_DEFENSIBILITY_LOW_METERS = 200.0
SETTLEMENT_DEFENSIBILITY_HIGH_METERS = 1500.0
SETTLEMENT_CLIMATE_LOW_CELSIUS = 5.0
SETTLEMENT_CLIMATE_HIGH_CELSIUS = 25.0
SETTLEMENT_POPULATION_ARABLE_BASE = 1000
SETTLEMENT_POPULATION_WATER_BONUS = 500
SETTLEMENT_POPULATION_MINERAL_BONUS = 200
SETTLEMENT_DEFENSIBILITY_RAMP_METERS = 1500.0
SETTLEMENT_CLIMATE_RAMP_COLD_CELSIUS = 25.0
SETTLEMENT_CLIMATE_RAMP_HOT_CELSIUS = 20.0
SETTLEMENT_CLIMATE_LOWER_BOUND_CELSIUS = -20.0
SETTLEMENT_WATER_DECAY_DIVISOR = 8.0
AGRICULTURE_ALGORITHM_VERSION = "caloric-accounting-v1"
AGRICULTURE_EXTRACTION_RADIUS_CELLS = 10
AGRICULTURE_BASE_YIELD_TONNES_PER_CELL = 4.0
AGRICULTURE_PRECIPITATION_OPTIMUM_MM = 1_000.0
AGRICULTURE_TEMPERATURE_OPTIMUM_CELSIUS = 18.0
AGRICULTURE_TEMPERATURE_RANGE_CELSIUS = 18.0
AGRICULTURE_SOIL_QUALITY = {
    "loam": 1.0,
    "clay": 0.7,
    "peat": 0.5,
    "sand": 0.3,
    "permafrost": 0.0,
}
AGRICULTURE_BIOME_QUALITY = {
    "temperate-forest": 1.0,
    "grassland": 0.9,
    "tropical-forest": 0.8,
    "desert": 0.1,
    "alpine": 0.1,
    "ice": 0.0,
    "ocean": 0.0,
}
AGRICULTURE_CALORIC_KCAL_PER_TONNE = 3_000_000.0
AGRICULTURE_KCAL_PER_PERSON_PER_YEAR = 800_000.0
AGRICULTURE_DEFICIT_YIELD_FRACTION = 0.25
AGRICULTURE_MINIMUM_ARABLE_CELLS = 1
INFRASTRUCTURE_ALGORITHM_VERSION = "min-cost-friction-v1"
INFRASTRUCTURE_IMPASSABLE = 1.0e9
INFRASTRUCTURE_BASE_FRICTION_PER_BIOME = {
    "grassland": 1.0,
    "temperate-forest": 1.5,
    "tropical-forest": 1.7,
    "desert": 2.0,
    "alpine": 2.5,
    "ice": 1.0e9,
    "ocean": 1.0e9,
}
INFRASTRUCTURE_SLOPE_PENALTY_PER_METER = 0.0015
INFRASTRUCTURE_RIVER_CROSSING_PENALTY = 6.0
INFRASTRUCTURE_DIAGONAL_COST = 1.4142135623730951
INFRASTRUCTURE_ROAD_NEIGHBOR_K = 3
INFRASTRUCTURE_COASTAL_RADIUS_CELLS = 1
INFRASTRUCTURE_RIVER_PROXIMITY_RADIUS_CELLS = 2
INFRASTRUCTURE_PORT_TONNAGE_THRESHOLD = 1.0
INFRASTRUCTURE_PORT_TONNAGE_PER_POPULATION = 1.0
INFRASTRUCTURE_MAX_CANALS = 8
INFRASTRUCTURE_CANAL_SLOPE_LIMIT_M_PER_CELL = 5000.0
INFRASTRUCTURE_CANAL_MIN_FLOW = 50_000_000.0
DEMOGRAPHY_ALGORITHM_VERSION = "aggregate-pools-v1"
DEMOGRAPHY_DEFAULT_TIME_STEPS = 50
DEMOGRAPHY_BASE_BIRTH_RATE = 0.07
DEMOGRAPHY_BASE_DEATH_RATE = 0.018
DEMOGRAPHY_CAPACITY_HEADROOM_BIRTH_BOOST = 1.0
DEMOGRAPHY_OVER_CAPACITY_DEATH_PENALTY = 0.001
DEMOGRAPHY_MAX_DEATH_RATE = 0.06
DEMOGRAPHY_CLIMATE_OPTIMUM_CELSIUS = 18.0
DEMOGRAPHY_CLIMATE_RANGE_CELSIUS = 18.0
DEMOGRAPHY_CONFLICT_THRESHOLD = 0.6
DEMOGRAPHY_CONFLICT_DEATH_MULTIPLIER = 1.5
DEMOGRAPHY_MIGRATION_PRESSURE_FACTOR = 0.10
DEMOGRAPHY_MIGRATION_PULL_FACTOR = 0.05
DEMOGRAPHY_MIGRATION_COST_DIVISOR = 50.0
EVENT_LOG_ALGORITHM_VERSION = "event-log-v1"
CULTURE_ALGORITHM_VERSION = "neighbor-correlated-drift-v1"
CULTURE_ATTRIBUTE_NAMES = (
    "values",
    "norms",
    "taboos",
    "ritual_forms",
    "cuisine",
    "music_motifs",
)
CULTURE_DRIFT_TIME_STEPS = 50
CULTURE_DRIFT_RATE = 0.02
CULTURE_DRIFT_PULL = 0.05
CULTURE_NEIGHBOR_K = 3
CULTURE_PER_ATTR_NOISE = 0.05
CULTURE_BIOME_BIAS_TABLE = {
    "ocean": (0.3, 0.5, 0.6, 0.4, 0.6, 0.4),
    "ice": (0.2, 0.6, 0.7, 0.3, 0.5, 0.2),
    "alpine": (0.4, 0.6, 0.6, 0.5, 0.3, 0.5),
    "desert": (0.5, 0.7, 0.6, 0.7, 0.4, 0.5),
    "tropical-forest": (0.5, 0.5, 0.4, 0.7, 0.7, 0.8),
    "temperate-forest": (0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
    "grassland": (0.5, 0.5, 0.5, 0.5, 0.6, 0.4),
}
RELIGION_ALGORITHM_VERSION = "biome-history-bias-v1"
RELIGION_DRIFT_TIME_STEPS = 50
RELIGION_INITIAL_RITUAL_COUNT_MIN = 3
RELIGION_INITIAL_RITUAL_COUNT_MAX = 5
RELIGION_RITUAL_DRIFT_RATE = 0.05
RELIGION_PRESSURE_WINDOW_STEPS = 10
# Recent-death-rate bucket thresholds. Pinned absolutely so the
# chi-square acceptance test (arid vs tropical water-ritual frequency)
# is deterministic across seeds. low = peaceful, mid = stressed,
# high = catastrophe-prone.
RELIGION_DEATH_RATE_LOW_THRESHOLD = 0.05
RELIGION_DEATH_RATE_HIGH_THRESHOLD = 0.15
RELIGION_BIOME_PANTHEON_RANGE = {
    "ocean": (3, 8),
    "ice": (2, 6),
    "alpine": (1, 4),
    "desert": (1, 5),
    "tropical-forest": (4, 12),
    "temperate-forest": (3, 8),
    "grassland": (3, 9),
}
# Per-biome ritual prevalence. Probabilities sum to 1.0 per biome.
# Order matches `RitualType`: WATER, HARVEST, FIRE, ANCESTOR, SKY, EARTH.
# Per spec: arid -> water rituals (desert carries the bias). Tropical /
# grassland biomes skew toward harvest + earth. Ice / alpine skew toward
# fire (cold) and sky (altitude).
RELIGION_BIOME_RITUAL_BIAS = {
    "ocean": (0.10, 0.30, 0.10, 0.20, 0.20, 0.10),
    "ice": (0.05, 0.20, 0.25, 0.15, 0.20, 0.15),
    "alpine": (0.15, 0.20, 0.15, 0.15, 0.25, 0.10),
    "desert": (0.50, 0.05, 0.10, 0.05, 0.20, 0.10),
    "tropical-forest": (0.15, 0.25, 0.05, 0.20, 0.10, 0.25),
    "temperate-forest": (0.15, 0.15, 0.15, 0.20, 0.15, 0.20),
    "grassland": (0.15, 0.20, 0.10, 0.20, 0.20, 0.15),
}
# Per-biome cosmology bias. Higher LINEAR weight for arid / cold biomes
# (linear-progress eschatology, more common in monotheistic traditions
# in harsh environments); higher CYCLE for lush biomes (renewal cycles).
RELIGION_BIOME_COSMOLOGY_BIAS = {
    "ocean": {"cycle": 0.7, "linear": 0.3},
    "ice": {"cycle": 0.4, "linear": 0.6},
    "alpine": {"cycle": 0.4, "linear": 0.6},
    "desert": {"cycle": 0.3, "linear": 0.7},
    "tropical-forest": {"cycle": 0.8, "linear": 0.2},
    "temperate-forest": {"cycle": 0.6, "linear": 0.4},
    "grassland": {"cycle": 0.7, "linear": 0.3},
}
# Per-conflict-rate eschatology bias. High recent-death-rate
# settlements skew apocalyptic (catastrophe-driven); low-death
# settlements skew renewal. Probabilities sum to 1.0 per bucket.
RELIGION_HISTORY_ESCHATOLOGY_BIAS = {
    "low": {"renewal": 0.5, "cyclical": 0.4, "apocalyptic": 0.1},
    "mid": {"renewal": 0.3, "cyclical": 0.4, "apocalyptic": 0.3},
    "high": {"renewal": 0.1, "cyclical": 0.2, "apocalyptic": 0.7},
}

# Phase 3b.3 — Kinship + naming. Algorithm-version-named suffix.
KINSHIP_ALGORITHM_VERSION = "lineage-typology-v1"
KINSHIP_LINEAGE_DEPTH_MIN = 3
KINSHIP_LINEAGE_DEPTH_MAX = 6
# Per-culture name-pool size bounds; biome-conditioned within range.
KINSHIP_NAMES_PER_CULTURE_MIN = 12
KINSHIP_NAMES_PER_CULTURE_MAX = 36
# Biome-conditioned (min, max) tuple per BiomeClass. Lush biomes
# get more names (rich cultural vocabulary); arid / ice get fewer.
KINSHIP_NAMES_PER_CULTURE_BIAS: dict[str, tuple[int, int]] = {
    "ocean": (12, 18),
    "ice": (12, 18),
    "alpine": (12, 20),
    "desert": (12, 18),
    "tropical-forest": (20, 36),
    "temperate-forest": (24, 36),
    "grassland": (15, 24),
}
# Per-biome kinship-system typology weights. Probabilities sum to
# 1.0 per biome. Order matches KinshipSystem enum:
# MATRILINEAL, PATRILINEAL, BILATERAL, AVUNCULATE, COGNATIC.
KINSHIP_TYPOGRAPHY: dict[str, tuple[float, float, float, float, float]] = {
    "ocean": (0.20, 0.40, 0.25, 0.10, 0.05),
    "ice": (0.30, 0.30, 0.25, 0.10, 0.05),
    "alpine": (0.20, 0.45, 0.20, 0.10, 0.05),
    "desert": (0.30, 0.40, 0.20, 0.05, 0.05),
    "tropical-forest": (0.45, 0.20, 0.25, 0.05, 0.05),
    "temperate-forest": (0.25, 0.30, 0.35, 0.05, 0.05),
    "grassland": (0.30, 0.40, 0.20, 0.05, 0.05),
}
# Phoneme inventory for templated given-names. 13 consonant/vowel
# groups × 5 vowel substitutions = 65 entries.
KINSHIP_NAME_PHONEMES: tuple[str, ...] = (
    "a", "e", "i", "o", "u",
    "ka", "ke", "ki", "ko", "ku",
    "ra", "re", "ri", "ro", "ru",
    "ta", "te", "ti", "to", "tu",
    "na", "ne", "ni", "no", "nu",
    "ma", "me", "mi", "mo", "mu",
    "sa", "se", "si", "so", "su",
    "la", "le", "li", "lo", "lu",
    "ba", "be", "bi", "bo", "bu",
    "ga", "ge", "gi", "go", "gu",
    "ha", "he", "hi", "ho", "hu",
    "pa", "pe", "pi", "po", "pu",
    "wa", "we", "wi", "wo", "wu",
)

# Per-biome phoneme-bag weights for KINSHIP_NAME_PHONEMES.
# Weights sum to 1.0 per biome. Indices parallel the phoneme tuple.
# Lush biomes favor labial (m-, p-, b-, w-); arid favors guttural
# (g-, h-, k-); cold (ice / alpine) favors clipped (t-, k-).
KINSHIP_NAME_PHONEME_BIAS: dict[str, tuple[float, ...]] = {
    "ocean": (
        0.032877, 0.032877, 0.032877, 0.032877, 0.032877, 0.006849, 0.006849, 0.006849,
        0.006849, 0.006849, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.013699, 0.020548, 0.020548, 0.020548, 0.020548, 0.020548, 0.013699, 0.013699,
        0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.019178, 0.019178, 0.019178, 0.019178, 0.019178, 0.006849, 0.006849, 0.006849,
        0.006849, 0.006849, 0.006849, 0.006849, 0.006849, 0.006849, 0.006849, 0.019178,
        0.019178, 0.019178, 0.019178, 0.019178, 0.019178, 0.019178, 0.019178, 0.019178,
        0.019178,
    ),
    "ice": (
        0.027027, 0.027027, 0.027027, 0.027027, 0.027027, 0.018919, 0.018919, 0.018919,
        0.018919, 0.018919, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.018919,
        0.018919, 0.018919, 0.018919, 0.018919, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514, 0.013514,
        0.013514,
    ),
    "alpine": (
        0.027778, 0.027778, 0.027778, 0.027778, 0.027778, 0.016667, 0.016667, 0.016667,
        0.016667, 0.016667, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.016667,
        0.016667, 0.016667, 0.016667, 0.016667, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889, 0.013889,
        0.013889,
    ),
    "desert": (
        0.021918, 0.021918, 0.021918, 0.021918, 0.021918, 0.020548, 0.020548, 0.020548,
        0.020548, 0.020548, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.013699, 0.010959, 0.010959, 0.010959, 0.010959, 0.010959, 0.013699, 0.013699,
        0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699, 0.013699,
        0.006849, 0.006849, 0.006849, 0.006849, 0.006849, 0.027397, 0.027397, 0.027397,
        0.027397, 0.027397, 0.027397, 0.027397, 0.027397, 0.027397, 0.027397, 0.009589,
        0.009589, 0.009589, 0.009589, 0.009589, 0.006849, 0.006849, 0.006849, 0.006849,
        0.006849,
    ),
    "tropical-forest": (
        0.036364, 0.036364, 0.036364, 0.036364, 0.036364, 0.006494, 0.006494, 0.006494,
        0.006494, 0.006494, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987,
        0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987,
        0.012987, 0.020779, 0.020779, 0.020779, 0.020779, 0.020779, 0.012987, 0.012987,
        0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987, 0.012987,
        0.019481, 0.019481, 0.019481, 0.019481, 0.019481, 0.006494, 0.006494, 0.006494,
        0.006494, 0.006494, 0.006494, 0.006494, 0.006494, 0.006494, 0.006494, 0.019481,
        0.019481, 0.019481, 0.019481, 0.019481, 0.019481, 0.019481, 0.019481, 0.019481,
        0.019481,
    ),
    "temperate-forest": (
        0.031579, 0.031579, 0.031579, 0.031579, 0.031579, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.013158, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.015789,
        0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789,
        0.015789,
    ),
    "grassland": (
        0.026316, 0.026316, 0.026316, 0.026316, 0.026316, 0.015789, 0.015789, 0.015789,
        0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789,
        0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789, 0.015789,
        0.015789, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.015789, 0.015789,
        0.015789, 0.015789, 0.015789, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158, 0.013158,
        0.013158, 0.013158, 0.013158, 0.013158, 0.015789, 0.015789, 0.015789, 0.015789,
        0.015789,
    ),
}

# Acceptance threshold: at SMALL scale across 20 seeds, no single
# kinship system should dominate the per-layer distribution beyond
# this fraction. Used by test_kinship_for_acceptance in test_kinship.py.
KINSHIP_MAX_DOMINANT_SYSTEM_FRACTION = 0.60

# Phase 3b.4 — Languages. Algorithm-version-named suffix per chain
# convention (3a.5 / 3b.1 / 3b.2 / 3b.3).
LANGUAGE_ALGORITHM_VERSION = "language-typology-v1"
# Lexicon-size thresholds: research note's 3,000+ word target applies
# only to the root (fully developed) language; derived languages get a
# smaller lexicon (200-500 words) per Ernie's plan-ack on Q1.
LANGUAGE_LEXICON_MIN_WORDS = 3000
LANGUAGE_LEXICON_DERIVED_MIN_WORDS = 200
LANGUAGE_LEXICON_DERIVED_MAX_WORDS = 500
# Phoneme inventory. Tight v1 set: 18 consonants + 6 vowels + 1 tone
# flag (per plan-ack Q5). WALS-scale expansion (~30+ consonants) is
# 3b.4.x.
LANGUAGE_PHONEMES: tuple[str, ...] = (
    # 18 consonants
    "p", "t", "k", "m", "n", "ŋ",
    "s", "ʃ", "h", "b", "d", "g",
    "f", "v", "z", "l", "r", "j",
    # 6 vowels (incl. schwa)
    "a", "e", "i", "o", "u", "ə",
)
# Syllable templates. CV is the universal default; the others add
# variation. ~7 templates × ~24 phonemes ~= combinatorial space
# for 3,000+ distinct words without random-IPA anti-pattern.
LANGUAGE_SYLLABLE_TEMPLATES: tuple[str, ...] = (
    "CV", "CVC", "CCV", "CCVC", "V", "VC", "CVV",
)
# Per-biome phonological-feature biases (per plan-ack Q4). Biome
# flavor emerges via (tonal_prob, harmonic_prob, click_prob).
# Per typological reality, every biome has small non-zero values for
# all three features; conditioning biases the distribution without
# making any biome monomorphic.
LANGUAGE_BIOME_PHONOLOGY_BIAS: dict[str, tuple[float, float, float]] = {
    "ocean":           (0.15, 0.20, 0.05),
    "ice":             (0.10, 0.10, 0.40),  # clicks up (Khoisan-style)
    "alpine":          (0.10, 0.20, 0.20),
    "desert":          (0.10, 0.55, 0.05),  # harmonic up
    "tropical-forest": (0.55, 0.15, 0.05),  # tonal up
    "temperate-forest":(0.15, 0.20, 0.05),  # default-ish
    "grassland":       (0.15, 0.20, 0.05),  # default-ish
}
# Per-word-order typology (WALS-anchored frequencies per Greenberg's
# universals). SOV is most common world-wide; SVO second. Probabilities
# sum to 1.0.
LANGUAGE_TYPOGRAPHY: dict[str, tuple[float, float, float, float, float, float]] = {
    "sov": (0.45, 0.35, 0.05, 0.05, 0.05, 0.05),
    "svo": (0.25, 0.45, 0.10, 0.10, 0.05, 0.05),
    "vso": (0.10, 0.10, 0.45, 0.10, 0.10, 0.15),
    "vos": (0.10, 0.05, 0.15, 0.30, 0.20, 0.20),
    "ovs": (0.05, 0.03, 0.13, 0.20, 0.30, 0.29),
    "osv": (0.05, 0.02, 0.12, 0.25, 0.30, 0.26),
}
# Per-word-order Bernoulli features. Cases correlated with SOV
# (typological; case-marking dominant in agglutinative languages).
# Gender more common in SVO European languages (less universal).
# Tense/aspect reasonably distributed across all orders.
LANGUAGE_WORD_ORDER_FEATURES: dict[str, dict[str, float]] = {
    "sov": {"has_cases": 0.85, "has_gender": 0.30, "has_tense_aspect": 0.85},
    "svo": {"has_cases": 0.20, "has_gender": 0.55, "has_tense_aspect": 0.90},
    "vso": {"has_cases": 0.40, "has_gender": 0.40, "has_tense_aspect": 0.80},
    "vos": {"has_cases": 0.30, "has_gender": 0.30, "has_tense_aspect": 0.85},
    "ovs": {"has_cases": 0.35, "has_gender": 0.30, "has_tense_aspect": 0.80},
    "osv": {"has_cases": 0.25, "has_gender": 0.30, "has_tense_aspect": 0.85},
}
# Semantic root table frequencies (WALS-style; kinship + nature +
# action dominate). Used by the lexicon generator to pick roots per
# category. Probabilities sum to 1.0 across the 7 categories.
LANGUAGE_SEMANTIC_CATEGORY_BIAS: dict[str, float] = {
    "kinship":    0.15,
    "nature":     0.25,
    "action":     0.20,
    "abstract":   0.10,
    "pronoun":    0.10,
    "numeral":    0.05,
    "adposition": 0.15,
}
# Cognitive cognate-retention bounds (per plan-ack Q3 and the
# research note's "60-80% cognate rate" target after one divergence
# event). Lower bound = aggressive replacement; upper bound = conservative.
LANGUAGE_DIVERGENCE_COGNATE_LOW = 0.60
LANGUAGE_DIVERGENCE_COGNATE_HIGH = 0.80
# v1 acceptance thresholds (per plan-ack Q1/Q3):
# - Root language: LANGUAGE_LEXICON_MIN_WORDS words.
# - Family graph: every non-root language has a LanguageFamily edge.
# - Phonotactic FSA (built in language.py) validates >= 90% of root
#   words.
LANGUAGE_PHONOTACTIC_VALIDITY_RATIO = 0.90
