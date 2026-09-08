//! Skill Inventory Manifest
//!
//! Used by `skillpack export` / `skillpack import` to snapshot
//! the set of installed skills for backup, migration, or provisioning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single skill entry in the exported inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    /// Skill name (from SKILL.md or skill.cnsb.json)
    pub name: String,

    /// Skill version
    pub version: String,

    /// Agent tool that owns this installation (windsurf, claude, cursor …)
    pub tool: String,

    /// Absolute path to the skill directory on disk
    pub location: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the skill is currently disabled (.disabled marker present)
    #[serde(default)]
    pub disabled: bool,

    /// Whether the path is a symlink
    #[serde(default)]
    pub symlink: bool,

    /// Optional source reference (OCI ref, git URL) for reinstallation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// The full skill inventory manifest (output of `skillpack export`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInventoryManifest {
    /// Schema version for forward-compatibility
    pub schema_version: u32,

    /// RFC-3339 timestamp when this manifest was generated
    pub exported_at: DateTime<Utc>,

    /// Hostname of the machine where the export was performed
    pub hostname: String,

    /// All discovered skills
    pub skills: Vec<SkillEntry>,

    /// Total count (convenience field)
    pub total: usize,
}

impl SkillInventoryManifest {
    pub fn new(skills: Vec<SkillEntry>) -> Self {
        let total = skills.len();
        Self {
            schema_version: 1,
            exported_at: Utc::now(),
            hostname: hostname_or_unknown(),
            skills,
            total,
        }
    }
}

fn hostname_or_unknown() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, tool: &str) -> SkillEntry {
        SkillEntry {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            tool: tool.to_string(),
            location: format!("/skills/{}", name),
            description: Some(format!("Description of {}", name)),
            disabled: false,
            symlink: false,
            source: None,
        }
    }

    #[test]
    fn test_manifest_new_sets_total() {
        let skills = vec![make_entry("a", "windsurf"), make_entry("b", "claude")];
        let m = SkillInventoryManifest::new(skills);
        assert_eq!(m.total, 2);
        assert_eq!(m.skills.len(), 2);
    }

    #[test]
    fn test_manifest_new_empty() {
        let m = SkillInventoryManifest::new(vec![]);
        assert_eq!(m.total, 0);
        assert!(m.skills.is_empty());
    }

    #[test]
    fn test_manifest_schema_version_is_one() {
        let m = SkillInventoryManifest::new(vec![]);
        assert_eq!(m.schema_version, 1);
    }

    #[test]
    fn test_manifest_hostname_is_populated() {
        let m = SkillInventoryManifest::new(vec![]);
        assert!(!m.hostname.is_empty());
    }

    #[test]
    fn test_manifest_round_trips_json() {
        let skills = vec![
            make_entry("skill-a", "windsurf"),
            SkillEntry {
                name: "skill-b".to_string(),
                version: "2.0.0".to_string(),
                tool: "claude".to_string(),
                location: "/skills/skill-b".to_string(),
                description: None,
                disabled: true,
                symlink: true,
                source: Some("ghcr.io/org/skill-b:2.0.0".to_string()),
            },
        ];
        let original = SkillInventoryManifest::new(skills);
        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored: SkillInventoryManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.schema_version, original.schema_version);
        assert_eq!(restored.total, original.total);
        assert_eq!(restored.skills.len(), 2);

        let a = &restored.skills[0];
        assert_eq!(a.name, "skill-a");
        assert_eq!(a.tool, "windsurf");
        assert!(!a.disabled);
        assert!(!a.symlink);
        assert!(a.source.is_none());

        let b = &restored.skills[1];
        assert_eq!(b.name, "skill-b");
        assert!(b.disabled);
        assert!(b.symlink);
        assert_eq!(b.source.as_deref(), Some("ghcr.io/org/skill-b:2.0.0"));
    }

    #[test]
    fn test_skill_entry_description_omitted_when_none() {
        let entry = SkillEntry {
            name: "no-desc".to_string(),
            version: "0.1.0".to_string(),
            tool: "cursor".to_string(),
            location: "/skills/no-desc".to_string(),
            description: None,
            disabled: false,
            symlink: false,
            source: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("description"));
    }

    #[test]
    fn test_skill_entry_source_omitted_when_none() {
        let entry = make_entry("no-src", "agents");
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("\"source\""));
    }

    #[test]
    fn test_skill_entry_source_included_when_present() {
        let mut entry = make_entry("with-src", "agents");
        entry.source = Some("oci://registry/skill:1.0.0".to_string());
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("oci://registry/skill:1.0.0"));
    }

    #[test]
    fn test_manifest_exported_at_is_recent() {
        let before = chrono::Utc::now();
        let m = SkillInventoryManifest::new(vec![]);
        let after = chrono::Utc::now();
        assert!(m.exported_at >= before);
        assert!(m.exported_at <= after);
    }
}
