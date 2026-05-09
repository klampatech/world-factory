//! Faction System for World Factory
//!
//! Provides persistent faction entities that control territories, wage wars,
//! form alliances, and drive political history. Factions are the primary
//! actors in the world's political narrative.
//!
//! ## Phase 5 Faction Turn System
//!
//! - TurnPhase: Income, Maintenance, Action, News
//! - FactionAsset: Force, Cunning, Wealth categories
//! - CampaignState: Multi-turn operations with homeworld transition
//! - BeastBond: Primal beast integration with alignment bonuses
//! - FactionGoal: Victory conditions with XP rewards

use crate::types::{EntityId, EntityType, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Phase 5: Faction Turn System Types
// ============================================================================

/// Phases in a faction turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnPhase {
    /// Income phase - collect resources
    #[default]
    Income,
    /// Maintenance phase - pay costs, resolve conflicts
    Maintenance,
    /// Action phase - execute orders, move units
    Action,
    /// News phase - report outcomes, trigger events
    News,
}

impl TurnPhase {
    /// Get the next phase in the turn cycle.
    pub fn next(&self) -> Self {
        match self {
            TurnPhase::Income => TurnPhase::Maintenance,
            TurnPhase::Maintenance => TurnPhase::Action,
            TurnPhase::Action => TurnPhase::News,
            TurnPhase::News => TurnPhase::Income,
        }
    }

    /// Get the phase name.
    pub fn name(&self) -> &'static str {
        match self {
            TurnPhase::Income => "income",
            TurnPhase::Maintenance => "maintenance",
            TurnPhase::Action => "action",
            TurnPhase::News => "news",
        }
    }
}

/// Asset category for faction assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    /// Military force (units, fortifications)
    Force,
    /// Strategic cunning (spies, agents)
    Cunning,
    /// Economic wealth (treasury, trade)
    Wealth,
}

/// A faction asset that can be used during turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionAsset {
    /// Unique ID for this asset
    pub id: Uuid,
    /// Category of this asset
    pub category: AssetCategory,
    /// Hit points / effectiveness
    pub hp: u32,
    /// Maximum HP
    pub max_hp: u32,
    /// Current location (cell ID or settlement ID)
    pub location: Option<u32>,
    /// Whether this asset can act this turn
    pub can_act: bool,
    /// Purchase year
    pub purchased_year: i32,
    /// Upgrade level
    pub upgrade_level: u8,
}

impl FactionAsset {
    /// Create a new faction asset.
    pub fn new(category: AssetCategory, hp: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            hp,
            max_hp: hp,
            location: None,
            can_act: true,
            purchased_year: 0,
            upgrade_level: 0,
        }
    }

    /// Restore ability to act for next turn.
    pub fn refresh(&mut self) {
        self.can_act = true;
    }

    /// Take damage to this asset.
    /// Returns true if the asset was destroyed.
    pub fn damage(&mut self, amount: u32) -> bool {
        if self.hp > amount {
            self.hp -= amount;
            false
        } else {
            self.hp = 0;
            self.can_act = false;
            true
        }
    }
}

/// Campaign state for multi-turn operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    /// Type of campaign
    pub campaign_type: String,
    /// Target planet or location ID
    pub target_id: Uuid,
    /// Turns remaining until completion
    pub turns_remaining: i32,
    /// Whether homeworld transition is active
    pub homeworld_transition: bool,
}

// ============================================================================
// Section 5.4: Primal Beast Integration
// ============================================================================

/// Bond between a faction and a Primal Beast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastBond {
    /// Beast ID
    pub beast_id: Uuid,
    /// Faction ID this beast is bonded to
    pub faction_id: Uuid,
    /// How the bond was formed
    pub bond_type: BeastBondType,
    /// Year the bond was established
    pub established_year: i32,
    /// Bonus provided by this bond
    pub bonus: AlignmentBonus,
}

impl BeastBond {
    /// Create a new beast bond.
    pub fn new(beast_id: Uuid, faction_id: Uuid, bond_type: BeastBondType) -> Self {
        Self {
            beast_id,
            faction_id,
            bond_type,
            established_year: 0,
            bonus: AlignmentBonus::neutral(),
        }
    }

    /// Calculate alignment bonus value.
    pub fn bonus_value(&self) -> f32 {
        self.bonus.value()
            * match self.bond_type {
                BeastBondType::Worshiped => 1.5,
                BeastBondType::Allied => 1.2,
                BeastBondType::Tolerated => 1.0,
                BeastBondType::Opposed => -0.5,
            }
    }
}

/// Type of bond between faction and beast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeastBondType {
    /// Beast is worshipped by the faction
    Worshiped,
    /// Beast has allied with the faction
    Allied,
    /// Beast tolerates the faction's presence
    Tolerated,
    /// Beast opposes the faction
    Opposed,
}

/// Alignment bonus from beast bond.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentBonus {
    /// No bonus
    Neutral,
    /// Military strength bonus
    Force,
    /// Economic bonus
    Wealth,
    /// Espionage bonus
    Cunning,
    /// Defense bonus
    Fortification,
    /// Population growth bonus
    Fertility,
}

impl AlignmentBonus {
    /// Get neutral bonus.
    pub fn neutral() -> Self {
        AlignmentBonus::Neutral
    }

    /// Get bonus value multiplier.
    pub fn value(&self) -> f32 {
        match self {
            AlignmentBonus::Neutral => 0.0,
            AlignmentBonus::Force => 0.15,
            AlignmentBonus::Wealth => 0.12,
            AlignmentBonus::Cunning => 0.10,
            AlignmentBonus::Fortification => 0.20,
            AlignmentBonus::Fertility => 0.08,
        }
    }

    /// Get bonus name.
    pub fn name(&self) -> &'static str {
        match self {
            AlignmentBonus::Neutral => "none",
            AlignmentBonus::Force => "force",
            AlignmentBonus::Wealth => "wealth",
            AlignmentBonus::Cunning => "cunning",
            AlignmentBonus::Fortification => "fortification",
            AlignmentBonus::Fertility => "fertility",
        }
    }
}

// ============================================================================
// Section 5.5: Victory Conditions
// ============================================================================

/// Goal type for faction objectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    /// Military conquest goal
    MilitaryConquest,
    /// Commercial expansion goal
    CommercialExpansion,
    /// Cultural dominance goal
    CulturalDominance,
    /// Diplomatic supremacy goal
    DiplomaticSupremacy,
}

/// A faction goal/objective.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionGoal {
    pub id: Uuid,
    pub goal_type: GoalType,
    pub description: String,
    pub progress: f32,
    pub target_value: u32,
    pub current_value: u32,
    pub xp_reward: u32,
    pub completed: bool,
}

impl FactionGoal {
    /// Create a new faction goal.
    pub fn new(goal_type: GoalType, description: String, target_value: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            goal_type,
            description,
            progress: 0.0,
            target_value,
            current_value: 0,
            xp_reward: target_value / 10,
            completed: false,
        }
    }

    /// Update progress toward goal completion.
    pub fn update_progress(&mut self, new_value: u32) {
        self.current_value = new_value;
        self.progress = (self.current_value as f32 / self.target_value as f32).min(1.0);
        if self.current_value >= self.target_value {
            self.completed = true;
        }
    }
}

// ============================================================================
// Section 5.7: Faction Turn State (Data Model)
// ============================================================================

/// Faction turn state - tracks all turn-related data for a faction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTurnState {
    /// Current turn number
    pub turn_number: i32,
    /// Current phase within turn
    pub phase: TurnPhase,
    /// Current year
    pub year: i32,
    /// Assets owned by this faction
    pub assets: Vec<FactionAsset>,
    /// Active campaigns
    pub campaigns: Vec<CampaignState>,
    /// Current goals
    pub goals: Vec<FactionGoal>,
    /// Beast bonds (Section 5.4)
    #[serde(default)]
    pub beast_bonds: Vec<BeastBond>,
    /// Experience points
    pub xp: u32,
    /// Resources available this turn
    pub resources: u32,
    /// Resources spent this turn
    pub resources_spent: u32,
    /// Last turn processed (for validation)
    pub last_processed_turn: i32,
}

impl FactionTurnState {
    /// Create a new faction turn state.
    pub fn new(year: i32) -> Self {
        Self {
            turn_number: 1,
            phase: TurnPhase::Income,
            year,
            assets: Vec::new(),
            campaigns: Vec::new(),
            goals: Vec::new(),
            beast_bonds: Vec::new(),
            xp: 0,
            resources: 100,
            resources_spent: 0,
            last_processed_turn: 0,
        }
    }

    /// Add an asset to this faction.
    pub fn add_asset(&mut self, asset: FactionAsset) {
        self.assets.push(asset);
    }

    /// Get assets by category.
    pub fn assets_by_category(&self, category: AssetCategory) -> Vec<&FactionAsset> {
        self.assets
            .iter()
            .filter(|a| a.category == category)
            .collect()
    }

    /// Get active (can_act) assets.
    pub fn active_assets(&self) -> Vec<&FactionAsset> {
        self.assets
            .iter()
            .filter(|a| a.can_act && a.hp > 0)
            .collect()
    }

    /// Process end of turn - refresh assets and advance phase.
    pub fn end_turn(&mut self) {
        // Refresh all assets for next turn
        for asset in &mut self.assets {
            asset.refresh();
        }

        // Advance phase
        self.phase = self.phase.next();

        // If we completed all phases, advance turn number
        if matches!(self.phase, TurnPhase::Income) {
            self.turn_number += 1;
            self.year += 1;
        }

        self.resources_spent = 0;
        self.last_processed_turn = self.turn_number;
    }

    /// Advance to the next turn (alias for end_turn).
    pub fn advance_turn(&mut self) {
        self.end_turn();
    }

    /// Check if a goal is completed.
    pub fn check_goals(&self) -> Vec<Uuid> {
        self.goals
            .iter()
            .filter(|g| g.completed)
            .map(|g| g.id)
            .collect()
    }

    /// Add a beast bond to this faction's turn state.
    pub fn add_beast_bond(&mut self, bond: BeastBond) {
        if !self.beast_bonds.iter().any(|b| b.beast_id == bond.beast_id) {
            self.beast_bonds.push(bond);
        }
    }

    /// Remove beast bond.
    pub fn remove_beast_bond(&mut self, beast_id: Uuid) -> Option<BeastBond> {
        if let Some(pos) = self.beast_bonds.iter().position(|b| b.beast_id == beast_id) {
            Some(self.beast_bonds.remove(pos))
        } else {
            None
        }
    }

    /// Calculate total alignment bonus from all beast bonds.
    pub fn total_alignment_bonus(&self) -> f32 {
        self.beast_bonds.iter().map(|b| b.bonus_value()).sum()
    }

    /// Get all active beast bonds.
    pub fn active_beast_bonds(&self) -> Vec<&BeastBond> {
        self.beast_bonds
            .iter()
            .filter(|b| self.year - b.established_year < 50)
            .collect()
    }
}

// ============================================================================
// Core Faction Types
// ============================================================================

/// Errors specific to faction operations.
#[derive(Debug, thiserror::Error)]
pub enum FactionError {
    #[error("Faction {0} not found")]
    NotFound(Uuid),

    #[error("Faction {0} already exists")]
    AlreadyExists(String),

    #[error("Invalid territory assignment")]
    InvalidTerritory(String),

    #[error("Cannot form alliance: factions are at war")]
    AtWar,

    #[error("Self-alliance not allowed")]
    SelfAlliance,

    #[error("Faction {0} is inactive")]
    Inactive(Uuid),
}

/// Types of political factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionType {
    Clan,
    Tribe,
    Chiefdom,
    Kingdom,
    Empire,
    Theocracy,
    Republic,
    Confederation,
    Nomadic,
}

impl FactionType {
    /// Get snake_case name.
    pub fn name(&self) -> &'static str {
        match self {
            FactionType::Clan => "clan",
            FactionType::Tribe => "tribe",
            FactionType::Chiefdom => "chiefdom",
            FactionType::Kingdom => "kingdom",
            FactionType::Empire => "empire",
            FactionType::Theocracy => "theocracy",
            FactionType::Republic => "republic",
            FactionType::Confederation => "confederation",
            FactionType::Nomadic => "nomadic",
        }
    }

    /// Get display name with proper capitalization.
    pub fn display_name(&self) -> &'static str {
        match self {
            FactionType::Clan => "Clan",
            FactionType::Tribe => "Tribe",
            FactionType::Chiefdom => "Chiefdom",
            FactionType::Kingdom => "Kingdom",
            FactionType::Empire => "Empire",
            FactionType::Theocracy => "Theocracy",
            FactionType::Republic => "Republic",
            FactionType::Confederation => "Confederation",
            FactionType::Nomadic => "Nomadic",
        }
    }

    /// Get description of this faction type.
    pub fn description(&self) -> &'static str {
        match self {
            FactionType::Clan => "Extended family group, common in early societies",
            FactionType::Tribe => "Unified group with shared identity and culture",
            FactionType::Chiefdom => "Hierarchical leadership with chief at top",
            FactionType::Kingdom => "Hereditary monarchy controlling defined territory",
            FactionType::Empire => "Multi-ethnic state with centralized bureaucracy",
            FactionType::Theocracy => "Governed by religious authority",
            FactionType::Republic => "Elected government, merchant republics",
            FactionType::Confederation => "Union of semi-independent states",
            FactionType::Nomadic => "Mobile group without fixed territory",
        }
    }

    /// Get minimum population threshold for this faction type.
    pub fn min_population(&self) -> u64 {
        match self {
            FactionType::Clan => 50,
            FactionType::Tribe => 200,
            FactionType::Chiefdom => 1000,
            FactionType::Kingdom => 5000,
            FactionType::Empire => 20000,
            FactionType::Theocracy => 3000,
            FactionType::Republic => 5000,
            FactionType::Confederation => 10000,
            FactionType::Nomadic => 100,
        }
    }

    /// Get typical government structure description.
    pub fn government_description(&self) -> &'static str {
        match self {
            FactionType::Clan => "Elder's council",
            FactionType::Tribe => "Tribal council",
            FactionType::Chiefdom => "Chief and advisors",
            FactionType::Kingdom => "Monarch and court",
            FactionType::Empire => "Emperor and bureaucracy",
            FactionType::Theocracy => "High priest and clergy",
            FactionType::Republic => "Elected council",
            FactionType::Confederation => "Council of clan leaders",
            FactionType::Nomadic => "Elder/ Khan / Council",
        }
    }

    /// Determine faction type from population.
    pub fn from_population(population: u64) -> Self {
        if population >= 20000 {
            FactionType::Empire
        } else if population >= 10000 {
            FactionType::Kingdom
        } else if population >= 5000 {
            FactionType::Confederation
        } else if population >= 3000 {
            FactionType::Theocracy
        } else if population >= 1000 {
            FactionType::Chiefdom
        } else if population >= 200 {
            FactionType::Tribe
        } else {
            FactionType::Clan
        }
    }
}

/// Diplomatic relationship between factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionRelation {
    Unknown,
    Peace,
    Allied,
    DefensivePact,
    TradeAgreement,
    Rivals,
    War,
    Suzerainty,
}

impl FactionRelation {
    pub fn name(&self) -> &'static str {
        match self {
            FactionRelation::Unknown => "unknown",
            FactionRelation::Peace => "peace",
            FactionRelation::Allied => "allied",
            FactionRelation::DefensivePact => "defensive_pact",
            FactionRelation::TradeAgreement => "trade_agreement",
            FactionRelation::Rivals => "rivals",
            FactionRelation::War => "war",
            FactionRelation::Suzerainty => "suzerainty",
        }
    }
}

/// Information about a diplomatic relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticRelation {
    pub target_id: Uuid,
    pub relation: FactionRelation,
    pub established_year: Option<i32>,
    pub started_year: Option<i32>,
    pub is_active: bool,
    pub treaty_name: Option<String>,
    pub changed_year: Option<i32>,
}

/// A faction controlling territory in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: EntityId,
    pub world_id: Uuid,
    pub name: String,
    pub faction_type: FactionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub territory_ids: Vec<u32>,
    #[serde(default)]
    pub settlement_ids: Vec<Uuid>,
    pub population: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capital_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_faction_id: Option<Uuid>,
    #[serde(default)]
    pub child_faction_ids: Vec<Uuid>,
    #[serde(default)]
    pub relations: Vec<DiplomaticRelation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub government_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub religion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub founded_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolved_year: Option<i32>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_state: Option<FactionTurnState>,
    #[serde(default)]
    pub force: u32,
    #[serde(default)]
    pub cunning: u32,
    #[serde(default)]
    pub wealth: u32,
    #[serde(default)]
    pub hp: u32,
    #[serde(default)]
    pub max_hp: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Faction {
    /// Create a new faction.
    pub fn new(world_id: Uuid, name: String, faction_type: FactionType, founded_year: i32) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Faction),
            world_id,
            name,
            faction_type,
            description: None,
            territory_ids: Vec::new(),
            settlement_ids: Vec::new(),
            population: 0,
            capital_id: None,
            leader_id: None,
            parent_faction_id: None,
            child_faction_ids: Vec::new(),
            relations: Vec::new(),
            government_type: None,
            culture: None,
            religion: None,
            color: None,
            founded_year: Some(founded_year),
            dissolved_year: None,
            is_active: true,
            history: Vec::new(),
            turn_state: Some(FactionTurnState::new(founded_year)),
            force: 10,   // Base force, recalculated via calculate_force()
            cunning: 10, // Base cunning, recalculated via calculate_cunning()
            wealth: 10,  // Base wealth, recalculated via calculate_wealth()
            hp: 10,      // Base HP, recalculated via calculate_hp()
            max_hp: 10,  // Base max HP, recalculated via calculate_hp()
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a kingdom faction.
    pub fn new_kingdom(world_id: Uuid, name: String, capital_id: Uuid, founded_year: i32) -> Self {
        let mut faction = Self::new(world_id, name, FactionType::Kingdom, founded_year);
        faction.capital_id = Some(capital_id);
        faction.government_type = Some("Monarchy".to_string());
        faction
    }

    /// Create a clan faction.
    pub fn new_clan(world_id: Uuid, name: String, founded_year: i32) -> Self {
        Self::new(world_id, name, FactionType::Clan, founded_year)
    }

    /// Check if this faction controls a specific cell.
    pub fn controls_cell(&self, cell_id: u32) -> bool {
        self.territory_ids.contains(&cell_id)
    }

    /// Add a territory cell to this faction.
    pub fn add_territory(&mut self, cell_id: u32) {
        if !self.territory_ids.contains(&cell_id) {
            self.territory_ids.push(cell_id);
            self.updated_at = Timestamp::now();
        }
    }

    /// Remove a territory cell from this faction.
    pub fn remove_territory(&mut self, cell_id: u32) -> bool {
        if let Some(i) = self.territory_ids.iter().position(|&id| id == cell_id) {
            self.territory_ids.remove(i);
            self.updated_at = Timestamp::now();
            true
        } else {
            false
        }
    }

    /// Add a settlement to this faction.
    pub fn add_settlement(&mut self, settlement_id: Uuid) {
        if !self.settlement_ids.contains(&settlement_id) {
            self.settlement_ids.push(settlement_id);
            self.updated_at = Timestamp::now();
        }
    }

    /// Get current diplomatic relation with another faction.
    pub fn get_relation(&self, target_id: Uuid) -> FactionRelation {
        self.relations
            .iter()
            .find(|r| r.target_id == target_id)
            .map(|r| r.relation)
            .unwrap_or(FactionRelation::Unknown)
    }

    /// Set diplomatic relation with another faction.
    pub fn set_relation(&mut self, target_id: Uuid, relation: FactionRelation, year: i32) {
        if let Some(existing) = self.relations.iter_mut().find(|r| r.target_id == target_id) {
            existing.relation = relation;
            existing.changed_year = Some(year);
        } else {
            self.relations.push(DiplomaticRelation {
                target_id,
                relation,
                established_year: Some(year),
                started_year: Some(year),
                is_active: true,
                treaty_name: None,
                changed_year: Some(year),
            });
        }
        self.updated_at = Timestamp::now();
    }

    /// Check if this faction is allied with another.
    pub fn is_allied_with(&self, faction_id: Uuid) -> bool {
        matches!(
            self.get_relation(faction_id),
            FactionRelation::Allied | FactionRelation::DefensivePact
        )
    }

    /// Check if this faction is at war with another.
    pub fn is_at_war_with(&self, faction_id: Uuid) -> bool {
        matches!(self.get_relation(faction_id), FactionRelation::War)
    }

    /// Disband/dissolve this faction.
    pub fn dissolve(&mut self, year: i32) {
        self.dissolved_year = Some(year);
        self.is_active = false;
        self.updated_at = Timestamp::now();
    }

    /// Calculate power score.
    pub fn power_score(&self) -> u64 {
        let territory_score = self.territory_ids.len() as u64 * 10;
        let population_score = self.population / 100;
        let ally_score = self
            .relations
            .iter()
            .filter(|r| {
                matches!(
                    r.relation,
                    FactionRelation::Allied | FactionRelation::DefensivePact
                )
            })
            .count() as u64
            * 50;
        territory_score + population_score + ally_score
    }

    /// Calculate Force stat based on world state.
    /// Represents military strength: territory count, standing armies, martial culture.
    /// Formula: 10 + (territory_ids.len() / 10) + (settlement_ids.len() * 2)
    pub fn calculate_force(&self) -> u32 {
        10 + (self.territory_ids.len() / 10) as u32 + (self.settlement_ids.len() * 2) as u32
    }

    /// Calculate Cunning stat based on world state.
    /// Represents political/espionage power: intelligence traditions, trade volume.
    /// Formula: 10 + (territory_ids.len() / 20) + (settlement_ids.len() * 3)
    pub fn calculate_cunning(&self) -> u32 {
        10 + (self.territory_ids.len() / 20) as u32 + (self.settlement_ids.len() * 3) as u32
    }

    /// Calculate Wealth stat based on world state.
    /// Represents economic resources: resource richness, trade routes, commercial settlements.
    /// Formula: 10 + (territory_ids.len() / 15) + (settlement_ids.len() * 4)
    pub fn calculate_wealth(&self) -> u32 {
        10 + (self.territory_ids.len() / 15) as u32 + (self.settlement_ids.len() * 4) as u32
    }

    /// Calculate HP based on Force, Cunning, and Wealth stats.
    /// HP represents faction stability.
    /// Formula: 10 + (force + cunning + wealth) / 3
    pub fn calculate_hp(&self) -> u32 {
        10 + (self.force + self.cunning + self.wealth) / 3
    }

    /// Recalculate all stats and HP from current world state.
    /// Call this when territory or settlement counts change.
    pub fn recalculate_stats(&mut self) {
        self.force = self.calculate_force();
        self.cunning = self.calculate_cunning();
        self.wealth = self.calculate_wealth();
        self.max_hp = self.calculate_hp();
        // HP cannot exceed max_hp
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
        self.updated_at = Timestamp::now();
    }

    /// Apply damage to the faction (e.g., from wars, failed maintenance).
    /// Returns the actual damage dealt.
    pub fn take_damage(&mut self, damage: u32) -> u32 {
        if damage >= self.hp {
            let actual_damage = self.hp;
            self.hp = 0;
            // Faction at 0 HP becomes a client state (soft failure per SPEC.md §5.5)
            self.is_active = false;
            actual_damage
        } else {
            self.hp -= damage;
            damage
        }
    }

    /// Heal the faction (e.g., successful diplomacy, economic recovery).
    pub fn heal(&mut self, amount: u32) {
        self.hp = self.hp.saturating_add(amount).min(self.max_hp);
        self.updated_at = Timestamp::now();
    }

    /// Check if the faction is at critical stability (HP below 25%).
    pub fn is_critical(&self) -> bool {
        self.hp > 0 && self.hp < self.max_hp / 4
    }
}

/// Registry for managing all factions in a world.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionRegistry {
    factions: HashMap<Uuid, Faction>,
    name_index: HashMap<String, Uuid>,
    active_ids: Vec<Uuid>,
}

impl FactionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
            name_index: HashMap::new(),
            active_ids: Vec::new(),
        }
    }

    /// Register a new faction.
    pub fn add(&mut self, faction: Faction) -> Result<(), FactionError> {
        if self.name_index.contains_key(&faction.name) {
            return Err(FactionError::AlreadyExists(faction.name.clone()));
        }

        let id = faction.id.to_uuid();
        if self.factions.contains_key(&id) {
            return Err(FactionError::AlreadyExists(faction.name.clone()));
        }

        self.factions.insert(id, faction.clone());
        self.name_index.insert(faction.name.clone(), id);

        if faction.is_active && !self.active_ids.contains(&id) {
            self.active_ids.push(id);
        }

        Ok(())
    }

    /// Get a faction by ID.
    pub fn get(&self, id: Uuid) -> Option<&Faction> {
        self.factions.get(&id)
    }

    /// Get a mutable faction by ID.
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Faction> {
        self.factions.get_mut(&id)
    }

    /// Get all factions.
    pub fn factions(&self) -> impl Iterator<Item = &Faction> {
        self.factions.values()
    }

    /// Get all factions (mutable).
    pub fn factions_mut(&mut self) -> impl Iterator<Item = &mut Faction> {
        self.factions.values_mut()
    }

    /// Get all active factions.
    pub fn active_factions(&self) -> impl Iterator<Item = &Faction> {
        self.active_ids
            .iter()
            .filter_map(|id| self.factions.get(id))
    }

    /// Get number of factions.
    pub fn len(&self) -> usize {
        self.factions.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.factions.is_empty()
    }

    /// Load a faction registry from a TOML file.
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            // Return empty registry if file doesn't exist
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let registry: FactionRegistry = toml::from_str(&content)?;
        Ok(registry)
    }

    /// Save the faction registry to a TOML file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod faction_stats_tests {
    use super::*;

    /// Test the stat calculation formulas per SPEC.md §5.0
    mod stat_calculations {
        use super::*;

        fn create_faction_with_territories(territory_count: usize, settlement_count: usize) -> Faction {
            let mut faction = Faction::new(
                Uuid::new_v4(),
                "Test Faction".to_string(),
                FactionType::Kingdom,
                1000,
            );
            // Add territories
            for i in 0..territory_count {
                faction.territory_ids.push(i as u32);
            }
            // Add settlements
            for _ in 0..settlement_count {
                faction.settlement_ids.push(Uuid::new_v4());
            }
            faction
        }

        #[test]
        fn test_force_calculation() {
            // Force: 10 + (territory_ids.len() / 10) + (settlement_ids.len() * 2)
            let faction = create_faction_with_territories(0, 0);
            assert_eq!(faction.calculate_force(), 10); // Base only

            let faction = create_faction_with_territories(10, 0);
            assert_eq!(faction.calculate_force(), 11); // 10 + 1 + 0

            let faction = create_faction_with_territories(0, 5);
            assert_eq!(faction.calculate_force(), 20); // 10 + 0 + 10

            let faction = create_faction_with_territories(100, 5);
            assert_eq!(faction.calculate_force(), 30); // 10 + 10 + 10
        }

        #[test]
        fn test_cunning_calculation() {
            // Cunning: 10 + (territory_ids.len() / 20) + (settlement_ids.len() * 3)
            let faction = create_faction_with_territories(0, 0);
            assert_eq!(faction.calculate_cunning(), 10); // Base only

            let faction = create_faction_with_territories(20, 0);
            assert_eq!(faction.calculate_cunning(), 11); // 10 + 1 + 0

            let faction = create_faction_with_territories(0, 5);
            assert_eq!(faction.calculate_cunning(), 25); // 10 + 0 + 15

            let faction = create_faction_with_territories(100, 5);
            assert_eq!(faction.calculate_cunning(), 30); // 10 + 5 + 15
        }

        #[test]
        fn test_wealth_calculation() {
            // Wealth: 10 + (territory_ids.len() / 15) + (settlement_ids.len() * 4)
            let faction = create_faction_with_territories(0, 0);
            assert_eq!(faction.calculate_wealth(), 10); // Base only

            let faction = create_faction_with_territories(15, 0);
            assert_eq!(faction.calculate_wealth(), 11); // 10 + 1 + 0

            let faction = create_faction_with_territories(0, 5);
            assert_eq!(faction.calculate_wealth(), 30); // 10 + 0 + 20

            let faction = create_faction_with_territories(100, 5);
            // 100 / 15 = 6 (integer division), 5 * 4 = 20
            // 10 + 6 + 20 = 36
            assert_eq!(faction.calculate_wealth(), 36);
        }

        #[test]
        fn test_hp_calculation() {
            // HP: 10 + (force + cunning + wealth) / 3
            let mut faction = Faction::new(
                Uuid::new_v4(),
                "Test".to_string(),
                FactionType::Kingdom,
                1000,
            );
            // Set base values to test HP formula
            faction.force = 30;
            faction.cunning = 30;
            faction.wealth = 30;
            assert_eq!(faction.calculate_hp(), 40); // 10 + 90/3 = 40

            faction.force = 10;
            faction.cunning = 10;
            faction.wealth = 10;
            assert_eq!(faction.calculate_hp(), 20); // 10 + 30/3 = 20
        }
    }

    /// Test HP mechanics
    mod hp_mechanics {
        use super::*;


        fn create_test_faction() -> Faction {
            let mut faction = Faction::new(
                Uuid::new_v4(),
                "Test Faction".to_string(),
                FactionType::Kingdom,
                1000,
            );
            faction.force = 30;
            faction.cunning = 30;
            faction.wealth = 30;
            faction.recalculate_stats();
            faction
        }

        #[test]
        fn test_new_faction_has_base_stats() {
            let faction = Faction::new(
                Uuid::new_v4(),
                "New Faction".to_string(),
                FactionType::Kingdom,
                1000,
            );
            assert_eq!(faction.force, 10);
            assert_eq!(faction.cunning, 10);
            assert_eq!(faction.wealth, 10);
            assert_eq!(faction.hp, 10);
            assert_eq!(faction.max_hp, 10);
        }

        #[test]
        fn test_recalculate_stats() {
            let mut faction = Faction::new(
                Uuid::new_v4(),
                "Test".to_string(),
                FactionType::Kingdom,
                1000,
            );
            // Add territories and settlements
            for i in 0..50 {
                faction.territory_ids.push(i);
            }
            for _ in 0..10 {
                faction.settlement_ids.push(Uuid::new_v4());
            }
            faction.recalculate_stats();

            // Force: 10 + 50/10 + 10*2 = 10 + 5 + 20 = 35
            assert_eq!(faction.calculate_force(), 35);
            // Cunning: 10 + 50/20 + 10*3 = 10 + 2 + 30 = 42
            assert_eq!(faction.calculate_cunning(), 42);
            // Wealth: 10 + 50/15 + 10*4 = 10 + 3 + 40 = 53
            assert_eq!(faction.calculate_wealth(), 53);
            // HP: 10 + (force + cunning + wealth) / 3 = 10 + (35 + 42 + 53) / 3 = 10 + 43 = 53
            assert_eq!(faction.max_hp, 53);
        }

        #[test]
        fn test_take_damage() {
            let mut faction = create_test_faction();
            let initial_hp = faction.hp;

            // Take 5 damage
            let damage = faction.take_damage(5);
            assert_eq!(damage, 5);
            assert_eq!(faction.hp, initial_hp - 5);
            assert!(faction.is_active); // Still active
        }

        #[test]
        fn test_take_fatal_damage() {
            let mut faction = create_test_faction();
            let hp = faction.hp;

            // Take damage equal to current HP
            let damage = faction.take_damage(hp);
            assert_eq!(damage, hp);
            assert_eq!(faction.hp, 0);
            assert!(!faction.is_active); // Becomes client state (inactive)
        }

        #[test]
        fn test_take_overkill_damage() {
            let mut faction = create_test_faction();
            let hp = faction.hp;


            // Take more damage than current HP
            let damage = faction.take_damage(hp + 100);
            assert_eq!(damage, hp); // Only actual HP was dealt
            assert_eq!(faction.hp, 0);
            assert!(!faction.is_active);
        }

        #[test]
        fn test_heal() {
            let mut faction = create_test_faction();
            faction.take_damage(10); // Reduce HP by 10
            let damaged_hp = faction.hp;

            faction.heal(5);
            assert_eq!(faction.hp, damaged_hp + 5);

            // Heal cannot exceed max_hp
            faction.heal(1000);
            assert_eq!(faction.hp, faction.max_hp);
        }

        #[test]
        fn test_is_critical() {
            let mut faction = Faction::new(
                Uuid::new_v4(),
                "Critical Test".to_string(),
                FactionType::Kingdom,
                1000,
            );
            
            // Add territories so recalculate_stats gives higher stats
            // With 50 territories and 10 settlements, stats should be:
            // force = 10 + 50/10 + 10*2 = 10 + 5 + 20 = 35
            // cunning = 10 + 50/20 + 10*3 = 10 + 2 + 30 = 42
            // wealth = 10 + 50/15 + 10*4 = 10 + 3 + 40 = 53
            // max_hp = 10 + (35+42+53)/3 = 10 + 43 = 53
            for i in 0..50 {
                faction.territory_ids.push(i);
            }
            for _ in 0..10 {
                faction.settlement_ids.push(Uuid::new_v4());
            }
            faction.recalculate_stats();
            
            // HP doesn't get reset to max_hp by recalculate_stats
            // We need to manually set it for the test
            faction.hp = faction.max_hp;
            
            assert_eq!(faction.max_hp, 53);
            assert_eq!(faction.hp, 53);
            assert!(!faction.is_critical()); // Full HP

            // Take enough damage to reduce HP below 25%
            // With integer division, max_hp / 4 = 53 / 4 = 13
            // is_critical is true when hp < max_hp / 4, so hp must be <= 12
            // Take 41 damage, leaving HP at 12 (53 - 41 = 12, 12 < 13 = true)
            let damage = 41;
            if damage <= faction.hp {
                faction.take_damage(damage);
            }
            
            // HP should now be 12, which is below 25% threshold (13)
            if faction.hp > 0 {
                assert!(faction.is_critical(), 
                    "HP {} should be below 25% of max_hp {} (25% = {})", 
                    faction.hp, faction.max_hp, faction.max_hp / 4);
            }

            // HP at 0 is not critical (it's dead/inactive)
            if faction.hp > 0 {
                faction.take_damage(faction.hp);
            }
            assert!(!faction.is_critical());
        }
    }
}
