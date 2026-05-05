//! Planet & Geography Types
//!
//! Core domain types for planetary bodies and their geographic properties.
//! These types formalize temperature, precipitation, drainage, tectonic plates,
//! and the planet container that aggregates all geographic data.
//!
//! ## Type Inventory (WOR-8)
//!
//! | Type | Kind | Description |
//! |-------|------|-------------|
//! | [`Temperature`] | scalar newtype | Mean annual temperature in °C with validation |
//! | [`Precipitation`] | scalar newtype | Annual precipitation in mm with validation |
//! | [`Drainage`] | classification | Drainage basin and flow characteristics |
//! | [`TectonicPlate`] | entity | Tectonic plate with boundary types and activity |
//! | [`TectonicBoundary`] | enum | Type of plate boundary (divergent, convergent, transform) |
//! | [`Geography`] | metadata | Geographic context for a region or cell |
//! | [`Planet`] | aggregate | Top-level planet containing all geographic systems |
//!
//! ## Relationship to Existing Types
//!
//! - `Planet` wraps the terrain system's `TerrainGrid`, `PolygonGraph`, and biome data.
//! - `Geography` is stored on `Region` (in `types.rs`) as an optional field.
//! - `Temperature` and `Precipitation` values are produced by `TerrainGenerator`
//!   (currently as raw `f32`; this module formalizes them).
//! - `Drainage` complements `hydro::DrainTarget` with basin-level classification.
//! - `TectonicPlate` is a new top-level entity replacing the config-only
//!   `TectonicSettings` in `terrain/terrain_generator.rs`.
//!
//! ## Serialization
//!
//! All types derive `Serialize`/`Deserialize` for JSON persistence, consistent
//! with the rest of the codebase.
//!
//! ## Validation Invariants
//!
//! - `Temperature::new()` rejects values outside [-90.0, 60.0] °C.
//! - `Precipitation::new()` rejects values outside [0.0, 12000.0] mm/year.
//! - `TectonicPlate::validate()` ensures the plate has at least one boundary
//!   or is marked as interior.
//! - `Planet::validate()` returns an error if no terrain data is present.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

// ============================================================================
// Temperature
// ============================================================================

/// Mean annual temperature for a geographic cell or region.
///
/// Represented in degrees Celsius (°C). The value range is bounded to the
/// physically plausible range of [-90, 60] °C (polar ice caps to equatorial deserts).
///
/// This is a **newtype** wrapper around `f32` that:
/// - Makes the unit (°C) explicit in the type system.
/// - Validates the value range on construction.
/// - Implements `Copy` for cheap cell-level storage.
///
/// ## Relationship
///
/// Produced by `TerrainGenerator::estimate_temperature()` (currently returns raw
/// `f32`). After WOR-8, generators should return `Temperature` directly.
///
/// ## Example
///
/// ```rust,ignore
/// use world_factory::world::entities::planet::Temperature;
/// 
/// let temp = Temperature::new(28.0).unwrap();   // Tropical
/// let cold = Temperature::new(-40.0).unwrap();  // Polar
/// assert!(temp > cold);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Temperature(f32);

impl Temperature {
    /// Absolute minimum plausible temperature in °C (Antarctica record).
    pub const MIN_C: f32 = -90.0;
    /// Absolute maximum plausible temperature in °C (Death Valley record).
    pub const MAX_C: f32 = 60.0;

    /// Create a new Temperature, validating the range.
    ///
    /// Returns `None` if `value` is outside `[-90.0, 60.0]`.
    #[inline]
    pub fn new(value: f32) -> Option<Self> {
        if value >= Self::MIN_C && value <= Self::MAX_C {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Create a Temperature without validation (use only when deserializing
    /// or when the caller has already validated).
    #[inline]
    pub fn from_unchecked(value: f32) -> Self {
        debug_assert!(value >= Self::MIN_C && value <= Self::MAX_C);
        Self(value)
    }

    /// Raw Celsius value.
    #[inline]
    pub fn as_celsius(&self) -> f32 {
        self.0
    }

    /// Convert to Kelvin.
    #[inline]
    pub fn as_kelvin(&self) -> f32 {
        self.0 + 273.15
    }

    /// Create from Kelvin.
    #[inline]
    pub fn from_kelvin(k: f32) -> Option<Self> {
        Self::new(k - 273.15)
    }

    /// Classify this temperature into a [`TemperatureZone`].
    #[inline]
    pub fn zone(&self) -> TemperatureZone {
        TemperatureZone::from_temperature(self.0)
    }

    /// Linear interpolation between two temperatures (0.0 = a, 1.0 = b).
    #[inline]
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self::from_unchecked(a.0 + (b.0 - a.0) * t.clamp(0.0, 1.0))
    }
}

impl Default for Temperature {
    fn default() -> Self {
        // Global default: 15°C (global mean annual temperature)
        Self(15.0)
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°C", self.0)
    }
}

impl std::ops::Add<f32> for Temperature {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::from_unchecked(self.0 + rhs)
    }
}

impl std::ops::Sub<f32> for Temperature {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::from_unchecked(self.0 - rhs)
    }
}

/// Temperature zone classification for biome assignment and climate mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureZone {
    /// Below -30°C — polar ice cap
    PolarIce,
    /// -30°C to -10°C — tundra/permafrost
    Polar,
    /// -10°C to 5°C — boreal/cold continental
    Cold,
    /// 5°C to 15°C — temperate
    Cool,
    /// 15°C to 25°C — subtropical/warm temperate
    Warm,
    /// Above 25°C — tropical
    Tropical,
}

impl TemperatureZone {
    /// Classify a temperature value (°C) into a zone.
    pub fn from_temperature(celsius: f32) -> Self {
        if celsius < -30.0 {
            Self::PolarIce
        } else if celsius < -10.0 {
            Self::Polar
        } else if celsius < 5.0 {
            Self::Cold
        } else if celsius < 15.0 {
            Self::Cool
        } else if celsius < 25.0 {
            Self::Warm
        } else {
            Self::Tropical
        }
    }
}

// ============================================================================
// Precipitation
// ============================================================================

/// Annual precipitation for a geographic cell or region.
///
/// Represented in millimeters (mm) per year. The value range covers
/// the full physically plausible range from absolute desert (0 mm) to
/// the wettest places on Earth (~12,000 mm).
///
/// This is a **newtype** wrapper around `f32` that:
/// - Makes the unit (mm/year) explicit in the type system.
/// - Validates the value range on construction.
///
/// ## Relationship
///
/// Produced by `TerrainGenerator::estimate_precipitation()` (currently returns
/// raw `f32`). After WOR-8, generators should return `Precipitation` directly.
///
/// ## Example
///
/// ```rust,ignore
/// use world_factory::world::entities::planet::Precipitation;
/// 
/// let rain = Precipitation::new(2500.0).unwrap();   // Very wet
/// let arid = Precipitation::ZERO;                  // Desert
/// assert!(rain > arid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Precipitation(f32);

impl Precipitation {
    /// Minimum precipitation (absolute desert).
    pub const MIN_MM: f32 = 0.0;
    /// Maximum precipitation (Cherrapunji / Mt. Waialeale range).
    pub const MAX_MM: f32 = 12_000.0;
    /// Global mean annual precipitation (rough estimate).
    pub const GLOBAL_MEAN_MM: f32 = 1000.0;
    /// Zero precipitation sentinel.
    pub const ZERO: Self = Self(0.0);

    /// Create a new Precipitation, validating the range.
    ///
    /// Returns `None` if `value` is outside `[0.0, 12000.0]`.
    #[inline]
    pub fn new(value: f32) -> Option<Self> {
        if value >= Self::MIN_MM && value <= Self::MAX_MM {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Create without validation.
    #[inline]
    pub fn from_unchecked(value: f32) -> Self {
        debug_assert!(value >= Self::MIN_MM && value <= Self::MAX_MM);
        Self(value)
    }

    /// Raw value in mm/year.
    #[inline]
    pub fn as_mm(&self) -> f32 {
        self.0
    }

    /// Convert to inches per year.
    #[inline]
    pub fn as_inches(&self) -> f32 {
        self.0 / 25.4
    }

    /// Create from inches per year.
    #[inline]
    pub fn from_inches(inches: f32) -> Option<Self> {
        Self::new(inches * 25.4)
    }

    /// Classify this precipitation into a [`PrecipitationZone`].
    #[inline]
    pub fn zone(&self) -> PrecipitationZone {
        PrecipitationZone::from_mm(self.0)
    }

    /// Check if this represents an arid (dry) climate.
    #[inline]
    pub fn is_arid(&self) -> bool {
        self.0 < 250.0
    }

    /// Check if this represents a humid (wet) climate.
    #[inline]
    pub fn is_humid(&self) -> bool {
        self.0 >= 1000.0
    }
}

impl Default for Precipitation {
    fn default() -> Self {
        Self(Self::GLOBAL_MEAN_MM)
    }
}

impl fmt::Display for Precipitation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}mm/yr", self.0)
    }
}

impl std::ops::Add<f32> for Precipitation {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::from_unchecked(self.0 + rhs)
    }
}

impl std::ops::Sub<f32> for Precipitation {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::from_unchecked((self.0 - rhs).max(0.0))
    }
}

/// Precipitation zone classification for biome assignment.
///
/// Based on the UNESCO/world meteorology classification scheme adapted
/// for procedural generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrecipitationZone {
    /// < 100 mm/year — hyperarid (absolute desert)
    HyperArid,
    /// 100–250 mm/year — arid (desert)
    Arid,
    /// 250–500 mm/year — semi-arid (steppe)
    SemiArid,
    /// 500–1000 mm/year — sub-humid (dry forest/savanna)
    SubHumid,
    /// 1000–2000 mm/year — humid (forest)
    Humid,
    /// > 2000 mm/year — per-humid (rainforest)
    PerHumid,
}

impl PrecipitationZone {
    /// Classify a precipitation value (mm/year) into a zone.
    pub fn from_mm(mm: f32) -> Self {
        if mm < 100.0 {
            Self::HyperArid
        } else if mm < 250.0 {
            Self::Arid
        } else if mm < 500.0 {
            Self::SemiArid
        } else if mm < 1000.0 {
            Self::SubHumid
        } else if mm < 2000.0 {
            Self::Humid
        } else {
            Self::PerHumid
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::HyperArid => "Hyper-Arid (Desert)",
            Self::Arid => "Arid (Desert)",
            Self::SemiArid => "Semi-Arid (Steppe)",
            Self::SubHumid => "Sub-Humid (Dry Forest)",
            Self::Humid => "Humid (Forest)",
            Self::PerHumid => "Per-Humid (Rainforest)",
        }
    }

    /// Aridity index (inverse of wetness): higher = drier.
    ///
    /// Range: 0.0 (per-humid) to 5.0 (hyper-arid).
    pub fn aridity_index(&self) -> f32 {
        match self {
            Self::HyperArid => 5.0,
            Self::Arid => 3.5,
            Self::SemiArid => 2.0,
            Self::SubHumid => 1.0,
            Self::Humid => 0.5,
            Self::PerHumid => 0.0,
        }
    }
}

// ============================================================================
// Drainage
// ============================================================================

/// Drainage classification for a geographic region.
///
/// Describes how water flows across and through a region — whether it
/// collects into rivers that flow to the ocean, evaporates in an
/// endorheic basin, or percolates into an aquifer.
///
/// Complements `hydro::DrainTarget` which describes where a specific
/// river terminates. `Drainage` describes the overall basin behavior
/// of a region or cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrainageType {
    /// Water flows to the ocean via rivers (exorheic).
    Exorheic,
    /// Water collects in an inland basin with no ocean outlet (endorheic).
    /// Common in desert regions (e.g., Caspian Sea, Dead Sea basins).
    Endorheic,
    /// Water percolates into groundwater/aquifer rather than surface flow.
    /// Common in karst limestone regions.
    Infiltration,
    /// No significant drainage (flat desert, polar, or underground system).
    /// Water evaporates or infiltrates locally.
    Internal,
}

impl DrainageType {
    /// Check if this drainage type produces surface rivers.
    #[inline]
    pub fn has_surface_outflow(&self) -> bool {
        matches!(self, Self::Exorheic)
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Exorheic => "Water flows to the ocean via river systems.",
            Self::Endorheic => "Inland basin with no ocean outlet; water evaporates or drains to a terminal lake.",
            Self::Infiltration => "Water percolates into groundwater; limited surface drainage.",
            Self::Internal => "Negligible drainage; water cycles locally through evaporation or soil moisture.",
        }
    }
}

/// A drainage basin — a contiguous area of land where all surface water
/// drains to a common outlet (river mouth, lake, or inland basin).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainageBasin {
    /// Unique identifier for this basin.
    pub id: Uuid,
    /// Human-readable name (e.g., "Mississippi Basin", "Aral Sea Basin").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Type of drainage for this basin.
    pub drainage_type: DrainageType,
    /// IDs of the cells/polygons belonging to this basin.
    pub cell_ids: Vec<u32>,
    /// Basin area in km².
    pub area_km2: f64,
    /// Average annual discharge at the outlet in m³/s (0 if endorheic).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_discharge_m3s: Option<f32>,
    /// The outlet cell/polygon ID (where water leaves the basin).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlet_cell_id: Option<u32>,
    /// ID of the parent basin if this is a sub-basin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_basin_id: Option<Uuid>,
}

impl DrainageBasin {
    /// Create a new drainage basin.
    pub fn new(id: Uuid, drainage_type: DrainageType, cell_ids: Vec<u32>, area_km2: f64) -> Self {
        Self {
            id,
            name: None,
            drainage_type,
            cell_ids,
            area_km2,
            mean_discharge_m3s: None,
            outlet_cell_id: None,
            parent_basin_id: None,
        }
    }

    /// Check if this is a first-order basin (no parent).
    pub fn is_primary(&self) -> bool {
        self.parent_basin_id.is_none()
    }

    /// Validate that the basin has at least one cell.
    pub fn validate(&self) -> Result<(), DrainageError> {
        if self.cell_ids.is_empty() {
            return Err(DrainageError::EmptyBasin(self.id));
        }
        if self.area_km2 <= 0.0 {
            return Err(DrainageError::InvalidArea(self.id));
        }
        Ok(())
    }
}

/// Errors that can occur during drainage operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrainageError {
    /// Basin has no cells.
    EmptyBasin(Uuid),
    /// Basin has a non-positive area.
    InvalidArea(Uuid),
    /// A cell belongs to multiple basins (overlap).
    BasinOverlap { cell_id: u32, basin_a: Uuid, basin_b: Uuid },
    /// A cell is not part of any basin (orphan).
    OrphanCell { cell_id: u32 },
}

impl fmt::Display for DrainageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBasin(id) => write!(f, "Drainage basin {} has no cells", id),
            Self::InvalidArea(id) => write!(f, "Drainage basin {} has invalid area", id),
            Self::BasinOverlap { cell_id, basin_a, basin_b } => {
                write!(f, "Cell {} belongs to both basin {} and {}", cell_id, basin_a, basin_b)
            }
            Self::OrphanCell { cell_id } => {
                write!(f, "Cell {} is not part of any drainage basin", cell_id)
            }
        }
    }
}

impl std::error::Error for DrainageError {}

// ============================================================================
// Tectonic Plate
// ============================================================================

/// A tectonic plate — a massive segment of Earth's lithosphere that
/// moves relative to other plates.
///
/// In World Factory, tectonic plates are generated procedurally and
/// drive mountain building, earthquake zones, and volcanic activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TectonicPlate {
    /// Unique identifier for this plate.
    pub id: Uuid,
    /// Human-readable name (e.g., "Indo-Australian Plate", "Pacific Plate").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Movement direction in degrees (0 = North, 90 = East).
    pub movement_direction_deg: f32,
    /// Relative movement speed in cm/year.
    pub movement_speed_cm_yr: f32,
    /// The type of this plate.
    pub plate_type: TectonicPlateType,
    /// Boundary segments where this plate meets others.
    pub boundaries: Vec<TectonicBoundary>,
    /// IDs of cells/polygons belonging to this plate.
    pub cell_ids: Vec<u32>,
    /// Plate area in km².
    pub area_km2: f64,
}

impl TectonicPlate {
    /// Create a new tectonic plate.
    pub fn new(
        id: Uuid,
        movement_direction_deg: f32,
        movement_speed_cm_yr: f32,
        plate_type: TectonicPlateType,
        cell_ids: Vec<u32>,
        area_km2: f64,
    ) -> Self {
        Self {
            id,
            name: None,
            movement_direction_deg: movement_direction_deg.rem_euclid(360.0),
            movement_speed_cm_yr: movement_speed_cm_yr.max(0.0),
            plate_type,
            boundaries: Vec::new(),
            cell_ids,
            area_km2: area_km2.max(0.0),
        }
    }

    /// Get the plate's velocity vector (cm/year) as (dx, dy).
    pub fn velocity_vector(&self) -> (f32, f32) {
        let rad = self.movement_direction_deg.to_radians();
        (
            self.movement_speed_cm_yr * rad.sin(),
            self.movement_speed_cm_yr * rad.cos(),
        )
    }

    /// Add a boundary segment to this plate.
    pub fn add_boundary(&mut self, boundary: TectonicBoundary) {
        self.boundaries.push(boundary);
    }

    /// Check if this is a continental plate (has continental crust).
    pub fn is_continental(&self) -> bool {
        matches!(self.plate_type, TectonicPlateType::Continental)
    }

    /// Validate the plate's integrity.
    pub fn validate(&self) -> Result<(), TectonicError> {
        if self.cell_ids.is_empty() {
            return Err(TectonicError::EmptyPlate(self.id));
        }
        if self.area_km2 <= 0.0 {
            return Err(TectonicError::InvalidArea(self.id));
        }
        if self.movement_speed_cm_yr > 20.0 {
            // Sanity check: fastest plate on Earth moves ~18 cm/year
            return Err(TectonicError::UnrealisticSpeed(self.id, self.movement_speed_cm_yr));
        }
        Ok(())
    }

    /// Calculate the orogenic (mountain-building) intensity along this plate's
    /// convergent boundaries, expressed as a score in [0.0, 1.0].
    pub fn orogenic_intensity(&self) -> f32 {
        let convergent_count = self
            .boundaries
            .iter()
            .filter(|b| matches!(b.boundary_type, TectonicBoundaryType::Convergent { .. }))
            .count() as f32;

        // Normalize: 0 convergent boundaries = 0.0, 4+ = 1.0
        (convergent_count / 4.0).clamp(0.0, 1.0)
    }
}

/// Kinds of tectonic plates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TectonicPlateType {
    /// Primarily continental lithosphere.
    Continental,
    /// Primarily oceanic lithosphere.
    Oceanic,
    /// Mixed continental-oceanic plate (e.g., South American Plate).
    Mixed,
}

/// Type of tectonic boundary between two plates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TectonicBoundaryType {
    /// Plates move apart — creates mid-ocean ridges, rift valleys.
    /// Associated: volcanism, shallow earthquakes.
    Divergent {
        /// Spreading rate in cm/year.
        spreading_rate_cm_yr: f32,
    },
    /// Plates move toward each other.
    /// Creates: trenches, mountain ranges, volcanic arcs.
    /// Associated: deep earthquakes, tsunamis.
    Convergent {
        /// Subduction rate in cm/year (positive = one plate subducts).
        subduction_rate_cm_yr: f32,
        /// Which plate subducts (if applicable): `None` = obduction/thrust.
        subducting_plate: Option<Uuid>,
        /// Whether this is an oceanic-continental, oceanic-oceanic, or
        /// continental-continental convergence.
        subduction_type: SubductionType,
    },
    /// Plates slide past each other horizontally.
    /// Associated: transform faults, strike-slip earthquakes.
    Transform {
        /// Strike-slip rate in cm/year.
        slip_rate_cm_yr: f32,
    },
    /// Conservative margin — deformation but no plate creation/destruction.
    Conservative {
        /// Deformation rate in cm/year.
        deformation_rate_cm_yr: f32,
    },
}

impl TectonicBoundaryType {
    /// Check if this boundary type is seismically active.
    pub fn is_seismically_active(&self) -> bool {
        !matches!(self, Self::Conservative { .. })
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Divergent { .. } => "Divergent Boundary",
            Self::Convergent { .. } => "Convergent Boundary",
            Self::Transform { .. } => "Transform Boundary",
            Self::Conservative { .. } => "Conservative Margin",
        }
    }
}

/// Subduction configuration for convergent boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubductionType {
    /// Oceanic plate subducts beneath continental plate.
    OceanicUnderContinental,
    /// Two oceanic plates collide; older/denser one subducts.
    OceanicUnderOceanic,
    /// Two continental plates collide; neither subducts (Himalayan-type).
    ContinentalUnderContinental,
}

/// A boundary segment between two tectonic plates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TectonicBoundary {
    /// Unique identifier for this boundary segment.
    pub id: Uuid,
    /// The type of boundary.
    pub boundary_type: TectonicBoundaryType,
    /// IDs of the two plates this boundary separates.
    pub plate_ids: [Uuid; 2],
    /// Cell/polygon IDs that form this boundary segment.
    pub cell_ids: Vec<u32>,
    /// Approximate total length of this boundary in km.
    pub length_km: f64,
    /// Volcanic activity level along this boundary [0.0, 1.0].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volcanic_activity: Option<f32>,
    /// Seismic activity level along this boundary [0.0, 1.0].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seismic_activity: Option<f32>,
}

impl TectonicBoundary {
    /// Create a new boundary.
    pub fn new(
        id: Uuid,
        boundary_type: TectonicBoundaryType,
        plate_ids: [Uuid; 2],
        cell_ids: Vec<u32>,
        length_km: f64,
    ) -> Self {
        Self {
            id,
            boundary_type,
            plate_ids,
            cell_ids,
            length_km: length_km.max(0.0),
            volcanic_activity: None,
            seismic_activity: None,
        }
    }

    /// Check if this is a volcanic boundary.
    pub fn is_volcanic(&self) -> bool {
        matches!(
            self.boundary_type,
            TectonicBoundaryType::Divergent { .. }
                | TectonicBoundaryType::Convergent {
                    subduction_type: SubductionType::OceanicUnderContinental
                        | SubductionType::OceanicUnderOceanic,
                    ..
                }
        )
    }

    /// Check if this is a subduction zone.
    pub fn is_subduction_zone(&self) -> bool {
        matches!(
            self.boundary_type,
            TectonicBoundaryType::Convergent {
                subduction_type: SubductionType::OceanicUnderContinental
                    | SubductionType::OceanicUnderOceanic,
                ..
            }
        )
    }
}

/// Errors during tectonic operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TectonicError {
    /// Plate has no cells assigned.
    EmptyPlate(Uuid),
    /// Plate has invalid area.
    InvalidArea(Uuid),
    /// Plate movement speed is physically implausible.
    UnrealisticSpeed(Uuid, f32),
    /// Boundary references unknown plate.
    UnknownPlate(Uuid),
    /// Boundary has no cells.
    EmptyBoundary(Uuid),
}

impl fmt::Display for TectonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPlate(id) => write!(f, "Tectonic plate {} has no cells", id),
            Self::InvalidArea(id) => write!(f, "Tectonic plate {} has invalid area", id),
            Self::UnrealisticSpeed(id, speed) => {
                write!(f, "Tectonic plate {} has unrealistic speed {} cm/yr", id, speed)
            }
            Self::UnknownPlate(id) => write!(f, "Tectonic boundary references unknown plate {}", id),
            Self::EmptyBoundary(id) => write!(f, "Tectonic boundary {} has no cells", id),
        }
    }
}

impl std::error::Error for TectonicError {}

// ============================================================================
// Geography
// ============================================================================

/// Geographic metadata for a region or terrain cell.
///
/// This struct provides a comprehensive geographic profile for any
/// geographic unit (region, province, or terrain cell) by combining
/// climate, terrain, and hydrological data.
///
/// It is stored as an optional field on `Region` (in `types.rs`) and
/// computed during world generation.
///
/// ## Example
///
/// ```rust,ignore
/// use world_factory::world::entities::planet::{
///     Geography, Temperature, Precipitation, DrainageType, ElevationZone
/// };
/// 
/// let geo = Geography::new(
///     Temperature::new(22.0).unwrap(),
///     Precipitation::new(1800.0).unwrap(),
///     DrainageType::Exorheic,
///     ElevationZone::Lowland,
///     45.0,  // latitude
/// );
/// assert!(geo.is_temperate());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geography {
    /// Mean annual temperature.
    pub temperature: Temperature,
    /// Total annual precipitation in mm.
    pub precipitation: Precipitation,
    /// Drainage classification.
    pub drainage_type: DrainageType,
    /// Elevation zone.
    pub elevation_zone: ElevationZone,
    /// Latitude in degrees (-90 to 90). Used for climate zone derivation.
    pub latitude_deg: f32,
    /// Surface roughness [0.0 (flat) to 1.0 (mountainous)].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,
    /// Soil type classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soil_type: Option<SoilType>,
    /// Freshwater availability index [0.0 (scarce) to 1.0 (abundant)].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshwater_index: Option<f32>,
}

impl Geography {
    /// Create a new Geography profile.
    pub fn new(
        temperature: Temperature,
        precipitation: Precipitation,
        drainage_type: DrainageType,
        elevation_zone: ElevationZone,
        latitude_deg: f32,
    ) -> Self {
        Self {
            temperature,
            precipitation,
            drainage_type,
            elevation_zone,
            latitude_deg: latitude_deg.clamp(-90.0, 90.0),
            roughness: None,
            soil_type: None,
            freshwater_index: None,
        }
    }

    /// Derive the climate zone from latitude.
    pub fn climate_zone(&self) -> ClimateZone {
        ClimateZone::from_latitude(self.latitude_deg)
    }

    /// Derive the combined moisture+temperature climate classification.
    pub fn climate_classification(&self) -> ClimateClassification {
        ClimateClassification::from_geo(self)
    }

    /// Check if this is a temperate climate (5–25°C, moderate precipitation).
    pub fn is_temperate(&self) -> bool {
        let t = self.temperature.as_celsius();
        t >= 5.0 && t <= 25.0 && self.precipitation.as_mm() >= 500.0
    }

    /// Check if this is an arid region (precipitation < 250 mm/year).
    pub fn is_arid(&self) -> bool {
        self.precipitation.is_arid()
    }

    /// Check if this is a high-elevation region (above 1500m).
    pub fn is_highland(&self) -> bool {
        matches!(
            self.elevation_zone,
            ElevationZone::Highland | ElevationZone::Alpine | ElevationZone::Nival
        )
    }

    /// Estimated population capacity of this region based on geography.
    /// Returns a rough carrying capacity in people per km².
    pub fn carrying_capacity(&self) -> f64 {
        use ElevationZone::*;
        use DrainageType::*;
        use TemperatureZone::*;

        let zone_cap = match self.temperature.zone() {
            PolarIce | Polar => 0.1,  // Near zero
            Cold => 5.0,
            Cool => 50.0,
            Warm => 100.0,
            Tropical => 150.0,
        };

        let precip_factor = match self.precipitation.zone() {
            PrecipitationZone::HyperArid | PrecipitationZone::Arid => 0.1,
            PrecipitationZone::SemiArid => 0.5,
            PrecipitationZone::SubHumid => 0.8,
            PrecipitationZone::Humid => 1.0,
            PrecipitationZone::PerHumid => 0.9, // Too wet reduces capacity
        };

        let elev_factor = match self.elevation_zone {
            Lowland => 1.0,
            Midland => 0.8,
            Highland => 0.5,
            Alpine => 0.2,
            Nival => 0.0,
        };

        let drainage_factor = match self.drainage_type {
            Exorheic => 1.0,
            Endorheic => 0.7,
            Infiltration => 0.6,
            Internal => 0.3,
        };

        zone_cap * precip_factor * elev_factor * drainage_factor
    }
}

/// Combined climate classification based on temperature and precipitation.
///
/// This is a simplified Holdridge Life Zone classification derived from
/// the temperature, precipitation, and latitude of a region.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClimateClassification {
    TropicalRainforest,
    TropicalSeasonalForest,
    TropicalSavanna,
    TropicalDryForest,
    SubtropicalWetForest,
    SubtropicalDryForest,
    TemperateForest,
    TemperateGrassland,
    TemperateDesert,
    BorealForest,
    Tundra,
    PolarDesert,
    Montane,
}

impl ClimateClassification {
    /// Derive the climate classification from a Geography profile.
    pub fn from_geo(geo: &Geography) -> Self {
        use TemperatureZone::*;
        use PrecipitationZone::*;
        use ElevationZone::*;

        let t = geo.temperature.zone();
        let p = geo.precipitation.zone();
        let e = &geo.elevation_zone;

        if matches!(e, Alpine | Nival) {
            return Self::Montane;
        }

        match t {
            PolarIce | Polar => Self::PolarDesert,
            Cold => {
                if matches!(p, Humid | PerHumid) {
                    Self::BorealForest
                } else {
                    Self::Tundra
                }
            }
            Cool | Warm => {
                match p {
                    HyperArid | Arid => Self::TemperateDesert,
                    SemiArid => Self::TemperateGrassland,
                    SubHumid => Self::TemperateForest,
                    Humid | PerHumid => Self::TemperateForest,
                }
            }
            Tropical => {
                match p {
                    HyperArid | Arid => Self::TropicalDryForest,
                    SemiArid => Self::TropicalSavanna,
                    SubHumid => Self::TropicalSeasonalForest,
                    Humid | PerHumid => Self::TropicalRainforest,
                }
            }
        }
    }
}

/// Soil type classification for agricultural/settlement suitability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoilType {
    Alluvial,      // River delta / floodplain — very fertile
    Clay,          // Heavy clay — poor drainage
    Sandy,         // Sandy — good drainage, low nutrients
    Loamy,         // Balanced — ideal for agriculture
    Volcanic,      // Volcanic ash — very fertile
    Chalky,        // High pH — poor for most crops
    Peaty,         // Organic-rich — acidic, wetland
    Rocky,         // Mountain/stony — low suitability
    Limestone,     // Karst-forming — variable
}

/// Extension trait to derive ClimateZone from latitude.
pub trait ClimateZoneFromLatitude {
    fn from_latitude(latitude_deg: f32) -> ClimateZone;
}

impl ClimateZoneFromLatitude for ClimateZone {
    /// Derive climate zone from absolute latitude in degrees.
    fn from_latitude(latitude_deg: f32) -> ClimateZone {
        let abs_lat = latitude_deg.abs();
        if abs_lat < 23.5 {
            Self::Tropical
        } else if abs_lat < 35.0 {
            Self::Subtropical
        } else if abs_lat < 55.0 {
            Self::Temperate
        } else if abs_lat < 65.0 {
            Self::Boreal
        } else {
            Self::Polar
        }
    }
}

// Re-export ElevationZone from terrain for use in this module.
// Note: terrain module defines ElevationZone; this re-export makes it
// available without a fully-qualified path in Geography::new().
pub use crate::terrain::ElevationZone;
pub use crate::terrain::ClimateZone;

// ============================================================================
// Planet
// ============================================================================

/// The top-level planet entity — a complete geographic world.
///
/// `Planet` aggregates all geographic systems: terrain, climate,
/// tectonic plates, drainage basins, and derived data structures.
///
/// A `World` (from `types.rs`) represents the narrative/societal layer;
/// a `Planet` represents the physical/geographic layer. A `World`
/// references exactly one `Planet`.
///
/// ## Type Relationships
///
/// ```text
/// World (types.rs)         Planet (this module)
///      │                         │
///      │ references ──────────►  │
///      │                         ├── TerrainGrid (terrain/)
///      │                         ├── PolygonGraph (terrain/elevation)
///      │                         ├── Vec<TectonicPlate>
///      │                         ├── Vec<DrainageBasin>
///      │                         └── Geography (per-region metadata)
/// ```
///
/// ## Serialization
///
/// Planet serializes to a large JSON object. For performance, terrain
/// grid data is stored separately and referenced by ID.
///
/// ## Validation
///
/// `Planet::validate()` checks:
/// - At least one tectonic plate is defined.
/// - All drainage basins are non-overlapping and cover all cells.
/// - All referenced terrain IDs resolve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Planet {
    /// Unique identifier for this planet.
    pub id: Uuid,
    /// Human-readable name (e.g., "Azeroth", "Kepler-442b").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Dimensions of the underlying terrain grid.
    pub terrain_dimensions: PlanetDimensions,
    /// Sea level elevation in meters (elevation values below this are ocean).
    pub sea_level_m: f32,
    /// All tectonic plates for this planet.
    pub tectonic_plates: Vec<TectonicPlate>,
    /// All drainage basins for this planet.
    pub drainage_basins: Vec<DrainageBasin>,
    /// World seed used for deterministic generation.
    pub seed: u64,
    /// Planet radius in km (Earth = 6371 km).
    /// Used for area calculations and scale normalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius_km: Option<f64>,
    /// Axial tilt in degrees (Earth = 23.5°).
    /// Determines seasonality.
    pub axial_tilt_deg: f32,
    /// Rotation period in hours (Earth = 24h).
    pub rotation_period_h: f32,
    /// Orbital period in days (Earth = 365.25 days).
    pub orbital_period_d: f32,
    /// Mean surface gravity in m/s² (Earth = 9.81 m/s²).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gravity_m_s2: Option<f32>,
    /// Planet mass relative to Earth (Earth = 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass_earths: Option<f64>,
    /// Has surface water (affects erosion, climate, habitability).
    pub has_surface_water: bool,
    /// Has an active magnetic field (affects radiation shielding).
    pub has_magnetic_field: bool,
    /// Has active volcanism / tectonics.
    pub is_geologically_active: bool,
}

impl Planet {
    /// Create a new planet with required fields.
    pub fn new(
        id: Uuid,
        terrain_dimensions: PlanetDimensions,
        sea_level_m: f32,
        tectonic_plates: Vec<TectonicPlate>,
        drainage_basins: Vec<DrainageBasin>,
        seed: u64,
        axial_tilt_deg: f32,
        rotation_period_h: f32,
        orbital_period_d: f32,
    ) -> Self {
        Self {
            id,
            name: None,
            terrain_dimensions,
            sea_level_m,
            tectonic_plates,
            drainage_basins,
            seed,
            radius_km: None,
            axial_tilt_deg: axial_tilt_deg.clamp(0.0, 90.0),
            rotation_period_h,
            orbital_period_d,
            gravity_m_s2: None,
            mass_earths: None,
            has_surface_water: true,
            has_magnetic_field: true,
            is_geologically_active: true,
        }
    }

    /// Earth-like planet preset with sensible defaults.
    pub fn earth_like(id: Uuid, terrain_dimensions: PlanetDimensions, seed: u64) -> Self {
        Self::new(
            id,
            terrain_dimensions,
            0.0, // sea level at 0m
            Vec::new(), // tectonic plates — generated separately
            Vec::new(), // drainage basins — generated separately
            seed,
            23.5,  // axial tilt
            24.0,  // rotation period
            365.25, // orbital period
        )
    }

    /// Get the total surface area in km².
    ///
    /// Uses the Hadley equation: A = 4πr².
    /// Returns 0 if `radius_km` is not set.
    pub fn surface_area_km2(&self) -> Option<f64> {
        self.radius_km.map(|r| 4.0 * std::f64::consts::PI * r * r)
    }

    /// Get the land area fraction (cells above sea level / total cells).
    ///
    /// Requires terrain data to be available.
    pub fn land_fraction(&self, land_cell_count: u32, total_cell_count: u32) -> f32 {
        if total_cell_count == 0 {
            0.0
        } else {
            land_cell_count as f32 / total_cell_count as f32
        }
    }

    /// Get a tectonic plate by ID.
    pub fn get_plate(&self, id: Uuid) -> Option<&TectonicPlate> {
        self.tectonic_plates.iter().find(|p| p.id == id)
    }

    /// Get a tectonic plate by ID (mutable).
    pub fn get_plate_mut(&mut self, id: Uuid) -> Option<&mut TectonicPlate> {
        self.tectonic_plates.iter_mut().find(|p| p.id == id)
    }

    /// Get a drainage basin by ID.
    pub fn get_basin(&self, id: Uuid) -> Option<&DrainageBasin> {
        self.drainage_basins.iter().find(|b| b.id == id)
    }

    /// Validate the planet's internal consistency.
    pub fn validate(&self) -> Result<(), PlanetValidationError> {
        if self.terrain_dimensions.width == 0 || self.terrain_dimensions.height == 0 {
            return Err(PlanetValidationError::InvalidDimensions);
        }
        if self.tectonic_plates.is_empty() {
            return Err(PlanetValidationError::NoTectonicPlates);
        }
        // Check for duplicate plate IDs
        let plate_ids: std::collections::HashSet<_> =
            self.tectonic_plates.iter().map(|p| p.id).collect();
        if plate_ids.len() != self.tectonic_plates.len() {
            return Err(PlanetValidationError::DuplicatePlateId);
        }
        // Check for duplicate basin IDs
        let basin_ids: std::collections::HashSet<_> =
            self.drainage_basins.iter().map(|b| b.id).collect();
        if basin_ids.len() != self.drainage_basins.len() {
            return Err(PlanetValidationError::DuplicateBasinId);
        }
        Ok(())
    }

    /// Calculate orogenic (mountain-building) potential for the whole planet.
    /// Returns the fraction of convergent plate boundaries [0.0, 1.0].
    pub fn orogenic_potential(&self) -> f32 {
        if self.tectonic_plates.is_empty() {
            return 0.0;
        }
        let total_intensity: f32 = self
            .tectonic_plates
            .iter()
            .map(|p| p.orogenic_intensity())
            .sum();
        total_intensity / self.tectonic_plates.len() as f32
    }
}

/// Physical dimensions of the planet's terrain grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlanetDimensions {
    /// Grid width in cells.
    pub width: u32,
    /// Grid height in cells.
    pub height: u32,
    /// Cell size in meters (each cell represents this much distance).
    pub cell_size_m: f32,
}

impl Eq for PlanetDimensions {}

impl PartialEq for PlanetDimensions {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.cell_size_m.to_bits() == other.cell_size_m.to_bits()
    }
}

impl Hash for PlanetDimensions {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        self.cell_size_m.to_bits().hash(state);
    }
}

impl PlanetDimensions {
    /// Create new dimensions.
    pub fn new(width: u32, height: u32, cell_size_m: f32) -> Self {
        Self {
            width,
            height,
            cell_size_m: cell_size_m.max(0.0),
        }
    }

    /// Total number of cells.
    #[inline]
    pub fn cell_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Physical width of the planet surface in km.
    pub fn width_km(&self) -> f64 {
        (self.width as f64 * self.cell_size_m as f64) / 1000.0
    }

    /// Physical height of the planet surface in km.
    pub fn height_km(&self) -> f64 {
        (self.height as f64 * self.cell_size_m as f64) / 1000.0
    }

    /// Physical area in km² (approximate — assumes flat projection).
    pub fn area_km2(&self) -> f64 {
        self.width_km() * self.height_km()
    }
}

impl Default for PlanetDimensions {
    fn default() -> Self {
        // Earth-like defaults: 256x256 grid, 1km cells
        Self {
            width: 256,
            height: 256,
            cell_size_m: 1000.0,
        }
    }
}

/// Validation errors for planet consistency checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanetValidationError {
    /// Terrain dimensions are zero or invalid.
    InvalidDimensions,
    /// No tectonic plates defined.
    NoTectonicPlates,
    /// Duplicate tectonic plate IDs found.
    DuplicatePlateId,
    /// Duplicate drainage basin IDs found.
    DuplicateBasinId,
    /// A cell belongs to multiple tectonic plates.
    PlateCellOverlap { cell_id: u32, plate_a: Uuid, plate_b: Uuid },
    /// A cell is not covered by any tectonic plate.
    OrphanCell { cell_id: u32 },
}

impl fmt::Display for PlanetValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => write!(f, "Planet has invalid terrain dimensions"),
            Self::NoTectonicPlates => write!(f, "Planet has no tectonic plates defined"),
            Self::DuplicatePlateId => write!(f, "Planet has duplicate tectonic plate IDs"),
            Self::DuplicateBasinId => write!(f, "Planet has duplicate drainage basin IDs"),
            Self::PlateCellOverlap { cell_id, plate_a, plate_b } => {
                write!(f, "Cell {} belongs to both plate {} and {}", cell_id, plate_a, plate_b)
            }
            Self::OrphanCell { cell_id } => {
                write!(f, "Cell {} is not covered by any tectonic plate", cell_id)
            }
        }
    }
}

impl std::error::Error for PlanetValidationError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Temperature ──────────────────────────────────────────────────────────

    #[test]
    fn test_temperature_bounds() {
        assert!(Temperature::new(-100.0).is_none());
        assert!(Temperature::new(-90.0).is_some());
        assert!(Temperature::new(60.0).is_some());
        assert!(Temperature::new(70.0).is_none());
    }

    #[test]
    fn test_temperature_zone() {
        assert_eq!(Temperature::new(-50.0).unwrap().zone(), TemperatureZone::PolarIce);
        assert_eq!(Temperature::new(-20.0).unwrap().zone(), TemperatureZone::Polar);
        assert_eq!(Temperature::new(0.0).unwrap().zone(), TemperatureZone::Cold);
        assert_eq!(Temperature::new(10.0).unwrap().zone(), TemperatureZone::Cool);
        assert_eq!(Temperature::new(20.0).unwrap().zone(), TemperatureZone::Warm);
        assert_eq!(Temperature::new(30.0).unwrap().zone(), TemperatureZone::Tropical);
    }

    #[test]
    fn test_temperature_kelvin() {
        let t = Temperature::new(0.0).unwrap();
        assert!((t.as_kelvin() - 273.15).abs() < 0.001);
        assert_eq!(Temperature::from_kelvin(273.15).unwrap().as_celsius(), 0.0);
    }

    // ── Precipitation ───────────────────────────────────────────────────────

    #[test]
    fn test_precipitation_bounds() {
        assert!(Precipitation::new(-1.0).is_none());
        assert!(Precipitation::new(0.0).is_some());
        assert!(Precipitation::new(12_000.0).is_some());
        assert!(Precipitation::new(15_000.0).is_none());
    }

    #[test]
    fn test_precipitation_zone() {
        assert_eq!(Precipitation::new(50.0).unwrap().zone(), PrecipitationZone::HyperArid);
        assert_eq!(Precipitation::new(200.0).unwrap().zone(), PrecipitationZone::Arid);
        assert_eq!(Precipitation::new(400.0).unwrap().zone(), PrecipitationZone::SemiArid);
        assert_eq!(Precipitation::new(800.0).unwrap().zone(), PrecipitationZone::SubHumid);
        assert_eq!(Precipitation::new(1500.0).unwrap().zone(), PrecipitationZone::Humid);
        assert_eq!(Precipitation::new(3000.0).unwrap().zone(), PrecipitationZone::PerHumid);
    }

    #[test]
    fn test_precipitation_aridity() {
        assert!(Precipitation::new(100.0).unwrap().is_arid());
        assert!(!Precipitation::new(1500.0).unwrap().is_arid());
        assert!(Precipitation::new(1500.0).unwrap().is_humid());
        assert!(!Precipitation::new(200.0).unwrap().is_humid());
    }

    #[test]
    fn test_precipitation_inches() {
        let p = Precipitation::new(254.0).unwrap();
        assert!((p.as_inches() - 10.0).abs() < 0.1);
        assert_eq!(Precipitation::from_inches(10.0).unwrap().as_mm(), 254.0);
    }

    // ── Drainage ────────────────────────────────────────────────────────────

    #[test]
    fn test_drainage_type_outflow() {
        assert!(DrainageType::Exorheic.has_surface_outflow());
        assert!(!DrainageType::Endorheic.has_surface_outflow());
        assert!(!DrainageType::Infiltration.has_surface_outflow());
        assert!(!DrainageType::Internal.has_surface_outflow());
    }

    #[test]
    fn test_drainage_basin_validation() {
        let bad = DrainageBasin::new(Uuid::new_v4(), DrainageType::Exorheic, vec![], 1000.0);
        assert!(bad.validate().is_err());

        let good = DrainageBasin::new(Uuid::new_v4(), DrainageType::Exorheic, vec![1, 2, 3], 1000.0);
        assert!(good.validate().is_ok());
    }

    // ── Tectonic Plate ─────────────────────────────────────────────────────

    #[test]
    fn test_tectonic_plate_velocity() {
        let plate = TectonicPlate::new(
            Uuid::new_v4(),
            90.0, // East
            5.0,  // 5 cm/year
            TectonicPlateType::Oceanic,
            vec![1, 2, 3],
            50_000_000.0,
        );
        let (dx, dy) = plate.velocity_vector();
        assert!((dx - 5.0).abs() < 0.001); // sin(90°) = 1
        assert!(dy.abs() < 0.001); // cos(90°) = 0
    }

    #[test]
    fn test_tectonic_plate_orogenic() {
        let mut plate = TectonicPlate::new(
            Uuid::new_v4(),
            0.0,
            3.0,
            TectonicPlateType::Continental,
            vec![1, 2],
            30_000_000.0,
        );
        // No boundaries = 0 orogenic intensity
        assert_eq!(plate.orogenic_intensity(), 0.0);

        // Add a convergent boundary
        plate.add_boundary(TectonicBoundary::new(
            Uuid::new_v4(),
            TectonicBoundaryType::Convergent {
                subduction_rate_cm_yr: 5.0,
                subducting_plate: None,
                subduction_type: SubductionType::OceanicUnderContinental,
            },
            [plate.id, Uuid::new_v4()],
            vec![1],
            500.0,
        ));
        assert!(plate.orogenic_intensity() > 0.0);
    }

    #[test]
    fn test_tectonic_plate_validation() {
        let bad_speed = TectonicPlate::new(
            Uuid::new_v4(), 0.0, 50.0, TectonicPlateType::Continental, vec![1], 1_000_000.0,
        );
        assert!(matches!(bad_speed.validate(), Err(TectonicError::UnrealisticSpeed(_, 50.0))));

        let good = TectonicPlate::new(
            Uuid::new_v4(), 0.0, 5.0, TectonicPlateType::Continental, vec![1], 1_000_000.0,
        );
        assert!(good.validate().is_ok());
    }

    #[test]
    fn test_boundary_is_volcanic() {
        let divergent = TectonicBoundary::new(
            Uuid::new_v4(),
            TectonicBoundaryType::Divergent { spreading_rate_cm_yr: 2.0 },
            [Uuid::new_v4(), Uuid::new_v4()],
            vec![1],
            1000.0,
        );
        assert!(divergent.is_volcanic());

        let transform = TectonicBoundary::new(
            Uuid::new_v4(),
            TectonicBoundaryType::Transform { slip_rate_cm_yr: 3.0 },
            [Uuid::new_v4(), Uuid::new_v4()],
            vec![1],
            200.0,
        );
        assert!(!transform.is_volcanic());
    }

    // ── Geography ──────────────────────────────────────────────────────────

    #[test]
    fn test_geography_carrying_capacity() {
        // Temperate lowland exorheic should have decent capacity
        let geo = Geography::new(
            Temperature::new(15.0).unwrap(),
            Precipitation::new(1000.0).unwrap(),
            DrainageType::Exorheic,
            ElevationZone::Lowland,
            50.0,
        );
        assert!(geo.carrying_capacity() > 0.0);

        // Polar nival should be near zero
        let polar = Geography::new(
            Temperature::new(-40.0).unwrap(),
            Precipitation::new(200.0).unwrap(),
            DrainageType::Internal,
            ElevationZone::Nival,
            85.0,
        );
        assert!(polar.carrying_capacity() < 1.0);
    }

    #[test]
    fn test_climate_zone_from_latitude() {
        assert_eq!(ClimateZone::from_latitude(10.0), ClimateZone::Tropical);
        assert_eq!(ClimateZone::from_latitude(30.0), ClimateZone::Subtropical);
        assert_eq!(ClimateZone::from_latitude(45.0), ClimateZone::Temperate);
        assert_eq!(ClimateZone::from_latitude(60.0), ClimateZone::Boreal);
        assert_eq!(ClimateZone::from_latitude(80.0), ClimateZone::Polar);
    }

    // ── Planet ─────────────────────────────────────────────────────────────

    #[test]
    fn test_planet_dimensions() {
        let dims = PlanetDimensions::new(100, 100, 1000.0);
        assert_eq!(dims.cell_count(), 10_000);
        assert!((dims.width_km() - 100.0).abs() < 0.001);
        assert!((dims.area_km2() - 10_000.0).abs() < 0.001);
    }

    #[test]
    fn test_planet_validation() {
        let planet = Planet::new(
            Uuid::new_v4(),
            PlanetDimensions::new(100, 100, 1000.0),
            0.0,
            vec![], // No plates — should fail
            vec![],
            42,
            23.5,
            24.0,
            365.25,
        );
        assert!(matches!(
            planet.validate(),
            Err(PlanetValidationError::NoTectonicPlates)
        ));
    }

    #[test]
    fn test_planet_earth_preset() {
        let dims = PlanetDimensions::new(256, 256, 1000.0);
        let planet = Planet::earth_like(Uuid::new_v4(), dims, 12345);
        assert_eq!(planet.axial_tilt_deg, 23.5);
        assert_eq!(planet.rotation_period_h, 24.0);
        assert!(planet.has_surface_water);
        assert!(planet.has_magnetic_field);
    }
}
