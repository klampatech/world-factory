//! Configuration Validation Module
//! 
//! Validates WorldConfig values and returns descriptive errors.

use super::{WorldConfig, ConfigError, MAX_DIMENSION};

/// Validate a world configuration.
/// 
/// Checks:
/// - Dimensions don't exceed maximum allowed
/// - Sea level is within valid range
/// - Noise parameters are reasonable
/// - River settings are valid
pub fn validate_world_config(config: &WorldConfig) -> Result<(), ConfigError> {
    // Validate width
    if config.dimensions.width == 0 {
        return Err(ConfigError::Validation(
            "Width must be greater than 0".to_string()
        ));
    }
    if config.dimensions.width > MAX_DIMENSION {
        return Err(ConfigError::Validation(format!(
            "Width {} exceeds maximum allowed dimension {}",
            config.dimensions.width, MAX_DIMENSION
        )));
    }
    
    // Validate height
    if config.dimensions.height == 0 {
        return Err(ConfigError::Validation(
            "Height must be greater than 0".to_string()
        ));
    }
    if config.dimensions.height > MAX_DIMENSION {
        return Err(ConfigError::Validation(format!(
            "Height {} exceeds maximum allowed dimension {}",
            config.dimensions.height, MAX_DIMENSION
        )));
    }
    
    // Validate sea level
    if !(0.0..=1.0).contains(&config.terrain.sea_level) {
        return Err(ConfigError::Validation(format!(
            "Sea level must be between 0.0 and 1.0, got {}",
            config.terrain.sea_level
        )));
    }
    
    // Validate noise octaves
    if config.terrain.noise.octaves == 0 {
        return Err(ConfigError::Validation(
            "Noise octaves must be greater than 0".to_string()
        ));
    }
    if config.terrain.noise.octaves > 16 {
        return Err(ConfigError::Validation(format!(
            "Noise octaves {} exceeds maximum of 16",
            config.terrain.noise.octaves
        )));
    }
    
    // Validate noise persistence
    if !(0.0..=1.0).contains(&config.terrain.noise.persistence) {
        return Err(ConfigError::Validation(format!(
            "Noise persistence must be between 0.0 and 1.0, got {}",
            config.terrain.noise.persistence
        )));
    }
    
    // Validate noise lacunarity
    if config.terrain.noise.lacunarity < 1.0 {
        return Err(ConfigError::Validation(format!(
            "Noise lacunarity must be >= 1.0, got {}",
            config.terrain.noise.lacunarity
        )));
    }
    
    // Validate river density
    if !(0.0..=1.0).contains(&config.rivers.density) {
        return Err(ConfigError::Validation(format!(
            "River density must be between 0.0 and 1.0, got {}",
            config.rivers.density
        )));
    }
    
    // Validate river lengths
    if config.rivers.min_length > config.rivers.max_length {
        return Err(ConfigError::Validation(format!(
            "Minimum river length ({}) exceeds maximum ({})",
            config.rivers.min_length, config.rivers.max_length
        )));
    }
    
    // Validate erosion intensity
    if !(0.0..=1.0).contains(&config.rivers.erosion_intensity) {
        return Err(ConfigError::Validation(format!(
            "Erosion intensity must be between 0.0 and 1.0, got {}",
            config.rivers.erosion_intensity
        )));
    }
    
    // Validate tectonic settings
    if config.terrain.tectonics.plate_count == 0 {
        return Err(ConfigError::Validation(
            "Plate count must be greater than 0".to_string()
        ));
    }
    if config.terrain.tectonics.plate_count > 20 {
        return Err(ConfigError::Validation(format!(
            "Plate count {} exceeds maximum of 20",
            config.terrain.tectonics.plate_count
        )));
    }
    if !(0.0..=1.0).contains(&config.terrain.tectonics.intensity) {
        return Err(ConfigError::Validation(format!(
            "Tectonic intensity must be between 0.0 and 1.0, got {}",
            config.terrain.tectonics.intensity
        )));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_config() {
        let config = WorldConfig::default();
        assert!(validate_world_config(&config).is_ok());
    }
    
    #[test]
    fn test_invalid_width() {
        let mut config = WorldConfig::default();
        config.dimensions.width = 0;
        assert!(validate_world_config(&config).is_err());
    }
    
    #[test]
    fn test_invalid_sea_level() {
        let mut config = WorldConfig::default();
        config.terrain.sea_level = 1.5;
        assert!(validate_world_config(&config).is_err());
    }
    
    #[test]
    fn test_invalid_octaves() {
        let mut config = WorldConfig::default();
        config.terrain.noise.octaves = 20;
        assert!(validate_world_config(&config).is_err());
    }
}