//! Notable Figures System for World Factory
//!
//! This module provides historical figure management for procedural world generation.
//! Figures are generated from significant events and linked to settlements, species,
//! and other world entities.
//!
//! ## Core Types
//!
//! - [`NotableFigure`] - A person with historical significance, extending the base `Person` type
//! - [`FigureType`] - Enum categorizing figure roles (Monarch, MilitaryLeader, Scholar, etc.)
//! - [`FigureStore`] - In-memory storage for figure collections
//!
//! ## Generation
//!
//! The [`FigureGenerator`] creates statistically valid historical figures based on:
//! - World era and timeline density
//! - Significant historical events
//! - Settlement populations and cultures
//! - Power-law significance distribution

use crate::events::{Event, EventType};
use crate::types::{EntityId, EntityType, Timestamp};
use crate::util::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Figure Types
// ============================================================================

/// Types of notable figures in world history
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FigureType {
    /// Ruling monarch (king, queen, emperor, etc.)
    Monarch,
    /// Military commander or general
    MilitaryLeader,
    /// Scholar, philosopher, scientist
    Scholar,
    /// Artist, musician, poet
    Artist,
    /// Religious leader or prophet
    ReligiousLeader,
    /// Explorer or adventurer
    Explorer,
    /// Inventor or engineer
    Inventor,
    /// Hero who performed notable deeds
    Hero,
    /// Villain or antagonist figure
    Villain,
    /// Folk hero from oral tradition
    FolkHero,
    /// Legendary or mythical figure
    Legendary,
}

impl FigureType {
    /// Get human-readable label for the figure type
    pub fn label(&self) -> &'static str {
        match self {
            FigureType::Monarch => "Monarch",
            FigureType::MilitaryLeader => "Military Leader",
            FigureType::Scholar => "Scholar",
            FigureType::Artist => "Artist",
            FigureType::ReligiousLeader => "Religious Leader",
            FigureType::Explorer => "Explorer",
            FigureType::Inventor => "Inventor",
            FigureType::Hero => "Hero",
            FigureType::Villain => "Villain",
            FigureType::FolkHero => "Folk Hero",
            FigureType::Legendary => "Legendary Figure",
        }
    }

    /// Get typical lifespan modifier for this figure type
    /// Monarchs and scholars often live longer, military leaders in wartime may die younger
    pub fn lifespan_modifier(&self) -> f32 {
        match self {
            FigureType::Monarch => 1.1,         // Better nutrition, care
            FigureType::MilitaryLeader => 0.85, // Combat risk
            FigureType::Scholar => 1.15,        // Sedentary life, access to medicine
            FigureType::Artist => 1.0,
            FigureType::ReligiousLeader => 1.1,
            FigureType::Explorer => 0.8, // Hardship and danger
            FigureType::Inventor => 1.05,
            FigureType::Hero => 0.75, // Heroic deaths
            FigureType::Villain => 0.9,
            FigureType::FolkHero => 1.0,
            FigureType::Legendary => 1.5, // Mythical longevity
        }
    }

    /// Determine figure type from an event
    pub fn from_event(event_type: &EventType) -> Option<Self> {
        match event_type {
            EventType::SettlementFounded => Some(FigureType::Monarch),
            EventType::WarDeclared | EventType::Battle => Some(FigureType::MilitaryLeader),
            EventType::TreatySigned | EventType::AllianceFormed => Some(FigureType::Monarch),
            EventType::Plague | EventType::Famine => Some(FigureType::ReligiousLeader),
            EventType::Earthquake | EventType::Flood | EventType::Volcano => Some(FigureType::Hero),
            EventType::ArtCreated | EventType::Festival => Some(FigureType::Artist),
            EventType::Exploration | EventType::Discovery => Some(FigureType::Explorer),
            EventType::Invention => Some(FigureType::Inventor),
            EventType::ReligiousReformation | EventType::ReligiousReveal => {
                Some(FigureType::ReligiousLeader)
            }
            EventType::ScholarlyWork => Some(FigureType::Scholar),
            EventType::Conquest => Some(FigureType::MilitaryLeader),
            EventType::Assassination => Some(FigureType::Villain),
            EventType::HeroicAct => Some(FigureType::Hero),
            _ => None,
        }
    }
}

// ============================================================================
// Lifecycle States
// ============================================================================

/// Lifecycle states for notable figures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FigureLifecycleState {
    /// Currently active (living ruler, current general)
    Active,
    /// Had significant influence but not currently active
    Retired,
    /// Historical figure (deceased, recorded history)
    Historical,
    /// Mythologized figure (oral tradition, legends)
    Legendary,
    /// Figure only known through archaeological evidence
    ArchaeologicallyKnown,
}

impl FigureLifecycleState {
    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            FigureLifecycleState::Active => "Active",
            FigureLifecycleState::Retired => "Retired",
            FigureLifecycleState::Historical => "Historical",
            FigureLifecycleState::Legendary => "Legendary",
            FigureLifecycleState::ArchaeologicallyKnown => "Archaeologically Known",
        }
    }

    /// Get API visibility level (higher = more prominent)
    pub fn visibility_level(&self) -> u8 {
        match self {
            FigureLifecycleState::Active => 5,
            FigureLifecycleState::Legendary => 4,
            FigureLifecycleState::Historical => 3,
            FigureLifecycleState::Retired => 2,
            FigureLifecycleState::ArchaeologicallyKnown => 1,
        }
    }

    /// Get influence multiplier for this state
    pub fn influence_multiplier(&self) -> f32 {
        match self {
            FigureLifecycleState::Active => 1.2,
            FigureLifecycleState::Legendary => 1.5,
            FigureLifecycleState::Historical => 1.0,
            FigureLifecycleState::Retired => 0.8,
            FigureLifecycleState::ArchaeologicallyKnown => 0.5,
        }
    }
}

impl Default for FigureLifecycleState {
    fn default() -> Self {
        FigureLifecycleState::Active
    }
}

// ============================================================================
// Relationship Types
// ============================================================================

/// Typed relationship between figures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureRelationship {
    /// Target figure ID
    pub target_figure_id: Uuid,
    /// Type of relationship
    pub relationship_type: FigureRelationshipType,
    /// When the relationship started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_year: Option<i32>,
    /// When the relationship ended (None = ongoing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i32>,
    /// Whether this is bidirectional (e.g., Parent implies Child on target)
    #[serde(default)]
    pub bidirectional: bool,
}

impl FigureRelationship {
    /// Create a new relationship
    pub fn new(target_figure_id: Uuid, relationship_type: FigureRelationshipType) -> Self {
        Self {
            target_figure_id,
            relationship_type,
            start_year: None,
            end_year: None,
            bidirectional: Self::is_symmetric(relationship_type),
        }
    }

    /// Create with time bounds
    pub fn with_years(mut self, start_year: Option<i32>, end_year: Option<i32>) -> Self {
        self.start_year = start_year;
        self.end_year = end_year;
        self
    }

    /// Check if relationship type is symmetric
    fn is_symmetric(rel_type: FigureRelationshipType) -> bool {
        matches!(
            rel_type,
            FigureRelationshipType::Sibling
                | FigureRelationshipType::Rival
                | FigureRelationshipType::Ally
        )
    }

    /// Check if relationship is active at a given year
    pub fn is_active_at(&self, year: i32) -> bool {
        let after_start = self.start_year.map(|y| year >= y).unwrap_or(true);
        let before_end = self.end_year.map(|y| year < y).unwrap_or(true);
        after_start && before_end
    }
}

/// Types of relationships between figures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FigureRelationshipType {
    /// Parent-child relationship (bidirectional)
    Parent,
    /// Parent-child relationship (bidirectional)
    Child,
    /// Sibling relationship (bidirectional)
    Sibling,
    /// Spouse/marriage relationship (bidirectional)
    Spouse,
    /// Rivalry/enmity (bidirectional)
    Rival,
    /// Alliance/friendship (bidirectional)
    Ally,
    /// Teacher-student relationship
    Mentor,
    /// Teacher-student relationship
    Apprentice,
    /// Succession (successor to throne)
    Successor,
    /// Succession (predecessor on throne)
    Predecessor,
}

impl FigureRelationshipType {
    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            FigureRelationshipType::Parent => "Parent",
            FigureRelationshipType::Child => "Child",
            FigureRelationshipType::Sibling => "Sibling",
            FigureRelationshipType::Spouse => "Spouse",
            FigureRelationshipType::Rival => "Rival",
            FigureRelationshipType::Ally => "Ally",
            FigureRelationshipType::Mentor => "Mentor",
            FigureRelationshipType::Apprentice => "Apprentice",
            FigureRelationshipType::Successor => "Successor",
            FigureRelationshipType::Predecessor => "Predecessor",
        }
    }

    /// Get the inverse relationship type
    pub fn inverse(&self) -> Self {
        match self {
            FigureRelationshipType::Parent => FigureRelationshipType::Child,
            FigureRelationshipType::Child => FigureRelationshipType::Parent,
            FigureRelationshipType::Sibling => FigureRelationshipType::Sibling,
            FigureRelationshipType::Spouse => FigureRelationshipType::Spouse,
            FigureRelationshipType::Rival => FigureRelationshipType::Rival,
            FigureRelationshipType::Ally => FigureRelationshipType::Ally,
            FigureRelationshipType::Mentor => FigureRelationshipType::Apprentice,
            FigureRelationshipType::Apprentice => FigureRelationshipType::Mentor,
            FigureRelationshipType::Successor => FigureRelationshipType::Predecessor,
            FigureRelationshipType::Predecessor => FigureRelationshipType::Successor,
        }
    }
}

// ============================================================================
// Dynasty
// ============================================================================

/// Dynasty - a family line of rulers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynasty {
    /// Unique identifier
    pub id: Uuid,
    /// Dynasty name (e.g., "House Pendragon", "The Habsburgs")
    pub name: String,
    /// Founder of the dynasty
    pub founder_id: Uuid,
    /// Current head of the dynasty (living monarch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_head_id: Option<Uuid>,
    /// All figures in this dynasty
    pub member_ids: Vec<Uuid>,
    /// Coat of arms / heraldic symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coat_of_arms: Option<String>,
    /// House motto/saying
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motto: Option<String>,
    /// When dynasty was founded
    pub start_year: i32,
    /// When dynasty ended (None = ongoing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_year: Option<i32>,
    /// Realm/nation they ruled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<Uuid>,
    /// Timestamp
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Dynasty {
    /// Create a new dynasty with a founder figure
    pub fn new(name: String, founder_id: Uuid, start_year: i32) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4(),
            name,
            founder_id,
            current_head_id: Some(founder_id),
            member_ids: vec![founder_id],
            coat_of_arms: None,
            motto: None,
            start_year,
            end_year: None,
            realm_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a member to the dynasty
    pub fn add_member(&mut self, figure_id: Uuid) {
        if !self.member_ids.contains(&figure_id) {
            self.member_ids.push(figure_id);
            self.updated_at = Timestamp::now();
        }
    }

    /// Check if dynasty is active (no end year)
    pub fn is_active(&self) -> bool {
        self.end_year.is_none()
    }

    /// Get duration in years
    pub fn duration(&self, current_year: i32) -> i32 {
        let end = self.end_year.unwrap_or(current_year);
        end - self.start_year
    }

    /// End the dynasty at a given year
    pub fn end(&mut self, end_year: i32) {
        self.end_year = Some(end_year);
        self.current_head_id = None;
        self.updated_at = Timestamp::now();
    }

    /// Set the current head of the dynasty
    pub fn set_current_head(&mut self, figure_id: Uuid) {
        self.current_head_id = Some(figure_id);
        self.updated_at = Timestamp::now();
    }
}

// ============================================================================
// Dynasty Store
// ============================================================================

/// In-memory storage for dynasties
#[derive(Debug, Clone, Default)]
pub struct DynastyStore {
    dynasties: std::collections::HashMap<Uuid, Dynasty>,
}

impl DynastyStore {
    /// Create a new empty dynasty store
    pub fn new() -> Self {
        Self {
            dynasties: std::collections::HashMap::new(),
        }
    }

    /// Add a dynasty to the store
    pub fn add(&mut self, dynasty: Dynasty) {
        self.dynasties.insert(dynasty.id, dynasty);
    }

    /// Get a dynasty by ID
    pub fn get(&self, id: &Uuid) -> Option<&Dynasty> {
        self.dynasties.get(id)
    }

    /// Get a mutable dynasty by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Dynasty> {
        self.dynasties.get_mut(id)
    }

    /// Get all dynasties for a realm
    pub fn get_by_realm(&self, realm_id: &Uuid) -> Vec<&Dynasty> {
        self.dynasties
            .values()
            .filter(|d| d.realm_id == Some(*realm_id))
            .collect()
    }

    /// Get the current active dynasty for a realm
    pub fn get_active_for_realm(&self, realm_id: &Uuid) -> Option<&Dynasty> {
        self.get_by_realm(realm_id)
            .into_iter()
            .find(|d| d.is_active())
    }

    /// Get dynasty by founder
    pub fn get_by_founder(&self, founder_id: &Uuid) -> Option<&Dynasty> {
        self.dynasties
            .values()
            .find(|d| d.founder_id == *founder_id)
    }

    /// Get dynasty containing a figure
    pub fn get_for_figure(&self, figure_id: &Uuid) -> Option<&Dynasty> {
        self.dynasties
            .values()
            .find(|d| d.member_ids.contains(figure_id))
    }

    /// Get all active dynasties
    pub fn get_active(&self) -> Vec<&Dynasty> {
        self.dynasties.values().filter(|d| d.is_active()).collect()
    }

    /// Get total count
    pub fn count(&self) -> usize {
        self.dynasties.len()
    }

    /// List all dynasties
    pub fn dynasties(&self) -> impl Iterator<Item = &Dynasty> {
        self.dynasties.values()
    }
}

// ============================================================================
// Region Influence
// ============================================================================

/// Influence of a figure on a region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfluence {
    /// Figure exerting influence
    pub figure_id: Uuid,
    /// Region being influenced
    pub region_id: Uuid,
    /// Influence score (0.0 - 1.0)
    pub influence_score: f32,
    /// Whether this is the figure's primary region
    #[serde(default)]
    pub primary_region: bool,
}

impl RegionInfluence {
    /// Create a new region influence
    pub fn new(
        figure_id: Uuid,
        region_id: Uuid,
        influence_score: f32,
        primary_region: bool,
    ) -> Self {
        Self {
            figure_id,
            region_id,
            influence_score: influence_score.clamp(0.0, 1.0),
            primary_region,
        }
    }
}

// ============================================================================
// Relationship Graph
// ============================================================================

/// Relationship graph for figures
#[derive(Debug, Clone, Default)]
pub struct FigureRelationshipGraph {
    /// Adjacency list: figure_id -> relationships
    edges: std::collections::HashMap<Uuid, Vec<FigureRelationship>>,
    /// Index for quick lookup by type
    by_type: std::collections::HashMap<(Uuid, FigureRelationshipType), Vec<Uuid>>,
}

impl FigureRelationshipGraph {
    /// Create a new empty relationship graph
    pub fn new() -> Self {
        Self {
            edges: std::collections::HashMap::new(),
            by_type: std::collections::HashMap::new(),
        }
    }

    /// Add a relationship between two figures
    pub fn add_relationship(&mut self, figure_id: Uuid, relationship: FigureRelationship) {
        // Add to adjacency list
        self.edges
            .entry(figure_id)
            .or_insert_with(Vec::new)
            .push(relationship.clone());

        // Update type index
        let key = (figure_id, relationship.relationship_type);
        self.by_type
            .entry(key)
            .or_insert_with(Vec::new)
            .push(relationship.target_figure_id);

        // If bidirectional, add inverse relationship
        if relationship.bidirectional {
            let inverse = FigureRelationship {
                target_figure_id: figure_id,
                relationship_type: relationship.relationship_type.inverse(),
                start_year: relationship.start_year,
                end_year: relationship.end_year,
                bidirectional: true,
            };

            self.edges
                .entry(relationship.target_figure_id)
                .or_insert_with(Vec::new)
                .push(inverse);
        }
    }

    /// Get all relationships for a figure
    pub fn get_relationships(&self, figure_id: &Uuid) -> Vec<FigureRelationship> {
        self.edges
            .get(figure_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
            .to_vec()
    }

    /// Get relationships of a specific type
    pub fn get_by_type(
        &self,
        figure_id: &Uuid,
        relationship_type: FigureRelationshipType,
    ) -> Vec<Uuid> {
        self.by_type
            .get(&(*figure_id, relationship_type))
            .cloned()
            .unwrap_or_default()
    }

    /// Get figure IDs related to this figure
    pub fn get_related_figure_ids(&self, figure_id: &Uuid) -> Vec<Uuid> {
        self.get_relationships(figure_id)
            .iter()
            .map(|r| r.target_figure_id)
            .collect()
    }

    /// Check if two figures are related
    pub fn are_related(&self, figure_id_1: &Uuid, figure_id_2: &Uuid) -> bool {
        self.edges
            .get(figure_id_1)
            .map(|rels| rels.iter().any(|r| r.target_figure_id == *figure_id_2))
            .unwrap_or(false)
    }

    /// Get family members (parent/child/sibling)
    pub fn get_family(&self, figure_id: &Uuid) -> Vec<Uuid> {
        let mut family = Vec::new();
        for rel_type in [
            FigureRelationshipType::Parent,
            FigureRelationshipType::Child,
            FigureRelationshipType::Sibling,
        ] {
            family.extend(self.get_by_type(figure_id, rel_type));
        }
        family
    }

    /// Get rivals
    pub fn get_rivals(&self, figure_id: &Uuid) -> Vec<Uuid> {
        self.get_by_type(figure_id, FigureRelationshipType::Rival)
    }

    /// Get allies
    pub fn get_allies(&self, figure_id: &Uuid) -> Vec<Uuid> {
        self.get_by_type(figure_id, FigureRelationshipType::Ally)
    }

    /// Get dynasty ancestors (parent chains going back)
    pub fn get_ancestors(&self, figure_id: &Uuid, max_depth: usize) -> Vec<Uuid> {
        let mut ancestors = Vec::new();
        let mut to_visit = vec![*figure_id];
        let mut visited = std::collections::HashSet::new();

        for _ in 0..max_depth {
            let current = to_visit.clone();
            to_visit.clear();

            for id in current {
                if visited.contains(&id) {
                    continue;
                }
                visited.insert(id);

                for parent_id in self.get_by_type(&id, FigureRelationshipType::Parent) {
                    if !ancestors.contains(&parent_id) {
                        ancestors.push(parent_id);
                        to_visit.push(parent_id);
                    }
                }
            }
        }

        ancestors
    }

    /// Get dynasty descendants (child chains going forward)
    pub fn get_descendants(&self, figure_id: &Uuid, max_depth: usize) -> Vec<Uuid> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![*figure_id];
        let mut visited = std::collections::HashSet::new();

        for _ in 0..max_depth {
            let current = to_visit.clone();
            to_visit.clear();

            for id in current {
                if visited.contains(&id) {
                    continue;
                }
                visited.insert(id);

                for child_id in self.get_by_type(&id, FigureRelationshipType::Child) {
                    if !descendants.contains(&child_id) {
                        descendants.push(child_id);
                        to_visit.push(child_id);
                    }
                }
            }
        }

        descendants
    }
}

/// Person name with optional components for figures
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FigureName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epithet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl FigureName {
    pub fn new(given: Option<String>, family: Option<String>) -> Self {
        Self {
            given,
            family,
            epithet: None,
            title: None,
        }
    }

    pub fn with_epithet(mut self, epithet: String) -> Self {
        self.epithet = Some(epithet);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Format name as a full string
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        if let Some(title) = &self.title {
            parts.push(title.clone());
        }
        if let Some(given) = &self.given {
            parts.push(given.clone());
        }
        if let Some(family) = &self.family {
            parts.push(family.clone());
        }
        if let Some(epithet) = &self.epithet {
            parts.push(epithet.clone());
        }
        parts.join(" ")
    }
}

// ============================================================================
// Figure Name Generator
// ============================================================================

/// Generator for procedural figure names with honorifics.
/// Extends the settlement naming pattern for personal names.
#[derive(Debug, Clone)]
pub struct FigureNameGenerator {
    /// RNG seed for deterministic generation
    seed: u64,
}

impl FigureNameGenerator {
    /// Create a new figure name generator.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a random figure name.
    /// Combines syllables into 1-3 syllable personal names.
    pub fn generate_name(&mut self, figure_type: FigureType) -> FigureName {
        // First syllable pool (consonant-vowel opening)
        let first = [
            "Al", "An", "Ar", "Bel", "Ca", "Da", "El", "Fal", "Ga", "Hal", "Ka", "La", "Ma", "Na",
            "Or", "Pa", "Ra", "Sa", "Ta", "Ul", "Var", "Wil", "Xan", "Zan", "Keth", "Mor", "Sal",
            "Theo", "Fen", "Gor",
        ];

        // Middle syllables (vowel-heavy)
        let middle = [
            "ian", "eon", "ara", "ina", "ena", "ali", "ori", "eli", "alo", "iro", "eth", "ath",
            "ith", "oth", "uth", "ax", "ex", "ix", "ox", "ux",
        ];

        // Final syllables (consonant-ending for polish)
        let last = [
            "os", "us", "is", "as", "es", "al", "el", "il", "ol", "an", "en", "in", "on", "ar",
            "ir", "or", "ius", "ian", "eon", "ax",
        ];

        // Simple hash-based RNG for simplicity
        let mut hash = self.seed.wrapping_mul(31).wrapping_add(17);

        // Determine name length (1-3 syllables based on figure type)
        let syllables = match figure_type {
            FigureType::Legendary => 3,                     // Epic names
            FigureType::Monarch => 2 + (hash % 2) as usize, // Regal names
            FigureType::Hero => 2 + (hash % 2) as usize,    // Heroic names
            _ => 1 + (hash % 3) as usize,                   // 1-3 syllables
        };

        hash = hash.rotate_left(5);
        let first_idx = (hash as usize) % first.len();

        let given = first[first_idx].to_string();

        let mut family = String::new();
        if syllables >= 2 {
            hash = hash.rotate_left(3);
            let mid_idx = (hash as usize) % middle.len();
            family.push_str(middle[mid_idx]);
        }
        if syllables >= 3 {
            hash = hash.rotate_left(7);
            let last_idx = (hash as usize) % last.len();
            family.push_str(last[last_idx]);
        }

        // Update seed for next call
        self.seed = hash;

        FigureName::new(
            Some(given),
            if syllables >= 2 { Some(family) } else { None },
        )
    }

    /// Generate a random honorific/epithet.
    /// These are the "the Bold", "Ironhand" style descriptors.
    pub fn generate_honorific(&mut self, figure_type: FigureType) -> String {
        // Strength/power honorifics
        let strength = [
            "the Bold",
            "the Brave",
            "the Strong",
            "the Mighty",
            "the Powerful",
            "Ironhand",
            "Ironfist",
            "Steelshield",
            "Stoneheart",
            "Ironwill",
            "the Fierce",
            "the Terrible",
            "the Conqueror",
            "the Unbroken",
            "the Unstoppable",
        ];

        // Wisdom/knowledge honorifics
        let wisdom = [
            "the Wise",
            "the Sage",
            "the Scholar",
            "the Learned",
            "the Enlightened",
            "the Seeker",
            "the Oracle",
            "the Visionary",
            "the Mindful",
            "the Thoughtful",
        ];

        // Cunning honorifics
        let cunning = [
            "the Swift",
            "the Quick",
            "the Agile",
            "the Shadow",
            "the Silent",
            "the Fox",
            "the Serpent",
            "the Trickster",
            "the Deceiver",
            "the Masked",
        ];

        // Piety/divine honorifics
        let piety = [
            "the Devout",
            "the Blessed",
            "the Chosen",
            "the Holy",
            "the Sacred",
            "Voice of the Gods",
            "the Illuminated",
            "the Pure",
            "the Faithful",
            "the Saintly",
        ];

        // Arts/beauty honorifics
        let arts = [
            "the Beautiful",
            "the Graceful",
            "the Harmonious",
            "the Melodic",
            "the Inspired",
            "the Poet",
            "the Artful",
            "the Creative",
            "the Visionary",
            "the Talented",
        ];

        // Exploration honorifics
        let exploration = [
            "the Explorer",
            "the Bold",
            "the Trailblazer",
            "the Discoverer",
            "the Pioneer",
            "the Wanderer",
            "the Voyager",
            "the Pathfinder",
            "the Seeker",
            "the Adventurer",
        ];

        // Legendary/epic honorifics
        let legendary = [
            "the Legendary",
            "the Immortal",
            "the Eternal",
            "the Undying",
            "the Mythic",
            "Dragonborn",
            "Starshaper",
            "the Legend",
            "the Myth",
            "the Immortal One",
        ];

        let pool: &[&str] = match figure_type {
            FigureType::MilitaryLeader | FigureType::Hero => &strength,
            FigureType::Scholar => &wisdom,
            FigureType::Explorer => &exploration,
            FigureType::ReligiousLeader => &piety,
            FigureType::Artist => &arts,
            FigureType::Legendary => &legendary,
            FigureType::Inventor => &cunning,
            FigureType::FolkHero => &strength,
            _ => &strength,
        };

        let hash = self.seed.wrapping_mul(13).wrapping_add(7);
        let idx = (hash as usize) % pool.len();
        self.seed = hash.rotate_left(4);
        pool[idx].to_string()
    }

    /// Generate a title based on figure type and achievements.
    pub fn generate_title(&mut self, figure_type: FigureType, significance: f32) -> Option<String> {
        let titles = match figure_type {
            FigureType::Monarch => vec![
                "King",
                "Queen",
                "Emperor",
                "Empress",
                "Duke",
                "Duchess",
                "Lord",
                "Lady",
                "Prince",
                "Princess",
                "Ruler",
                "Sovereign",
                "Warlord", // If military
            ],
            FigureType::MilitaryLeader => vec![
                "General",
                "Commander",
                "Marshal",
                "Captain",
                "Lord Commander",
                "Warlord",
                "Champion",
                "Supreme Commander",
                "High General",
            ],
            FigureType::Scholar => vec![
                "Sage",
                "Master",
                "Professor",
                "Archmage",
                "High Scholar",
                "Philosopher",
                "Chronicler",
                "Keeper of Lore",
            ],
            FigureType::ReligiousLeader => vec![
                "High Priest",
                "Pope",
                "Archbishop",
                "Prophet",
                "Oracle",
                "Bishop",
                "Elder",
                "Spiritual Guide",
                "Divine Voice",
            ],
            FigureType::Artist => vec![
                "Master",
                "Maestro",
                "Virtuoso",
                "Grand Artist",
                "Creative Genius",
            ],
            FigureType::Explorer => vec![
                "Explorer",
                "Pathfinder",
                "Voyager",
                "Navigator",
                "Chartmaker",
            ],
            FigureType::Inventor => vec![
                "Artificer",
                "Engineer",
                "Inventor",
                "Tinker",
                "Grand Artificer",
            ],
            FigureType::Hero => vec![
                "Champion",
                "Hero",
                "Champion of the Realm",
                "Sworn Protector",
            ],
            FigureType::Villain => vec!["Tyrant", "Despot", "Usurper", "the Fallen"],
            FigureType::FolkHero => vec!["Guardian", "Protector", "Folk Hero", "People's Champion"],
            FigureType::Legendary => vec!["Mythic Being", "Legend", "Immortal", "Myth"],
        };

        // Higher significance = more prestigious title
        let _idx = if significance > 0.8 {
            0 // Most prestigious
        } else if significance > 0.6 {
            1 + (self.seed % 2) as usize
        } else {
            2 + (self.seed as usize % (titles.len() - 2).max(1)) as usize
        };

        let hash = self.seed.wrapping_mul(17).wrapping_add(3);
        let title_idx = (hash as usize) % titles.len();
        self.seed = hash.rotate_left(6);

        titles.get(title_idx).cloned().map(|s| s.to_string())
    }
}

/// Notable figure - a person with historical significance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotableFigure {
    /// Unique identifier
    pub id: EntityId,

    /// World this figure belongs to
    pub world_id: Uuid,

    /// Person name components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<FigureName>,

    /// Type/category of figure
    pub figure_type: FigureType,

    /// Era or period name (e.g., "The Age of Kings", "The Dark Century")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub era: Option<String>,

    /// Birth year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_year: Option<i32>,

    /// Death year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_year: Option<i32>,

    /// Birthplace settlement ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthplace_id: Option<Uuid>,

    /// Primary culture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,

    /// Titles held (King, General, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub titles: Option<Vec<String>>,

    /// One-line description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Full biography text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,

    /// Major accomplishments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accomplishments: Option<Vec<String>>,

    /// Historical significance (0.0 to 1.0)
    pub significance: f32,

    /// Related event IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_events: Option<Vec<Uuid>>,

    /// Related figure IDs (allies, rivals, family)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_figures: Option<Vec<Uuid>>,

    /// Typed relationships (supersedes related_figures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Vec<FigureRelationship>>,

    /// Lifecycle state
    #[serde(default)]
    pub lifecycle_state: FigureLifecycleState,

    /// Dynasty this figure belongs to (monarchs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynasty_id: Option<Uuid>,

    /// Geographic influence radius (km)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influence_radius: Option<f32>,

    /// Species ID if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub species_id: Option<Uuid>,

    /// Region of influence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influence_region_id: Option<Uuid>,

    /// Timestamp
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl NotableFigure {
    /// Create a new notable figure
    pub fn new(world_id: Uuid, figure_type: FigureType, significance: f32) -> Self {
        let now = Timestamp::now();
        Self {
            id: EntityId::new(EntityType::Person), // Use Person type for figures
            world_id,
            name: None,
            figure_type,
            era: None,
            birth_year: None,
            death_year: None,
            birthplace_id: None,
            culture: None,
            titles: None,
            description: None,
            biography: None,
            accomplishments: None,
            significance: significance.clamp(0.0, 1.0),
            related_events: None,
            related_figures: None,
            relationships: None,
            lifecycle_state: FigureLifecycleState::default(),
            dynasty_id: None,
            influence_radius: None,
            species_id: None,
            influence_region_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a related event
    pub fn add_event(&mut self, event_id: Uuid) {
        match &mut self.related_events {
            Some(events) => {
                if !events.contains(&event_id) {
                    events.push(event_id);
                }
            }
            None => self.related_events = Some(vec![event_id]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Add a related figure
    pub fn add_related_figure(&mut self, figure_id: Uuid) {
        match &mut self.related_figures {
            Some(figures) => {
                if !figures.contains(&figure_id) {
                    figures.push(figure_id);
                }
            }
            None => self.related_figures = Some(vec![figure_id]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Add an accomplishment
    pub fn add_accomplishment(&mut self, accomplishment: String) {
        match &mut self.accomplishments {
            Some(accomplishments) => accomplishments.push(accomplishment),
            None => self.accomplishments = Some(vec![accomplishment]),
        }
        self.updated_at = Timestamp::now();
    }

    /// Get lifespan in years, if birth/death known
    pub fn lifespan(&self) -> Option<i32> {
        match (self.birth_year, self.death_year) {
            (Some(b), Some(d)) if d > b => Some(d - b),
            _ => None,
        }
    }
}

// ============================================================================
// Figure Store
// ============================================================================

/// In-memory storage for notable figures
#[derive(Debug, Clone, Default)]
pub struct FigureStore {
    figures: std::collections::HashMap<Uuid, NotableFigure>,
}

impl FigureStore {
    pub fn new() -> Self {
        Self {
            figures: std::collections::HashMap::new(),
        }
    }

    /// Add a figure to the store
    pub fn add(&mut self, figure: NotableFigure) {
        self.figures.insert(figure.id.to_uuid(), figure);
    }

    /// Get a figure by ID
    pub fn get(&self, id: &Uuid) -> Option<&NotableFigure> {
        self.figures.get(id)
    }

    /// Get a mutable figure by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut NotableFigure> {
        self.figures.get_mut(id)
    }

    /// Get all figures for a world
    pub fn get_by_world(&self, world_id: &Uuid) -> Vec<&NotableFigure> {
        self.figures
            .values()
            .filter(|f| f.world_id == *world_id)
            .collect()
    }

    /// Get figures filtered by type
    pub fn get_by_type(&self, world_id: &Uuid, figure_type: FigureType) -> Vec<&NotableFigure> {
        self.figures
            .values()
            .filter(|f| f.world_id == *world_id && f.figure_type == figure_type)
            .collect()
    }

    /// Get figures above a significance threshold
    pub fn get_by_significance(
        &self,
        world_id: &Uuid,
        min_significance: f32,
    ) -> Vec<&NotableFigure> {
        self.figures
            .values()
            .filter(|f| f.world_id == *world_id && f.significance >= min_significance)
            .collect()
    }

    /// Get figures by region
    pub fn get_by_region(&self, world_id: &Uuid, region_id: &Uuid) -> Vec<&NotableFigure> {
        self.figures
            .values()
            .filter(|f| f.world_id == *world_id && f.influence_region_id == Some(*region_id))
            .collect()
    }

    /// Get total count for a world
    pub fn count(&self, world_id: &Uuid) -> usize {
        self.figures
            .values()
            .filter(|f| f.world_id == *world_id)
            .count()
    }

    /// List all figures (for iteration)
    pub fn figures(&self) -> impl Iterator<Item = &NotableFigure> {
        self.figures.values()
    }
}

// ============================================================================
// Figure Generator
// ============================================================================

/// Configuration for figure generation
#[derive(Debug, Clone)]
pub struct FigureGeneratorConfig {
    /// Maximum figures per world
    pub max_figures: u32,
    /// Ratio of figures that are "notable" (significance > 0.5)
    pub notable_ratio: f32,
    /// Average lifespan in years
    pub average_lifespan: f32,
    /// Base significance for generated figures
    pub base_significance: f32,
}

impl Default for FigureGeneratorConfig {
    fn default() -> Self {
        Self {
            max_figures: 500,
            notable_ratio: 0.1, // 10% are highly notable
            average_lifespan: 70.0,
            base_significance: 0.3,
        }
    }
}

/// Figure generator that creates notable figures from events
#[derive(Debug, Clone)]
pub struct FigureGenerator {
    config: FigureGeneratorConfig,
}

impl FigureGenerator {
    pub fn new(config: FigureGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate figures from a set of significant events
    pub fn generate_from_events(
        &self,
        world_id: Uuid,
        events: &[Event],
        settlement_ids: &[Uuid],
        cultures: &[String],
        rng: &mut Rng,
    ) -> Vec<NotableFigure> {
        let mut figures = Vec::new();

        // Filter significant events (significance >= 0.5)
        let significant_events: Vec<_> = events
            .iter()
            .filter(|e| e.significance.unwrap_or(0.0) >= 0.5)
            .collect();

        for event in significant_events {
            // Skip if too many figures
            if figures.len() >= self.config.max_figures as usize {
                break;
            }

            // Determine figure type from event
            let Some(figure_type) = FigureType::from_event(&event.event_type) else {
                continue;
            };

            // Generate significance based on event significance
            let base_sig = event.significance.unwrap_or(self.config.base_significance);
            let significance = self.generate_significance(base_sig, rng);

            // Only generate notable figures above threshold
            if significance < 0.5 {
                continue;
            }

            let mut figure = NotableFigure::new(world_id, figure_type, significance);

            // Set birth/death years based on event time
            let event_year = event.time.get_year();
            let lifespan = self.generate_lifespan(figure_type, rng);

            // Figures born before event (they participate in it)
            figure.birth_year = Some(event_year - (lifespan * 0.6) as i32);
            figure.death_year = Some(event_year + ((lifespan * 0.4) as i32).max(1));

            // Link to event
            figure.add_event(event.id.to_uuid());

            // Add description from event
            figure.description = Some(event.description.clone());

            // Assign culture if available
            if !cultures.is_empty() {
                figure.culture = Some(cultures[rng.random_usize() % cultures.len()].clone());
            }

            // Assign birthplace if settlements available
            if !settlement_ids.is_empty() {
                figure.birthplace_id =
                    Some(settlement_ids[rng.random_usize() % settlement_ids.len()]);
            }

            // Generate accomplishments based on figure type
            self.generate_accomplishments(&mut figure, &event, rng);

            figures.push(figure);
        }

        figures
    }

    /// Generate a random notable figure
    pub fn generate_random(&self, world_id: Uuid, year: i32, rng: &mut Rng) -> NotableFigure {
        let figure_types = [
            FigureType::Monarch,
            FigureType::MilitaryLeader,
            FigureType::Scholar,
            FigureType::Artist,
            FigureType::ReligiousLeader,
        ];

        let figure_type = figure_types[rng.random_usize() % figure_types.len()];
        let lifespan = self.generate_lifespan(figure_type, rng);
        let significance = self.generate_significance(self.config.base_significance, rng);

        let mut figure = NotableFigure::new(world_id, figure_type, significance);
        figure.birth_year = Some(year - (lifespan * 0.6) as i32);
        figure.death_year = Some(year + ((lifespan * 0.4) as i32).max(1));

        figure
    }

    /// Generate lifespan based on figure type
    fn generate_lifespan(&self, figure_type: FigureType, rng: &mut Rng) -> f32 {
        let base = self.config.average_lifespan;
        let modifier = figure_type.lifespan_modifier();

        // Add some randomness (+/- 20%)
        let variance = rng.random_f32() * 0.4 - 0.2;
        (base * modifier * (1.0 + variance)).max(20.0)
    }

    /// Generate significance using power-law distribution
    /// Most figures are less notable, few are legendary
    fn generate_significance(&self, base: f32, rng: &mut Rng) -> f32 {
        // Power-law: sqrt(random) gives distribution favoring lower values
        let power = rng.random_f32().sqrt();
        let significance = base + (1.0 - base) * (1.0 - power);
        significance.clamp(0.0, 1.0)
    }

    /// Generate accomplishments for a figure based on their type
    fn generate_accomplishments(&self, figure: &mut NotableFigure, event: &Event, rng: &mut Rng) {
        let accomplishments = match figure.figure_type {
            FigureType::Monarch => vec![
                format!("Ruled during the {}", event.name),
                "Expanded the kingdom's borders".to_string(),
                "Established a new dynasty".to_string(),
            ],
            FigureType::MilitaryLeader => vec![
                format!("Led the forces during {}", event.name),
                "Never lost a major battle".to_string(),
                "Conquered neighboring territories".to_string(),
            ],
            FigureType::Scholar => vec![
                format!("Authored the famous treatise on {}", event.name),
                "Founded a school of thought".to_string(),
                "Mentored the next generation".to_string(),
            ],
            FigureType::Artist => vec![
                format!("Created masterworks inspired by {}", event.name),
                "Revolutionized the art form".to_string(),
            ],
            FigureType::ReligiousLeader => vec![
                format!("Led the faithful through {}", event.name),
                "Established new religious practices".to_string(),
            ],
            FigureType::Explorer => vec![
                format!("Discovered new lands during {}", event.name),
                "Mapped previously unknown territories".to_string(),
            ],
            FigureType::Inventor => vec![
                format!("Invented new technology during {}", event.name),
                "Revolutionized daily life".to_string(),
            ],
            FigureType::Hero => vec![
                format!("Saved the realm during {}", event.name),
                "Became a symbol of courage".to_string(),
            ],
            FigureType::Villain => vec![
                format!("Perpetrated {}", event.name),
                "Brought terror to the land".to_string(),
            ],
            FigureType::FolkHero => vec![
                format!("Legend of {}", event.name),
                "Stories told for generations".to_string(),
            ],
            FigureType::Legendary => vec![
                format!("Figured in the myths of {}", event.name),
                "Said to possess supernatural powers".to_string(),
            ],
        };

        // Select 1-3 random accomplishments
        let count = 1 + rng.random_usize() % 3;
        for i in 0..count.min(accomplishments.len()) {
            figure.add_accomplishment(accomplishments[i].clone());
        }
    }

    /// Generate a biography for a figure based on their deeds and context.
    /// Collects all related event descriptions and generates a narrative biography.
    pub fn generate_biography(
        &self,
        figure: &mut NotableFigure,
        _world_id: Uuid,
        world_name: &str,
        era_name: &str,
        society_name: Option<&str>,
    ) {
        // Collect deed descriptions from related events
        let deeds: Vec<String> = figure
            .related_events
            .as_ref()
            .map(|events| {
                events
                    .iter()
                    .take(5) // Limit to top 5 deeds
                    .map(|_| {
                        // In a real implementation, we would look up events
                        // For now, use accomplishments
                        figure
                            .accomplishments
                            .as_ref()
                            .and_then(|a| a.first())
                            .cloned()
                            .unwrap_or_else(|| "achieved great deeds".to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build deed summary
        let deed_summary = if deeds.is_empty() {
            "Known for their impact on the era.".to_string()
        } else if deeds.len() == 1 {
            format!("Remembered for: {}.", deeds[0])
        } else {
            let last_deed = deeds.last().cloned().unwrap_or_default();
            let other_deeds: Vec<&str> = deeds
                .iter()
                .take(deeds.len() - 1)
                .map(|s| s.as_str())
                .collect();
            format!(
                "Remembered for: {} and {}.",
                other_deeds.join(", "),
                last_deed
            )
        };
        let legacy = match figure.significance {
            s if s >= 0.9 => "Their legacy shaped the course of history for centuries.",
            s if s >= 0.7 => "Their deeds are still celebrated in song and story.",
            s if s >= 0.5 => "They are remembered as a significant figure of their era.",
            _ => "Their contributions, while modest, left their mark on history.",
        };

        // Get the title or figure type label
        let title = figure
            .titles
            .as_ref()
            .and_then(|t| t.first())
            .cloned()
            .unwrap_or_else(|| figure.figure_type.label().to_string());

        // Format birth/death years
        let life_years = match (figure.birth_year, figure.death_year) {
            (Some(b), Some(d)) => format!("({} - {})", b, d),
            (Some(b), None) => format!("(c. {} - present)", b),
            _ => String::new(),
        };

        // Build society reference
        let society_ref = society_name
            .map(|s| format!(" of {}", s))
            .unwrap_or_default();

        // Compose the full biography
        let name = figure
            .name
            .as_ref()
            .map(|n| n.full_name())
            .unwrap_or_else(|| "An Unknown Figure".to_string());

        figure.biography = Some(format!(
            "{} {} was a {} of {}{} during the {} era in {}. {} {}",
            name,
            life_years,
            title,
            era_name,
            society_ref,
            era_name,
            world_name,
            deed_summary,
            legacy
        ));
    }

    /// Update figure lifecycle state based on year and death probability.
    /// Called per year during history simulation.
    pub fn update_lifecycle(
        &self,
        figure: &mut NotableFigure,
        current_year: i32,
        years_simulated: i32,
        base_death_probability: f32,
        era_name: &str,
    ) -> bool {
        // Track if figure died this update
        let mut died = false;

        // Check if figure should die based on lifespan
        if let Some(death_year) = figure.death_year {
            if current_year >= death_year {
                died = true;
            }
        } else {
            // Calculate probability of death based on age
            let age = current_year - figure.birth_year.unwrap_or(current_year);
            if age > 0 {
                // Base death probability scaled by figure type
                let type_risk = match figure.figure_type {
                    FigureType::MilitaryLeader => 1.5, // Higher death risk
                    FigureType::Explorer => 1.4,
                    FigureType::Hero => 1.3,
                    FigureType::Monarch => 0.8, // Lower risk (better care)
                    FigureType::Scholar => 0.9,
                    FigureType::ReligiousLeader => 0.9,
                    FigureType::Legendary => 0.3, // Mythical longevity
                    _ => 1.0,
                };

                // Age-based probability curve (increases sharply after 60)
                let age_factor = if age < 30 {
                    base_death_probability * 0.2
                } else if age < 50 {
                    base_death_probability
                } else if age < 70 {
                    base_death_probability * 2.0
                } else {
                    base_death_probability * 5.0
                };

                let adjusted_prob = age_factor * type_risk;

                // Simple RNG check (in real impl would use proper RNG)
                if adjusted_prob > 0.8 || age > 100 {
                    figure.death_year = Some(current_year);
                    died = true;
                }
            }
        }

        // Update lifecycle state based on situation
        if died {
            // Transition to Historical or Legendary
            figure.lifecycle_state = if years_simulated > 100 {
                // After 100 years, becomes legendary
                FigureLifecycleState::Legendary
            } else {
                FigureLifecycleState::Historical
            };
            figure.era = Some(era_name.to_string());
        } else if figure.lifecycle_state == FigureLifecycleState::Active {
            // Active figures stay active while alive
            figure.era = Some(era_name.to_string());
        }

        died
    }

    /// Calculate impact score for a figure.
    /// Combines significance with event count and relationship network size.
    pub fn calculate_impact_score(&self, figure: &NotableFigure) -> f32 {
        let base = figure.significance;

        // Event participation multiplier
        let event_count = figure.related_events.as_ref().map(|e| e.len()).unwrap_or(0);
        let event_factor = 1.0 + (event_count as f32 * 0.05).min(0.5);

        // Relationship network factor
        let rel_count = figure.relationships.as_ref().map(|r| r.len()).unwrap_or(0);
        let rel_factor = 1.0 + (rel_count as f32 * 0.02).min(0.3);

        // Lifecycle state modifier
        let state_modifier = figure.lifecycle_state.influence_multiplier();

        // Combine factors
        let impact = base * event_factor * rel_factor * state_modifier;
        impact.min(1.0)
    }

    /// Propagate figure influence to adjacent regions.
    /// Returns a list of RegionInfluence with falloff based on distance.
    pub fn propagate_influence_to_regions(
        &self,
        figure: &NotableFigure,
        _primary_region_id: Uuid,
        adjacent_region_ids: &[(Uuid, f32)], // (region_id, distance_km)
        _world_radius_km: f32,               // Total world radius for normalization
    ) -> Vec<RegionInfluence> {
        let mut influences = Vec::new();

        // Primary region gets full influence
        if let Some(region_id) = figure.influence_region_id {
            influences.push(RegionInfluence::new(
                figure.id.to_uuid(),
                region_id,
                figure.significance * figure.lifecycle_state.influence_multiplier(),
                true, // primary
            ));
        }

        // Influence radius (default 100km if not set)
        let radius = figure.influence_radius.unwrap_or(100.0);

        // Add influence to adjacent regions with distance falloff
        for (region_id, distance) in adjacent_region_ids {
            if *distance <= radius {
                // Falloff: 1.0 at center, 0.0 at radius edge
                let falloff = 1.0 - (*distance / radius);
                let score =
                    figure.significance * falloff * figure.lifecycle_state.influence_multiplier();

                influences.push(RegionInfluence::new(
                    figure.id.to_uuid(),
                    *region_id,
                    score,
                    false, // not primary
                ));
            }
        }

        influences
    }

    /// Apply figure influence effects to settlement development.
    /// Updates settlement properties based on figure type and influence.
    pub fn apply_settlement_influence(
        &self,
        settlement_type_modifier: &mut Option<crate::types::SettlementType>,
        fortification_modifier: &mut f32,
        cultural_modifier: &mut f32,
        spiritual_modifier: &mut f32,
        figure: &NotableFigure,
        influence: f32,
    ) {
        let impact = influence * figure.lifecycle_state.influence_multiplier();

        match figure.figure_type {
            FigureType::Monarch => {
                // Monarchs can elevate settlement to city/capital
                if impact > 0.8 {
                    *settlement_type_modifier = Some(crate::types::SettlementType::City);
                } else if impact > 0.5 {
                    *settlement_type_modifier = Some(crate::types::SettlementType::Town);
                }
                *cultural_modifier += impact * 10.0;
            }
            FigureType::MilitaryLeader => {
                // Military leaders add fortifications
                *fortification_modifier += impact * 0.3;
            }
            FigureType::ReligiousLeader => {
                // Religious leaders add spiritual significance
                *spiritual_modifier += impact * 0.5;
                *cultural_modifier += impact * 5.0;
            }
            FigureType::Scholar => {
                // Scholars boost cultural/knowledge
                *cultural_modifier += impact * 15.0;
            }
            FigureType::Artist => {
                // Artists boost cultural score significantly
                *cultural_modifier += impact * 20.0;
            }
            FigureType::Explorer => {
                // Explorers boost trade routes
                *cultural_modifier += impact * 5.0;
            }
            FigureType::Inventor => {
                // Inventors boost technological advancement
                *cultural_modifier += impact * 10.0;
            }
            FigureType::Hero | FigureType::FolkHero => {
                // Heroes add reputation
                *cultural_modifier += impact * 8.0;
                *fortification_modifier += impact * 0.1;
            }
            FigureType::Villain => {
                // Villains can reduce settlement prosperity
                *cultural_modifier -= impact * 5.0;
            }
            FigureType::Legendary => {
                // Legendary figures have moderate global effect
                *cultural_modifier += impact * 25.0;
                *spiritual_modifier += impact * 0.3;
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventType;

    #[test]
    fn test_figure_type_from_event() {
        assert_eq!(
            FigureType::from_event(&EventType::Battle),
            Some(FigureType::MilitaryLeader)
        );
        assert_eq!(
            FigureType::from_event(&EventType::Invention),
            Some(FigureType::Inventor)
        );
        assert_eq!(
            FigureType::from_event(&EventType::ReligiousReformation),
            Some(FigureType::ReligiousLeader)
        );
    }

    #[test]
    fn test_figure_lifespan() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::Monarch, 0.8);
        figure.birth_year = Some(1000);
        figure.death_year = Some(1060);

        assert_eq!(figure.lifespan(), Some(60));
    }

    #[test]
    fn test_figure_store() {
        let mut store = FigureStore::new();
        let world_id = Uuid::new_v4();

        let figure = NotableFigure::new(world_id, FigureType::Hero, 0.9);
        let id = figure.id.to_uuid();
        store.add(figure);

        assert_eq!(store.count(&world_id), 1);
        assert!(store.get(&id).is_some());

        let by_type = store.get_by_type(&world_id, FigureType::Hero);
        assert_eq!(by_type.len(), 1);
    }

    #[test]
    fn test_significance_generation() {
        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);
        let mut rng = Rng::new(42);

        let mut total = 0.0;
        let iterations = 1000;

        for _ in 0..iterations {
            let sig = generator.generate_significance(0.3, &mut rng);
            total += sig;
        }

        let average = total / iterations as f32;
        // Average should be roughly in the 0.3-0.6 range
        assert!(average > 0.3 && average < 0.7);
    }

    #[test]
    fn test_figure_name_generator() {
        let mut generator = FigureNameGenerator::new(42);

        // Generate names for different figure types
        let name1 = generator.generate_name(FigureType::Monarch);
        assert!(name1.given.is_some());

        let name2 = generator.generate_name(FigureType::Hero);
        assert!(name2.given.is_some());

        // Names should be deterministic with same seed
        let mut generator2 = FigureNameGenerator::new(42);
        let name1_dup = generator2.generate_name(FigureType::Monarch);
        assert_eq!(name1.full_name(), name1_dup.full_name());
    }

    #[test]
    fn test_honorific_generation() {
        let mut generator = FigureNameGenerator::new(123);

        let honorific = generator.generate_honorific(FigureType::MilitaryLeader);
        assert!(!honorific.is_empty());
        assert!(
            honorific.starts_with("the ")
                || honorific.contains("hand")
                || honorific.contains("shield")
        );
    }

    #[test]
    fn test_title_generation() {
        let mut generator = FigureNameGenerator::new(456);

        let title = generator.generate_title(FigureType::Monarch, 0.9);
        assert!(title.is_some());
        assert!(title.unwrap().len() > 0);

        let title_low = generator.generate_title(FigureType::Artist, 0.4);
        assert!(title_low.is_some());
    }

    #[test]
    fn test_figure_lifecycle_state() {
        let state = FigureLifecycleState::Active;
        assert_eq!(state.label(), "Active");
        assert_eq!(state.visibility_level(), 5);
        assert_eq!(state.influence_multiplier(), 1.2);

        let legendary = FigureLifecycleState::Legendary;
        assert_eq!(legendary.influence_multiplier(), 1.5);
    }

    #[test]
    fn test_relationship_graph() {
        let mut graph = FigureRelationshipGraph::new();
        let figure1 = Uuid::new_v4();
        let figure2 = Uuid::new_v4();

        // Add parent-child relationship
        graph.add_relationship(
            figure1,
            FigureRelationship::new(figure2, FigureRelationshipType::Parent),
        );

        // Verify relationship exists
        let rels = graph.get_relationships(&figure1);
        assert_eq!(rels.len(), 1);

        // Verify bidirectional (child relationship auto-added)
        let child_rels = graph.get_relationships(&figure2);
        assert!(child_rels
            .iter()
            .any(|r| r.relationship_type == FigureRelationshipType::Child));

        // Check family members
        let family = graph.get_family(&figure1);
        assert!(family.contains(&figure2));
    }

    #[test]
    fn test_dynasty_creation() {
        let founder_id = Uuid::new_v4();
        let mut dynasty = Dynasty::new("House Pendragon".to_string(), founder_id, 450);

        assert_eq!(dynasty.name, "House Pendragon");
        assert_eq!(dynasty.founder_id, founder_id);
        assert!(dynasty.is_active());

        // Add members
        let member_id = Uuid::new_v4();
        dynasty.add_member(member_id);
        assert!(dynasty.member_ids.contains(&member_id));
    }

    #[test]
    fn test_biography_generation() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::Monarch, 0.85);
        figure.birth_year = Some(450);
        figure.death_year = Some(520);
        figure.add_accomplishment("United the warring tribes".to_string());
        figure.add_accomplishment("Founded the capital city".to_string());
        figure.name = Some(FigureName::new(
            Some("Arthur".to_string()),
            Some("Pendragon".to_string()),
        ));
        figure.titles = Some(vec!["King".to_string()]);
        let world_id = figure.world_id;

        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);
        generator.generate_biography(
            &mut figure,
            world_id,
            "Britannia",
            "the Age of Heroes",
            Some("Camelot"),
        );

        assert!(figure.biography.is_some());
        let bio = figure.biography.unwrap();
        assert!(bio.contains("Arthur"));
        assert!(bio.contains("King"));
        assert!(bio.contains("Age of Heroes"));
    }

    #[test]
    fn test_lifecycle_update() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::MilitaryLeader, 0.8);
        figure.birth_year = Some(900);
        // No death year set yet

        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);

        // Simulate 50 years - figure should still be alive
        let died = generator.update_lifecycle(
            &mut figure,
            950,
            50,
            0.01, // Base death probability
            "the Golden Age",
        );
        assert!(!died);
        assert!(figure.lifecycle_state == FigureLifecycleState::Active);
    }

    #[test]
    fn test_impact_score_calculation() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::Monarch, 0.7);
        figure.add_event(Uuid::new_v4());
        figure.add_event(Uuid::new_v4());
        figure.add_event(Uuid::new_v4());

        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);

        let impact = generator.calculate_impact_score(&figure);
        assert!(impact > figure.significance); // Events should increase impact
        assert!(impact <= 1.0); // Should be capped at 1.0
    }

    #[test]
    fn test_influence_propagation() {
        let mut figure = NotableFigure::new(Uuid::new_v4(), FigureType::Monarch, 0.8);
        figure.influence_region_id = Some(Uuid::new_v4());
        figure.influence_radius = Some(100.0);

        let primary_region = Uuid::new_v4();
        figure.influence_region_id = Some(primary_region);

        let adjacent: Vec<(Uuid, f32)> = vec![
            (Uuid::new_v4(), 30.0),
            (Uuid::new_v4(), 60.0),
            (Uuid::new_v4(), 150.0), // Outside radius
        ];

        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);

        let influences =
            generator.propagate_influence_to_regions(&figure, primary_region, &adjacent, 1000.0);

        // Should have primary + 2 within radius
        assert_eq!(influences.len(), 3);

        // Primary region should be marked
        let primary = influences.iter().find(|i| i.primary_region).unwrap();
        assert_eq!(primary.influence_score, 0.8 * 1.2); // significance * active modifier

        // Adjacent regions should have falloff
        let closer = influences
            .iter()
            .find(|i| !i.primary_region && i.influence_score > 0.4)
            .unwrap();
        let farther = influences
            .iter()
            .find(|i| !i.primary_region && i.influence_score < 0.4)
            .unwrap();
        assert!(closer.influence_score > farther.influence_score);
    }

    #[test]
    fn test_settlement_influence_modifiers() {
        let figure = NotableFigure::new(Uuid::new_v4(), FigureType::MilitaryLeader, 0.7);

        let config = FigureGeneratorConfig::default();
        let generator = FigureGenerator::new(config);

        let mut settlement_mod: Option<crate::types::SettlementType> = None;
        let mut fort_mod: f32 = 0.0;
        let mut cult_mod: f32 = 0.0;
        let mut spirit_mod: f32 = 0.0;

        generator.apply_settlement_influence(
            &mut settlement_mod,
            &mut fort_mod,
            &mut cult_mod,
            &mut spirit_mod,
            &figure,
            0.8,
        );

        // Military leader should add fortifications
        assert!(fort_mod > 0.0);
        assert_eq!(settlement_mod, None); // Shouldn't change settlement type
    }

    #[test]
    fn test_dynasty_store() {
        let mut store = DynastyStore::new();
        let founder_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        // Create and add dynasty
        let mut dynasty = Dynasty::new("House Valorian".to_string(), founder_id, 500);
        dynasty.realm_id = Some(realm_id);
        let dynasty_id = dynasty.id;
        store.add(dynasty);

        // Verify store operations
        assert_eq!(store.count(), 1);
        assert!(store.get(&dynasty_id).is_some());

        // Verify realm lookup
        let by_realm = store.get_by_realm(&realm_id);
        assert_eq!(by_realm.len(), 1);

        // Verify active dynasty
        let active = store.get_active_for_realm(&realm_id);
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "House Valorian");
    }

    #[test]
    fn test_dynasty_lifecycle() {
        let founder_id = Uuid::new_v4();
        let mut dynasty = Dynasty::new("House Stark".to_string(), founder_id, 300);

        // Initially active
        assert!(dynasty.is_active());

        // Add successor
        let successor_id = Uuid::new_v4();
        dynasty.add_member(successor_id);
        assert!(dynasty.member_ids.contains(&successor_id));

        // Change head
        dynasty.set_current_head(successor_id);
        assert_eq!(dynasty.current_head_id, Some(successor_id));

        // End dynasty
        dynasty.end(800);
        assert!(!dynasty.is_active());
        assert!(dynasty.current_head_id.is_none());
    }
}
