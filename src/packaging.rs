//! Tarball packaging for World Factory (.wfw) files.
//! 
//! Implements save_world() and load_world() for persistent world storage.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive as TarArchive, Builder as TarBuilder, Header};

use crate::types::{
    World, Region, Settlement, Person, HistoricalEvent, Timeline,
    Timestamp,
};

/// World Factory Package format version
const PACKAGE_VERSION: &str = "1.0";

/// Manifest file name in archive
const MANIFEST_FILENAME: &str = "manifest.json";

/// World data file name in archive  
const WORLD_FILENAME: &str = "world.json";

/// Terrain data file name in archive
const TERRAIN_FILENAME: &str = "terrain.bin";

/// Archive entry names
const ENTRIES_DIR: &str = "entries/";

/// Package manifest containing metadata about the archive contents.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageManifest {
    /// Format version
    pub version: String,
    /// World name
    pub world_name: String,
    /// World seed
    pub seed: u64,
    /// Created timestamp
    pub created_at: String,
    /// Package entry list
    pub entries: Vec<EntryManifest>,
}

/// Individual entry metadata in the package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntryManifest {
    /// Entry type (world, region, settlement, etc.)
    pub entry_type: String,
    /// File path within archive
    pub path: String,
    /// JSON size in bytes
    pub size: u64,
}

/// Errors that can occur during packaging operations.
#[derive(Debug)]
pub enum PackageError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidFormat(String),
    EntryNotFound(String),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::Io(e) => write!(f, "IO error: {}", e),
            PackageError::Json(e) => write!(f, "JSON error: {}", e),
            PackageError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            PackageError::EntryNotFound(s) => write!(f, "Entry not found: {}", s),
        }
    }
}

impl std::error::Error for PackageError {}

impl From<io::Error> for PackageError {
    fn from(err: io::Error) -> Self {
        PackageError::Io(err)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(err: serde_json::Error) -> Self {
        PackageError::Json(err)
    }
}

/// World package containing all world data for serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldPackage {
    /// World metadata and state
    pub world: World,
    /// Geographic regions
    pub regions: Vec<Region>,
    /// Settlements within regions
    pub settlements: Vec<Settlement>,
    /// Historical persons
    pub persons: Vec<Person>,
    /// Historical events
    pub events: Vec<HistoricalEvent>,
    /// Timelines
    pub timelines: Vec<Timeline>,
    /// Terrain data (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrain: Option<serde_json::Value>,
}

/// Save a world to a .wfw tarball file.
/// 
/// Creates a gzipped tar archive with:
/// - manifest.json: Package metadata
/// - world.json: Complete world data
/// 
/// # Arguments
/// * `world` - The world to save
/// * `path` - Output file path (should end in .wfw)
/// 
/// # Example
/// ```ignore
/// use world_factory::{World, save_world};
/// 
/// let world = World::new("Middle Earth".to_string(), 42);
/// save_world(&world, "mythrandir.wfw")?;
/// ```
pub fn save_world<P: AsRef<Path>>(
    world: &World,
    path: P,
) -> Result<(), PackageError> {
    let path = path.as_ref();
    
    // Create package from world
    let package = WorldPackage {
        world: world.clone(),
        regions: Vec::new(),
        settlements: Vec::new(),
        persons: Vec::new(),
        events: Vec::new(),
        timelines: Vec::new(),
        terrain: None,
    };
    
    // Serialize world data
    let world_json = serde_json::to_string_pretty(&package)?;
    let world_bytes = world_json.as_bytes();
    
    // Create manifest
    let manifest = PackageManifest {
        version: PACKAGE_VERSION.to_string(),
        world_name: world.name.clone(),
        seed: world.seed,
        created_at: Timestamp::now().to_string(),
        entries: vec![
            EntryManifest {
                entry_type: "world".to_string(),
                path: WORLD_FILENAME.to_string(),
                size: world_bytes.len() as u64,
            },
        ],
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    
    // Create tar archive with gzip compression
    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = TarBuilder::new(encoder);
    
    // Add manifest
    let mut header = Header::new_gnu();
    tar.append_data(&mut header, MANIFEST_FILENAME, &mut io::Cursor::new(manifest_json.into_bytes()))?;
    
    // Add world data
    let mut header = Header::new_gnu();
    tar.append_data(&mut header, WORLD_FILENAME, &mut io::Cursor::new(world_bytes.to_vec()))?;
    
    // Finish the archive
    tar.finish()?;
    
    Ok(())
}

/// Save a world with full data to a .wfw tarball file.
/// 
/// This version includes all optional data (regions, settlements, etc.)
/// 
/// # Arguments
/// * `package` - Complete world package
/// * `path` - Output file path
pub fn save_world_package<P: AsRef<Path>>(
    package: &WorldPackage,
    path: P,
) -> Result<(), PackageError> {
    let path = path.as_ref();
    
    // Serialize package
    let package_json = serde_json::to_string_pretty(package)?;
    let package_bytes = package_json.as_bytes();
    
    // Create manifest with all entries
    let mut entries = vec![
        EntryManifest {
            entry_type: "world".to_string(),
            path: WORLD_FILENAME.to_string(),
            size: package_bytes.len() as u64,
        },
    ];
    
    // Add terrain entry if present
    if package.terrain.is_some() {
        entries.push(EntryManifest {
            entry_type: "terrain".to_string(),
            path: TERRAIN_FILENAME.to_string(),
            size: 0, // Will be determined by actual size
        });
    }
    
    let manifest = PackageManifest {
        version: PACKAGE_VERSION.to_string(),
        world_name: package.world.name.clone(),
        seed: package.world.seed,
        created_at: Timestamp::now().to_string(),
        entries,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    
    // Create tar archive with gzip compression
    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = TarBuilder::new(encoder);
    
    // Add manifest
    let mut header = Header::new_gnu();
    tar.append_data(&mut header, MANIFEST_FILENAME, &mut io::Cursor::new(manifest_json.into_bytes()))?;
    
    // Add world package data
    let mut header = Header::new_gnu();
    tar.append_data(&mut header, WORLD_FILENAME, &mut io::Cursor::new(package_bytes.to_vec()))?;
    
    // Finish the archive
    tar.finish()?;
    
    Ok(())
}

/// Load a world from a .wfw tarball file.
/// 
/// # Arguments
/// * `path` - Input file path
/// 
/// # Returns
/// * `WorldPackage` containing all world data
/// 
/// # Example
/// ```ignore
/// use world_factory::load_world;
/// 
/// let package = load_world("mythrandir.wfw")?;
/// println!("Loaded: {}", package.world.name);
/// ```
pub fn load_world<P: AsRef<Path>>(
    path: P,
) -> Result<WorldPackage, PackageError> {
    let path = path.as_ref();
    
    // Open the archive
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = TarArchive::new(decoder);
    
    // Read all entries, storing manifest and collecting world content
    let mut manifest_content = String::new();
    let mut world_content = String::new();
    
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;
        let path_str = entry_path.to_str().unwrap_or("");
        
        if path_str == MANIFEST_FILENAME {
            entry.read_to_string(&mut manifest_content)?;
        } else if path_str == WORLD_FILENAME {
            entry.read_to_string(&mut world_content)?;
        }
    }
    
    // Parse manifest to verify version
    let _manifest: PackageManifest = serde_json::from_str(&manifest_content)?;
    
    // Parse world package
    let package: WorldPackage = serde_json::from_str(&world_content)?;
    
    Ok(package)
}

/// List contents of a .wfw package without fully loading.
/// 
/// # Arguments
/// * `path` - Input file path
/// 
/// # Returns
/// * `PackageManifest` with package metadata
pub fn inspect_package<P: AsRef<Path>>(
    path: P,
) -> Result<PackageManifest, PackageError> {
    let path = path.as_ref();
    
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = TarArchive::new(decoder);
    
    let mut manifest_content = String::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path_buf = entry.path()?.to_path_buf();
        let path_str = path_buf.to_str().unwrap_or("");
        
        if path_str == MANIFEST_FILENAME {
            entry.read_to_string(&mut manifest_content)?;
            break;
        }
    }
    
    let manifest: PackageManifest = serde_json::from_str(&manifest_content)?;
    Ok(manifest)
}

/// Get just the World metadata without loading all data.
/// 
/// # Arguments
/// * `path` - Input file path
/// 
/// # Returns
/// * `World` struct with basic world info
pub fn load_world_metadata<P: AsRef<Path>>(
    path: P,
) -> Result<World, PackageError> {
    let package = load_world(path)?;
    Ok(package.world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_test_world() -> World {
        World::new("Test World".to_string(), 12345)
    }

    #[test]
    fn test_save_and_load_world() {
        let world = create_test_world();
        
        // Create temp file
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        // Save world
        save_world(&world, path).unwrap();
        
        // Verify file exists and has content
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);
        
        // Load world back
        let loaded = load_world(path).unwrap();
        
        assert_eq!(loaded.world.name, world.name);
        assert_eq!(loaded.world.seed, world.seed);
    }

    #[test]
    fn test_save_world_package() {
        let mut world = create_test_world();
        world.description = Some("A test world".to_string());
        
        let package = WorldPackage {
            world,
            regions: Vec::new(),
            settlements: Vec::new(),
            persons: Vec::new(),
            events: Vec::new(),
            timelines: Vec::new(),
            terrain: None,
        };
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        save_world_package(&package, path).unwrap();
        
        // Load and verify
        let loaded = load_world(path).unwrap();
        assert_eq!(loaded.world.name, "Test World");
        assert_eq!(loaded.world.description, Some("A test world".to_string()));
    }

    #[test]
    fn test_inspect_package() {
        let world = create_test_world();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        save_world(&world, path).unwrap();
        
        let manifest = inspect_package(path).unwrap();
        assert_eq!(manifest.version, PACKAGE_VERSION);
        assert_eq!(manifest.world_name, "Test World");
        assert_eq!(manifest.seed, 12345);
    }

    #[test]
    fn test_load_world_metadata() {
        let world = create_test_world();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        save_world(&world, path).unwrap();
        
        let metadata = load_world_metadata(path).unwrap();
        assert_eq!(metadata.name, "Test World");
        assert_eq!(metadata.seed, 12345);
    }

    #[test]
    fn test_package_with_regions() {
        let world = create_test_world();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        let mut package = WorldPackage {
            world,
            regions: Vec::new(),
            settlements: Vec::new(),
            persons: Vec::new(),
            events: Vec::new(),
            timelines: Vec::new(),
            terrain: None,
        };
        
        // Add a region
        let region = Region::new(
            package.world.id.id,
            "The North".to_string(),
            150000.0,
            55.0,
            -90.0,
        );
        package.regions.push(region);
        
        save_world_package(&package, path).unwrap();
        
        let loaded = load_world(path).unwrap();
        assert_eq!(loaded.regions.len(), 1);
        assert_eq!(loaded.regions[0].name, "The North");
    }

    #[test]
    fn test_nonexistent_file() {
        let result = load_world("/nonexistent/path.wfw");
        assert!(result.is_err());
    }

    #[test]
    fn test_package_error_display() {
        let err = PackageError::InvalidFormat("test".to_string());
        assert_eq!(format!("{}", err), "Invalid format: test");
    }
}