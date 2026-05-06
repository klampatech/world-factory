//! Elevation Grid - efficient storage for elevation data
//!
//! Provides a grid-based elevation storage optimized for terrain operations.
//! This is the primary terrain representation used by the river generator and other systems.
//!
//! Memory layout: Row-major, with each cell storing a normalized elevation [0.0, 1.0]

use serde::{Deserialize, Serialize};

/// A 2D grid storing elevation values for terrain generation.
///
/// Values are normalized to [0.0, 1.0] where:
/// - 0.0 = lowest elevation (sea level or below)
/// - 1.0 = highest elevation (mountain peaks)
///
/// This is used by the river generator and other systems that need
/// efficient grid-based access patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationGrid {
    /// Grid width
    pub width: usize,
    /// Grid height
    pub height: usize,
    /// Row-major elevation data
    data: Vec<f32>,
}

impl ElevationGrid {
    /// Create a new grid filled with a default value.
    ///
    /// # Arguments
    /// * `width` - Grid width in cells
    /// * `height` - Grid height in cells
    /// * `default_value` - Initial value for all cells
    pub fn new(width: usize, height: usize, default_value: f32) -> Self {
        let data = vec![default_value; width * height];
        Self {
            width,
            height,
            data,
        }
    }

    /// Create a grid from existing data.
    ///
    /// # Arguments
    /// * `width` - Grid width
    /// * `height` - Grid height
    /// * `data` - Row-major elevation data (must be width * height elements)
    pub fn from_data(width: usize, height: usize, data: Vec<f32>) -> Self {
        assert_eq!(
            data.len(),
            width * height,
            "Data length {} doesn't match dimensions {}x{}",
            data.len(),
            width,
            height
        );
        Self {
            width,
            height,
            data,
        }
    }

    /// Get total number of cells.
    #[inline]
    pub fn len(&self) -> usize {
        self.width * self.height
    }

    /// Check if grid is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get index for coordinates (row-major).
    #[inline]
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Get index without bounds checking.
    #[inline]
    fn unchecked_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Check if coordinates are valid.
    #[inline]
    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    /// Get elevation at (x, y) with bounds checking.
    ///
    /// Returns None if coordinates are out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        self.index(x, y).map(|i| self.data[i])
    }

    /// Get elevation at (x, y) without bounds checking.
    ///
    /// # Safety
    /// Calling this with invalid coordinates is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> f32 {
        *self.data.get_unchecked(self.unchecked_index(x, y))
    }

    /// Get a reference to the underlying data slice.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get a mutable reference to the underlying data slice.
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get elevation with i32 coordinates (for river generator compatibility).
    #[inline]
    pub fn get_value_unchecked(&self, x: i32, y: i32) -> f32 {
        // This is safe because we trust the callers to pass valid coordinates
        // based on is_valid() checks they should have done first
        unsafe {
            *self
                .data
                .get_unchecked((y as usize) * self.width + (x as usize))
        }
    }

    /// Set elevation at (x, y) with bounds checking.
    pub fn set(&mut self, x: usize, y: usize, value: f32) -> bool {
        if let Some(i) = self.index(x, y) {
            self.data[i] = value.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Set elevation without bounds checking.
    ///
    /// # Safety
    /// Calling this with invalid coordinates is undefined behavior.
    #[inline]
    pub unsafe fn set_unchecked(&mut self, x: usize, y: usize, value: f32) {
        let idx = self.unchecked_index(x, y);
        *self.data.get_unchecked_mut(idx) = value.clamp(0.0, 1.0);
    }

    /// Set elevation with i32 coordinates (for river generator compatibility).
    #[inline]
    pub fn set_value_unchecked(&mut self, x: i32, y: i32, value: f32) {
        // Safety: Same as get_value_unchecked
        let idx = (y as usize) * self.width + (x as usize);
        unsafe {
            *self.data.get_unchecked_mut(idx) = value.clamp(0.0, 1.0);
        }
    }

    /// Get elevation as f64 (for API compatibility).
    #[inline]
    pub fn get_f64(&self, x: usize, y: usize) -> Option<f64> {
        self.get(x, y).map(|v| v as f64)
    }

    /// Set elevation from f64.
    #[inline]
    pub fn set_f64(&mut self, x: usize, y: usize, value: f64) -> bool {
        self.set(x, y, value as f32)
    }

    /// Fill the entire grid with a value.
    pub fn fill(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        self.data.fill(clamped);
    }

    /// Fill with a function.
    pub fn fill_with(&mut self, mut f: impl FnMut(usize, usize) -> f32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                self.data[idx] = f(x, y).clamp(0.0, 1.0);
            }
        }
    }

    /// Iterate over all elevations.
    pub fn values(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().copied()
    }

    /// Iterate with coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
        self.data
            .iter()
            .enumerate()
            .map(|(i, &v)| (i % self.width, i / self.width, v))
    }

    /// Get minimum elevation.
    pub fn min(&self) -> f32 {
        self.data.iter().copied().fold(f32::INFINITY, f32::min)
    }

    /// Get maximum elevation.
    pub fn max(&self) -> f32 {
        self.data.iter().copied().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Get average elevation.
    pub fn average(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f32>() / self.data.len() as f32
    }

    /// Get statistics about the elevation distribution.
    pub fn statistics(&self) -> ElevationStatistics {
        if self.data.is_empty() {
            return ElevationStatistics::default();
        }

        let min = self.min();
        let max = self.max();
        let mean = self.average();

        let variance =
            self.data.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / self.data.len() as f32;
        let std_dev = variance.sqrt();

        let below_sea_level = self.data.iter().filter(|&&v| v < 0.5).count();
        let above_sea_level = self.data.len() - below_sea_level;

        ElevationStatistics {
            min,
            max,
            mean,
            std_dev,
            variance,
            below_sea_level,
            above_sea_level,
            total_cells: self.data.len(),
        }
    }

    /// Apply a function to each cell, modifying in place.
    pub fn map_inplace(&mut self, mut f: impl FnMut(f32) -> f32) {
        for value in &mut self.data {
            *value = f(*value).clamp(0.0, 1.0);
        }
    }

    /// Sample elevation at a point (bilinear interpolation).
    pub fn sample_bilinear(&self, x: f32, y: f32) -> f32 {
        // Clamp to grid bounds
        let x = x.clamp(0.0, (self.width - 1) as f32);
        let y = y.clamp(0.0, (self.height - 1) as f32);

        // Get integer and fractional parts
        let x0 = x as usize;
        let y0 = y as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        // Bilinear interpolation
        let v00 = unsafe { self.get_unchecked(x0, y0) };
        let v10 = unsafe { self.get_unchecked(x1, y0) };
        let v01 = unsafe { self.get_unchecked(x0, y1) };
        let v11 = unsafe { self.get_unchecked(x1, y1) };

        let v0 = v00 * (1.0 - fx) + v10 * fx;
        let v1 = v01 * (1.0 - fx) + v11 * fx;

        v0 * (1.0 - fy) + v1 * fy
    }

    /// Get gradient at a point (for slope calculations).
    pub fn gradient(&self, x: i32, y: i32) -> (f32, f32) {
        let left = if x > 0 {
            unsafe { self.get_unchecked((x - 1) as usize, y as usize) }
        } else {
            unsafe { self.get_unchecked(x as usize, y as usize) }
        };
        let right = if (x as usize) < self.width - 1 {
            unsafe { self.get_unchecked((x + 1) as usize, y as usize) }
        } else {
            unsafe { self.get_unchecked(x as usize, y as usize) }
        };
        let up = if y > 0 {
            unsafe { self.get_unchecked(x as usize, (y - 1) as usize) }
        } else {
            unsafe { self.get_unchecked(x as usize, y as usize) }
        };
        let down = if (y as usize) < self.height - 1 {
            unsafe { self.get_unchecked(x as usize, (y + 1) as usize) }
        } else {
            unsafe { self.get_unchecked(x as usize, y as usize) }
        };

        (right - left, down - up)
    }

    /// Calculate slope magnitude at a point.
    pub fn slope(&self, x: i32, y: i32) -> f32 {
        let (dx, dy) = self.gradient(x, y);
        (dx * dx + dy * dy).sqrt()
    }

    /// Resize the grid to new dimensions.
    /// Uses nearest-neighbor sampling.
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let mut new_data = vec![0.0; new_width * new_height];

        for y in 0..new_height {
            for x in 0..new_width {
                // Map to old coordinates
                let src_x = (x as f32 / new_width as f32) * self.width as f32;
                let src_y = (y as f32 / new_height as f32) * self.height as f32;
                new_data[y * new_width + x] = self.sample_bilinear(src_x, src_y);
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.data = new_data;
    }

    /// Get a copy of a row.
    pub fn row(&self, y: usize) -> Option<&[f32]> {
        if y < self.height {
            let start = y * self.width;
            Some(&self.data[start..start + self.width])
        } else {
            None
        }
    }

    /// Get a mutable copy of a row.
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [f32]> {
        if y < self.height {
            let start = y * self.width;
            Some(&mut self.data[start..start + self.width])
        } else {
            None
        }
    }
}

/// Statistics about elevation distribution.
#[derive(Debug, Clone, Default)]
pub struct ElevationStatistics {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub variance: f32,
    pub below_sea_level: usize,
    pub above_sea_level: usize,
    pub total_cells: usize,
}

impl ElevationStatistics {
    /// Calculate sea level percentage.
    pub fn sea_level_percentage(&self) -> f32 {
        if self.total_cells == 0 {
            return 0.0;
        }
        self.below_sea_level as f32 / self.total_cells as f32
    }

    /// Calculate land percentage.
    pub fn land_percentage(&self) -> f32 {
        1.0 - self.sea_level_percentage()
    }

    /// Check if statistics are valid.
    pub fn is_valid(&self) -> bool {
        self.total_cells > 0 && self.min >= 0.0 && self.max <= 1.0 && self.min <= self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let grid = ElevationGrid::new(10, 10, 0.5);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.len(), 100);
    }

    #[test]
    fn test_get_set() {
        let mut grid = ElevationGrid::new(10, 10, 0.0);

        // Test with bounds checking
        assert_eq!(grid.get(0, 0), Some(0.0));
        assert!(grid.set(0, 0, 0.5));
        assert_eq!(grid.get(0, 0), Some(0.5));

        // Test out of bounds
        assert_eq!(grid.get(10, 0), None);
        assert!(!grid.set(10, 0, 0.5));

        // Test clamping
        assert!(grid.set(0, 0, 2.0));
        assert_eq!(grid.get(0, 0), Some(1.0));
        assert!(grid.set(0, 0, -1.0));
        assert_eq!(grid.get(0, 0), Some(0.0));
    }

    #[test]
    fn test_unchecked_access() {
        let mut grid = ElevationGrid::new(10, 10, 0.0);

        unsafe {
            assert_eq!(grid.get_unchecked(0, 0), 0.0);
            grid.set_unchecked(0, 0, 0.5);
            assert_eq!(grid.get_unchecked(0, 0), 0.5);
        }

        // i32 version for compatibility
        assert_eq!(grid.get_value_unchecked(0, 0), 0.5);
        grid.set_value_unchecked(0, 0, 0.8);
        assert_eq!(grid.get_value_unchecked(0, 0), 0.8);
    }

    #[test]
    fn test_is_valid() {
        let grid = ElevationGrid::new(10, 10, 0.0);

        assert!(grid.is_valid(0, 0));
        assert!(grid.is_valid(9, 9));
        assert!(!grid.is_valid(10, 0));
        assert!(!grid.is_valid(0, 10));
        assert!(!grid.is_valid(-1, 0));
        assert!(!grid.is_valid(0, -1));
    }

    #[test]
    fn test_statistics() {
        let mut grid = ElevationGrid::new(10, 10, 0.0);
        grid.fill_with(|x, y| (x + y) as f32 / 18.0);

        let stats = grid.statistics();

        assert_eq!(stats.min, 0.0);
        assert_eq!(stats.max, 1.0);
        assert!((stats.mean - 0.5).abs() < 0.01);
        assert!(stats.is_valid());
    }

    #[test]
    fn test_bilinear_interpolation() {
        let mut grid = ElevationGrid::new(4, 4, 0.0);

        // Create a simple pattern
        // 0.0  0.0  1.0  1.0
        // 0.0  0.0  1.0  1.0
        // 0.0  0.0  1.0  1.0
        // 0.0  0.0  1.0  1.0
        for y in 0..4 {
            for x in 0..4 {
                let value = if x >= 2 { 1.0 } else { 0.0 };
                grid.set(x, y, value);
            }
        }

        // Test corners
        assert!((grid.sample_bilinear(0.0, 0.0) - 0.0).abs() < 0.001);
        assert!((grid.sample_bilinear(3.0, 0.0) - 1.0).abs() < 0.001);

        // Test middle
        assert!((grid.sample_bilinear(1.5, 1.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_resize() {
        let mut grid = ElevationGrid::new(4, 4, 0.0);
        grid.fill_with(|x, y| (x + y) as f32 / 6.0);

        grid.resize(8, 8);

        assert_eq!(grid.width, 8);
        assert_eq!(grid.height, 8);
        assert_eq!(grid.len(), 64);

        // Original corner should still be roughly 0
        assert!(grid.get(0, 0).unwrap() < 0.1);
    }

    #[test]
    fn test_row_access() {
        let mut grid = ElevationGrid::new(4, 4, 0.0);

        // Fill with row number
        grid.fill_with(|_, y| y as f32 / 3.0);

        assert_eq!(grid.row(0), Some(&[0.0, 0.0, 0.0, 0.0][..]));
        assert_eq!(
            grid.row(1),
            Some(&[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0][..])
        );
        assert_eq!(grid.row(3), Some(&[1.0, 1.0, 1.0, 1.0][..]));
        assert!(grid.row(4).is_none());
    }

    #[test]
    fn test_gradient() {
        let mut grid = ElevationGrid::new(5, 5, 0.0);

        // Create a slope: high on the right
        grid.fill_with(|x, _| x as f32 / 4.0);

        let (dx, dy) = grid.gradient(2, 2);

        // Should have positive x gradient
        assert!(dx > 0.0);
        // Should have zero y gradient (uniform in y)
        assert!(dy.abs() < 0.01);
    }
}
