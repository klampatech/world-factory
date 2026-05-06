//! Entity module - storage and relationships for generated world elements
//!
//! Contains entity definitions, spatial positioning, and world population.

use crate::util::Vec2;
use serde::{Deserialize, Serialize};

/// Entity identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

/// Entity type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub position: Vec2,
    pub significance: Option<u8>,
}

impl Entity {
    pub fn new(id: String, entity_type: &str, name: String, position: Vec2) -> Self {
        Self {
            id,
            entity_type: entity_type.to_string(),
            name,
            position,
            significance: None,
        }
    }
}

/// Simple entity storage with spatial indexing.
#[derive(Debug, Default)]
pub struct EntityStore {
    entities: Vec<Entity>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }
}
