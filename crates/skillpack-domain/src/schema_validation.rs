//! Schema Validation Module
//!
//! Provides JSON Schema validation for CKODEX artifacts:
//! - CNSB (Skill Bundles)
//! - CNAAB (Agent App Bundles)
//! - Evidence Envelopes
//! - Policy Bundles

use jsonschema::Validator;
use serde_json::Value;
use std::fmt;

/// Embedded schemas (compile-time inclusion) - using standalone versions with embedded common types
mod embedded {
    pub const CNSB_V1: &str = include_str!("../../../schemas/cnsb/v1/cnsb-standalone.schema.json");
    pub const CNAAB_V1: &str =
        include_str!("../../../schemas/cnaab/v1/cnaab-standalone.schema.json");
    pub const EVIDENCE_V1: &str =
        include_str!("../../../schemas/evidence/v1/envelope-standalone.schema.json");
    pub const POLICY_V1: &str =
        include_str!("../../../schemas/prove/v1/policy-standalone.schema.json");
}

/// Schema validation errors
#[derive(Debug)]
pub struct SchemaValidationError {
    pub schema_type: SchemaType,
    pub errors: Vec<String>,
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} validation failed with {} error(s): {}",
            self.schema_type,
            self.errors.len(),
            self.errors.join("; ")
        )
    }
}

impl std::error::Error for SchemaValidationError {}

/// Schema types supported by the validator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    /// Cloud-Native Skill Bundle (CNSB)
    SkillBundle,
    /// CKODEX Agent App Bundle (CNAAB)
    AgentAppBundle,
    /// OpenEvidence Envelope
    EvidenceEnvelope,
    /// Prove Policy Bundle
    PolicyBundle,
}

impl fmt::Display for SchemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkillBundle => write!(f, "CNSB SkillBundle"),
            Self::AgentAppBundle => write!(f, "CNAAB AgentAppBundle"),
            Self::EvidenceEnvelope => write!(f, "Evidence Envelope"),
            Self::PolicyBundle => write!(f, "Policy Bundle"),
        }
    }
}

/// Schema validator with pre-compiled validators
pub struct SchemaValidator {
    cnsb_validator: Validator,
    cnaab_validator: Validator,
    evidence_validator: Validator,
    policy_validator: Validator,
}

impl SchemaValidator {
    /// Create a new schema validator with embedded schemas
    pub fn new() -> Result<Self, anyhow::Error> {
        let cnsb_schema: Value = serde_json::from_str(embedded::CNSB_V1)?;
        let cnaab_schema: Value = serde_json::from_str(embedded::CNAAB_V1)?;
        let evidence_schema: Value = serde_json::from_str(embedded::EVIDENCE_V1)?;
        let policy_schema: Value = serde_json::from_str(embedded::POLICY_V1)?;

        Ok(Self {
            cnsb_validator: Validator::new(&cnsb_schema)?,
            cnaab_validator: Validator::new(&cnaab_schema)?,
            evidence_validator: Validator::new(&evidence_schema)?,
            policy_validator: Validator::new(&policy_schema)?,
        })
    }

    /// Validate a CNSB skill bundle
    pub fn validate_skill_bundle(&self, json: &Value) -> Result<(), SchemaValidationError> {
        self.validate_with(&self.cnsb_validator, json, SchemaType::SkillBundle)
    }

    /// Validate a CNAAB agent app bundle
    pub fn validate_agent_app(&self, json: &Value) -> Result<(), SchemaValidationError> {
        self.validate_with(&self.cnaab_validator, json, SchemaType::AgentAppBundle)
    }

    /// Validate an evidence envelope
    pub fn validate_evidence(&self, json: &Value) -> Result<(), SchemaValidationError> {
        self.validate_with(&self.evidence_validator, json, SchemaType::EvidenceEnvelope)
    }

    /// Validate a policy bundle
    pub fn validate_policy(&self, json: &Value) -> Result<(), SchemaValidationError> {
        self.validate_with(&self.policy_validator, json, SchemaType::PolicyBundle)
    }

    /// Auto-detect schema type and validate
    pub fn validate_auto(&self, json: &Value) -> Result<SchemaType, SchemaValidationError> {
        let api_version = json.get("apiVersion").and_then(|v| v.as_str());
        let kind = json.get("kind").and_then(|v| v.as_str());

        match (api_version, kind) {
            (Some("cnsb.ckodex.org/v1"), Some("SkillBundle")) => {
                self.validate_skill_bundle(json)?;
                Ok(SchemaType::SkillBundle)
            }
            (Some("cnaab.ckodex.org/v1"), Some("AgentAppBundle")) => {
                self.validate_agent_app(json)?;
                Ok(SchemaType::AgentAppBundle)
            }
            (Some("prove.ckodex.org/v1"), Some("PolicyBundle")) => {
                self.validate_policy(json)?;
                Ok(SchemaType::PolicyBundle)
            }
            _ if json.get("subject").is_some() && json.get("statement").is_some() => {
                self.validate_evidence(json)?;
                Ok(SchemaType::EvidenceEnvelope)
            }
            _ => Err(SchemaValidationError {
                schema_type: SchemaType::SkillBundle,
                errors: vec!["Unknown schema type: could not detect apiVersion/kind".to_string()],
            }),
        }
    }

    fn validate_with(
        &self,
        validator: &Validator,
        json: &Value,
        schema_type: SchemaType,
    ) -> Result<(), SchemaValidationError> {
        let result = validator.validate(json);

        if result.is_ok() {
            Ok(())
        } else {
            let errors: Vec<String> = validator
                .iter_errors(json)
                .map(|e| format!("{}: {}", e.instance_path(), e))
                .collect();

            Err(SchemaValidationError {
                schema_type,
                errors,
            })
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new().expect("Failed to initialize schema validator")
    }
}

/// Quick validation function for CNSB bundles
pub fn validate_cnsb(json: &Value) -> Result<(), SchemaValidationError> {
    let validator = SchemaValidator::new().map_err(|e| SchemaValidationError {
        schema_type: SchemaType::SkillBundle,
        errors: vec![e.to_string()],
    })?;
    validator.validate_skill_bundle(json)
}

/// Quick validation function for evidence envelopes
pub fn validate_evidence(json: &Value) -> Result<(), SchemaValidationError> {
    let validator = SchemaValidator::new().map_err(|e| SchemaValidationError {
        schema_type: SchemaType::EvidenceEnvelope,
        errors: vec![e.to_string()],
    })?;
    validator.validate_evidence(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_cnsb_minimal() {
        let validator = SchemaValidator::new().unwrap();

        let valid = json!({
            "apiVersion": "cnsb.ckodex.org/v1",
            "kind": "SkillBundle",
            "metadata": {
                "name": "test-bundle",
                "urn": "urn:ckodex:cnsb:test:1.0.0",
                "version": "1.0.0"
            },
            "skills": [{
                "id": "test-skill",
                "entry": "skills://test#run",
                "galMin": 1,
                "galMax": 3
            }]
        });

        assert!(validator.validate_skill_bundle(&valid).is_ok());
    }

    #[test]
    fn test_invalid_cnsb_missing_skills() {
        let validator = SchemaValidator::new().unwrap();

        let invalid = json!({
            "apiVersion": "cnsb.ckodex.org/v1",
            "kind": "SkillBundle",
            "metadata": {
                "name": "test-bundle",
                "version": "1.0.0"
            }
        });

        let result = validator.validate_skill_bundle(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_detection() {
        let validator = SchemaValidator::new().unwrap();

        let cnsb = json!({
            "apiVersion": "cnsb.ckodex.org/v1",
            "kind": "SkillBundle",
            "metadata": {
                "name": "test",
                "urn": "urn:test",
                "version": "1.0.0"
            },
            "skills": [{
                "id": "s1",
                "entry": "test#run",
                "galMin": 1,
                "galMax": 5
            }]
        });

        let result = validator.validate_auto(&cnsb);
        assert!(matches!(result, Ok(SchemaType::SkillBundle)));
    }
}
