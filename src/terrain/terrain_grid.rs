//! Terrain Grid - efficient storage for terrain data
//! 
//! Uses bit-packing and chunked storage for memory efficiency.
//! Target: <500MB per million cells.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Chunk size for terrain grid storage (32x32 cells per chunk).
pub const CHUNK_SIZE: usize = 32;

/// Terrain grid with chunked storage for memory efficiency.
#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainGrid {
    width: u32,
    height: u32,
    chunks: Vec<TerrainChunk>,
}

impl TerrainGrid {
    /// Create a new terrain grid.
    pub fn new(width: u32, height: u32) -> Self {
        let chunk_count = (((width * height) as usize) / (CHUNK_SIZE * CHUNK_SIZE)).max(1);
        Self {
            width,
            height,
            chunks: Vec::with_capacity(chunk_count),
        }
    }
    
    /// Get grid dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Get cell at (x, y) coordinate.
    pub fn get(&self, x: u32, y: u32) -> Option<TerrainCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let chunk_x = x / CHUNK_SIZE as u32;
        let chunk_y = y / CHUNK_SIZE as u32;
        let local_x = x % CHUNK_SIZE as u32;
        let local_y = y % CHUNK_SIZE as u32;
        
        let chunk_idx = (chunk_y * (self.width / CHUNK_SIZE as u32) + chunk_x) as usize;
        let _cell_idx = (local_y * CHUNK_SIZE as u32 + local_x) as usize;
        
        self.chunks.get(chunk_idx).map(|c| c.get(local_x as usize, local_y as usize))
    }
    
    /// Set cell at (x, y) coordinate.
    pub fn set(&mut self, x: u32, y: u32, cell: TerrainCell) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let chunk_x = x / CHUNK_SIZE as u32;
        let chunk_y = y / CHUNK_SIZE as u32;
        let local_x = x % CHUNK_SIZE as u32;
        let local_y = y % CHUNK_SIZE as u32;
        
        let chunk_idx = (chunk_y * (self.width / CHUNK_SIZE as u32) + chunk_x) as usize;
        
        if let Some(chunk) = self.chunks.get_mut(chunk_idx) {
            chunk.set(local_x as usize, local_y as usize, cell);
            true
        } else {
            false
        }
    }
    
    /// Initialize chunks - call this after creating the grid.
    pub fn initialize(&mut self) {
        let chunk_w = (self.width / CHUNK_SIZE as u32).max(1);
        let chunk_h = (self.height / CHUNK_SIZE as u32).max(1);
        let total_chunks = (chunk_w * chunk_h) as usize;
        
        self.chunks.clear();
        self.chunks.reserve(total_chunks);
        
        for _ in 0..total_chunks {
            self.chunks.push(TerrainChunk::new());
        }
    }
    
    /// Calculate memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        // Each chunk is 32x32 cells × 4 bytes per cell = 4096 bytes per chunk
        let chunk_bytes = CHUNK_SIZE * CHUNK_SIZE * 4;
        self.chunks.len() * chunk_bytes
    }
    
    /// Get an iterator over all cells in row-major order.
    pub fn cells(&self) -> impl Iterator<Item = (u32, u32, TerrainCell)> + '_ {
        TerrainCellIterator {
            grid: self,
            x: 0,
            y: 0,
        }
    }
    
    /// Get total number of cells.
    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
    }
    
    /// Check if grid has no cells.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A chunk of terrain cells (32x32).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainChunk {
    #[serde(with = "serde_arrays")]
    cells: [u32; CHUNK_SIZE * CHUNK_SIZE],
}

impl TerrainChunk {
    pub fn new() -> Self {
        Self {
            cells: [0u32; CHUNK_SIZE * CHUNK_SIZE],
        }
    }
    
    pub fn get(&self, x: usize, y: usize) -> TerrainCell {
        let idx = y * CHUNK_SIZE + x;
        TerrainCell::from_raw(self.cells[idx])
    }
    
    pub fn set(&mut self, x: usize, y: usize, cell: TerrainCell) {
        let idx = y * CHUNK_SIZE + x;
        self.cells[idx] = cell.to_raw();
    }
}

impl Default for TerrainChunk {
    fn default() -> Self {
        Self::new()
    }
}

/// A single terrain cell with bit-packed data.
/// 
/// Layout (32 bits total):
/// - bits 0-9: height (0-1023 meters)
/// - bits 10-13: biome index (0-15)
/// - bits 14-15: moisture level (0-3)
/// - bit 16: is_water
/// - bits 17-21: feature flags
/// - bits 22-31: reserved
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainCell(u32);

impl TerrainCell {
    /// Create a new terrain cell from individual values.
    pub fn new(height: f32, biome: u8, moisture: u8, is_water: bool) -> Self {
        let height_raw = (height.clamp(0.0, 1023.0) as u32) << 0;
        let biome_raw = ((biome as u32) & 0xF) << 10;
        let moisture_raw = ((moisture as u32) & 0x3) << 14;
        let water_raw = (if is_water { 1u32 } else { 0u32 }) << 16;
        
        TerrainCell(height_raw | biome_raw | moisture_raw | water_raw)
    }
    
    /// Get height in meters.
    pub fn height(&self) -> f32 {
        ((self.0 >> 0) & 0x3FF) as f32
    }
    
    /// Get biome index.
    pub fn biome(&self) -> u8 {
        ((self.0 >> 10) & 0xF) as u8
    }
    
    /// Get moisture level index.
    pub fn moisture(&self) -> u8 {
        ((self.0 >> 14) & 0x3) as u8
    }
    
    /// Check if cell is water.
    pub fn is_water(&self) -> bool {
        ((self.0 >> 16) & 1) != 0
    }
    
    /// Set height value.
    pub fn set_height(&mut self, height: f32) {
        let h = (height.clamp(0.0, 1023.0) as u32) << 0;
        self.0 = (self.0 & !0x3FF) | h;
    }
    
    /// Set biome index.
    pub fn set_biome(&mut self, biome: u8) {
        let b = ((biome as u32) & 0xF) << 10;
        self.0 = (self.0 & !(0xF << 10)) | b;
    }
    
    /// Set moisture level.
    pub fn set_moisture(&mut self, moisture: u8) {
        let m = ((moisture as u32) & 0x3) << 14;
        self.0 = (self.0 & !(0x3 << 14)) | m;
    }
    
    /// Set water flag.
    pub fn set_water(&mut self, is_water: bool) {
        let w = if is_water { 1u32 } else { 0u32 } << 16;
        self.0 = (self.0 & !(1 << 16)) | w;
    }
    
    /// Convert to raw u32 for storage.
    pub fn to_raw(&self) -> u32 {
        self.0
    }
    
    /// Create from raw u32.
    pub fn from_raw(raw: u32) -> Self {
        TerrainCell(raw)
    }
}

/// Iterator over all cells in a terrain grid.
pub struct TerrainCellIterator<'a> {
    grid: &'a TerrainGrid,
    x: u32,
    y: u32,
}

impl<'a> Iterator for TerrainCellIterator<'a> {
    type Item = (u32, u32, TerrainCell);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.grid.height {
            return None;
        }
        
        let cell = self.grid.get(self.x, self.y).map(|c| (self.x, self.y, c));
        
        self.x += 1;
        if self.x >= self.grid.width {
            self.x = 0;
            self.y += 1;
        }
        
        cell
    }
}

impl fmt::Debug for TerrainGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerrainGrid")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("chunks", &self.chunks.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cell_bitpacking() {
        let cell = TerrainCell::new(523.5, 5, 2, false);
        
        assert_eq!(cell.height(), 523.0); // Clamped to int
        assert_eq!(cell.biome(), 5);
        assert_eq!(cell.moisture(), 2);
        assert!(!cell.is_water());
        
        // Test water flag
        let mut water_cell = cell;
        water_cell.set_water(true);
        assert!(water_cell.is_water());
    }
    
    #[test]
    fn test_cell_raw_conversion() {
        let original = TerrainCell::new(250.0, 3, 1, false);
        let raw = original.to_raw();
        let restored = TerrainCell::from_raw(raw);
        
        assert_eq!(original.height(), restored.height());
        assert_eq!(original.biome(), restored.biome());
        assert_eq!(original.moisture(), restored.moisture());
        assert_eq!(original.is_water(), restored.is_water());
    }
    
    #[test]
    fn test_grid_dimensions() {
        let grid = TerrainGrid::new(100, 100);
        let (w, h) = grid.dimensions();
        
        assert_eq!(w, 100);
        assert_eq!(h, 100);
    }
    
    #[test]
    fn test_memory_usage() {
        let mut grid = TerrainGrid::new(128, 128);
        grid.initialize();
        
        // 128x128 = 16 chunks (32x32 each)
        let expected_chunks = 16;
        let expected_bytes = expected_chunks * CHUNK_SIZE * CHUNK_SIZE * 4;
        
        assert_eq!(grid.memory_usage(), expected_bytes);
    }
}
