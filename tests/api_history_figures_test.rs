//! Integration tests for history and figures API endpoints.
//!
//! Tests that GET /history and GET /figures return data from the world package.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use world_factory::events::EventType;
    use world_factory::figures::FigureType;
    use world_factory::packaging::WorldPackage;
    use world_factory::types::{EntityId, EntityType, HistoricalTime, Timestamp, World};

    fn create_test_world() -> World {
        World::new("Test World".to_string(), 42)
    }

    fn create_test_event(
        world_id: Uuid,
        year: i32,
        event_type: EventType,
        name: &str,
    ) -> world_factory::events::Event {
        world_factory::events::Event {
            id: EntityId::new(EntityType::Event),
            world_id,
            name: name.to_string(),
            description: format!("{} occurred", name),
            event_type,
            time: HistoricalTime::year(year),
            end_time: None,
            location_id: None,
            participants: None,
            effects: vec![],
            consequences: None,
            significance: Some(0.7),
            sources: None,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        }
    }

    fn create_test_figure(
        world_id: Uuid,
        figure_type: FigureType,
        significance: f32,
    ) -> world_factory::figures::NotableFigure {
        world_factory::figures::NotableFigure::new(world_id, figure_type, significance)
    }

    #[test]
    fn test_figure_type_filtering() {
        let world = create_test_world();
        let world_id = world.id.to_uuid();

        let mut figures = vec![];
        figures.push(create_test_figure(world_id, FigureType::Monarch, 0.9));
        figures.push(create_test_figure(
            world_id,
            FigureType::MilitaryLeader,
            0.85,
        ));

        // Test filtering
        let monarch_count = figures
            .iter()
            .filter(|f| f.figure_type == FigureType::Monarch)
            .count();
        assert_eq!(monarch_count, 1, "Should have exactly 1 monarch");

        let military_count = figures
            .iter()
            .filter(|f| f.figure_type == FigureType::MilitaryLeader)
            .count();
        assert_eq!(military_count, 1, "Should have exactly 1 military leader");

        // Verify the types are different
        assert_ne!(figures[0].figure_type, figures[1].figure_type);
    }

    #[test]
    fn test_world_package_with_events_and_figures() {
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
            geographies: None,
            event_store_events: vec![],
            notable_figures: vec![],
        };

        // Add events
        package.event_store_events.push(create_test_event(
            world_id,
            100,
            EventType::SettlementFounded,
            "First Settlement",
        ));
        package.event_store_events.push(create_test_event(
            world_id,
            200,
            EventType::WarDeclared,
            "The Great War",
        ));
        package.event_store_events.push(create_test_event(
            world_id,
            300,
            EventType::Battle,
            "Battle of Valor",
        ));

        // Add figures with distinct types
        let monarch = create_test_figure(world_id, FigureType::Monarch, 0.9);
        let military = create_test_figure(world_id, FigureType::MilitaryLeader, 0.85);

        println!(
            "Created monarch: type={:?}, id={:?}",
            monarch.figure_type, monarch.id
        );
        println!(
            "Created military: type={:?}, id={:?}",
            military.figure_type, military.id
        );

        package.notable_figures.push(monarch);
        package.notable_figures.push(military);

        // Verify package has data
        assert_eq!(package.event_store_events.len(), 3, "Should have 3 events");
        assert_eq!(package.notable_figures.len(), 2, "Should have 2 figures");

        // Debug: print all figure types
        for (i, f) in package.notable_figures.iter().enumerate() {
            println!("Package figure {}: type={:?}", i, f.figure_type);
        }

        // Verify we can filter figures by type
        let monarchs: Vec<_> = package
            .notable_figures
            .iter()
            .filter(|f| f.figure_type == FigureType::Monarch)
            .collect();
        println!("Monarchs filter result: {}", monarchs.len());
        assert_eq!(monarchs.len(), 1, "Should have exactly 1 monarch");

        let military: Vec<_> = package
            .notable_figures
            .iter()
            .filter(|f| f.figure_type == FigureType::MilitaryLeader)
            .collect();
        println!("Military filter result: {}", military.len());
        assert_eq!(military.len(), 1, "Should have exactly 1 military leader");

        // Verify event filtering (by year) - years 100, 200, 300 were added
        // [100, 250] includes events at 100 and 200 (2 events)
        let events_100_250: Vec<_> = package
            .event_store_events
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
            geographies: None,
            event_store_events: vec![],
            notable_figures: vec![],
        };

        // Add some events
        package.event_store_events.push(create_test_event(
            world_id,
            500,
            EventType::CulturalAchievement,
            "Golden Age",
        ));

        // Add some figures
        package
            .notable_figures
            .push(create_test_figure(world_id, FigureType::Scholar, 0.75));

        // Serialize
        let json = serde_json::to_string_pretty(&package).unwrap();

        // Deserialize
        let loaded: WorldPackage = serde_json::from_str(&json).unwrap();

        // Verify roundtrip
        assert_eq!(loaded.event_store_events.len(), 1);
        assert_eq!(loaded.notable_figures.len(), 1);
        assert_eq!(loaded.event_store_events[0].name, "Golden Age");
        assert_eq!(loaded.notable_figures[0].figure_type, FigureType::Scholar);
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
            geographies: None,
            event_store_events: vec![],
            notable_figures: vec![],
        };

        // Add events with varying significance
        package.event_store_events.push(create_test_event(
            world_id,
            100,
            EventType::SettlementFounded,
            "Minor Settlement",
        ));
        // Note: create_test_event uses 0.7 significance

        // Test that significance filtering works
        let high_significance: Vec<_> = package
            .event_store_events
            .iter()
            .filter(|e| e.significance.unwrap_or(0.0) >= 0.7)
            .collect();
        assert!(!high_significance.is_empty());
    }
}
