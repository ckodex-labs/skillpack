//! CanIUse - Skill Capability Compatibility Matrix
//!
//! Like caniuse.com but for AI agent skill capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capability support level (like caniuse.com)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    /// Full support - feature works as expected
    Full,
    /// Partial support - works with limitations
    Partial,
    /// No support - feature not available
    None,
    /// Unknown - not tested
    Unknown,
    /// Deprecated - will be removed
    Deprecated,
}

impl SupportLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial => "partial",
            Self::None => "none",
            Self::Unknown => "unknown",
            Self::Deprecated => "deprecated",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Full => "✅",
            Self::Partial => "🟡",
            Self::None => "❌",
            Self::Unknown => "❓",
            Self::Deprecated => "⚠️",
        }
    }
}

/// A capability that skills can support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Category (e.g., "security", "templates", "mcp")
    pub category: CapabilityCategory,
    /// Spec version where introduced
    pub since_version: Option<String>,
}

/// Capability categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityCategory {
    Structure,
    Security,
    Templates,
    MCP,
    Governance,
    Provenance,
    Documentation,
    Integration,
}

/// Skill's support for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySupport {
    pub capability_id: String,
    pub level: SupportLevel,
    pub notes: Option<String>,
    pub verified_at: DateTime<Utc>,
}

/// Full compatibility matrix for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCompatibility {
    /// Skill identifier
    pub skill_ref: String,
    /// All capability support levels
    pub capabilities: HashMap<String, CapabilitySupport>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl SkillCompatibility {
    pub fn new(skill_ref: impl Into<String>) -> Self {
        Self {
            skill_ref: skill_ref.into(),
            capabilities: HashMap::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn set_support(
        &mut self,
        capability_id: impl Into<String>,
        level: SupportLevel,
        notes: Option<String>,
    ) {
        let id = capability_id.into();
        self.capabilities.insert(
            id.clone(),
            CapabilitySupport {
                capability_id: id,
                level,
                notes,
                verified_at: Utc::now(),
            },
        );
        self.updated_at = Utc::now();
    }

    /// Calculate overall compatibility score (0-100)
    pub fn compatibility_score(&self) -> u32 {
        if self.capabilities.is_empty() {
            return 0;
        }

        let total: u32 = self
            .capabilities
            .values()
            .map(|s| match s.level {
                SupportLevel::Full => 100,
                SupportLevel::Partial => 50,
                SupportLevel::None => 0,
                SupportLevel::Unknown => 25,
                SupportLevel::Deprecated => 10,
            })
            .sum();

        total / self.capabilities.len() as u32
    }

    /// Check if skill supports a specific capability
    pub fn supports(&self, capability_id: &str) -> SupportLevel {
        self.capabilities
            .get(capability_id)
            .map(|s| s.level)
            .unwrap_or(SupportLevel::Unknown)
    }
}

/// Standard capabilities registry
pub mod standard_capabilities {
    use super::*;

    pub fn all() -> Vec<Capability> {
        vec![
            // Structure
            Capability {
                id: "skill-md".to_string(),
                name: "SKILL.md".to_string(),
                description: "Valid SKILL.md manifest file".to_string(),
                category: CapabilityCategory::Structure,
                since_version: Some("1.0".to_string()),
            },
            Capability {
                id: "cnsb-bundle".to_string(),
                name: "CNSB Bundle".to_string(),
                description: "Canonical Skill Bundle JSON format".to_string(),
                category: CapabilityCategory::Structure,
                since_version: Some("1.1".to_string()),
            },
            // Security
            Capability {
                id: "security-md".to_string(),
                name: "SECURITY.md".to_string(),
                description: "Security policy documentation".to_string(),
                category: CapabilityCategory::Security,
                since_version: Some("1.0".to_string()),
            },
            Capability {
                id: "threat-model".to_string(),
                name: "Threat Model".to_string(),
                description: "STRIDE/threat modeling documentation".to_string(),
                category: CapabilityCategory::Security,
                since_version: Some("1.0".to_string()),
            },
            Capability {
                id: "no-secrets".to_string(),
                name: "No Secrets".to_string(),
                description: "No hardcoded secrets in templates".to_string(),
                category: CapabilityCategory::Security,
                since_version: Some("1.0".to_string()),
            },
            // Templates
            Capability {
                id: "handlebars".to_string(),
                name: "Handlebars Templates".to_string(),
                description: "Valid Handlebars template syntax".to_string(),
                category: CapabilityCategory::Templates,
                since_version: Some("1.0".to_string()),
            },
            // MCP
            Capability {
                id: "mcp-server".to_string(),
                name: "MCP Server".to_string(),
                description: "Model Context Protocol server support".to_string(),
                category: CapabilityCategory::MCP,
                since_version: Some("1.1".to_string()),
            },
            Capability {
                id: "mcp-tools".to_string(),
                name: "MCP Tools".to_string(),
                description: "MCP tool definitions".to_string(),
                category: CapabilityCategory::MCP,
                since_version: Some("1.1".to_string()),
            },
            // Provenance
            Capability {
                id: "slsa-provenance".to_string(),
                name: "SLSA Provenance".to_string(),
                description: "SLSA Level 2+ provenance attestation".to_string(),
                category: CapabilityCategory::Provenance,
                since_version: Some("1.0".to_string()),
            },
            Capability {
                id: "sbom".to_string(),
                name: "SBOM".to_string(),
                description: "Software Bill of Materials".to_string(),
                category: CapabilityCategory::Provenance,
                since_version: Some("1.0".to_string()),
            },
            Capability {
                id: "sigstore-signed".to_string(),
                name: "Sigstore Signed".to_string(),
                description: "Keyless signing via Sigstore".to_string(),
                category: CapabilityCategory::Provenance,
                since_version: Some("1.0".to_string()),
            },
            // Integration
            Capability {
                id: "dagger-pipeline".to_string(),
                name: "Dagger Pipeline".to_string(),
                description: "Dagger CI/CD pipeline support".to_string(),
                category: CapabilityCategory::Integration,
                since_version: Some("1.1".to_string()),
            },
            Capability {
                id: "github-actions".to_string(),
                name: "GitHub Actions".to_string(),
                description: "GitHub Actions workflow".to_string(),
                category: CapabilityCategory::Integration,
                since_version: Some("1.0".to_string()),
            },
        ]
    }
}
