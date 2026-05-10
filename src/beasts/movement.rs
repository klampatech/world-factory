//! Beast Movement - Elemental gradient seeking behavior
//!
//! Primal beasts move toward areas of their element concentration.
//! Movement speed: 1-10 km/year based on power level.
//!
//! ## Movement Rules
//!
//! 1. Each year, beasts seek higher elemental concentration
//! 2. Movement is gradual - beasts are territorial
//! 3. Stronger beasts move faster
//! 4. Beasts avoid opposing elements

use super::{BeastElement, BeastState, PrimalBeast, PrimalBeastInstance};

/// Movement request for a primal beast.
#[derive(Debug, Clone)]
pub struct BeastMovement {
    /// Beast instance
    pub beast: PrimalBeastInstance,
    /// Target cell index (None if staying)
    pub target_cell: Option<u32>,
    /// Movement distance in km
    pub distance_km: f32,
}

/// Calculate annual movement for a primal beast.
/// Note: This is a simplified version. Full implementation would need
/// access to the world state and terrain grid.
pub fn calculate_annual_movement(
    beast: &PrimalBeastInstance,
    elemental_map: &[f32], // Cell index -> elemental concentration (0.0-1.0)
) -> BeastMovement {
    // Only active beasts move
    if beast.state != BeastState::Active {
        return BeastMovement {
            beast: beast.clone(),
            target_cell: None,
            distance_km: 0.0,
        };
    }

    let profile = super::profiles::get_beast_profile(beast.beast);

    // Movement speed: 1-10 km/year based on power
    let base_speed = 1.0 + (beast.power_level * 0.9).min(9.0);
    let speed = base_speed * profile.terrain_modifier_strength;

    // Get current position and element
    let current_pos = beast.position as usize;

    // Find best direction: seek higher elemental concentration
    let mut best_cell = beast.position;
    let mut best_score = elemental_map.get(current_pos).copied().unwrap_or(0.0);

    // Check neighbors (simplified: just adjacent cells in a small grid)
    let directions = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1), // Cardinal
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1), // Diagonal
    ];

    // Use a reasonable grid size for movement calculations
    let grid_size = 10i32; // Assume 10x10 grid for simplicity
    let current_x = (beast.position % 10) as i32;
    let current_y = (beast.position / 10) as i32;

    for (dx, dy) in directions {
        let nx = current_x + dx;
        let ny = current_y + dy;

        // Check bounds
        if nx < 0 || nx >= grid_size || ny < 0 || ny >= grid_size {
            continue;
        }

        let cell_idx = (ny * grid_size + nx) as usize;

        // Get elemental score for this cell
        let score = elemental_map.get(cell_idx).copied().unwrap_or(0.0);

        if score > best_score {
            best_score = score;
            best_cell = cell_idx as u32;
        }
    }

    // Calculate actual distance moved
    let distance_km = if best_cell != beast.position {
        let dx = (best_cell % 10) as f32 - current_x as f32;
        let dy = (best_cell / 10) as f32 - current_y as f32;
        // Rough estimate: ~10km per cell
        (dx.abs() + dy.abs()) * 10.0
    } else {
        0.0
    };

    // Cap movement at calculated speed
    let actual_distance = distance_km.min(speed);

    BeastMovement {
        beast: beast.clone(),
        target_cell: if actual_distance > 0.1 {
            Some(best_cell)
        } else {
            None
        },
        distance_km: actual_distance,
    }
}

/// Process end-of-year movement, updating the beast position.
pub fn process_movement(movement: &BeastMovement) -> Option<u32> {
    movement.target_cell
}
