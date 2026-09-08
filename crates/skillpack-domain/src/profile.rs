//! Assessment Profile
//!
//! Skills come in two flavors with very different evidence surfaces:
//!
//! - **Cnsb** — CKODEX-native bundles carrying a `*.cnsb.json` manifest and the
//!   full governance/provenance artifact set (SBOM, threat model, evals CI).
//!   Held to the strict supply-chain bar.
//! - **AgentSkills** — real-world agent skills per the agentskills convention:
//!   `SKILL.md` with `name`/`description` frontmatter, a markdown body, and
//!   optional `references/`, `scripts/`, `assets/` for progressive disclosure.
//!   Scoring the CNSB artifact set against these zeroes 5 of 9 dimensions and
//!   collapses the grade distribution (observed: 400 of 404 fleet skills at F),
//!   so they get profile-specific dimension weights and checker rubrics.
//!
//! Detection is structural, not declared: the presence of a `*.cnsb.json`
//! manifest opts a skill into the strict profile.

use crate::{Dimension, DimensionId, SkillReader, Weight, standard_dimensions};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Which rubric a skill is assessed under.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum AssessmentProfile {
    /// CKODEX-native bundle (`*.cnsb.json` present) — strict supply-chain bar.
    #[default]
    Cnsb,
    /// SKILL.md-convention agent skill — content-quality bar.
    AgentSkills,
}

impl AssessmentProfile {
    /// Detect the profile for a skill directory.
    pub fn detect(reader: &dyn SkillReader, path: &Path) -> Self {
        if reader.list_files(path, r"\.cnsb\.json$").is_empty() {
            Self::AgentSkills
        } else {
            Self::Cnsb
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Cnsb => "cnsb",
            Self::AgentSkills => "agentskills",
        }
    }
}

/// Dimension weights for the given profile (always sum to 100).
pub fn dimensions_for(profile: AssessmentProfile) -> Vec<Dimension> {
    match profile {
        AssessmentProfile::Cnsb => standard_dimensions(),
        AssessmentProfile::AgentSkills => agentskills_dimensions(),
    }
}

/// Weights tuned for SKILL.md-convention skills: content quality dominates;
/// supply-chain dimensions stay present (they still differentiate the best
/// skills) but no longer drown the signal.
pub fn agentskills_dimensions() -> Vec<Dimension> {
    fn w(v: u8) -> Weight {
        Weight::new(v).expect("agentskills weight <= 100")
    }
    vec![
        Dimension {
            id: DimensionId::IdentityAndManifest,
            weight: w(15),
        },
        Dimension {
            id: DimensionId::Security,
            weight: w(20),
        },
        Dimension {
            id: DimensionId::Provenance,
            weight: w(4),
        },
        Dimension {
            id: DimensionId::Documentation,
            weight: w(25),
        },
        Dimension {
            id: DimensionId::Testing,
            weight: w(6),
        },
        Dimension {
            id: DimensionId::Compatibility,
            weight: w(12),
        },
        Dimension {
            id: DimensionId::Lifecycle,
            weight: w(6),
        },
        Dimension {
            id: DimensionId::Governance,
            weight: w(6),
        },
        Dimension {
            id: DimensionId::EvalsHitl,
            weight: w(6),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agentskills_weights_sum_to_100() {
        let total: u32 = agentskills_dimensions()
            .iter()
            .map(|d| d.weight.value() as u32)
            .sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn agentskills_covers_all_dimensions() {
        assert_eq!(agentskills_dimensions().len(), DimensionId::all().len());
    }
}
