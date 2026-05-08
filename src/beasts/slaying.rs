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
//! - Remnant drops (item for future summoning/buffs)
//! - Curse transfers to the slaying factions
//! - Beast enters dormant state for N years before possible resurrection

use super::{BeastElement, PrimalBeast, PrimalBeastInstance, BeastState, profiles::get_beast_profile};
use crate::artifacts::{Artifact, ArtifactPropertyType};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

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
        remnant: String,
        curse: String,
        participating_factions: Vec<Uuid>,
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
pub fn attempt_slaying(
    beast: &PrimalBeastInstance,
    participants: &[SlayingParticipant],
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
        BeastSlayingResult::Slain {
            remnant: profile.remnant.clone(),
            curse: profile.curse.clone(),
            participating_factions: participants.iter().map(|p| p.faction_id).collect(),
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
