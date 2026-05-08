use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::claim::{TerritoryClaim, ContestedZone, FactionId};
use super::generator::PolygonInfo;

/// Gap preservation rule - enforces minimum unclaimed polygons between rivals
pub struct GapPreservationRule {
    /// Minimum gap polygons between rival borders
    min_gap_polygons: usize,
}

impl GapPreservationRule {
    pub fn new(min_gap_polygons: usize) -> Self {
        GapPreservationRule {
            min_gap_polygons,
        }
    }

    /// Check if a polygon can be claimed without violating gap rules
    pub fn can_claim(
        &self,
        polygon_id: u64,
        faction_id: FactionId,
        current_claims: &HashMap<FactionId, TerritoryClaim>,
        all_polygons: &HashMap<u64, PolygonInfo>,
    ) -> bool {
        if !self.check_gap_distance(polygon_id, faction_id, current_claims, all_polygons) {
            return false;
        }

        // Cannot claim already claimed polygon
        for (other_fid, claim) in current_claims {
            if *other_fid != faction_id && claim.claimed_polygons.contains(&polygon_id) {
                return false;
            }
        }

        true
    }

    /// Verify minimum gap distance from rival borders
    fn check_gap_distance(
        &self,
        polygon_id: u64,
        faction_id: FactionId,
        current_claims: &HashMap<FactionId, TerritoryClaim>,
        all_polygons: &HashMap<u64, PolygonInfo>,
    ) -> bool {
        // BFS to find closest rival territory
        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: Vec<(u64, usize)> = vec![(polygon_id, 0)];
        visited.insert(polygon_id);

        while let Some((current, distance)) = queue.pop() {
            if distance > self.min_gap_polygons {
                continue;
            }

            if let Some(poly) = all_polygons.get(&current) {
                // Check if this polygon is claimed by a rival
                for (other_fid, claim) in current_claims {
                    if *other_fid != faction_id && claim.claimed_polygons.contains(&current) {
                        // Found rival within gap distance - violation!
                        if distance <= self.min_gap_polygons {
                            return false;
                        }
                    }
                }

                // Expand search to neighbors
                for &neighbor in &poly.neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push((neighbor, distance + 1));
                    }
                }
            }
        }

        true
    }

    /// Find gaps between factions that need preservation
    pub fn find_gaps(
        &self,
        claims: &HashMap<FactionId, TerritoryClaim>,
        all_polygons: &HashMap<u64, PolygonInfo>,
    ) -> Vec<u64> {
        let mut gap_polygons: Vec<u64> = Vec::new();
        let mut polygon_to_faction: HashMap<u64, FactionId> = HashMap::new();

        // Map each claimed polygon to its faction
        for (faction_id, claim) in claims {
            for &poly_id in &claim.claimed_polygons {
                polygon_to_faction.insert(poly_id, *faction_id);
            }
        }

        // Check each polygon for gap status
        for (&poly_id, poly) in all_polygons {
            if polygon_to_faction.contains_key(&poly_id) {
                continue; // Already claimed
            }

            // Count adjacent factions
            let mut adjacent_factions: HashSet<FactionId> = HashSet::new();
            for &neighbor in &poly.neighbors {
                if let Some(&fid) = polygon_to_faction.get(&neighbor) {
                    adjacent_factions.insert(fid);
                }
            }

            // If polygon is adjacent to multiple factions, it's a potential gap
            if adjacent_factions.len() >= 2 {
                gap_polygons.push(poly_id);
            }
        }

        gap_polygons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_distance_check() {
        let rule = GapPreservationRule::new(2);

        // Create test polygons
        let mut polygons: HashMap<u64, PolygonInfo> = HashMap::new();
        polygons.insert(1, PolygonInfo {
            id: 1,
            elevation: 100.0,
            neighbors: vec![2],
            is_coastal: false,
            is_island: false,
        });
        polygons.insert(2, PolygonInfo {
            id: 2,
            elevation: 100.0,
            neighbors: vec![1, 3],
            is_coastal: false,
            is_island: false,
        });

        let mut claims: HashMap<FactionId, TerritoryClaim> = HashMap::new();
        let mut claim1 = TerritoryClaim::default();
        claim1.claimed_polygons.insert(1);
        claims.insert(FactionId::new(1), claim1);

        // Should be able to claim polygon 2 (distance 1, gap is 2)
        // But since it's adjacent to faction 1's territory...
        let can_claim = rule.can_claim(2, FactionId::new(2), &claims, &polygons);
        assert!(can_claim, "Should be able to claim adjacent polygon with gap > claimed distance");
    }
}
