//! Plate Builder and Cell Allocator Utilities
//!
//! Helper types for constructing tectonic plates with controlled properties
//! and for allocating cells to plates based on various algorithms.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::world::entities::planet::{
    TectonicPlate, TectonicPlateType, TectonicBoundary, TectonicBoundaryType,
};
use crate::util::noise::SimplexNoise;

/// Builder for constructing tectonic plates with fluent API.
#[derive(Debug, Clone)]
pub struct PlateBuilder {
    id: Option<Uuid>,
    name: Option<String>,
    movement_direction: f32,
    movement_speed: f32,
    plate_type: TectonicPlateType,
    cell_ids: Vec<u32>,
    boundaries: Vec<TectonicBoundary>,
    area_km2: f64,
}

impl PlateBuilder {
    /// Create a new plate builder.
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            movement_direction: 0.0,
            movement_speed: 5.0,
            plate_type: TectonicPlateType::Oceanic,
            cell_ids: Vec::new(),
            boundaries: Vec::new(),
            area_km2: 0.0,
        }
    }
    
    /// Set the plate ID.
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }
    
    /// Set the plate name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set the plate type.
    pub fn with_type(mut self, plate_type: TectonicPlateType) -> Self {
        self.plate_type = plate_type;
        self
    }
    
    /// Set the movement vector.
    pub fn with_movement(mut self, direction_deg: f32, speed_cm_yr: f32) -> Self {
        self.movement_direction = direction_deg;
        self.movement_speed = speed_cm_yr;
        self
    }
    
    /// Set the cells belonging to this plate.
    pub fn with_cells(mut self, cells: Vec<u32>) -> Self {
        self.cell_ids = cells.clone();
        self.area_km2 = self.cell_ids.len() as f64; // ~1km² per cell
        self
    }
    
    /// Add cells to the plate.
    pub fn add_cells(mut self, cells: impl IntoIterator<Item = u32>) -> Self {
        self.cell_ids.extend(cells);
        self.area_km2 = self.cell_ids.len() as f64;
        self
    }
    
    /// Add a boundary to the plate.
    pub fn add_boundary(mut self, boundary: TectonicBoundary) -> Self {
        self.boundaries.push(boundary);
        self
    }
    
    /// Build the final TectonicPlate.
    pub fn build(self) -> TectonicPlate {
        let id = self.id.unwrap_or_else(Uuid::new_v4);
        
        let mut plate = TectonicPlate::new(
            id,
            self.movement_direction,
            self.movement_speed,
            self.plate_type,
            self.cell_ids,
            self.area_km2,
        );
        
        plate.name = self.name;
        for boundary in self.boundaries {
            plate.add_boundary(boundary);
        }
        
        plate
    }
}

impl Default for PlateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Algorithm for allocating cells to tectonic plates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellAllocationStrategy {
    /// Voronoi tessellation based on seed points.
    Voronoi,
    /// Poisson disk sampling for uniform distribution.
    BlueNoise,
    /// Uniform grid-based allocation.
    Grid,
    /// Based on elevation/continental mask.
    ElevationBased,
}

impl Default for CellAllocationStrategy {
    fn default() -> Self {
        CellAllocationStrategy::Voronoi
    }
}

/// Allocates cells to tectonic plates using various strategies.
#[derive(Debug, Clone)]
pub struct PlateCellAllocator {
    width: u32,
    height: u32,
    strategy: CellAllocationStrategy,
    noise: SimplexNoise,
}

impl PlateCellAllocator {
    /// Create a new allocator.
    pub fn new(width: u32, height: u32, seed: u64, strategy: CellAllocationStrategy) -> Self {
        Self {
            width,
            height,
            strategy,
            noise: SimplexNoise::new(seed),
        }
    }
    
    /// Generate plate seeds based on the configured strategy.
    pub fn generate_seeds(&self, plate_count: usize) -> Vec<(f32, f32, TectonicPlateType)> {
        match self.strategy {
            CellAllocationStrategy::Voronoi => self.voronoi_seeds(plate_count),
            CellAllocationStrategy::BlueNoise => self.blue_noise_seeds(plate_count),
            CellAllocationStrategy::Grid => self.grid_seeds(plate_count),
            CellAllocationStrategy::ElevationBased => self.elevation_seeds(plate_count),
        }
    }
    
    /// Voronoi-based seed placement with noise variation.
    fn voronoi_seeds(&self, count: usize) -> Vec<(f32, f32, TectonicPlateType)> {
        let mut seeds = Vec::with_capacity(count);
        
        for i in 0..count {
            let angle = (i as f64 / count as f64) * std::f64::consts::TAU;
            let radius = 0.4 + self.noise.get(i as f64 * 0.1, 0.0) * 0.2;
            
            let x = 0.5 + radius * angle.cos();
            let y = 0.5 + radius * angle.sin();
            
            let plate_type = self.determine_type(x, y, i);
            seeds.push((x as f32, y as f32, plate_type));
        }
        
        seeds
    }
    
    /// Blue noise / Poisson disk sampling for more uniform distribution.
    fn blue_noise_seeds(&self, count: usize) -> Vec<(f32, f32, TectonicPlateType)> {
        let mut seeds = Vec::new();
        let cell_area = (self.width * self.height) as f32 / count as f32;
        let min_distance = cell_area.sqrt() * 0.8;
        
        let mut attempts = 0;
        let max_attempts = count * 100;
        
        while seeds.len() < count && attempts < max_attempts {
            attempts += 1;
            
            // Generate candidate position
            let nx = self.noise.get(seeds.len() as f64 * 0.5, attempts as f64);
            let ny = self.noise.get(seeds.len() as f64 * 0.7, attempts as f64);
            
            let x = (nx * self.width as f64) as f32;
            let y = (ny * self.height as f64) as f32;
            
            // Check minimum distance from existing seeds
            let too_close = seeds.iter().any(|(sx, sy, _)| {
                let dx = x - sx;
                let dy = y - sy;
                let dist_sq = dx * dx + dy * dy;
                dist_sq < min_distance * min_distance
            });
            
            if !too_close {
                let plate_type = self.determine_type(x as f64, y as f64, seeds.len());
                seeds.push((x, y, plate_type));
            }
        }
        
        seeds
    }
    
    /// Uniform grid-based seed placement.
    fn grid_seeds(&self, count: usize) -> Vec<(f32, f32, TectonicPlateType)> {
        let side = ((count as f32).sqrt().ceil() as usize);
        let spacing_x = self.width as f32 / side as f32;
        let spacing_y = self.height as f32 / side as f32;
        
        let mut seeds = Vec::with_capacity(count);
        
        for i in 0..count {
            let row = i / side;
            let col = i % side;
            
            let x = (col as f64 + 0.5) * spacing_x as f64 + self.noise.get(col as f64, row as f64) * spacing_x as f64 * 0.3;
            let y = (row as f64 + 0.5) * spacing_y as f64 + self.noise.get(row as f64, col as f64) * spacing_y as f64 * 0.3;
            
            let plate_type = self.determine_type(x, y, i);
            seeds.push((x as f32, y as f32, plate_type));
        }
        
        seeds
    }
    
    /// Elevation-based seeds that prefer continental regions.
    fn elevation_seeds(&self, count: usize) -> Vec<(f32, f32, TectonicPlateType)> {
        let mut seeds = Vec::with_capacity(count);
        
        // Sample many potential positions and pick those in continental areas
        let samples = count * 4;
        let mut candidates: Vec<(f32, f32, f32)> = Vec::with_capacity(samples);
        
        for i in 0..samples {
            let nx = self.noise.get(i as f64 * 0.1, 0.0);
            let ny = self.noise.get(0.0, i as f64 * 0.1);
            
            // Continental mask from noise using octave_noise_2d as FBM
            let continent_score = self.noise.octave_noise_2d(
                nx * 2.0, ny * 2.0, 4, 0.5, 2.0
            ) as f32;
            
            let x = (nx * self.width as f64) as f32;
            let y = (ny * self.height as f64) as f32;
            
            candidates.push((x, y, continent_score));
        }
        
        // Sort by continent score and take top candidates
        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        
        for (i, (x, y, _)) in candidates.into_iter().enumerate().take(count) {
            let plate_type = self.determine_type(x as f64, y as f64, i);
            seeds.push((x, y, plate_type));
        }
        
        seeds
    }
    
    /// Determine plate type based on position.
    fn determine_type(&self, nx: f64, ny: f64, index: usize) -> TectonicPlateType {
        // Use octave_noise_2d as FBM equivalent
        let continent_score = self.noise.octave_noise_2d(
            nx * 3.0, ny * 3.0, 3, 0.5, 2.0
        ) as f32;
        let latitude_factor = ((ny - 0.5).abs() * 2.0) as f32; // Higher at poles
        
        let combined = continent_score * 0.5 + latitude_factor * 0.5;
        
        if combined > 0.55 {
            TectonicPlateType::Continental
        } else if combined > 0.35 {
            TectonicPlateType::Mixed
        } else {
            TectonicPlateType::Oceanic
        }
    }
    
    /// Assign cells to plates based on nearest seed.
    pub fn assign_cells(&self, seeds: &[(f32, f32, TectonicPlateType)]) -> Vec<(u32, usize)> {
        let mut assignments = Vec::with_capacity((self.width * self.height) as usize);
        
        for cell_id in 0..(self.width * self.height) {
            let x = cell_id % self.width;
            let y = cell_id / self.width;
            let cx = x as f32;
            let cy = y as f32;
            
            let mut nearest_idx = 0;
            let mut nearest_dist_sq = f32::MAX;
            
            for (seed_idx, (sx, sy, _)) in seeds.iter().enumerate() {
                let dx = cx - sx;
                let dy = cy - sy;
                let dist_sq = dx * dx + dy * dy;
                
                if dist_sq < nearest_dist_sq {
                    nearest_dist_sq = dist_sq;
                    nearest_idx = seed_idx;
                }
            }
            
            assignments.push((cell_id, nearest_idx));
        }
        
        assignments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plate_builder() {
        let plate = PlateBuilder::new()
            .with_id(Uuid::new_v4())
            .with_name("Pacific Plate")
            .with_type(TectonicPlateType::Oceanic)
            .with_movement(90.0, 10.0)
            .with_cells(vec![1, 2, 3, 4, 5])
            .build();
        
        assert_eq!(plate.name, Some("Pacific Plate".to_string()));
        assert!(matches!(plate.plate_type, TectonicPlateType::Oceanic));
        assert_eq!(plate.cell_ids.len(), 5);
    }
    
    #[test]
    fn test_voronoi_seeds() {
        let allocator = PlateCellAllocator::new(256, 256, 42, CellAllocationStrategy::Voronoi);
        let seeds = allocator.generate_seeds(7);
        
        assert_eq!(seeds.len(), 7);
        
        // Seeds should be within bounds
        for (x, y, _) in &seeds {
            assert!(*x >= 0.0 && *x < 256.0);
            assert!(*y >= 0.0 && *y < 256.0);
        }
    }
    
    #[test]
    fn test_cell_assignment() {
        let allocator = PlateCellAllocator::new(32, 32, 42, CellAllocationStrategy::Voronoi);
        let seeds = allocator.generate_seeds(4);
        let assignments = allocator.assign_cells(&seeds);
        
        // All cells should be assigned
        assert_eq!(assignments.len(), 32 * 32);
        
        // All assignments should be to valid seeds (0-3)
        for (_, seed_idx) in &assignments {
            assert!(*seed_idx < 4);
        }
    }
}