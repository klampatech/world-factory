# Phase 5 - Faction System Implementation

## Status: COMPLETED

## Overview

The Faction System provides persistent political entities that control territories, 
wage wars, form alliances, and drive political history. Factions are the primary 
actors in the world's political narrative.

## Files Created

### Core Module: `src/faction.rs`

- **Faction** struct - Core political entity with territory, settlements, population, relations
- **FactionRegistry** - Storage and management of all factions in a world
- **FactionType** enum (9 types):
  - Clan, Tribe, Chiefdom - Early/foundational political structures
  - Kingdom, Empire - Hereditary/monarchical structures
  - Theocracy - Religious governance
  - Republic - Elected governance
  - Confederation - Union of semi-independent states
  - Nomadic - Mobile groups without fixed territory
- **FactionRelation** enum (8 types):
  - Unknown, Peace, Allied, DefensivePact, TradeAgreement, Rivals, War, Suzerainty
- **DiplomaticRelation** struct - Stores relation details with treaty info

### Integration Module: `src/faction_integration.rs`

- **FactionGenerator** - Creates factions from societies/settlements
- **DiplomaticProcessor** - Generates diplomatic events over time
- **FactionRegistryExt** trait - Helper methods for faction operations
- **DiplomaticEvent** - Records diplomatic occurrences

### API Routes: `src/api/v1/factions.rs`

Endpoints:
- `GET /api/v1/factions` - List all factions
- `GET /api/v1/factions/:id` - Get faction details
- `GET /api/v1/factions/:id/relations` - Get diplomatic relations
- `GET /api/v1/factions/types` - List faction types with metadata

### API Models: `src/api/models.rs`

New types added:
- `FactionView` - Summary for listing
- `FactionDetailView` - Full faction details
- `DiplomaticRelationView` - Relation details
- `FactionTypeView` - Type metadata
- `FactionsListView` - Paginated list response

## Integration Points

### With History/Simulation Systems
- `FactionGenerator::generate_from_societies()` - Creates factions from Society entities
- `FactionGenerator::check_evolution()` - Updates faction type based on population
- `DiplomaticProcessor::process_year()` - Generates alliances/wars over time

### With Frontend (web/index.html)
- FACTION_NAMES array: Ironhold Clan, Meridian Empire, Mountain Kings, etc.
- FACTION_COLORS array: 8 distinct colors for map rendering
- Political overlay: Displays faction boundaries with colors

### With Existing Types
- Extends EntityType enum (implicit via EntityId)
- Compatible with PoliticalData in Region
- Works with Event system for war/alliance events

## Example Usage

```rust
use world_factory::{Faction, FactionRegistry, FactionType};

// Create a kingdom
let world_id = Uuid::new_v4();
let mut kingdom = Faction::new_kingdom(
    world_id,
    "Kingdom of Aldoria".to_string(),
    capital_id,
    1000,
);

// Add territory
kingdom.add_territory(42);
kingdom.add_territory(43);

// Register and manage relations
let mut registry = FactionRegistry::new();
registry.add(kingdom).unwrap();

// Create alliance
registry.create_alliance(kingdom_id, other_faction_id, 1050).unwrap();

// Declare war
registry.declare_war(kingdom_id, enemy_id, 1100).unwrap();
```

## Tests

Unit tests in `src/faction.rs` cover:
- Faction creation and properties
- Territory management
- Diplomatic relations
- Registry operations
- Type evolution based on population

## Next Steps (Future Phases)

1. Connect FactionRegistry to AppState for persistent storage
2. Generate initial factions during world generation
3. Integrate faction events with EventStore
4. Add faction-specific event types (Conquest, Treaty, etc.)
5. Connect factions to notable figures (leader_id)