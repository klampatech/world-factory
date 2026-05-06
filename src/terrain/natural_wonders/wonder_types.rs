//! Wonder Type Definitions for World Factory
//!
//! Defines all natural wonder types, their properties, constraints, and effects.

use super::{WonderIconType, WonderVisualProperties};
use serde::{Deserialize, Serialize};

/// Category of natural wonder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WonderCategory {
    /// Geological formations (mountains, canyons, caves)
    Geological = 0,
    /// Hydrological features (waterfalls, lakes, springs)
    Hydrological,
    /// Biological/ecosystem wonders (ancient forests, coral)
    Biological,
    /// Atmospheric phenomena (auroras, lightning storms)
    Atmospheric,
    /// Magical/supernatural wonders
    Magical,
    /// Combined or unique wonders
    Unique,
}

impl WonderCategory {
    pub fn name(&self) -> &'static str {
        match self {
            WonderCategory::Geological => "Geological",
            WonderCategory::Hydrological => "Hydrological",
            WonderCategory::Biological => "Biological",
            WonderCategory::Atmospheric => "Atmospheric",
            WonderCategory::Magical => "Magical",
            WonderCategory::Unique => "Unique",
        }
    }
}

/// Base effect that a wonder provides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderEffect {
    /// Type of bonus (e.g., resource bonus, population bonus)
    pub bonus_type: super::WonderBonusType,
    /// Magnitude of the effect
    pub magnitude: f32,
    /// Radius of effect (cells), 0 = single cell
    pub radius: f32,
    /// Whether this affects the entire region
    pub region_wide: bool,
}

/// Properties that define how a wonder type is spawned and behaves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderProperties {
    /// Base spawn weight (higher = more likely)
    pub spawn_weight: f32,
    /// Minimum elevation requirement (meters)
    pub min_elevation: f32,
    /// Maximum elevation requirement (meters)
    pub max_elevation: f32,
    /// Valid biome categories for spawning
    pub valid_biomes: Vec<super::wonder_types::BiomeConstraint>,
    /// Whether this wonder requires water nearby
    pub requires_water: bool,
    /// Whether this wonder requires mountains nearby
    pub requires_mountains: bool,
    /// Influence radius in cells
    pub influence_radius: u32,
    /// Unique name prefix for world gen
    pub name_prefix: &'static str,
}

/// Biome constraint for wonder spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConstraint {
    /// Biome types that allow this wonder (empty = any)
    pub allowed: Vec<super::WonderType>,
    /// Biome types that forbid this wonder
    pub forbidden: Vec<super::WonderType>,
    /// Minimum latitude constraint (0-1, 0.5 = equator)
    pub min_latitude: Option<f32>,
    /// Maximum latitude constraint
    pub max_latitude: Option<f32>,
}

/// Defines a specific wonder type with its properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderTypeDef {
    /// The wonder type enum variant
    pub wonder_type: WonderType,
    /// Display name
    pub name: &'static str,
    /// Category
    pub category: WonderCategory,
    /// Base properties for spawning
    pub properties: WonderProperties,
    /// Effects provided by this wonder type
    pub effects: Vec<WonderEffect>,
    /// Default visual properties
    pub visual: WonderVisualProperties,
    /// Base description for tooltips
    pub description: &'static str,
}

/// Specific instance data for a wonder (with unique name, position).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderData {
    /// Wonder type
    pub wonder_type: WonderType,
    /// Instance name (world-specific)
    pub instance_name: String,
    /// World position
    pub x: f32,
    pub y: f32,
    /// Unique seed for this instance
    pub seed: u64,
}

// ============================================================================
// WONDER TYPE DEFINITIONS
// ============================================================================

/// All natural wonder types in World Factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum WonderType {
    // === Geological Wonders ===
    /// Majestic mountain peak or mountain range
    SacredMountain = 0,
    /// Deep dramatic canyon
    GrandCanyon,
    /// Enormous ancient tree
    AncientTree,
    /// Crystal cave system
    CrystalCavern,
    /// Volcanic crater or mountain
    ActiveVolcano,
    /// Natural stone arch or bridge
    NaturalArch,

    // === Hydrological Wonders ===
    /// Massive waterfall
    MagnificentWaterfall,
    /// Large freshwater lake
    GreatLake,
    /// Underground hot spring
    MysticHotSpring,
    /// Desert oasis
    HiddenOasis,
    /// Geyser field
    GeyserField,

    // === Biological Wonders ===
    /// Ancient forest with unique ecology
    AncientForest,
    /// Bioluminescent lake
    BioluminescentLake,
    /// Enormous single tree
    WorldTree,
    /// Coral formation wonder
    CoralWonder,
    /// Fungal megastructure
    FungalTower,

    // === Atmospheric Wonders ===
    /// Northern lights phenomenon
    AuroraBorealis,
    /// Permanent lightning storm
    EternalStorm,
    /// Mysterious fog phenomenon
    EternalMist,

    // === Magical Wonders ===
    /// Intersection of ley lines
    LeyLineNexus,
    /// Source of magical energy
    ManaSpring,
    /// Portal to another realm
    MysticPortal,
    /// Ancient magical ruins
    AncientRuins,
    /// Swirling magical vortex
    MagicalVortex,

    // === Unique/Combined Wonders ===
    /// Combination waterfall and temple
    TempleFalls,
    /// Dragon's lair combination
    DragonsLair,
    /// Floating islands
    FloatingIslands,
    /// Giant crystal formation
    CrystalPeak,
}

impl WonderType {
    /// Get display name for this wonder type.
    pub fn name(&self) -> &'static str {
        match self {
            WonderType::SacredMountain => "Sacred Mountain",
            WonderType::GrandCanyon => "Grand Canyon",
            WonderType::AncientTree => "Ancient Tree",
            WonderType::CrystalCavern => "Crystal Cavern",
            WonderType::ActiveVolcano => "Active Volcano",
            WonderType::NaturalArch => "Natural Arch",
            WonderType::MagnificentWaterfall => "Magnificent Waterfall",
            WonderType::GreatLake => "Great Lake",
            WonderType::MysticHotSpring => "Mystic Hot Spring",
            WonderType::HiddenOasis => "Hidden Oasis",
            WonderType::GeyserField => "Geyser Field",
            WonderType::AncientForest => "Ancient Forest",
            WonderType::BioluminescentLake => "Bioluminescent Lake",
            WonderType::WorldTree => "World Tree",
            WonderType::CoralWonder => "Coral Wonder",
            WonderType::FungalTower => "Fungal Tower",
            WonderType::AuroraBorealis => "Aurora Borealis",
            WonderType::EternalStorm => "Eternal Storm",
            WonderType::EternalMist => "Eternal Mist",
            WonderType::LeyLineNexus => "Ley Line Nexus",
            WonderType::ManaSpring => "Mana Spring",
            WonderType::MysticPortal => "Mystic Portal",
            WonderType::AncientRuins => "Ancient Ruins",
            WonderType::MagicalVortex => "Magical Vortex",
            WonderType::TempleFalls => "Temple Falls",
            WonderType::DragonsLair => "Dragon's Lair",
            WonderType::FloatingIslands => "Floating Islands",
            WonderType::CrystalPeak => "Crystal Peak",
        }
    }

    /// Get the category for this wonder type.
    pub fn category(&self) -> WonderCategory {
        match self {
            WonderType::SacredMountain
            | WonderType::GrandCanyon
            | WonderType::CrystalCavern
            | WonderType::ActiveVolcano
            | WonderType::NaturalArch => WonderCategory::Geological,

            WonderType::MagnificentWaterfall
            | WonderType::GreatLake
            | WonderType::MysticHotSpring
            | WonderType::HiddenOasis
            | WonderType::GeyserField => WonderCategory::Hydrological,

            WonderType::AncientForest
            | WonderType::BioluminescentLake
            | WonderType::WorldTree
            | WonderType::AncientTree
            | WonderType::CoralWonder
            | WonderType::FungalTower => WonderCategory::Biological,

            WonderType::AuroraBorealis | WonderType::EternalStorm | WonderType::EternalMist => {
                WonderCategory::Atmospheric
            }

            WonderType::LeyLineNexus
            | WonderType::ManaSpring
            | WonderType::MysticPortal
            | WonderType::AncientRuins
            | WonderType::MagicalVortex => WonderCategory::Magical,

            WonderType::TempleFalls
            | WonderType::DragonsLair
            | WonderType::FloatingIslands
            | WonderType::CrystalPeak => WonderCategory::Unique,
        }
    }

    /// Get icon type for rendering.
    pub fn icon_type(&self) -> WonderIconType {
        match self {
            WonderType::SacredMountain => WonderIconType::Mountain,
            WonderType::GrandCanyon => WonderIconType::Canyon,
            WonderType::AncientTree | WonderType::WorldTree => WonderIconType::AncientTree,
            WonderType::CrystalCavern | WonderType::CrystalPeak => WonderIconType::Crystal,
            WonderType::ActiveVolcano => WonderIconType::Volcano,
            WonderType::NaturalArch => WonderIconType::Cave,
            WonderType::MagnificentWaterfall => WonderIconType::Waterfall,
            WonderType::GreatLake => WonderIconType::Lake,
            WonderType::MysticHotSpring => WonderIconType::HotSpring,
            WonderType::HiddenOasis => WonderIconType::Oasis,
            WonderType::GeyserField => WonderIconType::Geyser,
            WonderType::AncientForest => WonderIconType::Forest,
            WonderType::BioluminescentLake => WonderIconType::Lake,
            WonderType::CoralWonder => WonderIconType::Cave,
            WonderType::FungalTower => WonderIconType::AncientTree,
            WonderType::AuroraBorealis => WonderIconType::Aurora,
            WonderType::EternalStorm => WonderIconType::Lightning,
            WonderType::EternalMist => WonderIconType::Aura,
            WonderType::LeyLineNexus => WonderIconType::LeyLine,
            WonderType::ManaSpring => WonderIconType::Aura,
            WonderType::MysticPortal => WonderIconType::Portal,
            WonderType::AncientRuins => WonderIconType::Ruins,
            WonderType::MagicalVortex => WonderIconType::Portal,
            WonderType::TempleFalls => WonderIconType::Waterfall,
            WonderType::DragonsLair => WonderIconType::Volcano,
            WonderType::FloatingIslands => WonderIconType::Mountain,
        }
    }

    /// Get spawn properties for this wonder type.
    pub fn properties(&self) -> WonderProperties {
        match *self {
            WonderType::SacredMountain => WonderProperties {
                spawn_weight: 1.5,
                min_elevation: 1500.0,
                max_elevation: 6000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 5,
                name_prefix: "Mount",
            },
            WonderType::GrandCanyon => WonderProperties {
                spawn_weight: 1.0,
                min_elevation: 200.0,
                max_elevation: 2500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::AncientTree => WonderProperties {
                spawn_weight: 1.2,
                min_elevation: 0.0,
                max_elevation: 2000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "The",
            },
            WonderType::CrystalCavern => WonderProperties {
                spawn_weight: 0.8,
                min_elevation: -200.0,
                max_elevation: 500.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 2,
                name_prefix: "",
            },
            WonderType::ActiveVolcano => WonderProperties {
                spawn_weight: 0.6,
                min_elevation: 500.0,
                max_elevation: 3000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 4,
                name_prefix: "",
            },
            WonderType::NaturalArch => WonderProperties {
                spawn_weight: 0.5,
                min_elevation: 100.0,
                max_elevation: 2000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 2,
                name_prefix: "The",
            },
            WonderType::MagnificentWaterfall => WonderProperties {
                spawn_weight: 1.3,
                min_elevation: 100.0,
                max_elevation: 1500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "",
            },
            WonderType::GreatLake => WonderProperties {
                spawn_weight: 1.4,
                min_elevation: -50.0,
                max_elevation: 3000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 6,
                name_prefix: "Lake",
            },
            WonderType::MysticHotSpring => WonderProperties {
                spawn_weight: 0.9,
                min_elevation: 0.0,
                max_elevation: 1000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: true,
                influence_radius: 2,
                name_prefix: "",
            },
            WonderType::HiddenOasis => WonderProperties {
                spawn_weight: 0.7,
                min_elevation: -50.0,
                max_elevation: 200.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 2,
                name_prefix: "",
            },
            WonderType::GeyserField => WonderProperties {
                spawn_weight: 0.5,
                min_elevation: 100.0,
                max_elevation: 2000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 3,
                name_prefix: "",
            },
            WonderType::AncientForest => WonderProperties {
                spawn_weight: 1.1,
                min_elevation: 0.0,
                max_elevation: 1500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::BioluminescentLake => WonderProperties {
                spawn_weight: 0.6,
                min_elevation: -10.0,
                max_elevation: 500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "",
            },
            WonderType::WorldTree => WonderProperties {
                spawn_weight: 0.4,
                min_elevation: 0.0,
                max_elevation: 500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 5,
                name_prefix: "The",
            },
            WonderType::CoralWonder => WonderProperties {
                spawn_weight: 0.8,
                min_elevation: -50.0,
                max_elevation: 0.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::FungalTower => WonderProperties {
                spawn_weight: 0.3,
                min_elevation: 0.0,
                max_elevation: 1000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "The",
            },
            WonderType::AuroraBorealis => WonderProperties {
                spawn_weight: 0.7,
                min_elevation: 0.0,
                max_elevation: 5000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 10,
                name_prefix: "The",
            },
            WonderType::EternalStorm => WonderProperties {
                spawn_weight: 0.5,
                min_elevation: 0.0,
                max_elevation: 3000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 5,
                name_prefix: "The",
            },
            WonderType::EternalMist => WonderProperties {
                spawn_weight: 0.6,
                min_elevation: 0.0,
                max_elevation: 1000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::LeyLineNexus => WonderProperties {
                spawn_weight: 0.5,
                min_elevation: 0.0,
                max_elevation: 2000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 6,
                name_prefix: "",
            },
            WonderType::ManaSpring => WonderProperties {
                spawn_weight: 0.4,
                min_elevation: 0.0,
                max_elevation: 1500.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "",
            },
            WonderType::MysticPortal => WonderProperties {
                spawn_weight: 0.3,
                min_elevation: 0.0,
                max_elevation: 3000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 2,
                name_prefix: "The",
            },
            WonderType::AncientRuins => WonderProperties {
                spawn_weight: 0.8,
                min_elevation: 0.0,
                max_elevation: 2000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 3,
                name_prefix: "The",
            },
            WonderType::MagicalVortex => WonderProperties {
                spawn_weight: 0.3,
                min_elevation: 0.0,
                max_elevation: 4000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::TempleFalls => WonderProperties {
                spawn_weight: 0.4,
                min_elevation: 100.0,
                max_elevation: 1000.0,
                valid_biomes: vec![],
                requires_water: true,
                requires_mountains: true,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::DragonsLair => WonderProperties {
                spawn_weight: 0.3,
                min_elevation: 500.0,
                max_elevation: 2500.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 5,
                name_prefix: "",
            },
            WonderType::FloatingIslands => WonderProperties {
                spawn_weight: 0.2,
                min_elevation: 1000.0,
                max_elevation: 5000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: false,
                influence_radius: 4,
                name_prefix: "The",
            },
            WonderType::CrystalPeak => WonderProperties {
                spawn_weight: 0.5,
                min_elevation: 1000.0,
                max_elevation: 4000.0,
                valid_biomes: vec![],
                requires_water: false,
                requires_mountains: true,
                influence_radius: 3,
                name_prefix: "",
            },
        }
    }

    /// Get base effects for this wonder type.
    pub fn effects(&self) -> Vec<WonderEffect> {
        use super::WonderBonusType::*;
        match self {
            WonderType::SacredMountain => vec![
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 2.0,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: DefenseBonus,
                    magnitude: 1.5,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::GrandCanyon => vec![
                WonderEffect {
                    bonus_type: TradeBonus,
                    magnitude: 1.3,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.5,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::AncientTree => vec![
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.4,
                    radius: 3.0,
                    region_wide: false,
                },
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.2,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::CrystalCavern => vec![
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.5,
                    radius: 2.0,
                    region_wide: false,
                },
                WonderEffect {
                    bonus_type: ResourceBonus("Gemstones".to_string()),
                    magnitude: 2.0,
                    radius: 1.0,
                    region_wide: false,
                },
            ],
            WonderType::ActiveVolcano => vec![
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.6,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ResourceBonus("Sulfur".to_string()),
                    magnitude: 2.5,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::NaturalArch => vec![
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.4,
                    radius: 3.0,
                    region_wide: false,
                },
                WonderEffect {
                    bonus_type: TourismBonus,
                    magnitude: 2.0,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::MagnificentWaterfall => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.2,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::GreatLake => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.6,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.3,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::MysticHotSpring => vec![
                WonderEffect {
                    bonus_type: PopulationGrowth,
                    magnitude: 1.3,
                    radius: 2.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.4,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::HiddenOasis => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.8,
                    radius: 2.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.2,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::GeyserField => vec![
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.4,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: EnergyBonus,
                    magnitude: 2.0,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::AncientForest => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.4,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.3,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::BioluminescentLake => vec![
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.4,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::WorldTree => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.5,
                    radius: 5.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.4,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: PopulationGrowth,
                    magnitude: 1.3,
                    radius: 3.0,
                    region_wide: true,
                },
            ],
            WonderType::CoralWonder => vec![
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.7,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.4,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::FungalTower => vec![
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: false,
                },
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.3,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::AuroraBorealis => vec![
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.4,
                    radius: 6.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.6,
                    radius: 5.0,
                    region_wide: true,
                },
            ],
            WonderType::EternalStorm => vec![
                WonderEffect {
                    bonus_type: ProductionBonus,
                    magnitude: 1.5,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: DefenseBonus,
                    magnitude: 2.0,
                    radius: 3.0,
                    region_wide: true,
                },
            ],
            WonderType::EternalMist => vec![
                WonderEffect {
                    bonus_type: DefenseBonus,
                    magnitude: 1.6,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.3,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::LeyLineNexus => vec![
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.5,
                    radius: 5.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.4,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.3,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::ManaSpring => vec![
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.6,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.4,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::MysticPortal => vec![
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 2.0,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: TradeBonus,
                    magnitude: 1.8,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::AncientRuins => vec![
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.4,
                    radius: 2.0,
                    region_wide: false,
                },
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.3,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::MagicalVortex => vec![
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.8,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ScienceBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::TempleFalls => vec![
                WonderEffect {
                    bonus_type: FaithBonus,
                    magnitude: 1.8,
                    radius: 4.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.5,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: FoodBonus,
                    magnitude: 1.3,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
            WonderType::DragonsLair => vec![
                WonderEffect {
                    bonus_type: DefenseBonus,
                    magnitude: 2.0,
                    radius: 5.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.6,
                    radius: 3.0,
                    region_wide: false,
                },
            ],
            WonderType::FloatingIslands => vec![
                WonderEffect {
                    bonus_type: TradeBonus,
                    magnitude: 1.7,
                    radius: 5.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: CultureBonus,
                    magnitude: 1.5,
                    radius: 4.0,
                    region_wide: false,
                },
            ],
            WonderType::CrystalPeak => vec![
                WonderEffect {
                    bonus_type: GoldBonus,
                    magnitude: 1.7,
                    radius: 3.0,
                    region_wide: true,
                },
                WonderEffect {
                    bonus_type: ResourceBonus("Diamonds".to_string()),
                    magnitude: 2.5,
                    radius: 2.0,
                    region_wide: false,
                },
            ],
        }
    }

    /// Get base description for this wonder.
    pub fn description(&self) -> &'static str {
        match self {
            WonderType::SacredMountain => {
                "A majestic peak revered by civilizations throughout history."
            }
            WonderType::GrandCanyon => "A breathtaking chasm carved by ancient waters.",
            WonderType::AncientTree => "A towering specimen that has witnessed millennia pass.",
            WonderType::CrystalCavern => "Crystalline formations glitter in the eternal darkness.",
            WonderType::ActiveVolcano => "A smoldering giant that shapes the land around it.",
            WonderType::NaturalArch => "An impossible stone formation spanning the void.",
            WonderType::MagnificentWaterfall => "Cascading waters thunder into the abyss below.",
            WonderType::GreatLake => "A vast inland sea of crystalline waters.",
            WonderType::MysticHotSpring => {
                "Waters warmed by the earth's heart soothe all who bathe."
            }
            WonderType::HiddenOasis => "Life flourishes in this verdant sanctuary amid the sands.",
            WonderType::GeyserField => "Steam and boiling water erupt in rhythmic fury.",
            WonderType::AncientForest => "Trees older than memory shelter countless secrets.",
            WonderType::BioluminescentLake => {
                "Gentle light emanates from countless tiny organisms."
            }
            WonderType::WorldTree => "A tree so vast it touches the heavens themselves.",
            WonderType::CoralWonder => "An underwater cathedral of living stone.",
            WonderType::FungalTower => "Bioluminescent fungi reach toward the sky.",
            WonderType::AuroraBorealis => "Dancing lights paint the night sky in ethereal colors.",
            WonderType::EternalStorm => {
                "Lightning has struck this place for as long as records exist."
            }
            WonderType::EternalMist => {
                "An impenetrable fog that has shrouded this valley since time immemorial."
            }
            WonderType::LeyLineNexus => {
                "Magical energy concentrates at this intersection of power."
            }
            WonderType::ManaSpring => "Pure magical essence flows from this sacred spring.",
            WonderType::MysticPortal => "A shimmering gateway to realms unknown.",
            WonderType::AncientRuins => "Remnants of a civilization lost to time.",
            WonderType::MagicalVortex => "Reality itself seems to twist at this junction.",
            WonderType::TempleFalls => "Nature and architecture merge in perfect harmony.",
            WonderType::DragonsLair => "Only the bravest dare approach this fearsome place.",
            WonderType::FloatingIslands => "Earth defies gravity in this impossible vista.",
            WonderType::CrystalPeak => "Massive crystals catch and refract the sunlight.",
        }
    }
}

// ============================================================================
// WONDER REGISTRIES
// ============================================================================

/// All wonder types in an array for iteration.
pub const WONDER_TYPES: [WonderType; 28] = [
    WonderType::SacredMountain,
    WonderType::GrandCanyon,
    WonderType::AncientTree,
    WonderType::CrystalCavern,
    WonderType::ActiveVolcano,
    WonderType::NaturalArch,
    WonderType::MagnificentWaterfall,
    WonderType::GreatLake,
    WonderType::MysticHotSpring,
    WonderType::HiddenOasis,
    WonderType::GeyserField,
    WonderType::AncientForest,
    WonderType::BioluminescentLake,
    WonderType::WorldTree,
    WonderType::CoralWonder,
    WonderType::FungalTower,
    WonderType::AuroraBorealis,
    WonderType::EternalStorm,
    WonderType::EternalMist,
    WonderType::LeyLineNexus,
    WonderType::ManaSpring,
    WonderType::MysticPortal,
    WonderType::AncientRuins,
    WonderType::MagicalVortex,
    WonderType::TempleFalls,
    WonderType::DragonsLair,
    WonderType::FloatingIslands,
    WonderType::CrystalPeak,
];

/// Well-known named wonders that may appear in world generation.
/// These are special instances with unique names and guaranteed properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownWonder {
    pub wonder_type: WonderType,
    pub name: &'static str,
    pub description: &'static str,
    /// Minimum world size (cells) to support this wonder
    pub min_world_size: u32,
    /// Whether this wonder is unique per world (only one can spawn)
    pub unique_per_world: bool,
}

pub const KNOWN_WONDERS: [KnownWonder; 12] = [
    KnownWonder {
        wonder_type: WonderType::SacredMountain,
        name: "Mount Olympus",
        description: "The throne of the gods, piercing the clouds themselves.",
        min_world_size: 256,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::GreatLake,
        name: "Lake Avalon",
        description: "The mystical lake where legends go to be born.",
        min_world_size: 128,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::WorldTree,
        name: "Yggdrasil",
        description: "The cosmic ash tree connecting all realms.",
        min_world_size: 512,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::AncientRuins,
        name: "Atlantis",
        description: "The sunken city of an advanced civilization.",
        min_world_size: 256,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::ActiveVolcano,
        name: "Mount Doom",
        description: "A volcano of immense power and peril.",
        min_world_size: 128,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::MagnificentWaterfall,
        name: "Niagara",
        description: "Water cascades in thunderous majesty.",
        min_world_size: 64,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::LeyLineNexus,
        name: "Stonehenge",
        description: "Ancient stones aligned with the cosmic currents.",
        min_world_size: 64,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::MysticPortal,
        name: "The Great Gate",
        description: "A portal of unknown origin between worlds.",
        min_world_size: 128,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::FloatingIslands,
        name: "Laputa",
        description: "The legendary flying kingdom.",
        min_world_size: 256,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::GrandCanyon,
        name: "The Grand Gorge",
        description: "An immense chasm revealing millions of years of history.",
        min_world_size: 192,
        unique_per_world: true,
    },
    KnownWonder {
        wonder_type: WonderType::AuroraBorealis,
        name: "The Celestial Curtain",
        description: "Lights dance eternally across the polar sky.",
        min_world_size: 128,
        unique_per_world: false,
    },
    KnownWonder {
        wonder_type: WonderType::CrystalCavern,
        name: "Crystal Kingdom",
        description: "An underground palace of prismatic formations.",
        min_world_size: 128,
        unique_per_world: true,
    },
];
