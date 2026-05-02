/// Event Effect Definitions
/// 
/// Effects describe what happens as a result of an event.
/// Each effect has a type, magnitude, and target specification.
/// 
/// Effects are applied to world state when events are processed,
/// enabling reactive history generation where events cascade into consequences.

use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// All possible effects that can result from an event.
/// 
/// Effects are categorized by domain and describe specific changes
/// to world state. They can be positive, negative, or neutral.
/// 
/// # Effect Application
/// 
/// When an event is applied to world state:
/// 1. Event effects are collected
/// 2. Each effect is validated (target entities exist)
/// 3. Effects are applied in order (conflicts resolved by magnitude)
/// 4. Secondary events may be triggered by cascading effects
/// 
/// # Effect Magnitude
/// 
/// Magnitude determines how impactful an effect is:
/// - `Minor`: Localized, short-term impact
/// - `Moderate`: Regional, medium-term impact  
/// - `Major`: Continental, long-term impact
/// - `Catastrophic`: Global, permanent impact
/// 
/// # Example
/// 
/// ```rust
/// // Population loss effect
/// let effect = EventEffect::PopulationLoss {
///     target: settlement_id,
///     amount: 10000,
///     duration_years: Some(10),
///     cause: "The Great Plague".to_string(),
/// };
/// 
/// // Border shift effect
/// let effect = EventEffect::BorderShift {
///     from: old_owner_id,
///     to: new_owner_id,
///     territory: region_id,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EventEffect {
    // =========================================================================
    // POPULATION EFFECTS
    // =========================================================================
    
    /// Population loss due to war, plague, famine, etc.
    PopulationLoss {
        /// Entity that experienced loss.
        target: Uuid,
        /// Number of people lost.
        amount: u64,
        /// Optional duration over which loss occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
        /// Optional cause description.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Population growth due to birth, immigration, conquest, etc.
    PopulationGrowth {
        /// Entity that experienced growth.
        target: Uuid,
        /// Number of people added.
        amount: u64,
        /// Optional duration over which growth occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
        /// Optional cause description.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Population migration from one location to another.
    PopulationShift {
        /// Origin entity.
        from: Uuid,
        /// Destination entity.
        to: Uuid,
        /// Number of people who moved.
        count: u64,
        /// Optional cause (famine, war, opportunity).
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Displacement of population due to conflict or disaster.
    PopulationDisplacement {
        /// Entity people fled from.
        from: Uuid,
        /// Region(s) people fled to.
        to: Vec<Uuid>,
        /// Number displaced.
        count: u64,
        /// Optional cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // TERRITORY EFFECTS
    // =========================================================================
    
    /// Border change due to conquest, treaty, colonization.
    BorderShift {
        /// Previous owner (None if unclaimed).
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<Uuid>,
        /// New owner.
        to: Uuid,
        /// Territory transferred.
        territory: Uuid,
    },
    
    /// New territory claimed or colonized.
    TerritoryClaim {
        /// Entity claiming territory.
        claimer: Uuid,
        /// Territory being claimed.
        territory: Uuid,
        /// Type of claim (colonization, discovery, legal).
        #[serde(skip_serializing_if = "Option::is_none")]
        claim_type: Option<String>,
    },
    
    /// Territory abandoned or ceded.
    TerritoryLoss {
        /// Previous owner.
        owner: Uuid,
        /// Territory lost.
        territory: Uuid,
        /// Optional cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // MILITARY EFFECTS
    // =========================================================================
    
    /// Military strength change.
    MilitaryChange {
        /// Entity affected.
        target: Uuid,
        /// Amount of change (positive = gain, negative = loss).
        amount: i32,
        /// Type of military change.
        #[serde(skip_serializing_if = "Option::is_none")]
        change_type: Option<MilitaryChangeType>,
        /// Optional cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Formation of military alliance.
    AllianceFormed {
        /// Entities forming alliance.
        members: Vec<Uuid>,
        /// Alliance name/purpose.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Type of alliance (defensive, offensive, trade).
        #[serde(skip_serializing_if = "Option::is_none")]
        alliance_type: Option<String>,
    },
    
    /// Dissolution of military alliance.
    AllianceBroken {
        /// Former alliance members.
        former_members: Vec<Uuid>,
        /// Cause of dissolution.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // POLITICAL EFFECTS
    // =========================================================================
    
    /// Government or leadership change.
    LeadershipChange {
        /// Entity with new leadership.
        target: Uuid,
        /// Type of change.
        change_type: LeadershipChangeType,
        /// Previous leader (for records).
        #[serde(skip_serializing_if = "Option::is_none")]
        previous_leader: Option<Uuid>,
        /// New leader.
        #[serde(skip_serializing_if = "Option::is_none")]
        new_leader: Option<Uuid>,
    },
    
    /// Government or political system change.
    GovernmentChange {
        /// Entity whose government changed.
        target: Uuid,
        /// Type of government before.
        #[serde(skip_serializing_if = "Option::is_none")]
        from_government: Option<String>,
        /// Type of government after.
        #[serde(skip_serializing_if = "Option::is_none")]
        to_government: Option<String>,
        /// Cause of change.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Policy or law change.
    PolicyChange {
        /// Entity affected.
        target: Uuid,
        /// Policy that changed.
        policy: String,
        /// Change description.
        change: String,
    },
    
    /// Diplomatic relationship change.
    DiplomaticChange {
        /// First entity.
        entity1: Uuid,
        /// Second entity.
        entity2: Uuid,
        /// Type of change.
        change_type: DiplomaticChangeType,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // ECONOMIC EFFECTS
    // =========================================================================
    
    /// Economic prosperity or depression.
    EconomicChange {
        /// Entity affected.
        target: Uuid,
        /// Type of change.
        change_type: EconomicChangeType,
        /// Magnitude of change.
        magnitude: EffectMagnitude,
        /// Duration if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
    },
    
    /// New trade route established.
    TradeRouteEstablished {
        /// Trade route identifier.
        route_id: Uuid,
        /// Endpoint entities.
        endpoints: Vec<Uuid>,
        /// Type of trade (land, sea, river).
        #[serde(skip_serializing_if = "Option::is_none")]
        route_type: Option<String>,
    },
    
    /// Trade route disrupted or closed.
    TradeRouteClosed {
        /// Trade route identifier.
        route_id: Uuid,
        /// Cause of closure.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Resource discovery or depletion.
    ResourceChange {
        /// Entity affected.
        target: Uuid,
        /// Resource type.
        resource: String,
        /// Change type (discovered, depleted, increased, decreased).
        change_type: ResourceChangeType,
        /// Amount (for quantitative changes).
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<f64>,
    },
    
    // =========================================================================
    // CULTURAL EFFECTS
    // =========================================================================
    
    /// Cultural flourishing or decline.
    CulturalChange {
        /// Entity affected.
        target: Uuid,
        /// Change type.
        change_type: CulturalChangeType,
        /// Duration if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
    },
    
    /// Cultural adoption or assimilation.
    CulturalAdoption {
        /// Entity adopting.
        target: Uuid,
        /// Culture being adopted.
        culture: String,
        /// Source of adoption.
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<Uuid>,
    },
    
    /// Religious change or reformation.
    ReligiousChange {
        /// Entity affected.
        target: Uuid,
        /// Type of change.
        change_type: ReligiousChangeType,
        /// Previous state.
        #[serde(skip_serializing_if = "Option::is_none")]
        from_religion: Option<String>,
        /// New state.
        #[serde(skip_serializing_if = "Option::is_none")]
        to_religion: Option<String>,
    },
    
    /// Technological advancement or loss.
    TechnologicalChange {
        /// Entity affected.
        target: Uuid,
        /// Technology involved.
        technology: String,
        /// Change type.
        change_type: TechnologicalChangeType,
        /// Source/origin of technology.
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<Uuid>,
    },
    
    // =========================================================================
    // INFRASTRUCTURE EFFECTS
    // =========================================================================
    
    /// Construction of building/monument/infrastructure.
    Construction {
        /// Builder entity.
        builder: Uuid,
        /// What was built.
        structure: String,
        /// Location.
        location: Uuid,
        /// Construction type.
        #[serde(skip_serializing_if = "Option::is_none")]
        construction_type: Option<String>,
    },
    
    /// Destruction of building/monument/infrastructure.
    Destruction {
        /// Entity responsible.
        destroyer: Uuid,
        /// What was destroyed.
        structure: String,
        /// Location.
        location: Uuid,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // ENVIRONMENTAL EFFECTS
    // =========================================================================
    
    /// Climate or environmental change.
    EnvironmentalChange {
        /// Region affected.
        region: Uuid,
        /// Change type.
        change_type: EnvironmentalChangeType,
        /// Duration if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
        /// Magnitude of change.
        magnitude: EffectMagnitude,
    },
    
    /// Disease or plague outbreak.
    DiseaseOutbreak {
        /// Disease identifier.
        disease_id: Uuid,
        /// Origin location.
        origin: Uuid,
        /// Regions affected.
        affected: Vec<Uuid>,
        /// Mortality rate (0.0 to 1.0).
        #[serde(skip_serializing_if = "Option::is_none")]
        mortality_rate: Option<f32>,
        /// Duration in years.
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_years: Option<i32>,
    },
    
    /// Species extinction.
    SpeciesExtinction {
        /// Species that went extinct.
        species: String,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // SOCIAL EFFECTS
    // =========================================================================
    
    /// Social unrest or stability change.
    SocialUnrest {
        /// Entity affected.
        target: Uuid,
        /// Change direction.
        increase: bool,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Migration wave.
    MigrationWave {
        /// Origin.
        origin: Uuid,
        /// Destination.
        destination: Uuid,
        /// Migrant count.
        count: u64,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    /// Reputation or prestige change.
    ReputationChange {
        /// Entity affected.
        target: Uuid,
        /// Change amount (+/-).
        amount: i32,
        /// Cause.
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },
    
    // =========================================================================
    // GENERIC / COMPOUND EFFECTS
    // =========================================================================
    
    /// Generic effect with custom parameters for extensibility.
    Custom {
        /// Effect name/type identifier.
        name: String,
        /// Parameters as key-value pairs.
        params: std::collections::HashMap<String, String>,
    },
}

use EventEffect::*;

impl EventEffect {
    /// Get the effect type as a string for logging/debugging.
    pub fn effect_name(&self) -> &str {
        match self {
            PopulationLoss { .. } => "population_loss",
            PopulationGrowth { .. } => "population_growth",
            PopulationShift { .. } => "population_shift",
            PopulationDisplacement { .. } => "population_displacement",
            BorderShift { .. } => "border_shift",
            TerritoryClaim { .. } => "territory_claim",
            TerritoryLoss { .. } => "territory_loss",
            MilitaryChange { .. } => "military_change",
            AllianceFormed { .. } => "alliance_formed",
            AllianceBroken { .. } => "alliance_broken",
            LeadershipChange { .. } => "leadership_change",
            GovernmentChange { .. } => "government_change",
            PolicyChange { .. } => "policy_change",
            DiplomaticChange { .. } => "diplomatic_change",
            EconomicChange { .. } => "economic_change",
            TradeRouteEstablished { .. } => "trade_route_established",
            TradeRouteClosed { .. } => "trade_route_closed",
            ResourceChange { .. } => "resource_change",
            CulturalChange { .. } => "cultural_change",
            CulturalAdoption { .. } => "cultural_adoption",
            ReligiousChange { .. } => "religious_change",
            TechnologicalChange { .. } => "technological_change",
            Construction { .. } => "construction",
            Destruction { .. } => "destruction",
            EnvironmentalChange { .. } => "environmental_change",
            DiseaseOutbreak { .. } => "disease_outbreak",
            SpeciesExtinction { .. } => "species_extinction",
            SocialUnrest { .. } => "social_unrest",
            MigrationWave { .. } => "migration_wave",
            ReputationChange { .. } => "reputation_change",
            Custom { name, .. } => name,
        }
    }
    
    /// Get the primary target entity ID, if any.
    pub fn primary_target(&self) -> Option<Uuid> {
        match self {
            PopulationLoss { target, .. } => Some(*target),
            PopulationGrowth { target, .. } => Some(*target),
            PopulationShift { from, to, .. } => Some(*to),
            PopulationDisplacement { to, .. } => to.first().copied(),
            BorderShift { to, .. } => Some(*to),
            TerritoryClaim { claimer, .. } => Some(*claimer),
            TerritoryLoss { owner, .. } => Some(*owner),
            MilitaryChange { target, .. } => Some(*target),
            AllianceFormed { members, .. } => members.first().copied(),
            AllianceBroken { former_members, .. } => former_members.first().copied(),
            LeadershipChange { target, .. } => Some(*target),
            GovernmentChange { target, .. } => Some(*target),
            PolicyChange { target, .. } => Some(*target),
            DiplomaticChange { entity1, .. } => Some(*entity1),
            EconomicChange { target, .. } => Some(*target),
            TradeRouteEstablished { endpoints, .. } => endpoints.first().copied(),
            TradeRouteClosed { .. } => None,
            ResourceChange { target, .. } => Some(*target),
            CulturalChange { target, .. } => Some(*target),
            CulturalAdoption { target, .. } => Some(*target),
            ReligiousChange { target, .. } => Some(*target),
            TechnologicalChange { target, .. } => Some(*target),
            Construction { builder, .. } => Some(*builder),
            Destruction { destroyer, .. } => Some(*destroyer),
            EnvironmentalChange { region, .. } => Some(*region),
            DiseaseOutbreak { origin, .. } => Some(*origin),
            SpeciesExtinction { .. } => None,
            SocialUnrest { target, .. } => Some(*target),
            MigrationWave { destination, .. } => Some(*destination),
            ReputationChange { target, .. } => Some(*target),
            Custom { .. } => None,
        }
    }
    
    /// Check if this effect is positive (beneficial) for the target.
    pub fn is_positive(&self) -> bool {
        match self {
            PopulationGrowth { .. } => true,
            PopulationShift { .. } => true,
            TerritoryClaim { .. } => true,
            MilitaryChange { amount, .. } => *amount > 0,
            AllianceFormed { .. } => true,
            LeadershipChange { change_type: LeadershipChangeType::Normal { .. }, .. } => true,
            EconomicChange { change_type: EconomicChangeType::Prosperity, .. } => true,
            TradeRouteEstablished { .. } => true,
            ResourceChange { change_type: ResourceChangeType::Discovered, .. } => true,
            CulturalChange { change_type: CulturalChangeType::Flourishing, .. } => true,
            TechnologicalChange { change_type: TechnologicalChangeType::Advancement, .. } => true,
            Construction { .. } => true,
            ReputationChange { amount, .. } => *amount > 0,
            _ => false,
        }
    }
    
    /// Check if this effect is negative (harmful) for the target.
    pub fn is_negative(&self) -> bool {
        match self {
            PopulationLoss { .. } => true,
            TerritoryLoss { .. } => true,
            MilitaryChange { amount, .. } => *amount < 0,
            AllianceBroken { .. } => true,
            LeadershipChange { change_type: LeadershipChangeType::Overthrow, .. } => true,
            GovernmentChange { cause: Some(c), .. } if c.contains("collapse") => true,
            EconomicChange { change_type: EconomicChangeType::Depression, .. } => true,
            TradeRouteClosed { .. } => true,
            ResourceChange { change_type: ResourceChangeType::Depleted, .. } => true,
            CulturalChange { change_type: CulturalChangeType::Decline, .. } => true,
            TechnologicalChange { change_type: TechnologicalChangeType::Loss, .. } => true,
            Destruction { .. } => true,
            SocialUnrest { increase: true, .. } => true,
            ReputationChange { amount, .. } => *amount < 0,
            _ => false,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Magnitude of an effect for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectMagnitude {
    /// Localized, short-term impact.
    Minor,
    /// Regional, medium-term impact.
    Moderate,
    /// Continental, long-term impact.
    Major,
    /// Global, permanent impact.
    Catastrophic,
}

impl Default for EffectMagnitude {
    fn default() -> Self {
        Self::Moderate
    }
}

/// Type of military change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilitaryChangeType {
    Mobilization,
    Demobilization,
    Victory,
    Defeat,
    Reform,
    Collapse,
    StrengthGain,
    StrengthLoss,
}

/// Type of leadership change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadershipChangeType {
    /// Normal succession (monarch dies, heir takes throne).
    Normal,
    /// Forced removal (coup, revolution).
    Overthrow,
    /// Appointment by council/assembly.
    Election,
    /// Declaration of independence.
    Independence,
    /// Merging of entities.
    Merger,
}

/// Type of diplomatic relationship change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticChangeType {
    AllianceFormed,
    AllianceBroken,
    HostilityEstablished,
    HostilityEnded,
    Embargo,
    TradeAgreement,
    NonAggressionPact,
    SovereigntyRecognized,
}

/// Type of economic change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomicChangeType {
    Prosperity,
    Depression,
    Growth,
    Recession,
    Boom,
    Bust,
    TradeSurplus,
    TradeDeficit,
}

/// Type of resource change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceChangeType {
    Discovered,
    Depleted,
    Increased,
    Decreased,
    NewExtraction,
}

/// Type of cultural change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CulturalChangeType {
    Flourishing,
    Decline,
    Awakening,
    Assimilation,
    Preservation,
}

/// Type of religious change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReligiousChangeType {
    Conversion,
    Schism,
    Reformation,
    Decline,
    Rise,
    Suppression,
}

/// Type of technological change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologicalChangeType {
    Advancement,
    Loss,
    Transfer,
    Innovation,
}

/// Type of environmental change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentalChangeType {
    ClimateShift,
    Desertification,
    Deforestation,
    Flooding,
    Drought,
    IceAge,
    Warming,
    Cooling,
    SoilDegradation,
    WaterSourceChange,

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_effect_names() {
        let id = Uuid::new_v4();
        assert_eq!(EventEffect::PopulationLoss { target: id, amount: 100, duration_years: None, cause: None }.effect_name(), "population_loss");
        assert_eq!(EventEffect::BorderShift { from: Some(id), to: id, territory: id }.effect_name(), "border_shift");
    }
    
    #[test]
    fn test_effect_target() {
        let target_id = Uuid::new_v4();
        let effect = EventEffect::PopulationLoss { 
            target: target_id, 
            amount: 1000, 
            duration_years: None, 
            cause: None 
        };
        assert_eq!(effect.primary_target(), Some(target_id));
    }
    
    #[test]
    fn test_effect_valence() {
        let id = Uuid::new_v4();
        assert!(EventEffect::PopulationGrowth { target: id, amount: 100, duration_years: None, cause: None }.is_positive());
        assert!(EventEffect::PopulationLoss { target: id, amount: 100, duration_years: None, cause: None }.is_negative());
    }
}