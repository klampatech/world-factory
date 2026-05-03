//! Erosion simulation for terrain generation
//! 
//! Implements droplet-based hydraulic erosion and thermal weathering.
//! 
//! Based on the papers:
//! - "Fast Hydraulic Erosion Simulation and Visualization" (GPU Gems)
//! - "Modeling and Rendering Realistic Terrains" (Praeknie & Zaugg)
//! 
//! # Algorithm
//! 
//! Water droplets travel downhill following the gradient. As they move:
//! 1. **Erosion**: Pick up sediment from terrain based on speed and water volume
//! 2. **Transport**: Carry sediment downstream
//! 3. **Deposition**: Deposit sediment when speed decreases or terrain rises
//! 
//! Key parameters control:
//! - Erosion rate (how fast sediment is picked up)
//! - Deposition rate (how fast sediment settles)
//! - Evaporation (water volume decreases over time)
//! - Sediment capacity (max sediment based on speed and water)

use serde::{Deserialize, Serialize};
use super::TerrainGrid;
use crate::util::noise::SimplexNoise;

/// Erosion configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErosionConfig {
    /// Random seed for deterministic erosion patterns.
    pub seed: u64,
    /// Number of erosion iterations (droplets).
    pub iterations: usize,
    /// Erosion strength multiplier (0.0-1.0).
    pub erosion_strength: f32,
    /// Deposition rate (0.0-1.0).
    pub deposition_rate: f32,
    /// Evaporation rate per step (0.0-1.0).
    pub evaporation_rate: f32,
    /// Maximum sediment capacity multiplier.
    pub sediment_capacity: f32,
    /// Minimum slope for erosion (steepness threshold).
    pub min_slope: f32,
    /// Enable thermal weathering.
    pub thermal_weathering: bool,
    /// Thermal weathering iterations.
    pub thermal_iterations: usize,
    /// Maximum erosion depth per droplet (meters).
    pub max_erosion_depth: f32,
    /// Inertia factor (0.0-1.0) - how much previous direction affects new direction.
    pub inertia: f32,
    /// Starting water volume for droplets.
    pub initial_water: f32,
}

impl Default for ErosionConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            iterations: 1_000_000, // 1M droplets for a 256x256 grid
            erosion_strength: 0.3,
            deposition_rate: 0.3,
            evaporation_rate: 0.01,
            sediment_capacity: 4.0,
            min_slope: 0.01,
            thermal_weathering: true,
            thermal_iterations: 50,
            max_erosion_depth: 2.0,
            inertia: 0.05,
            initial_water: 1.0,
        }
    }
}

/// Erosion simulator using droplet-based hydraulic erosion.
#[derive(Debug, Clone)]
pub struct ErosionSimulator {
    config: ErosionConfig,
    noise: SimplexNoise,
}

impl ErosionSimulator {
    /// Create a new erosion simulator.
    pub fn new(config: ErosionConfig) -> Self {
        Self {
            noise: SimplexNoise::new(config.seed),
            config,
        }
    }
    
    /// Apply erosion to terrain grid.
    /// 
    /// This runs hydraulic erosion with water droplets, followed by
    /// optional thermal weathering.
    pub fn apply(&self, grid: &mut TerrainGrid) {
        // Calculate iterations per cell for good coverage
        let (width, height) = grid.dimensions();
        let total_cells = (width * height) as usize;
        let iterations = self.config.iterations;
        
        // Scale iterations if needed for good coverage
        // Target: ~4-10 droplets per cell for realistic results
        let target_coverage = 8;
        let required_iterations = total_cells * target_coverage;
        let actual_iterations = iterations.max(required_iterations);
        
        log::info!("Erosion: running {} iterations on {}x{} grid", 
                   actual_iterations, width, height);
        
        // Run hydraulic erosion with droplets
        for i in 0..actual_iterations {
            self.erode_droplet(grid, width, height, i);
            
            // Progress logging
            if i > 0 && i % 100_000 == 0 {
                log::debug!("Erosion progress: {}/{} iterations", i, actual_iterations);
            }
        }
        
        // Apply thermal weathering if enabled
        if self.config.thermal_weathering {
            log::info!("Erosion: applying thermal weathering ({} iterations)", 
                      self.config.thermal_iterations);
            self.thermal_weathering(grid, width, height);
        }
        
        log::info!("Erosion complete");
    }
    
    /// Simulate a single water droplet eroding terrain.
    fn erode_droplet(&self, grid: &mut TerrainGrid, width: u32, height: u32, seed: usize) {
        // Initialize droplet at random position
        let mut pos_x = self.noise.get_float(seed as f32 * 0.1) * width as f32;
        let mut pos_y = self.noise.get_float(seed as f32 * 0.2) * height as f32;
        
        // Droplet state
        let mut sediment: f32 = 0.0;
        let mut water = self.config.initial_water;
        
        // Direction of travel (normalized)
        let mut dir_x = 0.0f32;
        let mut dir_y = 0.0f32;
        
        // Droplet lifetime - max steps before it disappears
        let max_steps = 256;
        
        for _step in 0..max_steps {
            // Get cell coordinates
            let cell_x = pos_x as i32;
            let cell_y = pos_y as i32;
            
            // Check if droplet is out of bounds
            if cell_x < 1 || cell_x >= width as i32 - 1 || 
               cell_y < 1 || cell_y >= height as i32 - 1 {
                break;
            }
            
            let x = cell_x as u32;
            let y = cell_y as u32;
            
            // Get terrain height at current position (bilinear interpolation)
            let height_at_pos = self.interpolate_height(grid, pos_x, pos_y, width, height);
            
            // Calculate gradient at current position
            let (grad_x, grad_y) = self.calculate_gradient(grid, x, y, width, height);
            
            // Update direction with inertia
            dir_x = dir_x * self.config.inertia - grad_x * (1.0 - self.config.inertia);
            dir_y = dir_y * self.config.inertia - grad_y * (1.0 - self.config.inertia);
            
            // Normalize direction
            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            if len > 0.001 {
                dir_x /= len;
                dir_y /= len;
            } else {
                // Random direction if nearly stationary
                let angle = self.noise.get_float(seed as f32 + _step as f32) * std::f32::consts::TAU;
                dir_x = angle.cos();
                dir_y = angle.sin();
            }
            
            // Move droplet
            pos_x += dir_x;
            pos_y += dir_y;
            
            // Get new height and calculate change
            let new_height = self.interpolate_height(grid, pos_x, pos_y, width, height);
            let height_diff = new_height - height_at_pos;
            
            // Calculate sediment capacity based on speed
            let speed = len;
            let capacity = (-height_diff.max(self.config.min_slope) * self.config.sediment_capacity)
                .max(speed * water * self.config.erosion_strength);
            
            // Erode or deposit
            if sediment > capacity {
                // Deposit excess sediment
                let deposit = (sediment - capacity) * self.config.deposition_rate;
                self.deposit_sediment(grid, pos_x, pos_y, width, height, deposit);
                sediment -= deposit;
            } else {
                // Erode terrain
                let erode = (capacity - sediment).min(self.config.max_erosion_depth);
                
                if erode > 0.001 {
                    self.erode_terrain(grid, pos_x, pos_y, width, height, erode);
                    sediment += erode;
                }
            }
            
            // Evaporate water
            water *= 1.0 - self.config.evaporation_rate;
            
            // Stop if water is depleted
            if water < 0.001 {
                break;
            }
        }
    }
    
    /// Interpolate terrain height at floating-point position.
    fn interpolate_height(&self, grid: &TerrainGrid, x: f32, y: f32, width: u32, height: u32) -> f32 {
        let x0 = x as i32;
        let y0 = y as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;
        
        // Clamp to bounds
        let x0 = x0.clamp(0, width as i32 - 1);
        let y0 = y0.clamp(0, height as i32 - 1);
        let x1 = x1.clamp(0, width as i32 - 1);
        let y1 = y1.clamp(0, height as i32 - 1);
        
        // Get corner heights
        let h00 = grid.get(x0 as u32, y0 as u32).map(|c| c.height()).unwrap_or(0.0);
        let h10 = grid.get(x1 as u32, y0 as u32).map(|c| c.height()).unwrap_or(0.0);
        let h01 = grid.get(x0 as u32, y1 as u32).map(|c| c.height()).unwrap_or(0.0);
        let h11 = grid.get(x1 as u32, y1 as u32).map(|c| c.height()).unwrap_or(0.0);
        
        // Bilinear interpolation
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;
        
        h00 * (1.0 - fx) * (1.0 - fy) +
        h10 * fx * (1.0 - fy) +
        h01 * (1.0 - fx) * fy +
        h11 * fx * fy
    }
    
    /// Calculate gradient (slope direction) using finite differences.
    fn calculate_gradient(&self, grid: &TerrainGrid, x: u32, y: u32, width: u32, height: u32) -> (f32, f32) {
        let x = x.clamp(1, width.saturating_sub(2));
        let y = y.clamp(1, height.saturating_sub(2));
        
        let left = grid.get(x.saturating_sub(1), y).map(|c| c.height()).unwrap_or(0.0);
        let right = grid.get(x + 1, y).map(|c| c.height()).unwrap_or(0.0);
        let up = grid.get(x, y.saturating_sub(1)).map(|c| c.height()).unwrap_or(0.0);
        let down = grid.get(x, y + 1).map(|c| c.height()).unwrap_or(0.0);
        
        let grad_x = left - right;
        let grad_y = up - down;
        
        (grad_x, grad_y)
    }
    
    /// Eode terrain at position by specified amount.
    fn erode_terrain(&self, grid: &mut TerrainGrid, x: f32, y: f32, width: u32, height: u32, amount: f32) {
        let cx = x as i32;
        let cy = y as i32;
        
        // Affect 3x3 area around droplet position
        for dy in -1..=1 {
            for dx in -1..=1 {
                let px = (cx + dx) as i32;
                let py = (cy + dy) as i32;
                
                if px < 0 || px >= width as i32 || py < 0 || py >= height as i32 {
                    continue;
                }
                
                // Weight by distance from droplet center
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                let weight = 1.0 - dist / 2.0;
                
                if weight > 0.01 {
                    if let Some(mut cell) = grid.get(px as u32, py as u32) {
                        let current = cell.height();
                        let change = amount * weight * 0.1; // Distribute across nearby cells
                        cell.set_height(current - change);
                        grid.set(px as u32, py as u32, cell);
                    }
                }
            }
        }
    }
    
    /// Deposit sediment at position.
    fn deposit_sediment(&self, grid: &mut TerrainGrid, x: f32, y: f32, width: u32, height: u32, amount: f32) {
        let cx = x as i32;
        let cy = y as i32;
        
        if cx < 0 || cx >= width as i32 || cy < 0 || cy >= height as i32 {
            return;
        }
        
        // Deposit in center cell (simplified - could distribute)
        if let Some(mut cell) = grid.get(cx as u32, cy as u32) {
            let current = cell.height();
            cell.set_height(current + amount);
            grid.set(cx as u32, cy as u32, cell);
        }
    }
    
    /// Thermal weathering - steep slopes relax over time (talus angle).
    fn thermal_weathering(&self, grid: &mut TerrainGrid, width: u32, height: u32) {
        let talus_angle = 30.0_f32.to_radians(); // Natural angle of repose
        let creep_rate = 0.5; // Material moved per iteration
        
        for _iter in 0..self.config.thermal_iterations {
            // Process each cell
            for y in 1..height.saturating_sub(1) {
                for x in 1..width.saturating_sub(1) {
                    let Some(cell) = grid.get(x, y) else { continue };
                    let current_height = cell.height();
                    
                    // Check each neighbor
                    let neighbors = [
                        (x.wrapping_sub(1), y, grid.get(x.wrapping_sub(1), y).map(|c| c.height()).unwrap_or(current_height)),
                        (x + 1, y, grid.get(x + 1, y).map(|c| c.height()).unwrap_or(current_height)),
                        (x, y.wrapping_sub(1), grid.get(x, y.wrapping_sub(1)).map(|c| c.height()).unwrap_or(current_height)),
                        (x, y + 1, grid.get(x + 1, y).map(|c| c.height()).unwrap_or(current_height)),
                    ];
                    
                    for (nx, ny, neighbor_height) in neighbors {
                        if nx >= width || ny >= height { continue; }
                        
                        let drop = current_height - neighbor_height;
                        if drop > 0.0 {
                            let slope = drop; // Already accounting for 1-cell distance
                            
                            if slope > talus_angle {
                                // Calculate material transfer
                                let excess = (slope - talus_angle) * creep_rate;
                                
                                // Erode current cell
                                let mut mutable_cell = cell;
                                mutable_cell.set_height(current_height - excess);
                                grid.set(x, y, mutable_cell);
                                
                                // Deposit at neighbor
                                if let Some(mut neighbor_cell) = grid.get(nx, ny) {
                                    neighbor_cell.set_height(neighbor_height + excess);
                                    grid.set(nx, ny, neighbor_cell);
                                }
                                
                                break; // One transfer per cell per iteration
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Get erosion statistics for analysis.
    pub fn get_stats(&self, grid: &TerrainGrid) -> ErosionStats {
        let (width, height) = grid.dimensions();
        let mut total_height = 0.0f32;
        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;
        
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = grid.get(x, y) {
                    let h = cell.height();
                    total_height += h;
                    min_height = min_height.min(h);
                    max_height = max_height.max(h);
                }
            }
        }
        
        let cell_count = (width * height) as f32;
        ErosionStats {
            average_height: total_height / cell_count,
            min_height,
            max_height,
            height_range: max_height - min_height,
        }
    }
}

/// Erosion statistics.
#[derive(Debug, Clone, Default)]
pub struct ErosionStats {
    pub average_height: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub height_range: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{TerrainGenerator, TerrainConfig, TerrainLayer};
    
    #[test]
    fn test_erosion_changes_terrain() {
        let config = TerrainConfig {
            seed: 100,
            width: 64,
            height: 64,
            ..Default::default()
        };
        
        let mut grid = TerrainGenerator::new(config.clone()).generate(TerrainLayer::Mountains);
        let original_stats = calculate_stats(&grid);
        
        // Run minimal erosion
        let erosion = ErosionSimulator::new(ErosionConfig {
            seed: 100,
            iterations: 1000,
            erosion_strength: 0.3,
            thermal_weathering: false,
            ..Default::default()
        });
        erosion.apply(&mut grid);
        
        let new_stats = calculate_stats(&grid);
        
        // Heights should have changed
        assert_ne!(
            original_stats.0, new_stats.0,
            "Min height should change after erosion"
        );
    }
    
    #[test]
    fn test_deterministic_erosion() {
        let config = TerrainConfig {
            seed: 42,
            width: 32,
            height: 32,
            ..Default::default()
        };
        
        let mut grid1 = TerrainGenerator::new(config.clone()).generate(TerrainLayer::Mountains);
        let mut grid2 = TerrainGenerator::new(config.clone()).generate(TerrainLayer::Mountains);
        
        let erosion_config = ErosionConfig {
            seed: 12345,
            iterations: 100,
            ..Default::default()
        };
        
        ErosionSimulator::new(erosion_config.clone()).apply(&mut grid1);
        ErosionSimulator::new(erosion_config).apply(&mut grid2);
        
        // Same seed should produce same results
        for y in 0..32 {
            for x in 0..32 {
                let h1 = grid1.get(x, y).map(|c| c.height());
                let h2 = grid2.get(x, y).map(|c| c.height());
                assert_eq!(h1, h2, "Erosion should be deterministic with same seed");
            }
        }
    }
    
    fn calculate_stats(grid: &TerrainGrid) -> (f32, f32) {
        let (width, height) = grid.dimensions();
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = grid.get(x, y) {
                    let h = cell.height();
                    min = min.min(h);
                    max = max.max(h);
                }
            }
        }
        
        (min, max)
    }
}
