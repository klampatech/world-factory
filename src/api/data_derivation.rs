//! Data derivation helpers for API responses
//!
//! This module provides helpers for the following data derivations identified
//! in WOR-72:
//!
//! | ID | Location | Description | Helper Function |
//! |----|----------|-------------|-----------------|
//! | M1 | map_api.rs:708 | significance from population | `derive_significance()` |
//! | M2 | worlds.rs:1746 | latitude from location | `derive_latitude()` |
//! | M3 | worlds.rs:1747 | longitude from location | `derive_longitude()` |
//! | M6 | worlds.rs:1837 | planet name from metadata | `derive_planet_name()` |
//! | M8 | worlds.rs:2284 | Wonder stats | `derive_wonder_stats()` |
//! | M10 | worlds.rs:1381 | NotableFigures settlement linkage | `link_figures_to_settlements()` |
//! | M11 | river_service.rs:40 | Drainage basin wiring | `derive_basin_id()` |
//!
//! | M8 | Filter by params | `apply_wonder_filters()` |

use crate::api::models::{WonderStats, WonderView};
use std::collections::HashMap;

// =============================================================================
// Wonder Stats Derivation (M8)
// =============================================================================

/// Compute WonderStats from a list of wonders.
///
/// # Arguments
/// * `wonders` - List of wonder views
///
/// # Returns
/// * WonderStats with totals and category breakdown
pub fn derive_wonder_stats(wonders: &[WonderView]) -> WonderStats {
    let total = wonders.len();
    let mut by_category: HashMap<String, usize> = HashMap::new();
    let mut total_influence: f64 = 0.0;
    
    for wonder in wonders {
        *by_category.entry(wonder.category.clone()).or_insert(0) += 1;
        total_influence += wonder.influence_radius as f64;
    }
    
    let avg_influence = if total > 0 {
        total_influence / total as f64
    } else {
        50.0
    };
    
    WonderStats {
        total_wonders: total,
        by_category,
        avg_influence_radius: avg_influence as f32,
    }
}

/// Apply filter parameters to wonder list.
///
/// This implements the filtering logic that was previously marked as TODO
/// in the worlds endpoint.
///
/// # Arguments
/// * `wonders` - List of wonders to filter
/// * `category` - Optional category filter
/// * `wonder_type` - Optional type filter
///
/// # Returns
/// * Filtered wonder list
pub fn apply_wonder_filters(
    wonders: &[WonderView],
    category: Option<&str>,
    wonder_type: Option<&str>,
) -> Vec<WonderView> {
    wonders.iter()
        .filter(|w| {
            // Category filter
            if let Some(cat) = category {
                if !w.category.to_lowercase().contains(&cat.to_lowercase()) {
                    return false;
                }
            }
            
            // Type filter
            if let Some(wtype) = wonder_type {
                if !w.wonder_type.to_lowercase().contains(&wtype.to_lowercase()) {
                    return false;
                }
            }
            
            true
        })
        .cloned()
        .collect()
}