//! Dimension Value Objects
//!
//! Quality dimensions as immutable value objects.

use serde::{Deserialize, Serialize};

/// Quality dimension with weight and assessment criteria
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Dimension {
    pub id: DimensionId,
    pub weight: Weight,
}

/// Dimension identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DimensionId {
    IdentityAndManifest,
    Security,
    Provenance,
    Documentation,
    Testing,
    Compatibility,
    Lifecycle,
    Governance,
    EvalsHitl,
}

impl DimensionId {
    pub fn all() -> Vec<Self> {
        vec![
            Self::IdentityAndManifest,
            Self::Security,
            Self::Provenance,
            Self::Documentation,
            Self::Testing,
            Self::Compatibility,
            Self::Lifecycle,
            Self::Governance,
            Self::EvalsHitl,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::IdentityAndManifest => "IdentityAndManifest",
            Self::Security => "Security",
            Self::Provenance => "Provenance",
            Self::Documentation => "Documentation",
            Self::Testing => "Testing",
            Self::Compatibility => "Compatibility",
            Self::Lifecycle => "Lifecycle",
            Self::Governance => "Governance",
            Self::EvalsHitl => "EvalsHitl",
        }
    }
}

/// Weight as percentage (0-100)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Weight(u8);

impl Weight {
    pub fn new(value: u8) -> Result<Self, &'static str> {
        if value > 100 {
            return Err("Weight cannot exceed 100");
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn as_fraction(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self {
            id: DimensionId::IdentityAndManifest,
            weight: Weight(11),
        }
    }
}

/// Standard dimension weights per CKODEX SkillPack framework
pub fn standard_dimensions() -> Vec<Dimension> {
    vec![
        Dimension {
            id: DimensionId::IdentityAndManifest,
            weight: Weight(11),
        },
        Dimension {
            id: DimensionId::Security,
            weight: Weight(18),
        },
        Dimension {
            id: DimensionId::Provenance,
            weight: Weight(14),
        },
        Dimension {
            id: DimensionId::Documentation,
            weight: Weight(11),
        },
        Dimension {
            id: DimensionId::Testing,
            weight: Weight(10),
        },
        Dimension {
            id: DimensionId::Compatibility,
            weight: Weight(8),
        },
        Dimension {
            id: DimensionId::Lifecycle,
            weight: Weight(10),
        },
        Dimension {
            id: DimensionId::Governance,
            weight: Weight(9),
        },
        Dimension {
            id: DimensionId::EvalsHitl,
            weight: Weight(9),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_weights_sum_to_100() {
        let total: u32 = standard_dimensions()
            .iter()
            .map(|d| d.weight.value() as u32)
            .sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn canonical_dimension_count_is_nine() {
        assert_eq!(standard_dimensions().len(), 9);
    }
}
