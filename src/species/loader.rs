//! Species Template Loading Module
//!
//! Handles loading species definitions from JSON template files.
//! Provides validation, error handling, and merge capabilities.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use world_factory::species::loader::{SpeciesLoader, merge_with_defaults};
//!
//! // Load custom species from JSON
//! let loader = SpeciesLoader::new();
//! // let template = loader.load_json("species_custom.json")?;
//! // let species_data = loader.to_species_data(&template)?;
//!
//! // Or merge with default species
//! // let combined = merge_with_defaults(species_data);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use super::{ClimateTolerance, NameTemplate, Species, SpeciesData, SpeciesId, SpeciesTrait};
use crate::terrain::biome::BiomeType;

/// Errors that can occur during template loading.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Failed to read template file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse template JSON: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Invalid template version: {0} (expected '1.0')")]
    Version(String),

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid biome type: {0}")]
    InvalidBiome(String),

    #[error("Invalid species trait: {0}")]
    InvalidTrait(String),

    #[error("Species ID {0} already exists in template")]
    DuplicateId(u32),

    #[error("Custom species must have ID >= 100 (got {0})")]
    InvalidCustomId(u32),

    #[error("Validation failed: {0}")]
    Validation(String),
}

/// Template file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Pack name
    pub name: String,
    /// Pack author
    #[serde(default)]
    pub author: String,
    /// Pack description
    #[serde(default)]
    pub description: String,
    /// Source/URL reference
    #[serde(default)]
    pub source: String,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Name template section from template file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateNameTemplates {
    /// Valid name prefixes
    pub prefixes: Vec<String>,
    /// Valid name suffixes
    pub suffixes: Vec<String>,
    /// Optional compound name patterns (e.g., "{prefix1}{prefix2}{suffix}")
    #[serde(default)]
    pub compound_patterns: Vec<String>,
}

/// Climate tolerance in template format (uses f64 for JSON compatibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateClimateTolerance {
    /// Minimum temperature (Celsius)
    pub min_temp: f64,
    /// Maximum temperature (Celsius)
    pub max_temp: f64,
    /// Minimum precipitation (mm/year)
    pub min_precipitation: f64,
    /// Maximum precipitation (mm/year)
    pub max_precipitation: f64,
}

impl From<TemplateClimateTolerance> for ClimateTolerance {
    fn from(t: TemplateClimateTolerance) -> Self {
        ClimateTolerance {
            min_temp: t.min_temp as f32,
            max_temp: t.max_temp as f32,
            min_precipitation: t.min_precipitation as f32,
            max_precipitation: t.max_precipitation as f32,
        }
    }
}

/// Species definition from template file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesTemplate {
    /// Unique species ID (>= 100 for custom species)
    pub id: u32,
    /// Internal name (e.g., "Human", "Elf")
    pub name: String,
    /// Display name for UI
    pub display_name: String,
    /// Biomes where species naturally thrives
    pub home_biomes: Vec<String>,
    /// Biomes species can survive in with adaptation
    #[serde(default)]
    pub tolerable_biomes: Vec<String>,
    /// Climate tolerance ranges
    pub climate_tolerance: TemplateClimateTolerance,
    /// Behavioral traits affecting settlement
    #[serde(default)]
    pub traits: Vec<String>,
    /// Name generation templates
    #[serde(default)]
    pub name_templates: Option<TemplateNameTemplates>,
}

/// Complete template file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesTemplateFile {
    /// Schema version (must be "1.0")
    #[serde(default = "default_version")]
    pub version: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<TemplateMetadata>,
    /// Species definitions
    pub species: Vec<SpeciesTemplate>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Species loader for template files.
#[derive(Debug, Clone)]
pub struct SpeciesLoader {
    biome_map: HashMap<String, BiomeType>,
    trait_map: HashMap<String, SpeciesTrait>,
}

impl SpeciesLoader {
    /// Create a new species loader with all known biome/trait mappings.
    pub fn new() -> Self {
        let biome_map = Self::build_biome_map();
        let trait_map = Self::build_trait_map();
        Self {
            biome_map,
            trait_map,
        }
    }

    /// Build the biome name → BiomeType mapping.
    fn build_biome_map() -> HashMap<String, BiomeType> {
        HashMap::from([
            (
                "TropicalRainforest".to_string(),
                BiomeType::TropicalRainforest,
            ),
            (
                "TropicalSeasonalForest".to_string(),
                BiomeType::TropicalSeasonalForest,
            ),
            ("TropicalSavanna".to_string(), BiomeType::TropicalSavanna),
            (
                "TropicalDryForest".to_string(),
                BiomeType::TropicalDryForest,
            ),
            (
                "SubtropicalRainforest".to_string(),
                BiomeType::SubtropicalRainforest,
            ),
            (
                "SubtropicalSeasonalForest".to_string(),
                BiomeType::SubtropicalSeasonalForest,
            ),
            (
                "SubtropicalSteppe".to_string(),
                BiomeType::SubtropicalSteppe,
            ),
            (
                "SubtropicalDesert".to_string(),
                BiomeType::SubtropicalDesert,
            ),
            (
                "TemperateRainforest".to_string(),
                BiomeType::TemperateRainforest,
            ),
            (
                "TemperateDeciduousForest".to_string(),
                BiomeType::TemperateDeciduousForest,
            ),
            (
                "TemperateMixedForest".to_string(),
                BiomeType::TemperateMixedForest,
            ),
            ("TemperateSteppe".to_string(), BiomeType::TemperateSteppe),
            ("TemperateDesert".to_string(), BiomeType::TemperateDesert),
            ("BorealTaiga".to_string(), BiomeType::BorealTaiga),
            ("BorealForest".to_string(), BiomeType::BorealForest),
            (
                "TemperateGrassland".to_string(),
                BiomeType::TemperateGrassland,
            ),
            ("Tundra".to_string(), BiomeType::Tundra),
            ("Arctic".to_string(), BiomeType::Arctic),
            ("PolarDesert".to_string(), BiomeType::PolarDesert),
            ("MontaneForest".to_string(), BiomeType::MontaneForest),
            ("MontaneGrassland".to_string(), BiomeType::MontaneGrassland),
            ("AlpineTundra".to_string(), BiomeType::AlpineTundra),
            ("SnowGlacier".to_string(), BiomeType::SnowGlacier),
            ("CoastalWetland".to_string(), BiomeType::CoastalWetland),
            ("Mangrove".to_string(), BiomeType::Mangrove),
            ("CoralReef".to_string(), BiomeType::CoralReef),
            ("KelpForest".to_string(), BiomeType::KelpForest),
            ("OpenOcean".to_string(), BiomeType::OpenOcean),
            ("HotDesert".to_string(), BiomeType::HotDesert),
            ("ColdDesert".to_string(), BiomeType::ColdDesert),
            ("SemiAridSteppe".to_string(), BiomeType::SemiAridSteppe),
            ("MagicalForest".to_string(), BiomeType::MagicalForest),
            (
                "CrystallineDesert".to_string(),
                BiomeType::CrystallineDesert,
            ),
            (
                "BioluminescentOcean".to_string(),
                BiomeType::BioluminescentOcean,
            ),
            (
                "VolcanicLandscape".to_string(),
                BiomeType::VolcanicLandscape,
            ),
            ("ToxicSwamp".to_string(), BiomeType::ToxicSwamp),
            ("FloatingIslands".to_string(), BiomeType::FloatingIslands),
        ])
    }

    /// Build the trait name → SpeciesTrait mapping.
    fn build_trait_map() -> HashMap<String, SpeciesTrait> {
        HashMap::from([
            ("Aquatic".to_string(), SpeciesTrait::Aquatic),
            ("Flying".to_string(), SpeciesTrait::Flying),
            ("Subterranean".to_string(), SpeciesTrait::Subterranean),
            ("Nocturnal".to_string(), SpeciesTrait::Nocturnal),
            ("PackHunter".to_string(), SpeciesTrait::PackHunter),
            ("Nomadic".to_string(), SpeciesTrait::Nomadic),
            ("Sedentary".to_string(), SpeciesTrait::Sedentary),
            ("TradeFocused".to_string(), SpeciesTrait::TradeFocused),
            ("WarLike".to_string(), SpeciesTrait::WarLike),
            ("Peaceful".to_string(), SpeciesTrait::Peaceful),
            ("Adaptable".to_string(), SpeciesTrait::Adaptable),
            ("Curious".to_string(), SpeciesTrait::Curious),
        ])
    }

    /// Load species from a JSON template file.
    pub fn load_json<P: AsRef<Path>>(&self, path: P) -> Result<SpeciesTemplateFile, TemplateError> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        self.parse_json(&contents)
    }

    /// Parse species from JSON string.
    pub fn parse_json(&self, json: &str) -> Result<SpeciesTemplateFile, TemplateError> {
        let template: SpeciesTemplateFile = serde_json::from_str(json)?;
        self.validate_template(&template)?;
        Ok(template)
    }

    /// Convert template file to SpeciesData.
    pub fn to_species_data(
        &self,
        template: &SpeciesTemplateFile,
    ) -> Result<SpeciesData, TemplateError> {
        let mut species_list = Vec::new();
        let mut name_templates_map: HashMap<SpeciesId, NameTemplate> = HashMap::new();

        for spec in &template.species {
            let species = self.convert_species(spec)?;
            species_list.push(species);

            // Build name templates
            if let Some(ref templates) = spec.name_templates {
                name_templates_map.insert(
                    SpeciesId::from_u32(spec.id),
                    NameTemplate {
                        species_id: SpeciesId::from_u32(spec.id),
                        prefixes: templates.prefixes.clone(),
                        suffixes: templates.suffixes.clone(),
                        compound_patterns: templates.compound_patterns.clone(),
                    },
                );
            }
        }

        Ok(SpeciesData {
            species: species_list,
            name_templates: name_templates_map,
        })
    }

    /// Validate template file structure.
    fn validate_template(&self, template: &SpeciesTemplateFile) -> Result<(), TemplateError> {
        // Check version
        if template.version != "1.0" {
            return Err(TemplateError::Version(template.version.clone()));
        }

        // Check we have species
        if template.species.is_empty() {
            return Err(TemplateError::Validation(
                "species array cannot be empty".to_string(),
            ));
        }

        // Validate each species
        let mut seen_ids = std::collections::HashSet::new();
        for spec in &template.species {
            // Check for duplicate IDs
            if !seen_ids.insert(spec.id) {
                return Err(TemplateError::DuplicateId(spec.id));
            }

            // Custom species must have ID >= 100
            if spec.id < 100 {
                return Err(TemplateError::InvalidCustomId(spec.id));
            }

            // Validate biome names
            for biome in &spec.home_biomes {
                if !self.biome_map.contains_key(biome) {
                    return Err(TemplateError::InvalidBiome(biome.clone()));
                }
            }
            for biome in &spec.tolerable_biomes {
                if !self.biome_map.contains_key(biome) {
                    return Err(TemplateError::InvalidBiome(biome.clone()));
                }
            }

            // Validate traits
            for trait_name in &spec.traits {
                if !self.trait_map.contains_key(trait_name) {
                    return Err(TemplateError::InvalidTrait(trait_name.clone()));
                }
            }

            // Validate name templates if present
            if let Some(ref templates) = spec.name_templates {
                if templates.prefixes.is_empty() {
                    return Err(TemplateError::Validation(format!(
                        "Species '{}' name_templates.prefixes cannot be empty",
                        spec.name
                    )));
                }
                if templates.suffixes.is_empty() {
                    return Err(TemplateError::Validation(format!(
                        "Species '{}' name_templates.suffixes cannot be empty",
                        spec.name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Convert template species to runtime Species.
    fn convert_species(&self, spec: &SpeciesTemplate) -> Result<Species, TemplateError> {
        // Convert biome names to BiomeType
        let home_biomes: Vec<BiomeType> = spec
            .home_biomes
            .iter()
            .filter_map(|b| self.biome_map.get(b).copied())
            .collect();

        let tolerable_biomes: Vec<BiomeType> = spec
            .tolerable_biomes
            .iter()
            .filter_map(|b| self.biome_map.get(b).copied())
            .collect();

        // Convert trait names to SpeciesTrait
        let traits: Vec<SpeciesTrait> = spec
            .traits
            .iter()
            .filter_map(|t| self.trait_map.get(t).copied())
            .collect();

        // Build name prefixes/suffixes from templates if available
        let (name_prefixes, name_suffixes) = if let Some(ref templates) = spec.name_templates {
            (
                templates.prefixes.iter().map(|s| s.clone()).collect(),
                templates.suffixes.iter().map(|s| s.clone()).collect(),
            )
        } else {
            (Vec::new(), Vec::new())
        };

        Ok(Species {
            id: SpeciesId::from_u32(spec.id),
            name: spec.name.clone(),
            display_name: spec.display_name.clone(),
            home_biomes,
            tolerable_biomes,
            climate_tolerance: spec.climate_tolerance.clone().into(),
            traits,
            name_prefixes,
            name_suffixes,
        })
    }
}

impl Default for SpeciesLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge custom species data with default species.
///
/// Custom species with the same ID as defaults will override them.
/// Custom species with new IDs will be appended.
pub fn merge_with_defaults(custom: SpeciesData) -> SpeciesData {
    // Start with defaults
    let mut combined = SpeciesData::default_species();

    // Add/override with custom species
    for species in custom.species {
        // Remove existing species with same ID
        combined.species.retain(|s| s.id != species.id);
        combined.species.push(species);
    }

    // Merge name templates (custom takes precedence)
    for (id, template) in custom.name_templates {
        combined.name_templates.insert(id, template);
    }

    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TEMPLATE_JSON: &str = r#"{
        "version": "1.0",
        "metadata": {
            "name": "Test Species Pack",
            "author": "Test"
        },
        "species": [
            {
                "id": 101,
                "name": "TestSpecies",
                "display_name": "Test Species",
                "home_biomes": ["TemperateGrassland", "TemperateDeciduousForest"],
                "tolerable_biomes": ["BorealForest"],
                "climate_tolerance": {
                    "min_temp": -20.0,
                    "max_temp": 35.0,
                    "min_precipitation": 200.0,
                    "max_precipitation": 2500.0
                },
                "traits": ["Sedentary", "TradeFocused"],
                "name_templates": {
                    "prefixes": ["Test", "Mock"],
                    "suffixes": ["ville", "town"]
                }
            }
        ]
    }"#;

    #[test]
    fn test_parse_valid_template() {
        let loader = SpeciesLoader::new();
        let template = loader.parse_json(VALID_TEMPLATE_JSON).unwrap();

        assert_eq!(template.version, "1.0");
        assert_eq!(template.species.len(), 1);
        assert_eq!(template.species[0].id, 101);
        assert_eq!(template.species[0].name, "TestSpecies");
    }

    #[test]
    fn test_invalid_version() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "2.0",
            "species": [{"id": 101, "name": "Test", "display_name": "Test", "home_biomes": [], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}}]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::Version(_))));
    }

    #[test]
    fn test_duplicate_id_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": [
                {"id": 101, "name": "Test1", "display_name": "Test 1", "home_biomes": ["TemperateGrassland"], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}},
                {"id": 101, "name": "Test2", "display_name": "Test 2", "home_biomes": ["TemperateGrassland"], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}}
            ]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::DuplicateId(101))));
    }

    #[test]
    fn test_id_less_than_100_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": [{"id": 50, "name": "Test", "display_name": "Test", "home_biomes": [], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}}]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::InvalidCustomId(50))));
    }

    #[test]
    fn test_invalid_biome_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": [{"id": 101, "name": "Test", "display_name": "Test", "home_biomes": ["InvalidBiome"], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}}]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::InvalidBiome(_))));
    }

    #[test]
    fn test_invalid_trait_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": [{"id": 101, "name": "Test", "display_name": "Test", "home_biomes": ["TemperateGrassland"], "traits": ["InvalidTrait"], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}}]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::InvalidTrait(_))));
    }

    #[test]
    fn test_empty_species_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": []
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::Validation(_))));
    }

    #[test]
    fn test_empty_prefixes_rejected() {
        let loader = SpeciesLoader::new();
        let json = r#"{
            "version": "1.0",
            "species": [{"id": 101, "name": "Test", "display_name": "Test", "home_biomes": ["TemperateGrassland"], "climate_tolerance": {"min_temp": 0, "max_temp": 30, "min_precipitation": 100, "max_precipitation": 1000}, "name_templates": {"prefixes": [], "suffixes": ["town"]}}]
        }"#;
        let result = loader.parse_json(json);
        assert!(matches!(result, Err(TemplateError::Validation(_))));
    }

    #[test]
    fn test_convert_to_species_data() {
        let loader = SpeciesLoader::new();
        let template = loader.parse_json(VALID_TEMPLATE_JSON).unwrap();
        let species_data = loader.to_species_data(&template).unwrap();

        assert_eq!(species_data.species.len(), 1);
        let species = &species_data.species[0];
        assert_eq!(species.id, SpeciesId::from_u32(101));
        assert_eq!(
            species.home_biomes,
            vec![
                BiomeType::TemperateGrassland,
                BiomeType::TemperateDeciduousForest
            ]
        );
        assert_eq!(
            species.traits,
            vec![SpeciesTrait::Sedentary, SpeciesTrait::TradeFocused]
        );
    }

    #[test]
    fn test_merge_preserves_defaults() {
        let loader = SpeciesLoader::new();
        let template = loader.parse_json(VALID_TEMPLATE_JSON).unwrap();
        let custom_data = loader.to_species_data(&template).unwrap();
        let combined = merge_with_defaults(custom_data);

        // Should have default species (5) + custom (1) = 6
        assert_eq!(combined.species.len(), 6);

        // Default species should still be present
        assert!(combined.get(SpeciesId::Human).is_some());
        assert!(combined.get(SpeciesId::Elf).is_some());
        assert!(combined.get(SpeciesId::Dwarf).is_some());
    }

    #[test]
    fn test_merge_overrides_same_id() {
        let loader = SpeciesLoader::new();
        // Create a template with custom ID to add a new species
        let json = r#"{
            "version": "1.0",
            "species": [{
                "id": 101,
                "name": "CustomSpecies",
                "display_name": "Custom Species",
                "home_biomes": ["TropicalRainforest"],
                "climate_tolerance": {"min_temp": 20.0, "max_temp": 40.0, "min_precipitation": 1000.0, "max_precipitation": 4000.0}
            }]
        }"#;
        let template = loader.parse_json(json).unwrap();
        let custom_data = loader.to_species_data(&template).unwrap();
        let combined = merge_with_defaults(custom_data);

        // Should have default species (5) + custom (1) = 6
        assert_eq!(combined.species.len(), 6);

        // CustomSpecies should be present with its custom home biome
        let custom = combined.get(SpeciesId::from_u32(101)).unwrap();
        assert_eq!(custom.name, "CustomSpecies");
        assert_eq!(custom.home_biomes, vec![BiomeType::TropicalRainforest]);
    }

    #[test]
    fn test_deterministic_loading() {
        let loader = SpeciesLoader::new();

        let template1 = loader.parse_json(VALID_TEMPLATE_JSON).unwrap();
        let template2 = loader.parse_json(VALID_TEMPLATE_JSON).unwrap();

        let data1 = loader.to_species_data(&template1).unwrap();
        let data2 = loader.to_species_data(&template2).unwrap();

        // Same input should produce same output
        assert_eq!(data1.species.len(), data2.species.len());
        assert_eq!(data1.species[0].id, data2.species[0].id);
        assert_eq!(data1.species[0].name, data2.species[0].name);
    }
}
