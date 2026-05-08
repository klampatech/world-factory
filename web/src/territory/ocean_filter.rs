use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use super::generator::PolygonInfo;

/// Filters polygons based on ocean exclusion rules
pub struct OceanExclusionFilter {
    /// Deep ocean threshold (meters)
    deep_ocean_threshold: f32,
    /// Ocean shelf upper bound (meters)
    ocean_shelf_upper_bound: f32,
}

impl OceanExclusionFilter {
    pub fn new(deep_ocean_threshold: f32, ocean_shelf_upper_bound: f32) -> Self {
        OceanExclusionFilter {
            deep_ocean_threshold,
            ocean_shelf_upper_bound,
        }
    }

    /// Check if a polygon can be claimed by a faction
    /// - Factions cannot claim deep ocean tiles (-200m or below) unless holding adjacent island
    /// - Factions bordering sea may claim ocean shelf tiles (-200m to 0m) for fishing rights
    pub fn can_claim(
        &self,
        polygon_id: u64,
        faction_claims: &HashSet<u64>,
        all_polygons: &std::collections::HashMap<u64, PolygonInfo>,
    ) -> bool {
        let Some(polygon) = all_polygons.get(&polygon_id) else {
            return false;
        };

        // Deep ocean exclusion
        if polygon.elevation < self.deep_ocean_threshold {
            // Exception: can claim if faction holds adjacent island
            return self.has_adjacent_island(polygon_id, faction_claims, all_polygons);
        }

        true
    }

    /// Check if faction holds an island adjacent to the polygon
    fn has_adjacent_island(
        &self,
        polygon_id: u64,
        faction_claims: &HashSet<u64>,
        all_polygons: &std::collections::HashMap<u64, PolygonInfo>,
    ) -> bool {
        let Some(polygon) = all_polygons.get(&polygon_id) else {
            return false;
        };

        for &neighbor in &polygon.neighbors {
            // Check if neighbor is claimed and is an island
            if faction_claims.contains(&neighbor) {
                if let Some(neighbor_poly) = all_polygons.get(&neighbor) {
                    if neighbor_poly.is_island {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get all ocean polygons that can be claimed by a coastal faction
    pub fn get_claimable_ocean_polygons(
        &self,
        faction_claims: &HashSet<u64>,
        all_polygons: &std::collections::HashMap<u64, PolygonInfo>,
    ) -> HashSet<u64> {
        let mut claimable: HashSet<u64> = HashSet::new();

        for (poly_id, polygon) in all_polygons {
            // Only consider ocean shelf polygons
            if polygon.elevation >= self.deep_ocean_threshold
                && polygon.elevation < self.ocean_shelf_upper_bound
            {
                if self.can_claim(*poly_id, faction_claims, all_polygons) {
                    claimable.insert(*poly_id);
                }
            }
        }

        claimable
    }

    /// Check if faction has coastal access (can claim ocean tiles)
    pub fn has_coastal_access(
        &self,
        faction_claims: &HashSet<u64>,
        all_polygons: &std::collections::HashMap<u64, PolygonInfo>,
    ) -> bool {
        for &claimed_id in faction_claims {
            if let Some(poly) = all_polygons.get(&claimed_id) {
                if poly.is_coastal {
                    return true;
                }
            }
        }
        false
    }

    /// Get all excluded ocean polygons
    pub fn get_excluded_polygons(
        &self,
        all_polygons: &std::collections::HashMap<u64, PolygonInfo>,
        faction_claims: &HashSet<u64>,
    ) -> HashSet<u64> {
        let mut excluded: HashSet<u64> = HashSet::new();

        for (poly_id, polygon) in all_polygons {
            if polygon.elevation < self.deep_ocean_threshold {
                if !self.has_adjacent_island(*poly_id, faction_claims, all_polygons) {
                    excluded.insert(*poly_id);
                }
            }
        }

        excluded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_deep_ocean_exclusion() {
        let filter = OceanExclusionFilter::new(-200.0, 0.0);

        let mut polygons: HashMap<u64, PolygonInfo> = HashMap::new();
        polygons.insert(1, PolygonInfo {
            id: 1,
            elevation: -300.0, // Deep ocean
            neighbors: vec![],
            is_coastal: false,
            is_island: false,
        });
        polygons.insert(2, PolygonInfo {
            id: 2,
            elevation: 100.0, // Lowland
            neighbors: vec![1],
            is_coastal: true,
            is_island: false,
        });

        let faction_claims: HashSet<u64> = HashSet::new();

        // Cannot claim deep ocean without adjacent island
        assert!(!filter.can_claim(1, &faction_claims, &polygons));

        // Can claim lowland
        assert!(filter.can_claim(2, &faction_claims, &polygons));
    }

    #[test]
    fn test_adjacent_island_exception() {
        let filter = OceanExclusionFilter::new(-200.0, 0.0);

        let mut polygons: HashMap<u64, PolygonInfo> = HashMap::new();
        polygons.insert(1, PolygonInfo {
            id: 1,
            elevation: -300.0, // Deep ocean
            neighbors: vec![2],
            is_coastal: false,
            is_island: false,
        });
        polygons.insert(2, PolygonInfo {
            id: 2,
            elevation: 50.0,
            neighbors: vec![1, 3],
            is_coastal: true,
            is_island: true, // This is an island
        });
        polygons.insert(3, PolygonInfo {
            id: 3,
            elevation: 100.0,
            neighbors: vec![2],
            is_coastal: false,
            is_island: false,
        });

        let mut faction_claims: HashSet<u64> = HashSet::new();
        faction_claims.insert(2); // Faction holds the island

        // Can claim deep ocean because adjacent to owned island
        assert!(filter.can_claim(1, &faction_claims, &polygons));
    }
}
