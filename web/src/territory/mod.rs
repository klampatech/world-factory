//! Territory module - Geographic territory management
//! 
//! Provides territory generation, expansion, and conflict resolution.
//! 
//! Key types:
//! - TerritorySystem: Main coordinator for all territory operations
//! - TerritoryClaim: Faction's territorial holdings
//! - FactionId: Unique faction identifier
//! - PolygonInfo: Geographic polygon metadata

use std::collections::{HashMap, HashSet};

pub mod claim;
pub mod generator;
pub mod contested_zone;
pub mod gap_rule;
pub mod ocean_filter;
pub mod scale;

// Re-exports
pub use claim::{TerritoryClaim, FactionId, ContestedZone, TerritoryGenerationResult};
pub use generator::{ClusteredTerritoryGenerator, ExpansionFrontier, PolygonInfo};

/// Main territory system coordinator
/// 
/// Coordinates territory generation, expansion, and conflict resolution
/// for all factions in the world.
pub struct TerritorySystem {
    /// Territory generator for initial cluster placement
    generator: ClusteredTerritoryGenerator,
    /// Expansion algorithm
    expansion: ExpansionFrontier,
    /// Gap rule enforcement
    min_gap_polygons: usize,
}

impl TerritorySystem {
    /// Create a new territory system with default parameters
    pub fn new() -> Self {
        Self {
            generator: ClusteredTerritoryGenerator::new(3, 8, 5),
            expansion: ExpansionFrontier::new(1, 3),
            min_gap_polygons: 5,
        }
    }

    /// Create a new territory system with custom parameters
    pub fn with_params(
        min_cluster: usize,
        max_cluster: usize,
        min_gap: usize,
    ) -> Self {
        Self {
            generator: ClusteredTerritoryGenerator::new(min_cluster, max_cluster, min_gap),
            expansion: ExpansionFrontier::new(1, 3),
            min_gap_polygons: min_gap,
        }
    }

    /// Generate initial territories for factions based on prehistory age
    /// 
    /// Prehistory age determines cluster sizes:
    /// - Ancient (300+ years): Large clusters (8-15 polygons)
    /// - Classical (100-300 years): Medium clusters (5-8 polygons)
    /// - Recent (<100 years): Small clusters (3-5 polygons)
    pub fn generate_initial_territories(
        &self,
        pre_history_years: u32,
        polygons: &[PolygonInfo],
        elevation_map: &HashMap<u64, f32>,
    ) -> HashMap<FactionId, TerritoryClaim> {
        // Determine faction count based on polygon count
        let polygon_count = polygons.len();
        let faction_count = Self::calculate_faction_count(polygon_count);
        
        // Determine cluster size range based on prehistory age
        let (min_cluster, max_cluster) = self.cluster_size_for_age(pre_history_years);
        
        let sized_generator = ClusteredTerritoryGenerator::new(min_cluster, max_cluster, self.min_gap_polygons);
        
        // Generate clusters
        let clusters = sized_generator.generate_clusters(faction_count, polygons, elevation_map);
        
        // Convert clusters to territory claims
        let mut claims: HashMap<FactionId, TerritoryClaim> = HashMap::new();
        
        for (faction_id, cluster) in clusters {
            let core: HashSet<u64> = cluster.iter().take(2).cloned().collect(); // First 2 = core
            
            claims.insert(faction_id, TerritoryClaim {
                claimed_polygons: cluster.into_iter().collect(),
                core_territory: core,
                client_states: Vec::new(),
                contested_zones: HashMap::new(),
                strait_controls: Vec::new(),
            });
        }
        
        claims
    }

    /// Expand territories for all factions
    /// 
    /// Called each generation step to expand faction territories
    /// based on population pressure and strategic factors.
    pub fn expand_territories(
        &self,
        claims: &mut HashMap<FactionId, TerritoryClaim>,
        all_polygons: &HashMap<u64, PolygonInfo>,
        elevation_map: &HashMap<u64, f32>,
        active_wars: &HashMap<(FactionId, FactionId), i32>,
        year: i32,
    ) {
        // Collect already-claimed polygon IDs once before any mutations
        let already_claimed: HashSet<u64> = claims.values()
            .flat_map(|c| c.claimed_polygons.iter())
            .cloned()
            .collect();
        
        for (faction_id, claim) in claims.iter_mut() {
            // Skip factions in wars (they defend rather than expand)
            let in_war = active_wars.keys().any(|(f1, f2)| *f1 == *faction_id || *f2 == *faction_id);
            if in_war {
                continue;
            }
            
            // Calculate expansion candidates
            let expansion_count = self.expansion.min_expansion;
            let candidates = self.expansion.calculate_expansion_candidates(
                *faction_id,
                claim,
                all_polygons,
                elevation_map,
                expansion_count,
            );
            
            // Claim top candidates (avoiding already-claimed zones)
            for polygon_id in candidates.into_iter().take(2) {
                if !already_claimed.contains(&polygon_id) {
                    claim.claimed_polygons.insert(polygon_id);
                }
            }
        }
    }

    /// Calculate appropriate faction count based on available polygons
    fn calculate_faction_count(polygon_count: usize) -> usize {
        // Rule: 1 faction per ~50 polygons at minimum viable size
        // But cap at reasonable faction counts
        let ideal = (polygon_count as f64 / 50.0).ceil() as usize;
        ideal.max(2).min(20) // At least 2, at most 20 factions
    }

    /// Determine cluster size range based on prehistory age
    fn cluster_size_for_age(&self, pre_history_years: u32) -> (usize, usize) {
        if pre_history_years >= 300 {
            // Ancient: large initial territories
            (8, 15)
        } else if pre_history_years >= 100 {
            // Classical: medium territories
            (5, 8)
        } else {
            // Recent: small territories
            (3, 5)
        }
    }

    /// Add a contested zone between factions
    pub fn add_contested_zone(
        &self,
        claims: &mut HashMap<FactionId, TerritoryClaim>,
        polygon_id: u64,
        factions: Vec<FactionId>,
        year: i32,
    ) {
        let zone = ContestedZone::new(
            polygon_id,
            year,
            factions.iter().map(|f| f.0).collect(),
        );
        
        for faction_id in &factions {
            if let Some(claim) = claims.get_mut(faction_id) {
                claim.contested_zones.insert(polygon_id, zone.clone());
            }
        }
    }
}

impl Default for TerritorySystem {
    fn default() -> Self {
        Self::new()
    }
}
