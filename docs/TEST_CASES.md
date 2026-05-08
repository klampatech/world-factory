# World Factory — Comprehensive Test Case Specification

**Project**: https://github.com/klampatech/world-factory
**Generated**: 2026-05-08
**Scope**: Full-stack (Rust backend + TypeScript/Canvas frontend + E2E automation)
**Approach**: Subagent exploration of backend, frontend, and infrastructure → synthesized into unified test catalog

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│  Rust Backend (src/)                            │
│  ├── terrain/     — Voronoi generation, biomes  │
│  ├── simulation/  — PopulationModel, disease    │
│  ├── species/     — SpeciesData + loader       │
│  ├── events/      — EventTimeline, EventBuilder │
│  ├── faction/     — FactionRegistry, TurnPhase  │
│  ├── api/v1/      — 20+ HTTP handlers           │
│  ├── storage.rs   — StorageManager, paths      │
│  └── packaging.rs — .wfw tarball save/load      │
└─────────────────────────────────────────────────┘
                     │ REST API :8080
┌─────────────────────────────────────────────────┐
│  Frontend (web/ + src/)                        │
│  ├── Vanilla JS — app.js, map-view.js          │
│  ├── React/TS  — Dashboard.tsx, MapComponent   │
│  ├── Canvas    — MapViewer.ts rendering        │
│  └── Services  — dashboardService.ts           │
└─────────────────────────────────────────────────┘
                     │ Playwright :8765
┌─────────────────────────────────────────────────┐
│  Automation (e2e/)                              │
│  ├── 26+ smoke-test specs                       │
│  ├── API endpoint tests                        │
│  └── Console error detection                   │
└─────────────────────────────────────────────────┘
```

---

## PART 1 — UNIT TESTS

### 1A. Backend Unit Tests (Rust — `#[cfg(test)]` modules)

#### Types Module (`src/types.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B001** | `entity_id_new_world` | `EntityId::new(EntityType::World)` | — | Valid EntityId created, inner UUID present |
| **U-B002** | `entity_id_from_uuid` | `EntityId::from_uuid(uuid, Region)` | UUID generator | Roundtrips correctly via `to_uuid()` |
| **U-B003** | `entity_id_display` | `format!("{}", entity_id)` | EntityId with known UUID | Output format `"world:{uuid}"` |
| **U-B004** | `timestamp_now` | `Timestamp::now()` | — | Within 1 second of `SystemTime::now()` |
| **U-B005** | `timestamp_roundtrip` | `Timestamp::from_datetime(dt).as_datetime()` | DateTime<Utc> | `dt` preserved exactly |
| **U-B006** | `historical_time_year` | `HistoricalTime::year(1200)` | — | Is approximate, year=1200 |
| **U-B007** | `historical_time_date` | `HistoricalTime::date(1066, 10, 14)` | — | Is exact, not approximate |
| **U-B008** | `world_new` | `World::new("Moria".into(), seed)` | Seed 42 | Has valid UUID, timestamps within 1s, current_year=0 |
| **U-B009** | `world_json_roundtrip` | Serialize then deserialize | World with all fields | All fields preserved |
| **U-B010** | `settlement_carrying_capacity_ocean` | BiomeType::Ocean | — | Returns 0 |
| **U-B011** | `settlement_carrying_capacity_tropical` | BiomeType::TropicalRainforest | — | Returns 7000 |
| **U-B012** | `settlement_carrying_capacity_tundra` | BiomeType::Tundra | — | Returns 300 |
| **U-B013** | `geo_location_new` | `GeoLocation::new(45.0, -93.0)` | — | elevation=None |
| **U-B014** | `geo_location_with_elevation` | `GeoLocation::with_elevation(...)` | — | elevation=Some(...) |
| **U-B015** | `entity_type_short_all_variants` | All 11 EntityType variants | — | Each returns non-empty short_name |

#### Species Module (`src/species/mod.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B020** | `species_id_human` | `SpeciesId::from_u32(1)` | — | `== SpeciesId::Human` |
| **U-B021** | `species_id_invalid` | `SpeciesId::from_u32(99)` | — | `== SpeciesId::Undefined` |
| **U-B022** | `species_id_all_count` | `SpeciesId::all()` | — | len() == 5 |
| **U-B023** | `species_id_display_human` | `SpeciesId::Human.display_name()` | — | Returns `"Human"` |
| **U-B024** | `human_inhabits_temperate` | `Species::Human.inhabits(TemperateGrassland)` | — | Returns true |
| **U-B025** | `human_inhabits_not_desert` | `Species::Human.inhabits(HotDesert)` | — | Returns false |
| **U-B026** | `elf_inhabits_forest` | `Species::Elf.inhabits(TemperateDeciduousForest)` | — | Returns true |
| **U-B027** | `dwarf_inhabits_boreal` | `Species::Dwarf.inhabits(BorealForest)` | — | Returns true |
| **U-B028** | `orc_tolerates_tundra` | `Species::Orc.tolerates(Tundra)` | — | Returns true (not home) |
| **U-B029** | `species_suitability_home` | Any species × home biome | SpeciesData default | Returns 1.0 |
| **U-B030** | `species_suitability_tolerable` | Species × tolerable biome | SpeciesData default | Returns 0.5 |
| **U-B031** | `species_suitability_intolerable` | Species × intolerable biome | SpeciesData default | Returns 0.0 |
| **U-B032** | `trait_adaptable_bonus` | Adaptable species × tolerable biome | SpeciesData with Adaptable trait | `trait_biome_modifier()` returns +0.25 |
| **U-B033** | `trait_curious_growth` | Species with Curious trait | SpeciesData | `trait_growth_modifier()` returns 0.25 |
| **U-B034** | `trait_adaptable_growth` | Species with Adaptable trait | SpeciesData | `trait_growth_modifier()` returns 0.10 |
| **U-B035** | `species_data_default_5_species` | `SpeciesData::default_species()` | — | Contains 5 species |
| **U-B036** | `species_data_get_human` | `default_species().get(Human)` | — | Returns `Some(&Species)` |
| **U-B037** | `species_data_get_undefined` | `default_species().get(Undefined)` | — | Returns None |
| **U-B038** | `species_data_best_for_temperate_grassland` | SpeciesData × TemperateGrassland | — | Human or Halfling returned |
| **U-B039** | `species_data_best_for_boreal` | SpeciesData × BorealForest | — | Dwarf returned |
| **U-B040** | `species_data_generate_name_not_empty` | `generate_name(Human, &mut rng)` | rng with fixed seed | Non-empty string |
| **U-B041** | `species_data_generate_name_valid_suffix` | `generate_name(Orc, &mut rng)` | rng with fixed seed | Ends with Orc suffix template |
| **U-B042** | `species_data_generate_name_fallback` | `generate_name(Undefined, &mut rng)` | — | Falls back to Human templates |
| **U-B043** | `merge_with_defaults_overrides` | Custom species with same ID | SpeciesData with defaults | Custom overrides default |
| **U-B044** | `merge_with_defaults_appends` | New species not in defaults | SpeciesData with defaults | New species appended |
| **U-B045** | `merge_with_defaults_preserves_original` | Partial override | SpeciesData with defaults | Non-overridden defaults preserved |

#### Storage Module (`src/storage.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B050** | `storage_config_default` | `StorageConfig::default()` | — | All fields match expected defaults |
| **U-B051** | `storage_config_with_base_dir` | `StorageConfig::with_base_dir("/tmp/test")` | — | `base_dir()` returns `/tmp/test` |
| **U-B052** | `storage_config_base_dir_falls_back` | `StorageConfig::default()` on Linux | No env var set | Uses `~/.local/share/world-factory` |
| **U-B053** | `get_storage_dir_env_override` | `WORLD_FACTORY_DATA_DIR="/tmp/wf"` | env var set | Returns `/tmp/wf` |
| **U-B054** | `storage_manager_creation` | `StorageManager::new(config)` with create_dirs=true | TempDir | All subdirs created |
| **U-B055** | `storage_manager_creation_no_dirs` | `StorageManager::new(config)` with create_dirs=false | — | Succeeds without creating dirs |
| **U-B056** | `world_paths_strip_world_prefix` | `"world:abc-123"` to any path method | StorageManager, UUID "abc-123" | Normalized to "abc-123" |
| **U-B057** | `world_package_path_correct` | `world_package_path("abc-123")` | StorageManager with base /data | Returns `/data/generated/abc-123/world.wfw` |
| **U-B058** | `world_exists_false` | `world_exists("nonexistent-id")` | Empty generated/ dir | Returns false |
| **U-B059** | `world_exists_true` | `world_exists(id)` after fake .wfw written | TempDir | Returns true |
| **U-B060** | `list_worlds_empty` | `list_worlds()` | Empty generated/ dir | Returns empty vec |
| **U-B061** | `list_worlds_finds_two` | `list_worlds()` | Two world dirs with .wfw files | Returns 2 WorldStorageInfo, sorted by mtime descending |
| **U-B062** | `storage_stats_zero` | `storage_stats()` on fresh manager | TempDir | world_count=0, total_bytes=0 |
| **U-B063** | `delete_world_removes_dir` | `delete_world(id)` | World dir exists | Directory removed, returns ok |
| **U-B064** | `cleanup_temp_removes_old` | `cleanup_temp(Duration::from_secs(60))` | Temp dirs older than 60s | Old dirs removed, returns bytes freed |
| **U-B065** | `clean_cache_returns_bytes` | `clean_cache()` | Populated cache dir | Returns bytes removed, cache dir recreated |
| **U-B066** | `bytes_to_human_500` | `bytes_to_human(500)` | — | Returns `"500 B"` |
| **U-B067** | `bytes_to_human_1mb` | `bytes_to_human(1_048_576)` | — | Returns `"1.00 MB"` |
| **U-B068** | `bytes_to_human_1gb` | `bytes_to_human(1_073_741_824)` | — | Returns `"1.00 GB"` |
| **U-B069** | `world_storage_info_size_human` | `WorldStorageInfo` with 5MB | — | size_human returns `"5.00 MB"` |
| **U-B070** | `storage_error_is_permission` | `StorageError::PermissionDenied` | — | `is_permission_error()` returns true |

#### Packaging Module (`src/packaging.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B075** | `save_world_creates_tarball` | `save_world(&world, path)` | TempFile path, valid World | File exists at path |
| **U-B076** | `save_world_includes_manifest` | `save_world(&world, path)` | — | manifest.json present in tarball |
| **U-B077** | `save_world_includes_world_json` | `save_world(&world, path)` | — | world.json present in tarball |
| **U-B078** | `save_world_compresses` | `save_world(&world, path)` | — | Gzip magic bytes (`\x1f\x8b`) at file start |
| **U-B079** | `load_world_roundtrip` | save → load | World with all fields | World.name, world.seed, world.id preserved |
| **U-B080** | `load_world_unknown_entry` | tarball missing world.json | Malformed tarball | Returns `PackageError::EntryNotFound` |
| **U-B081** | `load_world_corrupted_json` | tarball with invalid JSON | Corrupted data | Returns `PackageError::Json(...)` error |
| **U-B082** | `save_world_package_full` | Full WorldPackage with all fields | WorldPackage struct | All entries serialized |

#### Terrain Module (`src/terrain/`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B090** | `terrain_generator_new_valid` | `TerrainGenerator::new(config)` | Config with width=128, height=128 | Struct created without panic |
| **U-B091** | `terrain_generator_generate_full` | `.generate(TerrainLayer::Full)` | Fixed seed 42 | Returns TerrainGrid 128×128 |
| **U-B092** | `terrain_generator_deterministic` | generate(seed=42) twice | Same config | Both grids identical |
| **U-B093** | `terrain_generator_get_tectonic_result` | generate with `enable_tectonics=true` | — | Returns Some(TectonicResult) |
| **U-B094** | `terrain_grid_dimensions` | `TerrainGrid::new(128, 128)` | — | cell_count == 128*128 = 16384 |
| **U-B095** | `terrain_grid_get_valid_coord` | `grid.get(50, 50)` | 128×128 grid | Returns Some(Cell) |
| **U-B096** | `terrain_grid_get_invalid_coord` | `grid.get(200, 200)` | 128×128 grid | Returns None |
| **U-B097** | `ocean_detector_detects_land_and_water` | `.detect_ocean(&grid)` | Grid with known land/water | Returns mix of OceanZone variants |
| **U-B098** | `ocean_detector_shallow_at_coast` | Coast cell elevation ~0.23 | Grid from terrain gen | ShallowWater at coastline |
| **U-B099** | | `biome_calculation(highland, cold, dry)` | Elevation>0.8, temp<5°C, precip<200mm | Returns Alpine/Montane biome |
| **U-B100** | `biome_calculation_lowland_warm_wet` | `biome_calculation(lowland, warm, wet)` | Elevation<0.3, temp>25°C, precip>2000mm | Returns TropicalRainforest |
| **U-B101** | `climate_equator_hot_wet` | `calculate(0.0, 0, 500.0)` | Lat=0° (equator) | High temperature, moderate precipitation |
| **U-B102** | `climate_poles_cold` | `calculate(85.0, 2000.0, 100.0)` | Lat=85° | Low temperature |
| **U-B103** | `climate_elevation_colder` | Same lat, elevation 2000m vs 0m | — | Higher elevation = lower temp |
| **U-B104** | `climate_wind_direction_trade` | Lat 30°S | — | Returns Trade winds |
| **U-B105** | `climate_wind_direction_westerlies` | Lat 50°N | — | Returns Westerly |
| **U-B106** | `lloyd_relaxation_zero_iterations` | `relax(polygons, 0, ...)` | PolygonGraph | No change to polygon centroids |
| **U-B107** | `lloyd_relaxation_even_cells` | `relax(polygons, 3, ...)` | PolygonGraph | Cell uniformity increases (stddev decreases) |
| **U-B108** | `quick_relax_default` | `quick_relax(polygons, ...)` | PolygonGraph | Calls relax with iterations=2 |
| **U-B109** | `elevation_assign_increases_inland` | ElevationAssigner with known coast | PolygonGraph | Cells farther from coast have higher elevation |
| **U-B110** | `elevation_assign_respects_sea_level` | ElevationConfig with sea_level=0.25 | PolygonGraph | Cells with elevation < 0.25 are ocean |

#### Simulation Module (`src/simulation/population.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B120** | `population_model_new` | `PopulationModel::new(seed=0)` | Seed 0 | Model created with default config |
| **U-B121** | `population_model_add_settlement` | Add settlement via `add_settlement()` | Settlement struct | Settlement tracked internally |
| **U-B122** | `population_model_advance_years_zero` | `advance_years(0)` | Model with population | Returns empty vec |
| **U-B123** | `population_model_population_grows` | `advance_years(100)` | Settlement with pop=100, carrying=5000 | Population increases over 100 years |
| **U-B124** | `population_model_growth_slows_near_cap` | Same settlement advanced many times | — | Growth rate decreases as pop approaches carrying |
| **U-B125** | `population_model_society_transition` | Advance years for growing settlement | — | Tribe→Chiefdom→Nation as pop crosses thresholds |
| **U-B126** | `population_model_wonder_bonus` | Add wonder to settlement, advance | Settlement with wonder bonus | Growth modifier increased |
| **U-B127** | `population_model_get_population_some` | Settlement added, query same ID | UUID of known settlement | Returns Some(u64) |
| **U-B128** | `population_model_get_population_none` | Query unknown settlement UUID | — | Returns None |
| **U-B129** | `population_model_disease_prob_clamped` | `set_disease_probability(5.0)` | — | Clamped to 1.0 |
| **U-B130** | `disease_common_cold_severity` | `DiseaseType::CommonCold.base_severity()` | — | mortality≈0.001, duration=1 |
| **U-B131** | `disease_pandemic_severity` | `DiseaseType::Pandemic.base_severity()` | — | mortality≈0.30, duration=5 |
| **U-B132** | `disease_magical_plague_severity` | `DiseaseType::MagicalPlague.base_severity()` | — | mortality≈0.50 |
| **U-B133** | `disease_contagious_flu` | `SeasonalFlu.is_contagious()` | — | Returns true |
| **U-B134** | `disease_not_contagious_waterborne` | `Waterborne.is_contagious()` | — | Returns false |
| **U-B135** | `disease_suitability_wet_biomes` | `Waterborne.biome_suitability(CoastalWetland)` | — | Returns 2.0 (higher than land biomes) |
| **U-B136** | `disaster_drought_affects_food` | `Drought.affects_food()` | — | Returns true |
| **U-B137** | `disaster_famine_severity` | `Famine.base_severity()` | — | mortality≈0.30 |
| **U-B138** | `disaster_volcanic_common_biome` | `Volcanic.common_biomes()` | — | Contains VolcanicLandscape |
| **U-B139** | `carrying_capacity_ocean_zero` | `calculate_carrying_capacity(Ocean, 1000)` | — | Returns 0 |
| **U-B140** | `carrying_capacity_tropical_rainforest` | High-capacity biome | — | Returns near maximum (7000) |

#### Events Module (`src/events/`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B150** | `event_builder_minimal` | `EventBuilder::new("Test").build(world_id)` | World UUID | Builds with only name + world_id |
| **U-B151** | `event_builder_full` | EventBuilder with all fields set | World UUID, time, location, participants | All fields on resulting Event populated |
| **U-B152** | `event_builder_settlement_founded` | `Event::settlement_founded(...)` | settlement UUID | Correct EventType set |
| **U-B153** | `event_timeline_add_event_sorted` | Add events at years 100, 300, 200 | EventTimeline, 3 events | Events retrievable in chronological order |
| **U-B154** | `event_timeline_get_range` | Range [150, 250] on timeline with events at 100, 200, 300 | — | Returns only event at year 200 |
| **U-B155** | `event_timeline_stats_count` | `get_stats()` on timeline with known events | — | event_count matches added events |
| **U-B156** | `event_effect_population_loss` | Construct `EventEffect::PopulationLoss(...)` | — | Variant constructed correctly |
| **U-B157** | `event_effect_war_declared` | Construct `EventEffect::WarDeclared(...)` | — | Variant constructed correctly |
| **U-B158** | `probability_engine_returns_0_to_1` | Various EventContext inputs | EventContext | Result clamped to [0.0, 1.0] |
| **U-B159** | `probability_engine_context_affects` | Two different EventContexts | Same EventType | Returns different probabilities |

#### Faction Module (`src/faction.rs`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B170** | `faction_registry_new` | `FactionRegistry::new()` | — | Starts empty |
| **U-B171** | `faction_registry_register_valid` | Add valid Faction | — | Succeeds, `get_faction()` returns Some |
| **U-B172** | `faction_registry_register_duplicate` | Register same ID twice | — | Returns `FactionError::DuplicateId` |
| **U-B173** | `faction_registry_get_faction` | Lookup after register | UUID | Returns Some, fields match |
| **U-B174** | `faction_registry_get_unknown` | Lookup unregistered UUID | — | Returns None |
| **U-B175** | `faction_registry_update` | Update existing faction | FactionRegistry with member | Update succeeds, `get_faction()` returns new value |
| **U-B176** | `faction_registry_save_load_roundtrip` | Save to TOML, reload | FactionRegistry with factions | Loaded registry equivalent to original |
| **U-B177** | `turn_phase_cycle` | Sequence of `.next()` calls | — | Income→Maintenance→Action→News→Income cycles |
| **U-B178** | `turn_phase_name` | Each TurnPhase variant | — | `name()` returns correct string |

#### API Module (`src/api/`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-B190** | `normalize_strips_world_prefix` | `"world:abc-123"` | — | Returns `"abc-123"` |
| **U-B191** | `normalize_preserves_plain_uuid` | `"abc-123"` | — | Returns `"abc-123"` unchanged |
| **U-B192** | `app_state_new_creates_storage` | `AppState::new()` | — | Returns Ok(AppState) with StorageManager |

---

### 1B. Frontend Unit Tests (TypeScript — Vitest)

#### Dashboard Service (`tests/dashboardService.test.ts` — existing, working)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-F001** | `fetchWorldStats success` | API returns valid `WorldStatsResponse` | `fetch` mock, 200 response | Returns `WorldStateMetrics` |
| **U-F002** | `fetchWorldStats fallback` | `fetch` throws network error | `fetch` mock, throws | Returns mock stats from `generateMockStats()` |
| **U-F003** | `transformToWorldStateMetrics` | `WorldStatsResponse` with nested data | — | Correctly maps to `WorldStateMetrics` interface |
| **U-F004** | `scarcity_calculation_abundant` | Resource ratio > 0.5 | — | Returns "abundant" |
| **U-F005** | `scarcity_calculation_common` | Resource ratio 0.25–0.5 | — | Returns "common" |
| **U-F006** | `scarcity_calculation_rare` | Resource ratio 0.1–0.25 | — | Returns "rare" |
| **U-F007** | `scarcity_calculation_critical` | Resource ratio < 0.1 | — | Returns "critical" |
| **U-F008** | `getDefaultWorldMetrics` | Called with no data | — | All numeric fields = 0, arrays empty |

#### MapViewer Class (`src/terrain/MapViewer.ts`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-F020** | `constructor_with_valid_canvas` | Mock canvas with `getContext('2d')` | — | `ctx` set, `onReady` called |
| **U-F021** | `constructor_with_invalid_canvas` | Canvas that returns null | — | `onError` called with message |
| **U-F022** | `fitToWorld_calculates_zoom` | MapData with dimensions 1000×800, canvas 500×400 | — | zoom ≈ 0.36 (min(500/1000, 400/800)*0.9) |
| **U-F023** | `setMapData_triggers_fit` | New MapData | — | `fitToWorld()` called, then `render()` via rAF |
| **U-F024** | `drawMap_all_layers` | MapData with biomes, polygons, resources, entities | — | 4 draw call batches executed |
| **U-F025** | `drawPolygons_with_holes` | Polygon with `holes: [[x,y]...]` | — | Holes rendered as dark fill |
| **U-F026** | `resourceColorCoding_iron` | ResourceLocation with type "iron" | — | Circle drawn with brown color |
| **U-F027** | `resourceColorCoding_gold` | ResourceLocation with type "gold" | — | Circle drawn with yellow color |
| **U-F028** | `entityColorCoding_city` | GeographicEntity with type "city" | — | Circle drawn with orange color |
| **U-F029** | `worldToScreen_conversion` | `{x:100, y:200}`, zoom=2, pan=(10,20) | — | Returns `{x:210, y:420}` |
| **U-F030** | `screenToWorld_inverse` | `{x:210, y:420}`, zoom=2, pan=(10,20) | — | Returns `{x:100, y:200}` |
| **U-F031** | `mouseWheel_zoom_in` | wheel deltaY=-100 | viewport.zoom=1.0 | Zoom increases to 1.1, clamped to [0.1, 10] |
| **U-F032** | `mouseWheel_zoom_out` | wheel deltaY=+100 | viewport.zoom=1.0 | Zoom decreases to 0.9 |
| **U-F033** | `mouseDrag_panning` | mousedown at (0,0), mousemove to (50,50) | — | viewport.x/y updated, `render()` called |
| **U-F034** | `touchDrag_single` | touchmove with single touch | — | Same pan behavior as mouse drag |
| **U-F035** | `destroy_cancels_animation` | Any state | — | `cancelAnimationFrame` called, id set to null |

#### MapData Types (`src/terrain/MapData.ts`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-F040** | `mapPolygon_valid_structure` | 4 vertices forming closed polygon | — | Rendered as closed shape |
| **U-F041** | `mapPolygon_holes_rendered` | Polygon with holes array | — | Holes area dark-filled |
| **U-F042** | `biome_rgb_to_string` | Biome color `[30, 100, 60]` | — | Canvas receives `"rgb(30, 100, 60)"` |
| **U-F043** | `resourceLocation_magnitude_radius` | magnitude=3 | — | Radius = `3*4+4 = 16` pixels |
| **U-F044** | `geographicEntity_significance_size` | significance=0.5 | — | Marker size = 8 pixels |

#### Elevation/Color Mapping (`web/js/map-view.js`)

| ID | Test Name | Input | Mock/Setup | Success Criteria |
|----|-----------|-------|------------|------------------|
| **U-F050** | `elevationToColor_ocean` | elevation=0.1 | — | Below ocean threshold (<0.2) → ocean blue |
| **U-F051** | `elevationToColor_shallow` | elevation=0.23 | — | Shallow water blue (#4a90d9) |
| **U-F052** | `elevationToColor_beach` | elevation=0.28 | — | Beach tan (#c2b280) |
| **U-F053** | `elevationToColor_grass` | elevation=0.4 | — | Grass green (#7cb342) |
| **U-F054** | `elevationToColor_forest` | elevation=0.6 | — | Forest dark green (#2e7d32) |
| **U-F055** | `elevationToColor_mountain` | elevation=0.78 | — | Mountain gray (#757575) |
| **U-F056** | `elevationToColor_snow` | elevation=0.9 | — | Snow white (#f5f5f5) |
| **U-F057** | `renderTileMap_with_terrain` | Tile data with terrain types | mapData with tiles | Each tile colored per TERRAIN_COLORS |
| **U-F058** | `renderTileMap_elevation_fallback` | Tile with elevation but no terrain | mapData with elevation-only tiles | Colored from `elevationToColor()` |
| **U-F059** | `renderTileMap_no_data` | mapData.tiles is falsy | — | Dark background with "No map data" message |
| **U-F060** | `renderPolygonMap_ocean` | Polygon with `is_ocean: true` | — | Ocean blue fill (#1e3a5f) |
| **U-F061** | `renderPolygonMap_coastal` | Polygon with `is_coastal: true` | — | Beach tan fill (#c2b280) |
| **U-F062** | `overlay_elevation` | options.overlay='elevation' | mapData loaded | Semi-transparent black overlay rendered |
| **U-F063** | `overlay_resources` | options.overlay='resources' | mapData with resources | Gold resource circles rendered |
| **U-F064** | `overlay_political` | options.overlay='political' | mapData with factions | Faction-colored tiles with alpha |

---

## PART 2 — INTEGRATION TESTS

### 2A. Backend Integration Tests (Rust — `tests/` directory)

#### World Generation Pipeline (`tests/integration_world_generation.rs`)

| ID | Test Name | Steps | Success Criteria |
|----|-----------|-------|------------------|
| **I-B001** | `test_world_gen_terrain_grid` | Generate 64×64 world with seed=12345 | Grid dimensions correct, cell count = 64×64 |
| **I-B002** | `test_world_gen_ocean_coverage` | Generate world, call `.detect_ocean()` | Ocean ratio within [0.0, 1.0] |
| **I-B003** | `test_world_gen_land_coverage` | Generate world | Land ratio >= 0.0 |
| **I-B004** | `test_world_gen_biome_diversity` | Generate world, collect unique biomes | At least 3 different biome types on land |
| **I-B005** | `test_world_gen_elevation_variance` | Generate world | Elevation range > 0 (not flat) |
| **I-B006** | `test_world_gen_voronoi_polygons` | Generate, check PolygonGraph | Cell count matches expected |
| **I-B007** | `test_world_gen_tectonic_result` | Generate with `enable_tectonics=true` | TectonicResult populated |
| **I-B008** | `test_world_gen_timing` | Generate world | Completes within 120 seconds |

#### Full Pipeline Save/Load (`tests/integration_world_generation.rs`)

| ID | Test Name | Steps | Success Criteria |
|----|-----------|-------|------------------|
| **I-B010** | `test_full_pipeline_save_load` | Generate → `save_world_package` → `load_world` | world.name, world.seed, id all preserved |
| **I-B011** | `test_api_crud_lifecycle` | POST /worlds → GET /worlds → GET /worlds/{id} → DELETE → GET 404 | All status codes correct, data consistent |
| **I-B012** | `test_population_simulation_integration` | Add settlements → advance 100 years | Population increased, society transitions occurred |
| **I-B013** | `test_event_timeline_integration` | Add events at years 100/200/300 → range query [150,250] | Only year 200 event returned |
| **I-B014** | `test_species_name_generation_integration` | Generate 10 names per species | All names non-empty, correct suffix per species |

#### API Endpoint Tests (`tests/api_endpoints_test.rs`)

| ID | Test Name | Method | Path | Body/Params | Success Criteria |
|----|-----------|--------|------|------------|------------------|
| **I-B020** | `health_returns_ok` | GET | `/health` | — | 200, `{"status":"ok"}` |
| **I-B021** | `list_worlds_empty` | GET | `/api/v1/worlds` | — | 200, `[]` |
| **I-B022** | `list_worlds_pagination` | GET | `/api/v1/worlds?limit=5&offset=0` | — | 200, respects limit/offset |
| **I-B023** | `list_worlds_invalid_sort` | GET | `/api/v1/worlds?sort_by=invalid` | — | 400 |
| **I-B024** | `create_world_valid` | POST | `/api/v1/worlds` | `{"name":"Test"}` | 201, returns world with id |
| **I-B025** | `create_world_missing_name` | POST | `/api/v1/worlds` | `{}` | 400 |
| **I-B026** | `get_world_valid_uuid` | GET | `/api/v1/worlds/{uuid}` | Valid UUID | 200, world data returned |
| **I-B027** | `get_world_invalid_uuid` | GET | `/api/v1/worlds/not-a-uuid` | — | 400 |
| **I-B028** | `delete_world_success` | DELETE | `/api/v1/worlds/{uuid}` | Existing world | 204, deleted |
| **I-B029** | `delete_world_not_found` | DELETE | `/api/v1/worlds/{uuid}` | Non-existent | 404 |
| **I-B030** | `trigger_generation` | POST | `/api/v1/worlds/{uuid}/generate` | — | 200, phase becomes "generating" |
| **I-B031** | `get_map_data` | GET | `/api/v1/worlds/{uuid}/map` | lod=1 | 200, polygon array present |
| **I-B032** | `get_timeline` | GET | `/api/v1/worlds/{uuid}/timeline` | — | 200 |
| **I-B033** | `get_events` | GET | `/api/v1/worlds/{uuid}/events` | — | 200 |
| **I-B034** | `get_history` | GET | `/api/v1/worlds/{uuid}/history` | — | 200 |
| **I-B035** | `get_figures` | GET | `/api/v1/worlds/{uuid}/figures` | — | 200 |
| **I-B036** | `get_societies` | GET | `/api/v1/worlds/{uuid}/societies` | — | 200 |
| **I-B037** | `get_planet` | GET | `/api/v1/worlds/{uuid}/planet` | — | 200 |
| **I-B038** | `get_tectonics` | GET | `/api/v1/worlds/{uuid}/tectonics` | — | 200 |
| **I-B039** | `get_artifacts` | GET | `/api/v1/worlds/{uuid}/artifacts` | — | 200 |
| **I-B040** | `get_cataclysms` | GET | `/api/v1/worlds/{uuid}/cataclysms` | — | 200 |
| **I-B041** | `get_wonders` | GET | `/api/v1/worlds/{uuid}/wonders` | — | 200 |
| **I-B042** | `get_resources` | GET | `/api/v1/worlds/{uuid}/resources` | — | 200 |
| **I-B043** | `get_disasters` | GET | `/api/v1/worlds/{uuid}/disasters` | — | 200 |
| **I-B044** | `species_list` | GET | `/api/v1/species` | — | 200, array of species |
| **I-B045** | `export_json` | GET | `/api/v1/worlds/{uuid}/export.json` | — | 200, valid JSON |
| **I-B046** | `export_data` | GET | `/api/v1/worlds/{uuid}/export` | — | 200 |

#### Export Endpoint Tests (`tests/export_endpoint_test.rs`)

| ID | Test Name | Steps | Success Criteria |
|----|-----------|-------|------------------|
| **I-B050** | `test_export_json_returns_valid` | GET /export.json | Valid JSON with world data |
| **I-B051** | `test_export_with_all_components` | GET /export | Contains all world components |
| **I-B052** | `test_export_respects_format` | GET /export?format=json | Correct format returned |

#### Serialization Tests (`src/serialization_tests.rs`)

| ID | Test Name | Input | Success Criteria |
|----|-----------|-------|------------------|
| **I-B060** | `world_json_roundtrip` | World with all fields → JSON → parse | All fields preserved |
| **I-B061** | `world_toml_roundtrip` | World → TOML → parse | All fields preserved |
| **I-B062** | `entity_id_serialization` | EntityId → JSON → parse | Roundtrips correctly |
| **I-B063** | `historical_time_serialization` | HistoricalTime → JSON → parse | Exact/approximate flag preserved |

---

### 2B. Frontend Integration Tests (Vitest)

| ID | Test Name | Steps | Success Criteria |
|----|-----------|-------|------------------|
| **I-F001** | `dashboard_full_load_flow` | Mount Dashboard → load worlds → select world → show metrics | Loading → world cards → click → metrics displayed |
| **I-F002** | `map_component_lifecycle` | Mount → load map data → render → resize → destroy | Canvas renders, resize recalculates, destroy cleans up |
| **I-F003** | `timeline_filter_pipeline` | Load events → apply type filter → apply year filter → apply search → render | Filtered count correct at each stage |

---

## PART 3 — E2E / AUTOMATION TESTS (Playwright)

### 3A. API Smoke Tests

#### Full Endpoint Coverage (`smoke-test-WOR-638.spec.ts` pattern)

| ID | Test Name | Request | Expected |
|----|-----------|---------|----------|
| **E-001** | `health_returns_200` | GET `http://127.0.0.1:8080/health` | status 200, body.status === 'ok' |
| **E-002** | `create_world_returns_201` | POST `/api/v1/worlds` | 201 or 202, body.success |
| **E-003** | `list_worlds_returns_200` | GET `/api/v1/worlds` | 200, array |
| **E-004** | `poll_world_until_ready` | GET `/api/v1/worlds/{id}` in loop, 60s timeout | phase='ready' or throw on phase='error' |
| **E-005** | `get_world_details` | GET `/api/v1/worlds/{id}` | 200/404 |
| **E-006** | `get_planet` | GET `/api/v1/worlds/{id}/planet` | 200/400/404 |
| **E-007** | `get_map_polygons` | GET `/api/v1/worlds/{id}/map` | 200/400/404, polygon array |
| **E-008** | `get_history` | GET `/api/v1/worlds/{id}/history` | 200/400/404 |
| **E-009** | `get_history_events` | GET `/api/v1/worlds/{id}/history/events` | 200/400/404 |
| **E-010** | `get_figures_list` | GET `/api/v1/worlds/{id}/figures` | 200/400/404 |
| **E-011** | `get_single_figure` | GET `/api/v1/worlds/{id}/figures/fig-0` | 200/400/404 |
| **E-012** | `get_settlements` | GET `/api/v1/worlds/{id}/settlements` | 200/400/404 |
| **E-013** | `get_settlements_map` | GET `/api/v1/worlds/{id}/settlements/map` | 200/400/404 |
| **E-014** | `get_resources_summary` | GET `/api/v1/worlds/{id}/resources/summary` | 200/400/404 |
| **E-015** | `get_disasters` | GET `/api/v1/worlds/{id}/disasters` | 200/400/404 |
| **E-016** | `get_artifacts` | GET `/api/v1/worlds/{id}/artifacts?limit=5` | 200/400/404 |
| **E-017** | `get_export` | GET `/api/v1/worlds/{id}/export` | 200/400/404 |
| **E-018** | `get_export_json` | GET `/api/v1/worlds/{id}/export.json` | 200/400/404 |
| **E-019** | `delete_world` | DELETE `/api/v1/worlds/{id}` | 200/204/400/404 |

### 3B. Frontend UI Tests

#### Landing Page (`frontend-smoke-tests.spec.ts`)

| ID | Test Name | Action | Expected |
|----|-----------|--------|----------|
| **E-020** | `index_page_200` | GET `/` | HTTP 200 |
| **E-021** | `hero_title_visible` | Check `.hero h2` | Text contains "Choose Your World" |
| **E-022** | `create_button_visible` | Check for create/generate button | Button present |
| **E-023** | `server_status_indicator` | Check `#server-status` | Element present |

#### World Detail Page (`frontend-smoke-tests.spec.ts` + world.html)

| ID | Test Name | Action | Expected |
|----|-----------|--------|----------|
| **E-030** | `world_page_loads` | GET `/world.html?id=test-id` | Status 200 |
| **E-031** | `header_elements` | Check `#page-title`, `#server-status`, `.back-link` | All visible |
| **E-032** | `tab_buttons_exist` | Check 4 tab buttons | overview, map, timeline, dashboard tabs present |
| **E-033** | `tab_panels_exist` | Check 4 panels | `#panel-overview`, `#panel-map`, `#panel-timeline`, `#panel-dashboard` in DOM |
| **E-034** | `map_canvas_exists` | Check `#world-map` canvas | Canvas element present |
| **E-035** | `timeline_content_exists` | Check `#timeline-content` | Element present |
| **E-036** | `dashboard_stats_grid` | Click dashboard tab, check `#stats-grid` | Element present |

#### Tab Navigation Cycle

| ID | Test Name | Action | Expected |
|----|-----------|--------|----------|
| **E-040** | `all_tabs_clickable` | Loop through all 4 tabs clicking each | Each tab clickable without error |
| **E-041** | `map_tab_renders` | Click map tab → wait | Screenshot captured (optional) |
| **E-042** | `timeline_tab_renders` | Click timeline tab → wait | Timeline content renders |
| **E-043** | `dashboard_tab_renders` | Click dashboard tab → wait | Dashboard stats visible |

### 3C. Create World Flow

| ID | Test Name | Action | Expected |
|----|-----------|--------|----------|
| **E-050** | `create_modal_opens` | Click create button | Modal gets `active` class |
| **E-051** | `create_form_submit` | Fill form, click create | POST to API → modal closes → worlds reload |
| **E-052** | `create_api_error_handled` | API returns error | Alert shown, button re-enabled, form remains open |

### 3D. Console Error Detection

| ID | Test Name | Page | Expected |
|----|-----------|------|----------|
| **E-060** | `index_page_no_errors` | `/` after 2s wait | Zero console errors (filter: `ERR_CONNECTION_REFUSED`, `Failed to load resource`, `favicon`) |
| **E-061** | `world_page_no_errors` | `/world.html?id=...` | Zero errors on load |
| **E-062** | `map_view_no_errors` | After map tab click | Zero errors during render |
| **E-063** | `timeline_view_no_errors` | After timeline tab click | Zero errors during event load |
| **E-064** | `dashboard_view_no_errors` | After dashboard tab click | Zero errors during stats load |
| **E-065** | `modal_no_errors` | Open/close create modal | No errors during interaction |

### 3E. Map Rendering (Canvas Verification via screenshot)

| ID | Test Name | Action | Expected |
|----|-----------|--------|----------|
| **E-070** | `map_renders_polygons` | Load world with polygons | Screenshot shows colored polygon regions |
| **E-071** | `map_overlay_elevation` | Set overlay='elevation' | Screenshot shows elevation shading |
| **E-072** | `map_overlay_resources` | Set overlay='resources' | Screenshot shows resource dots |
| **E-073** | `map_zoom_in` | Mouse wheel zoom in | Screenshot shows zoomed map |
| **E-074** | `map_zoom_out` | Mouse wheel zoom out | Screenshot shows zoomed-out map |
| **E-075** | `map_pan` | Mouse drag | Screenshot shows different view area |

---

## SUMMARY TABLE

| Category | Count | Priority |
|----------|-------|----------|
| Backend Unit Tests (Rust) | ~80 | High |
| Frontend Unit Tests (TypeScript) | ~40 | High |
| Backend Integration Tests (Rust) | ~40 | High |
| Frontend Integration Tests (Vitest) | 3 | Medium |
| E2E / Automation (Playwright) | ~75 | High |
| **Total** | **~238 test cases** | |

---

## TESTING INFRASTRUCTURE

### Commands (from `justfile`)

```bash
just test           # All Rust tests (falls back to Docker)
just test-unit      # cargo test --lib
just test-integration  # integration_world_generation + phase tests
cargo test          # All tests
cargo test --lib    # Unit tests only
cargo test tests/   # Integration tests
npx playwright test --project=chromium  # E2E
npm run build       # Web build
```

### Key Dependencies

| Tool | Version | Use |
|------|---------|-----|
| `proptest` | 1.2 | Property-based testing (Rust) |
| `tempfile` | 3.10 | Filesystem isolation |
| `tokio` | 1.x | Async test runtime |
| `tower` | 0.6 | ServiceExt for handler testing |
| `vitest` | (via package.json) | TS unit tests |
| `@playwright/test` | (via package.json) | E2E automation |
| `puppeteer` | 24.42 | Web build (headless Chrome) |

### Known Issue Areas (from QA reports — test coverage recommended)

| Issue | Severity | Tests to Add |
|-------|----------|--------------|
| API ID format inconsistency (`world:` prefix) | HIGH | U-B190, U-B191, I-B026 |
| Path case bug (storage) | HIGH | U-B055, U-B056, I-B011 |
| Slow world generation (blocking) | HIGH | I-B008 (timing test with timeout) |
| DELETE endpoint 405 | MEDIUM | I-B028, I-B029, E-019 |
| `/history/events` 404 | MEDIUM | I-B009 |
| `/figures/:id` 404 | MEDIUM | I-B011, E-011 |
| Export endpoints 404 | MEDIUM | I-B017, I-B018, I-B045, I-B046 |
| Visual clipping (status text) | LOW | E-073, E-074 (screenshot review) |

---

*Generated via subagent exploration of all source modules, test files, QA reports, and project documentation.*