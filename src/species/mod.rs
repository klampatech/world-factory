//! Species Definition Module
//!
//! Provides species data for settlement generation and civilization simulation.
//!
//! ## Overview
//!
//! This module defines species that can inhabit settlements across different biomes.
//! Each species has:
//! - Home biomes (where they naturally thrive)
//! - Tolerable biomes (where they can survive with adaptation)
//! - Climate preferences and tolerances
//! - Traits that affect settlement behavior
//!
//! ## Default Species
//!
//! - Human: Versatile, found in most temperate regions
//! - Elf: Forest-dwelling, prefer temperate to tropical climates
//! - Dwarf: Subterranean/forested mountains, boreal to temperate
//! - Orc: Hardy, adaptable to harsh conditions
//! - Halfling: Peaceful agricultural species, temperate grasslands

use crate::terrain::biome::BiomeType;
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod loader;

/// Unique identifier for a species.
///
/// Represents the five playable species plus an UNDEFINED fallback for
/// species-agnostic systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", from = "u32", into = "u32")]
pub enum SpeciesId {
    /// Undefined/placeholder species for compatibility.
    Undefined = 0,
    /// Human — versatile, adaptable, trade-focused.
    Human = 1,
    /// Elf — forest-dwelling, nocturnal, pack hunters.
    Elf = 2,
    /// Dwarf — mountain/subterranean, sedentary.
    Dwarf = 3,
    /// Orc — hardy, warlike, nomadic.
    Orc = 4,
    /// Halfling — peaceful, sedentary, agricultural.
    Halfling = 5,
}

impl SpeciesId {
    /// Create a SpeciesId from a u32 value.
    /// Maps unknown values to UNDEFINED for safety.
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => SpeciesId::Undefined,
            1 => SpeciesId::Human,
            2 => SpeciesId::Elf,
            3 => SpeciesId::Dwarf,
            4 => SpeciesId::Orc,
            5 => SpeciesId::Halfling,
            _ => SpeciesId::Undefined,
        }
    }

    /// Get the inner u32 value.
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    /// Check if this is the UNDEFINED placeholder.
    pub fn is_undefined(&self) -> bool {
        matches!(self, SpeciesId::Undefined)
    }

    /// Get all defined species IDs.
    pub fn all() -> [SpeciesId; 5] {
        [
            SpeciesId::Human,
            SpeciesId::Elf,
            SpeciesId::Dwarf,
            SpeciesId::Orc,
            SpeciesId::Halfling,
        ]
    }

    /// Get display name for this species.
    pub fn display_name(&self) -> &'static str {
        match self {
            SpeciesId::Undefined => "Undefined",
            SpeciesId::Human => "Human",
            SpeciesId::Elf => "Elf",
            SpeciesId::Dwarf => "Dwarf",
            SpeciesId::Orc => "Orc",
            SpeciesId::Halfling => "Halfling",
        }
    }
}

impl std::fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<u32> for SpeciesId {
    fn from(val: u32) -> Self {
        SpeciesId::from_u32(val)
    }
}

impl From<SpeciesId> for u32 {
    fn from(id: SpeciesId) -> Self {
        id.as_u32()
    }
}

impl Default for SpeciesId {
    fn default() -> Self {
        SpeciesId::Undefined
    }
}

/// Species trait that affects behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeciesTrait {
    Aquatic,
    Flying,
    Subterranean,
    Nocturnal,
    PackHunter,
    Nomadic,
    Sedentary,
    TradeFocused,
    WarLike,
    Peaceful,
    /// Species adapts quickly to new environments and biomes.
    Adaptable,
    /// Species has natural curiosity, enabling faster discovery and innovation.
    Curious,
}

/// Climate tolerance ranges for a species.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateTolerance {
    pub min_temp: f32, // Celsius
    pub max_temp: f32,
    pub min_precipitation: f32, // mm/year
    pub max_precipitation: f32,
}

/// A defined species with all its characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: SpeciesId,
    pub name: String,
    pub display_name: String,
    pub home_biomes: Vec<BiomeType>,
    pub tolerable_biomes: Vec<BiomeType>,
    pub climate_tolerance: ClimateTolerance,
    pub traits: Vec<SpeciesTrait>,
    pub name_prefixes: Vec<String>,
    pub name_suffixes: Vec<String>,
}

impl Species {
    /// Check if this species naturally inhabits a biome.
    pub fn inhabits(&self, biome: BiomeType) -> bool {
        self.home_biomes.contains(&biome)
    }

    /// Check if this species can tolerate a biome.
    pub fn tolerates(&self, biome: BiomeType) -> bool {
        self.tolerable_biomes.contains(&biome)
    }

    /// Calculate suitability score for a biome (0.0-1.0).
    pub fn biome_suitability(&self, biome: BiomeType) -> f32 {
        if self.home_biomes.contains(&biome) {
            1.0
        } else if self.tolerable_biomes.contains(&biome) {
            0.5
        } else {
            0.0
        }
    }

    /// Check if species has a specific trait.
    pub fn has_trait(&self, trait_: SpeciesTrait) -> bool {
        self.traits.contains(&trait_)
    }

    /// Calculate biome suitability modifier based on trait effects.
    /// Adaptable trait increases tolerable biome score from 0.5 to 0.75.
    pub fn trait_biome_modifier(&self, biome: BiomeType) -> f32 {
        if self.traits.contains(&SpeciesTrait::Adaptable) {
            // Adaptable species can better tolerate non-home biomes
            if self.tolerates(biome) && !self.inhabits(biome) {
                return 0.25; // +0.25 bonus to tolerable biomes
            }
        }
        0.0
    }

    /// Calculate settlement growth rate modifier based on traits.
    /// Curious trait accelerates discovery and innovation.
    pub fn trait_growth_modifier(&self) -> f32 {
        let mut modifier = 1.0;
        if self.traits.contains(&SpeciesTrait::Curious) {
            modifier += 0.25; // +25% innovation rate
        }
        if self.traits.contains(&SpeciesTrait::Adaptable) {
            modifier += 0.10; // +10% adaptation speed
        }
        modifier
    }
}

/// Name template for culturally-appropriate settlement naming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameTemplate {
    pub species_id: SpeciesId,
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
    #[serde(default)]
    pub compound_patterns: Vec<String>,
}

/// Society type representing organizational complexity and governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocietyType {
    /// Small, family-based groups with collective decision-making.
    /// Population: 50-500. Simple tool use, basic language.
    Tribe,
    /// Larger groups with emerging leadership structures.
    /// Population: 500-5000. Specialized roles, trade networks, simple laws.
    Chiefdom,
    /// Complex societies with formalized governance and institutions.
    /// Population: 5000+. Written language, armies, urban centers.
    Nation,
}

impl SocietyType {
    /// Get population range for this society type.
    pub fn population_range(&self) -> (u32, u32) {
        match self {
            SocietyType::Tribe => (50, 500),
            SocietyType::Chiefdom => (500, 5000),
            SocietyType::Nation => (5000, u32::MAX),
        }
    }

    /// Get the default society type.
    pub fn default_type() -> Self {
        SocietyType::Tribe
    }
}

/// Collection of all species data and utilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesData {
    pub species: Vec<Species>,
    pub name_templates: HashMap<SpeciesId, NameTemplate>,
}

impl SpeciesData {
    /// Create species data with default species.
    pub fn default_species() -> Self {
        let species = vec![
            // Human - versatile, found everywhere
            Species {
                id: SpeciesId::Human,
                name: "Human".to_string(),
                display_name: "Human".to_string(),
                home_biomes: vec![
                    BiomeType::TemperateGrassland,
                    BiomeType::TemperateDeciduousForest,
                    BiomeType::TemperateMixedForest,
                    BiomeType::TemperateSteppe,
                ],
                tolerable_biomes: vec![
                    BiomeType::BorealForest,
                    BiomeType::SubtropicalSeasonalForest,
                    BiomeType::CoastalWetland,
                    BiomeType::TemperateRainforest,
                    BiomeType::MontaneForest,
                ],
                climate_tolerance: ClimateTolerance {
                    min_temp: -20.0,
                    max_temp: 40.0,
                    min_precipitation: 200.0,
                    max_precipitation: 3000.0,
                },
                traits: vec![
                    SpeciesTrait::Adaptable,
                    SpeciesTrait::Curious,
                    SpeciesTrait::Sedentary,
                    SpeciesTrait::TradeFocused,
                ],
                name_prefixes: vec![
                    "New".to_string(),
                    "Old".to_string(),
                    "High".to_string(),
                    "Low".to_string(),
                    "East".to_string(),
                    "West".to_string(),
                    "North".to_string(),
                    "South".to_string(),
                ],
                name_suffixes: vec![
                    "ton".to_string(),
                    "ham".to_string(),
                    "bury".to_string(),
                    "ford".to_string(),
                    "haven".to_string(),
                    "stead".to_string(),
                    "worth".to_string(),
                    "dale".to_string(),
                ],
            },
            // Elf - forest dwelling
            Species {
                id: SpeciesId::Elf,
                name: "Elf".to_string(),
                display_name: "Elf".to_string(),
                home_biomes: vec![
                    BiomeType::TemperateDeciduousForest,
                    BiomeType::TemperateMixedForest,
                    BiomeType::SubtropicalSeasonalForest,
                    BiomeType::TropicalSeasonalForest,
                ],
                tolerable_biomes: vec![
                    BiomeType::BorealForest,
                    BiomeType::TemperateRainforest,
                    BiomeType::MontaneForest,
                    BiomeType::CoastalWetland,
                ],
                climate_tolerance: ClimateTolerance {
                    min_temp: -10.0,
                    max_temp: 35.0,
                    min_precipitation: 500.0,
                    max_precipitation: 3000.0,
                },
                traits: vec![SpeciesTrait::Nocturnal, SpeciesTrait::PackHunter],
                name_prefixes: vec![
                    "Elder".to_string(),
                    "Silver".to_string(),
                    "Moon".to_string(),
                    "Star".to_string(),
                    "Sun".to_string(),
                    "Wood".to_string(),
                    "Leaf".to_string(),
                    "Golden".to_string(),
                ],
                name_suffixes: vec![
                    "glin".to_string(),
                    "las".to_string(),
                    "nor".to_string(),
                    "sind".to_string(),
                    "thalion".to_string(),
                    "anor".to_string(),
                    "gwaith".to_string(),
                    "vorn".to_string(),
                ],
            },
            // Dwarf - mountain and underground
            Species {
                id: SpeciesId::Dwarf,
                name: "Dwarf".to_string(),
                display_name: "Dwarf".to_string(),
                home_biomes: vec![
                    BiomeType::BorealForest,
                    BiomeType::BorealTaiga,
                    BiomeType::MontaneForest,
                    BiomeType::MontaneGrassland,
                ],
                tolerable_biomes: vec![
                    BiomeType::TemperateSteppe,
                    BiomeType::SubtropicalSteppe,
                    BiomeType::AlpineTundra,
                ],
                climate_tolerance: ClimateTolerance {
                    min_temp: -40.0,
                    max_temp: 25.0,
                    min_precipitation: 100.0,
                    max_precipitation: 2000.0,
                },
                traits: vec![SpeciesTrait::Subterranean, SpeciesTrait::Sedentary],
                name_prefixes: vec![
                    "Iron".to_string(),
                    "Stone".to_string(),
                    "Gold".to_string(),
                    "Silver".to_string(),
                    "Coal".to_string(),
                    "Copper".to_string(),
                    "Mith".to_string(),
                    "Dark".to_string(),
                ],
                name_suffixes: vec![
                    "dal".to_string(),
                    "kar".to_string(),
                    "gor".to_string(),
                    "mord".to_string(),
                    "rung".to_string(),
                    "ak".to_string(),
                    "grim".to_string(),
                    "heim".to_string(),
                ],
            },
            // Orc - hardy and adaptable
            Species {
                id: SpeciesId::Orc,
                name: "Orc".to_string(),
                display_name: "Orc".to_string(),
                home_biomes: vec![
                    BiomeType::BorealTaiga,
                    BiomeType::SemiAridSteppe,
                    BiomeType::SubtropicalSteppe,
                ],
                tolerable_biomes: vec![
                    BiomeType::BorealForest,
                    BiomeType::TemperateSteppe,
                    BiomeType::Tundra,
                    BiomeType::MontaneGrassland,
                ],
                climate_tolerance: ClimateTolerance {
                    min_temp: -50.0,
                    max_temp: 40.0,
                    min_precipitation: 50.0,
                    max_precipitation: 2000.0,
                },
                traits: vec![SpeciesTrait::WarLike, SpeciesTrait::Nomadic],
                name_prefixes: vec![
                    "Grim".to_string(),
                    "Blood".to_string(),
                    "War".to_string(),
                    "Iron".to_string(),
                    "Death".to_string(),
                    "Skull".to_string(),
                    "Bone".to_string(),
                    "Frost".to_string(),
                ],
                name_suffixes: vec![
                    "mar".to_string(),
                    "gor".to_string(),
                    "zug".to_string(),
                    "mash".to_string(),
                    "gra".to_string(),
                    "bur".to_string(),
                    "lok".to_string(),
                    "thak".to_string(),
                ],
            },
            // Halfling - peaceful agricultural
            Species {
                id: SpeciesId::Halfling,
                name: "Halfling".to_string(),
                display_name: "Halfling".to_string(),
                home_biomes: vec![
                    BiomeType::TemperateGrassland,
                    BiomeType::TemperateSteppe,
                    BiomeType::SubtropicalSeasonalForest,
                ],
                tolerable_biomes: vec![
                    BiomeType::TemperateDeciduousForest,
                    BiomeType::CoastalWetland,
                    BiomeType::MontaneGrassland,
                ],
                climate_tolerance: ClimateTolerance {
                    min_temp: -15.0,
                    max_temp: 35.0,
                    min_precipitation: 300.0,
                    max_precipitation: 2500.0,
                },
                traits: vec![SpeciesTrait::Peaceful, SpeciesTrait::Sedentary],
                name_prefixes: vec![
                    "Good".to_string(),
                    "Warm".to_string(),
                    "Sunny".to_string(),
                    "River".to_string(),
                    "Green".to_string(),
                    "Sweet".to_string(),
                    "Light".to_string(),
                    "Happy".to_string(),
                ],
                name_suffixes: vec![
                    "hollow".to_string(),
                    "bottom".to_string(),
                    "wood".to_string(),
                    "dale".to_string(),
                    "brook".to_string(),
                    "vale".to_string(),
                    "ridge".to_string(),
                    "acre".to_string(),
                ],
            },
        ];

        let name_templates: HashMap<SpeciesId, NameTemplate> = species
            .iter()
            .map(|s| {
                let template = NameTemplate {
                    species_id: s.id,
                    prefixes: s.name_prefixes.clone(),
                    suffixes: s.name_suffixes.clone(),
                    compound_patterns: Vec::new(),
                };
                (s.id, template)
            })
            .collect();

        Self {
            species,
            name_templates,
        }
    }

    /// Get species by ID.
    pub fn get(&self, id: SpeciesId) -> Option<&Species> {
        self.species.iter().find(|s| s.id == id)
    }

    /// Find the best species for a given biome.
    /// Returns None if no species can inhabit or tolerate the biome.
    pub fn best_species_for_biome(&self, biome: BiomeType) -> Option<SpeciesId> {
        self.species
            .iter()
            .filter(|s| s.biome_suitability(biome) > 0.0)
            .max_by(|a, b| {
                let suit_a = a.biome_suitability(biome);
                let suit_b = b.biome_suitability(biome);
                suit_a.partial_cmp(&suit_b).unwrap()
            })
            .map(|s| s.id)
    }

    /// Generate a settlement name for a species.
    pub fn generate_name(&self, species_id: SpeciesId, rng: &mut Rng) -> String {
        if let Some(template) = self.name_templates.get(&species_id) {
            if template.prefixes.is_empty() || template.suffixes.is_empty() {
                // Fallback to Human names
                return self.generate_name(SpeciesId::Human, rng);
            }

            let prefix = &template.prefixes[rng.next() as usize % template.prefixes.len()];
            let suffix = &template.suffixes[rng.next() as usize % template.suffixes.len()];
            format!("{}{}", prefix, suffix)
        } else if let Some(species) = self.get(species_id) {
            // Try species' own name templates if available
            if !species.name_prefixes.is_empty() && !species.name_suffixes.is_empty() {
                let prefix =
                    &species.name_prefixes[rng.next() as usize % species.name_prefixes.len()];
                let suffix =
                    &species.name_suffixes[rng.next() as usize % species.name_suffixes.len()];
                return format!("{}{}", prefix, suffix);
            }
            // Fallback to Human
            self.generate_name(SpeciesId::Human, rng)
        } else {
            // Default fallback
            format!("Unknown{}{}", rng.next() % 1000, "")
        }
    }

    /// Get all species IDs.
    pub fn all_species(&self) -> Vec<SpeciesId> {
        self.species.iter().map(|s| s.id).collect()
    }

    /// Load species from a JSON template file.
    ///
    /// # Errors
    /// Returns `TemplateError` if the file cannot be read or parsed.
    pub fn from_template_file(path: &str) -> Result<Self, loader::TemplateError> {
        let loader = loader::SpeciesLoader::new();
        let template = loader.load_json(path)?;
        loader.to_species_data(&template)
    }

    /// Load and merge custom species with default species.
    ///
    /// Custom species with the same ID as defaults will override them.
    /// Custom species with new IDs will be appended.
    ///
    /// # Errors
    /// Returns `TemplateError` if the file cannot be read or parsed.
    pub fn load_and_merge(path: &str) -> Result<Self, loader::TemplateError> {
        let custom = Self::from_template_file(path)?;
        Ok(loader::merge_with_defaults(custom))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_species_data() {
        let data = SpeciesData::default_species();

        assert_eq!(data.species.len(), 5);
        assert!(data.name_templates.contains_key(&SpeciesId::Human));
        assert!(data.name_templates.contains_key(&SpeciesId::Elf));
        assert!(data.name_templates.contains_key(&SpeciesId::Dwarf));
        assert!(data.name_templates.contains_key(&SpeciesId::Orc));
        assert!(data.name_templates.contains_key(&SpeciesId::Halfling));
    }

    #[test]
    fn test_human_inhabits_temperate() {
        let data = SpeciesData::default_species();
        let human = data.get(SpeciesId::Human).unwrap();

        assert!(human.inhabits(BiomeType::TemperateGrassland));
        assert!(human.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(!human.inhabits(BiomeType::HotDesert));
        assert!(!human.inhabits(BiomeType::Tundra));
    }

    #[test]
    fn test_elf_forest_dwelling() {
        let data = SpeciesData::default_species();
        let elf = data.get(SpeciesId::Elf).unwrap();

        assert!(elf.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(elf.inhabits(BiomeType::TropicalSeasonalForest));
        assert!(!elf.inhabits(BiomeType::HotDesert));
        assert!(!elf.inhabits(BiomeType::Arctic));
    }

    #[test]
    fn test_dwarf_boreal() {
        let data = SpeciesData::default_species();
        let dwarf = data.get(SpeciesId::Dwarf).unwrap();

        assert!(dwarf.inhabits(BiomeType::BorealForest));
        assert!(dwarf.inhabits(BiomeType::MontaneForest));
        assert!(!dwarf.inhabits(BiomeType::TropicalSavanna));
    }

    #[test]
    fn test_best_species_for_biome() {
        let data = SpeciesData::default_species();

        // Both Human and Halfling inhabit TemperateGrassland with equal suitability
        // Either could be returned by best_species_for_biome
        let grassland_species = data.best_species_for_biome(BiomeType::TemperateGrassland);
        assert!(matches!(
            grassland_species,
            Some(SpeciesId::Human) | Some(SpeciesId::Halfling)
        ));

        assert_eq!(
            data.best_species_for_biome(BiomeType::TemperateDeciduousForest),
            Some(SpeciesId::Elf)
        );
        assert_eq!(
            data.best_species_for_biome(BiomeType::BorealForest),
            Some(SpeciesId::Dwarf)
        );
        assert_eq!(
            data.best_species_for_biome(BiomeType::HotDesert),
            None // No species naturally inhabits hot desert
        );
    }

    #[test]
    fn test_generate_name() {
        let data = SpeciesData::default_species();
        let mut rng = Rng::new(42);

        let name = data.generate_name(SpeciesId::Human, &mut rng);
        assert!(!name.is_empty());

        // Verify it's from human templates
        let human = data.get(SpeciesId::Human).unwrap();
        let valid_suffixes: Vec<&str> = human.name_suffixes.iter().map(|s| s.as_str()).collect();
        assert!(valid_suffixes.iter().any(|s| name.ends_with(s)));
    }

    #[test]
    fn test_species_suitability() {
        let data = SpeciesData::default_species();
        let human = data.get(SpeciesId::Human).unwrap();

        assert_eq!(human.biome_suitability(BiomeType::TemperateGrassland), 1.0);
        assert_eq!(human.biome_suitability(BiomeType::BorealForest), 0.5);
        assert_eq!(human.biome_suitability(BiomeType::HotDesert), 0.0);
    }
}
