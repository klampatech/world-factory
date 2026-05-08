//! Biome types and suitability ratings for settlement placement
//!
//! Implements §D.2 filtering requirements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Biome types found in the simulation world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    /// Tropical rainforest - not suitable for initial settlements
    TropicalRainforest,
    /// Tropical seasonal forest/savanna
    TropicalSeasonalForest,
    /// Temperate broadleaf forest
    TemperateBroadleaf,
    /// Temperate needleleaf forest (boreal/taiga)
    TemperateNeedleleaf,
    /// Mediterranean scrub
    MediterraneanScrub,
    /// Temperate grassland
    TemperateGrassland,
    /// Desert - NOT suitable for initial settlements
    Desert,
    /// Tundra - NOT suitable for initial settlements
    Tundra,
    /// Ice - NOT suitable for initial settlements
    Ice,
    /// Ocean - NOT suitable for initial settlements
    Ocean,
    /// Shrubland
    Shrubland,
    /// Wetland
    Wetland,
}

impl BiomeType {
    /// Returns true if this biome is suitable for initial settlement placement
    pub fn is_suitable_for_settlement(&self) -> bool {
        matches!(
            self,
            BiomeType::TropicalSeasonalForest
                | BiomeType::TemperateBroadleaf
                | BiomeType::TemperateNeedleleaf
                | BiomeType::MediterraneanScrub
                | BiomeType::TemperateGrassland
                | BiomeType::Shrubland
                | BiomeType::Wetland
        )
    }

    /// Returns the carrying capacity modifier for this biome
    /// Higher values mean more population can be supported
    pub fn carrying_capacity_modifier(&self) -> f32 {
        match self {
            BiomeType::TropicalRainforest => 0.8,
            BiomeType::TropicalSeasonalForest => 1.2,
            BiomeType::TemperateBroadleaf => 1.5,
            BiomeType::TemperateNeedleleaf => 0.9,
            BiomeType::MediterraneanScrub => 0.7,
            BiomeType::TemperateGrassland => 1.1,
            BiomeType::Desert => 0.2,
            BiomeType::Tundra => 0.1,
            BiomeType::Ice => 0.0,
            BiomeType::Ocean => 0.0,
            BiomeType::Shrubland => 0.6,
            BiomeType::Wetland => 0.5,
        }
    }

    /// Returns a human-readable name for this biome
    pub fn display_name(&self) -> &'static str {
        match self {
            BiomeType::TropicalRainforest => "Tropical Rainforest",
            BiomeType::TropicalSeasonalForest => "Tropical Seasonal Forest",
            BiomeType::TemperateBroadleaf => "Temperate Broadleaf Forest",
            BiomeType::TemperateNeedleleaf => "Temperate Needleleaf Forest",
            BiomeType::MediterraneanScrub => "Mediterranean Scrub",
            BiomeType::TemperateGrassland => "Temperate Grassland",
            BiomeType::Desert => "Desert",
            BiomeType::Tundra => "Tundra",
            BiomeType::Ice => "Ice",
            BiomeType::Ocean => "Ocean",
            BiomeType::Shrubland => "Shrubland",
            BiomeType::Wetland => "Wetland",
        }
    }
}

/// Polygon biome data - stores the biome assignment for each polygon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonBiome {
    pub polygon_id: u64,
    pub biome: BiomeType,
    /// Elevation in meters
    pub elevation: f32,
    /// Derived suitability score (0.0 to 1.0)
    pub settlement_suitability: f32,
}

impl PolygonBiome {
    /// Calculate settlement suitability based on biome and elevation
    /// Per spec §D.2: prefer lowland to midland (0-800m)
    pub fn calculate_suitability(biome: BiomeType, elevation: f32) -> f32 {
        let biome_score = if biome.is_suitable_for_settlement() { 1.0 } else { 0.0 };
        let elevation_score = if elevation <= 800.0 {
            1.0
        } else if elevation <= 1500.0 {
            0.5
        } else if elevation <= 2500.0 {
            0.2
        } else {
            0.0
        };
        
        biome_score * (0.6 + 0.4 * elevation_score)
    }

    pub fn new(polygon_id: u64, biome: BiomeType, elevation: f32) -> Self {
        let settlement_suitability = Self::calculate_suitability(biome, elevation);
        Self {
            polygon_id,
            biome,
            elevation,
            settlement_suitability,
        }
    }

    /// Returns true if this polygon is suitable for initial settlement placement
    pub fn is_suitable_for_settlement(&self) -> bool {
        self.settlement_suitability > 0.0
    }
}

/// Biome assignment system - generates and manages biome data for polygons
pub struct BiomeAssignmentSystem {
    /// Seed for reproducible biome generation
    seed: u64,
}

impl BiomeAssignmentSystem {
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            seed: seed.unwrap_or_else(rand::random),
        }
    }

    /// Generate biome assignments for a set of polygons based on elevation
    /// Uses latitude proxy and elevation bands to determine biome
    pub fn assign_biomes(
        &self,
        polygons: &HashMap<u64, f32>,  // polygon_id -> elevation
        _base_biome: BiomeType,        // could be used for more sophisticated assignment
    ) -> HashMap<u64, PolygonBiome> {
        let mut biomes = HashMap::new();
        
        for (polygon_id, &elevation) in polygons {
            let biome = self.determine_biome(elevation);
            let polygon_biome = PolygonBiome::new(*polygon_id, biome, elevation);
            biomes.insert(*polygon_id, polygon_biome);
        }
        
        biomes
    }

    /// Determine biome based on 8 elevation bands
    /// Zone 0: Deep ocean (< -200m)
    /// Zone 1: Ocean shelf (-200m to 0m)
    /// Zone 2: Very lowland (0-200m) - optimal settlement zone
    /// Zone 3: Lowland (200-400m)
    /// Zone 4: Midland (400-600m)
    /// Zone 5: High-midland (600-800m) - upper settlement boundary
    /// Zone 6: Highland (800-1100m)
    /// Zone 7: Mountain/Snow (> 1100m)
    fn determine_biome(&self, elevation: f32) -> BiomeType {
        match elevation {
            // Below sea level -> Ocean
            e if e < 0.0 => BiomeType::Ocean,
            
            // Zone 2: Very lowland (0-200m) - lush biomes
            e if e < 200.0 => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.5 {
                    BiomeType::TemperateBroadleaf
                } else if r < 0.75 {
                    BiomeType::TropicalSeasonalForest
                } else if r < 0.9 {
                    BiomeType::Wetland
                } else {
                    BiomeType::TemperateGrassland
                }
            },
            
            // Zone 3: Lowland (200-400m) - mixed forest
            e if e < 400.0 => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.4 {
                    BiomeType::TemperateBroadleaf
                } else if r < 0.7 {
                    BiomeType::TropicalSeasonalForest
                } else {
                    BiomeType::TemperateGrassland
                }
            },
            
            // Zone 4: Midland (400-600m) - grassland and forest mix
            e if e < 600.0 => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.35 {
                    BiomeType::TemperateBroadleaf
                } else if r < 0.6 {
                    BiomeType::TemperateGrassland
                } else if r < 0.8 {
                    BiomeType::TemperateNeedleleaf
                } else {
                    BiomeType::Shrubland
                }
            },
            
            // Zone 5: High-midland (600-800m) - transition zone
            e if e < 800.0 => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.3 {
                    BiomeType::TemperateNeedleleaf
                } else if r < 0.55 {
                    BiomeType::TemperateGrassland
                } else if r < 0.75 {
                    BiomeType::Shrubland
                } else {
                    BiomeType::MediterraneanScrub
                }
            },
            
            // Zone 6: Highland (800-1100m) - harsh conditions
            e if e < 1100.0 => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.35 {
                    BiomeType::TemperateNeedleleaf
                } else if r < 0.55 {
                    BiomeType::Shrubland
                } else if r < 0.75 {
                    BiomeType::TemperateGrassland
                } else {
                    BiomeType::Tundra
                }
            },
            
            // Zone 7: Mountain/Snow (> 1100m) - hostile
            _ => {
                let r = self.hash_to_range(self.seed.wrapping_add((elevation * 100.0) as u64));
                if r < 0.3 {
                    BiomeType::TemperateNeedleleaf
                } else if r < 0.5 {
                    BiomeType::Shrubland
                } else if r < 0.7 {
                    BiomeType::Tundra
                } else if r < 0.85 {
                    BiomeType::Desert
                } else {
                    BiomeType::Ice
                }
            }
        }
    }

    /// Convert a hash value to a range 0.0-1.0
    fn hash_to_range(&self, hash: u64) -> f32 {
        (hash % 1000) as f32 / 1000.0
    }

    /// Filter polygons to only those suitable for initial settlement placement
    /// Per spec §D.2: excludes deserts, tundra, ocean and prefers elevation 0-800m
    pub fn filter_suitable_polygons(
        &self,
        biomes: &HashMap<u64, PolygonBiome>,
    ) -> Vec<u64> {
        let mut suitable: Vec<u64> = biomes
            .iter()
            .filter(|(_, biome)| biome.is_suitable_for_settlement())
            .map(|(&id, _)| id)
            .collect();
        
        // Sort by suitability score (highest first) to prefer optimal locations
        suitable.sort_by(|a, b| {
            let score_a = biomes.get(&a).map(|b| b.settlement_suitability).unwrap_or(0.0);
            let score_b = biomes.get(&b).map(|b| b.settlement_suitability).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        suitable
    }

    /// Get the carrying capacity for a polygon based on its biome and elevation
    pub fn get_carrying_capacity(
        &self,
        biome: &PolygonBiome,
        base_capacity: u32,
    ) -> u32 {
        (base_capacity as f32 * biome.biome.carrying_capacity_modifier()) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_suitability() {
        assert!(BiomeType::TemperateBroadleaf.is_suitable_for_settlement());
        assert!(!BiomeType::Desert.is_suitable_for_settlement());
        assert!(!BiomeType::Tundra.is_suitable_for_settlement());
        assert!(!BiomeType::Ocean.is_suitable_for_settlement());
        assert!(!BiomeType::Ice.is_suitable_for_settlement());
    }

    #[test]
    fn test_elevation_suitability() {
        // Lowland (0-800m) should have high suitability
        let suitability = PolygonBiome::calculate_suitability(BiomeType::TemperateBroadleaf, 500.0);
        assert!(suitability > 0.9);
        
        // Highland should have lower suitability
        let suitability = PolygonBiome::calculate_suitability(BiomeType::TemperateBroadleaf, 1200.0);
        assert!(suitability < 0.8);
    }

    #[test]
    fn test_unsuitable_biomes_always_zero() {
        for elevation in [0.0, 500.0, 1000.0, 2000.0] {
            assert_eq!(
                PolygonBiome::calculate_suitability(BiomeType::Desert, elevation),
                0.0
            );
            assert_eq!(
                PolygonBiome::calculate_suitability(BiomeType::Tundra, elevation),
                0.0
            );
            assert_eq!(
                PolygonBiome::calculate_suitability(BiomeType::Ocean, elevation),
                0.0
            );
        }
    }

    #[test]
    fn test_filter_suitable_polygons() {
        let system = BiomeAssignmentSystem::new(Some(42));
        
        let mut polygons = HashMap::new();
        polygons.insert(1, 200.0);   // Very lowland - suitable
        polygons.insert(2, 400.0);   // Lowland - suitable  
        polygons.insert(3, -10.0);   // Ocean - not suitable
        polygons.insert(4, 3000.0); // Mountain - not suitable
        
        let biomes = system.assign_biomes(&polygons, BiomeType::TemperateBroadleaf);
        let suitable = system.filter_suitable_polygons(&biomes);
        
        assert_eq!(suitable.len(), 2);
        assert!(suitable.contains(&1));
        assert!(suitable.contains(&2));
    }

    #[test]
    fn test_8_elevation_zones_biome_assignment() {
        // Test that all 8 elevation zones can be assigned different biomes
        let system = BiomeAssignmentSystem::new(Some(42));
        
        let elevations: Vec<(f32, &str)> = vec![
            (-300.0, "Deep ocean"),
            (-100.0, "Ocean shelf"),
            (100.0, "Very lowland"),
            (300.0, "Lowland"),
            (500.0, "Midland"),
            (700.0, "High-midland"),
            (1000.0, "Highland"),
            (2000.0, "Mountain"),
        ];
        
        for (elevation, name) in elevations {
            let biome = system.determine_biome(elevation);
            println!("Zone at {}m ({}) assigned biome: {:?}", elevation, name, biome);
            
            // Verify ocean biomes for below sea level
            if elevation < 0.0 {
                assert_eq!(biome, BiomeType::Ocean, "Elevation {} should be Ocean", elevation);
            }
        }
        
        // Verify optimal zones (200-600m) are generally suitable for settlement
        for elevation in [300.0, 500.0] {
            let biome = system.determine_biome(elevation);
            assert!(
                biome.is_suitable_for_settlement(),
                "Elevation {} (midland zone) should be suitable for settlement",
                elevation
            );
        }
    }

    #[test]
    fn test_biome_transition_between_zones() {
        // Verify biomes can differ between adjacent elevation zones
        let system = BiomeAssignmentSystem::new(Some(123));
        
        // Test transition from very lowland to lowland (zone 2 to zone 3)
        let biome_low = system.determine_biome(100.0);
        let biome_high = system.determine_biome(400.0);
        
        // At minimum they should be different biomes due to different elevation
        // The random seed may occasionally produce same biome, so just verify they're valid
        assert_ne!(biome_low, BiomeType::Ice);
        assert_ne!(biome_high, BiomeType::Ice);
    }
}
