//! Settlement Spawning Module
//! 
//! Implements settlement placement logic for World Factory.
//! 
//! ## Algorithm Overview
//! 
//! Settlement spawning is a multi-phase process:
//! 
//! 1. **Suitability Analysis** — Score each terrain cell for settlement potential
//! 2. **Density Generation** — Create a population density map using noise
//! 3. **Site Selection** — Pick optimal locations based on density + constraints
//! 4. **Species Assignment** — Assign species to settlements based on biome suitability
//! 5. **Name Generation** — Assign culturally-appropriate names
//! 
//! ## Suitability Constraints
//! 
//! - ❌ Ocean (elevation < sea_level)
//! - ❌ Desert biomes (HotDesert, ColdDesert, SubtropicalDesert, TemperateDesert)
//! - ❌ Tundra biomes (Tundra, Arctic, PolarDesert, AlpineTundra)
//! - ❌ Permanent snow/glacier (SnowGlacier, Nival elevation)
//! - ✅ Preferred: Grassland, Forest, TemperateMixed, TemperateDeciduous
//! - ⚠️ Possible: Wetlands, Coastal areas (with elevation constraints)
//! 
//! ## Determinism
//! 
//! All decisions are seeded — same world seed produces identical settlements.

use serde::{Deserialize, Serialize};
use crate::util::{Rng, Seed};
use crate::terrain::biome::{BiomeType, ClimateZone};
use crate::types::{Settlement, SettlementType, GeoLocation};
use crate::species::{SpeciesId, SpeciesData};
use std::collections::HashMap;

/// Configuration for settlement generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    /// Target number of settlements per 1000 terrain cells.
    /// Default: 0.5 (0.05% of land is settled).
    pub density_target: f32,
    
    /// Minimum distance between settlements in cells.
    /// Default: 8 cells.
    pub min_spacing: u32,
    
    /// Maximum attempts to find a suitable site before giving up.
    /// Default: 100.
    pub max_attempts: u32,
    
    /// Minimum elevation for coastal settlements (0 = no constraint).
    /// Default: 2m.
    pub coastal_max_elevation: f32,
    
    /// Include ruins/abandoned settlements.
    /// Default: false.
    pub include_ruins: bool,
    
    /// Settlement size distribution weights.
    pub size_weights: SettlementSizeWeights,
}

impl Default for SettlementConfig {
    fn default() -> Self {
        Self {
            density_target: 0.5,
            min_spacing: 8,
            max_attempts: 100,
            coastal_max_elevation: 2.0, // coastal settlements only up to 2m elevation
            include_ruins: false,
            size_weights: SettlementSizeWeights::default(),
        }
    }
}

/// Weights for settlement size distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSizeWeights {
    pub hamlet: f32,   // < 100 people
    pub village: f32,  // 100-1000 people
    pub town: f32,     // 1000-10000 people
    pub city: f32,     // 10000-100000 people
    pub metropolis: f32, // 100000+ people
}

impl Default for SettlementSizeWeights {
    fn default() -> Self {
        Self {
            hamlet: 0.30,
            village: 0.35,
            town: 0.20,
            city: 0.12,
            metropolis: 0.03,
        }
    }
}

/// Result of suitability analysis for a terrain cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuitabilityScore {
    pub score: f32,           // 0.0 to 1.0, higher is better
    pub reasons: Vec<SuitabilityReason>,
    pub settlement_type: Option<SettlementType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuitabilityReason {
    /// Excellent conditions (e.g., river, fertile plains)
    Excellent(String),
    /// Good conditions (e.g., moderate climate, good soil)
    Good(String),
    /// Acceptable but not ideal
    Acceptable(String),
    /// Negative factor (e.g., harsh winters, poor soil)
    Negative(String),
}

/// Population density map for settlement placement.
#[derive(Debug, Clone)]
pub struct DensityMap {
    /// Width in cells.
    pub width: usize,
    /// Height in cells.
    pub height: usize,
    /// Density values 0.0 to 1.0.
    pub values: Vec<f32>,
}

impl DensityMap {
    /// Get density at coordinates.
    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.values.get(y * self.width + x).copied()
    }
}

/// A candidate settlement site.
#[derive(Debug, Clone)]
pub struct SettlementSite {
    pub x: usize,
    pub y: usize,
    pub density: f32,
    pub suitability: f32,
    pub biome: BiomeType,
    pub climate: ClimateZone,
    pub has_river: bool,
    pub has_coast: bool,
    pub elevation_m: f32,
}

/// Settlement generation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    pub settlements: Vec<Settlement>,
    pub stats: SettlementStats,
}

/// Statistics about generated settlements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementStats {
    pub total: usize,
    pub by_type: HashMap<SettlementType, usize>,
    pub by_biome: HashMap<BiomeType, usize>,
    pub coastal_count: usize,
    pub river_count: usize,
    pub average_population: f64,
}

impl Default for SettlementStats {
    fn default() -> Self {
        Self {
            total: 0,
            by_type: HashMap::new(),
            by_biome: HashMap::new(),
            coastal_count: 0,
            river_count: 0,
            average_population: 0.0,
        }
    }
}

/// Main settlement generator.
pub struct SettlementGenerator {
    config: SettlementConfig,
    rng: Rng,
}

impl SettlementGenerator {
    /// Create a new generator with configuration.
    pub fn new(config: SettlementConfig, seed: u64) -> Self {
        Self {
            config,
            rng: Rng::new(seed),
        }
    }
    
    /// Generate settlements for terrain data.
    /// 
    /// # Arguments
    /// 
    /// * `elevation_grid` — Elevation values per cell
    /// * `biome_grid` — Biome type for each cell
    /// * `climate_grid` — Climate zone for each cell
    /// * `sea_level` — Sea level threshold (elevation units)
    /// * `width` / `height` — Grid dimensions
    /// * `rivers` — Optional river cell positions for bonus calculation
    /// 
    /// # Returns
    /// 
    /// SettlementResult with all generated settlements and statistics.
    pub fn generate(
        &mut self,
        elevation_grid: &[f32],
        biome_grid: &[BiomeType],
        climate_grid: &[ClimateZone],
        sea_level: f32,
        width: usize,
        height: usize,
        river_cells: Option<&[(i32, i32)]>,
    ) -> SettlementResult {
        let total_cells = width * height;
        
        // Phase 1: Generate population density map
        let density_map = self.generate_density_map(width, height);
        
        // Phase 2: Find all suitable sites
        let sites = self.find_suitable_sites(
            elevation_grid,
            biome_grid,
            climate_grid,
            &density_map,
            sea_level,
            width,
            height,
            river_cells,
        );
        
        // Phase 3: Select settlement locations (greedy with spacing constraint)
        let selected_sites = self.select_sites(sites, width, height);
        
        // Phase 4: Create Settlement entities
        let settlements = self.create_settlements(selected_sites, width, height);
        
        // Phase 5: Generate statistics
        let stats = self.compute_stats(&settlements);
        
        SettlementResult { settlements, stats }
    }
    
    /// Generate settlements with species assignment.
    /// 
    /// This method extends the base `generate()` with full species integration:
    /// - Assigns the best-suited species to each settlement based on biome
    /// - Generates culturally-appropriate settlement names
    /// 
    /// # Arguments
    /// 
    /// * `elevation_grid` — Elevation values per cell
    /// * `biome_grid` — Biome type for each cell
    /// * `climate_grid` — Climate zone for each cell
    /// * `species_data` — Species definitions and name templates
    /// * `sea_level` — Sea level threshold (elevation units)
    /// * `width` / `height` — Grid dimensions
    /// * `river_cells` — Optional river cell positions for bonus calculation
    /// 
    /// # Returns
    /// 
    /// SettlementResult with all generated settlements and statistics.
    pub fn generate_with_species(
        &mut self,
        elevation_grid: &[f32],
        biome_grid: &[BiomeType],
        climate_grid: &[ClimateZone],
        species_data: &SpeciesData,
        sea_level: f32,
        width: usize,
        height: usize,
        river_cells: Option<&[(i32, i32)]>,
    ) -> SettlementResult {
        // Phase 1: Generate population density map
        let density_map = self.generate_density_map(width, height);
        
        // Phase 2: Find all suitable sites
        let sites = self.find_suitable_sites(
            elevation_grid,
            biome_grid,
            climate_grid,
            &density_map,
            sea_level,
            width,
            height,
            river_cells,
        );
        
        // Phase 3: Select settlement locations
        let selected_sites = self.select_sites(sites, width, height);
        
        // Phase 4: Create Settlement entities with species
        let settlements = self.create_settlements_with_species(
            selected_sites,
            species_data,
            width,
            height,
        );
        
        // Phase 5: Generate statistics
        let stats = self.compute_stats(&settlements);
        
        SettlementResult { settlements, stats }
    }
    
    /// Create Settlement entities from selected sites with species assignment.
    fn create_settlements_with_species(
        &mut self,
        sites: Vec<SettlementSite>,
        species_data: &SpeciesData,
        width: usize,
        height: usize,
    ) -> Vec<Settlement> {
        sites
            .into_iter()
            .map(|site| {
                let settlement_type = self.pick_settlement_type(site.density);
                let population = self.pick_population(settlement_type, site.density);
                
                // Create location
                let lat = ((site.y as f64 / height as f64) * 180.0) - 90.0;
                let lon = ((site.x as f64 / width as f64) * 360.0) - 180.0;
                let location = GeoLocation::with_elevation(lat, lon, site.elevation_m);
                
                // Determine best species for this biome
                let species_id = species_data.best_species_for_biome(site.biome)
                    .unwrap_or(SpeciesId::HUMAN);
                
                // Generate culturally-appropriate name
                let name = species_data.generate_name(species_id, &mut self.rng);
                
                // Build description
                let species_name = species_data.get(species_id)
                    .map(|s| s.name.as_ref())
                    .unwrap_or("Unknown");
                let mut description = format!("{} settlement on {} ({})", 
                    species_name, 
                    site.biome.name(), 
                    site.climate.short_name()
                );
                if site.has_river {
                    description.push_str(" (river)");
                }
                if site.has_coast {
                    description.push_str(" (coast)");
                }
                
                // Create settlement with species
                let settlement = Settlement::with_details(
                    uuid::Uuid::new_v4(),
                    name,
                    settlement_type,
                    population,
                    location,
                    Some(description),
                ).with_species(species_id);
                
                settlement
            })
            .collect()
    }
    
    /// Generate population density map using multi-scale noise.
    fn generate_density_map(&mut self, width: usize, height: usize) -> DensityMap {
        let mut values = Vec::with_capacity(width * height);
        
        // Use multiple octaves of noise for natural density patterns
        // Base scale: 1/64 of world size (large regions of high/low density)
        // Detail scale: 1/16 (local population centers)
        let base_scale = 0.015625; // 1/64
        let detail_scale = 0.0625;  // 1/16
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / width as f32;
                let ny = y as f32 / height as f32;
                
                // Combine base and detail noise
                let base = self.rng.simplex_2d(nx * base_scale, ny * base_scale);
                let detail = self.rng.simplex_2d(nx * detail_scale, ny * detail_scale);
                
                // Blend: 60% base, 40% detail
                let density = base * 0.6 + detail * 0.4;
                
                // Normalize to 0-1 range and apply power curve for clustering
                let normalized = (density + 1.0) * 0.5; // -1..1 → 0..1
                let clustered = normalized.powf(1.5); // Squish towards 0 for more variance
                
                values.push(clustered);
            }
        }
        
        DensityMap { width, height, values }
    }
    
    /// Find all cells that are suitable for settlement.
    fn find_suitable_sites(
        &mut self,
        elevation_grid: &[f32],
        biome_grid: &[BiomeType],
        climate_grid: &[ClimateZone],
        density_map: &DensityMap,
        sea_level: f32,
        width: usize,
        height: usize,
        river_cells: Option<&[(i32, i32)]>,
    ) -> Vec<SettlementSite> {
        let mut sites = Vec::new();
        
        // Create river lookup for fast checking
        let river_set: std::collections::HashSet<(i32, i32)> = river_cells
            .map(|cells| cells.iter().cloned().collect())
            .unwrap_or_default();
        
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                
                // Check terrain
                let elevation = elevation_grid[idx];
                if elevation < sea_level {
                    continue; // Skip ocean
                }
                
                let biome = biome_grid[idx];
                if Self::is_excluded_biome(biome) {
                    continue; // Skip desert/tundra
                }
                
                // Check adjacent for river/coast features
                let has_river = river_set.contains(&(x as i32, y as i32)) 
                    || self.has_adjacent_river(x, y, elevation_grid, sea_level, width, height);
                let has_coast = self.is_coastal(x, y, elevation_grid, sea_level, width, height);
                
                // Calculate suitability score
                let suitability = self.calculate_suitability(
                    biome,
                    climate_grid[idx],
                    elevation,
                    has_river,
                    has_coast,
                );
                
                // Only include cells with non-zero suitability
                if suitability > 0.0 {
                    if let Some(density) = density_map.get(x, y) {
                        sites.push(SettlementSite {
                            x,
                            y,
                            density,
                            suitability,
                            biome,
                            climate: climate_grid[idx],
                            has_river,
                            has_coast,
                            elevation_m: elevation,
                        });
                    }
                }
            }
        }
        
        sites
    }
    
    /// Check if a biome is excluded from settlement.
    fn is_excluded_biome(biome: BiomeType) -> bool {
        matches!(
            biome,
            BiomeType::HotDesert
            | BiomeType::ColdDesert
            | BiomeType::SubtropicalDesert
            | BiomeType::TemperateDesert
            | BiomeType::Tundra
            | BiomeType::Arctic
            | BiomeType::PolarDesert
            | BiomeType::SnowGlacier
            | BiomeType::AlpineTundra
        )
    }
    
    /// Check if a cell is adjacent to a river (lower elevation neighbor).
    fn has_adjacent_river(
        &self,
        x: usize,
        y: usize,
        elevation_grid: &[f32],
        sea_level: f32,
        width: usize,
        height: usize,
    ) -> bool {
        let neighbors = self.get_cardinal_neighbors(x, y, width, height);
        let center_elevation = elevation_grid[y * width + x];
        
        for (nx, ny) in neighbors {
            let nidx = ny * width + nx;
            let neighbor_elevation = elevation_grid[nidx];
            
            // River: cell is higher than neighbor and neighbor isn't ocean
            if center_elevation > neighbor_elevation && neighbor_elevation >= sea_level {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a cell is coastal (adjacent to ocean but above sea level).
    fn is_coastal(
        &self,
        x: usize,
        y: usize,
        elevation_grid: &[f32],
        sea_level: f32,
        width: usize,
        height: usize,
    ) -> bool {
        let neighbors = self.get_cardinal_neighbors(x, y, width, height);
        
        for (nx, ny) in neighbors {
            let nidx = ny * width + nx;
            if elevation_grid[nidx] < sea_level {
                return true;
            }
        }
        
        false
    }
    
    /// Get cardinal (4-directional) neighbors.
    fn get_cardinal_neighbors(&self, x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        
        if x > 0 { neighbors.push((x - 1, y)); }
        if x < width - 1 { neighbors.push((x + 1, y)); }
        if y > 0 { neighbors.push((x, y - 1)); }
        if y < height - 1 { neighbors.push((x, y + 1)); }
        
        neighbors
    }
    
    /// Calculate settlement suitability score (0.0 to 1.0).
    fn calculate_suitability(
        &self,
        biome: BiomeType,
        climate: ClimateZone,
        elevation: f32,
        has_river: bool,
        has_coast: bool,
    ) -> f32 {
        let mut score: f32 = 0.5; // Base score
        let mut reasons = Vec::new();
        
        // Biome preference score
        let biome_score: f32 = match biome {
            // Excellent biomes
            BiomeType::TemperateGrassland 
            | BiomeType::TropicalSavanna
            | BiomeType::TemperateDeciduousForest
            | BiomeType::TemperateMixedForest => {
                reasons.push(SuitabilityReason::Excellent("Fertile plains".to_string()));
                0.9
            }
            // Good biomes
            BiomeType::SubtropicalSeasonalForest
            | BiomeType::TropicalSeasonalForest
            | BiomeType::TemperateSteppe
            | BiomeType::BorealForest => {
                reasons.push(SuitabilityReason::Good("Good agricultural potential".to_string()));
                0.7
            }
            // Acceptable biomes
            BiomeType::BorealTaiga
            | BiomeType::TropicalDryForest
            | BiomeType::SubtropicalSteppe
            | BiomeType::MontaneForest
            | BiomeType::MontaneGrassland
            | BiomeType::CoastalWetland => {
                reasons.push(SuitabilityReason::Acceptable("Viable with adaptation".to_string()));
                0.5
            }
            // Poor but usable biomes
            BiomeType::Mangrove
            | BiomeType::SemiAridSteppe
            | BiomeType::SubtropicalDesert => {
                reasons.push(SuitabilityReason::Negative("Limited agriculture".to_string()));
                0.25
            }
            // Everything else excluded above
            _ => 0.0,
        };
        
        score += biome_score * 0.4; // 40% weight on biome
        
        // River bonus
        if has_river {
            score += 0.15;
            reasons.push(SuitabilityReason::Excellent("Fresh water access".to_string()));
        }
        
        // Coastal bonus (with elevation constraint)
        if has_coast && elevation <= self.config.coastal_max_elevation {
            score += 0.1;
            reasons.push(SuitabilityReason::Good("Coastal trade access".to_string()));
        }
        
        // Elevation penalty
        if elevation > 2000.0 {
            score *= 0.7;
            reasons.push(SuitabilityReason::Negative("High altitude".to_string()));
        } else if elevation > 1000.0 {
            score *= 0.9;
            reasons.push(SuitabilityReason::Acceptable("Moderate elevation".to_string()));
        }
        
        // Climate penalties
        match climate {
            ClimateZone::Polar => {
                score *= 0.5;
                reasons.push(SuitabilityReason::Negative("Harsh polar climate".to_string()));
            }
            ClimateZone::Boreal => {
                score *= 0.8;
                reasons.push(SuitabilityReason::Acceptable("Short growing season".to_string()));
            }
            _ => {}
        }
        
        score.min(1.0f32)
    }
    
    /// Select sites with spacing constraint.
    fn select_sites(&mut self, mut sites: Vec<SettlementSite>, width: usize, height: usize) -> Vec<SettlementSite> {
        if sites.is_empty() {
            return Vec::new();
        }
        
        // Sort by score (density * suitability) descending
        sites.sort_by(|a, b| {
            let score_a = a.density * a.suitability;
            let score_b = b.density * b.suitability;
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        let mut selected = Vec::new();
        let min_spacing_sq = (self.config.min_spacing as f32).powi(2);
        let target_count = ((width * height) as f32 * self.config.density_target / 1000.0).ceil() as usize;
        
        for site in sites {
            // Check spacing constraint
            let too_close = selected.iter().any(|s: &SettlementSite| {
                let dx = s.x as f32 - site.x as f32;
                let dy = s.y as f32 - site.y as f32;
                dx * dx + dy * dy < min_spacing_sq
            });
            
            if !too_close {
                selected.push(site);
            }
            
            // Stop if we've reached target
            if selected.len() >= target_count {
                break;
            }
        }
        
        selected
    }
    
    /// Create Settlement entities from selected sites.
    fn create_settlements(&mut self, sites: Vec<SettlementSite>, width: usize, height: usize) -> Vec<Settlement> {
        sites
            .into_iter()
            .map(|site| {
                let settlement_type = self.pick_settlement_type(site.density);
                let population = self.pick_population(settlement_type, site.density);
                
                // Create location (convert cell coords to approximate geo coords)
                let lat = ((site.y as f64 / height as f64) * 180.0) - 90.0;
                let lon = ((site.x as f64 / width as f64) * 360.0) - 180.0;
                let location = GeoLocation::with_elevation(lat, lon, site.elevation_m);
                
                // Generate settlement name (placeholder - will use species/culture module)
                let name = self.generate_settlement_name(settlement_type);
                
                // Build description with settlement details
                let mut description = format!("{} on {}", site.biome.name(), site.climate.short_name());
                if site.has_river {
                    description.push_str(" (river)");
                }
                if site.has_coast {
                    description.push_str(" (coast)");
                }
                
                Settlement::with_details(
                    uuid::Uuid::new_v4(),
                    name,
                    settlement_type,
                    population,
                    location,
                    Some(description),
                )
            })
            .collect()
    }
    
    /// Pick settlement type based on density and RNG.
    fn pick_settlement_type(&mut self, density: f32) -> SettlementType {
        let r = self.rng.next_f64();
        
        // Adjust weights based on density (more people = larger settlements)
        let mut weights = self.config.size_weights.clone();
        
        if density > 0.7 {
            // High density: shift towards larger settlements
            weights.hamlet *= 0.5;
            weights.village *= 0.8;
            weights.town *= 1.2;
            weights.city *= 1.5;
            weights.metropolis *= 2.0;
        } else if density < 0.3 {
            // Low density: shift towards smaller settlements
            weights.hamlet *= 2.0;
            weights.village *= 1.5;
            weights.town *= 0.8;
            weights.city *= 0.5;
            weights.metropolis *= 0.2;
        }
        
        let total: f64 = weights.hamlet as f64 + weights.village as f64 + weights.town as f64 
            + weights.city as f64 + weights.metropolis as f64;
        let normalized = r * total;
        
        let mut cum = 0.0;
        if normalized < cum + weights.hamlet as f64 {
            return SettlementType::Hamlet;
        }
        cum += weights.hamlet as f64;
        if normalized < cum + weights.village as f64 {
            return SettlementType::Village;
        }
        cum += weights.village as f64;
        if normalized < cum + weights.town as f64 {
            return SettlementType::Town;
        }
        cum += weights.town as f64;
        if normalized < cum + weights.city as f64 {
            return SettlementType::City;
        }
        SettlementType::Metropolis
    }
    
    /// Pick population based on settlement type and density.
    fn pick_population(&mut self, settlement_type: SettlementType, density: f32) -> u64 {
        let base = match settlement_type {
            SettlementType::Hamlet => self.rng.next() as u64 % 90 + 10,
            SettlementType::Village => self.rng.next() as u64 % 900 + 100,
            SettlementType::Town => self.rng.next() as u64 % 9000 + 1000,
            SettlementType::City => self.rng.next() as u64 % 90000 + 10000,
            SettlementType::Metropolis => self.rng.next() as u64 % 900000 + 100000,
            SettlementType::Capital => self.rng.next() as u64 % 450000 + 50000,
            SettlementType::Fortress => self.rng.next() as u64 % 4900 + 100,
            SettlementType::Port => self.rng.next() as u64 % 49500 + 500,
            SettlementType::SacredSite => self.rng.next() as u64 % 9950 + 50,
        };
        
        // Scale by density (high density = larger settlements)
        let scale = 0.5 + density as f64;
        (base as f64 * scale) as u64
    }
    
    /// Generate a settlement name (placeholder implementation).
    fn generate_settlement_name(&mut self, settlement_type: SettlementType) -> String {
        // This will be replaced with proper name generation from species templates
        // For now, use a simple procedural name
        let prefixes = ["Gre", "Val", "Storm", "Iron", "Stone", "Oak", "River", "Sun", "Moon", "Star", "Frost", "High", "Low", "Old", "New", "Elder", "Young", "Silver", "Golden", "Black"];
        let suffixes = ["wood", "haven", "ford", "bridge", "vale", "dale", "moor", "field", "ham", "ton", "bury", "stead", "port", "mouth", "keep", "hold", "fall", "rise", "watch", "gate"];
        
        let prefix = prefixes[(self.rng.next() as usize) % prefixes.len()];
        let suffix = suffixes[(self.rng.next() as usize) % suffixes.len()];
        
        format!("{}{}", prefix, suffix)
    }
    
    /// Compute settlement statistics.
    fn compute_stats(&self, settlements: &[Settlement]) -> SettlementStats {
        let mut by_type: HashMap<SettlementType, usize> = HashMap::new();
        let mut by_biome: HashMap<BiomeType, usize> = HashMap::new();
        let mut coastal_count = 0;
        let mut river_count = 0;
        let mut total_population: u64 = 0;
        
        for s in settlements {
            if let Some(stype) = &s.settlement_type {
                *by_type.entry(*stype).or_insert(0) += 1;
            }
            if let Some(pop) = s.population {
                total_population += pop;
            }
            
            // Track coastal/river based on description
            if let Some(desc) = &s.description {
                if desc.contains("coast") {
                    coastal_count += 1;
                }
                if desc.contains("river") {
                    river_count += 1;
                }
            }
        }
        
        let average_population = if settlements.is_empty() {
            0.0
        } else {
            total_population as f64 / settlements.len() as f64
        };
        
        SettlementStats {
            total: settlements.len(),
            by_type,
            by_biome,
            coastal_count,
            river_count,
            average_population,
        }
    }
}

impl Settlement {
    /// Create a settlement with all details.
    pub fn with_details(
        id: uuid::Uuid,
        name: String,
        settlement_type: SettlementType,
        population: u64,
        location: GeoLocation,
        description: Option<String>,
    ) -> Self {
        let mut settlement = Self::new(id, name, location);
        settlement.settlement_type = Some(settlement_type);
        settlement.population = Some(population);
        settlement.description = description;
        settlement
    }
    
    /// Add species assignment to a settlement.
    pub fn with_species(mut self, species_id: SpeciesId) -> Self {
        self.species_id = Some(species_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_excluded_biomes() {
        assert!(SettlementGenerator::is_excluded_biome(BiomeType::HotDesert));
        assert!(SettlementGenerator::is_excluded_biome(BiomeType::Tundra));
        assert!(SettlementGenerator::is_excluded_biome(BiomeType::SnowGlacier));
        
        assert!(!SettlementGenerator::is_excluded_biome(BiomeType::TemperateGrassland));
        assert!(!SettlementGenerator::is_excluded_biome(BiomeType::TemperateDeciduousForest));
    }
    
    #[test]
    fn test_settlement_config_default() {
        let config = SettlementConfig::default();
        assert_eq!(config.density_target, 0.5);
        assert_eq!(config.min_spacing, 8);
        assert_eq!(config.max_attempts, 100);
    }
    
    #[test]
    fn test_density_map_access() {
        let map = DensityMap {
            width: 10,
            height: 10,
            values: vec![0.5; 100],
        };
        
        assert_eq!(map.get(0, 0), Some(0.5));
        assert_eq!(map.get(5, 5), Some(0.5));
        assert_eq!(map.get(10, 10), None); // Out of bounds
    }
    
    #[test]
    fn test_settlement_generation_determinism() {
        let mut gen1 = SettlementGenerator::new(SettlementConfig::default(), 12345);
        let mut gen2 = SettlementGenerator::new(SettlementConfig::default(), 12345);
        
        // Create simple test grid
        let width = 64;
        let height = 64;
        let elevation: Vec<f32> = vec![0.6; width * height]; // All land
        let biome: Vec<BiomeType> = vec![BiomeType::TemperateGrassland; width * height];
        let climate: Vec<ClimateZone> = vec![ClimateZone::Temperate; width * height];
        
        let result1 = gen1.generate(&elevation, &biome, &climate, 0.5, width, height, None);
        let result2 = gen2.generate(&elevation, &biome, &climate, 0.5, width, height, None);
        
        // Same seed should produce same settlements
        assert_eq!(result1.stats.total, result2.stats.total);
        assert_eq!(result1.settlements.len(), result2.settlements.len());
        
        // Names should match
        for (s1, s2) in result1.settlements.iter().zip(result2.settlements.iter()) {
            assert_eq!(s1.name, s2.name);
        }
    }
    
    // === Species-Aware Generation Tests ===
    // These tests verify species integration with the actual Species module.
    
    #[test]
    fn test_species_suitability_by_biome() {
        let species_data = SpeciesData::default_species();
        
        // Verify Humans thrive in temperate grasslands
        let human = species_data.get(SpeciesId::HUMAN).unwrap();
        assert!(human.inhabits(BiomeType::TemperateGrassland));
        assert!(human.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(!human.inhabits(BiomeType::Tundra));
        
        // Verify Elves are forest-dwelling
        let elf = species_data.get(SpeciesId::ELF).unwrap();
        assert!(elf.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(elf.inhabits(BiomeType::TropicalSeasonalForest));
        assert!(!elf.inhabits(BiomeType::HotDesert));
        
        // Verify Dwarves prefer boreal/mountain regions
        let dwarf = species_data.get(SpeciesId::DWARF).unwrap();
        assert!(dwarf.inhabits(BiomeType::BorealForest));
        assert!(dwarf.inhabits(BiomeType::MontaneForest));
        assert!(!dwarf.inhabits(BiomeType::TropicalSavanna));
    }
    
    #[test]
    fn test_settlement_species_assignment() {
        let species_data = SpeciesData::default_species();
        
        // Verify species assignment by biome
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::TemperateGrassland),
            Some(SpeciesId::HUMAN)
        );
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::TemperateDeciduousForest),
            Some(SpeciesId::ELF)
        );
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::BorealForest),
            Some(SpeciesId::DWARF)
        );
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::HotDesert),
            None // No species naturally inhabits hot desert
        );
    }
    
    #[test]
    fn test_species_name_generation() {
        use crate::util::Rng;
        use crate::util::Seed;
        
        let species_data = SpeciesData::default_species();
        let mut rng = Rng::new(Seed::new(42));
        
        // Generate names for different species
        let human_name = species_data.generate_name(SpeciesId::HUMAN, &mut rng);
        let elf_name = species_data.generate_name(SpeciesId::ELF, &mut rng);
        
        // Verify names are generated
        assert!(!human_name.is_empty());
        assert!(!elf_name.is_empty());
        assert_ne!(human_name, elf_name); // Different species = different names
        
        // Verify names end with valid suffixes
        let human = species_data.get(SpeciesId::HUMAN).unwrap();
        assert!(human.name_suffixes.iter().any(|s| human_name.ends_with(s)));
    }
    
    #[test]
    fn test_generate_with_species_integration() {
        // Integration test: generate settlements with species data
        let species_data = SpeciesData::default_species();
        let mut generator = SettlementGenerator::new(SettlementConfig::default(), 42);
        
        // Create test terrain (all temperate grassland)
        let width = 128;
        let height = 128;
        let elevation: Vec<f32> = vec![0.6; width * height];
        let biome: Vec<BiomeType> = vec![BiomeType::TemperateGrassland; width * height];
        let climate: Vec<ClimateZone> = vec![ClimateZone::Temperate; width * height];
        
        // Generate with species
        let result = generator.generate_with_species(
            &elevation,
            &biome,
            &climate,
            &species_data,
            0.5,
            width,
            height,
            None,
        );
        
        // Verify settlements were generated
        assert!(result.stats.total > 0, "Should generate settlements on temperate grassland");
        
        // Verify species assignment (Human for temperate grassland)
        for settlement in &result.settlements {
            assert!(settlement.species_id.is_some(), "Settlement should have species_id assigned");
            assert_eq!(settlement.species_id.unwrap(), SpeciesId::HUMAN, 
                "Settlements on grassland should be Human");
        }
    }
}