//! Resource type definitions for World Factory.
//!
//! Resources are natural materials that can be extracted from biomes.
//! Each resource type has properties that affect generation, rendering, and value.
//!
//! # Design
//! - Resources are categorized by origin (mineral, organic, magical, etc.)
//! - Biomes define which resource categories they support
//! - Resource richness determines abundance in each region
//! - Both Earth-like and fantasy/sci-fi resources are supported
//!
//! # Example Usage
//! ```rust
//! use world_factory::terrain::resource_types::{ResourceType, ResourceRichness};
//! use world_factory::terrain::biome::VegetationType;
//!
//! // Check if a resource is viable in a biome
//! let iron = ResourceType::IronOre;
//! let forest = VegetationType::DenseForest;
//!
//! // Note: Use ALL_RESOURCE_CATEGORIES for viability checking
//! // Resource richness determines abundance in regions
//! let richness = ResourceRichness::Abundant;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core resource types found in worlds.
///
/// Each type has:
/// - `name`: Human-readable display name
/// - `category`: Grouping for UI and logic
/// - `base_value`: Relative economic value (arbitrary units)
/// - `rarity`: Spawn rarity modifier (0.0-1.0, higher = rarer)
/// - `is_magical`: Whether this requires fantasy/sci-fi rules
/// - `is_coastal`: Whether this is found near water bodies

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ResourceType {
    // =========================================================================
    // MINERAL RESOURCES
    // =========================================================================
    /// Iron ore - common, foundational for industry
    IronOre = 0,
    /// Copper ore - common, essential for alloys
    CopperOre,
    /// Tin ore - rare, used for bronze
    TinOre,
    /// Coal - common, fuel for industry
    Coal,
    /// Gold ore - rare, high value currency
    GoldOre,
    /// Silver ore - uncommon, currency and jewelry
    SilverOre,
    /// Lead ore - uncommon, construction
    LeadOre,
    /// Platinum ore - very rare, high value
    PlatinumOre,
    /// Gemstones (diamonds, rubies, emeralds) - rare
    Gemstones,
    /// Limestone - common, construction and steel
    Limestone,
    /// Granite - common, construction
    Granite,
    /// Marble - uncommon, luxury construction
    Marble,
    /// Salt - common, essential resource
    Salt,
    /// Sulfur - uncommon, industrial
    Sulfur,
    /// Clay - common, ceramics
    Clay,
    /// Precious gems (specific) - very rare
    Diamonds,
    Rubies,
    Emeralds,

    // =========================================================================
    // FUEL AND ENERGY RESOURCES
    // =========================================================================
    /// Crude oil - rare, liquid fuel
    Oil,
    /// Natural gas - rare, gaseous fuel
    NaturalGas,
    /// Uranium ore - very rare, nuclear fuel
    UraniumOre,
    /// Thorium ore - rare, nuclear fuel
    ThoriumOre,
    /// Lithium ore - rare, batteries
    LithiumOre,

    // =========================================================================
    // ORGANIC AND AGRICULTURAL RESOURCES
    // =========================================================================
    /// Timber from forests
    Timber,
    /// Stone for construction
    Stone,
    /// Fresh water sources
    FreshWater,
    /// Fertile soil for agriculture
    FertileSoil,
    /// Wild game animals
    Game,
    /// Fish from water bodies
    Fish,
    /// Whales and marine mammals
    WhaleOil,
    /// Spices and exotic plants
    Spices,
    /// Cotton and plant fibers
    Cotton,
    /// Herbs and medicinal plants
    Herbs,
    /// Tobacco and luxury plants
    Tobacco,
    /// Sugar cane and sugar beets
    Sugar,
    /// Grapes for wine production
    Grapes,
    /// Olives for oil production
    Olives,
    /// Furs from animals
    Furs,
    /// Leather from livestock
    Leather,

    // =========================================================================
    // LIVESTOCK RESOURCES
    // =========================================================================
    /// Cattle grazing land
    Cattle,
    /// Sheep grazing land
    Sheep,
    /// Horse breeding land
    Horses,
    /// Pigs
    Pigs,

    // =========================================================================
    // SPECIAL AND RARE RESOURCES
    // =========================================================================
    /// Pearl oysters in coastal zones
    Pearls,
    /// Rare coral formations
    Coral,
    /// Amber and fossilized resin
    Amber,
    /// Sea salt from evaporation
    SeaSalt,
    /// Bird feathers for luxury goods
    Feathers,
    /// Rare medicinal ingredients
    RareHerbs,

    // =========================================================================
    // FANTASY AND SCI-FI RESOURCES
    // =========================================================================
    /// Magic-infused crystals
    MagicCrystals,
    /// Infused wood from magical forests
    EnchantedWood,
    /// Magical herbs
    MysticalHerbs,
    /// Volcanic minerals (obsidian, sulfur, etc.)
    VolcanicMinerals,
    /// Dragon scale equivalent
    ExoticHide,
    /// Rare alchemical ingredients
    AlchemicalIngredients,
    /// Rare earth elements
    RareEarth,
    /// Mithril equivalent - legendary metal
    Mithril,
    /// Adamantine equivalent - legendary metal
    Adamantine,
    /// Star metal from meteors
    StarMetal,
    /// Soul essence for necromancy
    SoulEssence,
    /// Mana crystals
    ManaCrystals,
    /// Ethereal ores (planar)
    EtherealOre,
    /// Bioluminescent organisms
    BioluminescentOrganisms,
    /// Radiative crystals
    RadiativeCrystals,
    /// Void matter
    VoidMatter,
    /// Xenobiological compounds
    Xenocompounds,
}

impl ResourceType {
    /// Get the display name for this resource type.
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::IronOre => "Iron Ore",
            ResourceType::CopperOre => "Copper Ore",
            ResourceType::TinOre => "Tin Ore",
            ResourceType::Coal => "Coal",
            ResourceType::GoldOre => "Gold Ore",
            ResourceType::SilverOre => "Silver Ore",
            ResourceType::LeadOre => "Lead Ore",
            ResourceType::PlatinumOre => "Platinum Ore",
            ResourceType::Gemstones => "Gemstones",
            ResourceType::Limestone => "Limestone",
            ResourceType::Granite => "Granite",
            ResourceType::Marble => "Marble",
            ResourceType::Salt => "Salt",
            ResourceType::Sulfur => "Sulfur",
            ResourceType::Clay => "Clay",
            ResourceType::Diamonds => "Diamonds",
            ResourceType::Rubies => "Rubies",
            ResourceType::Emeralds => "Emeralds",
            ResourceType::Oil => "Oil",
            ResourceType::NaturalGas => "Natural Gas",
            ResourceType::UraniumOre => "Uranium Ore",
            ResourceType::ThoriumOre => "Thorium Ore",
            ResourceType::LithiumOre => "Lithium Ore",
            ResourceType::Timber => "Timber",
            ResourceType::Stone => "Stone",
            ResourceType::FreshWater => "Fresh Water",
            ResourceType::FertileSoil => "Fertile Soil",
            ResourceType::Game => "Game",
            ResourceType::Fish => "Fish",
            ResourceType::WhaleOil => "Whale Oil",
            ResourceType::Spices => "Spices",
            ResourceType::Cotton => "Cotton",
            ResourceType::Herbs => "Herbs",
            ResourceType::Tobacco => "Tobacco",
            ResourceType::Sugar => "Sugar",
            ResourceType::Grapes => "Grapes",
            ResourceType::Olives => "Olives",
            ResourceType::Furs => "Furs",
            ResourceType::Leather => "Leather",
            ResourceType::Cattle => "Cattle",
            ResourceType::Sheep => "Sheep",
            ResourceType::Horses => "Horses",
            ResourceType::Pigs => "Pigs",
            ResourceType::Pearls => "Pearls",
            ResourceType::Coral => "Coral",
            ResourceType::Amber => "Amber",
            ResourceType::SeaSalt => "Sea Salt",
            ResourceType::Feathers => "Feathers",
            ResourceType::RareHerbs => "Rare Herbs",
            ResourceType::MagicCrystals => "Magic Crystals",
            ResourceType::EnchantedWood => "Enchanted Wood",
            ResourceType::MysticalHerbs => "Mystical Herbs",
            ResourceType::VolcanicMinerals => "Volcanic Minerals",
            ResourceType::ExoticHide => "Exotic Hide",
            ResourceType::AlchemicalIngredients => "Alchemical Ingredients",
            ResourceType::RareEarth => "Rare Earth Elements",
            ResourceType::Mithril => "Mithril",
            ResourceType::Adamantine => "Adamantine",
            ResourceType::StarMetal => "Star Metal",
            ResourceType::SoulEssence => "Soul Essence",
            ResourceType::ManaCrystals => "Mana Crystals",
            ResourceType::EtherealOre => "Ethereal Ore",
            ResourceType::BioluminescentOrganisms => "Bioluminescent Organisms",
            ResourceType::RadiativeCrystals => "Radiative Crystals",
            ResourceType::VoidMatter => "Void Matter",
            ResourceType::Xenocompounds => "Xenobiological Compounds",
        }
    }

    /// Get the resource category this belongs to.
    pub fn category(&self) -> ResourceCategory {
        match self {
            // Mineral resources
            ResourceType::IronOre
            | ResourceType::CopperOre
            | ResourceType::TinOre
            | ResourceType::Coal
            | ResourceType::LeadOre => ResourceCategory::BaseMetals,
            ResourceType::GoldOre
            | ResourceType::SilverOre
            | ResourceType::PlatinumOre
            | ResourceType::Diamonds
            | ResourceType::Rubies
            | ResourceType::Emeralds
            | ResourceType::Gemstones => ResourceCategory::PreciousMetals,
            ResourceType::Limestone
            | ResourceType::Granite
            | ResourceType::Marble
            | ResourceType::Stone => ResourceCategory::Stone,
            ResourceType::Salt | ResourceType::Sulfur | ResourceType::Clay => {
                ResourceCategory::IndustrialMinerals
            }
            ResourceType::Oil | ResourceType::NaturalGas => ResourceCategory::FossilFuels,
            ResourceType::UraniumOre | ResourceType::ThoriumOre => ResourceCategory::Nuclear,
            ResourceType::LithiumOre | ResourceType::RareEarth => ResourceCategory::RareMetals,

            // Organic resources
            ResourceType::Timber => ResourceCategory::Timber,
            ResourceType::FreshWater => ResourceCategory::FreshWater,
            ResourceType::FertileSoil => ResourceCategory::Agriculture,
            ResourceType::Game | ResourceType::Furs | ResourceType::Feathers => {
                ResourceCategory::Hunting
            }
            ResourceType::Fish | ResourceType::WhaleOil | ResourceType::Pearls => {
                ResourceCategory::Fishing
            }
            ResourceType::Spices
            | ResourceType::Herbs
            | ResourceType::Tobacco
            | ResourceType::RareHerbs
            | ResourceType::MysticalHerbs => ResourceCategory::Botanical,
            ResourceType::Cotton | ResourceType::Leather => ResourceCategory::Fibers,
            ResourceType::Sugar | ResourceType::Grapes | ResourceType::Olives => {
                ResourceCategory::LuxuryCrops
            }
            ResourceType::Coral | ResourceType::Amber | ResourceType::SeaSalt => {
                ResourceCategory::MarineSpecialty
            }

            // Livestock
            ResourceType::Cattle
            | ResourceType::Sheep
            | ResourceType::Horses
            | ResourceType::Pigs => ResourceCategory::Livestock,

            // Fantasy/sci-fi
            ResourceType::MagicCrystals
            | ResourceType::ManaCrystals
            | ResourceType::SoulEssence => ResourceCategory::MagicalMaterials,
            ResourceType::VolcanicMinerals | ResourceType::StarMetal => {
                ResourceCategory::VolcanicMinerals
            }
            ResourceType::EnchantedWood | ResourceType::ExoticHide => {
                ResourceCategory::ExoticOrganics
            }
            ResourceType::AlchemicalIngredients => ResourceCategory::Alchemical,
            ResourceType::Mithril | ResourceType::Adamantine => ResourceCategory::LegendaryMetals,
            ResourceType::EtherealOre | ResourceType::VoidMatter => {
                ResourceCategory::PlanarResources
            }
            ResourceType::BioluminescentOrganisms => ResourceCategory::Bioluminescent,
            ResourceType::RadiativeCrystals => ResourceCategory::Radiative,
            ResourceType::Xenocompounds => ResourceCategory::Xenobiological,
        }
    }

    /// Get base economic value (relative scale).
    pub fn base_value(&self) -> f32 {
        match self {
            // Low value bulk resources
            ResourceType::Clay => 1.0,
            ResourceType::Stone => 1.0,
            ResourceType::Limestone => 1.0,
            ResourceType::Granite => 1.5,
            ResourceType::FreshWater => 1.0,
            ResourceType::FertileSoil => 2.0,
            ResourceType::Timber => 2.0,
            ResourceType::Coal => 3.0,
            ResourceType::Salt => 3.0,

            // Medium value metals
            ResourceType::IronOre => 5.0,
            ResourceType::CopperOre => 5.0,
            ResourceType::TinOre => 8.0,
            ResourceType::LeadOre => 4.0,
            ResourceType::SilverOre => 50.0,
            ResourceType::GoldOre => 100.0,
            ResourceType::PlatinumOre => 200.0,
            ResourceType::RareEarth => 150.0,
            ResourceType::LithiumOre => 100.0,

            // High value gems
            ResourceType::Gemstones => 80.0,
            ResourceType::Diamonds => 500.0,
            ResourceType::Rubies => 400.0,
            ResourceType::Emeralds => 400.0,
            ResourceType::Marble => 20.0,

            // Agricultural
            ResourceType::Fish => 5.0,
            ResourceType::Game => 8.0,
            ResourceType::Cattle => 15.0,
            ResourceType::Sheep => 12.0,
            ResourceType::Horses => 25.0,
            ResourceType::Pigs => 10.0,
            ResourceType::Cotton => 5.0,
            ResourceType::Leather => 8.0,
            ResourceType::Herbs => 10.0,
            ResourceType::Spices => 50.0,
            ResourceType::Tobacco => 20.0,
            ResourceType::Sugar => 15.0,
            ResourceType::Grapes => 18.0,
            ResourceType::Olives => 15.0,
            ResourceType::Furs => 30.0,
            ResourceType::Feathers => 5.0,

            // Fuels and energy
            ResourceType::Oil => 40.0,
            ResourceType::NaturalGas => 35.0,
            ResourceType::UraniumOre => 200.0,
            ResourceType::ThoriumOre => 80.0,
            ResourceType::WhaleOil => 15.0,
            ResourceType::SeaSalt => 8.0,

            // Specialty
            ResourceType::Pearls => 100.0,
            ResourceType::Coral => 50.0,
            ResourceType::Amber => 75.0,
            ResourceType::RareHerbs => 40.0,
            ResourceType::Sulfur => 5.0,

            // Fantasy/sci-fi (higher values)
            ResourceType::MagicCrystals => 500.0,
            ResourceType::ManaCrystals => 600.0,
            ResourceType::EnchantedWood => 200.0,
            ResourceType::MysticalHerbs => 150.0,
            ResourceType::VolcanicMinerals => 100.0,
            ResourceType::ExoticHide => 250.0,
            ResourceType::AlchemicalIngredients => 300.0,
            ResourceType::SoulEssence => 400.0,
            ResourceType::StarMetal => 800.0,
            ResourceType::Mithril => 2000.0,
            ResourceType::Adamantine => 2500.0,
            ResourceType::EtherealOre => 500.0,
            ResourceType::BioluminescentOrganisms => 350.0,
            ResourceType::RadiativeCrystals => 450.0,
            ResourceType::VoidMatter => 1500.0,
            ResourceType::Xenocompounds => 1000.0,
        }
    }

    /// Get spawn rarity modifier (0.0 = common, 1.0 = legendary).
    pub fn rarity(&self) -> f32 {
        match self {
            // Very common
            ResourceType::Clay => 0.1,
            ResourceType::Stone => 0.1,
            ResourceType::Limestone => 0.15,
            ResourceType::Granite => 0.15,
            ResourceType::FreshWater => 0.1,
            ResourceType::FertileSoil => 0.2,
            ResourceType::Timber => 0.15,
            ResourceType::Coal => 0.25,
            ResourceType::Salt => 0.2,
            ResourceType::IronOre => 0.2,
            ResourceType::CopperOre => 0.25,
            ResourceType::TinOre => 0.35,
            ResourceType::LeadOre => 0.3,

            // Common
            ResourceType::Fish => 0.2,
            ResourceType::Game => 0.25,
            ResourceType::Cattle => 0.25,
            ResourceType::Sheep => 0.25,
            ResourceType::Pigs => 0.25,
            ResourceType::Horses => 0.35,
            ResourceType::Cotton => 0.25,
            ResourceType::Leather => 0.25,
            ResourceType::Herbs => 0.3,
            ResourceType::WhaleOil => 0.4,
            ResourceType::SeaSalt => 0.35,

            // Uncommon
            ResourceType::SilverOre => 0.45,
            ResourceType::GoldOre => 0.55,
            ResourceType::Limestone => 0.35,
            ResourceType::Marble => 0.5,
            ResourceType::Spices => 0.5,
            ResourceType::Tobacco => 0.45,
            ResourceType::Sugar => 0.4,
            ResourceType::Grapes => 0.4,
            ResourceType::Olives => 0.4,
            ResourceType::Furs => 0.45,
            ResourceType::Sulfur => 0.45,
            ResourceType::Oil => 0.55,
            ResourceType::NaturalGas => 0.5,
            ResourceType::Gemstones => 0.55,
            ResourceType::Pearls => 0.6,
            ResourceType::Coral => 0.55,
            ResourceType::Amber => 0.5,
            ResourceType::RareHerbs => 0.55,
            ResourceType::Feathers => 0.4,

            // Rare
            ResourceType::PlatinumOre => 0.7,
            ResourceType::Diamonds => 0.85,
            ResourceType::Rubies => 0.8,
            ResourceType::Emeralds => 0.8,
            ResourceType::UraniumOre => 0.75,
            ResourceType::ThoriumOre => 0.7,
            ResourceType::LithiumOre => 0.7,
            ResourceType::RareEarth => 0.75,

            // Very rare / legendary
            ResourceType::MagicCrystals => 0.9,
            ResourceType::ManaCrystals => 0.95,
            ResourceType::EnchantedWood => 0.85,
            ResourceType::MysticalHerbs => 0.85,
            ResourceType::VolcanicMinerals => 0.75,
            ResourceType::ExoticHide => 0.9,
            ResourceType::AlchemicalIngredients => 0.88,
            ResourceType::SoulEssence => 0.92,
            ResourceType::StarMetal => 0.95,
            ResourceType::Mithril => 0.98,
            ResourceType::Adamantine => 0.99,
            ResourceType::EtherealOre => 0.93,
            ResourceType::BioluminescentOrganisms => 0.85,
            ResourceType::RadiativeCrystals => 0.92,
            ResourceType::VoidMatter => 0.98,
            ResourceType::Xenocompounds => 0.97,
        }
    }

    /// Check if this is a fantasy/sci-fi resource.
    pub fn is_fantasy(&self) -> bool {
        self.category().is_fantasy()
    }

    /// Check if this requires coastal or water proximity.
    pub fn is_aquatic(&self) -> bool {
        matches!(
            self,
            ResourceType::Fish
                | ResourceType::WhaleOil
                | ResourceType::Pearls
                | ResourceType::SeaSalt
                | ResourceType::BioluminescentOrganisms
        )
    }

    /// Check if this is a mineral/ore resource.
    pub fn is_mineral(&self) -> bool {
        matches!(
            self.category(),
            ResourceCategory::BaseMetals
                | ResourceCategory::PreciousMetals
                | ResourceCategory::IndustrialMinerals
                | ResourceCategory::Nuclear
                | ResourceCategory::RareMetals
                | ResourceCategory::VolcanicMinerals
                | ResourceCategory::LegendaryMetals
                | ResourceCategory::Radiative
        )
    }
}

/// Resource categories for grouping and logic.
///
/// Each category defines a type of natural resource with similar
/// generation patterns and value characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ResourceCategory {
    /// Base metals (iron, copper, tin, lead)
    BaseMetals = 100,
    /// Precious metals and gems (gold, silver, gems)
    PreciousMetals,
    /// Stone types (granite, marble)
    Stone,
    /// Industrial minerals (salt, sulfur, clay)
    IndustrialMinerals,
    /// Fossil fuels (oil, natural gas)
    FossilFuels,
    /// Nuclear materials (uranium, thorium)
    Nuclear,
    /// Rare earth elements
    RareMetals,

    /// Timber from forests
    Timber,
    /// Fresh water sources
    FreshWater,
    /// Agricultural land
    Agriculture,
    /// Hunting resources (game, furs)
    Hunting,
    /// Marine resources (fish, pearls)
    Fishing,
    /// Botanical resources (spices, herbs)
    Botanical,
    /// Fiber materials (cotton, leather)
    Fibers,
    /// Luxury crops (grapes, olives, sugar)
    LuxuryCrops,
    /// Marine specialty (coral, amber)
    MarineSpecialty,

    /// Livestock grazing (cattle, sheep, horses)
    Livestock,

    /// Magical materials
    MagicalMaterials,
    /// Volcanic minerals
    VolcanicMinerals,
    /// Exotic organic materials
    ExoticOrganics,
    /// Alchemical ingredients
    Alchemical,
    /// Legendary metals (mithril, adamantine)
    LegendaryMetals,
    /// Planar resources (ethereal, void)
    PlanarResources,
    /// Bioluminescent organisms
    Bioluminescent,
    /// Radioactive crystals
    Radiative,
    /// Alien biological compounds
    Xenobiological,
}

impl ResourceCategory {
    /// Get display name for this category.
    pub fn name(&self) -> &'static str {
        match self {
            ResourceCategory::BaseMetals => "Base Metals",
            ResourceCategory::PreciousMetals => "Precious Metals & Gems",
            ResourceCategory::Stone => "Stone",
            ResourceCategory::IndustrialMinerals => "Industrial Minerals",
            ResourceCategory::FossilFuels => "Fossil Fuels",
            ResourceCategory::Nuclear => "Nuclear Materials",
            ResourceCategory::RareMetals => "Rare Metals",
            ResourceCategory::Timber => "Timber",
            ResourceCategory::FreshWater => "Fresh Water",
            ResourceCategory::Agriculture => "Agriculture",
            ResourceCategory::Hunting => "Hunting",
            ResourceCategory::Fishing => "Fishing",
            ResourceCategory::Botanical => "Botanical",
            ResourceCategory::Fibers => "Fibers",
            ResourceCategory::LuxuryCrops => "Luxury Crops",
            ResourceCategory::MarineSpecialty => "Marine Specialty",
            ResourceCategory::Livestock => "Livestock",
            ResourceCategory::MagicalMaterials => "Magical Materials",
            ResourceCategory::VolcanicMinerals => "Volcanic Minerals",
            ResourceCategory::ExoticOrganics => "Exotic Organics",
            ResourceCategory::Alchemical => "Alchemical",
            ResourceCategory::LegendaryMetals => "Legendary Metals",
            ResourceCategory::PlanarResources => "Planar Resources",
            ResourceCategory::Bioluminescent => "Bioluminescent",
            ResourceCategory::Radiative => "Radiative",
            ResourceCategory::Xenobiological => "Xenobiological",
        }
    }

    /// Check if this is a fantasy/sci-fi category.
    pub fn is_fantasy(&self) -> bool {
        matches!(
            self,
            ResourceCategory::MagicalMaterials
                | ResourceCategory::VolcanicMinerals
                | ResourceCategory::ExoticOrganics
                | ResourceCategory::Alchemical
                | ResourceCategory::LegendaryMetals
                | ResourceCategory::PlanarResources
                | ResourceCategory::Bioluminescent
                | ResourceCategory::Radiative
                | ResourceCategory::Xenobiological
        )
    }
}

/// Resource richness level for a region.
///
/// Determines how abundant a resource is in a given location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResourceRichness {
    /// No resources of this type (never spawns)
    None = 0,
    /// Sparse deposits
    Sparse = 1,
    /// Normal deposits
    Normal = 2,
    /// Rich deposits
    Rich = 3,
    /// Abundant deposits
    Abundant = 4,
    /// Legendary deposits
    Legendary = 5,
}

impl ResourceRichness {
    /// Get the spawn probability multiplier for this richness.
    pub fn probability_multiplier(&self) -> f32 {
        match self {
            ResourceRichness::None => 0.0,
            ResourceRichness::Sparse => 0.2,
            ResourceRichness::Normal => 0.5,
            ResourceRichness::Rich => 0.8,
            ResourceRichness::Abundant => 1.0,
            ResourceRichness::Legendary => 1.5,
        }
    }

    /// Get the yield modifier for extraction.
    pub fn yield_modifier(&self) -> f32 {
        match self {
            ResourceRichness::None => 0.0,
            ResourceRichness::Sparse => 0.3,
            ResourceRichness::Normal => 0.6,
            ResourceRichness::Rich => 1.0,
            ResourceRichness::Abundant => 1.5,
            ResourceRichness::Legendary => 2.5,
        }
    }

    /// Get display label for this richness.
    pub fn label(&self) -> &'static str {
        match self {
            ResourceRichness::None => "None",
            ResourceRichness::Sparse => "Sparse",
            ResourceRichness::Normal => "Normal",
            ResourceRichness::Rich => "Rich",
            ResourceRichness::Abundant => "Abundant",
            ResourceRichness::Legendary => "Legendary",
        }
    }

    /// Convert to f32 for statistical calculations.
    pub fn as_f32(&self) -> f32 {
        *self as u8 as f32
    }

    /// Convert from numeric value (1-5).
    pub fn from_level(level: u8) -> Self {
        match level.min(5) {
            0 => ResourceRichness::None,
            1 => ResourceRichness::Sparse,
            2 => ResourceRichness::Normal,
            3 => ResourceRichness::Rich,
            4 => ResourceRichness::Abundant,
            _ => ResourceRichness::Legendary,
        }
    }
}

/// Individual resource deposit in a region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeposit {
    /// Type of resource
    pub resource_type: ResourceType,
    /// Richness level
    pub richness: ResourceRichness,
    /// Estimated total value
    pub estimated_value: f32,
    /// Extraction difficulty (1-10)
    pub extraction_difficulty: u8,
    /// Is this a renewable resource?
    pub is_renewable: bool,
}

impl ResourceDeposit {
    /// Create a new resource deposit.
    pub fn new(resource_type: ResourceType, richness: ResourceRichness) -> Self {
        let base_value = resource_type.base_value();
        let rarity = resource_type.rarity();
        let probability = richness.probability_multiplier();
        let yield_mod = richness.yield_modifier();

        // Calculate estimated value
        let estimated_value = base_value * probability * yield_mod * (1.0 - rarity * 0.3);

        // Calculate extraction difficulty based on rarity
        let extraction_difficulty = ((rarity * 10.0) as u8).min(10);

        // Determine renewability
        let is_renewable = matches!(
            resource_type.category(),
            ResourceCategory::Agriculture
                | ResourceCategory::Livestock
                | ResourceCategory::Botanical
                | ResourceCategory::Timber
        );

        Self {
            resource_type,
            richness,
            estimated_value,
            extraction_difficulty,
            is_renewable,
        }
    }
}

/// Set of resources found in a region.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSet {
    /// All deposits in this set
    pub deposits: Vec<ResourceDeposit>,
    /// Total estimated value
    pub total_value: f32,
    /// Primary resource (highest value)
    pub primary_resource: Option<ResourceType>,
}

impl ResourceSet {
    /// Create an empty resource set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a deposit to this set.
    pub fn add(&mut self, deposit: ResourceDeposit) {
        self.total_value += deposit.estimated_value;

        // Update primary resource if this is higher value
        if self.primary_resource.is_none()
            || deposit.estimated_value
                > self
                    .deposits
                    .iter()
                    .find(|d| Some(d.resource_type) == self.primary_resource)
                    .map(|d| d.estimated_value)
                    .unwrap_or(0.0)
        {
            self.primary_resource = Some(deposit.resource_type);
        }

        self.deposits.push(deposit);
    }

    /// Get all deposits of a specific type.
    pub fn get_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceDeposit> {
        self.deposits
            .iter()
            .filter(|d| d.resource_type == resource_type)
            .collect()
    }

    /// Get all deposits in a category.
    pub fn get_by_category(&self, category: ResourceCategory) -> Vec<&ResourceDeposit> {
        self.deposits
            .iter()
            .filter(|d| d.resource_type.category() == category)
            .collect()
    }

    /// Check if a resource type exists in this set.
    pub fn has(&self, resource_type: ResourceType) -> bool {
        self.deposits
            .iter()
            .any(|d| d.resource_type == resource_type)
    }

    /// Get total value for a category.
    pub fn category_value(&self, category: ResourceCategory) -> f32 {
        self.get_by_category(category)
            .iter()
            .map(|d| d.estimated_value)
            .sum()
    }
}

/// Configuration for resource generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGenConfig {
    /// Enable fantasy/sci-fi resources
    pub enable_fantasy_resources: bool,
    /// Enable legendary resources
    pub enable_legendary: bool,
    /// Base resource density per region
    pub base_density: f32,
    /// Maximum resources per region
    pub max_resources_per_region: usize,
}

impl Default for ResourceGenConfig {
    fn default() -> Self {
        Self {
            enable_fantasy_resources: true,
            enable_legendary: true,
            base_density: 0.5,
            max_resources_per_region: 10,
        }
    }
}

/// Resource generation state and helper methods.
#[derive(Debug, Clone)]
pub struct ResourceGenerator {
    config: ResourceGenConfig,
    /// Cache of resources by category for quick lookup
    resources_by_category: HashMap<ResourceCategory, Vec<ResourceType>>,
}

impl ResourceGenerator {
    /// Create a new resource generator with default config.
    pub fn new() -> Self {
        Self::with_config(ResourceGenConfig::default())
    }

    /// Create a new resource generator with custom config.
    pub fn with_config(config: ResourceGenConfig) -> Self {
        let mut resources_by_category: HashMap<ResourceCategory, Vec<ResourceType>> =
            HashMap::new();

        // Build category lookup table
        for rt in ALL_RESOURCE_TYPES.iter() {
            let category = rt.category();
            resources_by_category.entry(category).or_default().push(*rt);
        }

        Self {
            config,
            resources_by_category,
        }
    }

    /// Get all resource types for a given category.
    pub fn resources_in_category(&self, category: ResourceCategory) -> Vec<ResourceType> {
        self.resources_by_category
            .get(&category)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all base (non-fantasy) resource types.
    pub fn base_resources(&self) -> Vec<ResourceType> {
        self.resources_by_category
            .values()
            .flatten()
            .filter(|rt| !rt.is_fantasy())
            .copied()
            .collect()
    }

    /// Get all fantasy resource types.
    pub fn fantasy_resources(&self) -> Vec<ResourceType> {
        self.resources_by_category
            .values()
            .flatten()
            .filter(|rt| rt.is_fantasy())
            .copied()
            .collect()
    }

    /// Get resources viable in a vegetation type.
    pub fn viable_resources(&self, vegetation: &super::biome::VegetationType) -> Vec<ResourceType> {
        self.resources_by_category
            .values()
            .flatten()
            .filter(|rt| ViabilityMatrix::is_viable(rt, vegetation))
            .copied()
            .collect()
    }

    /// Check if resource generation should include fantasy types.
    pub fn include_fantasy(&self) -> bool {
        self.config.enable_fantasy_resources
    }

    /// Check if legendary resources can spawn.
    pub fn allow_legendary(&self) -> bool {
        self.config.enable_legendary
    }
}

/// Static viability matrix - which resources can spawn in which vegetation types.
///
/// Format: (VegetationType, [ResourceCategories that are viable])
/// This is a simplified matrix; detailed biome-resource compatibility
/// is handled by the BiomeAssignmentMatrix.
pub struct ViabilityMatrix;

/// Static instance of the viability matrix.
static VIABILITY_MATRIX: ViabilityMatrix = ViabilityMatrix;

impl ViabilityMatrix {
    /// Check if a resource type is viable in a vegetation type.
    pub fn is_viable(resource: &ResourceType, vegetation: &super::biome::VegetationType) -> bool {
        match (resource.category(), vegetation) {
            // Dense forests: timber, game, some minerals
            (ResourceCategory::Timber, super::biome::VegetationType::DenseForest) => true,
            (ResourceCategory::Hunting, super::biome::VegetationType::DenseForest) => true,
            (ResourceCategory::BaseMetals, super::biome::VegetationType::DenseForest) => true,

            // Forest types: timber, game, some herbs
            (ResourceCategory::Timber, super::biome::VegetationType::Forest) => true,
            (ResourceCategory::Hunting, super::biome::VegetationType::Forest) => true,
            (ResourceCategory::Botanical, super::biome::VegetationType::Forest) => true,

            // Coniferous: timber (better quality), game
            (ResourceCategory::Timber, super::biome::VegetationType::ConiferousForest) => true,
            (ResourceCategory::Hunting, super::biome::VegetationType::ConiferousForest) => true,

            // Open forest: mixed resources
            (ResourceCategory::Timber, super::biome::VegetationType::OpenForest) => true,
            (ResourceCategory::Hunting, super::biome::VegetationType::OpenForest) => true,
            (ResourceCategory::Botanical, super::biome::VegetationType::OpenForest) => true,

            // Grasslands: livestock, grains, some game
            (ResourceCategory::Livestock, super::biome::VegetationType::Grassland) => true,
            (ResourceCategory::Agriculture, super::biome::VegetationType::Grassland) => true,
            (ResourceCategory::Hunting, super::biome::VegetationType::Grassland) => true,

            // Tundra: limited resources
            (ResourceCategory::Hunting, super::biome::VegetationType::Tundra) => true,
            (ResourceCategory::FreshWater, super::biome::VegetationType::Tundra) => true,

            // Desert: minerals, gems, some stone
            (ResourceCategory::BaseMetals, super::biome::VegetationType::Desert) => true,
            (ResourceCategory::PreciousMetals, super::biome::VegetationType::Desert) => true,
            (ResourceCategory::IndustrialMinerals, super::biome::VegetationType::Desert) => true,
            (ResourceCategory::Stone, super::biome::VegetationType::Desert) => true,

            // Wetland: fish, fresh water, reeds
            (ResourceCategory::Fishing, super::biome::VegetationType::Wetland) => true,
            (ResourceCategory::FreshWater, super::biome::VegetationType::Wetland) => true,
            (ResourceCategory::Fibers, super::biome::VegetationType::Wetland) => true,

            // Marine: fishing, pearls, coral
            (ResourceCategory::Fishing, super::biome::VegetationType::MarineVegetation) => true,
            (ResourceCategory::MarineSpecialty, super::biome::VegetationType::MarineVegetation) => {
                true
            }

            // Open water: fishing, whale oil
            (ResourceCategory::Fishing, super::biome::VegetationType::OpenWater) => true,

            // Magical: magical materials
            (
                ResourceCategory::MagicalMaterials,
                super::biome::VegetationType::MagicalVegetation,
            ) => true,
            (ResourceCategory::ExoticOrganics, super::biome::VegetationType::MagicalVegetation) => {
                true
            }
            (ResourceCategory::Alchemical, super::biome::VegetationType::MagicalVegetation) => true,

            // Crystal: minerals, gems
            (ResourceCategory::PreciousMetals, super::biome::VegetationType::CrystalVegetation) => {
                true
            }
            (
                ResourceCategory::IndustrialMinerals,
                super::biome::VegetationType::CrystalVegetation,
            ) => true,
            (ResourceCategory::Radiative, super::biome::VegetationType::CrystalVegetation) => true,

            // Volcanic: volcanic minerals, sulfur
            (
                ResourceCategory::VolcanicMinerals,
                super::biome::VegetationType::VolcanicVegetation,
            ) => true,
            (
                ResourceCategory::IndustrialMinerals,
                super::biome::VegetationType::VolcanicVegetation,
            ) => true,
            (
                ResourceCategory::LegendaryMetals,
                super::biome::VegetationType::VolcanicVegetation,
            ) => true,

            // Toxic: rare chemicals, alchemical
            (ResourceCategory::Alchemical, super::biome::VegetationType::ToxicVegetation) => true,
            (
                ResourceCategory::IndustrialMinerals,
                super::biome::VegetationType::ToxicVegetation,
            ) => true,

            // Sky: exotic organics
            (ResourceCategory::ExoticOrganics, super::biome::VegetationType::SkyVegetation) => true,
            (ResourceCategory::MagicalMaterials, super::biome::VegetationType::SkyVegetation) => {
                true
            }

            // Snow/ice: limited (maybe ice)
            (ResourceCategory::FreshWater, super::biome::VegetationType::SnowIce) => true,

            _ => false,
        }
    }
}

/// All resource types as a slice for iteration.
pub const ALL_RESOURCE_TYPES: [ResourceType; 66] = [
    // Mineral resources (18)
    ResourceType::IronOre,
    ResourceType::CopperOre,
    ResourceType::TinOre,
    ResourceType::Coal,
    ResourceType::GoldOre,
    ResourceType::SilverOre,
    ResourceType::LeadOre,
    ResourceType::PlatinumOre,
    ResourceType::Gemstones,
    ResourceType::Limestone,
    ResourceType::Granite,
    ResourceType::Marble,
    ResourceType::Salt,
    ResourceType::Sulfur,
    ResourceType::Clay,
    ResourceType::Diamonds,
    ResourceType::Rubies,
    ResourceType::Emeralds,
    // Fuel and energy (5)
    ResourceType::Oil,
    ResourceType::NaturalGas,
    ResourceType::UraniumOre,
    ResourceType::ThoriumOre,
    ResourceType::LithiumOre,
    // Organic and agricultural (16)
    ResourceType::Timber,
    ResourceType::Stone,
    ResourceType::FreshWater,
    ResourceType::FertileSoil,
    ResourceType::Game,
    ResourceType::Fish,
    ResourceType::WhaleOil,
    ResourceType::Spices,
    ResourceType::Cotton,
    ResourceType::Herbs,
    ResourceType::Tobacco,
    ResourceType::Sugar,
    ResourceType::Grapes,
    ResourceType::Olives,
    ResourceType::Furs,
    ResourceType::Leather,
    // Livestock (4)
    ResourceType::Cattle,
    ResourceType::Sheep,
    ResourceType::Horses,
    ResourceType::Pigs,
    // Special (6)
    ResourceType::Pearls,
    ResourceType::Coral,
    ResourceType::Amber,
    ResourceType::SeaSalt,
    ResourceType::Feathers,
    ResourceType::RareHerbs,
    // Fantasy and sci-fi (17)
    ResourceType::MagicCrystals,
    ResourceType::EnchantedWood,
    ResourceType::MysticalHerbs,
    ResourceType::VolcanicMinerals,
    ResourceType::ExoticHide,
    ResourceType::AlchemicalIngredients,
    ResourceType::RareEarth,
    ResourceType::Mithril,
    ResourceType::Adamantine,
    ResourceType::StarMetal,
    ResourceType::SoulEssence,
    ResourceType::ManaCrystals,
    ResourceType::EtherealOre,
    ResourceType::BioluminescentOrganisms,
    ResourceType::RadiativeCrystals,
    ResourceType::VoidMatter,
    ResourceType::Xenocompounds,
];

/// All resource categories as a slice.
pub const ALL_RESOURCE_CATEGORIES: [ResourceCategory; 26] = [
    ResourceCategory::BaseMetals,
    ResourceCategory::PreciousMetals,
    ResourceCategory::Stone,
    ResourceCategory::IndustrialMinerals,
    ResourceCategory::FossilFuels,
    ResourceCategory::Nuclear,
    ResourceCategory::RareMetals,
    ResourceCategory::Timber,
    ResourceCategory::FreshWater,
    ResourceCategory::Agriculture,
    ResourceCategory::Hunting,
    ResourceCategory::Fishing,
    ResourceCategory::Botanical,
    ResourceCategory::Fibers,
    ResourceCategory::LuxuryCrops,
    ResourceCategory::MarineSpecialty,
    ResourceCategory::Livestock,
    ResourceCategory::MagicalMaterials,
    ResourceCategory::VolcanicMinerals,
    ResourceCategory::ExoticOrganics,
    ResourceCategory::Alchemical,
    ResourceCategory::LegendaryMetals,
    ResourceCategory::PlanarResources,
    ResourceCategory::Bioluminescent,
    ResourceCategory::Radiative,
    ResourceCategory::Xenobiological,
];

/// Convenience reference for resource-vegetation compatibility lookup.
pub static RESOURCE_CATEGORIES: ViabilityMatrix = ViabilityMatrix;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::VegetationType;

    #[test]
    fn test_resource_type_names() {
        assert_eq!(ResourceType::IronOre.name(), "Iron Ore");
        assert_eq!(ResourceType::GoldOre.name(), "Gold Ore");
        assert_eq!(ResourceType::Mithril.name(), "Mithril");
    }

    #[test]
    fn test_resource_category() {
        assert_eq!(
            ResourceType::IronOre.category(),
            ResourceCategory::BaseMetals
        );
        assert_eq!(
            ResourceType::GoldOre.category(),
            ResourceCategory::PreciousMetals
        );
        assert_eq!(ResourceType::Timber.category(), ResourceCategory::Timber);
    }

    #[test]
    fn test_resource_values() {
        // Iron should be less valuable than gold
        assert!(ResourceType::IronOre.base_value() < ResourceType::GoldOre.base_value());

        // Clay should be less valuable than gems
        assert!(ResourceType::Clay.base_value() < ResourceType::Diamonds.base_value());
    }

    #[test]
    fn test_rarity() {
        // Common resources should have lower rarity
        assert!(ResourceType::Clay.rarity() < ResourceType::GoldOre.rarity());

        // Legendary resources should have very high rarity
        assert!(ResourceType::Mithril.rarity() > 0.9);
    }

    #[test]
    fn test_fantasy_detection() {
        // Earth-like resources
        assert!(!ResourceType::IronOre.is_fantasy());
        assert!(!ResourceType::Fish.is_fantasy());
        assert!(!ResourceType::Timber.is_fantasy());

        // Fantasy resources
        assert!(ResourceType::MagicCrystals.is_fantasy());
        assert!(ResourceType::Mithril.is_fantasy());
        assert!(ResourceType::VoidMatter.is_fantasy());
    }

    #[test]
    fn test_aquatic_detection() {
        assert!(ResourceType::Fish.is_aquatic());
        assert!(ResourceType::Pearls.is_aquatic());
        assert!(!ResourceType::IronOre.is_aquatic());
    }

    #[test]
    fn test_mineral_detection() {
        assert!(ResourceType::IronOre.is_mineral());
        assert!(ResourceType::GoldOre.is_mineral());
        assert!(ResourceType::Diamonds.is_mineral());
        assert!(!ResourceType::Fish.is_mineral());
    }

    #[test]
    fn test_richness_probability() {
        assert_eq!(ResourceRichness::None.probability_multiplier(), 0.0);
        assert!(
            ResourceRichness::Legendary.probability_multiplier()
                > ResourceRichness::Normal.probability_multiplier()
        );
    }

    #[test]
    fn test_richness_yield() {
        assert_eq!(ResourceRichness::None.yield_modifier(), 0.0);
        assert!(
            ResourceRichness::Legendary.yield_modifier()
                > ResourceRichness::Normal.yield_modifier()
        );
    }

    #[test]
    fn test_resource_deposit_creation() {
        let deposit = ResourceDeposit::new(ResourceType::IronOre, ResourceRichness::Rich);

        assert_eq!(deposit.resource_type, ResourceType::IronOre);
        assert_eq!(deposit.richness, ResourceRichness::Rich);
        assert!(deposit.estimated_value > 0.0);
    }

    #[test]
    fn test_resource_set_operations() {
        let mut set = ResourceSet::new();

        set.add(ResourceDeposit::new(
            ResourceType::IronOre,
            ResourceRichness::Normal,
        ));
        set.add(ResourceDeposit::new(
            ResourceType::GoldOre,
            ResourceRichness::Sparse,
        ));
        set.add(ResourceDeposit::new(
            ResourceType::Timber,
            ResourceRichness::Rich,
        ));

        assert_eq!(set.deposits.len(), 3);
        assert!(set.has(ResourceType::IronOre));
        assert!(!set.has(ResourceType::Diamonds));

        // Primary resource should be highest value
        assert_eq!(set.primary_resource, Some(ResourceType::GoldOre));
    }

    #[test]
    fn test_resource_generator() {
        let gen = ResourceGenerator::new();

        let forests = gen.viable_resources(&VegetationType::DenseForest);
        assert!(forests.contains(&ResourceType::Timber));
        assert!(forests.contains(&ResourceType::Game));
    }

    #[test]
    fn test_viability_matrix() {
        // Forests should support timber
        assert!(ViabilityMatrix::is_viable(
            &ResourceType::Timber,
            &VegetationType::DenseForest
        ));

        // Deserts should support minerals
        assert!(ViabilityMatrix::is_viable(
            &ResourceType::IronOre,
            &VegetationType::Desert
        ));

        // Forests should NOT support fish
        assert!(!ViabilityMatrix::is_viable(
            &ResourceType::Fish,
            &VegetationType::DenseForest
        ));
    }

    #[test]
    fn test_all_resource_types_count() {
        // Should have 66 resource types defined
        assert_eq!(ALL_RESOURCE_TYPES.len(), 66);
    }

    #[test]
    fn test_all_categories_count() {
        // Should have 26 categories
        assert_eq!(ALL_RESOURCE_CATEGORIES.len(), 26);
    }
}
