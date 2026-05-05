//! Tectonic Plate Simulation Engine
//!
//! Core simulation logic for plate tectonics that generates:
//! - TectonicPlate entities with movement vectors and cell assignments
//! - TectonicBoundary segments with classified boundary types
//! - Elevation modifiers for mountain building and rift formation

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use super::BoundaryEffect;
use crate::world::entities::planet::{
    TectonicPlate, TectonicPlateType, TectonicBoundary, TectonicBoundaryType,
    SubductionType,
};
use crate::util::noise::SimplexNoise;

/// Configuration for tectonic simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TectonicSimConfig {
    /// Number of major tectonic plates to generate (3-15).
    pub plate_count: usize,
    /// Seed for deterministic generation.
    pub seed: u64,
    /// World width in cells (for cell-to-plate mapping).
    pub width: u32,
    /// World height in cells.
    pub height: u32,
    /// Tectonic activity intensity [0.0, 1.0].
    /// Higher values produce more dramatic mountains/rifts.
    pub activity: f32,
    /// Enable continental drift animation.
    pub enable_drift: bool,
    /// Ratio of continental to oceanic plates.
    pub continental_ratio: f32,
}

impl Default for TectonicSimConfig {
    fn default() -> Self {
        Self {
            plate_count: 7,
            seed: 0,
            width: 256,
            height: 256,
            activity: 0.5,
            enable_drift: false,
            continental_ratio: 0.35, // ~35% of plates are continental
        }
    }
}

/// Result of tectonic simulation containing plates and boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TectonicResult {
    /// All generated tectonic plates.
    pub plates: Vec<TectonicPlate>,
    /// All generated boundaries between plates.
    pub boundaries: Vec<TectonicBoundary>,
    /// Cell-to-plate mapping for elevation influence.
    pub cell_to_plate: HashMap<u32, Uuid>,
    /// Grid of elevation modifiers from tectonics.
    pub elevation_modifiers: Vec<f32>,
    /// Width of the simulation grid.
    pub width: u32,
    /// Height of the simulation grid.
    pub height: u32,
}

impl TectonicResult {
    /// Get the plate containing a given cell.
    pub fn get_plate_for_cell(&self, cell_id: u32) -> Option<&TectonicPlate> {
        self.cell_to_plate.get(&cell_id)
            .and_then(|id| self.plates.iter().find(|p| &p.id == id))
    }
    
    /// Get all cells belonging to a plate.
    pub fn get_cells_for_plate(&self, plate_id: &Uuid) -> Vec<u32> {
        self.cell_to_plate.iter()
            .filter(|(_, p)| *p == plate_id)
            .map(|(c, _)| *c)
            .collect()
    }
    
    /// Get boundary effect at a specific cell.
    pub fn get_boundary_effect(&self, cell_id: u32) -> Option<BoundaryEffect> {
        // Binary search through sorted boundary cells
        for boundary in &self.boundaries {
            if boundary.cell_ids.contains(&cell_id) {
                return Some(self.effect_for_boundary(&boundary.boundary_type));
            }
        }
        None
    }
    
    fn effect_for_boundary(&self, boundary_type: &TectonicBoundaryType) -> BoundaryEffect {
        match boundary_type {
            TectonicBoundaryType::Convergent { .. } => BoundaryEffect::Uplift,
            TectonicBoundaryType::Divergent { .. } => BoundaryEffect::Subsidence,
            TectonicBoundaryType::Transform { .. } => BoundaryEffect::Shear,
            TectonicBoundaryType::Conservative { .. } => BoundaryEffect::Deformation,
        }
    }
}

/// Main tectonic simulator.
#[derive(Debug, Clone)]
pub struct TectonicSimulator {
    config: TectonicSimConfig,
    noise: SimplexNoise,
}

impl TectonicSimulator {
    /// Create a new simulator.
    pub fn new(config: TectonicSimConfig) -> Self {
        Self {
            noise: SimplexNoise::new(config.seed),
            config,
        }
    }
    
    /// Run the full tectonic simulation.
    ///
    /// Returns plates, boundaries, and elevation modifiers that can be
    /// applied to the terrain grid.
    pub fn simulate(&self) -> TectonicResult {
        // Phase 1: Generate plate seeds and Voronoi cells
        let plate_seeds = self.generate_plate_seeds();
        
        // Phase 2: Assign cell ownership to plates
        let (cell_to_plate, plate_cells) = self.assign_cells_to_plates(&plate_seeds);
        
        // Phase 3: Classify boundaries between adjacent plates
        let (boundaries, boundary_cells) = self.classify_boundaries(&plate_cells, &cell_to_plate);
        
        // Phase 4: Calculate elevation modifiers
        let elevation_modifiers = self.calculate_elevation_modifiers(
            &cell_to_plate,
            &boundary_cells,
            &boundaries,
        );
        
        // Phase 5: Build plate entities with movement data
        let plates = self.build_plates(&plate_cells, &plate_seeds, &cell_to_plate);
        
        TectonicResult {
            plates,
            boundaries,
            cell_to_plate,
            elevation_modifiers,
            width: self.config.width,
            height: self.config.height,
        }
    }
    
    /// Generate plate seed positions using blue noise / Poisson disk sampling.
    fn generate_plate_seeds(&self) -> Vec<(f32, f32, TectonicPlateType)> {
        let mut seeds = Vec::with_capacity(self.config.plate_count);
        let n = self.config.plate_count;
        
        // Generate initial seed positions with some randomization
        let cell_area = (self.config.width * self.config.height) as f32 / n as f32;
        let _spacing = cell_area.sqrt();
        
        for i in 0..n {
            // Use noise to position seeds in a roughly uniform but organic distribution
            let angle = (i as f64 / n as f64) * std::f64::consts::TAU;
            let radius = (i as f64 / n as f64).sqrt() * 0.8; // Concentric arrangement
            
            // Add noise variation
            let noise_x = self.noise.get(i as f64 * 0.1, 0.0);
            let noise_y = self.noise.get(0.0, i as f64 * 0.1);
            
            let x = (0.5 + (radius * angle.cos() + noise_x * 0.2) * (self.config.width as f64 - 1.0)) as f32;
            let y = (0.5 + (radius * angle.sin() + noise_y * 0.2) * (self.config.height as f64 - 1.0)) as f32;
            
            // Determine plate type based on position (latitude) and noise
            let plate_type = self.determine_plate_type(x, y, i);
            
            seeds.push((x, y, plate_type));
        }
        
        seeds
    }
    
    /// Determine plate type based on position and additional factors.
    fn determine_plate_type(&self, x: f32, y: f32, _index: usize) -> TectonicPlateType {
        // Use octave_noise_2d as FBM equivalent
        let continent_noise = self.noise.octave_noise_2d(
            (x * 0.01) as f64, (y * 0.01) as f64, 4, 0.5, 2.0
        ) as f32;
        
        // Higher latitude = more likely continental
        let latitude_factor = (y / self.config.height as f32).abs();
        let continental_score = continent_noise * 0.6 + latitude_factor * 0.4;
        
        if continental_score > 0.5 {
            TectonicPlateType::Continental
        } else if continental_score > 0.3 {
            TectonicPlateType::Mixed
        } else {
            TectonicPlateType::Oceanic
        }
    }
    
    /// Assign each cell to the nearest plate seed.
    fn assign_cells_to_plates(
        &self,
        seeds: &[(f32, f32, TectonicPlateType)],
    ) -> (HashMap<u32, Uuid>, HashMap<Uuid, Vec<u32>>) {
        let mut cell_to_plate: HashMap<u32, Uuid> = HashMap::new();
        let mut plate_cells: HashMap<Uuid, Vec<u32>> = HashMap::new();
        
        let width = self.config.width;
        let height = self.config.height;
        let total_cells = width * height;
        
        // Create plate IDs
        let plate_ids: Vec<Uuid> = seeds.iter()
            .enumerate()
            .map(|_| Uuid::new_v4())
            .collect();
        
        // Initialize plate cells map
        for id in &plate_ids {
            plate_cells.insert(*id, Vec::new());
        }
        
        // Assign each cell to nearest seed using squared distance (avoid sqrt)
        for cell_id in 0..total_cells {
            let x = cell_id % width;
            let y = cell_id / width;
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
            
            let plate_id = plate_ids[nearest_idx];
            cell_to_plate.insert(cell_id, plate_id);
            plate_cells.get_mut(&plate_id).unwrap().push(cell_id);
        }
        
        (cell_to_plate, plate_cells)
    }
    
    /// Classify boundaries between adjacent plates.
    fn classify_boundaries(
        &self,
        _plate_cells: &HashMap<Uuid, Vec<u32>>,
        cell_to_plate: &HashMap<u32, Uuid>,
    ) -> (Vec<TectonicBoundary>, HashMap<u32, Vec<(Uuid, Uuid)>>) {
        let mut boundaries: Vec<TectonicBoundary> = Vec::new();
        let mut boundary_cells: HashMap<u32, Vec<(Uuid, Uuid)>> = HashMap::new();
        
        let width = self.config.width;
        let height = self.config.height;
        
        // Find edges by checking cell neighbors
        for cell_id in 0..(width * height) {
            let x = cell_id % width;
            let y = cell_id / width;
            
            let plate_a = match cell_to_plate.get(&cell_id) {
                Some(p) => *p,
                None => continue,
            };
            
            // Check 4-connected neighbors
            let neighbors = [
                if x > 0 { Some((cell_id - 1, x - 1, y)) } else { None },
                if x < width - 1 { Some((cell_id + 1, x + 1, y)) } else { None },
                if y > 0 { Some((cell_id - width, x, y - 1)) } else { None },
                if y < height - 1 { Some((cell_id + width, x, y + 1)) } else { None },
            ];
            
            for neighbor in neighbors.iter().flatten() {
                let (neighbor_id, _, _) = *neighbor;
                let plate_b = match cell_to_plate.get(&neighbor_id) {
                    Some(p) => *p,
                    None => continue,
                };
                
                // Skip same-plate neighbors
                if plate_a == plate_b {
                    continue;
                }
                
                // Classify the boundary based on plate types and noise
                let boundary_type = self.classify_boundary_type(&plate_a, &plate_b, x, y);
                
                // Create or find existing boundary
                let _boundary_id = Self::find_or_create_boundary(
                    &mut boundaries,
                    &plate_a,
                    &plate_b,
                    boundary_type,
                    &mut boundary_cells,
                    cell_id,
                );
                
                // Add this cell to the boundary
                if let Some(cells) = boundary_cells.get_mut(&cell_id) {
                    cells.push((plate_a, plate_b));
                } else {
                    boundary_cells.insert(cell_id, vec![(plate_a, plate_b)]);
                }
            }
        }
        
        // Calculate boundary lengths and finalize
        for boundary in &mut boundaries {
            let length = boundary.cell_ids.len() as f64 * 1.0; // ~1km per cell
            boundary.length_km = length;
        }
        
        (boundaries, boundary_cells)
    }
    
    /// Classify the type of boundary between two plates.
    fn classify_boundary_type(
        &self,
        _plate_a: &Uuid,
        plate_b: &Uuid,
        x: u32,
        y: u32,
    ) -> TectonicBoundaryType {
        // Use noise to add variety to boundary types
        let nx = x as f64 / self.config.width as f64;
        let ny = y as f64 / self.config.height as f64;
        
        let type_noise = self.noise.get_fbm(nx * 4.0, ny * 4.0, 3, 0.5, 2.0);
        let movement_noise = self.noise.get(nx * 2.0, ny * 2.0);
        
        // Determine boundary type based on noise and plate types
        if type_noise < 0.35 {
            // Convergent boundary
            TectonicBoundaryType::Convergent {
                subduction_rate_cm_yr: (movement_noise.abs() * 10.0) as f32,
                subducting_plate: Some(*plate_b),
                subduction_type: SubductionType::OceanicUnderContinental,
            }
        } else if type_noise < 0.65 {
            // Divergent boundary
            TectonicBoundaryType::Divergent {
                spreading_rate_cm_yr: (movement_noise.abs() * 5.0 + 1.0) as f32,
            }
        } else {
            // Transform boundary
            TectonicBoundaryType::Transform {
                slip_rate_cm_yr: (movement_noise.abs() * 3.0 + 0.5) as f32,
            }
        }
    }
    
    /// Find existing boundary between plates or create new one.
    fn find_or_create_boundary(
        boundaries: &mut Vec<TectonicBoundary>,
        plate_a: &Uuid,
        plate_b: &Uuid,
        boundary_type: TectonicBoundaryType,
        _boundary_cells: &mut HashMap<u32, Vec<(Uuid, Uuid)>>,
        cell_id: u32,
    ) -> Uuid {
        // Sort plate IDs for consistent lookup
        let (min_id, max_id) = if plate_a < plate_b { (*plate_a, *plate_b) } else { (*plate_b, *plate_a) };
        
        // Find existing boundary
        if let Some(existing) = boundaries.iter_mut().find(|b| {
            let (min_b, max_b) = if b.plate_ids[0] < b.plate_ids[1] {
                (b.plate_ids[0], b.plate_ids[1])
            } else {
                (b.plate_ids[1], b.plate_ids[0])
            };
            min_b == min_id && max_b == max_id
        }) {
            existing.cell_ids.push(cell_id);
            return existing.id;
        }
        
        // Create new boundary
        let boundary = TectonicBoundary::new(
            Uuid::new_v4(),
            boundary_type,
            [min_id, max_id],
            vec![cell_id],
            0.0, // Will be calculated later
        );
        
        boundaries.push(boundary);
        boundaries.last().unwrap().id
    }
    
    /// Calculate elevation modifiers from tectonic activity.
    fn calculate_elevation_modifiers(
        &self,
        _cell_to_plate: &HashMap<u32, Uuid>,
        boundary_cells: &HashMap<u32, Vec<(Uuid, Uuid)>>,
        boundaries: &[TectonicBoundary],
    ) -> Vec<f32> {
        let total_cells = self.config.width * self.config.height;
        let mut modifiers = vec![0.0f32; total_cells as usize];
        
        let activity = self.config.activity;
        
        // Apply boundary effects
        for (cell_id, pairs) in boundary_cells {
            let idx = *cell_id as usize;
            if idx >= modifiers.len() {
                continue;
            }
            
            // Check what type of boundary this cell is on
            for pair in pairs {
                let boundary = boundaries.iter().find(|b| {
                    (b.plate_ids[0] == pair.0 && b.plate_ids[1] == pair.1) ||
                    (b.plate_ids[0] == pair.1 && b.plate_ids[1] == pair.0)
                });
                
                if let Some(b) = boundary {
                    let effect = match &b.boundary_type {
                        TectonicBoundaryType::Convergent { .. } => {
                            // Mountains form at convergent boundaries
                            // More dramatic at continental-oceanic or continental-continental
                            (1500.0 * activity as f64) as f32
                        }
                        TectonicBoundaryType::Divergent { .. } => {
                            // Rifts form at divergent boundaries
                            (-500.0 * activity as f64) as f32
                        }
                        TectonicBoundaryType::Transform { .. } => {
                            // Minimal vertical displacement
                            (50.0 * activity as f64) as f32
                        }
                        TectonicBoundaryType::Conservative { .. } => {
                            (20.0 * activity as f64) as f32
                        }
                    };
                    
                    // Add noise variation to make boundaries less uniform
                    let nx = (*cell_id % self.config.width) as f64 / self.config.width as f64;
                    let ny = (*cell_id / self.config.width) as f64 / self.config.height as f64;
                    let noise_variation = (self.noise.get(nx * 8.0, ny * 8.0) * 300.0) as f32;
                    
                    modifiers[idx] += effect + noise_variation;
                }
            }
        }
        
        // Apply subduction zone effects (deeper trenches adjacent to convergent boundaries)
        for boundary in boundaries {
            if matches!(boundary.boundary_type, TectonicBoundaryType::Convergent { .. }) {
                // Find adjacent cells for trench effect
                for &cell_id in &boundary.cell_ids {
                    let x = cell_id % self.config.width;
                    let y = cell_id / self.config.width;
                    
                    // Check neighbors for shallow water / trench
                    for (dx, dy) in &[(0i32, 1i32), (0i32, -1i32), (1i32, 0i32), (-1i32, 0i32)] {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        
                        if nx >= 0 && nx < self.config.width as i32 && 
                           ny >= 0 && ny < self.config.height as i32 {
                            let neighbor_id = (ny as u32) * self.config.width + nx as u32;
                            let neighbor_idx = neighbor_id as usize;
                            
                            // Only apply to cells that are on boundaries
                            if let Some(cell_pairs) = boundary_cells.get(&neighbor_id) {
                                // Check if this neighbor has different plate than boundary
                                if cell_pairs.iter().any(|(p1, p2)| 
                                    *p1 != boundary.plate_ids[0] && *p1 != boundary.plate_ids[1] &&
                                    *p2 != boundary.plate_ids[0] && *p2 != boundary.plate_ids[1]
                                ) {
                                    // Adjacent non-boundary cell gets trench effect
                                    if neighbor_idx < modifiers.len() {
                                        modifiers[neighbor_idx] -= 800.0 * activity;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        modifiers
    }
    
    /// Build plate entities from cell assignments and seeds.
    fn build_plates(
        &self,
        plate_cells: &HashMap<Uuid, Vec<u32>>,
        seeds: &[(f32, f32, TectonicPlateType)],
        _cell_to_plate: &HashMap<u32, Uuid>,
    ) -> Vec<TectonicPlate> {
        let mut plates = Vec::with_capacity(seeds.len());
        
        // We need to match seeds to plate IDs - use index mapping
        let mut seed_plate_map: HashMap<usize, Uuid> = HashMap::new();
        
        // First, collect all unique plate IDs
        let mut unique_ids: Vec<Uuid> = plate_cells.keys().copied().collect();
        unique_ids.sort_by(|a, b| a.cmp(b));
        
        // Assign seed indices to plate IDs (they should be in creation order)
        for (i, id) in unique_ids.iter().enumerate() {
            seed_plate_map.insert(i, *id);
        }
        
        for (i, (sx, sy, plate_type)) in seeds.iter().enumerate() {
            let plate_id = seed_plate_map.get(&i).copied().unwrap_or_else(Uuid::new_v4);
            let cells = plate_cells.get(&plate_id).cloned().unwrap_or_default();
            
            // Calculate area (each cell is ~1km²)
            let area_km2 = cells.len() as f64;
            
            // Generate movement vector based on position and noise
            let direction = (self.noise.get(*sx as f64 * 0.01, *sy as f64 * 0.01) * 360.0) as f32;
            let speed = (2.0 + self.noise.get(*sx as f64 * 0.02, *sy as f64 * 0.02).abs() * 10.0) as f32;
            
            let mut plate = TectonicPlate::new(
                plate_id,
                direction,
                speed,
                *plate_type,
                cells,
                area_km2,
            );
            
            // Add plate name based on type and position
            plate.name = Some(self.generate_plate_name(*plate_type, *sx, *sy));
            
            plates.push(plate);
        }
        
        plates
    }
    
    /// Generate a name for a tectonic plate.
    fn generate_plate_name(&self, plate_type: TectonicPlateType, _x: f32, y: f32) -> String {
        let base_name = match plate_type {
            TectonicPlateType::Continental => "Continental",
            TectonicPlateType::Oceanic => "Pacific",
            TectonicPlateType::Mixed => "Indo",
        };
        
        // Add directional qualifier
        let qualifier = if y < self.config.height as f32 * 0.4 {
            "Northern"
        } else if y > self.config.height as f32 * 0.6 {
            "Southern"
        } else {
            "Central"
        };
        
        format!("{} {}", qualifier, base_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulator_creation() {
        let config = TectonicSimConfig::default();
        let simulator = TectonicSimulator::new(config);
        assert!(simulator.config.plate_count > 0);
    }
    
    #[test]
    fn test_deterministic_simulation() {
        let config = TectonicSimConfig {
            seed: 42,
            plate_count: 5,
            ..Default::default()
        };
        
        let sim1 = TectonicSimulator::new(config.clone());
        let sim2 = TectonicSimulator::new(config.clone());
        
        let result1 = sim1.simulate();
        let result2 = sim2.simulate();
        
        assert_eq!(result1.plates.len(), result2.plates.len());
        assert_eq!(result1.boundaries.len(), result2.boundaries.len());
    }
    
    #[test]
    fn test_boundary_classification() {
        let config = TectonicSimConfig {
            plate_count: 4,
            width: 32,
            height: 32,
            ..Default::default()
        };
        
        let simulator = TectonicSimulator::new(config);
        let result = simulator.simulate();
        
        // Should have some boundaries
        assert!(!result.boundaries.is_empty());
        
        // All boundaries should have valid types
        for boundary in &result.boundaries {
            match &boundary.boundary_type {
                TectonicBoundaryType::Divergent { spreading_rate_cm_yr } => assert!(*spreading_rate_cm_yr >= 0.0),
                TectonicBoundaryType::Convergent { subduction_rate_cm_yr, .. } => assert!(*subduction_rate_cm_yr >= 0.0),
                TectonicBoundaryType::Transform { slip_rate_cm_yr } => assert!(*slip_rate_cm_yr >= 0.0),
                TectonicBoundaryType::Conservative { .. } => {}
            }
        }
    }
    
    #[test]
    fn test_plate_coverage() {
        let config = TectonicSimConfig {
            plate_count: 6,
            width: 16,
            height: 16,
            seed: 123,
            ..Default::default()
        };
        
        let simulator = TectonicSimulator::new(config);
        let result = simulator.simulate();
        
        // All cells should be assigned to a plate
        let total_cells = result.width * result.height;
        assert_eq!(result.cell_to_plate.len(), total_cells as usize);
        
        // Total cells across all plates should equal total cells
        let plate_cell_count: usize = result.plates.iter().map(|p| p.cell_ids.len()).sum();
        assert_eq!(plate_cell_count, total_cells as usize, "all cells should be distributed to plates");
        
        // Simulation should produce some plates
        assert!(!result.plates.is_empty(), "should have at least one plate");
    }
}