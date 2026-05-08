use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Core territory claim data for a faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryClaim {
    /// All polygons currently claimed by this faction
    pub claimed_polygons: HashSet<u64>,
    /// Initial/core territory polygons (starting settlements)
    pub core_territory: HashSet<u64>,
    /// Client states / vassals under this faction
    pub client_states: Vec<u64>,
    /// Zones contested with other factions (with war start year)
    pub contested_zones: HashMap<u64, ContestedZone>,
    /// Strait controls for maritime passage dominance
    pub strait_controls: Vec<u64>,
}

/// Information about a contested zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestedZone {
    /// The polygon ID being contested
    pub polygon_id: u64,
    /// When the conflict started
    pub since_year: i32,
    /// Factions involved in the contest
    pub involved_factions: Vec<u64>,
    /// Current control strength per faction (0.0 - 1.0)
    pub control_strength: HashMap<u64, f32>,
}

/// Faction identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(pub u64);

impl FactionId {
    pub fn new(id: u64) -> Self {
        FactionId(id)
    }
}

impl std::fmt::Display for FactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Result of territory generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryGenerationResult {
    pub claims: HashMap<FactionId, TerritoryClaim>,
    pub contested_zones: Vec<ContestedZone>,
    pub ocean_exclusions: HashSet<u64>,
}

impl Default for TerritoryClaim {
    fn default() -> Self {
        TerritoryClaim {
            claimed_polygons: HashSet::new(),
            core_territory: HashSet::new(),
            client_states: Vec::new(),
            contested_zones: HashMap::new(),
            strait_controls: Vec::new(),
        }
    }
}

impl ContestedZone {
    pub fn new(polygon_id: u64, year: i32, factions: Vec<u64>) -> Self {
        let mut control_strength = HashMap::new();
        let equal_share = 1.0 / factions.len() as f32;
        for &f in &factions {
            control_strength.insert(f, equal_share);
        }
        ContestedZone {
            polygon_id,
            since_year: year,
            involved_factions: factions,
            control_strength,
        }
    }
}
