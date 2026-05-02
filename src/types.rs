//! Core types for World Factory.
//! 
//! This module defines all core data types with full serialization support.
//! All types derive Serialize/Deserialize for JSON persistence.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// Identifier Types
// ============================================================================

/// Unique identifier for any world entity.
/// Uses UUID v4 for generation, includes entity type for disambiguation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    /// The actual UUID
    pub id: Uuid,
    /// Entity type hint for faster deserialization
    #[serde(rename = "type")]
    pub entity_type: EntityType,
}

impl EntityId {
    /// Create a new entity ID with the given type.
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
        }
    }
    
    /// Create from an existing UUID.
    pub fn from_uuid(id: Uuid, entity_type: EntityType) -> Self {
        Self { id, entity_type }
    }
    
    /// Get the underlying UUID.
    pub fn to_uuid(&self) -> Uuid {
        self.id
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.entity_type.short_name(), self.id)
    }
}

/// Categories of entities in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    World,
    Continent,
    Region,
    Nation,
    Province,
    Settlement,
    Person,
    Event,
    Timeline,
}

impl EntityType {
    /// Short name for compact serialization (e.g., in EntityId).
    pub fn short_name(&self) -> &'static str {
        match self {
            EntityType::World => "world",
            EntityType::Continent => "cont",
            EntityType::Region => "reg",
            EntityType::Nation => "nat",
            EntityType::Province => "prv",
            EntityType::Settlement => "stl",
            EntityType::Person => "per",
            EntityType::Event => "evt",
            EntityType::Timeline => "tl",
        }
    }
}

// ============================================================================
// Timestamp Types  
// ============================================================================

/// Timestamp with full precision for historical records.
/// Internal representation is UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Current time.
    pub fn now() -> Self {
        Self(Utc::now())
    }
    
    /// Create from a DateTime.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
    
    /// Create from Unix timestamp seconds.
    pub fn from_unix(seconds: i64) -> Self {
        Self(DateTime::from_timestamp(seconds, 0).unwrap_or_else(Utc::now))
    }
    
    /// Convert to Unix timestamp.
    pub fn to_unix(&self) -> i64 {
        self.0.timestamp()
    }
    
    /// Access the underlying DateTime.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

/// Historical time point with optional precision.
/// Used when exact timing is unknown or varies by source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalTime {
    /// Specific year with optional month/day
    Year {
        year: i32,
        month: Option<u8>,
        day: Option<u8>,
        /// Approximate flag - true if this is an estimate
        approximate: bool,
    },
    /// Time relative to reference (e.g., "50 years before current event")
    Relative {
        years: i32,
        months: u8,
    },
    /// Unknown time
    Unknown,
}

impl HistoricalTime {
    /// Create a year-only timestamp, marking as approximate by default.
    pub fn year(year: i32) -> Self {
        Self::Year { year, month: None, day: None, approximate: true }
    }
    
    /// Create a specific date.
    pub fn date(year: i32, month: u8, day: u8) -> Self {
        Self::Year { year, month: Some(month), day: Some(day), approximate: false }
    }
    
    /// Mark current timepoint as approximate.
    pub fn approximate(self) -> Self {
        match self {
            Self::Year { year, month, day, .. } => Self::Year { year, month, day, approximate: true },
            other => other,
        }
    }
}

// ============================================================================
// World State Core
// ============================================================================

/// Core world metadata and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub id: EntityId,
    pub name: String,
    pub seed: u64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorldMetadata>,
}

impl World {
    pub fn new(name: String, seed: u64) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::World),
            name,
            seed,
            created_at: now,
            updated_at: now,
            description: None,
            metadata: None,
        }
    }
}

/// Optional metadata for world configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// Genre setting affects available biomes, events, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<Genre>,
    /// Starting technology level (affects available content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_level: Option<TechLevel>,
    /// Magic system presence and type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic: Option<MagicSystem>,
}

impl Default for WorldMetadata {
    fn default() -> Self {
        Self {
            genre: Some(Genre::Fantasy), // Default to fantasy for World Factory
            tech_level: Some(TechLevel::Medieval),
            magic: Some(MagicSystem::default()),
        }
    }
}

/// Genre settings for world generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Genre {
    Fantasy,
    SciFi,
    Historical,
    Modern,
    PostApocalyptic,
    Horror,
    Cyberpunk,
    Steampunk,
}

/// Technology advancement levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechLevel {
    Prehistoric,
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Industrial,
    Modern,
    Future,
}

/// Magic system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSystem {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_type: Option<MagicType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<MagicRarity>,
}

impl Default for MagicSystem {
    fn default() -> Self {
        Self {
            enabled: true,
            system_type: Some(MagicType::Arcane),
            rarity: Some(MagicRarity::Rare),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagicType {
    Arcane,
    Divine,
    Primal,
    Psionic,
    Technological,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagicRarity {
    Common,  // Magic in everyday life
    Uncommon,
    Rare,    // Wizards and magical regions exist
    Epic,    // Legendary heroes, unique artifacts
    Mythic,  // Gods walk the earth
}

// ============================================================================
// Region & Geography Types
// ============================================================================

/// Geographic region with optional administrative status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: EntityId,
    pub world_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_region_id: Option<Uuid>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub area_km2: f64,
    pub center_lat: f64,
    pub center_lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub climate: Option<ClimateZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub political_data: Option<PoliticalData>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Region {
    pub fn new(world_id: Uuid, name: String, area_km2: f64, center_lat: f64, center_lon: f64) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Region),
            world_id,
            parent_region_id: None,
            name,
            description: None,
            area_km2,
            center_lat,
            center_lon,
            climate: None,
            political_data: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Political organization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub government_type: Option<GovernmentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capital_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ruling_faction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub founded_year: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernmentType {
    Autocracy,
    Oligarchy,
    Theocracy,
    Republic,
    Democracy,
    Monarchy,
    Tribal,
    Confederation,
    Anarchy,
}

// ============================================================================
// Entity & Settlement Types
// ============================================================================

/// A settlement (city, town, village).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub id: EntityId,
    pub region_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_type: Option<SettlementType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<u64>,
    pub location: GeoLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_id: Option<crate::species::SpeciesId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notable_features: Option<Vec<String>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Settlement {
    pub fn new(region_id: Uuid, name: String, location: GeoLocation) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Settlement),
            region_id,
            name,
            settlement_type: None,
            population: None,
            location,
            species_id: None,
            description: None,
            notable_features: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementType {
    Hamlet,
    Village,
    Town,
    City,
    Metropolis,
    Capital,
    Fortress,
    Port,
    SacredSite,
}

/// Geographic location with optional elevation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f32>,
}

impl GeoLocation {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            elevation_m: None,
        }
    }
    
    pub fn with_elevation(lat: f64, lon: f64, elevation: f32) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            elevation_m: Some(elevation),
        }
    }
}

// ============================================================================
// Person & Character Types
// ============================================================================

/// A person (historical figure, NPC, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: EntityId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<PersonName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_time: Option<HistoricalTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_time: Option<HistoricalTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Person {
    pub fn new() -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Person),
            name: None,
            birth_time: None,
            death_time: None,
            birthplace_id: None,
            culture: None,
            titles: None,
            description: None,
            biography: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for Person {
    fn default() -> Self {
        Self::new()
    }
}

/// Person name with optional components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epithet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl PersonName {
    pub fn new(given: String, family: String) -> Self {
        Self {
            given: Some(given),
            family: Some(family),
            epithet: None,
            title: None,
        }
    }
}

impl std::fmt::Display for PersonName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if let Some(title) = &self.title {
            parts.push(title.as_str());
        }
        if let Some(given) = &self.given {
            parts.push(given.as_str());
        }
        if let Some(family) = &self.family {
            parts.push(family.as_str());
        }
        write!(f, "{}", parts.join(" "))
    }
}

// ============================================================================
// Event Types
// ============================================================================

/// A historical event in the world timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub id: EntityId,
    pub world_id: Uuid,
    pub name: String,
    pub time: HistoricalTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<HistoricalTime>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consequences: Option<Vec<String>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl HistoricalEvent {
    pub fn new(world_id: Uuid, name: String, time: HistoricalTime, description: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Event),
            world_id,
            name,
            time,
            end_time: None,
            description,
            event_type: None,
            participants: None,
            location_id: None,
            consequences: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Political,
    Military,
    Natural,
    Cultural,
    Religious,
    Economic,
    Discovery,
    Catastrophe,
    Founding,
    Treaty,
}

// Re-export comprehensive event types from events module
pub use crate::events::event_type::EventType as ComprehensiveEventType;
pub use crate::events::Event;

// ============================================================================
// Timeline Types
// ============================================================================

/// A timeline tracking events chronologically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: EntityId,
    pub world_id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub events: Vec<Uuid>,  // Event IDs in chronological order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i32>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Timeline {
    pub fn new(world_id: Uuid, name: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Timeline),
            world_id,
            name,
            description: None,
            events: Vec::new(),
            start_year: None,
            end_year: None,
            created_at: now,
            updated_at: now,
        }
    }
}

// ============================================================================
// Re-exports from terrain module
// ============================================================================

// Re-export terrain types for convenience
pub use super::terrain::biome::{BiomeType, VegetationType, ClimateZone, MoistureLevel, ElevationZone, BiomeColor, BiomeColorMapping, AlpineBiomeConfig};
