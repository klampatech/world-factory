//! Geometric primitives for World Factory
//! 
//! Provides Vec2, Direction, and other geometric types used throughout
//! the codebase for coordinate manipulation and spatial operations.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign};

/// A 2D vector type with generic numeric type.
/// 
/// Used extensively for grid coordinates, offsets, and positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vec2<T = i32> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    /// Create a new vector.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vec2<i32> {
    /// Zero vector constant.
    pub const ZERO: Vec2<i32> = Vec2 { x: 0, y: 0 };
    
    /// Create from usize coordinates.
    pub fn from_usize(x: usize, y: usize) -> Self {
        Self::new(x as i32, y as i32)
    }
    
    /// Convert to usize coordinates (with truncation).
    pub fn to_usize(self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
    
    /// Calculate Manhattan distance.
    pub fn manhattan_dist(&self, other: &Vec2<i32>) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    
    /// Calculate squared Euclidean distance.
    pub fn dist_sq(&self, other: &Vec2<i32>) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Vec2<f32> {
    /// Zero vector constant for f32.
    pub const ZERO: Vec2<f32> = Vec2 { x: 0.0, y: 0.0 };
}

impl<T: Copy + Add<Output = T>> Add for Vec2<T> {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Copy + Sub<Output = T>> Sub for Vec2<T> {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Copy + Mul<Output = T> + From<u32>> Mul<T> for Vec2<T> {
    type Output = Self;
    
    fn mul(self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl<T: Copy + Div<Output = T> + From<u32>> Div<T> for Vec2<T> {
    type Output = Self;
    
    fn div(self, scalar: T) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl<T: AddAssign> AddAssign for Vec2<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T: SubAssign> SubAssign for Vec2<T> {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl<T> Default for Vec2<T> where T: Default {
    fn default() -> Self {
        Self::new(T::default(), T::default())
    }
}

/// Cardinal and diagonal directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    /// Get the delta vector for this direction.
    pub fn delta(&self) -> Vec2<i32> {
        match self {
            Direction::North => Vec2::new(0, -1),
            Direction::Northeast => Vec2::new(1, -1),
            Direction::East => Vec2::new(1, 0),
            Direction::Southeast => Vec2::new(1, 1),
            Direction::South => Vec2::new(0, 1),
            Direction::Southwest => Vec2::new(-1, 1),
            Direction::West => Vec2::new(-1, 0),
            Direction::Northwest => Vec2::new(-1, -1),
        }
    }
    
    /// Get cardinal directions only (N, E, S, W).
    pub fn cardinal() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
    
    /// Get diagonal directions.
    pub fn diagonal() -> [Direction; 4] {
        [
            Direction::Northeast,
            Direction::Southeast,
            Direction::Southwest,
            Direction::Northwest,
        ]
    }
    
    /// Get all 8 directions.
    pub fn all() -> [Direction; 8] {
        [
            Direction::North,
            Direction::Northeast,
            Direction::East,
            Direction::Southeast,
            Direction::South,
            Direction::Southwest,
            Direction::West,
            Direction::Northwest,
        ]
    }
    
    /// Rotate 90 degrees clockwise.
    pub fn rotate_cw(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::Northeast => Direction::Southeast,
            Direction::East => Direction::South,
            Direction::Southeast => Direction::Southwest,
            Direction::South => Direction::West,
            Direction::Southwest => Direction::Northwest,
            Direction::West => Direction::North,
            Direction::Northwest => Direction::Northeast,
        }
    }
    
    /// Rotate 90 degrees counter-clockwise.
    pub fn rotate_ccw(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::Northeast => Direction::Northwest,
            Direction::East => Direction::North,
            Direction::Southeast => Direction::Northeast,
            Direction::South => Direction::East,
            Direction::Southwest => Direction::Southeast,
            Direction::West => Direction::South,
            Direction::Northwest => Direction::Southwest,
        }
    }
    
    /// Get the opposite direction.
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::Northeast => Direction::Southwest,
            Direction::East => Direction::West,
            Direction::Southeast => Direction::Northwest,
            Direction::South => Direction::North,
            Direction::Southwest => Direction::Northeast,
            Direction::West => Direction::East,
            Direction::Northwest => Direction::Southeast,
        }
    }
}

/// A seed value wrapper for deterministic generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed(pub u64);

impl Seed {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for Seed {
    fn default() -> Self {
        Self(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_basic() {
        let v = Vec2::new(3, 4);
        assert_eq!(v.x, 3);
        assert_eq!(v.y, 4);
    }

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(1, 2);
        let b = Vec2::new(3, 4);
        
        assert_eq!(a + b, Vec2::new(4, 6));
        assert_eq!(b - a, Vec2::new(2, 2));
    }

    #[test]
    fn test_vec2_scaling() {
        let v = Vec2::new(2, 3);
        assert_eq!(v * 2, Vec2::new(4, 6));
        assert_eq!(v / 2, Vec2::new(1, 1));
    }

    #[test]
    fn test_vec2_distance() {
        let a = Vec2::new(0, 0);
        let b = Vec2::new(3, 4);
        
        assert_eq!(a.manhattan_dist(&b), 7);
        assert_eq!(a.dist_sq(&b), 25);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::North.delta(), Vec2::new(0, -1));
        assert_eq!(Direction::South.delta(), Vec2::new(0, 1));
        assert_eq!(Direction::East.delta(), Vec2::new(1, 0));
        assert_eq!(Direction::West.delta(), Vec2::new(-1, 0));
    }

    #[test]
    fn test_direction_rotation() {
        let north = Direction::North;
        assert_eq!(north.rotate_cw(), Direction::East);
        assert_eq!(north.rotate_ccw(), Direction::West);
        assert_eq!(north.opposite(), Direction::South);
    }

    #[test]
    fn test_seed() {
        let s1 = Seed::new(123);
        let s2 = Seed::new(123);
        let s3 = Seed::new(456);
        
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert_eq!(s1.value(), 123);
    }
}
