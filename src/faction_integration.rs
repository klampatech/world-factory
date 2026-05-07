//! Faction Integration with History System
//! 
//! Provides integration between factions, societies, settlements, and world generation.
//! This module bridges the faction system with the existing history/simulation modules.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::faction::{
    Faction, FactionRegistry, FactionType, FactionRelation, 
    FactionTurnState, TurnPhase, FactionGoal, GoalType
};

// ============================================================================
// Section 5.6: AI Faction Behavior
// ============================================================================

/// Faction turn processor for Phase 5.
/// Processes turn phases, advances diplomatic relations, updates goals.
#[derive(Debug, Clone)]
pub struct FactionTurnProcessor {
    /// Base income per turn (resources gained)
    pub base_income: u32,
    /// Maintenance cost per asset per turn
    pub maintenance_cost_per_asset: u32,
    /// Probability of random event per turn
    pub random_event_probability: f32,
}

impl Default for FactionTurnProcessor {
    fn default() -> Self {
        Self {
            base_income: 50,
            maintenance_cost_per_asset: 5,
            random_event_probability: 0.1,
        }
    }
}

impl FactionTurnProcessor {
    /// Create a new turn processor.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process a single turn for all factions.
    /// Returns the new year after turn processing.
    pub fn process_turn(
        &self,
        registry: &mut FactionRegistry,
        current_year: i32,
        diplomatic_processor: &DiplomaticProcessor,
    ) -> i32 {
        let mut new_year = current_year;
        
        // Process each faction
        for faction in registry.factions_mut() {
            if !faction.is_active {
                continue;
            }
            
            // Initialize turn state if needed
            if faction.turn_state.is_none() {
                faction.turn_state = Some(FactionTurnState::new(current_year));
            }
            
            let turn_state = faction.turn_state.as_mut().unwrap();
            
            // Process based on current phase
            let phase = turn_state.phase;
            
            match phase {
                TurnPhase::Income => {
                    // Calculate income from turn state data (avoids borrow conflict)
                    let income = self.base_income + turn_state.xp as u32 + turn_state.assets.len() as u32;
                    turn_state.resources += income;
                }
                TurnPhase::Maintenance => {
                    let costs = self.calculate_maintenance_costs(turn_state);
                    if turn_state.resources >= costs {
                        turn_state.resources -= costs;
                    } else {
                        for asset in &mut turn_state.assets {
                            if simple_rand() < 0.3 {
                                asset.damage(1);
                            }
                        }
                    }
                }
                TurnPhase::Action => {
                    turn_state.assets.retain(|a| a.hp > 0);
                }
                TurnPhase::News => {
                    if !turn_state.assets.is_empty() {
                        turn_state.xp += 1;
                    }
                }
            }
            
            // Advance phase
            turn_state.end_turn();
            
            if matches!(turn_state.phase, TurnPhase::Income) {
                new_year = turn_state.year;
            }
        }
        
        // Process diplomatic events for the year
        let _events = diplomatic_processor.process_year(registry, new_year);
        
        new_year
    }
    
    /// Calculate income based on faction resources.
    fn calculate_income(&self, faction: &Faction) -> u32 {
        let base = self.base_income;
        let population_bonus = (faction.population / 1000) as u32;
        let territory_bonus = faction.territory_ids.len() as u32;
        base + population_bonus + territory_bonus
    }
    
    /// Calculate maintenance costs.
    fn calculate_maintenance_costs(&self, turn_state: &FactionTurnState) -> u32 {
        turn_state.assets.len() as u32 * self.maintenance_cost_per_asset
    }
    
    /// Update progress on faction goals.
    fn update_goal_progress(&self, faction: &mut Faction) {
        if let Some(ref mut turn_state) = faction.turn_state {
            for goal in &mut turn_state.goals {
                if goal.completed {
                    continue;
                }
                
                match goal.goal_type {
                    GoalType::MilitaryConquest => {
                        let territory_progress = faction.territory_ids.len() as u32;
                        goal.update_progress(territory_progress);
                    }
                    GoalType::CommercialExpansion => {
                        let pop_progress = (faction.population / 100) as u32;
                        goal.update_progress(pop_progress);
                    }
                    GoalType::CulturalDominance => {
                        let settlement_progress = faction.settlement_ids.len() as u32;
                        goal.update_progress(settlement_progress);
                    }
                    GoalType::DiplomaticSupremacy => {
                        let alliance_count = faction.relations.iter()
                            .filter(|r| r.relation == FactionRelation::Allied)
                            .count() as u32;
                        goal.update_progress(alliance_count);
                    }
                }
                
                if goal.completed && goal.xp_reward > 0 {
                    turn_state.xp += goal.xp_reward;
                }
            }
        }
    }
    
    /// Add a goal to a faction.
    pub fn add_goal(
        &self,
        faction: &mut Faction,
        goal_type: GoalType,
        description: String,
        target_value: u32,
    ) {
        if let Some(ref mut turn_state) = faction.turn_state {
            let goal = FactionGoal::new(goal_type, description, target_value);
            turn_state.goals.push(goal);
        }
    }
    
    /// Get AI decision for a faction (Section 5.6).
    /// Returns suggested action based on goals and resources.
    pub fn ai_decide_action(
        &self,
        faction: &Faction,
        difficulty: AIDifficulty,
    ) -> Option<AIAction> {
        let turn_state = faction.turn_state.as_ref()?;
        
        // Check active goals
        if let Some(goal) = turn_state.goals.iter().find(|g| !g.completed) {
            let action = match goal.goal_type {
                GoalType::MilitaryConquest => {
                    if turn_state.resources >= 30 && difficulty.aggression_modifier() > 0.5 {
                        Some(AIAction::PurchaseAsset {
                            category: "force".to_string(),
                            budget: 30,
                        })
                    } else {
                        None
                    }
                }
                GoalType::CommercialExpansion => {
                    if turn_state.resources >= 40 {
                        Some(AIAction::PurchaseAsset {
                            category: "wealth".to_string(),
                            budget: 40,
                        })
                    } else {
                        None
                    }
                }
                GoalType::CulturalDominance => {
                    if turn_state.resources >= 25 {
                        Some(AIAction::ExpandTerritory {
                            priority: 25,
                        })
                    } else {
                        None
                    }
                }
                GoalType::DiplomaticSupremacy => {
                    if turn_state.resources >= 35 {
                        Some(AIAction::PurchaseAsset {
                            category: "cunning".to_string(),
                            budget: 35,
                        })
                    } else {
                        None
                    }
                }
            };
            return action;
        }
        
        // Default behavior
        if turn_state.resources < 50 {
            None
        } else if difficulty.aggression_modifier() > 0.5 {
            Some(AIAction::PurchaseAsset {
                category: "force".to_string(),
                budget: 30,
            })
        } else {
            Some(AIAction::BuildEconomy {
                budget: 40,
            })
        }
    }
}

/// AI difficulty level for faction behavior.
#[derive(Debug, Clone, Copy)]
pub enum AIDifficulty {
    Easy,
    Medium,
    Hard,
    Legendary,
}

impl AIDifficulty {
    /// Returns aggression modifier (0.0 - 1.0).
    pub fn aggression_modifier(&self) -> f32 {
        match self {
            AIDifficulty::Easy => 0.3,
            AIDifficulty::Medium => 0.5,
            AIDifficulty::Hard => 0.7,
            AIDifficulty::Legendary => 0.9,
        }
    }
    
    /// Returns resource bonus multiplier.
    pub fn resource_modifier(&self) -> f32 {
        match self {
            AIDifficulty::Easy => 1.2,
            AIDifficulty::Medium => 1.0,
            AIDifficulty::Hard => 0.9,
            AIDifficulty::Legendary => 0.8,
        }
    }
}

/// AI action decision.
#[derive(Debug, Clone)]
pub enum AIAction {
    /// Purchase a faction asset
    PurchaseAsset {
        category: String,
        budget: u32,
    },
    /// Attempt to expand territory
    ExpandTerritory {
        priority: u32,
    },
    /// Invest in economy
    BuildEconomy {
        budget: u32,
    },
    /// Form diplomatic relations
    DiplomaticAction {
        target_faction_id: Uuid,
        action_type: String,
    },
}

// ============================================================================
// Diplomatic System
// ============================================================================

/// Diplomatic event between factions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticEvent {
    pub id: Uuid,
    pub year: i32,
    pub faction1_id: Uuid,
    pub faction2_id: Uuid,
    pub relation: FactionRelation,
    pub treaty_name: Option<String>,
    pub description: Option<String>,
}

/// Processes diplomatic relations over time.
pub struct DiplomaticProcessor {
    /// Base probability of alliance per year
    pub alliance_probability: f32,
    /// Base probability of war per year
    pub war_probability: f32,
    /// Years before peace treaty can be signed
    pub min_peace_years: i32,
}

impl Default for DiplomaticProcessor {
    fn default() -> Self {
        Self {
            alliance_probability: 0.02,
            war_probability: 0.05,
            min_peace_years: 10,
        }
    }
}

impl DiplomaticProcessor {
    /// Create a new diplomatic processor.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process diplomatic relations for a year.
    /// Returns a list of diplomatic events that occurred.
    pub fn process_year(
        &self,
        registry: &mut FactionRegistry,
        year: i32,
    ) -> Vec<DiplomaticEvent> {
        let mut events = Vec::new();
        
        let faction_ids: Vec<_> = registry.active_factions()
            .map(|f| f.id.to_uuid())
            .collect();
        
        for i in 0..faction_ids.len() {
            for j in (i + 1)..faction_ids.len() {
                let id1 = faction_ids[i];
                let id2 = faction_ids[j];
                
                if let Some(f1) = registry.get(id1) {
                    if f1.is_at_war_with(id2) {
                        continue;
                    }
                }
                
                self.try_form_alliance(registry, id1, id2, year, &mut events);
                self.try_declare_war(registry, id1, id2, year, &mut events);
            }
        }
        
        events
    }
    
    fn try_form_alliance(
        &self,
        registry: &mut FactionRegistry,
        id1: Uuid,
        id2: Uuid,
        year: i32,
        events: &mut Vec<DiplomaticEvent>,
    ) {
        let can_alliance = if let (Some(f1), Some(f2)) = (registry.get(id1), registry.get(id2)) {
            f1.population > 1000 && f2.population > 1000 &&
            !f1.is_at_war_with(id2)
        } else {
            false
        };
        
        if can_alliance && simple_rand() < self.alliance_probability {
            if registry.create_alliance(id1, id2, year).is_ok() {
                events.push(DiplomaticEvent {
                    id: Uuid::new_v4(),
                    year,
                    faction1_id: id1,
                    faction2_id: id2,
                    relation: FactionRelation::Allied,
                    treaty_name: Some(format!("Treaty of Year {}", year)),
                    description: Some(format!("Alliance formed in year {}", year)),
                });
            }
        }
    }
    
    fn try_declare_war(
        &self,
        registry: &mut FactionRegistry,
        id1: Uuid,
        id2: Uuid,
        year: i32,
        events: &mut Vec<DiplomaticEvent>,
    ) {
        let can_war = if let (Some(f1), Some(f2)) = (registry.get(id1), registry.get(id2)) {
            let power_ratio = f1.power_score() as f32 / f2.power_score().max(1) as f32;
            power_ratio > 1.5 && simple_rand() < self.war_probability
        } else {
            false
        };
        
        if can_war {
            if registry.declare_war(id1, id2, year).is_ok() {
                events.push(DiplomaticEvent {
                    id: Uuid::new_v4(),
                    year,
                    faction1_id: id1,
                    faction2_id: id2,
                    relation: FactionRelation::War,
                    treaty_name: None,
                    description: Some(format!("War declared in year {}", year)),
                });
            }
        }
    }
}

/// Extension methods for FactionRegistry
impl FactionRegistry {
    /// Create an alliance between two factions.
    pub fn create_alliance(&mut self, faction1_id: Uuid, faction2_id: Uuid, year: i32) -> Result<(), crate::faction::FactionError> {
        if faction1_id == faction2_id {
            return Err(crate::faction::FactionError::SelfAlliance);
        }
        
        if let Some(f1) = self.get(&faction1_id) {
            if f1.is_at_war_with(faction2_id) {
                return Err(crate::faction::FactionError::AtWar);
            }
        }
        
        if let Some(f1) = self.get_mut(&faction1_id) {
            f1.set_relation(faction2_id, FactionRelation::Allied, year);
        }
        if let Some(f2) = self.get_mut(&faction2_id) {
            f2.set_relation(faction1_id, FactionRelation::Allied, year);
        }
        
        Ok(())
    }
    
    /// Declare war between two factions.
    pub fn declare_war(&mut self, faction1_id: Uuid, faction2_id: Uuid, year: i32) -> Result<(), crate::faction::FactionError> {
        if faction1_id == faction2_id {
            return Err(crate::faction::FactionError::SelfAlliance);
        }
        
        if let Some(f1) = self.get_mut(&faction1_id) {
            f1.set_relation(faction2_id, FactionRelation::War, year);
        }
        if let Some(f2) = self.get_mut(&faction2_id) {
            f2.set_relation(faction1_id, FactionRelation::War, year);
        }
        
        Ok(())
    }
    
    /// Sign a peace treaty between factions.
    pub fn sign_peace(&mut self, faction1_id: Uuid, faction2_id: Uuid, treaty_name: &str, year: i32) -> Result<(), crate::faction::FactionError> {
        if faction1_id == faction2_id {
            return Err(crate::faction::FactionError::SelfAlliance);
        }
        
        if let Some(f1) = self.get_mut(&faction1_id) {
            f1.set_relation(faction2_id, FactionRelation::Peace, year);
            if let Some(rel) = f1.relations.iter_mut().find(|r| r.target_id == faction2_id) {
                rel.treaty_name = Some(treaty_name.to_string());
            }
        }
        if let Some(f2) = self.get_mut(&faction2_id) {
            f2.set_relation(faction1_id, FactionRelation::Peace, year);
            if let Some(rel) = f2.relations.iter_mut().find(|r| r.target_id == faction1_id) {
                rel.treaty_name = Some(treaty_name.to_string());
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// Faction Generation
// ============================================================================

/// Faction generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionGeneratorConfig {
    /// Target number of major factions
    pub target_faction_count: usize,
    /// Minimum territory cells per faction
    pub min_territory_cells: usize,
    /// Allow nomadic factions
    pub allow_nomadic: bool,
    /// Allow confederations
    pub allow_confederations: bool,
}

impl Default for FactionGeneratorConfig {
    fn default() -> Self {
        Self {
            target_faction_count: 8,
            min_territory_cells: 50,
            allow_nomadic: false,
            allow_confederations: true,
        }
    }
}

/// Generates factions from societies and settlements.
pub struct FactionGenerator {
    config: FactionGeneratorConfig,
}

impl FactionGenerator {
    /// Create a new faction generator with default config.
    pub fn new() -> Self {
        Self {
            config: FactionGeneratorConfig::default(),
        }
    }
    
    /// Create with custom config.
    pub fn with_config(config: FactionGeneratorConfig) -> Self {
        Self { config }
    }
    
    /// Create initial faction for early history.
    pub fn create_founding_faction(
        &self,
        world_id: Uuid,
        name: &str,
        capital_id: Uuid,
        population: u64,
        year: i32,
    ) -> Faction {
        let faction_type = FactionType::from_population(population);
        
        let mut faction = Faction::new(world_id, name.to_string(), faction_type, year);
        faction.capital_id = Some(capital_id);
        faction.population = population;
        faction.settlement_ids = vec![capital_id];
        faction.government_type = Some(faction_type.government_description().to_string());
        
        faction
    }
    
    /// Check for faction type evolution based on population.
    pub fn check_evolution(faction: &mut Faction) -> Option<FactionType> {
        let new_type = FactionType::from_population(faction.population);
        
        if new_type != faction.faction_type {
            faction.faction_type = new_type;
            faction.government_type = Some(new_type.government_description().to_string());
            return Some(new_type);
        }
        
        None
    }
}

impl Default for FactionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Simple random function for probability checks
fn simple_rand() -> f32 {
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
    fn test_faction_processor_creation() {
        let processor = FactionTurnProcessor::new();
        assert_eq!(processor.base_income, 50);
        assert_eq!(processor.maintenance_cost_per_asset, 5);
    }

    #[test]
    fn test_ai_difficulty() {
        assert_eq!(AIDifficulty::Easy.aggression_modifier(), 0.3);
        assert_eq!(AIDifficulty::Legendary.aggression_modifier(), 0.9);
    }

    #[test]
    fn test_faction_generator() {
        let gen = FactionGenerator::new();
        let config = gen.config;
        assert_eq!(config.target_faction_count, 8);
        assert!(config.allow_confederations);
    }

    #[test]
    fn test_diplomatic_processor() {
        let processor = DiplomaticProcessor::new();
        assert_eq!(processor.alliance_probability, 0.02);
        assert_eq!(processor.war_probability, 0.05);
    }
}