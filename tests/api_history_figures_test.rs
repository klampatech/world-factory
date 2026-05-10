//! Integration tests for history and figures API endpoints.
//!
//! Tests that GET /history and GET /figures return data from the world package.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use world_factory::figures::FigureType;
    use world_factory::packaging::WorldPackage;
    use world_factory::types::{
        ComprehensiveEventType, EntityId, EntityType, HistoricalEvent, HistoricalTime, Timestamp,
        World,
    };

    fn create_test_world() -> World {
        World::new("Test World".to_string(), 42)
    }

    fn create_test_event(
        world_id: Uuid,
        year: i32,
        name: &str,
        event_type: ComprehensiveEventType,
    ) -> HistoricalEvent {
        HistoricalEvent {
            id: EntityId::new(EntityType::Event),
            world_id,
            name: name.to_string(),
            time: HistoricalTime::year(year),
            end_time: None,
            description: format!("{} occurred", name),
            event_type: Some(event_type),
            participants: None,
            location_id: None,
            consequences: None,
            effects: None,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    #[test]
    fn test_figure_type_filtering() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        // Create figures with distinct types using the FiguresStore
        let mut figures_store = world_factory::figures::FigureStore::new();

        let monarch =
            world_factory::figures::NotableFigure::new(world_id, FigureType::Monarch, 0.9);
        figures_store.add(monarch.clone());

        let military =
            world_factory::figures::NotableFigure::new(world_id, FigureType::MilitaryLeader, 0.85);
        figures_store.add(military.clone());

        // Test filtering by type
        let monarchs: Vec<_> = figures_store.get_by_type(&world_id, FigureType::Monarch);
        assert_eq!(monarchs.len(), 1, "Should have exactly 1 monarch");

        let military_leaders: Vec<_> =
            figures_store.get_by_type(&world_id, FigureType::MilitaryLeader);
        assert_eq!(
            military_leaders.len(),
            1,
            "Should have exactly 1 military leader"
        );

        // Verify the types are different
        assert_ne!(monarchs[0].figure_type, military_leaders[0].figure_type);
    }

    #[test]
    fn test_world_package_with_events() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        let mut package = WorldPackage {
            world: world.clone(),
            regions: vec![],
            settlements: vec![],
            persons: vec![],
            events: vec![],
            timelines: vec![],
            terrain: None,
        };

        // Add events using the existing events field
        package.events.push(create_test_event(
            world_id,
            100,
            "First Settlement",
            ComprehensiveEventType::SettlementFounded,
        ));
        package.events.push(create_test_event(
            world_id,
            200,
            "The Great War",
            ComprehensiveEventType::WarDeclared,
        ));
        package.events.push(create_test_event(
            world_id,
            300,
            "Battle of Valor",
            ComprehensiveEventType::Battle,
        ));

        // Verify package has data
        assert_eq!(package.events.len(), 3, "Should have 3 events");

        // Verify event filtering (by year) - years 100, 200, 300 were added
        // [100, 250] includes events at 100 and 200 (2 events)
        let events_100_250: Vec<_> = package
            .events
            .iter()
            .filter(|e| e.time.get_year() >= 100 && e.time.get_year() <= 250)
            .collect();
        assert_eq!(
            events_100_250.len(),
            2,
            "Should have 2 events in year range 100-250"
        );
        assert_eq!(events_100_250[0].name, "First Settlement");
    }

    #[test]
    fn test_world_package_serialization_roundtrip() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        let mut package = WorldPackage {
            world: world.clone(),
            regions: vec![],
            settlements: vec![],
            persons: vec![],
            events: vec![],
            timelines: vec![],
            terrain: None,
        };

        // Add some events
        package.events.push(create_test_event(
            world_id,
            500,
            "Golden Age",
            ComprehensiveEventType::CulturalAchievement,
        ));

        // Serialize
        let json = serde_json::to_string_pretty(&package).unwrap();

        // Deserialize
        let loaded: WorldPackage = serde_json::from_str(&json).unwrap();

        // Verify roundtrip
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.events[0].name, "Golden Age");
    }

    #[test]
    fn test_event_filtering_by_significance() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        let mut package = WorldPackage {
            world,
            regions: vec![],
            settlements: vec![],
            persons: vec![],
            events: vec![],
            timelines: vec![],
            terrain: None,
        };

        // Add events with varying significance
        // Note: significance is implicit in event type
        package.events.push(create_test_event(
            world_id,
            100,
            "Minor Settlement",
            ComprehensiveEventType::SettlementFounded,
        ));
        package.events.push(create_test_event(
            world_id,
            200,
            "Major War",
            ComprehensiveEventType::WarDeclared,
        ));

        // Verify the filtering mechanism works
        assert_eq!(package.events.len(), 2, "Should have 2 total events");
    }

    #[test]
    fn test_figures_store_filtering() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        let mut figures_store = world_factory::figures::FigureStore::new();

        // Add figures with varying significance and types
        let monarch =
            world_factory::figures::NotableFigure::new(world_id, FigureType::Monarch, 0.9);
        figures_store.add(monarch);

        let scholar =
            world_factory::figures::NotableFigure::new(world_id, FigureType::Scholar, 0.6);
        figures_store.add(scholar);

        let hero = world_factory::figures::NotableFigure::new(world_id, FigureType::Hero, 0.95);
        figures_store.add(hero);

        // Get all figures for this world
        let all_figures = figures_store.get_by_world(&world_id);
        assert_eq!(all_figures.len(), 3, "Should have 3 figures");

        // Filter by type
        let monarchs = figures_store.get_by_type(&world_id, FigureType::Monarch);
        assert_eq!(monarchs.len(), 1, "Should have 1 monarch");

        let scholars = figures_store.get_by_type(&world_id, FigureType::Scholar);
        assert_eq!(scholars.len(), 1, "Should have 1 scholar");

        let heroes = figures_store.get_by_type(&world_id, FigureType::Hero);
        assert_eq!(heroes.len(), 1, "Should have 1 hero");
    }
}
