//! Tectonic Elevation Effects
//!
//! Defines the relationship between tectonic boundaries and terrain elevation.
//! Calculates uplift/subsidence effects based on plate movement and boundary type.

use serde::{Deserialize, Serialize};

/// Effect type of a tectonic boundary on elevation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoundaryEffect {
    /// Crustal shortening → mountain building (convergent boundaries).
    Uplift,
    /// Crustal thinning → rift valleys and depressions (divergent boundaries).
    Subsidence,
    /// Horizontal shearing with minimal vertical displacement (transform boundaries).
    Shear,
    /// Distributed deformation without clear vertical preference (conservative margins).
    Deformation,
}

impl BoundaryEffect {
    /// Maximum elevation change in meters for this effect type.
    pub fn max_elevation_change(&self) -> f32 {
        match self {
            Self::Uplift => 4000.0,
            Self::Subsidence => -2000.0,
            Self::Shear => 200.0,
            Self::Deformation => 500.0,
        }
    }
    
    /// Width of the effect zone in cells (distance from boundary).
    pub fn effect_zone_width(&self) -> u32 {
        match self {
            Self::Uplift => 8,     // Wide mountain building zone
            Self::Subsidence => 4, // Narrow rift zone
            Self::Shear => 2,      // Narrow fault zone
            Self::Deformation => 6, // Broad deformation zone
        }
    }
    
    /// Falloff function for elevation change based on distance from boundary.
    /// Returns a multiplier [0.0, 1.0] based on distance.
    pub fn distance_falloff(&self, distance: u32) -> f32 {
        let zone_width = self.effect_zone_width() as f32;
        let d = distance as f32;
        
        if d >= zone_width {
            0.0
        } else {
            // Cosine falloff for smoother transition
            (1.0 + (d * std::f32::consts::PI / zone_width).cos()) * 0.5
        }
    }
}

/// Elevation modifier from tectonic activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationModifier {
    /// Base elevation change in meters.
    pub base_change: f32,
    /// Effect type determining the modifier calculation.
    pub effect: BoundaryEffect,
    /// Activity intensity [0.0, 1.0].
    pub intensity: f32,
    /// Distance from boundary center in cells.
    pub distance_from_boundary: u32,
}

impl ElevationModifier {
    /// Calculate the actual elevation change in meters.
    pub fn calculate(&self) -> f32 {
        let _max_change = self.effect.max_elevation_change();
        let falloff = self.effect.distance_falloff(self.distance_from_boundary);
        self.base_change * falloff * self.intensity
    }
    
    /// Create an uplift modifier.
    pub fn uplift(intensity: f32, distance: u32) -> Self {
        Self {
            base_change: 1500.0,
            effect: BoundaryEffect::Uplift,
            intensity,
            distance_from_boundary: distance,
        }
    }
    
    /// Create a subsidence modifier.
    pub fn subsidence(intensity: f32, distance: u32) -> Self {
        Self {
            base_change: -800.0,
            effect: BoundaryEffect::Subsidence,
            intensity,
            distance_from_boundary: distance,
        }
    }
}

/// Calculate elevation modifiers for a grid based on tectonic result.
pub fn calculate_grid_modifiers(
    width: u32,
    height: u32,
    _boundary_cells: &[(u32, u32)], // List of (x, y) cells on boundaries
    boundary_effects: &[(u32, u32, BoundaryEffect)], // (x, y, effect)
    activity: f32,
) -> Vec<f32> {
    let total_cells = (width * height) as usize;
    let mut modifiers = vec![0.0f32; total_cells];
    
    // Build a map of boundary cells for quick lookup
    let _boundary_map: std::collections::HashMap<(u32, u32), BoundaryEffect> = 
        boundary_effects.iter().map(|(bx, by, e)| ((*bx, *by), *e)).collect();
    
    for y in 0..height {
        for x in 0..width {
            let cell_idx = (y * width + x) as usize;
            
            // Find distance to nearest boundary
            let mut min_distance = u32::MAX;
            let mut nearest_effect = BoundaryEffect::Deformation;
            
            for (bx, by, effect) in boundary_effects {
                let dx = (x as i32 - *bx as i32).abs() as u32;
                let dy = (y as i32 - *by as i32).abs() as u32;
                let distance = dx.max(dy); // Chebyshev distance
                
                if distance < min_distance {
                    min_distance = distance;
                    nearest_effect = *effect;
                }
            }
            
            // Calculate modifier based on distance and effect type
            if min_distance < nearest_effect.effect_zone_width() {
                let modifier = ElevationModifier {
                    base_change: nearest_effect.max_elevation_change(),
                    effect: nearest_effect,
                    intensity: activity,
                    distance_from_boundary: min_distance,
                };
                modifiers[cell_idx] = modifier.calculate();
            }
        }
    }
    
    modifiers
}

/// Apply tectonic elevation modifiers to a terrain grid.
pub fn apply_tectonic_modifiers(
    terrain: &mut [f32], // Flat elevation array
    modifiers: &[f32],
    width: u32,
    height: u32,
) {
    let total = (width * height) as usize;
    for i in 0..total.min(terrain.len()).min(modifiers.len()) {
        terrain[i] += modifiers[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uplift_modifier() {
        let modifier = ElevationModifier::uplift(1.0, 0);
        assert!(modifier.calculate() > 0.0);
        assert_eq!(modifier.effect, BoundaryEffect::Uplift);
    }
    
    #[test]
    fn test_subsidence_modifier() {
        let modifier = ElevationModifier::subsidence(1.0, 0);
        assert!(modifier.calculate() < 0.0);
        assert_eq!(modifier.effect, BoundaryEffect::Subsidence);
    }
    
    #[test]
    fn test_distance_falloff() {
        // At zero distance, effect should be maximum
        let uplift = BoundaryEffect::Uplift;
        assert!(uplift.distance_falloff(0) > 0.9);
        
        // At full width, effect should be zero
        assert!(uplift.distance_falloff(uplift.effect_zone_width()) < 0.01);
        
        // Mid-distance should be intermediate
        let mid_dist = uplift.effect_zone_width() / 2;
        let falloff_mid = uplift.distance_falloff(mid_dist);
        assert!(falloff_mid > 0.3 && falloff_mid < 0.8);
    }
    
    #[test]
    fn test_grid_modifiers() {
        let boundary_effects = vec![
            (5, 5, BoundaryEffect::Uplift),
            (10, 10, BoundaryEffect::Subsidence),
        ];
        
        let modifiers = calculate_grid_modifiers(
            20, 20,
            &[],
            &boundary_effects,
            1.0,
        );
        
        // Check that cells near boundaries have non-zero modifiers
        // Cell (5, 5) should have uplift
        let idx_55 = 5 * 20 + 5;
        assert!(modifiers[idx_55 as usize] > 0.0);
        
        // Cell (10, 10) should have subsidence
        let idx_1010 = 10 * 20 + 10;
        assert!(modifiers[idx_1010 as usize] < 0.0);
    }
}