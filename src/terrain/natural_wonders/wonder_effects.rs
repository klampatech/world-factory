//! Wonder Effects Module for World Factory
//! 
//! Handles application of wonder bonuses to regions, settlements, and entities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of bonuses that wonders can provide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WonderBonusType {
    /// Food production bonus (multiplier)
    FoodBonus,
    /// Production/industry bonus (multiplier)
    ProductionBonus,
    /// Gold/economy bonus (multiplier)
    GoldBonus,
    /// Science/research bonus (multiplier)
    ScienceBonus,
    /// Culture/art bonus (multiplier)
    CultureBonus,
    /// Faith/religious bonus (multiplier)
    FaithBonus,
    /// Trade/commerce bonus (multiplier)
    TradeBonus,
    /// Population growth bonus (multiplier)
    PopulationGrowth,
    /// Defense/fortification bonus (multiplier)
    DefenseBonus,
    /// Energy/power bonus (multiplier)
    EnergyBonus,
    /// Specific resource bonus (by name)
    ResourceBonus(String),
    /// Tourism bonus (multiplier)
    TourismBonus,
    /// Happiness/lifestyle bonus (multiplier)
    HappinessBonus,
}

/// Bonus provided by a natural wonder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WonderBonus {
    /// Type of bonus
    pub bonus_type: WonderBonusType,
    /// Magnitude of the bonus (multiplier, typically 1.0-2.0)
    pub magnitude: f32,
    /// Radius of influence in cells
    pub radius: f32,
    /// Whether this applies to entire region
    pub region_wide: bool,
}

/// Source of a bonus (which wonder provides it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WonderBonusSource {
    /// Wonder ID that provides this bonus
    pub wonder_id: u32,
    /// Wonder type
    pub wonder_type: super::WonderType,
}

/// Statistics about bonuses in a region.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WonderBonusStats {
    /// Total bonuses affecting this cell/region
    pub bonuses: Vec<WonderBonusSource>,
    /// Stacked multipliers per bonus type
    pub multipliers: HashMap<String, f32>,
    /// Whether this has any wonder bonuses at all
    pub has_wonder_bonus: bool,
}

impl WonderBonusStats {
    /// Get the combined multiplier for a specific bonus type.
    pub fn get_multiplier(&self, bonus_type: &WonderBonusType) -> f32 {
        let key = match bonus_type {
            WonderBonusType::FoodBonus => "food",
            WonderBonusType::ProductionBonus => "production",
            WonderBonusType::GoldBonus => "gold",
            WonderBonusType::ScienceBonus => "science",
            WonderBonusType::CultureBonus => "culture",
            WonderBonusType::FaithBonus => "faith",
            WonderBonusType::TradeBonus => "trade",
            WonderBonusType::PopulationGrowth => "population_growth",
            WonderBonusType::DefenseBonus => "defense",
            WonderBonusType::EnergyBonus => "energy",
            WonderBonusType::ResourceBonus(name) => name,
            WonderBonusType::TourismBonus => "tourism",
            WonderBonusType::HappinessBonus => "happiness",
        };
        
        *self.multipliers.get(key).unwrap_or(&1.0)
    }
    
    /// Apply a bonus to these stats.
    pub fn apply(&mut self, bonus: &WonderBonus, source: WonderBonusSource) {
        self.bonuses.push(source);
        self.has_wonder_bonus = true;
        
        let key = match &bonus.bonus_type {
            WonderBonusType::FoodBonus => "food",
            WonderBonusType::ProductionBonus => "production",
            WonderBonusType::GoldBonus => "gold",
            WonderBonusType::ScienceBonus => "science",
            WonderBonusType::CultureBonus => "culture",
            WonderBonusType::FaithBonus => "faith",
            WonderBonusType::TradeBonus => "trade",
            WonderBonusType::PopulationGrowth => "population_growth",
            WonderBonusType::DefenseBonus => "defense",
            WonderBonusType::EnergyBonus => "energy",
            WonderBonusType::ResourceBonus(name) => return, // Handle separately
            WonderBonusType::TourismBonus => "tourism",
            WonderBonusType::HappinessBonus => "happiness",
        };
        
        let current = self.multipliers.entry(key.to_string()).or_insert(1.0);
        *current = (*current).max(bonus.magnitude);
    }
}

/// Compute wonder bonuses for a position from all nearby wonders.
pub fn compute_wonder_bonuses(
    x: f32,
    y: f32,
    wonders: &[super::NaturalWonder],
) -> WonderBonusStats {
    let mut stats = WonderBonusStats::default();
    
    for wonder in wonders {
        let dx = x - wonder.x;
        let dy = y - wonder.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance <= wonder.influence_radius {
            let source = WonderBonusSource {
                wonder_id: wonder.id,
                wonder_type: wonder.wonder_type,
            };
            
            for bonus in &wonder.bonuses {
                if bonus.radius >= distance {
                    stats.apply(bonus, source);
                }
            }
        }
    }
    
    stats
}

/// Apply wonder effects to a region's base production values.
pub fn apply_wonder_effects(
    base_values: &mut RegionProductionValues,
    stats: &WonderBonusStats,
) {
    base_values.food *= stats.get_multiplier(&WonderBonusType::FoodBonus);
    base_values.production *= stats.get_multiplier(&WonderBonusType::ProductionBonus);
    base_values.gold *= stats.get_multiplier(&WonderBonusType::GoldBonus);
    base_values.science *= stats.get_multiplier(&WonderBonusType::ScienceBonus);
    base_values.culture *= stats.get_multiplier(&WonderBonusType::CultureBonus);
    base_values.faith *= stats.get_multiplier(&WonderBonusType::FaithBonus);
}

/// Base production values for a region.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegionProductionValues {
    pub food: f32,
    pub production: f32,
    pub gold: f32,
    pub science: f32,
    pub culture: f32,
    pub faith: f32,
}