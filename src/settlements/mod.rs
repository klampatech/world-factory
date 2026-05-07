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

use crate::species::{SpeciesData, SpeciesId};
use crate::terrain::biome::{BiomeType, ClimateZone};
use crate::types::{EntityId, EntityType, GeoLocation, Settlement, SettlementType, Timestamp};
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    pub hamlet: f32,     // < 100 people
    pub village: f32,    // 100-1000 people
    pub town: f32,       // 1000-10000 people
    pub city: f32,       // 10000-100000 people
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
    pub score: f32, // 0.0 to 1.0, higher is better
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

/// A candidate settlement site with full scoring details.
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
    // Extended scoring fields per WOR-95 2.2.2
    pub freshwater_bonus: f32,  // Freshwater adjacency bonus
    pub soil_fertility: f32,    // Fertile soil bonus
    pub latitude_factor: f32,   // Temperate latitude bonus
    pub elevation_penalty: f32, // Extreme elevation penalty
    pub ocean_penalty: f32,     // Non-coastal ocean adjacency penalty
}

impl SettlementSite {
    /// Get full suitability score with reasons per WOR-95 2.2.2.
    pub fn get_suitability_score(&self) -> SuitabilityScore {
        let mut score = self.suitability;
        let mut reasons = Vec::new();

        // Freshwater adjacency bonus (+50%)
        if self.freshwater_bonus > 0.0 {
            score += self.freshwater_bonus * 0.5;
            reasons.push(SuitabilityReason::Excellent(
                "Fresh water access (+50%)".to_string(),
            ));
        }

        // Fertile soil bonus (+30%)
        if self.soil_fertility > 0.0 {
            score += self.soil_fertility * 0.3;
            reasons.push(SuitabilityReason::Good(
                "Fertile soil bonus (+30%)".to_string(),
            ));
        }

        // Temperate latitude bonus (+20%)
        if self.latitude_factor > 0.0 {
            score += self.latitude_factor * 0.2;
            reasons.push(SuitabilityReason::Good(
                "Temperate latitude bonus (+20%)".to_string(),
            ));
        }

        // Extreme elevation penalty (×0.5)
        if self.elevation_penalty > 0.0 {
            score *= (1.0 - self.elevation_penalty * 0.5).max(0.1);
            reasons.push(SuitabilityReason::Negative(
                "Extreme elevation penalty".to_string(),
            ));
        }

        // Ocean adjacency penalty for non-coastal (−30%)
        if self.ocean_penalty > 0.0 {
            score *= (1.0 - self.ocean_penalty * 0.3).max(0.1);
            reasons.push(SuitabilityReason::Negative(
                "Ocean proximity penalty (-30%)".to_string(),
            ));
        }

        SuitabilityScore {
            score: score.min(1.0).max(0.0),
            reasons,
            settlement_type: None,
        }
    }
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
        let _total_cells = width * height;

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
        let settlements =
            self.create_settlements_with_species(selected_sites, species_data, width, height);

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
                let species_id = species_data
                    .best_species_for_biome(site.biome)
                    .unwrap_or(SpeciesId::Human);

                // Generate culturally-appropriate name
                let name = species_data.generate_name(species_id, &mut self.rng);

                // Build description
                let species_name = species_data
                    .get(species_id)
                    .map(|s| s.name.as_ref())
                    .unwrap_or("Unknown");
                let mut description = format!(
                    "{} settlement on {} ({})",
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

                // Calculate polygon_id from cell coordinates
                let polygon_id: u32 = (site.y * width + site.x) as u32;

                // Create settlement with species
                let mut settlement = Settlement::with_details(
                    uuid::Uuid::new_v4(),
                    Some(polygon_id),
                    name,
                    settlement_type,
                    population,
                    location,
                    Some(description),
                )
                .with_species(species_id);

                // Assign carrying capacity based on biome
                settlement.carrying_capacity =
                    Some(Settlement::calculate_carrying_capacity(site.biome));

                // Set founding year (year 0 for initial settlements)
                settlement.founded_year = Some(0);

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
        let detail_scale = 0.0625; // 1/16

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

        DensityMap {
            width,
            height,
            values,
        }
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

                // Calculate suitability score with extended scoring per WOR-95 2.2.2
                let (
                    suitability,
                    freshwater_bonus,
                    soil_fertility,
                    latitude_factor,
                    elevation_penalty,
                    ocean_penalty,
                ) = self.calculate_extended_suitability(
                    biome,
                    climate_grid[idx],
                    elevation,
                    has_river,
                    has_coast,
                    y,
                    height,
                    width,
                    x,
                    elevation_grid,
                    sea_level,
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
                            freshwater_bonus,
                            soil_fertility,
                            latitude_factor,
                            elevation_penalty,
                            ocean_penalty,
                        });
                    }
                }
            }
        }

        sites
    }

    /// Calculate suitability with extended scoring per WOR-95 2.2.2.
    fn calculate_extended_suitability(
        &mut self,
        biome: BiomeType,
        climate: ClimateZone,
        elevation: f32,
        has_river: bool,
        has_coast: bool,
        y: usize,
        height: usize,
        width: usize,
        x: usize,
        elevation_grid: &[f32],
        sea_level: f32,
    ) -> (f32, f32, f32, f32, f32, f32) {
        let mut base_suitability: f32 = 0.0;
        let mut freshwater_bonus: f32 = 0.0;
        let mut soil_fertility: f32 = 0.0;
        let mut latitude_factor: f32 = 0.0;
        let mut elevation_penalty: f32 = 0.0;
        let mut ocean_penalty: f32 = 0.0;

        // Base score from biome carrying capacity (> 0 = habitable)
        let carrying_capacity = Settlement::calculate_carrying_capacity(biome) as f32;
        if carrying_capacity > 0.0 {
            base_suitability = (carrying_capacity / 7000.0).min(1.0) * 0.6;
        } else {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Freshwater adjacency bonus (+50%)
        if has_river {
            freshwater_bonus = 0.5;
            base_suitability += 0.15;
        }

        // Fertile soil bonus (+30%)
        soil_fertility = match biome {
            BiomeType::TemperateGrassland | BiomeType::TropicalSavanna => 1.0,
            BiomeType::TemperateDeciduousForest | BiomeType::TemperateMixedForest => 0.8,
            BiomeType::SubtropicalSeasonalForest | BiomeType::TropicalSeasonalForest => 0.7,
            _ => 0.3,
        };
        base_suitability += soil_fertility * 0.1;

        // Temperate latitude bonus (+20%)
        let latitude = (y as f32 / height as f32) * 90.0;
        if (35.0..55.0).contains(&latitude) {
            latitude_factor = 0.2;
            base_suitability += 0.1;
        } else if (23.0..35.0).contains(&latitude) || (55.0..65.0).contains(&latitude) {
            latitude_factor = 0.1;
            base_suitability += 0.05;
        }

        // Extreme elevation penalty (x0.5)
        if elevation > 3000.0 {
            elevation_penalty = 1.0;
        } else if elevation > 2000.0 {
            elevation_penalty = 0.7;
        } else if elevation > 1500.0 {
            elevation_penalty = 0.3;
        }
        if elevation_penalty > 0.0 {
            base_suitability *= (1.0 - elevation_penalty * 0.5).max(0.1);
        }

        // Ocean adjacency penalty for non-coastal (-30%)
        if !has_coast && !has_river {
            let ocean_neighbors =
                self.count_adjacent_ocean(x, y, elevation_grid, sea_level, width, height);
            if ocean_neighbors >= 3 {
                ocean_penalty = 0.3;
                base_suitability *= 0.7;
            }
        }

        // Coastal bonus
        if has_coast && elevation <= self.config.coastal_max_elevation {
            base_suitability += 0.1;
        }

        // Climate penalties
        match climate {
            ClimateZone::Polar => base_suitability *= 0.5,
            ClimateZone::Boreal => base_suitability *= 0.8,
            _ => {}
        }

        (
            base_suitability.min(1.0),
            freshwater_bonus,
            soil_fertility,
            latitude_factor,
            elevation_penalty,
            ocean_penalty,
        )
    }

    /// Count adjacent ocean cells.
    fn count_adjacent_ocean(
        &self,
        x: usize,
        y: usize,
        elevation_grid: &[f32],
        sea_level: f32,
        width: usize,
        height: usize,
    ) -> usize {
        let neighbors = self.get_cardinal_neighbors(x, y, width, height);
        neighbors
            .iter()
            .filter(|(nx, ny)| {
                let idx = ny * width + nx;
                elevation_grid[idx] < sea_level
            })
            .count()
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
    fn get_cardinal_neighbors(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();

        if x > 0 {
            neighbors.push((x - 1, y));
        }
        if x < width - 1 {
            neighbors.push((x + 1, y));
        }
        if y > 0 {
            neighbors.push((x, y - 1));
        }
        if y < height - 1 {
            neighbors.push((x, y + 1));
        }

        neighbors
    }

    /// Calculate settlement suitability score (0.0 to 1.0).
    /// Deprecated: Use calculate_extended_suitability instead for WOR-95 compliance.
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
                reasons.push(SuitabilityReason::Good(
                    "Good agricultural potential".to_string(),
                ));
                0.7
            }
            // Acceptable biomes
            BiomeType::BorealTaiga
            | BiomeType::TropicalDryForest
            | BiomeType::SubtropicalSteppe
            | BiomeType::MontaneForest
            | BiomeType::MontaneGrassland
            | BiomeType::CoastalWetland => {
                reasons.push(SuitabilityReason::Acceptable(
                    "Viable with adaptation".to_string(),
                ));
                0.5
            }
            // Poor but usable biomes
            BiomeType::Mangrove | BiomeType::SemiAridSteppe | BiomeType::SubtropicalDesert => {
                reasons.push(SuitabilityReason::Negative(
                    "Limited agriculture".to_string(),
                ));
                0.25
            }
            // Everything else excluded above
            _ => 0.0,
        };

        score += biome_score * 0.4; // 40% weight on biome

        // River bonus
        if has_river {
            score += 0.15;
            reasons.push(SuitabilityReason::Excellent(
                "Fresh water access".to_string(),
            ));
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
            reasons.push(SuitabilityReason::Acceptable(
                "Moderate elevation".to_string(),
            ));
        }

        // Climate penalties
        match climate {
            ClimateZone::Polar => {
                score *= 0.5;
                reasons.push(SuitabilityReason::Negative(
                    "Harsh polar climate".to_string(),
                ));
            }
            ClimateZone::Boreal => {
                score *= 0.8;
                reasons.push(SuitabilityReason::Acceptable(
                    "Short growing season".to_string(),
                ));
            }
            _ => {}
        }

        score.min(1.0f32)
    }

    /// Select sites with spacing constraint.
    fn select_sites(
        &mut self,
        mut sites: Vec<SettlementSite>,
        width: usize,
        height: usize,
    ) -> Vec<SettlementSite> {
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
        let target_count =
            ((width * height) as f32 * self.config.density_target / 1000.0).ceil() as usize;

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
    fn create_settlements(
        &mut self,
        sites: Vec<SettlementSite>,
        width: usize,
        height: usize,
    ) -> Vec<Settlement> {
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
                let mut description =
                    format!("{} on {}", site.biome.name(), site.climate.short_name());
                if site.has_river {
                    description.push_str(" (river)");
                }
                if site.has_coast {
                    description.push_str(" (coast)");
                }

                // Calculate polygon_id from cell coordinates
                let polygon_id: u32 = (site.y * width + site.x) as u32;

                let mut settlement = Settlement::with_details(
                    uuid::Uuid::new_v4(),
                    Some(polygon_id),
                    name,
                    settlement_type,
                    population,
                    location,
                    Some(description),
                );

                // Assign carrying capacity based on biome
                settlement.carrying_capacity =
                    Some(Settlement::calculate_carrying_capacity(site.biome));

                // Set founding year (year 0 for initial settlements)
                settlement.founded_year = Some(0);

                settlement
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

        let total: f64 = weights.hamlet as f64
            + weights.village as f64
            + weights.town as f64
            + weights.city as f64
            + weights.metropolis as f64;
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

    /// Generate a settlement name using multi-syllable procedural generation.
    /// Combines linguistic patterns for culturally-appropriate names.
    fn generate_settlement_name(&mut self, settlement_type: SettlementType) -> String {
        // Multi-syllable name generation (per WOR-95 spec)
        // Patterns: prefix + syllable + suffix, with optional secondary suffix

        // Base syllables for settlement names
        let syllables_1 = [
            "Al", "Val", "Storm", "Iron", "Stone", "Oak", "River", "Sun", "Moon", "Star", "Frost",
            "Silver", "Golden", "Black", "White", "Green", "Red", "High", "Low", "Old", "New",
            "Elder", "Young", "North", "South", "East", "West",
        ];
        let syllables_2 = [
            "an", "ar", "or", "en", "in", "on", "ia", "ea", "oa", "ael", "ir", "or", "um", "al",
            "el", "il", "ol",
        ];
        let syllables_3 = [
            "wood", "haven", "ford", "bridge", "vale", "dale", "moor", "field", "ham", "ton",
            "bury", "stead", "port", "mouth", "keep", "hold", "fall", "rise", "watch", "gate",
            "heim", "burg", "ton", "worth", "wick", "ford", "dale", "ville", "burg", "stadt",
        ];

        // Secondary suffixes for larger settlements
        let secondary_suffix = ["-el-", "-ar-", "-in-", "-", ""];
        let extra_syllables = ["ar", "an", "ia", "os", "us", "el", "in", "on"];

        // Generate name based on settlement size (larger = longer name)
        let use_extended = match settlement_type {
            SettlementType::Metropolis | SettlementType::Capital => true,
            SettlementType::City => self.rng.next_f64() > 0.3,
            SettlementType::Town => self.rng.next_f64() > 0.6,
            _ => false,
        };

        let prefix_idx = (self.rng.next() as usize) % syllables_1.len();
        let mid_idx = (self.rng.next() as usize) % syllables_2.len();
        let suffix_idx = (self.rng.next() as usize) % syllables_3.len();

        let mut name = format!("{}{}", syllables_1[prefix_idx], syllables_2[mid_idx]);

        if use_extended {
            // Add extra syllable for larger settlements
            let sep_idx = (self.rng.next() as usize) % secondary_suffix.len();
            let extra_idx = (self.rng.next() as usize) % extra_syllables.len();
            name.push_str(&format!(
                "{}{}",
                secondary_suffix[sep_idx], extra_syllables[extra_idx]
            ));
        }

        name.push_str(syllables_3[suffix_idx]);

        // Capitalize first letter
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            let capitalized = first.to_uppercase().to_string();
            capitalized + chars.as_str()
        } else {
            name
        }
    }

    /// Compute settlement statistics.
    fn compute_stats(&self, settlements: &[Settlement]) -> SettlementStats {
        let mut by_type: HashMap<SettlementType, usize> = HashMap::new();
        let by_biome: HashMap<BiomeType, usize> = HashMap::new();
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
        polygon_id: Option<u32>,
        name: String,
        settlement_type: SettlementType,
        population: u64,
        location: GeoLocation,
        description: Option<String>,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::from_uuid(id, EntityType::Settlement),
            region_id: Uuid::nil(), // Will be set by caller
            polygon_id,
            name,
            settlement_type: Some(settlement_type),
            population: Some(population),
            location,
            species_id: None,
            description,
            notable_features: None,
            carrying_capacity: None,
            founded_year: None,
            society_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a settlement with full details per WOR-95 2.2.3.
    /// Note: Renamed from with_full_details to avoid conflict with Settlement impl.
    pub fn create_with_full_details(
        id: uuid::Uuid,
        region_id: Uuid,
        polygon_id: Option<u32>,
        name: String,
        settlement_type: SettlementType,
        population: u64,
        location: GeoLocation,
        species_id: Option<SpeciesId>,
        carrying_capacity: u64,
        founded_year: i32,
        society_id: Option<Uuid>,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::from_uuid(id, EntityType::Settlement),
            region_id,
            name,
            polygon_id,
            settlement_type: Some(settlement_type),
            population: Some(population),
            location,
            species_id,
            description: None,
            notable_features: None,
            carrying_capacity: Some(carrying_capacity),
            founded_year: Some(founded_year),
            society_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add species assignment to a settlement.
    pub fn with_species(mut self, species_id: SpeciesId) -> Self {
        self.species_id = Some(species_id);
        self
    }

    /// Add society assignment to a settlement.
    pub fn with_society(mut self, society_id: Uuid) -> Self {
        self.society_id = Some(society_id);
        self
    }

    /// Set the founding year of a settlement.
    pub fn with_founded_year(mut self, year: i32) -> Self {
        self.founded_year = Some(year);
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
        assert!(SettlementGenerator::is_excluded_biome(
            BiomeType::SnowGlacier
        ));

        assert!(!SettlementGenerator::is_excluded_biome(
            BiomeType::TemperateGrassland
        ));
        assert!(!SettlementGenerator::is_excluded_biome(
            BiomeType::TemperateDeciduousForest
        ));
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
        let human = species_data.get(SpeciesId::Human).unwrap();
        assert!(human.inhabits(BiomeType::TemperateGrassland));
        assert!(human.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(!human.inhabits(BiomeType::Tundra));

        // Verify Elves are forest-dwelling
        let elf = species_data.get(SpeciesId::Elf).unwrap();
        assert!(elf.inhabits(BiomeType::TemperateDeciduousForest));
        assert!(elf.inhabits(BiomeType::TropicalSeasonalForest));
        assert!(!elf.inhabits(BiomeType::HotDesert));

        // Verify Dwarves prefer boreal/mountain regions
        let dwarf = species_data.get(SpeciesId::Dwarf).unwrap();
        assert!(dwarf.inhabits(BiomeType::BorealForest));
        assert!(dwarf.inhabits(BiomeType::MontaneForest));
        assert!(!dwarf.inhabits(BiomeType::TropicalSavanna));
    }

    #[test]
    fn test_settlement_species_assignment() {
        let species_data = SpeciesData::default_species();

        // Verify species assignment by biome - some biomes have multiple suitable species
        assert!(matches!(
            species_data.best_species_for_biome(BiomeType::TemperateGrassland),
            Some(SpeciesId::Human) | Some(SpeciesId::Halfling)
        ));
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::TemperateDeciduousForest),
            Some(SpeciesId::Elf)
        );
        assert_eq!(
            species_data.best_species_for_biome(BiomeType::BorealForest),
            Some(SpeciesId::Dwarf)
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
        let mut rng = Rng::new(42);

        // Generate names for different species
        let human_name = species_data.generate_name(SpeciesId::Human, &mut rng);
        let elf_name = species_data.generate_name(SpeciesId::Elf, &mut rng);

        // Verify names are generated
        assert!(!human_name.is_empty());
        assert!(!elf_name.is_empty());
        assert_ne!(human_name, elf_name); // Different species = different names

        // Verify names end with valid suffixes
        let human = species_data.get(SpeciesId::Human).unwrap();
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
        assert!(
            result.stats.total > 0,
            "Should generate settlements on temperate grassland"
        );

        // Verify species assignment (Human or Halfling for temperate grassland)
        for settlement in &result.settlements {
            assert!(
                settlement.species_id.is_some(),
                "Settlement should have species_id assigned"
            );
            assert!(
                matches!(
                    settlement.species_id.unwrap(),
                    SpeciesId::Human | SpeciesId::Halfling
                ),
                "Settlements on grassland should be Human or Halfling"
            );

            // Verify carrying capacity is assigned
            assert!(
                settlement.carrying_capacity.is_some(),
                "Settlement should have carrying_capacity assigned"
            );
        }
    }

    #[test]
    fn test_carrying_capacity_varies_by_biome() {
        // Test that carrying capacity differs by biome type per WOR-95 spec
        // High: TropicalRainforest (7000) > TemperateForest (5000) > BorealForest (1500) > Desert (200)
        let tropical_rainforest =
            Settlement::calculate_carrying_capacity(BiomeType::TropicalRainforest);
        let temperate_forest =
            Settlement::calculate_carrying_capacity(BiomeType::TemperateDeciduousForest);
        let boreal_forest = Settlement::calculate_carrying_capacity(BiomeType::BorealForest);
        let hot_desert = Settlement::calculate_carrying_capacity(BiomeType::HotDesert);

        // Verify hierarchy
        assert!(
            tropical_rainforest > temperate_forest,
            "Tropical Rainforest ({}) should exceed Temperate Forest ({})",
            tropical_rainforest,
            temperate_forest
        );
        assert!(
            temperate_forest > boreal_forest,
            "Temperate Forest ({}) should exceed Boreal Forest ({})",
            temperate_forest,
            boreal_forest
        );
        assert!(
            boreal_forest > hot_desert,
            "Boreal Forest ({}) should exceed Hot Desert ({})",
            boreal_forest,
            hot_desert
        );
    }

    #[test]
    fn test_carrying_capacity_values() {
        // Verify specific carrying capacity values per WOR-95 spec
        // Uninhabitable
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::OpenOcean),
            0
        );
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::Arctic),
            0
        );
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::SnowGlacier),
            0
        );

        // Low
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::HotDesert),
            200
        );
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::Tundra),
            300
        );

        // Medium-low
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::TemperateSteppe),
            2000
        );

        // Medium
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::BorealForest),
            1500
        );
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::TropicalSavanna),
            3000
        );

        // High
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::TemperateDeciduousForest),
            5000
        );
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::TemperateRainforest),
            6000
        );

        // Highest
        assert_eq!(
            Settlement::calculate_carrying_capacity(BiomeType::TropicalRainforest),
            7000
        );
    }
}
// Additional tests for WOR-95 2.2.3 settlement fields

#[cfg(test)]
mod settlement_entity_tests {
    use super::*;

    #[test]
    fn test_settlement_founded_year_assignment() {
        let mut generator = SettlementGenerator::new(SettlementConfig::default(), 42);

        // Create test terrain
        let width = 64;
        let height = 64;
        let elevation: Vec<f32> = vec![0.6; width * height];
        let biome: Vec<BiomeType> = vec![BiomeType::TemperateGrassland; width * height];
        let climate: Vec<ClimateZone> = vec![ClimateZone::Temperate; width * height];

        let result = generator.generate(&elevation, &biome, &climate, 0.5, width, height, None);

        // Verify settlements have founding year set
        for settlement in &result.settlements {
            assert!(
                settlement.founded_year.is_some(),
                "Settlement {} should have founded_year",
                settlement.name
            );
            assert_eq!(
                settlement.founded_year.unwrap(),
                0,
                "Initial settlements should have founded_year of 0"
            );
        }
    }

    #[test]
    fn test_settlement_with_full_details() {
        use uuid::Uuid;

        let settlement = Settlement::with_full_details(
            Uuid::new_v4(),
            Uuid::nil(),
            None, // polygon_id
            "TestTown".to_string(),
            SettlementType::Town,
            5000,
            GeoLocation::new(45.0, -122.0),
            Some(SpeciesId::Human),
            5000, // carrying capacity
            150,  // founded year
            None, // no society yet
        );

        assert_eq!(settlement.name, "TestTown");
        assert_eq!(settlement.population, Some(5000));
        assert_eq!(settlement.carrying_capacity, Some(5000));
        assert_eq!(settlement.founded_year, Some(150));
        assert_eq!(settlement.species_id, Some(SpeciesId::Human));
    }

    #[test]
    fn test_settlement_with_society() {
        use uuid::Uuid;

        let society_id = Uuid::new_v4();
        let settlement = Settlement::new(
            Uuid::nil(),
            "Village".to_string(),
            GeoLocation::new(40.0, -75.0),
        )
        .with_society(society_id);

        assert_eq!(settlement.society_id, Some(society_id));
    }
}

// Settlement name generator tests
#[cfg(test)]
mod settlement_name_tests {
    use super::*;

    #[test]
    fn test_generate_settlement_name_creates_names() {
        let mut generator = SettlementGenerator::new(SettlementConfig::default(), 42);

        let name = generator.generate_settlement_name(SettlementType::Village);

        // Verify name is not empty
        assert!(!name.is_empty(), "Name should not be empty");
        assert!(name.len() >= 4, "Name should be at least 4 characters");

        // Verify first letter is uppercase
        assert!(
            name.chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false),
            "First letter should be uppercase"
        );
    }

    #[test]
    fn test_generate_settlement_name_determinism() {
        let mut generator1 = SettlementGenerator::new(SettlementConfig::default(), 42);
        let mut generator2 = SettlementGenerator::new(SettlementConfig::default(), 42);

        let name1 = generator1.generate_settlement_name(SettlementType::Town);
        let name2 = generator2.generate_settlement_name(SettlementType::Town);

        // Same seed should produce same name
        assert_eq!(name1, name2, "Same seed should produce same name");
    }

    #[test]
    fn test_generate_settlement_name_different_types() {
        let mut generator = SettlementGenerator::new(SettlementConfig::default(), 123);

        // Generate names for different settlement types
        let hamlet_name = generator.generate_settlement_name(SettlementType::Hamlet);
        let village_name = generator.generate_settlement_name(SettlementType::Village);
        let city_name = generator.generate_settlement_name(SettlementType::City);

        // Verify all names are different (due to RNG state advancement)
        assert_ne!(hamlet_name, village_name);
        assert_ne!(village_name, city_name);
    }
}
