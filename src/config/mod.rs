//! Configuration Module for World Factory
//!
//! Provides YAML configuration loading with validation and defaults.
//! Config files allow customizing world generation parameters.

pub mod validation;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Maximum allowed dimension for world width/height
const MAX_DIMENSION: usize = 64;

/// Default configuration file name
const DEFAULT_CONFIG_NAME: &str = "world.toml";

/// World configuration loaded from file or defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// World dimensions (width x height in cells)
    #[serde(default)]
    pub dimensions: Dimensions,

    /// Terrain generation parameters
    #[serde(default)]
    pub terrain: TerrainSettings,

    /// River generation parameters
    #[serde(default)]
    pub rivers: RiverSettings,

    /// Biome generation parameters
    #[serde(default)]
    pub biomes: BiomeSettings,

    /// World metadata
    #[serde(default)]
    pub metadata: WorldMetadata,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            dimensions: Dimensions::default(),
            terrain: TerrainSettings::default(),
            rivers: RiverSettings::default(),
            biomes: BiomeSettings::default(),
            metadata: WorldMetadata::default(),
        }
    }
}

/// World dimensions in grid cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    /// Width in cells (max 64)
    pub width: usize,
    /// Height in cells (max 64)
    pub height: usize,
    /// Cell size in meters (1 cell = N meters)
    pub cell_size: f32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            width: 32,
            height: 32,
            cell_size: 1000.0,
        }
    }
}

/// Terrain generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSettings {
    /// Noise generation parameters
    #[serde(default)]
    pub noise: NoiseSettings,
    /// Sea level threshold (0.0 - 1.0)
    pub sea_level: f32,
    /// Tectonic plate simulation
    #[serde(default)]
    pub tectonics: TectonicSettings,
    /// Elevation adjustment
    #[serde(default)]
    pub elevation: ElevationSettings,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            noise: NoiseSettings::default(),
            sea_level: 0.4,
            tectonics: TectonicSettings::default(),
            elevation: ElevationSettings::default(),
        }
    }
}

/// Noise generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseSettings {
    /// Noise scale (frequency)
    pub scale: f32,
    /// Number of octaves
    pub octaves: usize,
    /// Persistence (amplitude decay per octave)
    pub persistence: f32,
    /// Lacunarity (frequency growth per octave)
    pub lacunarity: f32,
    /// Seed for deterministic generation (0 = random)
    pub seed: u64,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            scale: 0.01,
            octaves: 6,
            persistence: 0.5,
            lacunarity: 2.0,
            seed: 0,
        }
    }
}

/// Tectonic plate simulation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TectonicSettings {
    /// Enable tectonic plate simulation
    pub enabled: bool,
    /// Number of major plates
    pub plate_count: usize,
    /// Collision intensity (0.0 - 1.0)
    pub intensity: f32,
}

impl Default for TectonicSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            plate_count: 7,
            intensity: 0.5,
        }
    }
}

/// Elevation adjustment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationSettings {
    /// Base elevation in meters
    pub base_elevation: f32,
    /// Mountain amplitude multiplier
    pub mountain_amplitude: f32,
}

impl Default for ElevationSettings {
    fn default() -> Self {
        Self {
            base_elevation: 500.0,
            mountain_amplitude: 2000.0,
        }
    }
}

/// River generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiverSettings {
    /// River density (0.0 - 1.0, percentage of land)
    pub density: f32,
    /// Minimum river length in cells
    pub min_length: usize,
    /// Maximum river length in cells
    pub max_length: usize,
    /// Erosion intensity (0.0 - 1.0)
    pub erosion_intensity: f32,
}

impl Default for RiverSettings {
    fn default() -> Self {
        Self {
            density: 0.3,
            min_length: 10,
            max_length: 500,
            erosion_intensity: 0.5,
        }
    }
}

/// Biome generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSettings {
    /// Enable fantasy biomes
    pub fantasy_biomes: bool,
    /// Enable coastal variants
    pub coastal_variants: bool,
    /// Temperature variation
    #[serde(default)]
    pub temperature: BiomeVariationSettings,
    /// Precipitation variation
    #[serde(default)]
    pub precipitation: BiomeVariationSettings,
}

impl Default for BiomeSettings {
    fn default() -> Self {
        Self {
            fantasy_biomes: true,
            coastal_variants: true,
            temperature: BiomeVariationSettings::default(),
            precipitation: BiomeVariationSettings::default(),
        }
    }
}

/// Biome variation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeVariationSettings {
    /// Enable noise-based variation
    pub noise_enabled: bool,
    /// Variation intensity (0.0 - 1.0)
    pub intensity: f32,
}

impl Default for BiomeVariationSettings {
    fn default() -> Self {
        Self {
            noise_enabled: true,
            intensity: 0.5,
        }
    }
}

/// World metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// World name
    pub name: String,
    /// World description
    pub description: String,
    /// Genre setting (fantasy, sci-fi, etc.)
    #[serde(default)]
    pub genre: String,
}

impl Default for WorldMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed World".to_string(),
            description: String::new(),
            genre: "fantasy".to_string(),
        }
    }
}

impl WorldConfig {
    /// Load configuration from a TOML file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;

        Self::from_toml(&contents)
    }
    
    /// Create a simple config with just the essentials.
    /// This is a convenience constructor for testing and simple use cases.
    pub fn simple(seed: u64, width: usize, height: usize, sea_level: f32) -> Self {
        Self {
            dimensions: Dimensions {
                width,
                height,
                cell_size: 1000.0,
            },
            terrain: TerrainSettings {
                noise: NoiseSettings {
                    seed,
                    scale: 0.01,
                    octaves: 6,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
                sea_level,
                tectonics: TectonicSettings::default(),
                elevation: ElevationSettings::default(),
            },
            rivers: RiverSettings::default(),
            biomes: BiomeSettings::default(),
            metadata: WorldMetadata::default(),
        }
    }
    
    /// Get the seed value
    pub fn seed(&self) -> u64 {
        self.terrain.noise.seed
    }
    
    /// Get the width value
    pub fn width(&self) -> usize {
        self.dimensions.width
    }
    
    /// Get the height value
    pub fn height(&self) -> usize {
        self.dimensions.height
    }
    
    /// Get the sea level value
    pub fn sea_level(&self) -> f32 {
        self.terrain.sea_level
    }
    
    /// Set the seed value
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.terrain.noise.seed = seed;
        self
    }

    /// Load configuration from TOML string
    ///
    /// # Errors
    /// Returns an error if the TOML is invalid
    pub fn from_toml(toml: &str) -> Result<Self, ConfigError> {
        let config: WorldConfig = toml::from_str(toml)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    ///
    /// # Errors
    /// Returns validation errors for invalid values
    pub fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_world_config(self)
    }

    /// Create from defaults with overrides
    pub fn with_overrides(overrides: &str) -> Result<Self, ConfigError> {
        let mut config = WorldConfig::default();
        let override_config: WorldConfig = toml::from_str(overrides)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        // Apply overrides (simple merge - override values win)
        if override_config.dimensions.width != 0 {
            config.dimensions.width = override_config.dimensions.width;
        }
        if override_config.dimensions.height != 0 {
            config.dimensions.height = override_config.dimensions.height;
        }
        config.dimensions.cell_size = override_config.dimensions.cell_size;
        config.terrain = override_config.terrain;
        config.rivers = override_config.rivers;
        config.biomes = override_config.biomes;

        config.validate()?;
        Ok(config)
    }

    /// Get total cell count
    pub fn total_cells(&self) -> usize {
        self.dimensions.width * self.dimensions.height
    }

    /// Get approximate world area in km2
    pub fn world_area_km2(&self) -> f64 {
        let cell_area_km2 = (self.dimensions.cell_size as f64 / 1000.0).powi(2);
        (self.total_cells() as f64) * cell_area_km2
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(String),

    #[error("Failed to parse config: {0}")]
    Parse(String),

    #[error("Validation failed: {0}")]
    Validation(String),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorldConfig::default();
        assert_eq!(config.dimensions.width, 32);
        assert_eq!(config.dimensions.height, 32);
        assert_eq!(config.terrain.sea_level, 0.4);
    }

    #[test]
    fn test_valid_custom_config() {
        let toml = r#"
            [dimensions]
            width = 64
            height = 64
            cell_size = 500.0

            [terrain]
            sea_level = 0.35
            seed = 42

            [terrain.noise]
            scale = 0.02
            octaves = 8
            persistence = 0.6
            lacunarity = 2.5
            seed = 42

            [terrain.tectonics]
            enabled = true
            plate_count = 12
            intensity = 0.7

            [rivers]
            density = 0.25
            min_length = 15
            max_length = 300
            erosion_intensity = 0.6
        "#;

        let config = WorldConfig::from_toml(toml).unwrap();
        assert_eq!(config.dimensions.width, 64);
        assert_eq!(config.dimensions.height, 64);
        assert_eq!(config.terrain.noise.octaves, 8);
        assert_eq!(config.rivers.density, 0.25);
    }

    #[test]
    fn test_validation_width_exceeds_max() {
        let config = WorldConfig {
            dimensions: Dimensions {
                width: 128, // Exceeds MAX_DIMENSION (64)
                height: 32,
                cell_size: 1000.0,
            },
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Validation(_)));
    }

    #[test]
    fn test_validation_height_exceeds_max() {
        let config = WorldConfig {
            dimensions: Dimensions {
                width: 32,
                height: 128, // Exceeds MAX_DIMENSION (64)
                cell_size: 1000.0,
            },
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_sea_level() {
        let toml = r#"
            [dimensions]
            width = 32
            height = 32

            [terrain]
            sea_level = 1.5  # Invalid: > 1.0
        "#;

        let result = WorldConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_total_cells() {
        let config = WorldConfig {
            dimensions: Dimensions {
                width: 32,
                height: 32,
                cell_size: 1000.0,
            },
            ..Default::default()
        };

        assert_eq!(config.total_cells(), 1024);
    }

    #[test]
    fn test_world_area() {
        let config = WorldConfig {
            dimensions: Dimensions {
                width: 10,
                height: 10,
                cell_size: 1000.0, // 1km
            },
            ..Default::default()
        };

        // 10x10 = 100 cells, each 1km2 = 100km2
        assert_eq!(config.world_area_km2(), 100.0);
    }
}