//! World Factory CLI
//! 
//! Command-line interface for world generation.
//! Can run as CLI terrain generator or as API server.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "world-factory")]
#[command(about = "World Factory - Procedural World Generation Engine", long_about = None)]
struct Cli {
    /// Run the API server instead of CLI mode
    #[arg(short, long, default_value_t = false)]
    server: bool,
    
    /// Port for API server (default: 3000)
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a world using CLI mode
    Generate {
        /// Seed for world generation
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Width of the world grid
        #[arg(short, long, default_value_t = 128)]
        width: u32,
        
        /// Height of the world grid  
        #[arg(short, long, default_value_t = 128)]
        height: u32,
    },
}

#[cfg(feature = "api")]
mod server {
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    pub async fn start(port: u16) {
        // Initialize tracing for logging
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()))
            .init();
        
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
        // Run as API server
        println!("World Factory - API Server Mode");
        println!("================================\n");
        
        #[cfg(feature = "api")]
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(server::start(cli.port));
        }
        
        #[cfg(not(feature = "api"))]
        {
            eprintln!("Error: API feature not enabled. Rebuild with --features api");
            std::process::exit(1);
        }
    } else {
        // Run as CLI terrain generator
        match cli.command {
            Some(Commands::Generate { seed, width, height }) => {
                run_terrain_generator(seed, width, height);
            },
            None => {
                // Default to terrain generation with defaults
                run_terrain_generator(42, 128, 128);
            }
        }
    }
}

fn run_terrain_generator(seed: u64, width: u32, height: u32) {
    use world_factory::terrain::{TerrainGenerator, TerrainConfig, TerrainLayer};
    
    println!("World Factory - Procedural World Generator");
    println!("=========================================\n");
    
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
    
    // Sample some cells
    println!("\nSample biomes:");
    for y in [0, 32, 64, 96] {
        for x in [0, 32, 64, 96] {
            if let Some(cell) = grid.get(x, y) {
                let biome = match cell.biome() {
                    0 => "Tropical Rainforest",
                    1 => "Tropical Seasonal Forest",
                    2 => "Tropical Savanna",
                    3 => "Subtropical Desert",
                    4 => "Temperate Forest",
                    5 => "Boreal Taiga",
                    6 => "Tundra",
                    7 => "Arctic",
                    8 => "Hot Desert",
                    9 => "Ocean",
                    _ => "Unknown",
                };
                println!("  ({:3}, {:3}): {} at {}m", x, y, biome, cell.height() as u32);
            }
        }
    }
    
    println!("\nWorld generation complete!");
}
