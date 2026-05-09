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
    Generate {
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        #[arg(short, long, default_value_t = 128)]
        width: u32,

        /// Height of the world grid
        #[arg(short = 'y', long, default_value_t = 128)]
        height: u32,
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
            Some(Commands::Generate { seed, width, height }) => run_terrain_generator(seed, width, height),
            None => run_terrain_generator(42, 128, 128),
        }
    }
}

fn run_terrain_generator(seed: u64, width: u32, height: u32) {
    use world_factory::storage::StorageManager;
    use world_factory::terrain::{TerrainConfig, TerrainGenerator, TerrainLayer};
    use world_factory::types::World;
    use world_factory::packaging::{save_world_package, WorldPackage};

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

    println!("Generated terrain grid: {}x{} cells", grid.dimensions().0, grid.dimensions().1);
    println!("Memory usage: {} bytes", grid.memory_usage());

    println!("\nSample biomes:");
    for y in [0, 32, 64, 96] {
        for x in [0, 32, 64, 96] {
            if let Some(cell) = grid.get(x, y) {
                println!("  ({:3}, {:3}): biome={}, height={}m", x, y, cell.biome(), cell.height() as u32);
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

    let storage = StorageManager::default_manager().expect("Failed to get storage manager");
    let package_path = storage.world_package_path(&world_id);

    if let Some(parent) = package_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create world directory");
    }

    save_world_package(&package, &package_path).expect("Failed to save world package");

    println!("\nWorld saved to: {}", package_path.display());
    println!("World ID: {}", world_id);
    println!("\nWorld generation complete!");
}
