//! Biome definitions and assignment matrix for World Factory.
//!
//! Biomes are assigned based on elevation, latitude (climate), and precipitation.
//! The assignment matrix implements deterministic generation.

use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export ResourceCategory for external use
pub use crate::terrain::resource_types::ResourceCategory;

// ============================================================================
// Biome Color Types
// ============================================================================

/// RGB color for biome rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BiomeColor(pub u8, pub u8, pub u8);

impl BiomeColor {
    /// Create a new biome color.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }
    
    /// Get RGB as array.
    pub fn rgb(&self) -> [u8; 3] {
        [self.0, self.1, self.2]
    }
    
    /// Get as CSS rgb() string.
    pub fn to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.0, self.1, self.2)
    }
}

impl fmt::Display for BiomeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.0, self.1, self.2)
    }
}

/// Biome color mapping - provides RGB colors for each biome type.
/// Used for map rendering and visualization.
pub struct BiomeColorMapping;

impl BiomeColorMapping {
    /// Get the RGB color for a biome type.
    /// Colors follow GOAL.md Phase 1.5 specification.
    pub fn get_color(biome: BiomeType) -> BiomeColor {
        match biome {
            // GOAL.md colors (exact hex values converted to RGB)
            BiomeType::TropicalRainforest => BiomeColor::new(0, 102, 51),   // #006633
            BiomeType::TropicalSeasonalForest => BiomeColor::new(0, 102, 51), // #006633 (same as rainforest)
            BiomeType::TropicalSavanna => BiomeColor::new(179, 179, 39),       // #b3b327
            BiomeType::TropicalDryForest => BiomeColor::new(179, 179, 39),     // #b3b327 (savanna color)
            
            // Mediterranean = subtropical (GOAL.md spec)
            BiomeType::SubtropicalRainforest => BiomeColor::new(0, 102, 51), // #006633
            BiomeType::SubtropicalSeasonalForest => BiomeColor::new(179, 153, 51), // #b39933 Mediterranean
            BiomeType::SubtropicalSteppe => BiomeColor::new(179, 179, 102),   // #b3b366 Steppe
            BiomeType::SubtropicalDesert => BiomeColor::new(230, 204, 51),    // #e6cc33 Desert
            
            // GOAL.md colors
            BiomeType::TemperateRainforest => BiomeColor::new(0, 102, 51),   // #006633
            BiomeType::TemperateDeciduousForest => BiomeColor::new(26, 128, 51), // #1a8033 TemperateForest
            BiomeType::TemperateMixedForest => BiomeColor::new(26, 128, 51), // #1a8033
            BiomeType::TemperateSteppe => BiomeColor::new(179, 179, 102),    // #b3b366
            BiomeType::TemperateDesert => BiomeColor::new(230, 204, 51),     // #e6cc33
            
            // GOAL.md: BorealForest = #1a4d1a, Taiga = #00331a
            BiomeType::BorealTaiga => BiomeColor::new(0, 51, 26),             // #00331a
            BiomeType::BorealForest => BiomeColor::new(26, 77, 26),          // #1a4d1a
            BiomeType::TemperateGrassland => BiomeColor::new(179, 179, 102), // #b3b366 Steppe
            
            // GOAL.md: Tundra = #b3cccc, Arctic = #ffffff
            BiomeType::Tundra => BiomeColor::new(179, 204, 204),             // #b3cccc
            BiomeType::Arctic => BiomeColor::new(255, 255, 255),             // #ffffff
            BiomeType::PolarDesert => BiomeColor::new(179, 204, 204),        // #b3cccc (same as tundra)
            
            // GOAL.md: Alpine = #999966, Mountain = #666666
            BiomeType::MontaneForest => BiomeColor::new(102, 102, 102),      // #666666 Mountain
            BiomeType::MontaneGrassland => BiomeColor::new(153, 153, 102),    // #999966 Alpine
            BiomeType::AlpineTundra => BiomeColor::new(153, 153, 102),        // #999966 Alpine
            BiomeType::SnowGlacier => BiomeColor::new(204, 245, 255),        // #ccf5ff Ice
            
            // GOAL.md: Swamp = #1a331a, Marsh = #2d4d1a, Ocean = #1a6699
            BiomeType::CoastalWetland => BiomeColor::new(26, 51, 26),        // #1a331a Swamp
            BiomeType::Mangrove => BiomeColor::new(45, 77, 26),              // #2d4d1a Marsh
            BiomeType::CoralReef => BiomeColor::new(255, 153, 153),          // #ff9999 Reef
            BiomeType::KelpForest => BiomeColor::new(26, 102, 153),           // #1a6699 Ocean
            BiomeType::OpenOcean => BiomeColor::new(26, 102, 153),           // #1a6699 Ocean
            
            // Desert colors from GOAL.md
            BiomeType::HotDesert => BiomeColor::new(230, 204, 51),           // #e6cc33
            BiomeType::ColdDesert => BiomeColor::new(230, 204, 51),          // #e6cc33
            BiomeType::SemiAridSteppe => BiomeColor::new(179, 179, 102),     // #b3b366
            
            // Fantasy biomes - use distinct vibrant colors
            BiomeType::MagicalForest => BiomeColor::new(148, 0, 211),         // Purple
            BiomeType::CrystallineDesert => BiomeColor::new(100, 200, 200),   // Cyan
            BiomeType::BioluminescentOcean => BiomeColor::new(51, 153, 204), // River blue
            BiomeType::VolcanicLandscape => BiomeColor::new(80, 40, 30),    // Dark red
            BiomeType::ToxicSwamp => BiomeColor::new(100, 150, 0),           // Yellow-green
            BiomeType::FloatingIslands => BiomeColor::new(180, 160, 200),   // Light purple
        }
    }
    
    /// Get all colors as a lookup table (indexed by BiomeType as u16).
    pub fn all_colors() -> Vec<BiomeColor> {
        (0u16..37).map(|i| Self::get_color(Self::from_index(i))).collect()
    }
    
    /// Biome count for indexing.
    pub fn biome_count() -> usize {
        37 // Total biomes in BiomeType enum
    }
    
    /// Convert u16 index to BiomeType.
    pub fn from_index(idx: u16) -> BiomeType {
        match idx {
            0 => BiomeType::TropicalRainforest,
            1 => BiomeType::TropicalSeasonalForest,
            2 => BiomeType::TropicalSavanna,
            3 => BiomeType::TropicalDryForest,
            4 => BiomeType::SubtropicalRainforest,
            5 => BiomeType::SubtropicalSeasonalForest,
            6 => BiomeType::SubtropicalSteppe,
            7 => BiomeType::SubtropicalDesert,
            8 => BiomeType::TemperateRainforest,
            9 => BiomeType::TemperateDeciduousForest,
            10 => BiomeType::TemperateMixedForest,
            11 => BiomeType::TemperateSteppe,
            12 => BiomeType::TemperateDesert,
            13 => BiomeType::BorealTaiga,
            14 => BiomeType::BorealForest,
            15 => BiomeType::TemperateGrassland,
            16 => BiomeType::Tundra,
            17 => BiomeType::Arctic,
            18 => BiomeType::PolarDesert,
            19 => BiomeType::MontaneForest,
            20 => BiomeType::MontaneGrassland,
            21 => BiomeType::AlpineTundra,
            22 => BiomeType::SnowGlacier,
            23 => BiomeType::CoastalWetland,
            24 => BiomeType::Mangrove,
            25 => BiomeType::CoralReef,
            26 => BiomeType::KelpForest,
            27 => BiomeType::OpenOcean,
            28 => BiomeType::HotDesert,
            29 => BiomeType::ColdDesert,
            30 => BiomeType::SemiAridSteppe,
            31 => BiomeType::MagicalForest,
            32 => BiomeType::CrystallineDesert,
            33 => BiomeType::BioluminescentOcean,
            34 => BiomeType::VolcanicLandscape,
            35 => BiomeType::ToxicSwamp,
            36 => BiomeType::FloatingIslands,
            _ => BiomeType::OpenOcean, // Default fallback
        }
    }
}

// ============================================================================
// Elevation Configuration
// ============================================================================

/// Elevation adjustment configuration for alpine biomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpineBiomeConfig {
    /// Minimum elevation for alpine biome (meters).
    pub alpine_min_elevation: f32,
    /// Elevation where snow line begins (meters).
    pub snow_line_elevation: f32,
    /// Temperature lapse rate (°C per 1000m).
    pub lapse_rate: f32,
}

impl Default for AlpineBiomeConfig {
    fn default() -> Self {
        Self {
            alpine_min_elevation: 3000.0,
            snow_line_elevation: 4500.0,
            lapse_rate: -6.5,
        }
    }
}

// ============================================================================
// Biome Type Enum
// ============================================================================

/// Core biome types based on Holdridge Life Zone classification.
/// Extended for fantasy/sci-fi genres while maintaining Earth-like base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum BiomeType {
    // Tropical (latitude 0-23°)
    TropicalRainforest = 0,
    TropicalSeasonalForest,
    TropicalSavanna,
    TropicalDryForest,
    
    // Subtropical (latitude 23-35°)
    SubtropicalRainforest,
    SubtropicalSeasonalForest,
    SubtropicalSteppe,
    SubtropicalDesert,
    
    // Temperate (latitude 35-55°)
    TemperateRainforest,
    TemperateDeciduousForest,
    TemperateMixedForest,
    TemperateSteppe,
    TemperateDesert,
    
    // Continental (latitude 55-65°)
    BorealTaiga,
    BorealForest,
    TemperateGrassland,
    
    // Polar (latitude 65-90°)
    Tundra,
    Arctic,
    PolarDesert,
    
    // Special/Ecological
    MontaneForest,
    MontaneGrassland,
    AlpineTundra,
    SnowGlacier,
    
    // Water/Coastal
    CoastalWetland,
    Mangrove,
    CoralReef,
    KelpForest,
    OpenOcean,
    
    // Arid (independent of latitude)
    HotDesert,
    ColdDesert,
    SemiAridSteppe,
    
    // Fantasy/Sci-Fi Extensions
    MagicalForest,
    CrystallineDesert,
    BioluminescentOcean,
    VolcanicLandscape,
    ToxicSwamp,
    FloatingIslands,
}

impl BiomeType {
    /// Get the biome color for rendering.
    pub fn color(&self) -> BiomeColor {
        BiomeColorMapping::get_color(*self)
    }
    
    /// Apply elevation adjustments for alpine biomes.
    /// Returns adjusted biome type based on elevation and temperature.
    pub fn with_elevation_adjustment(&self, elevation: f32, temperature: f32, config: &AlpineBiomeConfig) -> BiomeType {
        // Only apply adjustment for non-special biomes
        if matches!(*self, BiomeType::MontaneForest | BiomeType::MontaneGrassland |
                          BiomeType::AlpineTundra | BiomeType::SnowGlacier |
                          BiomeType::OpenOcean | BiomeType::CoastalWetland |
                          BiomeType::MagicalForest | BiomeType::VolcanicLandscape |
                          BiomeType::FloatingIslands) {
            return *self;
        }
        
        if elevation >= config.snow_line_elevation {
            // Above snow line
            if temperature < 0.0 {
                BiomeType::SnowGlacier
            } else {
                BiomeType::AlpineTundra
            }
        } else if elevation >= config.alpine_min_elevation {
            // Alpine zone
            if temperature < 5.0 {
                BiomeType::AlpineTundra
            } else {
                BiomeType::MontaneGrassland
            }
        } else {
            *self // No adjustment
        }
    }
    
    /// Get the base name for this biome type.
    pub fn name(&self) -> &'static str {
        match self {
            BiomeType::TropicalRainforest => "Tropical Rainforest",
            BiomeType::TropicalSeasonalForest => "Tropical Seasonal Forest",
            BiomeType::TropicalSavanna => "Tropical Savanna",
            BiomeType::TropicalDryForest => "Tropical Dry Forest",
            BiomeType::SubtropicalRainforest => "Subtropical Rainforest",
            BiomeType::SubtropicalSeasonalForest => "Subtropical Seasonal Forest",
            BiomeType::SubtropicalSteppe => "Subtropical Steppe",
            BiomeType::SubtropicalDesert => "Subtropical Desert",
            BiomeType::TemperateRainforest => "Temperate Rainforest",
            BiomeType::TemperateDeciduousForest => "Temperate Deciduous Forest",
            BiomeType::TemperateMixedForest => "Temperate Mixed Forest",
            BiomeType::TemperateSteppe => "Temperate Steppe",
            BiomeType::TemperateDesert => "Temperate Desert",
            BiomeType::BorealTaiga => "Boreal Taiga",
            BiomeType::BorealForest => "Boreal Forest",
            BiomeType::TemperateGrassland => "Temperate Grassland",
            BiomeType::Tundra => "Tundra",
            BiomeType::Arctic => "Arctic",
            BiomeType::PolarDesert => "Polar Desert",
            BiomeType::MontaneForest => "Montane Forest",
            BiomeType::MontaneGrassland => "Montane Grassland",
            BiomeType::AlpineTundra => "Alpine Tundra",
            BiomeType::SnowGlacier => "Snow/Glacier",
            BiomeType::CoastalWetland => "Coastal Wetland",
            BiomeType::Mangrove => "Mangrove",
            BiomeType::CoralReef => "Coral Reef",
            BiomeType::KelpForest => "Kelp Forest",
            BiomeType::OpenOcean => "Open Ocean",
            BiomeType::HotDesert => "Hot Desert",
            BiomeType::ColdDesert => "Cold Desert",
            BiomeType::SemiAridSteppe => "Semi-Arid Steppe",
            BiomeType::MagicalForest => "Magical Forest",
            BiomeType::CrystallineDesert => "Crystalline Desert",
            BiomeType::BioluminescentOcean => "Bioluminescent Ocean",
            BiomeType::VolcanicLandscape => "Volcanic Landscape",
            BiomeType::ToxicSwamp => "Toxic Swamp",
            BiomeType::FloatingIslands => "Floating Islands",
            BiomeType::SubtropicalDesert => "Subtropical Desert",
            BiomeType::TemperateDesert => "Temperate Desert",
        }
    }
    
    /// Get vegetation type for this biome.
    pub fn vegetation(&self) -> VegetationType {
        match self {
            BiomeType::TropicalRainforest | BiomeType::SubtropicalRainforest 
            | BiomeType::TemperateRainforest => VegetationType::DenseForest,
            BiomeType::TropicalSeasonalForest | BiomeType::SubtropicalSeasonalForest
            | BiomeType::TemperateDeciduousForest | BiomeType::TemperateMixedForest
            | BiomeType::BorealForest => VegetationType::Forest,
            BiomeType::TropicalDryForest | BiomeType::MontaneForest => VegetationType::OpenForest,
            BiomeType::BorealTaiga => VegetationType::ConiferousForest,
            BiomeType::TropicalSavanna | BiomeType::SubtropicalSteppe
            | BiomeType::TemperateSteppe | BiomeType::TemperateGrassland
            | BiomeType::SemiAridSteppe | BiomeType::MontaneGrassland => VegetationType::Grassland,
            BiomeType::Tundra | BiomeType::AlpineTundra => VegetationType::Tundra,
            BiomeType::Arctic | BiomeType::PolarDesert | BiomeType::SnowGlacier => VegetationType::SnowIce,
            BiomeType::HotDesert | BiomeType::ColdDesert => VegetationType::Desert,
            BiomeType::CoastalWetland | BiomeType::Mangrove => VegetationType::Wetland,
            BiomeType::CoralReef | BiomeType::KelpForest => VegetationType::MarineVegetation,
            BiomeType::OpenOcean | BiomeType::BioluminescentOcean => VegetationType::OpenWater,
            BiomeType::MagicalForest => VegetationType::MagicalVegetation,
            BiomeType::CrystallineDesert => VegetationType::CrystalVegetation,
            BiomeType::VolcanicLandscape => VegetationType::VolcanicVegetation,
            BiomeType::ToxicSwamp => VegetationType::ToxicVegetation,
            BiomeType::FloatingIslands => VegetationType::SkyVegetation,
            BiomeType::SubtropicalDesert => VegetationType::Desert,
            BiomeType::TemperateDesert => VegetationType::Desert,
        }
    }
    
    /// Check if this biome is a mountain/high elevation type.
    /// Used for resource affinity calculations (e.g., mountains boost minerals).
    pub fn is_mountain(&self) -> bool {
        matches!(
            self,
            BiomeType::AlpineTundra 
            | BiomeType::MontaneForest 
            | BiomeType::MontaneGrassland
            | BiomeType::VolcanicLandscape
            | BiomeType::SnowGlacier
            | BiomeType::PolarDesert
        )
    }
    
    /// Check if this biome supports the given resource type.
    pub fn can_have_resource(&self, resource: &ResourceCategory) -> bool {
        match (self, resource) {
            // Forests have timber and game
            (vt, ResourceCategory::Timber) if matches!(
                vt.vegetation(), 
                VegetationType::DenseForest | VegetationType::Forest | VegetationType::OpenForest | VegetationType::ConiferousForest
            ) => true,
            // Grasslands have livestock and agriculture (grain)
            (vt, ResourceCategory::Livestock | ResourceCategory::Agriculture) if matches!(
                vt.vegetation(),
                VegetationType::Grassland
            ) => true,
            // Deserts can have minerals (industrial) and fossil fuels
            (vt, ResourceCategory::IndustrialMinerals | ResourceCategory::FossilFuels) if matches!(
                vt.vegetation(),
                VegetationType::Desert | VegetationType::CrystalVegetation
            ) => true,
            // Wetlands have fish and reeds
            (vt, ResourceCategory::Fishing) if matches!(
                vt.vegetation(),
                VegetationType::Wetland | VegetationType::MarineVegetation | VegetationType::OpenWater
            ) => true,
            // All biomes have water access based on coastal
            (vt, ResourceCategory::FreshWater) if matches!(
                vt.vegetation(),
                VegetationType::Wetland | VegetationType::DenseForest | VegetationType::Forest
            ) => true,
            // Special biomes have unique resources
            (BiomeType::MagicalForest, ResourceCategory::MagicalMaterials) => true,
            (BiomeType::VolcanicLandscape, ResourceCategory::VolcanicMinerals) => true,
            (BiomeType::ToxicSwamp, ResourceCategory::Alchemical) => true,
            // Default deny
            _ => false,
        }
    }
}

// ============================================================================
// Vegetation Type
// ============================================================================

/// Vegetation classification for rendering/complexity grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VegetationType {
    DenseForest,
    Forest,
    OpenForest,
    ConiferousForest,
    Grassland,
    Tundra,
    SnowIce,
    Desert,
    Wetland,
    MarineVegetation,
    OpenWater,
    MagicalVegetation,
    CrystalVegetation,
    VolcanicVegetation,
    ToxicVegetation,
    SkyVegetation,
}

/// Resource categories that can be found in biomes.
/// 
/// DEPRECATED: Use `ResourceCategory` from `resource_types` module instead.
/// This enum is kept for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[deprecated(since = "0.1.0", note = "Use ResourceCategory from resource_types module")]
pub enum OldResourceCategory {
    Timber,
    Livestock,
    Grain,
    Minerals,
    Oil,
    Fish,
    FreshWater,
    MagicalMaterials,
    VolcanicMinerals,
    RareChemicals,
}

// ============================================================================
// Climate and Environment Types
// ============================================================================

/// Climate zone based on latitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClimateZone {
    Tropical,     // 0-23° latitude
    Subtropical,  // 23-35° latitude
    Temperate,    // 35-55° latitude
    Boreal,       // 55-65° latitude
    Polar,        // 65-90° latitude
}

impl ClimateZone {
    /// Short name for display.
    pub fn short_name(&self) -> &'static str {
        match self {
            ClimateZone::Tropical => "tropical",
            ClimateZone::Subtropical => "subtropical",
            ClimateZone::Temperate => "temperate",
            ClimateZone::Boreal => "boreal",
            ClimateZone::Polar => "polar",
        }
    }
}

/// Moisture level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoistureLevel {
    HyperArid,   // 0-10% precipitation
    Arid,        // 10-25% precipitation
    SemiArid,    // 25-50% precipitation
    SubHumid,    // 50-75% precipitation
    Humid,       // 75-90% precipitation
    PerHumid,    // 90-100% precipitation
}

/// Elevation zone classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElevationZone {
    Lowland,      // 0-500m - sea level to hills
    Midland,      // 500-1500m - rolling hills to lower mountains
    Highland,     // 1500-3000m - mountain slopes
    Alpine,       // 3000-5000m - above tree line
    Nival,        // 5000m+ - permanent snow/ice
}

impl ElevationZone {
    /// Determine elevation zone from height in meters.
    pub fn from_height(height_m: f32) -> Self {
        if height_m < 0.0 {
            ElevationZone::Lowland // Below sea level, treat as lowland
        } else if height_m < 500.0 {
            ElevationZone::Lowland
        } else if height_m < 1500.0 {
            ElevationZone::Midland
        } else if height_m < 3000.0 {
            ElevationZone::Highland
        } else if height_m < 5000.0 {
            ElevationZone::Alpine
        } else {
            ElevationZone::Nival
        }
    }
}