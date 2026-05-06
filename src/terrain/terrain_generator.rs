//! Terrain Generator - procedural terrain generation
//!
//! Uses multi-octave noise for elevation, with optional plate tectonics simulation.

use super::{
    BiomeAssignmentMatrix, ElevationGrid, ErosionConfig, ErosionSimulator, TectonicSimConfig,
    TectonicSimulator, TerrainCell, TerrainGrid,
};
use crate::util::noise::SimplexNoise;
use serde::{Deserialize, Serialize};

/// Configuration for terrain generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// World seed for deterministic generation.
    pub seed: u64,
    /// World width in cells.
    pub width: u32,
    /// World height in cells.
    pub height: u32,
    /// Cell size in meters.
    pub cell_size: f32,
    /// Number of noise octaves for base terrain.
    pub octaves: usize,
    /// Base elevation in meters.
    pub base_elevation: f32,
    /// Mountain amplitude.
    pub mountain_amplitude: f32,
    /// Sea level threshold in meters.
    pub sea_level: f32,
    /// Enable plate tectonics simulation.
    pub enable_tectonics: bool,
    /// Tectonic activity level (0.0-1.0).
    pub tectonic_activity: f32,
    /// Number of Lloyd relaxation iterations for Voronoi cells (Phase 1).
    /// 0 = no relaxation, 2 = standard, 5 = maximum (default: 2)
    pub lloyd_iterations: u32,
    /// Enable erosion simulation (hydraulic + thermal).
    pub enable_erosion: Option<bool>,
    /// Number of erosion iterations. Default: 100,000
    pub erosion_iterations: Option<usize>,
    /// Erosion strength (0.0-1.0). Default: 0.3
    pub erosion_strength: Option<f32>,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            width: 256,
            height: 256,
            cell_size: 1000.0, // 1km cells
            octaves: 6,
            base_elevation: 500.0,
            mountain_amplitude: 2000.0,
            sea_level: 0.0,
            enable_tectonics: true,
            tectonic_activity: 0.5,
            lloyd_iterations: 2, // Standard Lloyd relaxation (Phase 1)
            enable_erosion: None,
            erosion_iterations: None,
            erosion_strength: None,
        }
    }
}

/// Terrain generation layer options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainLayer {
    /// Base elevation only.
    Base,
    /// With mountain ranges from tectonics.
    Mountains,
    /// With rivers and valleys from erosion.
    Eroded,
    /// Full generation with biomes.
    Full,
}

/// Main terrain generator.
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    config: TerrainConfig,
    noise: SimplexNoise,
    biome_matrix: BiomeAssignmentMatrix,
    tectonic_sim: Option<TectonicSimulator>,
    tectonic_result: Option<super::TectonicResult>,
}

impl TerrainGenerator {
    /// Create a new terrain generator.
    pub fn new(config: TerrainConfig) -> Self {
        let tectonic_sim = if config.enable_tectonics {
            let sim_config = TectonicSimConfig {
                plate_count: 7,
                seed: config.seed.wrapping_add(0x54454354), // "TECT" in hex
                width: config.width,
                height: config.height,
                activity: config.tectonic_activity,
                enable_drift: false,
                continental_ratio: 0.35,
            };
            Some(TectonicSimulator::new(sim_config))
        } else {
            None
        };

        Self {
            noise: SimplexNoise::new(config.seed),
            biome_matrix: BiomeAssignmentMatrix::new(),
            config,
            tectonic_sim,
            tectonic_result: None,
        }
    }

    /// Generate terrain grid with specified detail level.
    ///
    /// Note: When using `TerrainLayer::Mountains` or higher with tectonics enabled,
    /// tectonic simulation is run automatically. Call `simulate_tectonics()` first
    /// if you need access to tectonic results (plates, boundaries) separately.
    pub fn generate(&mut self, layer: TerrainLayer) -> TerrainGrid {
        let mut grid = TerrainGrid::new(self.config.width, self.config.height);
        grid.initialize();

        self.generate_into_grid(layer, &mut grid);

        grid
    }

    /// Generate terrain directly into an ElevationGrid.
    ///
    /// This is more efficient when you only need elevation data (e.g., for river generation)
    /// and don't need the full TerrainGrid with biome and moisture data.
    pub fn generate_elevation_grid(&mut self) -> ElevationGrid {
        let (width, height) = (self.config.width as usize, self.config.height as usize);
        let mut elevation_grid = ElevationGrid::new(width, height, 0.0);

        // Run tectonic simulation if needed
        if self.tectonic_sim.is_some() && self.tectonic_result.is_none() {
            self.simulate_tectonics();
        }

        // Generate base layer into elevation grid
        let noise = &self.noise;

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32;
                let ny = y as f32 / height as f32;

                // Multi-octave noise for natural terrain
                let elevation =
                    self.octave_noise_2d(noise, nx * 4.0, ny * 4.0, self.config.octaves);

                // Scale to elevation range and add base
                let height_m =
                    self.config.base_elevation + elevation * self.config.mountain_amplitude;

                // Normalize to [0.0, 1.0] range (0-2500m normalized to 0-1)
                let normalized = (height_m / 2500.0).clamp(0.0, 1.0) as f32;
                elevation_grid.set(x, y, normalized);
            }
        }

        // Apply mountain layer if we have tectonic results
        if let Some(ref result) = self.tectonic_result {
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    if idx < result.elevation_modifiers.len() {
                        let modifier = result.elevation_modifiers[idx];
                        // Apply tectonic modifier to normalized elevation
                        let current = elevation_grid.get(x, y).unwrap_or(0.0);
                        // Modifier is in meters, convert to normalized range
                        let normalized_modifier = (modifier / 2500.0).clamp(-0.5, 0.5) as f32;
                        elevation_grid.set(x, y, (current + normalized_modifier).clamp(0.0, 1.0));
                    }
                }
            }
        }

        // Apply erosion if enabled (Phase 1 feature)
        if self.config.enable_erosion.unwrap_or(true) {
            log::debug!("TerrainGenerator: applying erosion simulation to elevation grid");
            self.apply_erosion_to_elevation_grid(&mut elevation_grid);
        }

        elevation_grid
    }

    /// Apply erosion simulation to an ElevationGrid.
    ///
    /// This modifies the elevation grid in-place to add realistic valleys
    /// and water-carved features. Used primarily for river generation.
    fn apply_erosion_to_elevation_grid(&self, elevation_grid: &mut ElevationGrid) {
        let (width, height) = (self.config.width as usize, self.config.height as usize);

        // Configure erosion
        let erosion_config = ErosionConfig {
            seed: self.config.seed,
            iterations: self.config.erosion_iterations.unwrap_or(100_000),
            erosion_strength: self.config.erosion_strength.unwrap_or(0.3),
            deposition_rate: 0.3,
            evaporation_rate: 0.01,
            sediment_capacity: 4.0,
            min_slope: 0.01,
            thermal_weathering: true,
            thermal_iterations: 50,
            max_erosion_depth: 2.0,
            inertia: 0.05,
            initial_water: 1.0,
        };

        let erosion = ErosionSimulator::new(erosion_config);

        // Convert ElevationGrid to TerrainGrid for erosion
        let mut terrain_grid = TerrainGrid::new(width as u32, height as u32);
        terrain_grid.initialize();

        // Copy elevation data
        for y in 0..height {
            for x in 0..width {
                if let Some(elev) = elevation_grid.get(x, y) {
                    if let Some(cell) = terrain_grid.get(x as u32, y as u32) {
                        let mut mutable_cell = cell;
                        // Convert normalized elevation to meters for the cell
                        let height_m = elev * 2500.0;
                        mutable_cell.set_height(height_m);
                        terrain_grid.set(x as u32, y as u32, mutable_cell);
                    }
                }
            }
        }

        // Apply erosion
        erosion.apply(&mut terrain_grid);

        // Copy back to elevation grid
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = terrain_grid.get(x as u32, y as u32) {
                    let height_m = cell.height();
                    let normalized = (height_m / 2500.0).clamp(0.0, 1.0) as f32;
                    elevation_grid.set(x, y, normalized);
                }
            }
        }

        log::debug!("Erosion applied to elevation grid");
    }

    /// Generate terrain into an existing TerrainGrid.
    /// Used internally by both `generate()` and `generate_elevation_grid()`.
    fn generate_into_grid(&mut self, layer: TerrainLayer, grid: &mut TerrainGrid) {
        // Run tectonic simulation if mountains or higher layer requested
        if matches!(
            layer,
            TerrainLayer::Mountains | TerrainLayer::Eroded | TerrainLayer::Full
        ) {
            if self.tectonic_sim.is_some() && self.tectonic_result.is_none() {
                self.simulate_tectonics();
            }
        }

        match layer {
            TerrainLayer::Base => self.generate_base_layer(grid),
            TerrainLayer::Mountains => {
                self.generate_base_layer(grid);
                self.generate_mountain_layer(grid);
            }
            TerrainLayer::Eroded => {
                self.generate_base_layer(grid);
                self.generate_mountain_layer(grid);
                self.apply_erosion(grid);
            }
            TerrainLayer::Full => {
                self.generate_base_layer(grid);
                self.generate_mountain_layer(grid);
                self.apply_erosion(grid);
                self.assign_biomes(grid);
            }
        }
    }

    /// Run tectonic plate simulation and get results.
    ///
    /// Call this before `generate()` with `TerrainLayer::Mountains` or higher
    /// if you need access to tectonic data (plates, boundaries).
    pub fn simulate_tectonics(&mut self) -> Option<&super::TectonicResult> {
        if let Some(ref mut sim) = self.tectonic_sim {
            let result = sim.simulate();
            self.tectonic_result = Some(result);
            self.tectonic_result.as_ref()
        } else {
            None
        }
    }

    /// Get the tectonic simulation result.
    ///
    /// Returns `None` if tectonics is disabled or `simulate_tectonics()`
    /// hasn't been called yet.
    pub fn get_tectonic_result(&self) -> Option<&super::TectonicResult> {
        self.tectonic_result.as_ref()
    }

    /// Generate base elevation layer using multi-octave simplex noise.
    fn generate_base_layer(&self, grid: &mut TerrainGrid) {
        let (width, height) = grid.dimensions();
        let noise = &self.noise;

        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32;
                let ny = y as f32 / height as f32;

                // Multi-octave noise for natural terrain
                let elevation =
                    self.octave_noise_2d(noise, nx * 4.0, ny * 4.0, self.config.octaves);

                // Scale to elevation range and add base
                let height_m =
                    self.config.base_elevation + elevation * self.config.mountain_amplitude;

                let cell = TerrainCell::new(height_m, 0, 0, height_m < self.config.sea_level);
                grid.set(x, y, cell);
            }
        }
    }

    /// Generate mountain ranges using plate tectonics simulation.
    fn generate_mountain_layer(&self, grid: &mut TerrainGrid) {
        let (width, height) = grid.dimensions();

        // If we have tectonic simulation results, use them for elevation
        if let Some(ref result) = self.tectonic_result {
            // Apply elevation modifiers from tectonic simulation
            let total_cells = width * height;
            for cell_id in 0..total_cells {
                let idx = cell_id as usize;
                if idx < result.elevation_modifiers.len() {
                    let x = cell_id % width;
                    let y = cell_id / width;

                    if let Some(cell) = self.get_cell(grid, x, y) {
                        let modifier = result.elevation_modifiers[idx];
                        let current_height = cell.height();
                        let mut new_cell = cell;
                        new_cell.set_height(current_height + modifier);
                        // Update is_water flag since height changed
                        new_cell.set_water(current_height + modifier < self.config.sea_level);
                        grid.set(x, y, new_cell);
                    }
                }
            }
        } else if self.config.enable_tectonics {
            // Fallback: use simple noise-based boundary detection
            let noise = &self.noise;
            let activity = self.config.tectonic_activity;

            for y in 0..height {
                for x in 0..width {
                    let nx = x as f64 / width as f64;
                    let ny = y as f64 / height as f64;

                    // Plate boundary noise
                    let boundary = noise.get_billow(nx * 2.0, ny * 2.0, 4);

                    if boundary > 0.7 {
                        // Near plate boundary - uplift mountains
                        let boundary_f32 = boundary as f32;
                        let activity_f32 = activity as f32;
                        let uplift = ((boundary_f32 - 0.7) * 3.33 * activity_f32 * 3000.0) as f64;

                        if let Some(cell) = self.get_cell(grid, x, y) {
                            let current_height = cell.height();
                            let mut new_cell = cell;
                            new_cell.set_height(current_height + uplift as f32);
                            grid.set(x, y, new_cell);
                        }
                    }
                }
            }
        }
        // If tectonics disabled, no mountain layer applied
    }

    /// Apply erosion simulation for realistic valleys.
    ///
    /// Uses droplet-based hydraulic erosion to create natural river valleys
    /// and water-carved terrain features.
    fn apply_erosion(&self, grid: &mut TerrainGrid) {
        // Use proper droplet-based hydraulic erosion
        let erosion_config = ErosionConfig {
            seed: self.config.seed,
            iterations: self.config.erosion_iterations.unwrap_or(100_000),
            erosion_strength: self.config.erosion_strength.unwrap_or(0.3),
            deposition_rate: 0.3,
            evaporation_rate: 0.01,
            sediment_capacity: 4.0,
            min_slope: 0.01,
            thermal_weathering: true,
            thermal_iterations: 50,
            max_erosion_depth: 2.0,
            inertia: 0.05,
            initial_water: 1.0,
        };

        let erosion = ErosionSimulator::new(erosion_config);
        erosion.apply(grid);

        log::debug!("Erosion simulation complete");
    }

    /// Assign biomes to all terrain cells.
    fn assign_biomes(&self, grid: &mut TerrainGrid) {
        let (width, height) = grid.dimensions();

        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = self.get_cell(grid, x, y) {
                    let height_m = cell.height();

                    // Calculate latitude from y coordinate (assuming polar origin)
                    let latitude = (y as f32 / height as f32) * 90.0;

                    // Calculate temperature from latitude and elevation
                    let temperature = self.estimate_temperature(latitude, height_m);

                    // Calculate precipitation (simplified - would be more complex in full sim)
                    let precipitation = self.estimate_precipitation(x, y, width, height, height_m);

                    // Assign biome
                    let assignment =
                        self.biome_matrix
                            .assign(height_m, latitude, precipitation, temperature);

                    let mut new_cell = cell;
                    new_cell.set_biome(assignment.biome as u8);
                    new_cell.set_moisture(
                        self.moisture_to_index(
                            assignment
                                .factors
                                .iter()
                                .find(|f| f.name == "moisture_level")
                                .map(|f| &f.value)
                                .unwrap_or(&"SubHumid".to_string()),
                        ),
                    );

                    // Update is_water flag based on current height
                    new_cell.set_water(height_m < self.config.sea_level);

                    grid.set(x, y, new_cell);
                }
            }
        }
    }

    /// Estimate temperature based on latitude and elevation.
    fn estimate_temperature(&self, latitude: f32, elevation: f32) -> f32 {
        // Lapse rate: -6.5°C per 1000m elevation
        // Base temperature by latitude (simplified)
        let base_temp = 30.0 - latitude * 0.6; // 30°C at equator, ~-24°C at poles
        let lapse = (elevation / 1000.0) * -6.5;
        (base_temp + lapse).max(-50.0).min(50.0)
    }

    /// Estimate precipitation based on position and elevation.
    fn estimate_precipitation(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        elevation: f32,
    ) -> f32 {
        let noise = &self.noise;
        let nx = x as f64 / width as f64;
        let ny = y as f64 / height as f64;

        // Base precipitation from noise
        let base_precip = noise.get(nx * 2.0, ny * 2.0);

        // Orographic effect: more precipitation on windward slopes
        // Simplified: higher elevation = more precipitation
        let orographic = ((elevation / 2000.0).min(1.0) * 500.0) as f64;

        // Scale to mm/year (0-4000mm range)
        let precip: f64 = (base_precip * 0.5 + 0.5) * 2000.0 + orographic;
        (precip.max(0.0).min(4000.0)) as f32
    }

    /// Convert moisture string to index.
    fn moisture_to_index(&self, moisture: &str) -> u8 {
        match moisture {
            "HyperArid" => 0,
            "Arid" => 1,
            "SemiArid" => 2,
            "SubHumid" => 3,
            "Humid" => 4,
            "PerHumid" => 5,
            _ => 3,
        }
    }

    /// Get mutable cell reference.
    fn get_cell(&self, grid: &TerrainGrid, x: u32, y: u32) -> Option<TerrainCell> {
        let (width, height) = grid.dimensions();
        if x >= width || y >= height {
            return None;
        }
        grid.get(x, y)
    }

    /// Multi-octave 2D noise.
    fn octave_noise_2d(&self, noise: &SimplexNoise, x: f32, y: f32, octaves: usize) -> f32 {
        let mut value = 0.0f32;
        let mut amplitude = 1.0f32;
        let mut frequency = 1.0f32;
        let mut max_value = 0.0f32;

        for _ in 0..octaves {
            value += noise.get_f32(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value / max_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generation() {
        let config = TerrainConfig {
            seed: 42,
            width: 64,
            height: 64,
            ..Default::default()
        };

        let mut generator = TerrainGenerator::new(config);
        let grid = generator.generate(TerrainLayer::Full);

        let (width, height) = grid.dimensions();
        assert_eq!(width, 64);
        assert_eq!(height, 64);

        // Verify cells have valid data
        let sample = grid.get(32, 32);
        assert!(sample.is_some());
    }

    #[test]
    fn test_deterministic_generation() {
        let config = TerrainConfig {
            seed: 12345,
            width: 32,
            height: 32,
            ..Default::default()
        };

        let mut gen1 = TerrainGenerator::new(config.clone());
        let mut gen2 = TerrainGenerator::new(config.clone());

        let grid1 = gen1.generate(TerrainLayer::Base);
        let grid2 = gen2.generate(TerrainLayer::Base);

        // Compare a few cells
        for i in 0..10 {
            let c1 = grid1.get(i, i);
            let c2 = grid2.get(i, i);
            assert_eq!(c1.map(|c| c.height()), c2.map(|c| c.height()));
        }
    }

    #[test]
    fn test_temperature_estimation() {
        let config = TerrainConfig::default();
        let mut generator = TerrainGenerator::new(config);

        // Equator, sea level
        let temp1 = generator.estimate_temperature(0.0, 0.0);
        assert!(temp1 > 25.0);

        // Pole
        let temp2 = generator.estimate_temperature(90.0, 0.0);
        assert!(temp2 < 0.0);

        // High altitude
        let temp3 = generator.estimate_temperature(45.0, 4000.0);
        assert!(temp3 < generator.estimate_temperature(45.0, 0.0));
    }

    #[test]
    fn test_tectonic_integration() {
        let config = TerrainConfig {
            seed: 42,
            width: 64,
            height: 64,
            enable_tectonics: true,
            tectonic_activity: 0.7,
            ..Default::default()
        };

        let mut generator = TerrainGenerator::new(config);

        // Run tectonic simulation explicitly
        let result = generator.simulate_tectonics();
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(!result.plates.is_empty());
        assert!(!result.elevation_modifiers.is_empty());

        // Generate terrain - should use tectonic results
        let grid = generator.generate(TerrainLayer::Mountains);

        // Verify grid was generated
        let (width, height) = grid.dimensions();
        assert_eq!(width, 64);
        assert_eq!(height, 64);
    }

    #[test]
    fn test_tectonic_result_access() {
        let config = TerrainConfig {
            seed: 99,
            width: 32,
            height: 32,
            enable_tectonics: true,
            tectonic_activity: 0.5,
            ..Default::default()
        };

        let mut generator = TerrainGenerator::new(config);

        // Generate with mountains (auto-runs tectonics)
        let grid = generator.generate(TerrainLayer::Mountains);

        // Should be able to access results after generation
        let result = generator.get_tectonic_result();
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.plates.len(), 7); // default plate count

        // Verify all cells assigned to plates
        let total_cells = 32 * 32;
        assert_eq!(result.cell_to_plate.len(), total_cells);
    }
}
