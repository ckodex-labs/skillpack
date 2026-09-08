//! Skill Discovery Module
//!
//! Implements skill.txt and .well-known/skills.json parsing for federated discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parsed skill.txt file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillTxt {
    /// Registered skills with their locations
    pub skills: HashMap<String, SkillRegistration>,
    /// Access control policies
    pub policies: Vec<SkillPolicy>,
}

/// A skill registration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRegistration {
    pub name: String,
    pub version: String,
    pub oci_ref: Option<String>,
    pub git_ref: Option<String>,
    pub sha256: Option<String>,
}

/// Access policy for skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicy {
    pub pattern: String,
    pub allow: bool,
    pub principals: Vec<String>,
}

/// .well-known/skills.json discovery document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsDiscovery {
    pub version: String,
    pub issuer: String,
    pub registry_endpoint: String,
    pub skills: Vec<DiscoveredSkill>,
    #[serde(default)]
    pub verification: VerificationInfo,
}

/// A discovered skill from .well-known
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSkill {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub oci_ref: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub gal_min: u8,
    #[serde(default)]
    pub gal_max: u8,
}

/// Verification endpoints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub signature_endpoint: Option<String>,
    pub sbom_endpoint: Option<String>,
    pub provenance_endpoint: Option<String>,
}

impl SkillTxt {
    /// Parse a skill.txt file content
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut result = SkillTxt::default();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse skill registration: Skill: name version oci_ref
            if let Some(stripped) = line.strip_prefix("Skill:") {
                let parts: Vec<&str> = stripped.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    let oci_ref = parts.get(2).map(|s| s.to_string());

                    result.skills.insert(
                        name.clone(),
                        SkillRegistration {
                            name,
                            version,
                            oci_ref,
                            git_ref: None,
                            sha256: None,
                        },
                    );
                }
            }

            // Parse policy: Allow/Deny pattern principal1,principal2
            if line.starts_with("Allow:") || line.starts_with("Deny:") {
                let allow = line.starts_with("Allow:");
                let content = if allow { &line[6..] } else { &line[5..] };
                let parts: Vec<&str> = content.split_whitespace().collect();

                if !parts.is_empty() {
                    let pattern = parts[0].to_string();
                    let principals: Vec<String> = parts
                        .get(1)
                        .map(|p| p.split(',').map(|s| s.to_string()).collect())
                        .unwrap_or_default();

                    result.policies.push(SkillPolicy {
                        pattern,
                        allow,
                        principals,
                    });
                }
            }
        }

        Ok(result)
    }
}

impl SkillsDiscovery {
    /// Load from JSON content
    pub fn from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_txt() {
        let content = r#"
# Skills Registry
Skill: my-skill 1.0.0 ghcr.io/example/my-skill:1.0.0

# Access Policies
Allow: * authenticated
Deny: internal/* external
"#;
        let txt = SkillTxt::parse(content).unwrap();
        assert_eq!(txt.skills.len(), 1);
        assert_eq!(txt.policies.len(), 2);
        assert!(txt.policies[0].allow);
        assert!(!txt.policies[1].allow);
    }

    #[test]
    fn test_skills_discovery_json() {
        let json = r#"{
            "version": "1.0",
            "issuer": "https://example.com",
            "registry_endpoint": "https://registry.example.com",
            "skills": [{
                "name": "test-skill",
                "description": "A test skill",
                "oci_ref": "ghcr.io/example/test:1.0.0",
                "tags": ["ai", "test"],
                "gal_min": 1,
                "gal_max": 3
            }]
        }"#;

        let discovery = SkillsDiscovery::from_json(json).unwrap();
        assert_eq!(discovery.skills.len(), 1);
        assert_eq!(discovery.skills[0].name, "test-skill");
    }
}
