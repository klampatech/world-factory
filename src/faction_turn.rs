//! Faction Turn System - Core Turn Manager
//!
//! Implements the 4-phase turn cycle per WOR-719:
//! - Phase 1: INCOME - Collect FacCreds from assets + beast alignment bonuses
//! - Phase 2: MAINTENANCE - Pay upkeep, cascade damage to assets that can't pay
//! - Phase 3: ACTION - Execute orders (Attack/Move/Purchase/Diplomacy/Expand)
//! - Phase 4: NEWS - Report outcomes, trigger events
//!
//! ## Maintenance Cascade (per SPEC.md §5.1)
//! - Consecutive failures → escalating asset damage → destruction → Remnant
//! - 3 consecutive turns failure → faction disbands
//!
//! ## Action Resolution (per SPEC.md §5.1)
//! - Attack: `attacker_score + 2d6 >= defender_score + 10`
//! - Move: Assets relocate along connected territory
//! - Expand: 1 tile per settlement on frontier
//! - Purchase: 1 asset per turn hard cap

use crate::beasts::RemnantArtifact;
use crate::faction::{
    AlignmentBonus, AssetCategory, BeastBond, BeastBondType, CampaignState, Faction, FactionAsset,
    FactionGoal, FactionRegistry, FactionTurnState, FactionType, GoalType, TurnPhase,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Turn Manager - Orchestrates the 4-Phase Turn Cycle
// ============================================================================

/// Global turn manager that orchestrates the 4-phase turn cycle across all factions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnManager {
    /// Current world turn number
    pub turn_number: u32,
    /// Current year in the simulation
    pub current_year: i32,
    /// Current phase within the turn
    pub current_phase: TurnPhase,
    /// Configuration for turn processing
    pub config: TurnManagerConfig,
    /// Turn events logged this turn
    #[serde(default)]
    pub events: Vec<TurnEvent>,
    /// Faction-specific turn managers
    #[serde(default)]
    pub faction_managers: HashMap<Uuid, FactionTurnManager>,
}

impl Default for TurnManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TurnManager {
    /// Create a new turn manager.
    pub fn new() -> Self {
        Self {
            turn_number: 1,
            current_year: 0,
            current_phase: TurnPhase::Income,
            config: TurnManagerConfig::default(),
            events: Vec::new(),
            faction_managers: HashMap::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: TurnManagerConfig) -> Self {
        Self {
            turn_number: 1,
            current_year: 0,
            current_phase: TurnPhase::Income,
            config,
            events: Vec::new(),
            faction_managers: HashMap::new(),
        }
    }

    /// Initialize the turn manager for a world.
    pub fn initialize(&mut self, initial_year: i32) {
        self.current_year = initial_year;
        self.turn_number = 1;
        self.current_phase = TurnPhase::Income;
        self.events.clear();
    }

    /// Process a complete turn for all factions.
    pub fn process_turn(&mut self, registry: &mut FactionRegistry) -> TurnResult {
        let mut result = TurnResult::new();
        result.turn_number = self.turn_number;
        result.year = self.current_year;

        // Clear events from previous turn
        self.events.clear();

        // Process each phase in order
        for phase in &[
            TurnPhase::Income,
            TurnPhase::Maintenance,
            TurnPhase::Action,
            TurnPhase::News,
        ] {
            self.current_phase = *phase;
            let phase_result = self.process_phase(registry);
            result.phases.push(phase_result);
        }

        // Advance to next turn
        self.advance_turn();
        result.new_year = self.current_year;
        result.new_turn = self.turn_number;

        result
    }

    /// Process a single phase.
    fn process_phase(&self, registry: &mut FactionRegistry) -> PhaseResult {
        let mut result = PhaseResult {
            phase: self.current_phase,
            ..Default::default()
        };

        for faction in registry.factions_mut() {
            if !faction.is_active {
                continue;
            }

            // Get or create faction turn manager
            let faction_id = faction.id.to_uuid();
            let fm = self.faction_managers.get(&faction_id);

            let phase_result = match self.current_phase {
                TurnPhase::Income => self.process_income_phase(faction, fm),
                TurnPhase::Maintenance => self.process_maintenance_phase(faction, fm),
                TurnPhase::Action => self.process_action_phase(faction, fm),
                TurnPhase::News => self.process_news_phase(faction, fm),
            };

            result.faction_results.insert(faction_id, phase_result);
        }

        result
    }

    /// Advance to the next turn.
    fn advance_turn(&mut self) {
        self.turn_number += 1;
        self.current_year += self.config.years_per_turn;

        // Reset phase to income for next turn
        self.current_phase = TurnPhase::Income;
    }

    /// Get turn status summary.
    pub fn status(&self) -> TurnStatusSummary {
        TurnStatusSummary {
            turn_number: self.turn_number,
            current_year: self.current_year,
            current_phase: self.current_phase,
            active_factions: self.faction_managers.len(),
        }
    }
}

/// Configuration for turn manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnManagerConfig {
    /// Years per turn
    pub years_per_turn: i32,
    /// Base income per asset per turn
    pub base_income_per_asset: u32,
    /// Base maintenance cost per asset per turn
    pub base_maintenance_cost: u32,
    /// Maximum assets purchasable per turn
    pub max_purchases_per_turn: u32,
    /// Minimum turns of failure before disband
    pub failure_disband_threshold: u32,
}

impl Default for TurnManagerConfig {
    fn default() -> Self {
        Self {
            years_per_turn: 10,
            base_income_per_asset: 10,
            base_maintenance_cost: 5,
            max_purchases_per_turn: 1,
            failure_disband_threshold: 3,
        }
    }
}

/// Status summary for the turn manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnStatusSummary {
    pub turn_number: u32,
    pub current_year: i32,
    pub current_phase: TurnPhase,
    pub active_factions: usize,
}

/// Result of processing a complete turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub turn_number: u32,
    pub year: i32,
    pub new_year: i32,
    pub new_turn: u32,
    #[serde(default)]
    pub phases: Vec<PhaseResult>,
}

impl TurnResult {
    pub fn new() -> Self {
        Self {
            turn_number: 0,
            year: 0,
            new_year: 0,
            new_turn: 0,
            phases: Vec::new(),
        }
    }

    /// Get total events generated this turn.
    pub fn total_events(&self) -> usize {
        self.phases.iter().map(|p| p.events.len()).sum()
    }
}

/// Result of processing a single phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase: TurnPhase,
    #[serde(default)]
    pub faction_results: HashMap<Uuid, FactionPhaseResult>,
    #[serde(default)]
    pub events: Vec<TurnEvent>,
}

/// Result for a single faction's phase processing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionPhaseResult {
    pub resources_changed: i32,
    pub assets_affected: u32,
    pub goals_updated: u32,
    pub success: bool,
    pub message: Option<String>,
}

/// A turn event that occurred during processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnEvent {
    pub id: Uuid,
    pub year: i32,
    pub phase: TurnPhase,
    pub faction_id: Uuid,
    pub event_type: TurnEventType,
    pub description: String,
    pub target_id: Option<Uuid>,
}

impl TurnEvent {
    pub fn new(
        year: i32,
        phase: TurnPhase,
        faction_id: Uuid,
        event_type: TurnEventType,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            year,
            phase,
            faction_id,
            event_type,
            description,
            target_id: None,
        }
    }
}

/// Types of turn events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnEventType {
    IncomeCollected,
    MaintenancePaid,
    MaintenanceFailed,
    AssetDamaged,
    AssetDestroyed,
    FactionRemnant,
    FactionDisbanded,
    AttackLaunched,
    AttackSucceeded,
    AttackFailed,
    MoveExecuted,
    PurchaseCompleted,
    GoalCompleted,
    BeastBondActivated,
    CampaignStarted,
    CampaignCompleted,
    AllianceFormed,
    WarDeclared,
    PeaceSigned,
}

// ============================================================================
// Faction Turn Manager - Per-Faction Turn Processing
// ============================================================================

/// Per-faction turn manager that tracks faction-specific turn state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTurnManager {
    /// Faction ID this manager belongs to
    pub faction_id: Uuid,
    /// Consecutive maintenance failures (for cascade damage tracking)
    pub consecutive_failures: u32,
    /// Assets purchased this turn (for purchase cap enforcement)
    pub purchases_this_turn: u32,
    /// Orders queued for the action phase
    #[serde(default)]
    pub pending_orders: Vec<FactionOrder>,
    /// Active campaigns for this faction
    #[serde(default)]
    pub campaigns: Vec<CampaignState>,
    /// Turn history for tracking
    #[serde(default)]
    pub history: Vec<FactionTurnSummary>,
}

impl FactionTurnManager {
    /// Create a new faction turn manager.
    pub fn new(faction_id: Uuid) -> Self {
        Self {
            faction_id,
            consecutive_failures: 0,
            purchases_this_turn: 0,
            pending_orders: Vec::new(),
            campaigns: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Record a maintenance failure.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
    }

    /// Record a successful maintenance payment.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
    }

    /// Check if faction should disband due to consecutive failures.
    pub fn should_disband(&self, threshold: u32) -> bool {
        self.consecutive_failures >= threshold
    }

    /// Calculate escalation factor for maintenance damage.
    /// Higher consecutive failures = more damage.
    pub fn escalation_factor(&self) -> u32 {
        // 1 failure = 1x, 2 failures = 2x, 3 failures = 3x (then disband)
        self.consecutive_failures.min(3)
    }

    /// Check if purchase is allowed (1 per turn hard cap).
    pub fn can_purchase(&self, max_per_turn: u32) -> bool {
        self.purchases_this_turn < max_per_turn
    }

    /// Record a purchase.
    pub fn record_purchase(&mut self) {
        self.purchases_this_turn += 1;
    }

    /// Queue an order for action phase.
    pub fn queue_order(&mut self, order: FactionOrder) {
        self.pending_orders.push(order);
    }

    /// Clear orders after action phase.
    pub fn clear_orders(&mut self) {
        self.pending_orders.clear();
        self.purchases_this_turn = 0;
    }

    /// Add a campaign.
    pub fn add_campaign(&mut self, campaign: CampaignState) {
        self.campaigns.push(campaign);
    }

    /// Advance campaigns.
    pub fn advance_campaigns(&mut self) {
        for campaign in &mut self.campaigns {
            campaign.turns_remaining -= 1;
        }
        self.campaigns.retain(|c| c.turns_remaining > 0);
    }

    /// Record turn summary for history.
    pub fn record_turn(&mut self, summary: FactionTurnSummary) {
        self.history.push(summary);
        // Keep only last 20 turns
        if self.history.len() > 20 {
            self.history.remove(0);
        }
    }
}

/// Summary of a faction's turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionTurnSummary {
    pub turn_number: u32,
    pub year: i32,
    pub phase: TurnPhase,
    pub resources_before: u32,
    pub resources_after: u32,
    pub assets_count: u32,
    pub outcome: String,
}

// ============================================================================
// Faction Orders - Action Phase Commands
// ============================================================================

/// An order issued by a faction for the action phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionOrder {
    pub id: Uuid,
    pub order_type: FactionOrderType,
    pub target_id: Option<Uuid>,
    pub target_location: Option<u32>,
    pub budget: u32,
    pub priority: u8,
}

impl FactionOrder {
    /// Create a new order.
    pub fn new(order_type: FactionOrderType) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_type,
            target_id: None,
            target_location: None,
            budget: 0,
            priority: 0,
        }
    }

    /// Create an attack order.
    pub fn attack(target_id: Uuid) -> Self {
        Self::new(FactionOrderType::Attack).with_target_id(target_id)
    }

    /// Create a move order.
    pub fn move_to(location: u32) -> Self {
        Self::new(FactionOrderType::Move).with_location(location)
    }

    /// Create a purchase order.
    pub fn purchase(category: AssetCategory, budget: u32) -> Self {
        Self::new(FactionOrderType::Purchase)
            .with_category(category)
            .with_budget(budget)
    }

    /// Create an expand order.
    pub fn expand(priority: u8) -> Self {
        Self::new(FactionOrderType::Expand).with_priority(priority)
    }

    /// Create a diplomacy order.
    pub fn diplomacy(target_id: Uuid, action: DiplomacyAction) -> Self {
        Self::new(FactionOrderType::Diplomacy {
            action,
            target_faction_id: target_id,
        })
    }

    // Builder methods
    pub fn with_target_id(mut self, id: Uuid) -> Self {
        self.target_id = Some(id);
        self
    }

    pub fn with_location(mut self, loc: u32) -> Self {
        self.target_location = Some(loc);
        self
    }

    pub fn with_budget(mut self, budget: u32) -> Self {
        self.budget = budget;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_category(mut self, category: AssetCategory) -> Self {
        self.order_type = FactionOrderType::Purchase;
        self
    }
}

/// Types of faction orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactionOrderType {
    /// Attack a target (faction or location)
    Attack,
    /// Move assets to a location
    Move,
    /// Purchase new assets
    Purchase,
    /// Expand territory
    Expand,
    /// Diplomatic action
    Diplomacy {
        action: DiplomacyAction,
        target_faction_id: Uuid,
    },
}

/// Diplomatic actions available to factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomacyAction {
    ProposeAlliance,
    BreakAlliance,
    DeclareWar,
    SignPeace,
    ProposeTrade,
}

// ============================================================================
// Phase Implementation
// ============================================================================

impl TurnManager {
    /// Process the INCOME phase.
    fn process_income_phase(
        &self,
        faction: &mut Faction,
        _fm: Option<&FactionTurnManager>,
    ) -> FactionPhaseResult {
        let mut result = FactionPhaseResult::default();
        result.success = true;

        // Initialize turn state if needed
        let turn_state = match faction.turn_state.as_mut() {
            Some(ts) => ts,
            None => {
                faction.turn_state = Some(FactionTurnState::new(self.current_year));
                faction.turn_state.as_mut().unwrap()
            }
        };

        // Calculate income from assets
        let asset_income: u32 = turn_state
            .assets
            .iter()
            .filter(|a| a.hp > 0)
            .map(|a| self.income_for_asset(&a.category))
            .sum();

        // Calculate beast alignment bonus
        let beast_bonus = self.calculate_beast_alignment_bonus(turn_state);

        // Total income
        let total_income = asset_income + beast_bonus;

        turn_state.resources += total_income;
        turn_state.resources_spent = 0;

        result.resources_changed = total_income as i32;

        // Log income event
        if total_income > 0 {
            // Event logged by caller
        }

        result
    }

    /// Process the MAINTENANCE phase.
    fn process_maintenance_phase(
        &self,
        faction: &mut Faction,
        fm: Option<&FactionTurnManager>,
    ) -> FactionPhaseResult {
        let mut result = FactionPhaseResult::default();

        let turn_state = match faction.turn_state.as_mut() {
            Some(ts) => ts,
            None => return result,
        };

        // Calculate maintenance costs
        let active_assets = turn_state.assets.iter().filter(|a| a.hp > 0).count();
        let total_cost = active_assets as u32 * self.config.base_maintenance_cost;

        if turn_state.resources >= total_cost {
            // Can pay maintenance
            turn_state.resources -= total_cost;
            turn_state.resources_spent += total_cost;
            result.success = true;
            result.resources_changed = -(total_cost as i32);

            // Record success if we have faction manager
            if let Some(fm) = fm {
                // Note: We'd need &mut but we already have &mut to faction
                // The consecutive_failures tracking is handled externally
            }
        } else {
            // Cannot pay full maintenance - cascade damage
            result.success = false;
            result.message = Some(format!(
                "Maintenance failure: needed {}, had {}",
                total_cost, turn_state.resources
            ));

            // Calculate escalation factor from consecutive failures
            let escalation = fm.map(|f| f.escalation_factor()).unwrap_or(1);

            // Apply damage to assets
            let damage_per_asset = self.config.base_maintenance_cost * escalation;
            for asset in &mut turn_state.assets {
                if asset.hp > 0 {
                    let destroyed = asset.damage(damage_per_asset);
                    result.assets_affected += 1;
                    if destroyed {
                        // Create a Remnant artifact from the destroyed asset
                        // Faction remnants are named after the faction and provide faction-specific bonuses
                        let remnant = RemnantArtifact::from_faction_asset(
                            asset,
                            faction.id.to_uuid(),
                            turn_state.year,
                        );
                        turn_state.remnant_system.add_remnant(remnant);
                    }
                }
            }

            // Remove destroyed assets
            turn_state.assets.retain(|a| a.hp > 0);
        }

        result
    }

    /// Process the ACTION phase.
    fn process_action_phase(
        &self,
        faction: &mut Faction,
        fm: Option<&FactionTurnManager>,
    ) -> FactionPhaseResult {
        let mut result = FactionPhaseResult::default();
        result.success = true;

        let turn_state = match faction.turn_state.as_mut() {
            Some(ts) => ts,
            None => return result,
        };

        // Get faction manager for order processing
        let faction_manager = fm
            .cloned()
            .unwrap_or_else(|| FactionTurnManager::new(faction.id.to_uuid()));

        // Process pending orders
        for order in &faction_manager.pending_orders {
            let order_result = self.execute_order(order, turn_state);
            match order.order_type {
                FactionOrderType::Purchase => {
                    result.resources_changed -= order.budget as i32;
                }
                _ => {}
            }
            result.assets_affected += order_result.assets_affected;
        }

        result
    }

    /// Process the NEWS phase.
    fn process_news_phase(
        &self,
        faction: &mut Faction,
        fm: Option<&FactionTurnManager>,
    ) -> FactionPhaseResult {
        let mut result = FactionPhaseResult::default();
        result.success = true;

        let turn_state = match faction.turn_state.as_mut() {
            Some(ts) => ts,
            None => return result,
        };

        // Award XP for active participation
        if !turn_state.assets.is_empty() {
            turn_state.xp += 1;
        }

        // Check goal completion
        for goal in &mut turn_state.goals {
            if !goal.completed {
                // Update goal progress based on goal type
                let progress = match goal.goal_type {
                    GoalType::MilitaryConquest => faction.territory_ids.len() as u32,
                    GoalType::CommercialExpansion => (faction.population / 100) as u32,
                    GoalType::CulturalDominance => faction.settlement_ids.len() as u32,
                    GoalType::DiplomaticSupremacy => faction
                        .relations
                        .iter()
                        .filter(|r| matches!(r.relation, crate::faction::FactionRelation::Allied))
                        .count() as u32,
                };
                goal.update_progress(progress);

                if goal.completed {
                    turn_state.xp += goal.xp_reward;
                    result.goals_updated += 1;
                }
            }
        }

        // Advance campaigns
        if let Some(fm) = fm {
            let mut fm_mut = fm.clone();
            fm_mut.advance_campaigns();
        }

        // Refresh assets for next turn
        for asset in &mut turn_state.assets {
            asset.refresh();
        }

        // Mark phase complete in turn state
        turn_state.phase = TurnPhase::Income;
        turn_state.last_processed_turn = turn_state.turn_number;

        result
    }

    /// Calculate income value for an asset category.
    fn income_for_asset(&self, category: &AssetCategory) -> u32 {
        match category {
            AssetCategory::Force => self.config.base_income_per_asset,
            AssetCategory::Cunning => (self.config.base_income_per_asset as f32 * 1.2) as u32,
            AssetCategory::Wealth => (self.config.base_income_per_asset as f32 * 1.5) as u32,
        }
    }

    /// Calculate beast alignment bonus for a faction.
    fn calculate_beast_alignment_bonus(&self, turn_state: &FactionTurnState) -> u32 {
        let mut total_bonus = 0.0f32;

        for bond in &turn_state.beast_bonds {
            let bond_value = match bond.bond_type {
                BeastBondType::Worshiped => 1.5,
                BeastBondType::Allied => 1.2,
                BeastBondType::Tolerated => 1.0,
                BeastBondType::Opposed => -0.5,
            };

            let bonus = bond.bonus.value() * bond_value;
            total_bonus += bonus;
        }

        (total_bonus * 10.0) as u32
    }

    /// Execute a single order.
    fn execute_order(
        &self,
        order: &FactionOrder,
        turn_state: &mut FactionTurnState,
    ) -> OrderResult {
        let mut result = OrderResult::default();

        match &order.order_type {
            FactionOrderType::Attack => {
                if let Some(target_id) = order.target_id {
                    result = self.execute_attack(target_id, turn_state);
                }
            }
            FactionOrderType::Move => {
                if let Some(location) = order.target_location {
                    result = self.execute_move(location, turn_state);
                }
            }
            FactionOrderType::Purchase => {
                // Determine category from order context (simplified)
                let category = if order.budget >= 40 {
                    AssetCategory::Wealth
                } else if order.budget >= 30 {
                    AssetCategory::Force
                } else {
                    AssetCategory::Cunning
                };
                result = self.execute_purchase(category, order.budget, turn_state);
            }
            FactionOrderType::Expand => {
                result = self.execute_expand(turn_state);
            }
            FactionOrderType::Diplomacy {
                action,
                target_faction_id,
            } => {
                result = self.execute_diplomacy(*target_faction_id, *action, self.current_year);
            }
        }

        result
    }

    /// Execute an attack order.
    /// Attack resolution: attacker_score + 2d6 >= defender_score + 10
    fn execute_attack(&self, target_id: Uuid, turn_state: &mut FactionTurnState) -> OrderResult {
        let mut result = OrderResult::default();

        // Calculate attacker's score
        let force_assets = turn_state.assets_by_category(AssetCategory::Force);
        let attacker_score: i32 =
            force_assets.iter().map(|a| a.hp as i32).sum::<i32>() + turn_state.xp as i32 / 10;

        // Deterministic dice roll based on target and turn
        let roll = ((target_id.as_u128() + self.current_year as u128) % 6) as i32
            + ((target_id.as_u128() + turn_state.xp as u128) % 6) as i32
            + 2; // range: 2-12

        let total_attack = attacker_score + roll;

        // Simplified defense calculation (in real impl would look up target)
        let defender_score = 20; // Placeholder
        let defense_threshold = defender_score + 10;

        if total_attack >= defense_threshold {
            result.success = true;
            result.message = Some(format!(
                "Attack succeeded: {} + {} >= {}",
                attacker_score, roll, defense_threshold
            ));
            // Apply damage to defender, gain territory, etc.
        } else {
            result.success = false;
            result.message = Some(format!(
                "Attack failed: {} + {} < {}",
                attacker_score, roll, defense_threshold
            ));
            // Apply counter-damage to attacker
            for asset in &mut turn_state.assets {
                if asset.category == AssetCategory::Force {
                    asset.damage(5);
                    result.assets_affected += 1;
                    break;
                }
            }
        }

        result
    }

    /// Execute a move order.
    fn execute_move(&self, location: u32, turn_state: &mut FactionTurnState) -> OrderResult {
        let mut result = OrderResult::default();

        // Move first available asset to location
        if let Some(asset) = turn_state.assets.iter_mut().find(|a| a.can_act && a.hp > 0) {
            asset.location = Some(location);
            result.success = true;
            result.message = Some(format!("Asset moved to location {}", location));
            result.assets_affected = 1;
        } else {
            result.success = false;
            result.message = Some("No assets available to move".to_string());
        }

        result
    }

    /// Execute a purchase order.
    fn execute_purchase(
        &self,
        category: AssetCategory,
        budget: u32,
        turn_state: &mut FactionTurnState,
    ) -> OrderResult {
        let mut result = OrderResult::default();

        // Check budget
        if turn_state.resources < budget {
            result.success = false;
            result.message = Some(format!(
                "Insufficient resources: have {}, need {}",
                turn_state.resources, budget
            ));
            return result;
        }

        // Create asset
        let hp = match category {
            AssetCategory::Force => 20,
            AssetCategory::Cunning => 15,
            AssetCategory::Wealth => 10,
        };

        let mut asset = FactionAsset::new(category, hp);
        asset.purchased_year = turn_state.year;

        turn_state.add_asset(asset);
        turn_state.resources -= budget;

        result.success = true;
        result.assets_affected = 1;
        result.message = Some(format!(
            "Purchased {:?} asset for {} resources",
            category, budget
        ));

        result
    }

    /// Execute an expand order.
    fn execute_expand(&self, turn_state: &mut FactionTurnState) -> OrderResult {
        let mut result = OrderResult::default();

        // Expand based on asset count
        let asset_count = turn_state.assets.len().min(3);
        let territories_added = asset_count as u32;

        // Placeholder: in real implementation would find frontier tiles
        if asset_count > 0 {
            result.success = true;
            result.message = Some(format!("Expanded {} territories", territories_added));
        } else {
            result.success = false;
            result.message = Some("No assets to expand territory with".to_string());
        }

        result
    }

    /// Execute a diplomacy order.
    fn execute_diplomacy(
        &self,
        target_id: Uuid,
        action: DiplomacyAction,
        _year: i32,
    ) -> OrderResult {
        let mut result = OrderResult::default();

        // Simplified diplomacy - in real implementation would check relations, etc.
        match action {
            DiplomacyAction::ProposeAlliance => {
                result.success = true;
                result.message = Some(format!("Alliance proposed to faction {:?}", target_id));
            }
            DiplomacyAction::DeclareWar => {
                result.success = true;
                result.message = Some(format!("War declared on faction {:?}", target_id));
            }
            DiplomacyAction::SignPeace => {
                result.success = true;
                result.message = Some(format!("Peace signed with faction {:?}", target_id));
            }
            _ => {
                result.success = true;
                result.message = Some(format!("Diplomatic action {:?} executed", action));
            }
        }

        result
    }
}

/// Result of executing an order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderResult {
    pub success: bool,
    pub message: Option<String>,
    pub assets_affected: u32,
    pub resources_changed: i32,
}

// ============================================================================
// Turn Manager Factory
// ============================================================================

/// Factory for creating and managing turn managers.
pub struct TurnManagerFactory;

impl TurnManagerFactory {
    /// Create a new turn manager for a world.
    pub fn create(initial_year: i32) -> TurnManager {
        let mut tm = TurnManager::new();
        tm.initialize(initial_year);
        tm
    }

    /// Create with custom configuration.
    pub fn create_with_config(initial_year: i32, config: TurnManagerConfig) -> TurnManager {
        let mut tm = TurnManager::with_config(config);
        tm.initialize(initial_year);
        tm
    }

    /// Initialize faction managers for all factions in a registry.
    pub fn initialize_faction_managers(
        registry: &FactionRegistry,
    ) -> HashMap<Uuid, FactionTurnManager> {
        let mut managers = HashMap::new();
        for faction in registry.active_factions() {
            let fm = FactionTurnManager::new(faction.id.to_uuid());
            managers.insert(faction.id.to_uuid(), fm);
        }
        managers
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_manager_creation() {
        let tm = TurnManager::new();
        assert_eq!(tm.turn_number, 1);
        assert_eq!(tm.current_phase, TurnPhase::Income);
    }

    #[test]
    fn test_faction_turn_manager_failure_tracking() {
        let mut fm = FactionTurnManager::new(Uuid::new_v4());
        assert_eq!(fm.consecutive_failures, 0);

        fm.record_failure();
        assert_eq!(fm.consecutive_failures, 1);
        assert_eq!(fm.escalation_factor(), 1);

        fm.record_failure();
        assert_eq!(fm.consecutive_failures, 2);
        assert_eq!(fm.escalation_factor(), 2);

        fm.record_success();
        assert_eq!(fm.consecutive_failures, 0);
    }

    #[test]
    fn test_faction_disband_threshold() {
        let mut fm = FactionTurnManager::new(Uuid::new_v4());

        fm.record_failure();
        fm.record_failure();
        assert!(!fm.should_disband(3));

        fm.record_failure();
        assert!(fm.should_disband(3));
    }

    #[test]
    fn test_purchase_cap() {
        let mut fm = FactionTurnManager::new(Uuid::new_v4());
        assert!(fm.can_purchase(1));

        fm.record_purchase();
        assert!(!fm.can_purchase(1));

        fm.clear_orders();
        assert!(fm.can_purchase(1));
    }

    #[test]
    fn test_order_creation() {
        let order = FactionOrder::attack(Uuid::new_v4());
        assert!(matches!(order.order_type, FactionOrderType::Attack));

        let order = FactionOrder::move_to(42);
        assert!(matches!(order.order_type, FactionOrderType::Move));

        let order = FactionOrder::purchase(AssetCategory::Force, 30);
        assert!(matches!(order.order_type, FactionOrderType::Purchase));
    }

    #[test]
    fn test_turn_manager_config() {
        let config = TurnManagerConfig::default();
        assert_eq!(config.years_per_turn, 10);
        assert_eq!(config.base_income_per_asset, 10);
        assert_eq!(config.base_maintenance_cost, 5);
        assert_eq!(config.max_purchases_per_turn, 1);
        assert_eq!(config.failure_disband_threshold, 3);
    }
}
