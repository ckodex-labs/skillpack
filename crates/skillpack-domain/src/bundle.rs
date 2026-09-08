//! Skill Bundle Domain Model
//!
//! CNSB (Canonical Skill Bundle) support

use serde::{Deserialize, Serialize};

/// Skill Bundle (CNSB format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBundle {
    #[serde(rename = "$schema", default)]
    pub schema: String,
    #[serde(rename = "apiVersion", default)]
    pub api_version: String,
    #[serde(default)]
    pub kind: String,
    pub metadata: BundleMetadata,
    #[serde(default)]
    pub skills: Vec<SkillDefinition>,
    #[serde(default)]
    pub agents: Vec<AgentDefinition>,
    #[serde(default)]
    pub governance: Option<GovernanceConfig>,
    #[serde(default)]
    pub lifecycle: Option<LifecycleConfig>,
    #[serde(default)]
    pub operations: Option<OperationsConfig>,
}

/// Bundle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default, rename = "type")]
    pub bundle_type: Option<String>,
    #[serde(default, rename = "dalVersion", alias = "galVersion")]
    pub dal_version: Option<String>,
    #[serde(default)]
    pub urn: Option<String>,
}

/// Individual skill definition within bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub entry_point: Option<String>,
    #[serde(default, alias = "capabilities")]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<SkillDependency>,
    #[serde(default)]
    pub asc: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default, rename = "galMin")]
    pub gal_min: Option<u8>,
    #[serde(default, rename = "galMax")]
    pub gal_max: Option<u8>,
}

/// Agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub gal: Option<u8>,
}

/// Operations block (v2 schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsConfig {
    #[serde(default)]
    pub lifecycle: Option<LifecycleConfig>,
}

/// Lifecycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    #[serde(default)]
    pub install: Option<LifecycleHook>,
    #[serde(default)]
    pub verify: Option<LifecycleHook>,
    #[serde(default)]
    pub test: Option<LifecycleHook>,
    #[serde(default)]
    pub clean: Option<LifecycleHook>,
    #[serde(default)]
    pub assess: Option<LifecycleHook>,
    #[serde(default)]
    pub publish: Option<LifecycleHook>,
}

/// Individual lifecycle hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHook {
    pub command: String,
    #[serde(default)]
    pub timeout: Option<String>,
}

/// Skill dependency reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDependency {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    #[serde(default)]
    pub slsa_level: u8,
    #[serde(default)]
    pub signing: Option<SigningConfig>,
    #[serde(default)]
    pub sbom: Option<SbomConfig>,
    #[serde(default)]
    pub threat_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub keyless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomConfig {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub version: String,
}

impl SkillBundle {
    /// Parse from JSON (auto-detects version)
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Detect schema version from $schema URL
    pub fn schema_version(&self) -> &str {
        if self.schema.contains("-v2") || self.schema.contains("/v2/") {
            "2.0.0"
        } else {
            "1.0.0"
        }
    }

    /// Migrate v1 schema to v2 in-place
    pub fn migrate_v1_to_v2(&mut self) {
        if self.schema_version() == "2.0.0" {
            return;
        }

        // 1. Update schema URL
        self.schema = "https://skillpack.dev/schemas/cnsb-v2.schema.json".to_string();

        // 2. Update apiVersion
        self.api_version = "cnsb.ckodex.dev/v2".to_string();

        // 3. Rename capabilities → extensions (handled by serde alias during round-trip)

        // 4. Add dal_version if missing
        if self.metadata.dal_version.is_none() {
            self.metadata.dal_version = Some("1.0.0".to_string());
        }

        // 5. Wrap lifecycle in operations block
        if self.lifecycle.is_some() && self.operations.is_none() {
            self.operations = Some(OperationsConfig {
                lifecycle: self.lifecycle.take(),
            });
        }
    }

    /// Total skills in bundle
    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    /// Total agents in bundle
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Check if bundle meets governance requirements
    pub fn meets_governance(&self, min_slsa: u8) -> bool {
        self.governance
            .as_ref()
            .map(|g| g.slsa_level >= min_slsa)
            .unwrap_or(false)
    }
}

/// Bundle assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleAssessment {
    pub bundle: BundleMetadata,
    pub skill_assessments: Vec<SkillAssessmentSummary>,
    pub aggregate_score: f64,
    pub aggregate_grade: String,
    pub governance_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAssessmentSummary {
    pub skill_name: String,
    pub skill_version: String,
    pub score: f64,
    pub grade: String,
    pub issue_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_v1_bundle() -> SkillBundle {
        SkillBundle {
            schema: "https://skillpack.dev/schemas/cnsb-v1.schema.json".into(),
            api_version: "cnsb.ckodex.dev/v1".into(),
            kind: "SkillBundle".into(),
            metadata: BundleMetadata {
                name: "test-bundle".into(),
                version: "1.0.0".into(),
                description: None,
                authors: vec![],
                homepage: None,
                repository: None,
                license: None,
                bundle_type: None,
                dal_version: None,
                urn: None,
            },
            skills: vec![SkillDefinition {
                name: "test-skill".into(),
                version: None,
                description: None,
                entry_point: None,
                extensions: vec![],
                dependencies: vec![],
                asc: vec![],
                examples: vec![],
                gal_min: None,
                gal_max: None,
            }],
            agents: vec![],
            governance: None,
            lifecycle: Some(LifecycleConfig {
                install: Some(LifecycleHook {
                    command: "echo install".into(),
                    timeout: None,
                }),
                verify: None,
                test: None,
                clean: None,
                assess: None,
                publish: None,
            }),
            operations: None,
        }
    }

    #[test]
    fn schema_version_detects_v1() {
        let bundle = minimal_v1_bundle();
        assert_eq!(bundle.schema_version(), "1.0.0");
    }

    #[test]
    fn schema_version_detects_v2() {
        let mut bundle = minimal_v1_bundle();
        bundle.schema = "https://skillpack.dev/schemas/cnsb-v2.schema.json".into();
        assert_eq!(bundle.schema_version(), "2.0.0");
    }

    #[test]
    fn migrate_v1_to_v2_updates_schema_and_api_version() {
        let mut bundle = minimal_v1_bundle();
        bundle.migrate_v1_to_v2();
        assert_eq!(bundle.schema_version(), "2.0.0");
        assert_eq!(bundle.api_version, "cnsb.ckodex.dev/v2");
    }

    #[test]
    fn migrate_v1_to_v2_adds_dal_version() {
        let mut bundle = minimal_v1_bundle();
        assert!(bundle.metadata.dal_version.is_none());
        bundle.migrate_v1_to_v2();
        assert_eq!(bundle.metadata.dal_version, Some("1.0.0".into()));
    }

    #[test]
    fn migrate_v1_to_v2_wraps_lifecycle_in_operations() {
        let mut bundle = minimal_v1_bundle();
        assert!(bundle.lifecycle.is_some());
        assert!(bundle.operations.is_none());
        bundle.migrate_v1_to_v2();
        assert!(bundle.lifecycle.is_none());
        assert!(bundle.operations.is_some());
        assert!(bundle.operations.as_ref().unwrap().lifecycle.is_some());
    }

    #[test]
    fn migrate_v1_to_v2_is_idempotent() {
        let mut bundle = minimal_v1_bundle();
        bundle.migrate_v1_to_v2();
        let first = bundle.clone();
        bundle.migrate_v1_to_v2();
        assert_eq!(first.schema, bundle.schema);
        assert_eq!(first.api_version, bundle.api_version);
        assert_eq!(first.metadata.dal_version, bundle.metadata.dal_version);
    }

    #[test]
    fn serde_alias_capabilities_to_extensions() {
        let json = r#"{"$schema":"https://skillpack.dev/schemas/cnsb-v1.schema.json","apiVersion":"cnsb.ckodex.dev/v1","kind":"SkillBundle","metadata":{"name":"t","version":"1.0.0"},"skills":[{"name":"s","capabilities":["web-search"]}]}"#;
        let bundle: SkillBundle = serde_json::from_str(json).unwrap();
        assert_eq!(bundle.skills[0].extensions, vec!["web-search"]);
    }

    #[test]
    fn serde_alias_gal_version_to_dal_version() {
        let json = r#"{"$schema":"...","apiVersion":"v1","kind":"SkillBundle","metadata":{"name":"t","version":"1.0.0","galVersion":"2.0.0"},"skills":[]}"#;
        let bundle: SkillBundle = serde_json::from_str(json).unwrap();
        assert_eq!(bundle.metadata.dal_version, Some("2.0.0".into()));
    }

    #[test]
    fn roundtrip_preserves_v2_fields() {
        let bundle = minimal_v1_bundle();
        let json = bundle.to_json().unwrap();
        let parsed = SkillBundle::from_json(&json).unwrap();
        assert_eq!(parsed.metadata.name, bundle.metadata.name);
        assert_eq!(parsed.skills.len(), bundle.skills.len());
    }
}
