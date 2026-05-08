//! Groundwater and aquifer system
//! 
//! Models subsurface water storage, flow, and recharge for realistic hydrological simulation.
//! 
//! Aquifer types implemented:
//! - Porous (sand, gravel) - high porosity, slow flow
//! - Fractured (rock) - low porosity, fast flow through fractures  
//! - Karst (limestone) - variable porosity, conduit flow
//! - Volcanic - high permeability in lava rock
//! 
//! The system tracks:
//! - Aquifer storage capacity and current volume
//! - Recharge rates from precipitation
//! - Baseflow contribution to rivers/streams
//! - Seasonal water table fluctuations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;

/// Aquifer classification based on geology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AquiferType {
    /// Porous media aquifers (sand, gravel, unconsolidated sediments)
    /// High porosity, slow groundwater flow
    Porous,
    /// Fractured rock aquifers (igneous, metamorphic basement)
    /// Low porosity, flow concentrated in fractures
    Fractured,
    /// Karst aquifers (limestone, dolomite)
    /// Solution-enhanced porosity, conduit flow
    Karst,
    /// Volcanic aquifers (lava flows, pyroclastic)
    /// High permeability zones in fractured volcanic rock
    Volcanic,
}

impl AquiferType {
    /// Base porosity percentage (void space)
    pub fn base_porosity(&self) -> f32 {
        match self {
            AquiferType::Porous => 0.25,      // 25% for sand/gravel
            AquiferType::Fractured => 0.02,   // 2% for fractured rock
            AquiferType::Karst => 0.10,      // 10% average for karst
            AquiferType::Volcanic => 0.15,   // 15% for volcanic rock
        }
    }
    
    /// Hydraulic conductivity multiplier (relative flow speed)
    pub fn hydraulic_conductivity(&self) -> f32 {
        match self {
            AquiferType::Porous => 1.0,       // Reference
            AquiferType::Fractured => 0.1,    // 10x slower through fractures
            AquiferType::Karst => 10.0,      // 10x faster in conduits
            AquiferType::Volcanic => 5.0,    // 5x faster in lava cracks
        }
    }
    
    /// Storage coefficient (specific yield)
    pub fn storage_coefficient(&self) -> f32 {
        match self {
            AquiferType::Porous => 0.20,      // High specific yield
            AquiferType::Fractured => 0.01,   // Low specific yield
            AquiferType::Karst => 0.08,      // Variable specific yield
            AquiferType::Volcanic => 0.12,   // Moderate specific yield
        }
    }
    
    /// Human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            AquiferType::Porous => "Porous Aquifer",
            AquiferType::Fractured => "Fractured Rock Aquifer",
            AquiferType::Karst => "Karst Aquifer",
            AquiferType::Volcanic => "Volcanic Aquifer",
        }
    }
}

/// Stored data for a single aquifer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquiferData {
    /// Unique aquifer identifier
    pub id: u64,
    /// Parent polygon (surface recharge zone)
    pub polygon_id: u64,
    /// Aquifer classification
    pub aquifer_type: AquiferType,
    /// Maximum storage capacity (cubic meters)
    pub max_capacity_m3: f64,
    /// Current water volume (cubic meters)
    pub current_volume_m3: f64,
    /// Saturated thickness (meters)
    pub saturated_thickness_m: f32,
    /// Elevation of water table (meters above sea level)
    pub water_table_elevation_m: f32,
    /// Base elevation of aquifer (meters below sea level)
    pub base_elevation_m: f32,
    /// Average annual recharge rate (meters/year)
    pub annual_recharge_rate_m: f32,
    /// Baseflow contribution to surface water (0.0-1.0)
    pub baseflow_coefficient: f32,
}

/// Recharge event from precipitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquiferRecharge {
    /// Source polygon
    pub polygon_id: u64,
    /// Recharge rate in mm/year equivalent
    pub recharge_mm_year: f32,
    /// When recharge occurred (year)
    pub year: i32,
}

/// Groundwater system - manages aquifers and water table dynamics
pub struct GroundwaterSystem {
    /// All aquifers mapped by polygon ID
    aquifers: HashMap<u64, AquiferData>,
    /// Recharge history for the current year
    current_recharge: Vec<AquiferRecharge>,
    /// Random number generator for aquifer properties
    rng: StdRng,
    /// Configuration
    config: GroundwaterConfig,
}

/// Configuration for groundwater simulation
#[derive(Debug, Clone, Copy)]
pub struct GroundwaterConfig {
    /// Base aquifer depth below surface (meters)
    pub base_depth_m: f32,
    /// Minimum aquifer capacity (cubic meters)
    pub min_capacity_m3: f64,
    /// Maximum aquifer capacity (cubic meters)
    pub max_capacity_m3: f64,
    /// Global recharge efficiency (fraction of precipitation that becomes recharge)
    pub recharge_efficiency: f32,
    /// Baseflow recession constant (fraction of storage released per year)
    pub baseflow_recession: f32,
}

impl Default for GroundwaterConfig {
    fn default() -> Self {
        Self {
            base_depth_m: 50.0,           // Aquifers typically 50-200m below surface
            min_capacity_m3: 1_000_000.0,  // 1 million cubic meters minimum
            max_capacity_m3: 500_000_000.0, // 500 million cubic meters maximum
            recharge_efficiency: 0.15,     // 15% of precipitation becomes groundwater
            baseflow_recession: 0.30,     // 30% of storage contributes to baseflow annually
        }
    }
}

impl GroundwaterSystem {
    /// Create a new groundwater system
    pub fn new(seed: Option<u64>) -> Self {
        let rng = seed.unwrap_or_else(rand::random);
        Self {
            aquifers: HashMap::new(),
            current_recharge: Vec::new(),
            rng: StdRng::from_seed([
                rng as u8, (rng >> 8) as u8, (rng >> 16) as u8, (rng >> 24) as u8,
                (rng >> 32) as u8, (rng >> 40) as u8, (rng >> 48) as u8, (rng >> 56) as u8,
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                16, 17, 18, 19, 20, 21, 22, 23,
            ]),
            config: GroundwaterConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: GroundwaterConfig, seed: Option<u64>) -> Self {
        let mut system = Self::new(seed);
        system.config = config;
        system
    }
    
    /// Initialize aquifers for a set of polygons based on elevation and geology
    /// 
    /// Aquifer formation is determined by:
    /// - Elevation bands (lowlands more likely to have shallow aquifers)
    /// - Coastal proximity (influences aquifer type)
    /// - Geological factors (inferred from elevation patterns)
    pub fn initialize_aquifers(
        &mut self,
        polygons: &HashMap<u64, f32>,  // polygon_id -> elevation
        precipitation_mm_year: f32,    // Average annual precipitation
    ) {
        for (polygon_id, &elevation) in polygons {
            // Skip ocean polygons (negative elevation)
            if elevation < 0.0 {
                continue;
            }
            
            let aquifer = self.create_aquifer(*polygon_id, elevation, precipitation_mm_year);
            self.aquifers.insert(*polygon_id, aquifer);
        }
    }
    
    /// Create aquifer data for a single polygon
    fn create_aquifer(
        &mut self,
        polygon_id: u64,
        elevation: f32,
        precipitation_mm_year: f32,
    ) -> AquiferData {
        // Determine aquifer type based on elevation
        let aquifer_type = self.determine_aquifer_type(elevation);
        
        // Calculate capacity based on aquifer type and elevation
        let base_capacity = self.calculate_base_capacity(elevation, aquifer_type);
        let capacity_m3 = base_capacity
            .max(self.config.min_capacity_m3)
            .min(self.config.max_capacity_m3);
        
        // Initial volume at 60% capacity
        let current_volume_m3 = capacity_m3 * 0.6;
        
        // Water table elevation based on surface elevation minus base depth
        let water_table_elevation_m = (elevation - self.config.base_depth_m).max(0.0);
        
        // Base elevation (aquifer bottom)
        let base_elevation_m = elevation - self.config.base_depth_m - 100.0; // 100m thick aquifer
        
        // Saturated thickness
        let saturated_thickness_m = 100.0_f32.min(water_table_elevation_m.max(1.0));
        
        // Annual recharge rate
        let annual_recharge_rate_m = precipitation_mm_year * self.config.recharge_efficiency / 1000.0;
        
        // Baseflow coefficient varies by aquifer type
        let baseflow_coefficient = match aquifer_type {
            AquiferType::Porous => 0.35,
            AquiferType::Fractured => 0.15,
            AquiferType::Karst => 0.50,
            AquiferType::Volcanic => 0.40,
        };
        
        AquiferData {
            id: polygon_id,
            polygon_id,
            aquifer_type,
            max_capacity_m3: capacity_m3,
            current_volume_m3,
            saturated_thickness_m,
            water_table_elevation_m,
            base_elevation_m,
            annual_recharge_rate_m,
            baseflow_coefficient,
        }
    }
    
    /// Determine aquifer type based on elevation and geological factors
    fn determine_aquifer_type(&mut self, elevation: f32) -> AquiferType {
        let rand_val = self.rng.gen::<f32>();
        
        match elevation {
            // Lowlands (< 400m): Mostly porous, some volcanic
            e if e < 400.0 => {
                if rand_val < 0.70 {
                    AquiferType::Porous
                } else if rand_val < 0.90 {
                    AquiferType::Volcanic
                } else {
                    AquiferType::Fractured
                }
            },
            // Midlands (400-800m): Mixed porous and fractured
            e if e < 800.0 => {
                if rand_val < 0.40 {
                    AquiferType::Porous
                } else if rand_val < 0.70 {
                    AquiferType::Fractured
                } else if rand_val < 0.90 {
                    AquiferType::Volcanic
                } else {
                    AquiferType::Karst
                }
            },
            // Highlands (800-1500m): More fractured, some karst in limestone areas
            e if e < 1500.0 => {
                if rand_val < 0.30 {
                    AquiferType::Porous
                } else if rand_val < 0.60 {
                    AquiferType::Fractured
                } else if rand_val < 0.80 {
                    AquiferType::Karst
                } else {
                    AquiferType::Volcanic
                }
            },
            // Mountains (> 1500m): Mostly fractured, volcanic where applicable
            _ => {
                if rand_val < 0.25 {
                    AquiferType::Fractured
                } else if rand_val < 0.50 {
                    AquiferType::Volcanic
                } else if rand_val < 0.75 {
                    AquiferType::Karst
                } else {
                    AquiferType::Porous
                }
            }
        }
    }
    
    /// Calculate base storage capacity
    fn calculate_base_capacity(&self, elevation: f32, aquifer_type: AquiferType) -> f64 {
        // Base capacity scales with elevation (larger catchments at higher elevations)
        let elevation_factor = 1.0 + (elevation / 1000.0).min(2.0);
        
        // Capacity varies by aquifer type (porous can hold more)
        let type_factor = match aquifer_type {
            AquiferType::Porous => 1.0,
            AquiferType::Volcanic => 0.8,
            AquiferType::Karst => 0.6,
            AquiferType::Fractured => 0.4,
        };
        
        // Base capacity: 50 million cubic meters, scaled by factors
        (50_000_000.0 * elevation_factor * type_factor) as f64
    }
    
    /// Process annual recharge from precipitation
    pub fn apply_recharge(&mut self, year: i32, precipitation_mm_year: f32) {
        self.current_recharge.clear();
        
        for aquifer in self.aquifers.values_mut() {
            // Calculate recharge based on precipitation and efficiency
            let recharge_mm = precipitation_mm_year * self.config.recharge_efficiency;
            let recharge_m3 = (recharge_mm as f64) * 1000.0; // Convert mm to m³ per 1km² area
            
            aquifer.current_volume_m3 = (aquifer.current_volume_m3 + recharge_m3)
                .min(aquifer.max_capacity_m3);
            
            self.current_recharge.push(AquiferRecharge {
                polygon_id: aquifer.polygon_id,
                recharge_mm_year: recharge_mm,
                year,
            });
        }
    }
    
    /// Simulate baseflow discharge to surface water bodies
    /// Returns baseflow volume in cubic meters for each polygon
    pub fn calculate_baseflow(&self) -> HashMap<u64, f64> {
        let mut baseflow = HashMap::new();
        
        for aquifer in self.aquifers.values() {
            // Baseflow is a fraction of current storage
            let discharge = aquifer.current_volume_m3 * (aquifer.baseflow_coefficient as f64);
            
            baseflow.insert(aquifer.polygon_id, discharge);
        }
        
        baseflow
    }
    
    /// Simulate groundwater recession (natural depletion)
    pub fn simulate_recession(&mut self) {
        for aquifer in self.aquifers.values_mut() {
            // Remove baseflow contribution from storage
            let baseflow_loss = aquifer.current_volume_m3 * (self.config.baseflow_recession as f64);
            aquifer.current_volume_m3 -= baseflow_loss;
            
            // Ensure minimum volume (aquifer never fully depletes)
            aquifer.current_volume_m3 = aquifer.current_volume_m3.max(1_000_000.0);
            
            // Update saturated thickness based on remaining volume
            let fill_percentage = (aquifer.current_volume_m3 / aquifer.max_capacity_m3) as f32;
            aquifer.saturated_thickness_m = (100.0 * fill_percentage).max(1.0);
        }
    }
    
    /// Get current aquifer data for a polygon
    pub fn get_aquifer(&self, polygon_id: u64) -> Option<&AquiferData> {
        self.aquifers.get(&polygon_id)
    }
    
    /// Get all aquifers
    pub fn get_all_aquifers(&self) -> &HashMap<u64, AquiferData> {
        &self.aquifers
    }
    
    /// Get total water storage across all aquifers
    pub fn total_storage(&self) -> f64 {
        self.aquifers.values().map(|a| a.current_volume_m3).sum()
    }
    
    /// Calculate average fill percentage
    pub fn average_fill_percentage(&self) -> f32 {
        if self.aquifers.is_empty() {
            return 0.0;
        }
        
        let total_current: f64 = self.aquifers.values().map(|a| a.current_volume_m3).sum();
        let total_max: f64 = self.aquifers.values().map(|a| a.max_capacity_m3).sum();
        
        if total_max > 0.0 {
            (total_current / total_max) as f32
        } else {
            0.0
        }
    }
    
    /// Get water table elevation for a polygon
    pub fn get_water_table_elevation(&self, polygon_id: u64) -> Option<f32> {
        self.aquifers.get(&polygon_id).map(|a| a.water_table_elevation_m)
    }
    
    /// Check if a polygon has a shallow water table (potential for flooding/wetlands)
    pub fn is_shallow_water_table(&self, polygon_id: u64, depth_threshold_m: f32) -> bool {
        if let Some(aquifer) = self.aquifers.get(&polygon_id) {
            let water_table_depth = aquifer.polygon_id as f32; // Placeholder - would need actual surface elevation
            water_table_depth < depth_threshold_m
        } else {
            false
        }
    }
    
    /// Simulate one year of groundwater dynamics
    pub fn simulate_year(&mut self, year: i32, precipitation_mm_year: f32) {
        self.apply_recharge(year, precipitation_mm_year);
        self.simulate_recession();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aquifer_type_properties() {
        assert_eq!(AquiferType::Porous.base_porosity(), 0.25);
        assert_eq!(AquiferType::Fractured.base_porosity(), 0.02);
        assert!(AquiferType::Karst.hydraulic_conductivity() > AquiferType::Fractured.hydraulic_conductivity());
    }

    #[test]
    fn test_groundwater_initialization() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let mut polygons = HashMap::new();
        polygons.insert(1, 200.0);   // Lowland
        polygons.insert(2, 600.0);   // Midland
        polygons.insert(3, 1200.0);  // Highland
        polygons.insert(4, -50.0);  // Ocean - should be skipped
        
        system.initialize_aquifers(&polygons, 1000.0); // 1000mm/year precipitation
        
        // Ocean should not have aquifer
        assert!(system.get_aquifer(4).is_none());
        
        // Land polygons should have aquifers
        assert!(system.get_aquifer(1).is_some());
        assert!(system.get_aquifer(2).is_some());
        assert!(system.get_aquifer(3).is_some());
    }

    #[test]
    fn test_recharge_increases_volume() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let polygons: HashMap<u64, f32> = vec![(1, 200.0)].into_iter().collect();
        system.initialize_aquifers(&polygons, 1000.0);
        
        let initial_volume = system.get_aquifer(1).unwrap().current_volume_m3;
        
        system.apply_recharge(1, 1000.0);
        
        let after_recharge = system.get_aquifer(1).unwrap().current_volume_m3;
        assert!(after_recharge > initial_volume);
    }

    #[test]
    fn test_total_storage() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let polygons: HashMap<u64, f32> = vec![
            (1, 200.0),
            (2, 400.0),
            (3, 600.0),
        ].into_iter().collect();
        
        system.initialize_aquifers(&polygons, 1000.0);
        
        let total = system.total_storage();
        assert!(total > 0.0);
    }

    #[test]
    fn test_average_fill_percentage() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let polygons: HashMap<u64, f32> = vec![(1, 200.0)].into_iter().collect();
        system.initialize_aquifers(&polygons, 1000.0);
        
        let fill = system.average_fill_percentage();
        assert!(fill > 0.5 && fill < 0.7); // Initial fill at 60%
    }

    #[test]
    fn test_year_simulation() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let polygons: HashMap<u64, f32> = vec![(1, 200.0)].into_iter().collect();
        system.initialize_aquifers(&polygons, 1000.0);
        
        let initial_fill = system.average_fill_percentage();
        system.simulate_year(1, 1000.0);
        let after_fill = system.average_fill_percentage();
        
        // After recharge + recession, should be lower due to baseflow discharge
        assert!(after_fill >= 0.0 && after_fill <= 1.0);
    }

    #[test]
    fn test_baseflow_calculation() {
        let mut system = GroundwaterSystem::new(Some(42));
        
        let polygons: HashMap<u64, f32> = vec![(1, 200.0)].into_iter().collect();
        system.initialize_aquifers(&polygons, 1000.0);
        
        let baseflow = system.calculate_baseflow();
        assert!(baseflow.contains_key(&1));
        assert!(baseflow[&1] >= 0.0);
    }
}
