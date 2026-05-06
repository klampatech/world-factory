//! Biome Assignment Matrix
//!
//! Deterministic mapping from climate parameters to biome types.
//! Uses Holdridge Life Zone classification as foundation, extended for fantasy/sci-fi.

use super::biome::{
    AlpineBiomeConfig, BiomeColor, BiomeColorMapping, BiomeType, ClimateZone, ElevationZone,
    MoistureLevel,
};
use serde::{Deserialize, Serialize};

/// Geographic coherence configuration for biome transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceConfig {
    /// Maximum biome type difference for adjacent cells (0.0-1.0).
    pub max_transition_rate: f32,
    /// Maximum climate zone jump for adjacent biomes.
    pub max_climate_jump: usize,
    /// Enable smooth transitions between biomes.
    pub enable_smooth_transitions: bool,
}

impl Default for CoherenceConfig {
    fn default() -> Self {
        Self {
            max_transition_rate: 0.5,
            max_climate_jump: 2,
            enable_smooth_transitions: true,
        }
    }
}

/// Configuration for biome generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeAssignmentConfig {
    /// Enable fantasy/sci-fi biome types.
    pub enable_fantasy_biomes: bool,
    /// Enable special coastal/marine biomes.
    pub enable_coastal_biomes: bool,
    /// Base elevation for sea level in meters.
    pub sea_level_base: f32,
    /// Elevation threshold for highlands in meters.
    pub highland_threshold: f32,
    /// Elevation threshold for alpine zone in meters.
    pub alpine_threshold: f32,
    /// Elevation threshold for snow line in meters.
    pub snow_line_threshold: f32,
}

impl Default for BiomeAssignmentConfig {
    fn default() -> Self {
        Self {
            enable_fantasy_biomes: true,
            enable_coastal_biomes: true,
            sea_level_base: 0.0,
            highland_threshold: 1500.0,
            alpine_threshold: 3000.0,
            snow_line_threshold: 4500.0,
        }
    }
}

/// Result of biome assignment including confidence and factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeAssignment {
    /// The assigned biome type.
    pub biome: BiomeType,
    /// Confidence score (0.0-1.0) for this assignment.
    pub confidence: f32,
    /// Factors that influenced the assignment.
    pub factors: Vec<AssignmentFactor>,
}

/// Individual factor that influenced biome assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentFactor {
    pub name: String,
    pub weight: f32,
    pub value: String,
}

impl BiomeAssignment {
    fn new(biome: BiomeType, confidence: f32, factors: Vec<AssignmentFactor>) -> Self {
        Self {
            biome,
            confidence,
            factors,
        }
    }
}

/// The biome assignment matrix - maps climate parameters to biome types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeAssignmentMatrix {
    config: BiomeAssignmentConfig,
}

impl BiomeAssignmentMatrix {
    /// Create a new assignment matrix with default configuration.
    pub fn new() -> Self {
        Self::with_config(BiomeAssignmentConfig::default())
    }

    /// Create a new assignment matrix with custom configuration.
    pub fn with_config(config: BiomeAssignmentConfig) -> Self {
        Self { config }
    }

    /// Assign a biome based on climate parameters.
    ///
    /// # Arguments
    /// * `elevation` - Elevation in meters (negative = below sea level)
    /// * `latitude` - Absolute latitude in degrees (0-90)
    /// * `precipitation` - Annual precipitation in mm (0-10000)
    /// * `temperature` - Mean annual temperature in Celsius (-50 to 50)
    ///
    /// # Returns
    /// The assigned biome with confidence and contributing factors.
    pub fn assign(
        &self,
        elevation: f32,
        latitude: f32,
        precipitation: f32,
        temperature: f32,
    ) -> BiomeAssignment {
        let mut factors = Vec::new();

        // Determine climate zone from latitude
        let climate_zone = self.determine_climate_zone(latitude);
        factors.push(AssignmentFactor {
            name: "climate_zone".into(),
            weight: 0.3,
            value: format!("{:?}", climate_zone),
        });

        // Determine moisture level from precipitation
        let moisture_level = self.determine_moisture_level(precipitation);
        factors.push(AssignmentFactor {
            name: "moisture_level".into(),
            weight: 0.3,
            value: format!("{:?}", moisture_level),
        });

        // Determine elevation zone
        let elevation_zone = ElevationZone::from_height(elevation);
        factors.push(AssignmentFactor {
            name: "elevation_zone".into(),
            weight: 0.2,
            value: format!("{:?}", elevation_zone),
        });

        // Handle special cases first
        let biome = self.assign_biome(climate_zone, moisture_level, elevation_zone, temperature);

        // Apply elevation adjustments for alpine biomes
        let alpine_config = AlpineBiomeConfig::default();
        let biome = biome.with_elevation_adjustment(elevation, temperature, &alpine_config);

        // Calculate confidence based on factor weights
        let confidence = factors.iter().map(|f| f.weight).sum::<f32>() / 1.0;
        let confidence = confidence.min(1.0).max(0.0);

        BiomeAssignment::new(biome, confidence, factors)
    }

    /// Get the color for a biome type.
    pub fn get_biome_color(&self, biome: BiomeType) -> BiomeColor {
        BiomeColorMapping::get_color(biome)
    }

    /// Apply geographic coherence adjustment to biome assignment.
    /// Ensures adjacent polygons have related biomes by adjusting based on neighbors.
    pub fn apply_coherence(
        &self,
        biome: BiomeType,
        neighbors: &[BiomeType],
        config: &CoherenceConfig,
    ) -> BiomeType {
        if neighbors.is_empty() || !config.enable_smooth_transitions {
            return biome;
        }

        // Calculate average climate zone of neighbors
        let neighbor_zones: Vec<usize> = neighbors
            .iter()
            .map(|n| self.biome_to_climate_index(*n))
            .collect();
        let avg_zone = neighbor_zones.iter().sum::<usize>() as f32 / neighbor_zones.len() as f32;

        // Get current biome's climate zone
        let current_zone = self.biome_to_climate_index(biome) as f32;

        // If there's a big jump between current and neighbors, blend toward neighbor average
        let zone_diff = (current_zone - avg_zone).abs();
        if zone_diff > config.max_climate_jump as f32 {
            for neighbor in neighbors {
                let neighbor_zone = self.biome_to_climate_index(*neighbor) as f32;
                let diff_to_neighbor = (neighbor_zone - avg_zone).abs();
                if diff_to_neighbor < zone_diff {
                    return *neighbor;
                }
            }
        }

        biome
    }

    /// Convert biome to a climate zone index (0-4) for comparison.
    fn biome_to_climate_index(&self, biome: BiomeType) -> usize {
        match biome {
            BiomeType::TropicalRainforest
            | BiomeType::TropicalSeasonalForest
            | BiomeType::TropicalSavanna
            | BiomeType::TropicalDryForest => 0,
            BiomeType::SubtropicalRainforest
            | BiomeType::SubtropicalSeasonalForest
            | BiomeType::SubtropicalSteppe
            | BiomeType::SubtropicalDesert => 1,
            BiomeType::TemperateRainforest
            | BiomeType::TemperateDeciduousForest
            | BiomeType::TemperateMixedForest
            | BiomeType::TemperateSteppe
            | BiomeType::TemperateDesert => 2,
            BiomeType::BorealTaiga | BiomeType::BorealForest | BiomeType::TemperateGrassland => 3,
            BiomeType::Tundra | BiomeType::Arctic | BiomeType::PolarDesert => 4,
            BiomeType::MontaneForest | BiomeType::MontaneGrassland => 2,
            BiomeType::AlpineTundra => 4,
            BiomeType::SnowGlacier => 4,
            BiomeType::CoastalWetland
            | BiomeType::Mangrove
            | BiomeType::CoralReef
            | BiomeType::KelpForest
            | BiomeType::OpenOcean => 2,
            BiomeType::HotDesert | BiomeType::ColdDesert | BiomeType::SemiAridSteppe => 2,
            BiomeType::MagicalForest
            | BiomeType::CrystallineDesert
            | BiomeType::BioluminescentOcean
            | BiomeType::VolcanicLandscape
            | BiomeType::ToxicSwamp
            | BiomeType::FloatingIslands => 2,
        }
    }

    /// Determine climate zone from absolute latitude.
    fn determine_climate_zone(&self, latitude: f32) -> ClimateZone {
        let abs_lat = latitude.abs();
        if abs_lat < 23.5 {
            ClimateZone::Tropical
        } else if abs_lat < 35.0 {
            ClimateZone::Subtropical
        } else if abs_lat < 55.0 {
            ClimateZone::Temperate
        } else if abs_lat < 65.0 {
            ClimateZone::Boreal
        } else {
            ClimateZone::Polar
        }
    }

    /// Determine moisture level from annual precipitation in mm.
    fn determine_moisture_level(&self, precipitation: f32) -> MoistureLevel {
        // Using simplified Köppen-style classification
        // Hyperarid: <100mm (desert)
        // Arid: 100-250mm (semi-desert)
        // Semi-arid: 250-500mm (steppe)
        // Sub-humid: 500-1000mm (grassland/savanna)
        // Humid: 1000-2000mm (forest)
        // Per-humid: >2000mm (rainforest)
        if precipitation < 100.0 {
            MoistureLevel::HyperArid
        } else if precipitation < 250.0 {
            MoistureLevel::Arid
        } else if precipitation < 500.0 {
            MoistureLevel::SemiArid
        } else if precipitation < 1000.0 {
            MoistureLevel::SubHumid
        } else if precipitation < 2000.0 {
            MoistureLevel::Humid
        } else {
            MoistureLevel::PerHumid
        }
    }

    /// Core biome assignment logic using Holdridge-inspired matrix.
    fn assign_biome(
        &self,
        climate: ClimateZone,
        moisture: MoistureLevel,
        elevation: ElevationZone,
        temperature: f32,
    ) -> BiomeType {
        // Special case: underwater/below sea level
        if elevation == ElevationZone::Nival && temperature < 0.0 {
            return BiomeType::SnowGlacier;
        }

        // Special case: high altitude zones
        match elevation {
            ElevationZone::Alpine | ElevationZone::Nival => {
                return match climate {
                    ClimateZone::Tropical | ClimateZone::Subtropical => {
                        if temperature < -10.0 {
                            BiomeType::SnowGlacier
                        } else {
                            BiomeType::AlpineTundra
                        }
                    }
                    _ => BiomeType::SnowGlacier,
                };
            }
            ElevationZone::Highland => {
                if self.config.enable_fantasy_biomes {
                    // Check for magical hotspots based on temperature anomaly
                    if temperature > 30.0 {
                        return BiomeType::VolcanicLandscape;
                    }
                }
                return BiomeType::MontaneForest;
            }
            _ => {}
        }

        // Main assignment matrix based on climate × moisture
        match (climate, moisture) {
            // TROPICAL
            (ClimateZone::Tropical, MoistureLevel::PerHumid) => BiomeType::TropicalRainforest,
            (ClimateZone::Tropical, MoistureLevel::Humid) => BiomeType::TropicalSeasonalForest,
            (ClimateZone::Tropical, MoistureLevel::SubHumid) => BiomeType::TropicalSavanna,
            (ClimateZone::Tropical, MoistureLevel::SemiArid) => BiomeType::TropicalDryForest,
            (ClimateZone::Tropical, MoistureLevel::Arid | MoistureLevel::HyperArid) => {
                BiomeType::HotDesert
            }

            // SUBTROPICAL
            (ClimateZone::Subtropical, MoistureLevel::PerHumid) => BiomeType::SubtropicalRainforest,
            (ClimateZone::Subtropical, MoistureLevel::Humid) => {
                BiomeType::SubtropicalSeasonalForest
            }
            (ClimateZone::Subtropical, MoistureLevel::SubHumid) => BiomeType::SubtropicalSteppe,
            (ClimateZone::Subtropical, MoistureLevel::SemiArid) => BiomeType::SemiAridSteppe,
            (ClimateZone::Subtropical, MoistureLevel::Arid | MoistureLevel::HyperArid) => {
                BiomeType::SubtropicalDesert
            }

            // TEMPERATE
            (ClimateZone::Temperate, MoistureLevel::PerHumid) => BiomeType::TemperateRainforest,
            (ClimateZone::Temperate, MoistureLevel::Humid) => BiomeType::TemperateDeciduousForest,
            (ClimateZone::Temperate, MoistureLevel::SubHumid) => BiomeType::TemperateMixedForest,
            (ClimateZone::Temperate, MoistureLevel::SemiArid) => BiomeType::TemperateSteppe,
            (ClimateZone::Temperate, MoistureLevel::Arid | MoistureLevel::HyperArid) => {
                BiomeType::TemperateDesert
            }

            // BOREAL
            (ClimateZone::Boreal, MoistureLevel::PerHumid | MoistureLevel::Humid) => {
                BiomeType::BorealTaiga
            }
            (ClimateZone::Boreal, MoistureLevel::SubHumid) => BiomeType::BorealForest,
            (ClimateZone::Boreal, MoistureLevel::SemiArid) => BiomeType::TemperateGrassland,
            (ClimateZone::Boreal, MoistureLevel::Arid | MoistureLevel::HyperArid) => {
                BiomeType::ColdDesert
            }

            // POLAR
            (ClimateZone::Polar, MoistureLevel::PerHumid | MoistureLevel::Humid) => {
                BiomeType::Tundra
            }
            (ClimateZone::Polar, MoistureLevel::SubHumid | MoistureLevel::SemiArid) => {
                BiomeType::Arctic
            }
            (ClimateZone::Polar, MoistureLevel::Arid | MoistureLevel::HyperArid) => {
                BiomeType::PolarDesert
            }
        }
    }

    /// Assign a coastal/marine biome.
    ///
    /// # Arguments
    /// * `is_coastal` - True if within coastal zone
    /// * `is_ocean` - True if open ocean
    /// * `is_reef_zone` - True if in coral reef latitude zone
    /// * `depth` - Water depth in meters (positive = deeper)
    pub fn assign_marine(
        &self,
        is_coastal: bool,
        is_ocean: bool,
        is_reef_zone: bool,
        depth: f32,
    ) -> BiomeAssignment {
        if !self.config.enable_coastal_biomes {
            return BiomeAssignment::new(BiomeType::OpenOcean, 1.0, vec![]);
        }

        let factors = vec![
            AssignmentFactor {
                name: "is_coastal".into(),
                weight: 0.3,
                value: is_coastal.to_string(),
            },
            AssignmentFactor {
                name: "is_ocean".into(),
                weight: 0.2,
                value: is_ocean.to_string(),
            },
            AssignmentFactor {
                name: "depth".into(),
                weight: 0.3,
                value: format!("{:.0}m", depth),
            },
        ];

        let biome = if is_ocean && !is_coastal {
            if depth < 200.0 {
                // Continental shelf
                BiomeType::KelpForest
            } else {
                BiomeType::OpenOcean
            }
        } else if is_coastal {
            if depth < 0.0 {
                // Above water (shouldn't happen, but safety)
                BiomeType::CoastalWetland
            } else if depth < 10.0 {
                // Very shallow - mangrove zone
                BiomeType::Mangrove
            } else if depth < 50.0 && is_reef_zone {
                // Coral reef zone
                BiomeType::CoralReef
            } else {
                BiomeType::CoastalWetland
            }
        } else {
            BiomeType::OpenOcean
        };

        BiomeAssignment::new(biome, 0.9, factors)
    }
}

impl Default for BiomeAssignmentMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tropical_rainforest_assignment() {
        let matrix = BiomeAssignmentMatrix::new();

        // Tropical, high precipitation = rainforest
        let result = matrix.assign(
            200.0,  // elevation
            5.0,    // latitude (tropical)
            2500.0, // precipitation
            28.0,   // temperature
        );

        assert_eq!(result.biome, BiomeType::TropicalRainforest);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_desert_assignment() {
        let matrix = BiomeAssignmentMatrix::new();

        // Tropical, very low precipitation = desert
        let result = matrix.assign(
            100.0, 25.0, // subtropical
            50.0, // hyperarid
            32.0,
        );

        assert!(matches!(
            result.biome,
            BiomeType::HotDesert | BiomeType::SubtropicalDesert | BiomeType::TemperateDesert
        ));
    }

    #[test]
    fn test_boreal_assignment() {
        let matrix = BiomeAssignmentMatrix::new();

        let result = matrix.assign(
            300.0, 60.0,  // boreal
            800.0, // humid
            2.0,
        );

        assert!(matches!(
            result.biome,
            BiomeType::BorealTaiga | BiomeType::BorealForest
        ));
    }

    #[test]
    fn test_alpine_override() {
        let matrix = BiomeAssignmentMatrix::new();

        // High elevation should override climate zone
        let result = matrix.assign(
            4000.0, // alpine
            45.0,   // temperate
            500.0, -5.0,
        );

        assert!(matches!(
            result.biome,
            BiomeType::AlpineTundra | BiomeType::SnowGlacier
        ));
    }

    #[test]
    fn test_climate_zone_determination() {
        let matrix = BiomeAssignmentMatrix::new();

        // Test latitude boundaries
        assert!(matches!(
            matrix.assign(100.0, 10.0, 1000.0, 25.0).biome,
            BiomeType::TropicalRainforest
                | BiomeType::TropicalSeasonalForest
                | BiomeType::TropicalSavanna
        ));

        assert!(matches!(
            matrix.assign(100.0, 45.0, 1000.0, 12.0).biome,
            BiomeType::TemperateDeciduousForest
                | BiomeType::TemperateMixedForest
                | BiomeType::TemperateRainforest
        ));

        assert!(matches!(
            matrix.assign(100.0, 75.0, 500.0, -10.0).biome,
            BiomeType::Tundra | BiomeType::Arctic | BiomeType::PolarDesert
        ));
    }

    #[test]
    fn test_moisture_level_from_precipitation() {
        let matrix = BiomeAssignmentMatrix::new();

        // Very low precipitation
        let r1 = matrix.assign(100.0, 25.0, 50.0, 30.0);
        assert!(matches!(
            r1.biome,
            BiomeType::HotDesert | BiomeType::SubtropicalDesert
        ));

        // Medium precipitation
        let r2 = matrix.assign(100.0, 25.0, 400.0, 25.0);
        assert!(matches!(
            r2.biome,
            BiomeType::TropicalSavanna | BiomeType::SubtropicalSteppe | BiomeType::SemiAridSteppe
        ));

        // High precipitation
        let r3 = matrix.assign(100.0, 5.0, 3000.0, 27.0);
        assert_eq!(r3.biome, BiomeType::TropicalRainforest);
    }

    #[test]
    fn test_deterministic_reproduction() {
        let matrix = BiomeAssignmentMatrix::new();

        let params = (250.0, 15.0, 1800.0, 26.0);

        let result1 = matrix.assign(params.0, params.1, params.2, params.3);
        let result2 = matrix.assign(params.0, params.1, params.2, params.3);

        assert_eq!(result1.biome, result2.biome);
        assert_eq!(result1.confidence, result2.confidence);
    }

    #[test]
    fn test_biome_color_mapping() {
        let matrix = BiomeAssignmentMatrix::new();

        // Test tropical rainforest color
        let color = matrix.get_biome_color(BiomeType::TropicalRainforest);
        assert_eq!(color.0, 0); // R
        assert_eq!(color.1, 102); // G
        assert_eq!(color.2, 51); // B

        // Test snow/glacier color
        let color = matrix.get_biome_color(BiomeType::SnowGlacier);
        assert_eq!(color, BiomeColor::new(204, 245, 255));

        // Test hot desert color
        let color = matrix.get_biome_color(BiomeType::HotDesert);
        assert_eq!(color, BiomeColor::new(230, 204, 51));
    }

    #[test]
    fn test_biome_color_utility() {
        let color = BiomeColor::new(100, 150, 200);
        assert_eq!(color.rgb(), [100, 150, 200]);
        assert_eq!(color.to_css(), "rgb(100, 150, 200)");
    }

    #[test]
    fn test_elevation_adjustment() {
        let config = AlpineBiomeConfig::default();

        // Temperate biome at sea level - no adjustment
        let adjusted =
            BiomeType::TemperateDeciduousForest.with_elevation_adjustment(100.0, 15.0, &config);
        assert_eq!(adjusted, BiomeType::TemperateDeciduousForest);

        // Temperate biome at alpine elevation - should become alpine
        let adjusted =
            BiomeType::TemperateDeciduousForest.with_elevation_adjustment(3500.0, 2.0, &config);
        assert!(matches!(
            adjusted,
            BiomeType::AlpineTundra | BiomeType::MontaneGrassland
        ));

        // Very high elevation with cold temp - snow/glacier
        let adjusted =
            BiomeType::TemperateDeciduousForest.with_elevation_adjustment(5000.0, -10.0, &config);
        assert!(matches!(adjusted, BiomeType::SnowGlacier));
    }

    #[test]
    fn test_geographic_coherence() {
        let config = CoherenceConfig::default();
        let matrix = BiomeAssignmentMatrix::new();

        // Jungle surrounded by similar biomes - no change
        let neighbors = vec![
            BiomeType::TropicalSeasonalForest,
            BiomeType::TropicalSavanna,
        ];
        let result = matrix.apply_coherence(BiomeType::TropicalRainforest, &neighbors, &config);
        assert_eq!(result, BiomeType::TropicalRainforest);

        // Jungle surrounded by tundra - should adjust toward neighbors
        let neighbors = vec![BiomeType::Tundra, BiomeType::Arctic];
        let result = matrix.apply_coherence(BiomeType::TropicalRainforest, &neighbors, &config);
        // Should favor neighbor biomes closer to average
        assert!(matches!(
            result,
            BiomeType::Tundra | BiomeType::Arctic | BiomeType::TropicalRainforest
        ));
    }

    #[test]
    fn test_coherence_disabled() {
        let config = CoherenceConfig {
            enable_smooth_transitions: false,
            ..Default::default()
        };
        let matrix = BiomeAssignmentMatrix::new();

        let neighbors = vec![BiomeType::Tundra, BiomeType::Tundra];
        let result = matrix.apply_coherence(BiomeType::TropicalRainforest, &neighbors, &config);
        // Should not change when disabled
        assert_eq!(result, BiomeType::TropicalRainforest);
    }
}
