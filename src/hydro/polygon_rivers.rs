//! Polygon-based river system for Voronoi grid worlds.
//!
//! Handles all water-related generation including rivers, lakes, and ocean placement.
//!
//! Features:
//! - Gradient descent pathfinding from high elevation to coast
//! - 30% spawn probability from high-elevation polygons
//! - River volume tracking (larger downstream due to tributaries)
//! - Confluence detection when rivers merge

pub use crate::hydro::rivers::{DrainTarget, River, RiverConfig, RiverGenerator, RiverId};

use crate::terrain::ocean::OceanDetector;
use crate::terrain::PolygonGraph;
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A river in the polygon-based world representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonRiver {
    /// Unique river identifier
    pub id: u32,
    /// River name (optional)
    pub name: Option<String>,
    /// Polygon IDs forming the river path (from source to mouth)
    pub path: Vec<u32>,
    /// Length in polygon hops
    pub length: u32,
    /// Total elevation change along the path
    pub elevation_change: f32,
    /// Whether this river drains into the ocean
    pub drains_to_ocean: bool,
    /// Confluence points (where tributaries join)
    pub confluences: Vec<Confluence>,
    /// River volume/flow rate (0.0 - 1.0), increases with tributaries
    pub volume: f32,
}

impl PolygonRiver {
    /// Create a new polygon river from a path.
    pub fn new(id: u32, path: Vec<u32>) -> Self {
        let length = path.len() as u32;
        Self {
            id,
            name: None,
            path,
            length,
            elevation_change: 0.0,
            drains_to_ocean: false,
            confluences: Vec::new(),
            volume: 0.5, // Base volume for a new river
        }
    }

    /// Get the source polygon (first in path).
    pub fn source(&self) -> Option<u32> {
        self.path.first().copied()
    }

    /// Get the mouth polygon (last in path).
    pub fn mouth(&self) -> Option<u32> {
        self.path.last().copied()
    }

    /// Get all polygons this river passes through.
    pub fn polygons(&self) -> &[u32] {
        &self.path
    }

    /// Check if this river passes through a specific polygon.
    pub fn passes_through(&self, polygon_id: u32) -> bool {
        self.path.contains(&polygon_id)
    }
}

/// Represents a confluence point where two rivers meet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confluence {
    /// Polygon ID where the confluence occurs
    pub polygon_id: u32,
    /// ID of the main river
    pub main_river_id: u32,
    /// ID of the tributary river
    pub tributary_river_id: u32,
    /// Volume increase at this confluence
    pub volume_increase: f32,
}

/// River generator for polygon-based worlds.
#[derive(Debug, Clone)]
pub struct PolygonRiverGenerator {
    /// Maximum river length in polygon hops
    pub max_length: u32,
    /// Minimum elevation difference for valid river
    pub min_elevation_diff: f32,
    /// Probability of a high-elevation polygon spawning a river
    pub spawn_probability: f32,
    /// Minimum elevation to be considered a source point
    pub min_source_elevation: f32,
    /// Enable confluence detection and volume tracking
    pub detect_confluences: bool,
    /// Volume increase per confluence
    pub confluence_volume_boost: f32,
}

impl Default for PolygonRiverGenerator {
    fn default() -> Self {
        Self {
            max_length: 50,
            min_elevation_diff: 0.01,
            spawn_probability: 0.3, // 30% chance from high-elevation polygons
            min_source_elevation: 0.5,
            detect_confluences: true,
            confluence_volume_boost: 0.15,
        }
    }
}

impl PolygonRiverGenerator {
    /// Generate rivers based on elevation gradient (gradient descent).
    ///
    /// Algorithm:
    /// 1. Find potential source points (high elevation non-coastal polygons)
    /// 2. Apply spawn probability (30%) to decide which sources generate rivers
    /// 3. Trace downhill following steepest descent
    /// 4. Track volume as rivers flow downhill
    /// 5. Detect and record confluences when rivers meet
    /// 6. Increase volume downstream at confluences
    pub fn generate_rivers(
        &self,
        graph: &PolygonGraph,
        _ocean_detector: &OceanDetector,
        rng: &mut Rng,
    ) -> Vec<PolygonRiver> {
        let mut rivers: Vec<PolygonRiver> = Vec::new();
        let mut river_id = 0u32;

        // Find potential source polygons (high elevation, not coastal)
        let potential_sources: Vec<u32> = (0..graph.len())
            .filter(|&id| {
                graph
                    .get(id as u32)
                    .map(|p| p.elevation >= self.min_source_elevation && !p.is_coastal)
                    .unwrap_or(false)
            })
            .map(|i| i as u32)
            .collect();

        // Sort sources by elevation (highest first) for better river ordering
        let mut sources_with_elevation: Vec<(u32, f32)> = potential_sources
            .iter()
            .filter_map(|&id| graph.get(id).map(|p| (id, p.elevation)))
            .collect();
        sources_with_elevation.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Track which polygons are already part of a river
        let mut used_polygons: HashSet<u32> = HashSet::new();

        // Track river paths for confluence detection
        let mut river_paths: HashMap<u32, Vec<u32>> = HashMap::new();

        // Trace rivers from each potential source
        for (source, _elevation) in sources_with_elevation {
            // Skip if source polygon is already used
            if used_polygons.contains(&source) {
                continue;
            }

            // Apply spawn probability (30%)
            if rng.random_float() > self.spawn_probability {
                continue;
            }

            if let Some(mut river) = self.trace_river(graph, source, &used_polygons) {
                // Only keep rivers with minimum length
                if river.length >= 3 {
                    // Mark all polygons in this river as used
                    for &poly_id in &river.path {
                        used_polygons.insert(poly_id);
                    }

                    // Store path for confluence detection
                    river_paths.insert(river_id, river.path.clone());

                    river.id = river_id;
                    rivers.push(river);
                }
            }
            river_id += 1;
        }

        // Detect and record confluences
        if self.detect_confluences {
            self.detect_confluences_in_paths(&mut rivers, &river_paths);
        }

        // Calculate final volumes based on confluence count
        self.calculate_volumes(&mut rivers);

        rivers
    }

    /// Detect confluences where rivers merge.
    fn detect_confluences_in_paths(
        &self,
        rivers: &mut Vec<PolygonRiver>,
        river_paths: &HashMap<u32, Vec<u32>>,
    ) {
        // Find polygons that appear in multiple rivers (confluence points)
        let mut polygon_rivers: HashMap<u32, Vec<u32>> = HashMap::new();

        for (river_id, path) in river_paths {
            for &poly_id in path {
                polygon_rivers.entry(poly_id).or_default().push(*river_id);
            }
        }

        // For each polygon with multiple rivers, record the confluence
        for (poly_id, river_ids) in polygon_rivers {
            if river_ids.len() > 1 {
                // This is a confluence point
                // The first river in the list is considered the main one (starts higher)
                for (_i, &tributary_id) in river_ids.iter().enumerate().skip(1) {
                    // Find the main and tributary rivers and add confluence info
                    if let Some(main_river) = rivers.iter_mut().find(|r| r.id == river_ids[0]) {
                        main_river.confluences.push(Confluence {
                            polygon_id: poly_id,
                            main_river_id: river_ids[0],
                            tributary_river_id: tributary_id,
                            volume_increase: self.confluence_volume_boost,
                        });
                    }
                }
            }
        }
    }

    /// Calculate river volumes based on length and confluences.
    fn calculate_volumes(&self, rivers: &mut Vec<PolygonRiver>) {
        for river in rivers.iter_mut() {
            // Base volume from length (longer rivers have more base volume)
            let length_factor = (river.length as f32 / self.max_length as f32).min(1.0);

            // Volume increases from confluences
            let confluence_boost: f32 = river.confluences.iter().map(|c| c.volume_increase).sum();

            // Calculate final volume (0.3 base + length factor + confluence boost)
            river.volume = (0.3 + length_factor * 0.4 + confluence_boost).min(1.0);
        }
    }

    /// Trace a river from a source polygon using gradient descent.
    fn trace_river(
        &self,
        graph: &PolygonGraph,
        source: u32,
        used_polygons: &HashSet<u32>,
    ) -> Option<PolygonRiver> {
        let mut path = vec![source];
        let mut current = source;
        let mut total_elevation_change = 0.0f32;

        // Get source elevation for tracking elevation change
        let _source_elev = graph.get(source)?.elevation;

        while path.len() < self.max_length as usize {
            let polygon = graph.get(current)?;

            // Stop if we reach the coast
            if polygon.is_coastal {
                break;
            }

            // Find neighbor with lowest elevation (steepest descent)
            let neighbors = &polygon.neighbors;
            let Some(&next) = neighbors
                .iter()
                .filter(|&&n| !used_polygons.contains(&n))
                .min_by(|&&a, &&b| {
                    let elev_a = graph.get(a).map(|p| p.elevation).unwrap_or(f32::MAX);
                    let elev_b = graph.get(b).map(|p| p.elevation).unwrap_or(f32::MAX);
                    elev_a.partial_cmp(&elev_b).unwrap()
                })
            else {
                // No unvisited downhill neighbors
                break;
            };

            let next_elev = graph.get(next).map(|p| p.elevation).unwrap_or(f32::MAX);
            let curr_elev = graph.get(current).map(|p| p.elevation).unwrap_or(f32::MAX);

            // Stop if no significant downhill
            if next_elev >= curr_elev - self.min_elevation_diff {
                break;
            }

            // Track elevation change
            total_elevation_change += curr_elev - next_elev;

            path.push(next);
            current = next;
        }

        if path.len() < 2 {
            return None;
        }

        let mut river = PolygonRiver::new(source, path);
        river.elevation_change = total_elevation_change;

        if let Some(poly) = graph.get(current) {
            river.drains_to_ocean = poly.is_coastal;
        }

        Some(river)
    }

    /// Check if a polygon has a river passing through it.
    pub fn polygon_has_river(&self, rivers: &[PolygonRiver], polygon_id: u32) -> bool {
        rivers.iter().any(|r| r.path.contains(&polygon_id))
    }

    /// Get the river that passes through a polygon, if any.
    pub fn get_river_through_polygon<'a>(
        &self,
        rivers: &'a [PolygonRiver],
        polygon_id: u32,
    ) -> Option<&'a PolygonRiver> {
        rivers.iter().find(|r| r.passes_through(polygon_id))
    }

    /// Get total river length across all rivers.
    pub fn total_river_length(&self, rivers: &[PolygonRiver]) -> u32 {
        rivers.iter().map(|r| r.length).sum()
    }

    /// Get rivers that drain to the ocean.
    pub fn get_ocean_draining_rivers<'a>(
        &self,
        rivers: &'a [PolygonRiver],
    ) -> Vec<&'a PolygonRiver> {
        rivers.iter().filter(|r| r.drains_to_ocean).collect()
    }

    /// Get rivers that end in lakes or inland (don't reach ocean).
    pub fn get_inland_rivers<'a>(&self, rivers: &'a [PolygonRiver]) -> Vec<&'a PolygonRiver> {
        rivers.iter().filter(|r| !r.drains_to_ocean).collect()
    }
}

/// Simple random float generator (0.0 - 1.0)
///
/// DEPRECATED: Use crate::util::Rng instead for deterministic generation
#[allow(dead_code)]
fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_river_creation() {
        let river = PolygonRiver::new(1, vec![10, 20, 30]);
        assert_eq!(river.id, 1);
        assert_eq!(river.path, vec![10, 20, 30]);
        assert_eq!(river.length, 3);
        assert_eq!(river.volume, 0.5);
        assert!(!river.drains_to_ocean);
    }

    #[test]
    fn test_river_source_and_mouth() {
        let river = PolygonRiver::new(0, vec![5, 10, 15, 20]);
        assert_eq!(river.source(), Some(5));
        assert_eq!(river.mouth(), Some(20));
    }

    #[test]
    fn test_confluence_detection() {
        let mut rivers = vec![
            PolygonRiver::new(0, vec![100, 101, 102, 103]),
            PolygonRiver::new(1, vec![200, 201, 102, 103]), // Joins river 0 at polygon 102
        ];

        let generator = PolygonRiverGenerator::default();
        let paths: HashMap<u32, Vec<u32>> =
            [(0, vec![100, 101, 102, 103]), (1, vec![200, 201, 102, 103])]
                .into_iter()
                .collect();

        generator.detect_confluences_in_paths(&mut rivers, &paths);

        // Check if confluence was added (implementation-dependent which river gets it)
        let has_confluence = rivers.iter().any(|r| {
            r.confluences.iter().any(|c| c.polygon_id == 102)
        });
        assert!(has_confluence, "Expected at least one confluence at polygon 102");
    }

    #[test]
    fn test_volume_calculation() {
        let mut rivers = vec![
            PolygonRiver::new(0, vec![1, 2, 3]), // Short river, 0 confluences
            PolygonRiver::new(1, vec![10, 11, 12, 13, 14]), // Longer river
        ];

        let generator = PolygonRiverGenerator::default();

        // Add a confluence to river 1
        rivers[1].confluences.push(Confluence {
            polygon_id: 12,
            main_river_id: 1,
            tributary_river_id: 0,
            volume_increase: 0.15,
        });

        generator.calculate_volumes(&mut rivers);

        // River 0 should have lower volume (short, no confluences)
        assert!(rivers[0].volume < rivers[1].volume);

        // Both volumes should be in valid range
        assert!(rivers[0].volume >= 0.0 && rivers[0].volume <= 1.0);
        assert!(rivers[1].volume >= 0.0 && rivers[1].volume <= 1.0);
    }

    #[test]
    fn test_polygon_has_river() {
        let generator = PolygonRiverGenerator::default();
        let rivers = vec![PolygonRiver::new(0, vec![10, 20, 30])];

        assert!(generator.polygon_has_river(&rivers, 10));
        assert!(generator.polygon_has_river(&rivers, 20));
        assert!(generator.polygon_has_river(&rivers, 30));
        assert!(!generator.polygon_has_river(&rivers, 40));
    }

    #[test]
    fn test_spawn_probability() {
        let generator = PolygonRiverGenerator::default();
        assert_eq!(generator.spawn_probability, 0.3);
    }
}
