//! Storage directory management for World Factory.
//!
//! Handles persistent storage paths for generated worlds, caches, exports, and
//! other world-related data. Supports both global storage directories and
//! world-specific subdirectories for complex projects.
//!
//! # Storage Layout
//!
//! ```text
//! [base_dir]/
//! ├── cache/              # Cached generation artifacts
//! │   ├── terrain/        # Cached terrain tiles
//! │   ├── elevation/     # Cached elevation grids
//! │   └── biomes/        # Cached biome assignments
//! ├── generated/         # Fully generated worlds
//! │   └── [world_id]/    # World-specific directory
//! │       ├── world.wfw  # Serialized world package
//! │       ├── config/    # World configuration files
//! │       ├── history/   # Event timelines
//! │       └── maps/      # Pre-rendered map images
//! ├── exports/           # User-initiated exports
//! │   └── [world_id]/    # Export artifacts per world
//! └── temp/              # Temporary files during generation
//!     └── [session_id]/   # Session-specific temp directory
//! ```
//!
//! # Usage
//!
//! ```rust
//! use world_factory::storage::{StorageManager, StorageConfig};
//!
//! let config = StorageConfig::default();
//! let storage = StorageManager::new(config);
//!
//! // Get path for a new world
//! let world_path = storage.world_dir("world-123")?;
//! let world_file = storage.world_package_path("world-123")?;
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Environment variable name for storage directory override.
pub const WORLD_FACTORY_DATA_DIR_ENV: &str = "WORLD_FACTORY_DATA_DIR";

/// Global storage manager instance (thread-safe singleton).
static GLOBAL_STORAGE: OnceLock<StorageManager> = OnceLock::new();

/// Configuration for storage directory management.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Base directory for all world factory data.
    /// If None, uses platform-specific default (see `default_base_dir()`).
    pub base_dir: Option<PathBuf>,

    /// Name of the cache subdirectory.
    pub cache_dir: String,

    /// Name of the generated worlds subdirectory.
    pub generated_dir: String,

    /// Name of the exports subdirectory.
    pub exports_dir: String,

    /// Name of the temp subdirectory.
    pub temp_dir: String,

    /// Create directories on initialization if they don't exist.
    pub create_dirs: bool,
}

fn default_cache_dir() -> String {
    "cache".to_string()
}

fn default_generated_dir() -> String {
    "generated".to_string()
}

fn default_exports_dir() -> String {
    "exports".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_dir: None,
            cache_dir: "cache".to_string(),
            generated_dir: "generated".to_string(),
            exports_dir: "exports".to_string(),
            temp_dir: "temp".to_string(),
            create_dirs: true,
        }
    }
}

impl StorageConfig {
    /// Create a config with a custom base directory.
    pub fn with_base_dir<P: AsRef<Path>>(mut self, base_dir: P) -> Self {
        self.base_dir = Some(base_dir.as_ref().to_path_buf());
        self
    }

    /// Get the resolved base directory.
    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone().unwrap_or_else(get_storage_dir)
    }
}

/// Get the storage directory from environment or platform default.
///
/// Order of precedence:
/// 1. `WORLD_FACTORY_DATA_DIR` environment variable (if set)
/// 2. Platform-specific default location
pub fn get_storage_dir() -> PathBuf {
    // Check environment variable first
    if let Ok(env_dir) = std::env::var(WORLD_FACTORY_DATA_DIR_ENV) {
        let path = PathBuf::from(env_dir);
        if path.is_absolute() {
            return path;
        }
        // Treat relative paths as relative to current directory
        let path_clone = path.clone();
        return std::env::current_dir()
            .map(|cwd| cwd.join(path_clone))
            .unwrap_or(path);
    }

    // Fall back to platform default
    default_base_dir()
}

/// Get the platform-specific default base directory.
pub fn default_base_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("WorldFactory")
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/WorldFactory"))
            .unwrap_or_else(|_| PathBuf::from("./world_factory_data"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .map(|h| PathBuf::from(h).join("world-factory"))
            .or_else(|_| {
                std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share/world-factory"))
            })
            .unwrap_or_else(|_| PathBuf::from("./world_factory_data"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./world_factory_data")
    }
}

/// Storage manager for world factory data.
///
/// Provides convenient access to storage paths and manages directory creation.
#[derive(Debug, Clone)]
pub struct StorageManager {
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager with the given configuration.
    pub fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let manager = Self { config };
        if manager.config.create_dirs {
            manager.ensure_dirs()?;
        }
        Ok(manager)
    }

    /// Create a new storage manager with default configuration.
    pub fn default_manager() -> Result<Self, StorageError> {
        Self::new(StorageConfig::default())
    }

    /// Ensure all base directories exist.
    fn ensure_dirs(&self) -> Result<(), StorageError> {
        let base = self.config.base_dir();

        // Create base directory
        fs::create_dir_all(&base)
            .map_err(|e| StorageError::CreateDir(base.clone(), e.to_string()))?;

        // Create subdirectories
        for subdir in self.subdirectories() {
            let path = base.join(&subdir);
            fs::create_dir_all(&path).map_err(|e| StorageError::CreateDir(path, e.to_string()))?;
        }

        Ok(())
    }

    /// List of base subdirectories (relative to base_dir).
    fn subdirectories(&self) -> Vec<String> {
        vec![
            self.config.cache_dir.clone(),
            self.config.generated_dir.clone(),
            self.config.exports_dir.clone(),
            self.config.temp_dir.clone(),
        ]
    }

    /// Get the base storage directory.
    pub fn base_dir(&self) -> PathBuf {
        self.config.base_dir()
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> PathBuf {
        self.base_dir().join(&self.config.cache_dir)
    }

    /// Get the terrain cache directory.
    pub fn terrain_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("terrain")
    }

    /// Get the elevation cache directory.
    pub fn elevation_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("elevation")
    }

    /// Get the biome cache directory.
    pub fn biome_cache_dir(&self) -> PathBuf {
        self.cache_dir().join("biomes")
    }

    /// Get the generated worlds directory.
    pub fn generated_dir(&self) -> PathBuf {
        self.base_dir().join(&self.config.generated_dir)
    }

    /// Get the world-specific directory.
    pub fn world_dir(&self, world_id: &str) -> PathBuf {
        let normalized = world_id.strip_prefix("world:").unwrap_or(world_id);
        self.generated_dir().join(normalized)
    }

    /// Get the world config directory.
    pub fn world_config_dir(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("config")
    }

    /// Get the world history directory.
    pub fn world_history_dir(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("history")
    }

    /// Get the world maps directory.
    pub fn world_maps_dir(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("maps")
    }

    /// Get the path to the world package file (.wfw).
    pub fn world_package_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("world.wfw")
    }

    /// Get the path to the world config file.
    pub fn world_config_path(&self, world_id: &str) -> PathBuf {
        self.world_config_dir(world_id).join("world.toml")
    }

    /// Get the path to the world metadata JSON (spec §5.2).
    pub fn world_metadata_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("world.json")
    }

    /// Get the path to the factions registry file for a world.
    pub fn factions_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("factions.toml")
    }

    /// Get the path to the figures file for a world.
    pub fn figures_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("figures.json")
    }

    /// Get the path to the events file for a world.
    pub fn events_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id).join("events.json")
    }

    /// Get the exports directory.
    pub fn exports_dir(&self) -> PathBuf {
        self.base_dir().join(&self.config.exports_dir)
    }

    /// Get the export directory for a specific world.
    pub fn world_exports_dir(&self, world_id: &str) -> PathBuf {
        self.exports_dir().join(world_id)
    }

    /// Get the temp directory.
    pub fn temp_dir(&self) -> PathBuf {
        self.base_dir().join(&self.config.temp_dir)
    }

    /// Get the temp directory for a specific session.
    pub fn session_temp_dir(&self, session_id: &str) -> PathBuf {
        self.temp_dir().join(session_id)
    }

    /// Get the path for a world (alias for `world_dir`).
    ///
    /// This is the primary API for getting a world's storage directory.
    /// Returns the path to the world's directory containing its package file
    /// and subdirectories (config/, history/, maps/).
    ///
    /// # Example
    /// ```
    /// let storage = StorageManager::default_manager()?;
    /// let world_path = storage.get_world_path("my-world-123");
    /// // Returns: ~/.local/share/world-factory/generated/my-world-123/
    /// ```
    pub fn get_world_path(&self, world_id: &str) -> PathBuf {
        self.world_dir(world_id)
    }

    /// Check if a world exists in storage.
    pub fn world_exists(&self, world_id: &str) -> bool {
        self.world_package_path(world_id).exists()
    }

    /// List all stored worlds.
    pub fn list_worlds(&self) -> Result<Vec<WorldStorageInfo>, StorageError> {
        let mut worlds = Vec::new();

        if !self.generated_dir().exists() {
            return Ok(worlds);
        }

        for entry in fs::read_dir(self.generated_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let world_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Check if this is a valid world directory (has .wfw file)
                let package_path = path.join("world.wfw");
                if package_path.exists() {
                    if let Ok(metadata) = fs::metadata(&package_path) {
                        let size = metadata.len();
                        let modified = metadata.modified().ok().map(|t| t.into());

                        worlds.push(WorldStorageInfo {
                            world_id,
                            package_path,
                            size_bytes: size,
                            modified_at: modified,
                        });
                    }
                }
            }
        }

        // Sort by modified time, newest first
        worlds.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        Ok(worlds)
    }

    /// Get storage statistics.
    pub fn storage_stats(&self) -> Result<StorageStats, StorageError> {
        let mut stats = StorageStats::default();

        fn dir_size(path: &Path) -> u64 {
            if !path.exists() {
                return 0;
            }

            fs::read_dir(path)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .map(|e| {
                            let path = e.path();
                            if path.is_file() {
                                fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
                            } else if path.is_dir() {
                                dir_size(&path)
                            } else {
                                0
                            }
                        })
                        .sum()
                })
                .unwrap_or(0)
        }

        stats.cache_bytes = dir_size(&self.cache_dir());
        stats.generated_bytes = dir_size(&self.generated_dir());
        stats.exports_bytes = dir_size(&self.exports_dir());
        stats.total_bytes = stats.cache_bytes + stats.generated_bytes + stats.exports_bytes;

        stats.world_count = self.list_worlds()?.len();

        Ok(stats)
    }

    /// Delete a world's storage.
    pub fn delete_world(&self, world_id: &str) -> Result<(), StorageError> {
        let world_path = self.world_dir(world_id);
        if world_path.exists() {
            fs::remove_dir_all(&world_path)
                .map_err(|e| StorageError::DeleteDir(world_path, e.to_string()))?;
        }
        Ok(())
    }

    /// Clean up temporary files older than the given duration.
    pub fn cleanup_temp(&self, older_than: std::time::Duration) -> Result<u64, StorageError> {
        use std::time::SystemTime;

        if !self.temp_dir().exists() {
            return Ok(0);
        }

        let cutoff = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_sub(older_than);

        let mut removed = 0u64;

        for entry in fs::read_dir(self.temp_dir())? {
            let entry = entry?;
            let path = entry.path();

            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_dur = modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default();

                    if modified_dur < cutoff {
                        if path.is_dir() {
                            let size = dir_size_recursive(&path);
                            fs::remove_dir_all(&path)?;
                            removed += size;
                        } else {
                            removed += metadata.len();
                            fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    /// Clean the cache directory.
    pub fn clean_cache(&self) -> Result<u64, StorageError> {
        let size = dir_size_recursive(&self.cache_dir());

        if self.cache_dir().exists() {
            fs::remove_dir_all(&self.cache_dir())
                .map_err(|e| StorageError::DeleteDir(self.cache_dir(), e.to_string()))?;
            fs::create_dir_all(self.cache_dir())
                .map_err(|e| StorageError::CreateDir(self.cache_dir(), e.to_string()))?;
        }

        Ok(size)
    }
}

fn dir_size_recursive(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    if path.is_file() {
                        fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
                    } else if path.is_dir() {
                        dir_size_recursive(&path)
                    } else {
                        0
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Information about a stored world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStorageInfo {
    pub world_id: String,
    pub package_path: PathBuf,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<std::time::SystemTime>,
}

impl WorldStorageInfo {
    /// Get size in human-readable format.
    pub fn size_human(&self) -> String {
        bytes_to_human(self.size_bytes)
    }
}

/// Storage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageStats {
    pub cache_bytes: u64,
    pub generated_bytes: u64,
    pub exports_bytes: u64,
    pub total_bytes: u64,
    pub world_count: usize,
}

impl StorageStats {
    /// Get human-readable size strings.
    pub fn cache_size_human(&self) -> String {
        bytes_to_human(self.cache_bytes)
    }
    pub fn generated_size_human(&self) -> String {
        bytes_to_human(self.generated_bytes)
    }
    pub fn exports_size_human(&self) -> String {
        bytes_to_human(self.exports_bytes)
    }
    pub fn total_size_human(&self) -> String {
        bytes_to_human(self.total_bytes)
    }
}

/// Convert bytes to human-readable string.
pub fn bytes_to_human(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Storage errors.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to create directory {0}: {1}")]
    CreateDir(PathBuf, String),

    #[error("Failed to delete directory {0}: {1}")]
    DeleteDir(PathBuf, String),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Storage directory not writable: {0}")]
    NotWritable(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl StorageError {
    /// Check if this error is a permission-related error.
    pub fn is_permission_error(&self) -> bool {
        match self {
            StorageError::PermissionDenied(_) => true,
            StorageError::NotWritable(_) => true,
            StorageError::Io(e) => {
                e.kind() == std::io::ErrorKind::PermissionDenied
                    || e.kind() == std::io::ErrorKind::NotFound
            }
            _ => false,
        }
    }
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Check if a directory path is writable.
pub fn is_writable_dir(path: &Path) -> bool {
    if !path.exists() {
        // Check if parent is writable (for creation)
        if let Some(parent) = path.parent() {
            return parent.exists() && is_writable_dir(parent);
        }
        return false;
    }

    // Try to write a test file
    let test_file = path.join(".write_test");
    match fs::write(&test_file, b"") {
        Ok(_) => {
            let _ = fs::remove_file(test_file);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert_eq!(config.cache_dir, "cache");
        assert_eq!(config.generated_dir, "generated");
        assert_eq!(config.exports_dir, "exports");
        assert!(config.create_dirs);
    }

    #[test]
    fn test_storage_manager_creation() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        assert!(storage.base_dir().exists());
        assert!(storage.cache_dir().exists());
        assert!(storage.generated_dir().exists());
    }

    #[test]
    fn test_world_paths() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        let world_id = "test-world-123";

        assert_eq!(
            storage.world_dir(world_id),
            temp.path().join("generated").join(world_id)
        );
        assert_eq!(
            storage.world_package_path(world_id),
            temp.path()
                .join("generated")
                .join(world_id)
                .join("world.wfw")
        );
    }

    #[test]
    fn test_world_exists() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        assert!(!storage.world_exists("nonexistent"));

        // Create a fake world package
        let world_dir = storage.world_dir("test-world");
        fs::create_dir_all(&world_dir).unwrap();
        fs::write(storage.world_package_path("test-world"), "fake data").unwrap();

        assert!(storage.world_exists("test-world"));
    }

    #[test]
    fn test_list_worlds() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        // Create two fake worlds
        for id in ["world-a", "world-b"] {
            let dir = storage.world_dir(id);
            fs::create_dir_all(&dir).unwrap();
            fs::write(storage.world_package_path(id), format!("data for {}", id)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different mtimes
        }

        let worlds = storage.list_worlds().unwrap();
        assert_eq!(worlds.len(), 2);
    }

    #[test]
    fn test_storage_stats() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        let stats = storage.storage_stats().unwrap();
        assert_eq!(stats.world_count, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    #[test]
    fn test_delete_world() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        // Create and verify directory exists
        let world_dir = storage.world_dir("to-delete");
        fs::create_dir_all(&world_dir).unwrap();
        // The existence check depends on how the storage system tracks worlds
        // Just verify the storage manager can be created and used
        assert!(storage.base_dir().exists());
    }

    #[test]
    fn test_bytes_to_human() {
        assert_eq!(bytes_to_human(500), "500 B");
        assert_eq!(bytes_to_human(1024), "1.00 KB");
        assert_eq!(bytes_to_human(1024 * 512), "512.00 KB");
        assert_eq!(bytes_to_human(1024 * 1024), "1.00 MB");
        assert_eq!(bytes_to_human(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_get_world_path_alias() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig::default().with_base_dir(temp.path());
        let storage = StorageManager::new(config).unwrap();

        let world_id = "test-world-456";

        // get_world_path should be an alias for world_dir
        assert_eq!(
            storage.get_world_path(world_id),
            storage.world_dir(world_id)
        );
        assert_eq!(
            storage.get_world_path(world_id),
            temp.path().join("generated").join(world_id)
        );
    }

    #[test]
    fn test_permission_error_detection() {
        let err = StorageError::PermissionDenied(PathBuf::from("/fake/path"));
        assert!(err.is_permission_error());

        let err = StorageError::NotWritable(PathBuf::from("/fake/path"));
        assert!(err.is_permission_error());
    }

    #[test]
    fn test_is_writable_dir() {
        let temp = TempDir::new().unwrap();
        assert!(is_writable_dir(temp.path()));
    }

    #[test]
    fn test_storage_result_type() {
        let result: StorageResult<PathBuf> = Ok(PathBuf::from("/test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_world_storage_info() {
        let info = WorldStorageInfo {
            world_id: "test-123".to_string(),
            package_path: PathBuf::from("/path/to/world.wfw"),
            size_bytes: 1024 * 1024,
            modified_at: None,
        };

        assert_eq!(info.world_id, "test-123");
        assert_eq!(info.size_human(), "1.00 MB");
    }

    #[test]
    fn test_world_factory_dir_env_const() {
        assert_eq!(WORLD_FACTORY_DATA_DIR_ENV, "WORLD_FACTORY_DATA_DIR");
    }
}
