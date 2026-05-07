//! Climate Zone Calculator for World Factory
//!
//! Implements climate modeling based on latitude and elevation.
//! Computes temperature and moisture values for each polygon in the world.
//!
//! ## Temperature Calculation
//! - Base temperature from latitude (equator = hottest, poles = coldest)
//! - Lapse rate: temperature decreases ~6.5°C per 1000m elevation
//!
//! ## Precipitation Calculation
//! - Prevailing wind direction affects moisture patterns
//! - Orographic lift: moisture increases on windward slopes
//! - Rain shadow: leeward side of mountains is drier
//!
//! ## Rain Shadow Algorithm
//! 1. Determine prevailing wind direction (trade winds/easterlies in tropics, westerlies in temperate zones)
//! 2. For each polygon, check if it's on the leeward side of higher terrain
//! 3. Apply moisture penalty based on mountain height and distance

use serde::{Deserialize, Serialize};

use super::biome::ClimateZone;
use super::elevation::{Polygon, PolygonGraph};

/// Wind direction patterns based on latitude zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindDirection {
    /// Easterly trade winds (0-30° latitude)
    Easterly,
    /// Westerly winds (30-60° latitude)
    Westerly,
    /// Polar easterlies (60-90° latitude)
    PolarEasterly,
}

impl WindDirection {
    /// Get wind direction for a given latitude.
    pub fn from_latitude(latitude: f32) -> Self {
        let abs_lat = latitude.abs();
        if abs_lat < 30.0 {
            WindDirection::Easterly
        } else if abs_lat < 60.0 {
            WindDirection::Westerly
        } else {
            WindDirection::PolarEasterly
        }
    }

    /// Get the wind angle in radians (direction wind is coming FROM).
    /// 0 = north, π/2 = east, π = south, 3π/2 = west
    pub fn angle(&self) -> f32 {
        match self {
            WindDirection::Easterly => std::f32::consts::PI, // Wind from east (toward west)
            WindDirection::Westerly => 0.0,                  // Wind from west (toward east)
            WindDirection::PolarEasterly => std::f32::consts::PI, // Wind from poles
        }
    }
}

/// Configuration for climate zone calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateCalculatorConfig {
    /// Base temperature at equator (°C)
    pub base_equator_temp: f32,
    /// Temperature at poles (°C)
    pub pole_temp: f32,
    /// Temperature lapse rate (°C per 1000m)
    pub lapse_rate: f32,
    /// Elevation where temperature reaches 0°C (meters)
    pub freezing_line: f32,
    /// Maximum precipitation factor (multiplier for orographic effect)
    pub max_precip_factor: f32,
    /// Rain shadow intensity (0.0-1.0)
    pub rain_shadow_factor: f32,
    /// Maximum rain shadow distance (polygon hops)
    pub max_rain_shadow_distance: u32,
}

impl Default for ClimateCalculatorConfig {
    fn default() -> Self {
        Self {
            base_equator_temp: 30.0,
            pole_temp: -30.0,
            lapse_rate: -6.5,
            freezing_line: 2500.0,
            max_precip_factor: 2.0,
            rain_shadow_factor: 0.4,
            max_rain_shadow_distance: 15,
        }
    }
}

/// Result of climate calculation for a polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonClimate {
    /// Climate zone based on latitude
    pub zone: ClimateZone,
    /// Temperature value (0.0-1.0)
    pub temperature: f32,
    /// Moisture level (0.0-1.0)
    pub moisture: f32,
    /// Prevailing wind direction at this location
    pub wind_direction: WindDirection,
    /// Whether this polygon is in a rain shadow
    pub in_rain_shadow: bool,
    /// Temperature in Celsius (for debugging/display)
    pub temperature_celsius: f32,
}

/// Climate zone calculator for Voronoi polygon grids.
#[derive(Debug, Clone)]
pub struct ClimateCalculator {
    config: ClimateCalculatorConfig,
}

impl ClimateCalculator {
    /// Create a new climate calculator with default configuration.
    pub fn new() -> Self {
        Self::with_config(ClimateCalculatorConfig::default())
    }

    /// Create a climate calculator with custom configuration.
    pub fn with_config(config: ClimateCalculatorConfig) -> Self {
        Self { config }
    }

    /// Calculate climate for all polygons in the graph.
    ///
    /// This will populate temperature and moisture fields on each polygon.
    ///
    /// # Arguments
    /// * `graph` - The polygon graph to process (will be modified in place)
    /// * `latitude_fn` - Function that returns latitude for a polygon ID (0° = equator, 90° = poles)
    /// * `seed` - Random seed for deterministic generation
    pub fn calculate_climate<F>(&self, graph: &mut PolygonGraph, latitude_fn: F, seed: u64)
    where
        F: Fn(u32) -> f32 + Send + Sync,
    {
        // First pass: calculate base temperature and moisture for all polygons
        let polygon_climates: Vec<PolygonClimate> = (0..graph.len() as u32)
            .map(|id| {
                let lat = latitude_fn(id);
                self.calculate_polygon_climate_base(lat)
            })
            .collect();

        // Calculate rain shadow effects
        let rain_shadow_map =
            self.calculate_rain_shadows(graph, &polygon_climates, latitude_fn, seed);

        // Apply climate values to polygons
        for (id, climate) in polygon_climates.iter().enumerate() {
            let polygon = graph.get_mut(id as u32).unwrap();

            // Apply rain shadow effect to moisture if applicable
            let final_moisture = if rain_shadow_map[id] {
                climate.moisture * (1.0 - self.config.rain_shadow_factor)
            } else {
                climate.moisture
            };

            polygon.set_temperature(climate.temperature);
            polygon.set_moisture(final_moisture);
        }

        log::debug!(
            "Climate calculation complete: {} polygons processed",
            graph.len()
        );
    }

    /// Calculate base climate for a single polygon (without rain shadow effects).
    fn calculate_polygon_climate_base(&self, latitude: f32) -> PolygonClimate {
        let abs_lat = latitude.abs();

        // Determine climate zone
        let zone = if abs_lat < 23.5 {
            ClimateZone::Tropical
        } else if abs_lat < 35.0 {
            ClimateZone::Subtropical
        } else if abs_lat < 55.0 {
            ClimateZone::Temperate
        } else if abs_lat < 65.0 {
            ClimateZone::Boreal
        } else {
            ClimateZone::Polar
        };

        // Calculate base temperature from latitude (no elevation yet)
        // Linear interpolation from equator to pole
        let lat_factor = abs_lat / 90.0;
        let base_temp = self.config.base_equator_temp
            + (self.config.pole_temp - self.config.base_equator_temp) * lat_factor;

        // Temperature in Celsius (elevation will be added later in full calculation)
        let temp_celsius = base_temp;

        // Convert to 0-1 range
        // We map: pole_temp -> 0.0, equator_temp -> 1.0
        let temp_range = self.config.base_equator_temp - self.config.pole_temp;
        let temperature = ((temp_celsius - self.config.pole_temp) / temp_range).clamp(0.0, 1.0);

        // Base moisture from climate zone (will be modified by elevation and terrain)
        let base_moisture = self.base_moisture_for_zone(&zone);

        let wind_direction = WindDirection::from_latitude(latitude);

        PolygonClimate {
            zone,
            temperature,
            moisture: base_moisture,
            wind_direction,
            in_rain_shadow: false,
            temperature_celsius: temp_celsius,
        }
    }

    /// Get base moisture level for a climate zone.
    fn base_moisture_for_zone(&self, zone: &ClimateZone) -> f32 {
        // Approximate global precipitation patterns
        // Higher values = more precipitation
        match zone {
            ClimateZone::Tropical => 0.7,    // High precipitation in tropics
            ClimateZone::Subtropical => 0.5, // Variable: deserts at 0.3, humid at 0.7
            ClimateZone::Temperate => 0.6,   // Moderate precipitation
            ClimateZone::Boreal => 0.5,      // Moderate, with seasonal variation
            ClimateZone::Polar => 0.3,       // Low precipitation (cold deserts)
        }
    }

    /// Calculate rain shadow effects by checking if polygons are in the wind shadow of mountains.
    fn calculate_rain_shadows<F>(
        &self,
        graph: &PolygonGraph,
        climates: &[PolygonClimate],
        latitude_fn: F,
        seed: u64,
    ) -> Vec<bool>
    where
        F: Fn(u32) -> f32,
    {
        let n = graph.len();
        let mut rain_shadow = vec![false; n];

        // Simple seeded random for adding variation to rain shadow effects
        let mut rng = SimpleRng::new(seed);

        for i in 0..n {
            let polygon = match graph.get(i as u32) {
                Some(p) => p,
                None => continue,
            };

            let climate = &climates[i];
            let wind_angle = climate.wind_direction.angle();

            // Calculate wind direction vector (normalized)
            let wind_dx = wind_angle.cos();
            let wind_dy = wind_angle.sin();

            // Check for mountains in the wind direction (upwind)
            let mut mountain_height = 0.0f32;
            let mut distance_to_mountain = 0u32;

            // Sample points upwind from this polygon
            let centroid = self.get_polygon_centroid_estimate(polygon, i as u32, &latitude_fn);

            for dist in 1..=self.config.max_rain_shadow_distance {
                // Calculate sample position (moving upwind)
                let _sample_x = centroid.0 - wind_dx * (dist as f32 * 0.1);
                let _sample_y = centroid.1 - wind_dy * (dist as f32 * 0.1);

                // Find polygon near this position (simplified: check by distance)
                // In a full implementation, we'd use spatial indexing
                for j in 0..n {
                    if i == j {
                        continue;
                    }

                    let other = match graph.get(j as u32) {
                        Some(p) => p,
                        None => continue,
                    };

                    // Check if this polygon is upwind of current and is a mountain
                    let other_centroid =
                        self.get_polygon_centroid_estimate(other, j as u32, &latitude_fn);

                    // Direction from other to current
                    let dx = centroid.0 - other_centroid.0;
                    let dy = centroid.1 - other_centroid.1;
                    let dist_to_current = (dx * dx + dy * dy).sqrt();

                    if dist_to_current < 0.01 {
                        continue;
                    }

                    // Check if other is roughly upwind of current (dot product > 0)
                    let dot = (dx / dist_to_current) * wind_dx + (dy / dist_to_current) * wind_dy;

                    // Check if this polygon is higher and roughly in the wind direction
                    let elevation_diff = other.elevation - polygon.elevation;
                    let is_upwind = dot > 0.5; // Other is upwind of current

                    if is_upwind && elevation_diff > 0.1 {
                        // Add some randomness based on seed
                        let variation = rng.next_float() * 0.2 + 0.8;

                        if elevation_diff * variation > mountain_height {
                            mountain_height = elevation_diff;
                            distance_to_mountain = dist;
                        }
                    }
                }
            }

            // Apply rain shadow if there's a significant mountain upwind
            // Effect decreases with distance from mountain
            if mountain_height > 0.2 && distance_to_mountain < self.config.max_rain_shadow_distance
            {
                let shadow_strength = mountain_height
                    * (1.0
                        - distance_to_mountain as f32
                            / self.config.max_rain_shadow_distance as f32);
                rain_shadow[i] = shadow_strength > 0.3;
            }
        }

        rain_shadow
    }

    /// Estimate centroid position for a polygon based on ID.
    /// This is a simplified version - a full implementation would store centroid coordinates.
    fn get_polygon_centroid_estimate<F>(
        &self,
        _polygon: &Polygon,
        id: u32,
        latitude_fn: F,
    ) -> (f32, f32)
    where
        F: Fn(u32) -> f32,
    {
        // Use polygon ID as a seed for position estimation
        // In a real implementation, we'd store centroid coordinates in the Polygon struct
        let lat = latitude_fn(id);
        let lon = (id as f32 * 0.1) % 360.0 - 180.0;
        (lon, lat) // (x, y) = (longitude, latitude)
    }

    /// Calculate temperature for a polygon with elevation consideration.
    ///
    /// Uses formula: temperature = base_temp - (elevation / 1000) * lapse_rate
    /// Converted to 0-1 range internally.
    pub fn calculate_temperature(&self, latitude: f32, elevation: f32) -> f32 {
        let abs_lat = latitude.abs();

        // Base temperature from latitude (°C)
        let lat_factor = abs_lat / 90.0;
        let base_temp = self.config.base_equator_temp
            + (self.config.pole_temp - self.config.base_equator_temp) * lat_factor;

        // Elevation effect (lapse rate)
        let elevation_temp = base_temp + (elevation / 1000.0) * self.config.lapse_rate;

        // Convert to 0-1 range
        let temp_range = self.config.base_equator_temp - self.config.pole_temp;
        ((elevation_temp - self.config.pole_temp) / temp_range).clamp(0.0, 1.0)
    }

    /// Calculate moisture based on various factors.
    ///
    /// Factors include:
    /// - Base moisture for climate zone
    /// - Elevation (orographic effect)
    /// - Distance from coast
    /// - Rain shadow from nearby mountains
    pub fn calculate_moisture(
        &self,
        latitude: f32,
        elevation: f32,
        distance_from_coast: f32,
        in_rain_shadow: bool,
    ) -> f32 {
        let abs_lat = latitude.abs();

        // Determine climate zone for base moisture
        let base_moisture = if abs_lat < 23.5 {
            0.7
        } else if abs_lat < 35.0 {
            0.5
        } else if abs_lat < 55.0 {
            0.6
        } else if abs_lat < 65.0 {
            0.5
        } else {
            0.3
        };

        // Orographic effect: higher elevation catches more moisture (up to a point)
        let orographic = (elevation / 3000.0).min(1.0) * 0.3;

        // Coastal effect: closer to coast = more moisture
        let coastal_effect = (-distance_from_coast * 0.5).exp() * 0.2;

        // Calculate base moisture
        let moisture = base_moisture + orographic + coastal_effect;

        // Apply rain shadow penalty
        let final_moisture = if in_rain_shadow {
            moisture * (1.0 - self.config.rain_shadow_factor)
        } else {
            moisture
        };

        final_moisture.clamp(0.0, 1.0)
    }
}

/// Simple deterministic RNG for reproducible rain shadow calculations.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407),
        }
    }

    fn next_float(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Extract a float from the upper bits for better distribution
        ((self.state >> 33) as f32) / (u32::MAX >> 9) as f32
    }
}

impl Default for ClimateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_at_equator() {
        let calc = ClimateCalculator::new();

        // At equator, sea level should be hot
        let temp = calc.calculate_temperature(0.0, 0.0);
        assert!(temp > 0.8, "Equator should be hot: {}", temp);
    }

    #[test]
    fn test_temperature_at_poles() {
        let calc = ClimateCalculator::new();

        // At poles, sea level should be cold
        let temp = calc.calculate_temperature(90.0, 0.0);
        assert!(temp < 0.2, "Poles should be cold: {}", temp);
    }

    #[test]
    fn test_elevation_cools_temperature() {
        let calc = ClimateCalculator::new();

        // Same latitude, different elevations
        let temp_low = calc.calculate_temperature(45.0, 0.0);
        let temp_high = calc.calculate_temperature(45.0, 4000.0);

        assert!(temp_high < temp_low, "Higher elevation should be colder");
    }

    #[test]
    fn test_wind_direction_zones() {
        // Trade winds in tropics
        assert!(matches!(
            WindDirection::from_latitude(15.0),
            WindDirection::Easterly
        ));

        // Westerlies in temperate
        assert!(matches!(
            WindDirection::from_latitude(45.0),
            WindDirection::Westerly
        ));

        // Polar easterlies
        assert!(matches!(
            WindDirection::from_latitude(75.0),
            WindDirection::PolarEasterly
        ));
    }

    #[test]
    fn test_moisture_coastal_effect() {
        let calc = ClimateCalculator::new();

        // Same latitude, different distances from coast
        let moist_near = calc.calculate_moisture(45.0, 500.0, 0.0, false);
        let moist_far = calc.calculate_moisture(45.0, 500.0, 5.0, false);

        assert!(
            moist_near > moist_far,
            "Closer to coast should be more moist"
        );
    }

    #[test]
    fn test_rain_shadow_effect() {
        let calc = ClimateCalculator::new();

        // With and without rain shadow
        let moist_shadow = calc.calculate_moisture(45.0, 500.0, 1.0, true);
        let moist_clear = calc.calculate_moisture(45.0, 500.0, 1.0, false);

        assert!(
            moist_shadow < moist_clear,
            "Rain shadow should reduce moisture"
        );
    }

    #[test]
    fn test_deterministic_temperature() {
        let calc = ClimateCalculator::new();

        let temp1 = calc.calculate_temperature(30.0, 1000.0);
        let temp2 = calc.calculate_temperature(30.0, 1000.0);

        assert_eq!(
            temp1, temp2,
            "Temperature calculation should be deterministic"
        );
    }

    #[test]
    fn test_polygon_climate_zones() {
        let calc = ClimateCalculator::new();

        // Test various latitudes
        let tropical = calc.calculate_polygon_climate_base(10.0);
        assert!(matches!(tropical.zone, ClimateZone::Tropical));

        let subtropical = calc.calculate_polygon_climate_base(28.0);
        assert!(matches!(subtropical.zone, ClimateZone::Subtropical));

        let temperate = calc.calculate_polygon_climate_base(45.0);
        assert!(matches!(temperate.zone, ClimateZone::Temperate));

        let boreal = calc.calculate_polygon_climate_base(60.0);
        assert!(matches!(boreal.zone, ClimateZone::Boreal));

        let polar = calc.calculate_polygon_climate_base(80.0);
        assert!(matches!(polar.zone, ClimateZone::Polar));
    }

    #[test]
    fn test_temperature_range_bounded() {
        let calc = ClimateCalculator::new();

        // Extreme values should still be bounded
        let temp_equator_high = calc.calculate_temperature(0.0, 8000.0); // Mount Everest altitude
        let temp_pole_low = calc.calculate_temperature(90.0, 0.0);

        assert!(temp_equator_high >= 0.0 && temp_equator_high <= 1.0);
        assert!(temp_pole_low >= 0.0 && temp_pole_low <= 1.0);
    }
}
