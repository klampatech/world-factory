//! Serialization tests for World Factory types.
//! 
//! Verifies that all types can be serialized to JSON and deserialized back
//! without loss of information.

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    
    // ========================================================================
    // Identifier Type Tests
    // ========================================================================
    
    #[test]
    fn test_entity_id_serialization() {
        let world_id = EntityId::new(EntityType::World);
        let json = serde_json::to_string(&world_id).unwrap();
        let parsed: EntityId = serde_json::from_str(&json).unwrap();
        assert_eq!(world_id, parsed);
    }
    
    #[test]
    fn test_entity_id_with_custom_uuid() {
        let custom_uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let entity_id = EntityId::from_uuid(custom_uuid, EntityType::Nation);
        let json = serde_json::to_string(&entity_id).unwrap();
        assert!(json.contains("550e8400"));
        assert!(json.contains("nation"));
    }
    
    // ========================================================================
    // Timestamp Tests
    // ========================================================================
    
    #[test]
    fn test_timestamp_serialization() {
        let ts = Timestamp::now();
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(ts.to_unix(), parsed.to_unix());
    }
    
    #[test]
    fn test_timestamp_from_unix() {
        let original = Timestamp::from_unix(1609459200); // 2021-01-01 00:00:00 UTC
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(original.to_unix(), parsed.to_unix());
    }
    
    #[test]
    fn test_historical_time_variants() {
        // Test Year variant
        let hist = HistoricalTime::date(1066, 10, 14);
        let json = serde_json::to_string(&hist).unwrap();
        let parsed: HistoricalTime = serde_json::from_str(&json).unwrap();
        assert_eq!(hist, parsed);
        
        // Test approximate
        let approx = HistoricalTime::year(500);
        let json = serde_json::to_string(&approx).unwrap();
        assert!(json.contains("approximate"));
        
        // Test Relative
        let relative = HistoricalTime::Relative { years: -50, months: 0 };
        let json = serde_json::to_string(&relative).unwrap();
        let parsed: HistoricalTime = serde_json::from_str(&json).unwrap();
        assert_eq!(relative, parsed);
        
        // Test Unknown
        let unknown = HistoricalTime::Unknown;
        let json = serde_json::to_string(&unknown).unwrap();
        let parsed: HistoricalTime = serde_json::from_str(&json).unwrap();
        assert_eq!(unknown, parsed);
    }
    
    // ========================================================================
    // World Type Tests
    // ========================================================================
    
    #[test]
    fn test_world_serialization() {
        let mut world = World::new("Middle Earth".to_string(), 42);
        world.description = Some("A fictional world".to_string());
        
        let json = serde_json::to_string(&world).unwrap();
        let parsed: World = serde_json::from_str(&json).unwrap();
        
        assert_eq!(world.id, parsed.id);
        assert_eq!(world.name, parsed.name);
        assert_eq!(world.seed, parsed.seed);
        assert_eq!(world.description, parsed.description);
    }
    
    #[test]
    fn test_world_minimal_serialization() {
        // World with only required fields
        let world = World::new("Test World".to_string(), 12345);
        let json = serde_json::to_string(&world).unwrap();
        
        // Verify no optional fields are present
        assert!(!json.contains("metadata"));
        assert!(!json.contains("description"));
    }
    
    #[test]
    fn test_world_with_metadata() {
        let mut world = World::new("Magical World".to_string(), 999);
        world.metadata = Some(WorldMetadata::default());
        
        let json = serde_json::to_string(&world).unwrap();
        let parsed: World = serde_json::from_str(&json).unwrap();
        
        assert!(parsed.metadata.is_some());
        let meta = parsed.metadata.unwrap();
        assert_eq!(meta.genre, Some(Genre::Fantasy));
        assert_eq!(meta.tech_level, Some(TechLevel::Medieval));
    }
    
    #[test]
    fn test_genre_serialization() {
        for genre in [
            Genre::Fantasy,
            Genre::SciFi,
            Genre::Historical,
            Genre::Modern,
            Genre::PostApocalyptic,
            Genre::Horror,
            Genre::Cyberpunk,
            Genre::Steampunk,
        ] {
            let json = serde_json::to_string(&genre).unwrap();
            let parsed: Genre = serde_json::from_str(&json).unwrap();
            assert_eq!(genre, parsed);
        }
    }
    
    #[test]
    fn test_tech_level_serialization() {
        for level in [
            TechLevel::Prehistoric,
            TechLevel::Ancient,
            TechLevel::Classical,
            TechLevel::Medieval,
            TechLevel::Renaissance,
            TechLevel::Industrial,
            TechLevel::Modern,
            TechLevel::Future,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let parsed: TechLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, parsed);
        }
    }
    
    #[test]
    fn test_magic_system_serialization() {
        let magic = MagicSystem {
            enabled: true,
            system_type: Some(MagicType::Arcane),
            rarity: Some(MagicRarity::Rare),
        };
        
        let json = serde_json::to_string(&magic).unwrap();
        let parsed: MagicSystem = serde_json::from_str(&json).unwrap();
        
        assert_eq!(magic.enabled, parsed.enabled);
        assert_eq!(magic.system_type, parsed.system_type);
        assert_eq!(magic.rarity, parsed.rarity);
    }
    
    #[test]
    fn test_magic_disabled() {
        let magic = MagicSystem {
            enabled: false,
            system_type: None,
            rarity: None,
        };
        
        let json = serde_json::to_string(&magic).unwrap();
        assert!(!json.contains("system_type"));
        
        let parsed: MagicSystem = serde_json::from_str(&json).unwrap();
        assert!(!parsed.enabled);
    }
    
    // ========================================================================
    // Region Type Tests
    // ========================================================================
    
    #[test]
    fn test_region_serialization() {
        let region = Region::new(
            uuid::Uuid::new_v4(),
            "The Shire".to_string(),
            29400.0,
            -25.0,
            150.0,
        );
        
        let json = serde_json::to_string(&region).unwrap();
        let parsed: Region = serde_json::from_str(&json).unwrap();
        
        assert_eq!(region.name, parsed.name);
        assert_eq!(region.area_km2, parsed.area_km2);
        assert_eq!(region.center_lat, parsed.center_lat);
        assert_eq!(region.center_lon, parsed.center_lon);
    }
    
    #[test]
    fn test_region_with_climate() {
        let mut region = Region::new(
            uuid::Uuid::new_v4(),
            "Frostfell".to_string(),
            150000.0,
            65.0,
            -120.0,
        );
        region.climate = Some(ClimateZone::Polar);
        
        let json = serde_json::to_string(&region).unwrap();
        let parsed: Region = serde_json::from_str(&json).unwrap();
        
        assert_eq!(region.climate, parsed.climate);
    }
    
    #[test]
    fn test_political_data_serialization() {
        let political = PoliticalData {
            government_type: Some(GovernmentType::Monarchy),
            capital_id: Some(uuid::Uuid::new_v4()),
            ruling_faction: Some("House Stark".to_string()),
            population: Some(45000),
            founded_year: Some(800),
        };
        
        let json = serde_json::to_string(&political).unwrap();
        let parsed: PoliticalData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(political.government_type, parsed.government_type);
        assert_eq!(political.ruling_faction, parsed.ruling_faction);
        assert_eq!(political.population, parsed.population);
    }
    
    // ========================================================================
    // Settlement Type Tests
    // ========================================================================
    
    #[test]
    fn test_settlement_serialization() {
        let settlement = Settlement::new(
            uuid::Uuid::new_v4(),
            "Winterfell".to_string(),
            GeoLocation::with_elevation(64.5, -90.0, 250.0),
        );
        
        let json = serde_json::to_string(&settlement).unwrap();
        let parsed: Settlement = serde_json::from_str(&json).unwrap();
        
        assert_eq!(settlement.name, parsed.name);
        assert_eq!(settlement.location.latitude, parsed.location.latitude);
        assert!(parsed.location.elevation_m.is_some());
    }
    
    #[test]
    fn test_settlement_type_variants() {
        for stype in [
            SettlementType::Hamlet,
            SettlementType::Village,
            SettlementType::Town,
            SettlementType::City,
            SettlementType::Metropolis,
            SettlementType::Capital,
            SettlementType::Fortress,
            SettlementType::Port,
            SettlementType::SacredSite,
        ] {
            let json = serde_json::to_string(&stype).unwrap();
            let parsed: SettlementType = serde_json::from_str(&json).unwrap();
            assert_eq!(stype, parsed);
        }
    }
    
    // ========================================================================
    // Person Type Tests
    // ========================================================================
    
    #[test]
    fn test_person_serialization() {
        let mut person = Person::new();
        person.name = Some(PersonName::new("Eddard".to_string(), "Stark".to_string()));
        person.birth_time = Some(HistoricalTime::year(-100));
        person.death_time = Some(HistoricalTime::date(283, 1, 15));
        person.culture = Some("Northman".to_string());
        person.titles = Some(vec!["Warden of the North".to_string(), "Lord of Winterfell".to_string()]);
        
        let json = serde_json::to_string(&person).unwrap();
        let parsed: Person = serde_json::from_str(&json).unwrap();
        
        assert_eq!(person.name, parsed.name);
        assert_eq!(person.culture, parsed.culture);
        assert_eq!(person.titles.as_ref().unwrap().len(), parsed.titles.unwrap().len());
    }
    
    #[test]
    fn test_person_name_display() {
        let name = PersonName {
            given: Some("John".to_string()),
            family: Some("Doe".to_string()),
            epithet: Some("the Bold".to_string()),
            title: Some("King".to_string()),
        };
        
        let display = name.to_string();
        assert_eq!(display, "King John Doe");
    }
    
    #[test]
    fn test_person_name_serialization() {
        let name = PersonName::new("Tyrion".to_string(), "Lannister".to_string());
        let json = serde_json::to_string(&name).unwrap();
        let parsed: PersonName = serde_json::from_str(&json).unwrap();
        assert_eq!(name, parsed);
    }
    
    // ========================================================================
    // Event Type Tests
    // ========================================================================
    
    #[test]
    fn test_historical_event_serialization() {
        let event = HistoricalEvent::new(
            uuid::Uuid::new_v4(),
            "Battle of Helms Deep".to_string(),
            HistoricalTime::date(3019, 3, 15),
            "The battle took place in the deep of Helm's Deep.".to_string(),
        );
        
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HistoricalEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.name, parsed.name);
        assert_eq!(event.description, parsed.description);
    }
    
    #[test]
    fn test_event_with_participants() {
        let mut event = HistoricalEvent::new(
            uuid::Uuid::new_v4(),
            "Treaty of Westphalia".to_string(),
            HistoricalTime::date(1648, 10, 24),
            "Peace treaty ending the Thirty Years' War.".to_string(),
        );
        event.event_type = Some(EventType::Treaty);
        event.participants = Some(vec![
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        ]);
        event.consequences = Some(vec![
            "End of religious wars in Europe".to_string(),
            "Sovereignty of states established".to_string(),
        ]);
        
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HistoricalEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.event_type, parsed.event_type);
        assert_eq!(event.participants.unwrap().len(), 2);
        assert_eq!(event.consequences.unwrap().len(), 2);
    }
    
    #[test]
    fn test_event_type_variants() {
        for etype in [
            crate::events::event_type::EventType::SettlementFounded,
            crate::events::event_type::EventType::WarDeclared,
            crate::events::event_type::EventType::Plague,
            crate::events::event_type::EventType::Migration,
            crate::events::event_type::EventType::Exploration,
            crate::events::event_type::EventType::Collapse,
            crate::events::event_type::EventType::Battle,
            crate::events::event_type::EventType::Invention,
        ] {
            let json = serde_json::to_string(&etype).unwrap();
            let parsed: crate::events::event_type::EventType = serde_json::from_str(&json).unwrap();
            assert_eq!(etype, parsed);
        }
    }
    
    // ========================================================================
    // Timeline Type Tests
    // ========================================================================
    
    #[test]
    fn test_timeline_serialization() {
        let timeline = Timeline::new(
            uuid::Uuid::new_v4(),
            "History of Middle Earth".to_string(),
        );
        
        let json = serde_json::to_string(&timeline).unwrap();
        let parsed: Timeline = serde_json::from_str(&json).unwrap();
        
        assert_eq!(timeline.name, parsed.name);
        assert!(parsed.events.is_empty());
    }
    
    #[test]
    fn test_timeline_with_events() {
        let mut timeline = Timeline::new(
            uuid::Uuid::new_v4(),
            "The Great War".to_string(),
        );
        timeline.events = vec![
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        ];
        timeline.start_year = Some(-1000);
        timeline.end_year = Some(-500);
        
        let json = serde_json::to_string(&timeline).unwrap();
        let parsed: Timeline = serde_json::from_str(&json).unwrap();
        
        assert_eq!(timeline.events.len(), parsed.events.len());
        assert_eq!(timeline.start_year, parsed.start_year);
        assert_eq!(timeline.end_year, parsed.end_year);
    }
    
    // ========================================================================
    // Terrain Type Tests (from biome.rs)
    // ========================================================================
    
    #[test]
    fn test_biome_type_serialization() {
        // Test all biome types
        let biomes = [
            BiomeType::TropicalRainforest,
            BiomeType::TemperateDesert,
            BiomeType::MagicalForest,
            BiomeType::VolcanicLandscape,
        ];
        
        for biome in biomes {
            let json = serde_json::to_string(&biome).unwrap();
            let parsed: BiomeType = serde_json::from_str(&json).unwrap();
            assert_eq!(biome, parsed);
        }
    }
    
    #[test]
    fn test_elevation_zone_from_height() {
        assert_eq!(ElevationZone::from_height(-100.0), ElevationZone::Lowland);
        assert_eq!(ElevationZone::from_height(250.0), ElevationZone::Lowland);
        assert_eq!(ElevationZone::from_height(1000.0), ElevationZone::Midland);
        assert_eq!(ElevationZone::from_height(2000.0), ElevationZone::Highland);
        assert_eq!(ElevationZone::from_height(4000.0), ElevationZone::Alpine);
        assert_eq!(ElevationZone::from_height(5500.0), ElevationZone::Nival);
    }
    
    #[test]
    fn test_climate_zone_serialization() {
        for zone in [
            ClimateZone::Tropical,
            ClimateZone::Subtropical,
            ClimateZone::Temperate,
            ClimateZone::Boreal,
            ClimateZone::Polar,
        ] {
            let json = serde_json::to_string(&zone).unwrap();
            let parsed: ClimateZone = serde_json::from_str(&json).unwrap();
            assert_eq!(zone, parsed);
        }
    }
    
    #[test]
    fn test_moisture_level_serialization() {
        for level in [
            MoistureLevel::HyperArid,
            MoistureLevel::Arid,
            MoistureLevel::SemiArid,
            MoistureLevel::SubHumid,
            MoistureLevel::Humid,
            MoistureLevel::PerHumid,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let parsed: MoistureLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, parsed);
        }
    }
    
    // ========================================================================
    // Optional Field Skip Tests
    // ========================================================================
    
    #[test]
    fn test_optional_fields_skipped_when_none() {
        // Person with only required fields
        let person = Person::new();
        let json = serde_json::to_string(&person).unwrap();
        
        // Verify optional fields are not present
        assert!(!json.contains("\"birth_time\""));
        assert!(!json.contains("\"culture\""));
        assert!(!json.contains("\"titles\""));
    }
    
    #[test]
    fn test_world_with_no_metadata() {
        let world = World::new("Minimal World".to_string(), 1);
        let json = serde_json::to_string(&world).unwrap();
        assert!(!json.contains("\"metadata\""));
    }
    
    // ========================================================================
    // Round-trip Integration Tests
    // ========================================================================
    
    #[test]
    fn test_world_full_roundtrip() {
        let mut world = World::new("Integration Test World".to_string(), 123456);
        world.description = Some("A test world for serialization".to_string());
        world.metadata = Some(WorldMetadata {
            genre: Some(Genre::SciFi),
            tech_level: Some(TechLevel::Future),
            magic: Some(MagicSystem {
                enabled: false,
                system_type: None,
                rarity: None,
            }),
        });
        
        let json = serde_json::to_string_pretty(&world).unwrap();
        let parsed: World = serde_json::from_str(&json).unwrap();
        
        assert_eq!(world.id, parsed.id);
        assert_eq!(world.name, parsed.name);
        assert_eq!(world.metadata.unwrap().genre, Some(Genre::SciFi));
    }
    
    #[test]
    fn test_entity_display_format() {
        let id = EntityId::from_uuid(
            uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap(),
            EntityType::Person,
        );
        assert_eq!(id.to_string(), "per:12345678-1234-1234-1234-123456789abc");
    }
}
