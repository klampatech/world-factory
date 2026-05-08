//! Wonders module - Large-scale geological and historical features
//! 
//! Implements wonder filtering based on world size per WOR-481

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wonder size classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonderSize {
    /// Small wonders suitable for all worlds
    Small,
    /// Medium wonders for worlds ≥32x32
    Medium,
    /// Large wonders only for worlds ≥64x64
    Large,
}

impl WonderSize {
    /// Returns the minimum world dimension required for this wonder size
    pub fn min_world_dimension(&self) -> u32 {
        match self {
            WonderSize::Small => 0,
            WonderSize::Medium => 32,
            WonderSize::Large => 64,
        }
    }
    
    /// Check if this wonder can appear on a world of given dimensions
    pub fn can_spawn(&self, world_width: u32, world_height: u32) -> bool {
        let min_dim = self.min_world_dimension();
        world_width >= min_dim && world_height >= min_dim
    }
}

/// Wonder types with size classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WonderType {
    // Small wonders (all worlds)
    NaturalSpring,
    AncientTree,
    SacredGrove,
    SmallCave,
    HistoricBattlefield,
    
    // Medium wonders (≥32x32)
    MountainPeak,
    GreatWaterfall,
    AncientRuins,
    SacredTemple,
    DeepCavern,
    VolcanicVent,
    
    // Large wonders (≥64x64)
    MountainRange,
    GreatCanyon,
    AncientCity,
    SacredMountain,
    SubterraneanLake,
    MagicalStorm,
}

impl WonderType {
    /// Get the size classification for this wonder type
    pub fn size(&self) -> WonderSize {
        match self {
            // Small wonders
            WonderType::NaturalSpring
            | WonderType::AncientTree
            | WonderType::SacredGrove
            | WonderType::SmallCave
            | WonderType::HistoricBattlefield => WonderSize::Small,
            
            // Medium wonders
            WonderType::MountainPeak
            | WonderType::GreatWaterfall
            | WonderType::AncientRuins
            | WonderType::SacredTemple
            | WonderType::DeepCavern
            | WonderType::VolcanicVent => WonderSize::Medium,
            
            // Large wonders
            WonderType::MountainRange
            | WonderType::GreatCanyon
            | WonderType::AncientCity
            | WonderType::SacredMountain
            | WonderType::SubterraneanLake
            | WonderType::MagicalStorm => WonderSize::Large,
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            WonderType::NaturalSpring => "Natural Spring",
            WonderType::AncientTree => "Ancient Tree",
            WonderType::SacredGrove => "Sacred Grove",
            WonderType::SmallCave => "Small Cave",
            WonderType::HistoricBattlefield => "Historic Battlefield",
            WonderType::MountainPeak => "Mountain Peak",
            WonderType::GreatWaterfall => "Great Waterfall",
            WonderType::AncientRuins => "Ancient Ruins",
            WonderType::SacredTemple => "Sacred Temple",
            WonderType::DeepCavern => "Deep Cavern",
            WonderType::VolcanicVent => "Volcanic Vent",
            WonderType::MountainRange => "Mountain Range",
            WonderType::GreatCanyon => "Great Canyon",
            WonderType::AncientCity => "Ancient City",
            WonderType::SacredMountain => "Sacred Mountain",
            WonderType::SubterraneanLake => "Subterranean Lake",
            WonderType::MagicalStorm => "Magical Storm",
        }
    }
    
    /// Get all wonder types suitable for a world of given dimensions
    pub fn for_world_dimensions(world_width: u32, world_height: u32) -> Vec<WonderType> {
        Self::all()
            .into_iter()
            .filter(|w| w.size().can_spawn(world_width, world_height))
            .collect()
    }
    
    /// Get all wonder types
    pub fn all() -> Vec<WonderType> {
        vec![
            WonderType::NaturalSpring,
            WonderType::AncientTree,
            WonderType::SacredGrove,
            WonderType::SmallCave,
            WonderType::HistoricBattlefield,
            WonderType::MountainPeak,
            WonderType::GreatWaterfall,
            WonderType::AncientRuins,
            WonderType::SacredTemple,
            WonderType::DeepCavern,
            WonderType::VolcanicVent,
            WonderType::MountainRange,
            WonderType::GreatCanyon,
            WonderType::AncientCity,
            WonderType::SacredMountain,
            WonderType::SubterraneanLake,
            WonderType::MagicalStorm,
        ]
    }
}

/// Wonder - a large-scale feature in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wonder {
    pub id: String,
    pub name: String,
    pub wonder_type: WonderType,
    pub polygon_id: u64,
    pub discovery_year: Option<u32>,
    pub significance: f32,
}

impl Wonder {
    pub fn new(id: String, wonder_type: WonderType, polygon_id: u64) -> Self {
        Self {
            id,
            name: wonder_type.display_name().to_string(),
            wonder_type,
            polygon_id,
            discovery_year: None,
            significance: 0.7,
        }
    }
}

/// Wonder filtering system
pub struct WonderFilter {
    /// Minimum dimension for medium wonders
    medium_wonder_threshold: u32,
    /// Minimum dimension for large wonders  
    large_wonder_threshold: u32,
}

impl WonderFilter {
    pub fn new() -> Self {
        Self {
            medium_wonder_threshold: 32,
            large_wonder_threshold: 64,
        }
    }
    
    /// Create a filter configured for a specific world size
    pub fn for_world(world_width: u32, world_height: u32) -> Self {
        Self {
            medium_wonder_threshold: if world_width < 32 || world_height < 32 { u32::MAX } else { 32 },
            large_wonder_threshold: if world_width < 64 || world_height < 64 { u32::MAX } else { 64 },
        }
    }
    
    /// Check if a wonder type can spawn on a world of given dimensions
    pub fn can_spawn(&self, wonder_type: WonderType, world_width: u32, world_height: u32) -> bool {
        let min_dim = match wonder_type.size() {
            WonderSize::Small => 0,
            WonderSize::Medium => self.medium_wonder_threshold,
            WonderSize::Large => self.large_wonder_threshold,
        };
        world_width >= min_dim && world_height >= min_dim
    }
    
    /// Filter wonder types to only those suitable for the given world size
    pub fn filter_wonder_types(
        &self,
        wonder_types: &[WonderType],
        world_width: u32,
        world_height: u32,
    ) -> Vec<WonderType> {
        wonder_types
            .iter()
            .filter(|&&w| self.can_spawn(w, world_width, world_height))
            .cloned()
            .collect()
    }
}

impl Default for WonderFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wonder_size_classification() {
        assert_eq!(WonderType::NaturalSpring.size(), WonderSize::Small);
        assert_eq!(WonderType::MountainPeak.size(), WonderSize::Medium);
        assert_eq!(WonderType::MountainRange.size(), WonderSize::Large);
    }
    
    #[test]
    fn test_small_world_filters_large_wonders() {
        // 32x32 world should filter out large wonders
        let filter = WonderFilter::for_world(32, 32);
        
        assert!(filter.can_spawn(WonderType::NaturalSpring, 32, 32));
        assert!(filter.can_spawn(WonderType::MountainPeak, 32, 32));
        assert!(!filter.can_spawn(WonderType::MountainRange, 32, 32));
    }
    
    #[test]
    fn test_small_world_filters_medium_wonders() {
        // 16x16 world should filter out medium and large wonders
        let filter = WonderFilter::for_world(16, 16);
        
        assert!(filter.can_spawn(WonderType::NaturalSpring, 16, 16));
        assert!(!filter.can_spawn(WonderType::MountainPeak, 16, 16));
        assert!(!filter.can_spawn(WonderType::MountainRange, 16, 16));
    }
    
    #[test]
    fn test_large_world_allows_all_wonders() {
        // 64x64 world should allow all wonder sizes
        let filter = WonderFilter::for_world(64, 64);
        
        assert!(filter.can_spawn(WonderType::NaturalSpring, 64, 64));
        assert!(filter.can_spawn(WonderType::MountainPeak, 64, 64));
        assert!(filter.can_spawn(WonderType::MountainRange, 64, 64));
    }
    
    #[test]
    fn test_wonder_types_for_world() {
        // 32x32 should include small and medium, exclude large
        let types = WonderType::for_world_dimensions(32, 32);
        assert!(types.contains(&WonderType::NaturalSpring));
        assert!(types.contains(&WonderType::MountainPeak));
        assert!(!types.contains(&WonderType::MountainRange));
    }
    
    #[test]
    fn test_filter_wonder_types() {
        let filter = WonderFilter::for_world(32, 32);
        let all_types = WonderType::all();
        let filtered = filter.filter_wonder_types(&all_types, 32, 32);
        
        assert_eq!(filtered.len(), 11); // 5 small + 6 medium = 11
        assert!(filtered.iter().all(|w| w.size() != WonderSize::Large));
    }
}
