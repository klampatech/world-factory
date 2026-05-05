//! Geography Generator - generates Geography objects from terrain data
//! 
//! Produces Geography metadata for each cell/region based on climate,
//! elevation, and drainage data. Geography objects are used by the API
//! to expose geographic context for rendered worlds.

use crate::world::entities::planet::{
    Geography, Temperature, Precipitation, DrainageType, ElevationZone,
};
use crate::terrain::{BiomeType, BiomeAssignmentMatrix};
use crate::hydro::River;
use crate::util::Rng;

/// Configuration for geography generation.
#[derive(Debug, Clone)]
pub struct GeographyConfig {
    /// Base temperature in Celsius for equator (latitude 0).
    pub base_temperature: f32,
    /// Temperature lapse rate per 1000m elevation (°C/km).
    pub lapse_rate: f32,
    /// How much latitude affects temperature (°C per degree).
    pub latitude_temp_gradient: f32,
    /// Enable freshwater index calculation based on river proximity.
    pub calculate_freshwater: bool,
}

impl Default for GeographyConfig {
    fn default() -> Self {
        Self {
            base_temperature: 30.0,
            lapse_rate: -6.5,
            latitude_temp_gradient: 0.6,
            calculate_freshwater: true,
        }
    }
}

/// Geography generator - produces Geography objects from terrain/climate data.
#[derive(Debug, Clone)]
pub struct GeographyGenerator {
    config: GeographyConfig,
    biome_matrix: BiomeAssignmentMatrix,
}

impl GeographyGenerator {
    /// Create a new geography generator with default configuration.
    pub fn new() -> Self {
        Self::with_config(GeographyConfig::default())
    }

    /// Create a new geography generator with custom configuration.
    pub fn with_config(config: GeographyConfig) -> Self {
        Self {
            config,
            biome_matrix: BiomeAssignmentMatrix::new(),
        }
    }

    /// Generate a single Geography object for a cell.
    ///
    /// # Arguments
    /// * `elevation` - Terrain elevation in meters
    /// * `latitude` - Latitude in degrees (-90 to 90)
    /// * `biome` - Assigned biome type
    /// * `rivers_nearby` - Whether a river passes through or near this cell
    /// * `rng` - Random number generator for deterministic variation
    pub fn generate_cell(
        &self,
        elevation: f32,
        latitude: f32,
        biome: BiomeType,
        rivers_nearby: bool,
        rng: &mut Rng,
    ) -> Geography {
        // Calculate temperature with lapse rate
        let base_temp = self.config.base_temperature - (latitude.abs() * self.config.latitude_temp_gradient);
        let lapse_adjustment = (elevation / 1000.0) * self.config.lapse_rate;
        let temperature = (base_temp + lapse_adjustment).clamp(-90.0, 60.0);
        
        // Estimate precipitation from biome characteristics
        let precipitation = self.estimate_precipitation(biome, latitude, rng);
        
        // Determine drainage type
        let drainage = self.determine_drainage(elevation, biome, rivers_nearby);
        
        // Determine elevation zone
        let elevation_zone = self.determine_elevation_zone(elevation, latitude);
        
        // Create and return Geography
        let mut geo = Geography::new(
            Temperature::new(temperature).unwrap_or(Temperature::new(15.0).unwrap()),
            Precipitation::new(precipitation).unwrap_or(Precipitation::new(500.0).unwrap()),
            drainage,
            elevation_zone,
            latitude,
        );
        
        // Set freshwater index if enabled
        if self.config.calculate_freshwater {
            geo.freshwater_index = Some(self.calculate_freshwater_index(elevation, rivers_nearby));
        }
        
        geo
    }

    /// Generate geography data for a grid of cells.
    /// 
    /// # Arguments
    /// * `width` - Grid width
    /// * `height` - Grid height
    /// * `elevation_fn` - Function to get elevation at (x, y)
    /// * `biome_grid` - Pre-computed biome types
    /// * `rivers` - River data for freshwater calculation
    /// * `seed` - Random seed for deterministic generation
    ///
    /// # Returns
    /// Vec of Geography objects (width * height elements, row-major order)
    pub fn generate_grid<F>(
        &self,
        width: usize,
        height: usize,
        mut elevation_fn: F,
        biome_grid: &[BiomeType],
        rivers: &[River],
        seed: u64,
    ) -> Vec<Geography>
    where
        F: FnMut(usize, usize) -> f32,
    {
        let mut rng = Rng::new(seed);
        let mut geographies = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let elevation = elevation_fn(x, y);
                let latitude = (y as f32 / height as f32) * 90.0 - 45.0; // Center at equator
                
                // Get biome from grid (row-major order)
                let biome_idx = y * width + x;
                let biome = biome_grid.get(biome_idx).copied().unwrap_or(BiomeType::TemperateGrassland);
                
                // Check if river is nearby
                let rivers_nearby = self.is_near_river(x as i32, y as i32, rivers);
                
                let geo = self.generate_cell(elevation, latitude, biome, rivers_nearby, &mut rng);
                geographies.push(geo);
            }
        }

        geographies
    }

    /// Estimate precipitation in mm/year based on biome and latitude.
    fn estimate_precipitation(&self, biome: BiomeType, latitude: f32, rng: &mut Rng) -> f32 {
        // Base precipitation ranges by biome type
        let base_precip = match biome {
            BiomeType::TropicalRainforest => 2500.0,
            BiomeType::TemperateRainforest => 1800.0,
            BiomeType::SubtropicalRainforest => 2000.0,
            BiomeType::TropicalSeasonalForest => 1500.0,
            BiomeType::SubtropicalSeasonalForest => 1400.0,
            BiomeType::TropicalDryForest => 1200.0,
            BiomeType::TemperateDeciduousForest => 1200.0,
            BiomeType::TemperateMixedForest => 1100.0,
            BiomeType::BorealForest => 800.0,
            BiomeType::BorealTaiga => 600.0,
            BiomeType::TropicalSavanna => 900.0,
            BiomeType::SubtropicalSteppe => 500.0,
            BiomeType::TemperateSteppe => 450.0,
            BiomeType::SemiAridSteppe => 350.0,
            BiomeType::TemperateGrassland => 600.0,
            BiomeType::HotDesert => 100.0,
            BiomeType::ColdDesert => 150.0,
            BiomeType::SubtropicalDesert => 80.0,
            BiomeType::TemperateDesert => 120.0,
            BiomeType::Tundra => 300.0,
            BiomeType::Arctic => 100.0,
            BiomeType::PolarDesert => 150.0,
            // Mountain/Highland biomes
            BiomeType::MontaneForest => 1000.0,
            BiomeType::MontaneGrassland => 700.0,
            BiomeType::AlpineTundra => 500.0,
            BiomeType::SnowGlacier => 200.0,
            // Wetland/Coastal biomes
            BiomeType::CoastalWetland => 2000.0,
            BiomeType::Mangrove => 2500.0,
            // Aquatic biomes
            BiomeType::OpenOcean => 0.0,
            BiomeType::CoralReef => 0.0,
            BiomeType::KelpForest => 0.0,
            // Fantasy biomes - use tempered defaults
            BiomeType::MagicalForest => 1500.0,
            BiomeType::CrystallineDesert => 50.0,
            BiomeType::BioluminescentOcean => 0.0,
            BiomeType::VolcanicLandscape => 200.0,
            BiomeType::ToxicSwamp => 3000.0,
            BiomeType::FloatingIslands => 800.0,
        };

        // Latitude affects precipitation (wet in tropics, dry in subtropics, wet in temperate, dry in polar)
        let lat_factor = if latitude.abs() < 15.0 {
            1.2 // Wet tropics
        } else if latitude.abs() < 35.0 {
            0.7 // Dry subtropics (horse latitudes)
        } else if latitude.abs() < 60.0 {
            1.0 // Temperate
        } else {
            0.8 // Polar
        };

        // Add some noise variation
        let noise = (rng.next_f64Signed() * 0.3 + 1.0) as f32;
        
        (base_precip * lat_factor * noise).max(0.0).min(10000.0)
    }

    /// Determine drainage type based on terrain and rivers.
    fn determine_drainage(&self, elevation: f32, biome: BiomeType, rivers_nearby: bool) -> DrainageType {
        // Ocean and water biomes have endorheic (closed basin) drainage
        if matches!(biome, BiomeType::OpenOcean | BiomeType::CoralReef | BiomeType::KelpForest) {
            return DrainageType::Endorheic;
        }
        
        // Rivers indicate exorheic drainage (to ocean)
        if rivers_nearby {
            return DrainageType::Exorheic;
        }
        
        // High elevation areas with no rivers might be endorheic (inland basins)
        if elevation > 1500.0 {
            return DrainageType::Endorheic;
        }
        
        // Desert regions may have internal drainage
        if matches!(biome, BiomeType::HotDesert | BiomeType::ColdDesert | BiomeType::SubtropicalDesert | BiomeType::TemperateDesert) {
            return DrainageType::Internal;
        }
        
        // Wetlands often have infiltration drainage
        if matches!(biome, BiomeType::CoastalWetland | BiomeType::Mangrove | BiomeType::ToxicSwamp) {
            return DrainageType::Infiltration;
        }
        
        // Default to exorheic for habitable areas
        DrainageType::Exorheic
    }

    /// Determine elevation zone based on meters and latitude.
    fn determine_elevation_zone(&self, elevation: f32, _latitude: f32) -> ElevationZone {
        use crate::terrain::ElevationZone::*;
        
        if elevation > 4500.0 {
            return Nival;
        } else if elevation > 3500.0 {
            return Alpine;
        } else if elevation > 1500.0 {
            return Highland;
        } else if elevation > 200.0 {
            return Lowland;
        } else {
            // Near sea level or below (ocean floor treated as lowland)
            return Lowland;
        }
    }

    /// Calculate freshwater availability index [0.0 to 1.0].
    fn calculate_freshwater_index(&self, elevation: f32, rivers_nearby: bool) -> f32 {
        // Base index from elevation (lower = more accessible water)
        let elevation_factor = if elevation < 500.0 {
            1.0
        } else if elevation < 1500.0 {
            0.7
        } else if elevation < 3000.0 {
            0.4
        } else {
            0.2
        };
        
        // Rivers add freshwater
        let river_bonus: f32 = if rivers_nearby { 0.3 } else { 0.0 };
        
        (elevation_factor + river_bonus).min(1.0)
    }

    /// Check if a cell is near a river.
    fn is_near_river(&self, x: i32, y: i32, rivers: &[River]) -> bool {
        const RIVER_PROXIMITY: i32 = 3;
        
        for river in rivers {
            for cell in &river.cells {
                let dx = (cell.x - x).abs();
                let dy = (cell.y - y).abs();
                if dx <= RIVER_PROXIMITY && dy <= RIVER_PROXIMITY {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for GeographyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_calculation() {
        let gen = GeographyGenerator::new();
        let mut rng = Rng::new(42);
        
        // Equator should be warm
        let geo = gen.generate_cell(0.0, 0.0, BiomeType::TropicalRainforest, false, &mut rng);
        assert!(geo.temperature.as_celsius() > 20.0);
        
        // Poles should be cold
        let geo = gen.generate_cell(0.0, 70.0, BiomeType::Tundra, false, &mut rng);
        assert!(geo.temperature.as_celsius() < 5.0);
    }

    #[test]
    fn test_elevation_affects_temperature() {
        let gen = GeographyGenerator::new();
        let mut rng = Rng::new(42);
        
        // Same latitude, different elevation
        let lowland = gen.generate_cell(200.0, 30.0, BiomeType::TemperateGrassland, false, &mut rng);
        let highland = gen.generate_cell(3000.0, 30.0, BiomeType::MontaneGrassland, false, &mut rng);
        
        assert!(highland.temperature.as_celsius() < lowland.temperature.as_celsius());
    }

    #[test]
    fn test_ocean_biome_drainage() {
        let gen = GeographyGenerator::new();
        let mut rng = Rng::new(42);
        
        let geo = gen.generate_cell(-100.0, 0.0, BiomeType::OpenOcean, false, &mut rng);
        assert_eq!(geo.drainage_type, DrainageType::Endorheic);
    }

    #[test]
    fn test_freshwater_river_bonus() {
        let gen = GeographyGenerator::new();
        let mut rng = Rng::new(42);
        
        // Use a higher elevation so the river bonus makes a difference
        // At 200m elevation, both hit the 1.0 cap, so use 1000m instead
        let no_river = gen.generate_cell(1000.0, 30.0, BiomeType::TemperateGrassland, false, &mut rng);
        let with_river = gen.generate_cell(1000.0, 30.0, BiomeType::TemperateGrassland, true, &mut rng);
        
        if let (Some(f1), Some(f2)) = (no_river.freshwater_index, with_river.freshwater_index) {
            assert!(f2 > f1);
        }
    }

    #[test]
    fn test_elevation_zones() {
        let gen = GeographyGenerator::new();
        let mut rng = Rng::new(42);
        
        let coastal = gen.generate_cell(50.0, 30.0, BiomeType::TemperateGrassland, false, &mut rng);
        assert_eq!(coastal.elevation_zone, ElevationZone::Lowland);
        
        let lowland = gen.generate_cell(500.0, 30.0, BiomeType::TemperateGrassland, false, &mut rng);
        assert_eq!(lowland.elevation_zone, ElevationZone::Lowland);
        
        let highland = gen.generate_cell(2500.0, 30.0, BiomeType::MontaneForest, false, &mut rng);
        assert_eq!(highland.elevation_zone, ElevationZone::Highland);
        
        let alpine = gen.generate_cell(4000.0, 30.0, BiomeType::AlpineTundra, false, &mut rng);
        assert_eq!(alpine.elevation_zone, ElevationZone::Alpine);
    }
}