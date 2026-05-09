//! Beast Slaying System
//!
//! Primal beasts can be slain through faction cooperation.
//! Requirements:
//! - 3+ factions cooperating
//! - Legendary artifacts with element-aligned properties
//! - Targeting the beast's elemental weakness
//!
//! ## Slaying Conditions
//!
//! 1. Minimum 3 cooperating factions (allied or joint operation)
//! 2. Each faction must possess a legendary artifact
//! 3. At least one artifact must align with the beast's weakness
//! 4. Combined power must exceed beast's power_level * 10
//!
//! ## Death Consequences
//!
//! - Remnant drops (actual Artifact with environmental effects)
//! - Curse transfers to the slaying factions (via Remnant possession)
//! - Beast enters dormant state for N years before possible resurrection

use super::{BeastElement, PrimalBeast, PrimalBeastInstance, profiles::get_beast_profile};
use crate::artifacts::Artifact;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// Placeholder for RemnantArtifact - to be implemented when remnants module is added
/// Remnant artifact created when a beast is slain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemnantArtifact {
    /// World ID where the beast was slain
    pub world_id: Uuid,
    /// Beast type
    pub beast: PrimalBeast,
    /// Year the beast was slain
    pub slaying_year: i32,
    /// Position where the beast was slain
    pub position: u32,
    /// Element of the beast
    pub element: BeastElement,
    /// Curse effect description
    pub curse_effect: String,
    /// Blessing effect description
    pub blessing_effect: String,
    /// Effect radius in km
    pub effect_radius_km: f32,
    /// Whether the curse is active
    pub curse_active: bool,
}

impl RemnantArtifact {
    /// Create a remnant from a successful beast slaying.
    pub fn from_beast_slaying(
        world_id: Uuid,
        beast: PrimalBeast,
        slaying_year: i32,
        position: u32,
    ) -> Self {
        let profile = get_beast_profile(beast);
        Self {
            world_id,
            beast,
            slaying_year,
            position,
            element: profile.element,
            curse_effect: profile.curse.clone(),
            blessing_effect: profile.blessing.clone(),
            effect_radius_km: 10.0,
            curse_active: true,
        }
    }
}

/// Requirements for slaying a primal beast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastSlayingRequirements {
    /// Minimum number of cooperating factions
    pub min_factions: u8,
    /// Minimum number of legendary artifacts required
    pub min_artifacts: u8,
    /// Whether weakness targeting is required
    pub requires_weakness_targeting: bool,
    /// Power threshold multiplier
    pub power_threshold_multiplier: f32,
}

impl Default for BeastSlayingRequirements {
    fn default() -> Self {
        Self {
            min_factions: 3,
            min_artifacts: 3,
            requires_weakness_targeting: true,
            power_threshold_multiplier: 10.0,
        }
    }
}

/// A faction participating in a beast slaying attempt.
#[derive(Debug, Clone)]
pub struct SlayingParticipant {
    /// Faction ID
    pub faction_id: Uuid,
    /// Faction's artifact (if any)
    pub artifact: Option<Artifact>,
    /// Contribution power level
    pub contribution: f32,
}

/// Result of a beast slaying attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeastSlayingResult {
    /// The beast was successfully slain
    Slain {
        /// The Remnant artifact dropped by the beast
        remnant: RemnantArtifact,
        /// Description of the curse
        curse: String,
        /// Factions that participated in the slaying
        participating_factions: Vec<Uuid>,
        /// The year the beast was slain
        slaying_year: i32,
    },
    /// The attempt failed
    Failed {
        reason: String,
        damage_dealt: f32,
        participating_factions: Vec<Uuid>,
    },
    /// The beast was weakened but not slain
    Weakened {
        damage_dealt: f32,
        participating_factions: Vec<Uuid>,
    },
}

/// Check if a beast slaying attempt meets requirements.
pub fn check_slaying_requirements(
    beast: &PrimalBeastInstance,
    participants: &[SlayingParticipant],
) -> Result<BeastSlayingRequirements, SlayingAttemptError> {
    let requirements = BeastSlayingRequirements::default();
    let profile = get_beast_profile(beast.beast);
    
    // Check faction count
    if participants.len() < requirements.min_factions as usize {
        return Err(SlayingAttemptError::InsufficientFactions {
            required: requirements.min_factions,
            actual: participants.len() as u8,
        });
    }
    
    // Check artifact count
    let artifact_count = participants.iter().filter(|p| p.artifact.is_some()).count();
    if artifact_count < requirements.min_artifacts as usize {
        return Err(SlayingAttemptError::InsufficientArtifacts {
            required: requirements.min_artifacts,
            actual: artifact_count as u8,
        });
    }
    
    // Check weakness targeting
    if requirements.requires_weakness_targeting {
        let has_weakness_artifact = participants.iter().any(|p| {
            if let Some(ref artifact) = p.artifact {
                has_element_alignment(artifact, profile.weakness)
            } else {
                false
            }
        });
        
        if !has_weakness_artifact {
            return Err(SlayingAttemptError::MissingWeaknessAlignment {
                weakness: profile.weakness,
            });
        }
    }
    
    // Check combined power
    let total_power: f32 = participants.iter().map(|p| p.contribution).sum();
    let required_power = beast.power_level * requirements.power_threshold_multiplier;
    
    if total_power < required_power {
        return Err(SlayingAttemptError::InsufficientPower {
            required: required_power,
            actual: total_power,
        });
    }
    
    Ok(requirements)
}

/// Check if an artifact has element alignment.
fn has_element_alignment(artifact: &Artifact, element: BeastElement) -> bool {
    // Check artifact properties for elemental alignment
    if let Some(ref properties) = artifact.properties {
        for prop in properties {
            let prop_type_str = format!("{:?}", prop.property_type).to_lowercase();
            let element_str = format!("{:?}", element).to_lowercase();
            if prop_type_str.contains(&element_str) {
                return true;
            }
        }
    }
    false
}

/// Execute a beast slaying attempt.
/// Returns the result of the attempt, including a RemnantArtifact if successful.
pub fn attempt_slaying(
    beast: &PrimalBeastInstance,
    participants: &[SlayingParticipant],
    world_id: Uuid,
    slaying_year: i32,
) -> BeastSlayingResult {
    let profile = get_beast_profile(beast.beast);
    let requirements = match check_slaying_requirements(beast, participants) {
        Ok(r) => r,
        Err(e) => {
            // Calculate partial damage on failed attempt
            let total_power: f32 = participants.iter().map(|p| p.contribution).sum();
            let damage = total_power * 0.3; // 30% effectiveness on failed attempt
            
            return BeastSlayingResult::Failed {
                reason: format!("{:?}", e),
                damage_dealt: damage,
                participating_factions: participants.iter().map(|p| p.faction_id).collect(),
            };
        }
    };
    
    // Calculate slaying power vs beast defense
    let total_power: f32 = participants.iter().map(|p| p.contribution).sum();
    let beast_defense = beast.power_level * requirements.power_threshold_multiplier;
    
    // Success check
    if total_power >= beast_defense {
        // Create the Remnant artifact using the local struct
        let remnant = RemnantArtifact::from_beast_slaying(
            world_id,
            beast.beast,
            slaying_year,
            beast.position,
        );
        
        BeastSlayingResult::Slain {
            remnant,
            curse: profile.curse.clone(),
            participating_factions: participants.iter().map(|p| p.faction_id).collect(),
            slaying_year,
        }
    } else {
        // Partial success - weaken beast
        let damage_ratio = total_power / beast_defense;
        let damage = damage_ratio * beast.power_level;
        
        BeastSlayingResult::Weakened {
            damage_dealt: damage,
            participating_factions: participants.iter().map(|p| p.faction_id).collect(),
        }
    }
}

/// Errors that can occur during slaying attempt.
#[derive(Debug, Clone)]
pub enum SlayingAttemptError {
    InsufficientFactions { required: u8, actual: u8 },
    InsufficientArtifacts { required: u8, actual: u8 },
    MissingWeaknessAlignment { weakness: BeastElement },
    InsufficientPower { required: f32, actual: f32 },
}

impl std::fmt::Display for SlayingAttemptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlayingAttemptError::InsufficientFactions { required, actual } => {
                write!(f, "Need {} factions, only have {}", required, actual)
            }
            SlayingAttemptError::InsufficientArtifacts { required, actual } => {
                write!(f, "Need {} artifacts, only have {}", required, actual)
            }
            SlayingAttemptError::MissingWeaknessAlignment { weakness } => {
                write!(f, "Need artifact aligned with {:?} weakness", weakness)
            }
            SlayingAttemptError::InsufficientPower { required, actual } => {
                write!(f, "Need power {:.1}, only have {:.1}", required, actual)
            }
        }
    }
}

/// Calculate the dormancy period after beast death (in years).
pub fn calculate_dormancy_period(beast: PrimalBeast) -> i32 {
    match beast {
        PrimalBeast::Pyraxes => 500,
        PrimalBeast::Tidarth => 400,
        PrimalBeast::Terros => 600,
        PrimalBeast::Lumina => 300,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beasts::PrimalBeastInstance;
    use crate::artifacts::{Artifact, ArtifactCategory, ArtifactProperty, ArtifactPropertyType};
    use uuid::Uuid;

    fn create_test_artifact(element_name: &str) -> Artifact {
        let mut artifact = Artifact::new(
            Uuid::new_v4(),
            format!("{} Element Artifact", element_name),
            ArtifactCategory::Weapon,
            1000,
            "Test artifact for slaying".to_string(),
            0.9,
        );
        artifact.properties = Some(vec![
            ArtifactProperty {
                name: format!("{} Alignment", element_name),
                description: format!("Aligned with {} element", element_name),
                property_type: ArtifactPropertyType::Magical,
            }
        ]);
        artifact
    }

    fn create_sufficient_participants(beast: PrimalBeast) -> Vec<SlayingParticipant> {
        let profile = get_beast_profile(beast);
        let weakness = profile.weakness;
        
        // Create participants with artifacts - one aligned with weakness
        vec![
            SlayingParticipant {
                faction_id: Uuid::new_v4(),
                artifact: Some(create_test_artifact(format!("{:?}", weakness).as_str())),
                contribution: 15.0,
            },
            SlayingParticipant {
                faction_id: Uuid::new_v4(),
                artifact: Some(create_test_artifact("Generic")),
                contribution: 15.0,
            },
            SlayingParticipant {
                faction_id: Uuid::new_v4(),
                artifact: Some(create_test_artifact("Generic")),
                contribution: 15.0,
            },
        ]
    }

    fn create_test_beast(beast_type: PrimalBeast, power: f32) -> PrimalBeastInstance {
        let mut beast = PrimalBeastInstance::new(beast_type, 42, 1000);
        beast.power_level = power;
        beast
    }

    #[test]
    fn test_slaying_creates_remnant() {
        let world_id = Uuid::new_v4();
        let beast = create_test_beast(PrimalBeast::Pyraxes, 5.0);
        let participants = create_sufficient_participants(PrimalBeast::Pyraxes);
        
        let result = attempt_slaying(&beast, &participants, world_id, 1200);
        
        match result {
            BeastSlayingResult::Slain {
                remnant,
                curse,
                participating_factions,
                slaying_year,
            } => {
                // Verify Remnant was created
                assert_eq!(remnant.beast, PrimalBeast::Pyraxes);
                assert_eq!(remnant.element, BeastElement::Fire);
                assert!(remnant.curse_active);
                assert_eq!(slaying_year, 1200);
                
                // Verify Remnant has correct effect radius
                assert_eq!(remnant.effect_radius_km, 10.0);
                
                // Verify curse is present
                assert!(!curse.is_empty());
                
                // Verify all participants are recorded
                assert_eq!(participating_factions.len(), 3);
            }
            _ => panic!("Expected Slain result"),
        }
    }

    #[test]
    fn test_all_beasts_create_remnants() {
        let world_id = Uuid::new_v4();
        
        for beast_type in [PrimalBeast::Pyraxes, PrimalBeast::Tidarth, PrimalBeast::Terros, PrimalBeast::Lumina] {
            let beast = create_test_beast(beast_type, 5.0);
            let participants = create_sufficient_participants(beast_type);
            
            let result = attempt_slaying(&beast, &participants, world_id, 1000);
            
            match result {
                BeastSlayingResult::Slain { remnant, .. } => {
                    assert_eq!(remnant.beast, beast_type);
                    assert!(remnant.effect_radius_km > 0.0);
                    assert!(!remnant.curse_effect.is_empty());
                    assert!(!remnant.blessing_effect.is_empty());
                }
                _ => panic!("Expected Slain result for {:?}", beast_type),
            }
        }
    }

    #[test]
    fn test_insufficient_factions_fails() {
        let world_id = Uuid::new_v4();
        let beast = create_test_beast(PrimalBeast::Pyraxes, 5.0);
        
        // Only 2 participants (need 3)
        let participants = vec![
            SlayingParticipant {
                faction_id: Uuid::new_v4(),
                artifact: Some(create_test_artifact("Water")),
                contribution: 100.0,
            },
            SlayingParticipant {
                faction_id: Uuid::new_v4(),
                artifact: Some(create_test_artifact("Fire")),
                contribution: 100.0,
            },
        ];
        
        let result = attempt_slaying(&beast, &participants, world_id, 1000);
        
        match result {
            BeastSlayingResult::Failed { reason, .. } => {
                assert!(reason.contains("Need 3 factions"));
            }
            _ => panic!("Expected Failed result"),
        }
    }

    #[test]
    fn test_insufficient_power_fails() {
        let world_id = Uuid::new_v4();
        let beast = create_test_beast(PrimalBeast::Pyraxes, 100.0); // Very powerful beast
        let participants = create_sufficient_participants(PrimalBeast::Pyraxes);
        
        let result = attempt_slaying(&beast, &participants, world_id, 1000);
        
        match result {
            BeastSlayingResult::Failed { reason, .. } => {
                assert!(reason.contains("Need power"));
            }
            _ => panic!("Expected Failed result for overpowered beast"),
        }
    }

    #[test]
    fn test_dormancy_period() {
        assert_eq!(calculate_dormancy_period(PrimalBeast::Pyraxes), 500);
        assert_eq!(calculate_dormancy_period(PrimalBeast::Tidarth), 400);
        assert_eq!(calculate_dormancy_period(PrimalBeast::Terros), 600);
        assert_eq!(calculate_dormancy_period(PrimalBeast::Lumina), 300);
    }
}
