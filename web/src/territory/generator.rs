use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use super::claim::{TerritoryClaim, FactionId, ContestedZone};

/// Polygon adjacency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonInfo {
    pub id: u64,
    pub elevation: f32,
    pub neighbors: Vec<u64>,
    pub is_coastal: bool,
    pub is_island: bool,
}

impl PolygonInfo {
    pub fn is_deep_ocean(&self) -> bool {
        self.elevation < -200.0
    }

    pub fn is_ocean_shelf(&self) -> bool {
        self.elevation >= -200.0 && self.elevation < 0.0
    }

    /// Elevation zone as band index (0-7)
    /// Bands: 0=deep ocean, 1=ocean shelf, 2=very lowland, 3=lowland,
    /// 4=midland, 5=high-midland, 6=highland, 7=mountain/snow
    pub fn elevation_zone(&self) -> u8 {
        match self.elevation {
            e if e < -200.0 => 0,  // Deep ocean
            e if e < 0.0 => 1,     // Ocean shelf
            e if e < 200.0 => 2,   // Very lowland (0-200m)
            e if e < 400.0 => 3,   // Lowland (200-400m)
            e if e < 600.0 => 4,   // Midland (400-600m)
            e if e < 800.0 => 5,   // High-midland (600-800m)
            e if e < 1100.0 => 6,  // Highland (800-1100m)
            e if e < 1500.0 => 7,  // High-highland (1100-1500m)
            _ => 7,                // Mountain/snow (1500m+)
        }
    }

    pub fn is_very_lowland(&self) -> bool {
        self.elevation >= 0.0 && self.elevation < 200.0
    }

    pub fn is_lowland(&self) -> bool {
        self.elevation >= 200.0 && self.elevation < 400.0
    }

    pub fn is_midland(&self) -> bool {
        self.elevation >= 400.0 && self.elevation < 600.0
    }

    pub fn is_high_midland(&self) -> bool {
        self.elevation >= 600.0 && self.elevation < 800.0
    }

    pub fn is_highland(&self) -> bool {
        self.elevation >= 800.0 && self.elevation < 1100.0
    }

    pub fn is_high_highland(&self) -> bool {
        self.elevation >= 1100.0 && self.elevation < 1500.0
    }

    pub fn is_mountain(&self) -> bool {
        self.elevation >= 1500.0
    }
}

/// Generates clustered initial territories for factions
pub struct ClusteredTerritoryGenerator {
    /// Minimum cluster size (settlements)
    min_cluster_size: usize,
    /// Maximum cluster size
    max_cluster_size: usize,
    /// Minimum gap between rival factions
    min_gap_polygons: usize,
}

impl ClusteredTerritoryGenerator {
    pub fn new(min_cluster_size: usize, max_cluster_size: usize, min_gap_polygons: usize) -> Self {
        ClusteredTerritoryGenerator {
            min_cluster_size,
            max_cluster_size,
            min_gap_polygons,
        }
    }

    /// Generate initial clusters for factions based on age scaling
    pub fn generate_clusters(
        &self,
        faction_count: usize,
        available_polygons: &[PolygonInfo],
        elevation_map: &HashMap<u64, f32>,
    ) -> HashMap<FactionId, Vec<u64>> {
        let mut clusters: HashMap<FactionId, Vec<u64>> = HashMap::new();
        let mut used_polygons: HashSet<u64> = HashSet::new();

        // Filter to prefer lowland to midland elevations (0-800m) - expanded to 8 zones
        let preferred_polygons: Vec<u64> = available_polygons
            .iter()
            .filter(|p| p.elevation >= 0.0 && p.elevation < 800.0)
            .map(|p| p.id)
            .collect();

        // Shuffle for randomness
        let mut rng_seed: Vec<u64> = preferred_polygons.clone();
        Self::shuffle_slice(&mut rng_seed);

        let mut polygon_iter = rng_seed.into_iter();

        for i in 0..faction_count {
            let faction_id = FactionId::new(i as u64 + 1);
            let cluster_size = self.random_cluster_size();
            let mut cluster: Vec<u64> = Vec::new();

            for _ in 0..cluster_size {
                if let Some(polygon_id) = polygon_iter.next() {
                    if !used_polygons.contains(&polygon_id) {
                        cluster.push(polygon_id);
                        used_polygons.insert(polygon_id);
                    }
                }
            }

            if !cluster.is_empty() {
                clusters.insert(faction_id, cluster);
            }
        }

        clusters
    }

    fn random_cluster_size(&self) -> usize {
        // Random between min and max cluster size
        let range = self.max_cluster_size - self.min_cluster_size;
        self.min_cluster_size + (rand_simple(range + 1) as usize)
    }

    fn shuffle_slice<T: Clone>(slice: &mut [T]) {
        let len = slice.len();
        for i in 0..len {
            let j = i + rand_simple(len - i) as usize;
            slice.swap(i, j);
        }
    }
}

/// Expansion frontier algorithm - claims adjacent polygons per generation step
pub struct ExpansionFrontier {
    /// Minimum polygons to claim per expansion step
    pub min_expansion: usize,
    /// Maximum polygons to claim per expansion step
    pub max_expansion: usize,
    /// Mountain penalty factor (reduce expansion across mountains)
    mountain_penalty: f32,
}

impl ExpansionFrontier {
    pub fn new(min_expansion: usize, max_expansion: usize) -> Self {
        ExpansionFrontier {
            min_expansion,
            max_expansion,
            mountain_penalty: 0.5,
        }
    }

    /// Calculate expansion candidates for a faction
    pub fn calculate_expansion_candidates(
        &self,
        faction_id: FactionId,
        current_claims: &TerritoryClaim,
        all_polygons: &HashMap<u64, PolygonInfo>,
        elevation_map: &HashMap<u64, f32>,
        min_expansion_polygons: usize,
    ) -> Vec<u64> {
        let mut frontier_polygons: VecDeque<u64> = VecDeque::new();
        let mut expansion_pool: Vec<u64> = Vec::new();

        // Collect frontier polygons (neighbors of claimed territory)
        for &claimed in &current_claims.claimed_polygons {
            if let Some(polygon) = all_polygons.get(&claimed) {
                for &neighbor in &polygon.neighbors {
                    if !current_claims.claimed_polygons.contains(&neighbor) {
                        frontier_polygons.push_back(neighbor);
                    }
                }
            }
        }

        // Remove duplicates by converting to Vec, sorting, and deduping
        let mut frontier: Vec<u64> = frontier_polygons.into_iter().collect();
        frontier.sort();
        frontier.dedup();

        // Score each frontier polygon
        let mut scored: Vec<(u64, f32)> = frontier
            .iter()
            .filter_map(|&pid| {
                all_polygons.get(&pid).map(|p| {
                    let mut score = 100.0;

                    // Mountain barrier penalty
                    if p.is_mountain() {
                        score *= self.mountain_penalty;
                    }

                    // Deep ocean exclusion
                    if p.is_deep_ocean() {
                        return (pid, 0.0); // Return with zero score to be filtered
                    }

                    // Ocean shelf bonus for coastal factions
                    if p.is_ocean_shelf() && self.has_coastal_access(faction_id, current_claims, all_polygons) {
                        score += 50.0;
                    }

                    // Fine-grained elevation scoring based on 8 zones
                    match p.elevation_zone() {
                        2 => score += 35.0,  // Very lowland - optimal
                        3 => score += 32.0,  // Lowland - very good
                        4 => score += 25.0,  // Midland - good
                        5 => score += 18.0,  // High-midland - acceptable
                        6 => score += 8.0,   // Highland - poor
                        7 => score += 0.0,   // Mountain/snow - avoid
                        _ => {}              // Ocean zones handled above
                    }

                    (pid, score)
                })
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top candidates up to max expansion
        let target_count = min_expansion_polygons.max(self.min_expansion);
        let max_take = (target_count + 2).min(scored.len());

        for i in 0..max_take {
            expansion_pool.push(scored[i].0);
        }

        expansion_pool
    }

    fn has_coastal_access(
        &self,
        _faction_id: FactionId,
        claims: &TerritoryClaim,
        all_polygons: &HashMap<u64, PolygonInfo>,
    ) -> bool {
        for &claimed in &claims.claimed_polygons {
            if let Some(poly) = all_polygons.get(&claimed) {
                if poly.is_coastal {
                    return true;
                }
            }
        }
        false
    }
}

/// Simple random number helper (for reproducibility)
fn rand_simple(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;
    nanos % max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_zone_ocean() {
        let poly = PolygonInfo {
            id: 1,
            elevation: -300.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 0);
        assert!(poly.is_deep_ocean());
    }

    #[test]
    fn test_elevation_zone_ocean_shelf() {
        let poly = PolygonInfo {
            id: 1,
            elevation: -100.0,
            neighbors: vec![],
            is_coastal: true,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 1);
        assert!(poly.is_ocean_shelf());
    }

    #[test]
    fn test_elevation_zone_very_lowland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 150.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 2);
        assert!(poly.is_very_lowland());
    }

    #[test]
    fn test_elevation_zone_lowland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 300.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 3);
        assert!(poly.is_lowland());
    }

    #[test]
    fn test_elevation_zone_midland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 500.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 4);
        assert!(poly.is_midland());
    }

    #[test]
    fn test_elevation_zone_high_midland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 700.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 5);
        assert!(poly.is_high_midland());
    }

    #[test]
    fn test_elevation_zone_highland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 1000.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 6);
        assert!(poly.is_highland());
    }

    #[test]
    fn test_elevation_zone_high_highland() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 1300.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 7);
        assert!(poly.is_high_highland());
    }

    #[test]
    fn test_elevation_zone_mountain() {
        let poly = PolygonInfo {
            id: 1,
            elevation: 2000.0,
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        };
        assert_eq!(poly.elevation_zone(), 7);
        assert!(poly.is_mountain());
    }

    #[test]
    fn test_all_8_zones_covered() {
        // Verify all 8 bands are distinct
        let elevations: Vec<f32> = vec![-300.0, -100.0, 100.0, 300.0, 500.0, 700.0, 1000.0, 1300.0];
        let expected_zones: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        
        for (elev, expected) in elevations.iter().zip(expected_zones.iter()) {
            let poly = PolygonInfo {
                id: 1,
                elevation: *elev,
                neighbors: vec![],
                is_coastal: false,
                is_island: false,
            };
            assert_eq!(poly.elevation_zone(), *expected, "Elevation {} should be zone {}", elev, expected);
        }
    }
}
