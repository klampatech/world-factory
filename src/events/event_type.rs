//! Event Type Definitions
//! 
//! Comprehensive enumeration of all historical event types in World Factory.
//! Events are categorized by their primary domain: Political, Military, Natural,
//! Cultural, Discovery, and Catastrophe.
//!
//! Each event type includes metadata about:
//! - Category grouping for filtering/aggregation
//! - Whether it typically has duration (ongoing vs instantaneous)
//! - Typical participant types (who/what is typically involved)
//! - Default significance weight for historical impact scoring

use serde::{Serialize, Deserialize};

/// All possible types of historical events.
/// 
/// Events are the atomic units of history in World Factory. Each event type
/// belongs to a category that determines how it's processed in history generation.
///
/// # Event Type Categories
///
/// | Category | Description | Typical Duration |
/// |----------|-------------|------------------|
/// | Political | Governance, treaties, founding | Instantaneous - Multi-year |
/// | Military | Wars, battles, sieges | Multi-year |
/// | Natural | Disasters, plagues, weather | Instantaneous - Multi-year |
/// | Cultural | Migrations, festivals, inventions | Multi-year - Centuries |
/// | Discovery | Exploration, first contact, inventions | Instantaneous |
/// | Catastrophe | Civilizational collapse, extinctions | Instantaneous - Multi-year |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // =========================================================================
    // POLITICAL EVENTS
    // =========================================================================
    
    /// A new settlement was founded.
    /// 
    /// Participants: founders, settlement entity
    /// Effects: PopulationGrowth, TerritorialChange, CulturalDevelopment
    SettlementFounded,
    
    /// A new society/civilization was formed from settlements.
    /// 
    /// Participants: founding settlements, founders
    /// Effects: SocietyFormation, CulturalDevelopment, TerritoryClaim
    SocietyFormed,
    
    /// A new nation or political entity was established.
    /// 
    /// Participants: founders, nation entity
    /// Effects: TerritorialChange, NewGovernment, CulturalDevelopment
    NationFounded,
    
    /// A government reform changed political structure.
    /// 
    /// Participants: ruling body, affected populations
    /// Effects: NewGovernment, PolicyChange, SocialUnrest
    GovernmentReform,
    
    /// A ruler ascended to power through succession.
    /// 
    /// Participants: new ruler, previous ruler, succession council
    /// Effects: LeadershipChange, PolicyChange, StabilityShift
    Succession,
    
    /// A treaty was signed between political entities.
    /// 
    /// Participants: all signatory entities
    /// Effects: BorderChange, AllianceFormed, TradeAgreement, PeaceEstablished
    Treaty,
    
    /// A treaty was signed (alias for Treaty).
    TreatySigned,
    
    /// An alliance was formed between political entities.
    /// 
    /// Participants: allied entities
    /// Effects: AllianceFormed, MilitaryCooperation, TradeBoost
    AllianceFormed,
    
    /// An alliance was dissolved or broken.
    /// 
    /// Participants: former allies
    /// Effects: AllianceBroken, DiplomaticTension, MilitarySeparation
    AllianceBroken,
    
    /// A political coup or revolution occurred.
    /// 
    /// Participants: new leaders, old regime, affected population
    /// Effects: LeadershipChange, GovernmentChange, SocialUnrest
    Coup,
    
    /// A political policy or law was enacted.
    /// 
    /// Participants: governing body, affected entities
    /// Effects: PolicyChange, SocialChange, EconomicImpact
    LawEnacted,
    
    /// Social unrest or civil disorder occurred.
    /// 
    /// Participants: affected population, authorities
    /// Effects: SocialUnrest, PolicyChange, EconomicImpact
    CivilUnrest,
    
    /// An economic change event (recession, boom, etc.).
    EconomicChange,
    
    /// Reconstruction after destruction or disaster.
    Reconstruction,
    
    // =========================================================================
    // FIGURE EVENTS
    // =========================================================================
    
    /// A notable figure rose to prominence.
    /// 
    /// Participants: figure, society
    /// Effects: LeadershipChange, CulturalFlourishing
    FigureRises,
    
    /// A notable figure passed away.
    /// 
    /// Participants: figure, society
    /// Effects: LeadershipChange, Succession, CulturalImpact
    FigureDies,
    
    // =========================================================================
    // MILITARY EVENTS
    // =========================================================================
    
    /// A war was declared between two or more factions.
    /// 
    /// Participants: all warring factions
    /// Effects: MilitaryConflict, BorderChanges, PopulationDisplacement
    WarDeclared,
    
    /// A war ended through surrender, treaty, or exhaustion.
    /// 
    /// Participants: former combatants
    /// Effects: PeaceEstablished, Reparations, BorderChanges
    WarEnded,
    
    /// A major battle took place.
    /// 
    /// Participants: opposing forces
    /// Effects: MilitaryLosses, TerritorialChange, MoraleImpact
    Battle,
    
    /// A settlement was besieged.
    /// 
    /// Participants: besiegers, defenders, settlement
    /// Effects: PopulationLoss, Destruction, Surrender
    Siege,
    
    /// One faction conquered another.
    /// 
    /// Participants: conqueror, conquered, territory
    /// Effects: TerritorialChange, CulturalAssimilation, PopulationDisplacement
    Conquest,
    
    /// A raid or plundering expedition occurred.
    /// 
    /// Participants: raiders, targets
    /// Effects: PopulationLoss, PropertyDestruction, CulturalTrauma
    Raid,
    
    /// A military victory was achieved.
    /// 
    /// Participants: victorious faction, defeated faction
    /// Effects: MilitaryPrestige, TerritorialGains, MoraleBoost
    Victory,
    
    /// A military defeat was suffered.
    /// 
    /// Participants: defeated faction
    /// Effects: MilitaryLosses, TerritorialLosses, MoraleDrop
    Defeat,
    
    /// An assassination occurred.
    Assassination,
    
    /// A heroic act was performed.
    HeroicAct,
    
    // =========================================================================
    // NATURAL EVENTS
    // =========================================================================
    
    /// A plague or epidemic spread through a region.
    /// 
    /// Participants: affected populations, disease vector
    /// Effects: PopulationLoss, EconomicDecline, SocialDisruption
    Plague,
    
    /// Population growth event.
    PopulationGrowth,
    
    /// An environmental change occurred (climate shift, etc.).
    EnvironmentalChange,
    
    /// A famine affected a region.
    /// 
    /// Participants: affected populations
    /// Effects: PopulationLoss, Migration, EconomicDecline
    Famine,
    
    /// An earthquake occurred.
    /// 
    /// Participants: affected regions, settlements
    /// Effects: Destruction, PopulationLoss, InfrastructureDamage
    Earthquake,
    
    /// A flood occurred.
    /// 
    /// Participants: affected regions
    /// Effects: Destruction, PopulationLoss, AgriculturalDamage, Displacement
    Flood,
    
    /// A drought affected a region.
    /// 
    /// Participants: affected regions, agricultural areas
    /// Effects: AgriculturalLoss, PopulationDisplacement, EconomicDecline
    Drought,
    
    /// A volcanic eruption occurred.
    /// 
    /// Participants: affected regions, nearby settlements
    /// Effects: Destruction, ClimateChange, PopulationLoss
    Volcano,
    
    /// A wildfire spread through a region.
    /// 
    /// Participants: affected regions, forests
    /// Effects: EnvironmentalDamage, PopulationDisplacement, EconomicLoss
    Wildfire,
    
    /// A severe storm or hurricane struck.
    /// 
    /// Participants: coastal regions, islands
    /// Effects: Destruction, PopulationLoss, InfrastructureDamage
    Storm,
    
    /// A tsunami struck coastal regions.
    /// 
    /// Participants: coastal settlements
    /// Effects: MassiveDestruction, PopulationLoss, CulturalTrauma
    Tsunami,
    
    /// An avalanche or landslide occurred.
    /// 
    /// Participants: mountain regions, settlements
    /// Effects: Destruction, PopulationLoss, InfrastructureDamage
    Avalanche,
    
    // =========================================================================
    // CULTURAL EVENTS
    // =========================================================================
    
    /// A population migration occurred.
    /// 
    /// Participants: migrating population, origin, destination
    /// Effects: PopulationShift, CulturalDiffusion, TerritorialChange
    Migration,
    
    /// A group immigrated to a new region.
    /// 
    /// Participants: immigrants, host region
    /// Effects: PopulationGrowth, CulturalChange, Integration
    Immigration,
    
    /// A cultural festival or celebration occurred.
    /// 
    /// Participants: celebrating culture, visitors
    /// Effects: CulturalUnity, EconomicBoost, SocialCohesion
    Festival,
    
    /// A major cultural achievement or artifact was created.
    /// 
    /// Participants: creators, culture
    /// Effects: CulturalLegacy, ReputationBoost, TouristAttraction
    CulturalAchievement,
    
    /// An art piece or work was created.
    ArtCreated,
    
    /// Cultural adoption of new practices or beliefs.
    CulturalAdoption,
    
    /// A religious event or miracle occurred.
    /// 
    /// Participants: religious figures, followers
    /// Effects: ReligiousSignificance, CulturalShift, PilgrimageBoost
    ReligiousEvent,
    
    /// A religious revelation or prophecy occurred.
    ReligiousReveal,
    
    /// A new religious movement began.
    /// 
    /// Participants: founders, early followers
    /// Effects: ReligiousChange, CulturalShift, PotentialConflict
    ReligiousReformation,
    
    /// A scholarly work was published.
    ScholarlyWork,
    
    /// A golden age of cultural prosperity began.
    /// 
    /// Participants: culture, civilization
    /// Effects: CulturalFlourishing, EconomicProsperity, ScientificAdvance
    GoldenAge,
    
    // =========================================================================
    // DISCOVERY EVENTS
    // =========================================================================
    
    /// A new geographic region was discovered or explored.
    /// 
    /// Participants: explorers, discovered region
    /// Effects: TerritorialClaim, ResourceDiscovery, CulturalContact
    Exploration,
    
    /// A significant discovery was made (scientific, magical, etc.).
    /// 
    /// Participants: discoverer, discovery subject
    /// Effects: TechnologicalAdvance, CulturalChange, EconomicGrowth
    Discovery,
    
    /// First contact between previously isolated cultures.
    /// 
    /// Participants: both cultures, contact location
    /// Effects: CulturalExchange, TradeRoutes, PotentialConflict
    FirstContact,
    
    /// A significant invention was created.
    /// 
    /// Participants: inventor, invention
    /// Effects: TechnologicalAdvance, EconomicChange, SocialChange
    Invention,
    
    /// A new trade route was established.
    /// 
    /// Participants: trade cultures, route endpoints
    /// Effects: EconomicGrowth, CulturalExchange, TerritorialInterest
    TradeRouteEstablished,
    
    /// A new resource deposit was found.
    /// 
    /// Participants: finding culture, location
    /// Effects: EconomicGrowth, TerritorialDispute, DevelopmentBoost
    ResourceDiscovery,
    
    // =========================================================================
    // CATASTROPHE EVENTS
    // =========================================================================
    
    /// A civilization or culture collapsed.
    /// 
    /// Participants: collapsed civilization, successor cultures
    /// Effects: CulturalLoss, PopulationDisplacement, PowerVacuum
    Collapse,
    
    /// Significant destruction occurred to a region or culture.
    /// 
    /// Participants: destroyed entity, perpetrators
    /// Effects: CulturalLoss, PopulationLoss, Reconstruction
    Destruction,
    
    /// A species went extinct.
    /// 
    /// Participants: extinct species, ecosystem
    /// Effects: EcologicalChange, CulturalImpact, FoodChainDisruption
    Extinction,
    
    /// A meteor or asteroid struck.
    /// 
    /// Participants: impact region, world
    /// Effects: MassiveDestruction, ClimateChange, Extinction
    MeteorStrike,
    
    /// A supernatural or magical catastrophe occurred.
    /// 
    /// Participants: affected cultures, magical entities
    /// Effects: MagicalConsequences, CulturalChange, PowerShift
    MagicalCatastrophe,
    
    /// A building or monument of significance was completed.
    /// 
    /// Participants: builders, culture
    /// Effects: CulturalLegacy, TourismBoost, EconomicActivity
    MonumentCompleted,
    
    /// An artifact was created by a notable figure.
    /// 
    /// Participants: creator figure, society
    /// Effects: CulturalLegacy, ArtifactCreation, ReputationBoost
    ArtifactCreated,
    
    /// An artifact was activated or awakened.
    /// 
    /// Participants: activator, artifact
    /// Effects: MagicalConsequences, PowerShift, PotentialCataclysm
    ArtifactActivated,
}

impl EventType {
    /// Get the category this event type belongs to.
    pub fn category(&self) -> EventCategory {
        match self {
            // Political
            Self::SettlementFounded
            | Self::NationFounded
            | Self::SocietyFormed
            | Self::GovernmentReform
            | Self::Succession
            | Self::Treaty
            | Self::AllianceFormed
            | Self::AllianceBroken
            | Self::Coup
            | Self::LawEnacted
            | Self::CivilUnrest
            | Self::TreatySigned
            | Self::EconomicChange
            | Self::Reconstruction
            | Self::FigureRises
            | Self::FigureDies => EventCategory::Political,
            
            // Military
            Self::WarDeclared
            | Self::WarEnded
            | Self::Battle
            | Self::Siege
            | Self::Conquest
            | Self::Raid
            | Self::Victory
            | Self::Defeat
            | Self::Assassination
            | Self::HeroicAct => EventCategory::Military,
            
            // Natural
            Self::Plague
            | Self::PopulationGrowth
            | Self::EnvironmentalChange
            | Self::Famine
            | Self::Earthquake
            | Self::Flood
            | Self::Drought
            | Self::Volcano
            | Self::Wildfire
            | Self::Storm
            | Self::Tsunami
            | Self::Avalanche => EventCategory::Natural,
            
            // Cultural
            Self::Migration
            | Self::Immigration
            | Self::Festival
            | Self::CulturalAchievement
            | Self::ArtCreated
            | Self::CulturalAdoption
            | Self::ReligiousEvent
            | Self::ReligiousReveal
            | Self::ReligiousReformation
            | Self::ScholarlyWork
            | Self::GoldenAge
            | Self::ArtifactCreated => EventCategory::Cultural,
            
            // Discovery
            Self::Exploration
            | Self::Discovery
            | Self::FirstContact
            | Self::Invention
            | Self::TradeRouteEstablished
            | Self::ResourceDiscovery => EventCategory::Discovery,
            
            // Catastrophe
            Self::Collapse
            | Self::Destruction
            | Self::Extinction
            | Self::MeteorStrike
            | Self::MagicalCatastrophe
            | Self::MonumentCompleted
            | Self::ArtifactActivated => EventCategory::Catastrophe,
        }
    }
    
    /// Check if this event type typically has a duration (ongoing).
    pub fn has_duration(&self) -> bool {
        matches!(
            self,
            Self::Plague
            | Self::Famine
            | Self::WarDeclared
            | Self::Migration
            | Self::Siege
            | Self::Collapse
            | Self::GoldenAge
            | Self::ReligiousReformation
            | Self::GovernmentReform
            | Self::CivilUnrest
        )
    }
    
    /// Get a human-readable name for this event type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SettlementFounded => "Settlement Founded",
            Self::SocietyFormed => "Society Formed",
            Self::NationFounded => "Nation Founded",
            Self::GovernmentReform => "Government Reform",
            Self::Succession => "Succession",
            Self::Treaty | Self::TreatySigned => "Treaty Signed",
            Self::AllianceFormed => "Alliance Formed",
            Self::AllianceBroken => "Alliance Broken",
            Self::Coup => "Coup",
            Self::LawEnacted => "Law Enacted",
            Self::CivilUnrest => "Civil Unrest",
            Self::EconomicChange => "Economic Change",
            Self::Reconstruction => "Reconstruction",
            Self::WarDeclared => "War Declared",
            Self::WarEnded => "War Ended",
            Self::Battle => "Battle",
            Self::Siege => "Siege",
            Self::Conquest => "Conquest",
            Self::Raid => "Raid",
            Self::Victory => "Victory",
            Self::Defeat => "Defeat",
            Self::Assassination => "Assassination",
            Self::HeroicAct => "Heroic Act",
            Self::Plague => "Plague",
            Self::PopulationGrowth => "Population Growth",
            Self::EnvironmentalChange => "Environmental Change",
            Self::Famine => "Famine",
            Self::Earthquake => "Earthquake",
            Self::Flood => "Flood",
            Self::Drought => "Drought",
            Self::Volcano => "Volcanic Eruption",
            Self::Wildfire => "Wildfire",
            Self::Storm => "Storm",
            Self::Tsunami => "Tsunami",
            Self::Avalanche => "Avalanche",
            Self::Migration => "Migration",
            Self::Immigration => "Immigration",
            Self::Festival => "Festival",
            Self::CulturalAchievement => "Cultural Achievement",
            Self::ArtCreated => "Art Created",
            Self::CulturalAdoption => "Cultural Adoption",
            Self::ReligiousEvent => "Religious Event",
            Self::ReligiousReveal => "Religious Reveal",
            Self::ReligiousReformation => "Religious Reformation",
            Self::ScholarlyWork => "Scholarly Work",
            Self::GoldenAge => "Golden Age",
            Self::Exploration => "Exploration",
            Self::Discovery => "Discovery",
            Self::FirstContact => "First Contact",
            Self::Invention => "Invention",
            Self::TradeRouteEstablished => "Trade Route Established",
            Self::ResourceDiscovery => "Resource Discovery",
            Self::Collapse => "Collapse",
            Self::Destruction => "Destruction",
            Self::Extinction => "Extinction",
            Self::MeteorStrike => "Meteor Strike",
            Self::MagicalCatastrophe => "Magical Catastrophe",
            Self::MonumentCompleted => "Monument Completed",
            Self::ArtifactCreated => "Artifact Created",
            Self::FigureRises => "Figure Rises to Prominence",
            Self::FigureDies => "Notable Figure Dies",
            Self::ArtifactActivated => "Artifact Activated",
        }
    }
    
    /// Get the default significance weight for this event type.
    /// Scale: 0.0 (minor) to 1.0 (world-altering).
    pub fn default_significance(&self) -> f32 {
        match self {
            // World-altering events (0.9+)
            Self::MeteorStrike => 1.0,
            Self::Extinction => 0.95,
            Self::Collapse => 0.9,
            Self::WarDeclared => 0.85,
            Self::FirstContact => 0.85,
            Self::Plague => 0.8,
            Self::NationFounded => 0.8,
            Self::SocietyFormed => 0.75,
            Self::SettlementFounded => 0.7,
            Self::FigureRises => 0.6,
            
            // Major events (0.6-0.8)
            Self::WarEnded => 0.75,
            Self::Conquest => 0.75,
            Self::Treaty => 0.7,
            Self::MagicalCatastrophe => 0.7,
            Self::ArtifactActivated => 0.65,
            Self::Discovery => 0.65,
            Self::GovernmentReform => 0.65,
            Self::Succession => 0.6,
            Self::ResourceDiscovery => 0.6,
            
            // Moderate events (0.4-0.6)
            Self::Battle => 0.55,
            Self::Migration => 0.55,
            Self::Siege => 0.5,
            Self::Immigration => 0.5,
            Self::Invention => 0.5,
            Self::TradeRouteEstablished => 0.5,
            Self::Coup => 0.5,
            Self::Earthquake => 0.5,
            Self::Volcano => 0.5,
            Self::Famine => 0.5,
            Self::GoldenAge => 0.55,
            Self::ReligiousReformation => 0.55,
            
            // Minor events (0.2-0.4)
            Self::Exploration => 0.45,
            Self::Raid => 0.4,
            Self::Flood => 0.4,
            Self::Storm => 0.35,
            Self::Drought => 0.35,
            Self::Victory => 0.4,
            Self::Defeat => 0.4,
            Self::AllianceFormed => 0.4,
            Self::AllianceBroken => 0.35,
            Self::CulturalAchievement => 0.35,
            Self::ReligiousEvent => 0.3,
            Self::Festival => 0.25,
            Self::LawEnacted => 0.3,
            Self::CivilUnrest => 0.45,
            Self::MonumentCompleted => 0.3,
            Self::Wildfire => 0.3,
            Self::Tsunami => 0.45,
            Self::Avalanche => 0.3,
            Self::Destruction => 0.45,
            // Additional political events
            Self::TreatySigned => 0.7,
            Self::EconomicChange => 0.6,
            Self::Reconstruction => 0.5,
            // Military/Political events
            Self::Assassination => 0.65,
            Self::HeroicAct => 0.5,
            Self::PopulationGrowth => 0.55,
            // Cultural events
            Self::ArtCreated => 0.35,
            Self::ArtifactCreated => 0.4,
            Self::FigureDies => 0.55,
            Self::CulturalAdoption => 0.3,
            Self::EnvironmentalChange => 0.45,
            Self::ReligiousReveal => 0.45,
            Self::ScholarlyWork => 0.4,
        }
    }
    
    /// Get typical participant types for this event.
    pub fn participant_types(&self) -> Vec<&'static str> {
        match self {
            Self::SettlementFounded => vec!["settlement", "founders", "culture"],
            Self::NationFounded => vec!["nation", "founders", "territory"],
            Self::GovernmentReform => vec!["government", "rulers", "citizens"],
            Self::Succession => vec!["ruler", "heir", "council"],
            Self::Treaty => vec!["nations", "diplomats"],
            Self::AllianceFormed | Self::AllianceBroken => vec!["nations"],
            Self::Coup => vec!["new_leaders", "old_regime", "military"],
            Self::LawEnacted => vec!["government", "citizens"],
            Self::WarDeclared | Self::WarEnded => vec!["nations", "military"],
            Self::Battle | Self::Victory | Self::Defeat => vec!["military_forces"],
            Self::Siege => vec!["besiegers", "defenders", "settlement"],
            Self::Conquest => vec!["conqueror", "conquered", "territory"],
            Self::Raid => vec!["raiders", "victims"],
            Self::Plague | Self::Famine => vec!["populations", "regions"],
            Self::Earthquake | Self::Flood | Self::Drought | Self::Volcano => vec!["regions", "settlements"],
            Self::Wildfire => vec!["regions", "forests"],
            Self::Storm | Self::Tsunami => vec!["coastal_regions", "settlements"],
            Self::Avalanche => vec!["mountain_regions"],
            Self::Migration => vec!["migrating_population", "origin", "destination"],
            Self::Immigration => vec!["immigrants", "host_region"],
            Self::Festival => vec!["culture", "participants"],
            Self::CulturalAchievement => vec!["creators", "culture"],
            Self::ReligiousEvent | Self::ReligiousReformation => vec!["religious_figures", "followers"],
            Self::GoldenAge => vec!["culture", "civilization"],
            Self::Exploration => vec!["explorers", "discovered_region"],
            Self::Discovery => vec!["discoverer", "discovery"],
            Self::FirstContact => vec!["cultures", "contact_location"],
            Self::Invention => vec!["inventor", "society"],
            Self::TradeRouteEstablished => vec!["trade_cultures", "route"],
            Self::ResourceDiscovery => vec!["finding_culture", "location"],
            Self::Collapse => vec!["civilization", "successors"],
            Self::Destruction => vec!["destroyed_entity", "perpetrators"],
            Self::Extinction => vec!["species", "ecosystem"],
            Self::MeteorStrike => vec!["impact_region", "world"],
            Self::MagicalCatastrophe => vec!["cultures", "magical_entities"],
            Self::MonumentCompleted => vec!["builders", "culture"],
            // Additional political events
            Self::TreatySigned => vec!["nations", "diplomats"],
            Self::EconomicChange => vec!["economies", "affected_populations"],
            Self::CivilUnrest => vec!["protesters", "authorities", "affected_population"],
            Self::Reconstruction => vec!["communities", "regions", "resources"],
            // Military/Political events
            Self::Assassination => vec!["target", "assassin", "affected_parties"],
            Self::HeroicAct => vec!["hero", "witnesses", "affected_party"],
            Self::PopulationGrowth => vec!["population", "region"],
            // Cultural events
            Self::ArtCreated => vec!["artist", "culture", "audience"],
            Self::CulturalAdoption => vec!["culture", "adopting_community"],
            Self::EnvironmentalChange => vec!["region", "affected_populations", "ecosystem"],
            Self::ReligiousReveal => vec!["prophet", "followers", "region"],
            Self::ScholarlyWork => vec!["scholar", "culture", "society"],
            // Society events
            Self::SocietyFormed => vec!["founders", "settlements", "species"],
            // Figure events
            Self::FigureRises => vec!["figure", "society"],
            Self::FigureDies => vec!["figure", "society"],
            // Artifact events
            Self::ArtifactCreated => vec!["creator_figure", "society", "artifact"],
            Self::ArtifactActivated => vec!["activator", "artifact"],
        }
    }
}

/// Event category grouping for filtering and aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    /// Political and governance events.
    Political,
    /// Military and conflict events.
    Military,
    /// Natural disasters and environmental events.
    Natural,
    /// Cultural, social, and religious events.
    Cultural,
    /// Exploration, discovery, and technological advancement.
    Discovery,
    /// Major catastrophes and destructive events.
    Catastrophe,
}

impl EventCategory {
    /// Get all event types in this category.
    pub fn event_types(&self) -> Vec<EventType> {
        match self {
            Self::Political => vec![
                EventType::SettlementFounded,
                EventType::SocietyFormed,
                EventType::NationFounded,
                EventType::GovernmentReform,
                EventType::Succession,
                EventType::Treaty,
                EventType::AllianceFormed,
                EventType::AllianceBroken,
                EventType::Coup,
                EventType::LawEnacted,
                EventType::FigureRises,
                EventType::FigureDies,
            ],
            Self::Military => vec![
                EventType::WarDeclared,
                EventType::WarEnded,
                EventType::Battle,
                EventType::Siege,
                EventType::Conquest,
                EventType::Raid,
                EventType::Victory,
                EventType::Defeat,
            ],
            Self::Natural => vec![
                EventType::Plague,
                EventType::Famine,
                EventType::Earthquake,
                EventType::Flood,
                EventType::Drought,
                EventType::Volcano,
                EventType::Wildfire,
                EventType::Storm,
                EventType::Tsunami,
                EventType::Avalanche,
            ],
            Self::Cultural => vec![
                EventType::Migration,
                EventType::Immigration,
                EventType::Festival,
                EventType::CulturalAchievement,
                EventType::ReligiousEvent,
                EventType::ReligiousReformation,
                EventType::GoldenAge,
                EventType::ArtifactCreated,
            ],
            Self::Discovery => vec![
                EventType::Exploration,
                EventType::Discovery,
                EventType::FirstContact,
                EventType::Invention,
                EventType::TradeRouteEstablished,
                EventType::ResourceDiscovery,
            ],
            Self::Catastrophe => vec![
                EventType::Collapse,
                EventType::Destruction,
                EventType::Extinction,
                EventType::MeteorStrike,
                EventType::MagicalCatastrophe,
                EventType::MonumentCompleted,
                EventType::ArtifactActivated,
            ],
        }
    }
    
    /// Get human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Political => "Political",
            Self::Military => "Military",
            Self::Natural => "Natural",
            Self::Cultural => "Cultural",
            Self::Discovery => "Discovery",
            Self::Catastrophe => "Catastrophe",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_type_categories() {
        assert_eq!(EventType::SettlementFounded.category(), EventCategory::Political);
        assert_eq!(EventType::WarDeclared.category(), EventCategory::Military);
        assert_eq!(EventType::Plague.category(), EventCategory::Natural);
        assert_eq!(EventType::Migration.category(), EventCategory::Cultural);
        assert_eq!(EventType::Discovery.category(), EventCategory::Discovery);
        assert_eq!(EventType::Collapse.category(), EventCategory::Catastrophe);
    }
    
    #[test]
    fn test_event_type_names() {
        assert_eq!(EventType::SettlementFounded.name(), "Settlement Founded");
        assert_eq!(EventType::MeteorStrike.name(), "Meteor Strike");
        assert_eq!(EventType::FirstContact.name(), "First Contact");
    }
    
    #[test]
    fn test_event_type_significance() {
        assert!(EventType::MeteorStrike.default_significance() >= 0.9);
        assert!(EventType::Festival.default_significance() < 0.4);
    }
    
    #[test]
    fn test_duration_events() {
        assert!(EventType::Plague.has_duration());
        assert!(EventType::WarDeclared.has_duration());
        assert!(!EventType::Battle.has_duration());
        assert!(!EventType::Discovery.has_duration());
    }
    
    #[test]
    fn test_event_category_event_types() {
        let political_types = EventCategory::Political.event_types();
        assert!(political_types.contains(&EventType::SettlementFounded));
        assert!(political_types.contains(&EventType::Treaty));
        assert_eq!(political_types.len(), 12);
    }
}