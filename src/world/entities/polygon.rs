//! Polygon entities and geometric types for World Factory.
//!
//! This module provides core geometric types for polygon-based terrain representation.
//! It complements the terrain/topology.rs module with specific entity types.
//!
//! # Types
//!
//! - `Point2D` - 2D point/vertex representation
//! - `BoundingBox` - 2D axis-aligned bounding box
//! - `Polygon` - Simple polygon with vertices and computed properties
//! - `PolygonMesh` - Collection of polygons forming a mesh
//! - `Triangle` - Triangle for mesh triangulation

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// A 2D point/vertex in world space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    /// Create a new point.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0).
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Create from array.
    pub fn from_array([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }

    /// Convert to array.
    pub fn to_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// Distance to another point.
    pub fn distance(&self, other: &Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Squared distance to another point (faster, for comparisons).
    pub fn distance_squared(&self, other: &Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Dot product with another vector.
    pub fn dot(&self, other: &Point2D) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Cross product (2D pseudo-cross, returns scalar z-component).
    pub fn cross(&self, other: &Point2D) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Magnitude/length of vector.
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Squared magnitude.
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Normalized unit vector.
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            Self::origin()
        }
    }

    /// Linear interpolation to another point.
    pub fn lerp(&self, other: &Point2D, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Perpendicular vector (rotated 90° counterclockwise).
    pub fn perpendicular(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    /// Angle from origin to this point (radians).
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Angle between two vectors.
    pub fn angle_to(&self, other: &Point2D) -> f32 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    /// Rotate around origin.
    pub fn rotate(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Scale by a factor.
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    /// Midpoint between this and another point.
    pub fn midpoint(&self, other: &Point2D) -> Self {
        Self {
            x: (self.x + other.x) * 0.5,
            y: self.y + other.y * 0.5,
        }
    }

    /// Check if point is approximately equal (within epsilon).
    pub fn approx_eq(&self, other: &Point2D, epsilon: f32) -> bool {
        (self.x - other.x).abs() < epsilon && (self.y - other.y).abs() < epsilon
    }
}

impl Default for Point2D {
    fn default() -> Self {
        Self::origin()
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

impl Add for Point2D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Point2D {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for Point2D {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        if scalar != 0.0 {
            Self {
                x: self.x / scalar,
                y: self.y / scalar,
            }
        } else {
            self
        }
    }
}

/// A 2D axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Point2D,
    pub max: Point2D,
}

impl BoundingBox {
    /// Create a new bounding box.
    pub fn new(min: Point2D, max: Point2D) -> Self {
        Self { min, max }
    }

    /// Create from min/max coordinates.
    pub fn from_coords(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Self {
        Self {
            min: Point2D::new(x_min, y_min),
            max: Point2D::new(x_max, y_max),
        }
    }

    /// Create a bounding box containing a set of points.
    pub fn from_points(points: &[Point2D]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for p in points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }

        Some(Self { min, max })
    }

    /// Empty bounding box (inverted).
    pub fn empty() -> Self {
        Self {
            min: Point2D::new(f32::MAX, f32::MAX),
            max: Point2D::new(f32::MIN, f32::MIN),
        }
    }

    /// Infinite bounding box.
    pub fn infinite() -> Self {
        Self {
            min: Point2D::new(f32::MIN, f32::MIN),
            max: Point2D::new(f32::MAX, f32::MAX),
        }
    }

    /// Width of the bounding box.
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height of the bounding box.
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Center point.
    pub fn center(&self) -> Point2D {
        Point2D::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    /// Area (width * height).
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Perimeter length.
    pub fn perimeter(&self) -> f32 {
        2.0 * (self.width() + self.height())
    }

    /// Aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        let h = self.height();
        if h > 0.0 {
            self.width() / h
        } else {
            0.0
        }
    }

    /// Check if point is inside the bounding box.
    pub fn contains_point(&self, point: &Point2D) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if another bounding box is inside this one.
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        self.contains_point(&other.min) && self.contains_point(&other.max)
    }

    /// Check if this box intersects another.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.y >= other.min.y
            && self.min.y <= other.max.y
    }

    /// Expand to include a point.
    pub fn expand_to_include(&mut self, point: &Point2D) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    /// Expand to include another bounding box.
    pub fn expand_to_include_box(&mut self, other: &BoundingBox) {
        self.expand_to_include(&other.min);
        self.expand_to_include(&other.max);
    }

    /// Compute union of two bounding boxes.
    pub fn union(&self, other: &BoundingBox) -> Self {
        let mut result = *self;
        result.expand_to_include_box(other);
        result
    }

    /// Compute intersection of two bounding boxes.
    pub fn intersection(&self, other: &BoundingBox) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        Some(Self {
            min: Point2D::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            max: Point2D::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        })
    }

    /// Check if this box is valid (min <= max).
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }

    /// Check if empty (zero area).
    pub fn is_empty(&self) -> bool {
        self.width() <= 0.0 || self.height() <= 0.0
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BBox[{} to {}]", self.min, self.max)
    }
}

/// A polygon defined by vertices in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    /// Unique identifier.
    pub id: u32,
    /// Vertices in order (counter-clockwise for positive area).
    pub vertices: Vec<Point2D>,
    /// Neighboring polygon IDs.
    pub neighbors: Vec<u32>,
    /// Pre-computed centroid.
    centroid: Option<Point2D>,
    /// Pre-computed area.
    area: Option<f32>,
    /// Pre-computed perimeter.
    perimeter: Option<f32>,
}

impl Polygon {
    /// Create a new polygon with the given vertices.
    pub fn new(id: u32, vertices: Vec<Point2D>) -> Self {
        Self {
            id,
            vertices,
            neighbors: Vec::new(),
            centroid: None,
            area: None,
            perimeter: None,
        }
    }

    /// Create a triangle.
    pub fn triangle(id: u32, p0: Point2D, p1: Point2D, p2: Point2D) -> Self {
        Self::new(id, vec![p0, p1, p2])
    }

    /// Get the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get a reference to the vertices.
    pub fn vertices(&self) -> &[Point2D] {
        &self.vertices
    }

    /// Check if polygon is valid (at least 3 vertices, non-zero area).
    pub fn is_valid(&self) -> bool {
        self.vertices.len() >= 3 && self.signed_area().abs() > f32::EPSILON
    }

    /// Compute centroid of the polygon (read-only, no caching).
    pub fn centroid(&self) -> Point2D {
        if let Some(c) = self.centroid {
            return c;
        }

        let area = self.signed_area();
        if area.abs() < f32::EPSILON {
            // Degenerate polygon, return average of vertices
            return self.average_vertex();
        }

        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let n = self.vertices.len();

        for i in 0..n {
            let p0 = &self.vertices[i];
            let p1 = &self.vertices[(i + 1) % n];
            let cross = p0.x * p1.y - p1.x * p0.y;
            cx += (p0.x + p1.x) * cross;
            cy += (p0.y + p1.y) * cross;
        }

        Point2D::new(cx / (6.0 * area), cy / (6.0 * area))
    }

    /// Compute and cache centroid.
    pub fn compute_centroid(&mut self) -> Point2D {
        let c = Polygon::centroid(self);
        self.centroid = Some(c);
        c
    }

    /// Compute signed area (positive = CCW, negative = CW).
    pub fn signed_area(&self) -> f32 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }

        let mut sum = 0.0f32;
        for i in 0..n {
            let p0 = &self.vertices[i];
            let p1 = &self.vertices[(i + 1) % n];
            sum += p0.x * p1.y - p1.x * p0.y;
        }
        sum * 0.5
    }

    /// Compute absolute area (non-mutating, no caching).
    pub fn area(&self) -> f32 {
        self.signed_area().abs()
    }

    /// Compute and cache area (requires &mut self).
    pub fn compute_area(&mut self) -> f32 {
        if let Some(a) = self.area {
            return a;
        }
        let a = self.signed_area().abs();
        self.area = Some(a);
        a
    }

    /// Compute perimeter length (non-mutating, no caching).
    pub fn perimeter(&self) -> f32 {
        let n = self.vertices.len();
        if n < 2 {
            return 0.0;
        }
        let mut sum = 0.0f32;
        for i in 0..n {
            sum += self.vertices[i].distance(&self.vertices[(i + 1) % n]);
        }
        sum
    }

    /// Compute and cache perimeter (requires &mut self).
    pub fn compute_perimeter(&mut self) -> f32 {
        if let Some(p) = self.perimeter {
            return p;
        }
        let n = self.vertices.len();
        if n < 2 {
            return 0.0;
        }
        let mut sum = 0.0f32;
        for i in 0..n {
            sum += self.vertices[i].distance(&self.vertices[(i + 1) % n]);
        }
        self.perimeter = Some(sum);
        sum
    }

    /// Average of all vertices.
    fn average_vertex(&self) -> Point2D {
        if self.vertices.is_empty() {
            return Point2D::origin();
        }

        let mut sum = Point2D::origin();
        for v in &self.vertices {
            sum = sum + *v;
        }
        sum / self.vertices.len() as f32
    }

    /// Get an edge as a pair of points.
    pub fn edge(&self, index: usize) -> Option<(Point2D, Point2D)> {
        if self.vertices.is_empty() {
            return None;
        }
        let n = self.vertices.len();
        let i = index % n;
        let j = (i + 1) % n;
        Some((self.vertices[i], self.vertices[j]))
    }

    /// Add a neighbor.
    pub fn add_neighbor(&mut self, neighbor_id: u32) {
        if !self.neighbors.contains(&neighbor_id) {
            self.neighbors.push(neighbor_id);
        }
    }

    /// Check if point is inside polygon using ray casting.
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let n = self.vertices.len();
        let mut inside = false;

        let mut j = n - 1;
        for i in 0..n {
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            if ((vi.y > point.y) != (vj.y > point.y))
                && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    /// Point-to-polygon distance (squared).
    pub fn distance_squared_to_point(&self, point: &Point2D) -> f32 {
        if self.contains_point(point) {
            return 0.0;
        }

        let n = self.vertices.len();
        let mut min_dist = f32::MAX;

        for i in 0..n {
            let (dist, _) = crate::world::entities::polygon::Polygon::point_to_segment_squared(
                point,
                &self.vertices[i],
                &self.vertices[(i + 1) % n],
            );
            min_dist = min_dist.min(dist);
        }

        min_dist
    }

    /// Point-to-segment distance (squared) and closest point.
    fn point_to_segment_squared(
        point: &Point2D,
        seg_start: &Point2D,
        seg_end: &Point2D,
    ) -> (f32, Point2D) {
        let dx = seg_end.x - seg_start.x;
        let dy = seg_end.y - seg_start.y;
        let seg_len_sq = dx * dx + dy * dy;

        if seg_len_sq < f32::EPSILON {
            let closest = *seg_start;
            return (point.distance_squared(&closest), closest);
        }

        let t = ((point.x - seg_start.x) * dx + (point.y - seg_start.y) * dy) / seg_len_sq;
        let t = t.clamp(0.0, 1.0);

        let closest = Point2D::new(seg_start.x + t * dx, seg_start.y + t * dy);

        (point.distance_squared(&closest), closest)
    }

    /// Compute bounding box.
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_points(&self.vertices).unwrap_or_default()
    }

    /// Compute shape compactness (4π * area / perimeter²).
    /// 1.0 = perfect circle, 0.0 = infinitely thin.
    pub fn compactness(&self) -> f32 {
        let a = self.area();
        let p = self.perimeter();
        if p > 0.0 {
            (4.0 * std::f32::consts::PI * a) / (p * p)
        } else {
            0.0
        }
    }

    /// Invalidate cached computations.
    fn invalidate_cache(&mut self) {
        self.centroid = None;
        self.area = None;
        self.perimeter = None;
    }

    /// Reverse vertex order (changes signed area sign).
    pub fn reverse(&mut self) {
        self.vertices.reverse();
        self.invalidate_cache();
    }

    /// Ensure counter-clockwise vertex order.
    pub fn ensure_ccw(&mut self) {
        if self.signed_area() < 0.0 {
            self.reverse();
        }
    }
}

/// A triangle for mesh triangulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Triangle {
    pub a: Point2D,
    pub b: Point2D,
    pub c: Point2D,
}

impl Triangle {
    /// Create a new triangle.
    pub fn new(a: Point2D, b: Point2D, c: Point2D) -> Self {
        Self { a, b, c }
    }

    /// Compute area using cross product.
    pub fn area(&self) -> f32 {
        ((self.b.x - self.a.x) * (self.c.y - self.a.y)
            - (self.c.x - self.a.x) * (self.b.y - self.a.y))
            .abs()
            * 0.5
    }

    /// Compute centroid.
    pub fn centroid(&self) -> Point2D {
        Point2D::new(
            (self.a.x + self.b.x + self.c.x) / 3.0,
            (self.a.y + self.b.y + self.c.y) / 3.0,
        )
    }

    /// Check if a point is inside the triangle (barycentric coords).
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let v0 = self.c - self.a;
        let v1 = self.b - self.a;
        let v2 = *point - self.a;

        let dot00 = v0.dot(&v0);
        let dot01 = v0.dot(&v1);
        let dot02 = v0.dot(&v2);
        let dot11 = v1.dot(&v1);
        let dot12 = v1.dot(&v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
    }

    /// Compute circumcircle center.
    pub fn circumcenter(&self) -> Point2D {
        let ax = self.a.x;
        let ay = self.a.y;
        let bx = self.b.x;
        let by = self.b.y;
        let cx = self.c.x;
        let cy = self.c.y;

        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));

        if d.abs() < f32::EPSILON {
            // Degenerate triangle
            return self.centroid();
        }

        let a_sq = ax * ax + ay * ay;
        let b_sq = bx * bx + by * by;
        let c_sq = cx * cx + cy * cy;

        let ux = (a_sq * (by - cy) + b_sq * (cy - ay) + c_sq * (ay - by)) / d;
        let uy = (a_sq * (cx - bx) + b_sq * (ax - cx) + c_sq * (bx - ax)) / d;

        Point2D::new(ux, uy)
    }

    /// Compute circumcircle radius squared.
    pub fn circumradius_squared(&self) -> f32 {
        let center = self.circumcenter();
        center.distance_squared(&self.a)
    }
}

/// A collection of polygons forming a mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonMesh {
    pub polygons: Vec<Polygon>,
}

impl PolygonMesh {
    /// Create a new empty mesh.
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    /// Create with capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            polygons: Vec::with_capacity(capacity),
        }
    }

    /// Add a polygon and return its ID.
    pub fn add_polygon(&mut self, polygon: Polygon) -> u32 {
        let id = self.polygons.len() as u32;
        self.polygons.push(polygon);
        id
    }

    /// Get polygon by ID.
    pub fn get(&self, id: u32) -> Option<&Polygon> {
        self.polygons.get(id as usize)
    }

    /// Get mutable polygon by ID.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Polygon> {
        self.polygons.get_mut(id as usize)
    }

    /// Number of polygons.
    pub fn polygon_count(&self) -> usize {
        self.polygons.len()
    }

    /// Total vertex count.
    pub fn total_vertex_count(&self) -> usize {
        self.polygons.iter().map(|p| p.vertex_count()).sum()
    }

    /// Total area of all polygons.
    pub fn total_area(&self) -> f32 {
        self.polygons.iter().map(|p| p.area()).sum()
    }

    /// Combined bounding box of all polygons.
    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::empty();
        for poly in &self.polygons {
            bbox.expand_to_include_box(&poly.bounding_box());
        }
        bbox
    }

    /// Check if mesh is empty.
    pub fn is_empty(&self) -> bool {
        self.polygons.is_empty()
    }

    /// Find polygon containing a point.
    pub fn find_polygon_containing(&self, point: &Point2D) -> Option<u32> {
        for poly in &self.polygons {
            if poly.contains_point(point) {
                return Some(poly.id);
            }
        }
        None
    }

    /// Get all neighbors of a polygon.
    pub fn get_neighbors(&self, polygon_id: u32) -> Vec<u32> {
        self.get(polygon_id)
            .map(|p| p.neighbors.clone())
            .unwrap_or_default()
    }

    /// Connect two polygons as neighbors.
    pub fn connect_neighbors(&mut self, id_a: u32, id_b: u32) {
        if let Some(poly_a) = self.get_mut(id_a) {
            poly_a.add_neighbor(id_b);
        }
        if let Some(poly_b) = self.get_mut(id_b) {
            poly_b.add_neighbor(id_a);
        }
    }
}

impl Default for PolygonMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_operations() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);

        assert_eq!(p1.distance(&p2), 5.0);
        assert_eq!(p1.distance_squared(&p2), 25.0);

        // Test addition
        let p3 = p1 + p2;
        assert_eq!(p3.x, 3.0);
        assert_eq!(p3.y, 4.0);

        // Test scaling
        let p4 = p2 * 2.0;
        assert_eq!(p4.x, 6.0);
        assert_eq!(p4.y, 8.0);
    }

    #[test]
    fn test_point2d_normalize() {
        let p = Point2D::new(3.0, 4.0);
        let norm = p.normalize();
        assert!((norm.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(5.0, 3.0),
            Point2D::new(-1.0, 2.0),
        ];

        let bbox = BoundingBox::from_points(&points).unwrap();
        assert_eq!(bbox.min.x, -1.0);
        assert_eq!(bbox.max.x, 5.0);
        assert_eq!(bbox.center(), Point2D::new(2.0, 1.5));
    }

    #[test]
    fn test_polygon_triangle() {
        let poly = Polygon::triangle(
            0,
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 0.0),
            Point2D::new(0.0, 4.0),
        );

        assert!(poly.is_valid());
        assert!((poly.area() - 6.0).abs() < 0.001);
        assert!((poly.perimeter() - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon_centroid() {
        // Square centered at origin
        let poly = Polygon::new(
            0,
            vec![
                Point2D::new(-1.0, -1.0),
                Point2D::new(1.0, -1.0),
                Point2D::new(1.0, 1.0),
                Point2D::new(-1.0, 1.0),
            ],
        );

        let centroid = poly.centroid();
        assert!((centroid.x).abs() < 0.001);
        assert!((centroid.y).abs() < 0.001);
        assert!((poly.area() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon_point_contains() {
        let mut poly = Polygon::new(
            0,
            vec![
                Point2D::new(0.0, 0.0),
                Point2D::new(4.0, 0.0),
                Point2D::new(4.0, 4.0),
                Point2D::new(0.0, 4.0),
            ],
        );
        poly.ensure_ccw();

        assert!(poly.contains_point(&Point2D::new(2.0, 2.0)));
        assert!(!poly.contains_point(&Point2D::new(5.0, 2.0)));
    }

    #[test]
    fn test_triangle() {
        let tri = Triangle::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(4.0, 0.0),
            Point2D::new(0.0, 3.0),
        );

        assert!((tri.area() - 6.0).abs() < 0.001);

        let centroid = tri.centroid();
        assert!((centroid.x - 4.0 / 3.0).abs() < 0.001);
        assert!((centroid.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon_mesh() {
        let mut mesh = PolygonMesh::new();

        mesh.add_polygon(Polygon::triangle(
            0,
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
        ));

        mesh.add_polygon(Polygon::triangle(
            1,
            Point2D::new(1.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 1.0),
        ));

        assert_eq!(mesh.polygon_count(), 2);
        assert!((mesh.total_area() - 1.0).abs() < 0.001);

        // Connect neighbors
        mesh.connect_neighbors(0, 1);

        let neighbors = mesh.get_neighbors(0);
        assert!(neighbors.contains(&1));
    }

    #[test]
    fn test_polygon_compactness() {
        // Square has high compactness
        let square = Polygon::new(
            0,
            vec![
                Point2D::new(-1.0, -1.0),
                Point2D::new(1.0, -1.0),
                Point2D::new(1.0, 1.0),
                Point2D::new(-1.0, 1.0),
            ],
        );

        let compactness = square.compactness();
        // Square: 4π * 4 / 64 ≈ 0.785
        assert!(compactness > 0.7 && compactness < 0.8);

        // Very long thin rectangle has low compactness
        let thin = Polygon::new(
            1,
            vec![
                Point2D::new(-5.0, -0.1),
                Point2D::new(5.0, -0.1),
                Point2D::new(5.0, 0.1),
                Point2D::new(-5.0, 0.1),
            ],
        );

        let thin_compactness = thin.compactness();
        assert!(thin_compactness < compactness);
    }

    #[test]
    fn test_serialization() {
        let poly = Polygon::triangle(
            0,
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(0.0, 1.0),
        );

        let json = serde_json::to_string(&poly).unwrap();
        let restored: Polygon = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, 0);
        assert_eq!(restored.vertex_count(), 3);
    }
}
