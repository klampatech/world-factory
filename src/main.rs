// World Factory CLI
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "world-factory")]
#[command(about = "World Factory - Procedural World Generation Engine", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = false)]
    server: bool,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new world
    Generate {
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        #[arg(short, long, default_value_t = 128)]
        width: u32,
        /// Height of the world grid
        #[arg(short = 'y', long, default_value_t = 128)]
        height: u32,
        /// Export world to a custom directory instead of default storage
        #[arg(long)]
        export_to: Option<String>,
    },
    /// List all saved worlds
    List,
    /// Load and display a saved world
    Load {
        /// Path to .wfw file or world ID
        path: String,
    },
    /// Show package metadata without loading full data
    Inspect {
        /// Path to .wfw file
        path: String,
    },
}

#[cfg(feature = "api")]
mod server {
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    pub async fn start(port: u16) {
        let app_state = world_factory::api::AppState::new().expect("Failed to create app state");
        let app = world_factory::api::create_router().with_state(app_state);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        println!("Starting World Factory API server on http://{}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

#[cfg(not(feature = "api"))]
mod server {
    pub fn start(_port: u16) {
        eprintln!("API support not compiled in. Build with --features api");
    }
}

fn main() {
    let cli = Cli::parse();
    if cli.server {
        println!("World Factory - API Server Mode");
        #[cfg(feature = "api")]
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(server::start(cli.port));
        }
        #[cfg(not(feature = "api"))]
        {
            eprintln!("Error: API feature not enabled");
            std::process::exit(1);
        }
    } else {
        match cli.command {
            Some(Commands::Generate {
                seed,
                width,
                height,
                export_to,
            }) => run_terrain_generator(seed, width, height, export_to),
            Some(Commands::List) => list_worlds(),
            Some(Commands::Load { path }) => load_world(&path),
            Some(Commands::Inspect { path }) => inspect_world(&path),
            None => run_terrain_generator(42, 128, 128, None),
        }
    }
}

fn run_terrain_generator(seed: u64, width: u32, height: u32, export_to: Option<String>) {
    use world_factory::packaging::{save_world_package, WorldPackage};
    use world_factory::storage::{StorageConfig, StorageManager};
    use world_factory::terrain::{TerrainConfig, TerrainGenerator, TerrainLayer};
    use world_factory::types::World;

    println!("World Factory - Procedural World Generator");
    println!("=========================================\n");

    let world = World::new(format!("World-{}", seed), seed);
    let world_id = format!("world:{}", world.id);

    let config = TerrainConfig {
        seed,
        width,
        height,
        ..Default::default()
    };

    println!("Generating world with seed {}...", config.seed);

    let mut generator = TerrainGenerator::new(config);
    let grid = generator.generate(TerrainLayer::Full);

    println!(
        "Generated terrain grid: {}x{} cells",
        grid.dimensions().0,
        grid.dimensions().1
    );
    println!("Memory usage: {} bytes", grid.memory_usage());

    println!("\nSample biomes:");
    for y in [0, 32, 64, 96] {
        for x in [0, 32, 64, 96] {
            if let Some(cell) = grid.get(x, y) {
                println!(
                    "  ({:3}, {:3}): biome={}, height={}m",
                    x,
                    y,
                    cell.biome(),
                    cell.height() as u32
                );
            }
        }
    }

    let package = WorldPackage {
        world,
        regions: Vec::new(),
        settlements: Vec::new(),
        persons: Vec::new(),
        events: Vec::new(),
        timelines: Vec::new(),
        terrain: None,
    };

    let storage = match export_to {
        Some(ref custom_dir) => {
            // Create a custom storage config pointing to the specified directory
            let config = StorageConfig {
                base_dir: Some(std::path::PathBuf::from(custom_dir)),
                create_dirs: true,
                ..Default::default()
            };
            StorageManager::new(config)
                .expect("Failed to create storage manager for custom directory")
        }
        None => StorageManager::default_manager().expect("Failed to get storage manager"),
    };
    let package_path = storage.world_package_path(&world_id);

    if let Some(parent) = package_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create world directory");
    }

    save_world_package(&package, &package_path).expect("Failed to save world package");

    println!("\nWorld saved to: {}", package_path.display());
    println!("World ID: {}", world_id);
    println!("\nWorld generation complete!");
}

fn list_worlds() {
    use world_factory::packaging::inspect_package;
    use world_factory::storage::StorageManager;

    println!("World Factory - Saved Worlds");
    println!("==============================\n");

    let storage = match StorageManager::default_manager() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to get storage manager: {}", e);
            std::process::exit(1);
        }
    };

    let generated_dir = storage.generated_dir();
    if !generated_dir.exists() {
        println!("No worlds saved yet.");
        println!(
            "Generated worlds are stored in: {}",
            generated_dir.display()
        );
        return;
    }

    let mut worlds: Vec<_> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&generated_dir) {
        for entry in entries.flatten() {
            let world_dir = entry.path();
            if world_dir.is_dir() {
                let package_path = world_dir.join("world.wfw");
                if package_path.exists() {
                    match inspect_package(&package_path) {
                        Ok(manifest) => {
                            worlds.push((package_path, manifest));
                        }
                        Err(_) => {
                            // Try to extract name from directory
                            if let Some(name) = world_dir.file_name().and_then(|n| n.to_str()) {
                                worlds.push((
                                    package_path.clone(),
                                    world_factory::packaging::PackageManifest {
                                        version: "unknown".to_string(),
                                        world_name: name.to_string(),
                                        seed: 0,
                                        created_at: String::new(),
                                        entries: Vec::new(),
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if worlds.is_empty() {
        println!("No worlds saved yet.");
        println!(
            "Generated worlds are stored in: {}",
            generated_dir.display()
        );
        return;
    }

    println!(
        "{:<40} {:>10} {:>12} {:>20}",
        "World Name", "Seed", "Version", "Path"
    );
    println!("{:-<40} {:-10} {:-12} {:-20}", "-", "-", "-", "-");

    for (path, manifest) in &worlds {
        let short_path = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(path.to_str().unwrap_or("?"));
        println!(
            "{:<40} {:>10} {:>12} {:>20}",
            manifest.world_name, manifest.seed, manifest.version, short_path
        );
    }

    println!("\nTotal: {} world(s)\n", worlds.len());
}

fn load_world(path_str: &str) {
    use world_factory::packaging::load_world;
    use world_factory::storage::StorageManager;

    // Resolve path: if it doesn't look like a path, look in generated dir
    let path = if std::path::Path::new(path_str).exists() {
        std::path::PathBuf::from(path_str)
    } else {
        // Treat as world ID and construct path
        let storage = match StorageManager::default_manager() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to get storage manager: {}", e);
                std::process::exit(1);
            }
        };
        storage.generated_dir().join(path_str).join("world.wfw")
    };

    println!("Loading world from: {}", path.display());

    let package = match load_world(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Failed to load world: {}", e);
            std::process::exit(1);
        }
    };

    println!("\nWorld Factory - World Details");
    println!("============================\n");
    println!("Name: {}", package.world.name);
    println!("ID: {}", package.world.id);
    println!("Seed: {}", package.world.seed);
    if let Some(desc) = &package.world.description {
        println!("Description: {}", desc);
    }
    println!("Created: {}", package.world.created_at);
    println!("\nContents:");
    println!("  Regions: {}", package.regions.len());
    println!("  Settlements: {}", package.settlements.len());
    println!("  Persons: {}", package.persons.len());
    println!("  Events: {}", package.events.len());
    println!("  Timelines: {}", package.timelines.len());
    println!(
        "  Terrain: {}",
        if package.terrain.is_some() {
            "Present"
        } else {
            "None"
        }
    );
    println!();
}

fn inspect_world(path_str: &str) {
    use world_factory::packaging::inspect_package;

    let path = std::path::PathBuf::from(path_str);

    if !path.exists() {
        eprintln!("Error: File not found: {}", path.display());
        std::process::exit(1);
    }

    let manifest = match inspect_package(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Failed to inspect package: {}", e);
            std::process::exit(1);
        }
    };

    println!("World Factory - Package Inspector");
    println!("==================================\n");
    println!("File: {}", path.display());
    println!("Version: {}", manifest.version);
    println!("World Name: {}", manifest.world_name);
    println!("Seed: {}", manifest.seed);
    println!("Created: {}", manifest.created_at);
    println!("\nPackage Entries:");
    for entry in &manifest.entries {
        println!("  [{}] {} ({})", entry.entry_type, entry.path, entry.size);
    }
    println!();
}
