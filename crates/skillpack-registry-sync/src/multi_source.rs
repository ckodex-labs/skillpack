//! Multi-source skill discovery.
//!
//! Scans every known agent path on the local machine and aggregates
//! all discovered skills into a unified list for deduplication.

use skillpack_domain::{AgentIntegrationType, AgentRegistry, CanonicalSkill};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A skill discovered from an agent-specific directory.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
    pub has_skill_md: bool,
    pub source_agent: String,
}

/// Scan all registered agent paths and return every skill found.
///
/// Skips agents whose directories don't exist. Does not dedup — caller handles that.
pub fn discover_all() -> Vec<DiscoveredSkill> {
    let registry = AgentRegistry::default_registry();
    let mut skills: Vec<DiscoveredSkill> = Vec::new();

    for agent in registry.agents() {
        let Some(dir) = agent.resolved_dir() else {
            continue;
        };

        match agent.integration_type {
            AgentIntegrationType::Symlink | AgentIntegrationType::RulesDir => {
                skills.extend(scan_skills_dir(&dir, &agent.name));
            }
            AgentIntegrationType::IndexFile | AgentIntegrationType::SingleFile => {
                // Index/single-file agents don't store skill directories — skip scanning.
            }
        }
    }

    skills
}

/// Also scan the well-known shared paths that are agent-agnostic.
pub fn discover_shared(roots: &[PathBuf]) -> Vec<DiscoveredSkill> {
    let mut skills = Vec::new();
    for root in roots {
        if root.exists() {
            skills.extend(scan_skills_dir(root, "shared"));
        }
    }
    skills
}

/// Scan a skills directory, returning one entry per skill subdirectory.
fn scan_skills_dir(dir: &Path, source_agent: &str) -> Vec<DiscoveredSkill> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let skip: HashSet<&str> = [".codegraph", "candidates", "cache", ".git"].into();

    entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if skip.contains(name.as_str()) || name.starts_with('.') {
                return None;
            }
            let path = e.path();
            let has_skill_md = path.join("SKILL.md").exists();
            Some(DiscoveredSkill {
                name,
                path,
                has_skill_md,
                source_agent: source_agent.to_string(),
            })
        })
        .collect()
}

/// Convert a `DiscoveredSkill` into a `CanonicalSkill`.
pub fn to_canonical(s: DiscoveredSkill) -> CanonicalSkill {
    CanonicalSkill {
        name: s.name,
        path: s.path,
        has_skill_md: s.has_skill_md,
        content_hash: None,
        source_agent: Some(s.source_agent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_all_does_not_panic_with_missing_dirs() {
        // Should return successfully even when most paths don't exist.
        let skills = discover_all();
        // At minimum, we should find skills from paths that exist on this machine.
        // Not asserting a specific count — environment-dependent.
        let _ = skills; // just ensure it doesn't panic
    }

    #[test]
    fn test_scan_skills_dir_skips_dotfiles() {
        let tmp = std::env::temp_dir().join("sctl-test-scan");
        let _ = std::fs::create_dir_all(&tmp);
        let _ = std::fs::create_dir_all(tmp.join(".hidden"));
        let _ = std::fs::create_dir_all(tmp.join("visible-skill"));
        let skills = scan_skills_dir(&tmp, "test");
        assert!(skills.iter().all(|s| !s.name.starts_with('.')));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
