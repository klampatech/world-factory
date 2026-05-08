use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::claim::{TerritoryClaim, ContestedZone, FactionId};
use super::generator::PolygonInfo;

/// Manages contested zones created by wars
pub struct ContestedZoneManager {
    /// Duration in years before contested zone auto-resolves
    default_contest_duration: i32,
}

impl ContestedZoneManager {
    pub fn new(default_contest_duration: i32) -> Self {
        ContestedZoneManager {
            default_contest_duration,
        }
    }

    /// Create a new contested zone from a war
    pub fn create_contested_zone(
        &self,
        polygon_id: u64,
        year: i32,
        faction_ids: Vec<u64>,
    ) -> ContestedZone {
        ContestedZone::new(polygon_id, year, faction_ids)
    }

    /// Update control strength based on ongoing conflict
    pub fn update_control(
        &mut self,
        zone: &mut ContestedZone,
        faction_id: u64,
        strength_delta: f32,
        current_year: i32,
    ) {
        if zone.involved_factions.contains(&faction_id) {
            let current = zone.control_strength.get(&faction_id).copied().unwrap_or(0.0);
            let new_strength = (current + strength_delta).clamp(0.0, 1.0);
            zone.control_strength.insert(faction_id, new_strength);
        }
    }

    /// Check if a contested zone should be resolved
    pub fn should_resolve(&self, zone: &ContestedZone, current_year: i32) -> bool {
        let duration = current_year - zone.since_year;
        duration > self.default_contest_duration
    }

    /// Resolve a contested zone, returning control to dominant faction
    pub fn resolve_zone(&self, zone: &ContestedZone) -> Option<FactionId> {
        let mut max_strength = 0.0;
        let mut dominant_faction: Option<FactionId> = None;

        for (&faction_id, &strength) in &zone.control_strength {
            if strength > max_strength {
                max_strength = strength;
                dominant_faction = Some(FactionId::new(faction_id));
            }
        }

        dominant_faction
    }

    /// Find all polygons that could become contested zones
    pub fn find_contestable_polygons(
        &self,
        claims: &HashMap<FactionId, TerritoryClaim>,
        all_polygons: &HashMap<u64, PolygonInfo>,
        active_wars: &HashMap<(FactionId, FactionId), i32>, // war start year
    ) -> Vec<u64> {
        let mut contestable: Vec<u64> = Vec::new();

        // Get all warring faction pairs
        for ((fid1, fid2), _start_year) in active_wars {
            let claims1 = claims.get(fid1);
            let claims2 = claims.get(fid2);

            if let (Some(c1), Some(c2)) = (claims1, claims2) {
                // Find border polygons between warring factions
                for &poly1 in &c1.claimed_polygons {
                    if let Some(poly) = all_polygons.get(&poly1) {
                        for &neighbor in &poly.neighbors {
                            if c2.claimed_polygons.contains(&neighbor) {
                                // This is a border polygon between warring factions
                                if !contestable.contains(&poly1) {
                                    contestable.push(poly1);
                                }
                            }
                        }
                    }
                }
            }
        }

        contestable
    }

    /// Process all contested zones for current year
    pub fn process_contested_zones(
        &self,
        zones: &mut HashMap<u64, ContestedZone>,
        current_year: i32,
    ) -> Vec<u64> {
        let mut resolved: Vec<u64> = Vec::new();

        for (poly_id, zone) in zones.iter() {
            if self.should_resolve(zone, current_year) {
                resolved.push(*poly_id);
            }
        }

        for poly_id in &resolved {
            zones.remove(poly_id);
        }

        resolved
    }
}
