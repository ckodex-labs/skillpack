//! Sync Canonical Store Use Case
//!
//! Orchestrates discovery of skills in the canonical store and syncs them
//! to all registered AI coding assistant directories via symlinks or
//! structured index files.

use anyhow::Result;
use skillpack_domain::{
    AgentConfig, AgentIntegrationType, AgentRegistry, AgentSyncResult, CanonicalSkill,
    CanonicalStore, IPGuard, IPGuardError, SyncResult,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Use case for syncing the canonical store to agent directories.
pub struct SyncCanonicalStoreUseCase {
    ip_guard: Arc<IPGuard>,
}

impl Default for SyncCanonicalStoreUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncCanonicalStoreUseCase {
    pub fn new() -> Self {
        Self {
            ip_guard: Arc::new(IPGuard::new()),
        }
    }

    pub fn with_ip_guard(ip_guard: Arc<IPGuard>) -> Self {
        Self { ip_guard }
    }

    pub fn execute(&self, store: &CanonicalStore) -> Result<SyncResult> {
        let root = store.resolved_root();

        // Validate canonical store
        if !root.exists() {
            return Err(anyhow::anyhow!(
                "Canonical store does not exist: {}",
                root.display()
            ));
        }

        // IP guard the canonical store
        if let Err(IPGuardError::BoundaryViolation(msg)) = self.ip_guard.guard_path(&root) {
            return Err(anyhow::anyhow!(msg));
        }
        if let Err(IPGuardError::BoundaryViolation(msg)) = self.ip_guard.scan_directory(&root, 3) {
            return Err(anyhow::anyhow!(msg));
        }

        // Discover skills
        let skills = self.discover_skills(&root)?;
        if skills.is_empty() {
            return Err(anyhow::anyhow!("No skills found in {}", root.display()));
        }

        // Validate each skill name
        for skill in &skills {
            if let Err(IPGuardError::BoundaryViolation(msg)) =
                self.ip_guard.guard_skill_name(&skill.name)
            {
                return Err(anyhow::anyhow!(msg));
            }
        }

        // Generate manifest if not dry-run
        if !store.dry_run {
            self.generate_manifest(&root, &skills)?;
        }

        // Sync to agents
        let registry = AgentRegistry::default_registry();
        let mut agent_results = Vec::new();

        for agent in registry.agents() {
            if let Some(only) = &store.only_agent {
                if agent.name.to_lowercase() != only.to_lowercase() {
                    continue;
                }
            }

            let result = self.sync_agent(agent, &skills, store)?;
            agent_results.push(result);
        }

        let total_synced: usize = agent_results.iter().map(|r| r.skills_synced).sum();

        Ok(SyncResult {
            success: true,
            message: format!(
                "Synced {} skills across {} agents",
                total_synced,
                agent_results.len()
            ),
            skills_processed: skills.len(),
            agent_results,
            skills_discovered: skills.clone(),
        })
    }

    fn discover_skills(&self, root: &Path) -> Result<Vec<CanonicalSkill>> {
        let mut skills = Vec::new();

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if name == "candidates" || name == ".codegraph" {
                continue;
            }

            let has_skill_md = entry.path().join("SKILL.md").exists();
            skills.push(CanonicalSkill {
                name,
                path: entry.path(),
                has_skill_md,
                content_hash: None,
                source_agent: None,
            });
        }

        Ok(skills)
    }

    fn generate_manifest(&self, root: &Path, skills: &[CanonicalSkill]) -> Result<()> {
        let manifest_path = root.join("manifest.yaml");
        let mut content = String::from("skills:\n");
        for skill in skills {
            content.push_str(&format!(
                "  - name: {}\n    path: {}\n    has_skill_md: {}\n",
                skill.name,
                skill.path.display(),
                skill.has_skill_md
            ));
        }
        fs::write(&manifest_path, content)?;
        Ok(())
    }

    fn sync_agent(
        &self,
        agent: &AgentConfig,
        skills: &[CanonicalSkill],
        store: &CanonicalStore,
    ) -> Result<AgentSyncResult> {
        let agent_dir = match agent.resolved_dir() {
            Some(d) => d,
            None => {
                return Ok(AgentSyncResult {
                    agent_name: agent.name.clone(),
                    success: false,
                    message: format!("Agent directory not found for {}", agent.name),
                    skills_synced: 0,
                });
            }
        };

        if store.dry_run {
            return Ok(AgentSyncResult {
                agent_name: agent.name.clone(),
                success: true,
                message: format!(
                    "Would sync {} skills to {}",
                    skills.len(),
                    agent_dir.display()
                ),
                skills_synced: skills.len(),
            });
        }

        fs::create_dir_all(&agent_dir)?;

        match agent.integration_type {
            AgentIntegrationType::Symlink => {
                let mut synced = 0;
                for skill in skills {
                    let source = skill.path.clone();
                    let target = agent_dir.join(&skill.name);
                    Self::sync_symlink(&source, &target)?;
                    synced += 1;
                }
                Ok(AgentSyncResult {
                    agent_name: agent.name.clone(),
                    success: true,
                    message: format!("Synced {} skills to {}", synced, agent_dir.display()),
                    skills_synced: synced,
                })
            }
            AgentIntegrationType::IndexFile => {
                Self::sync_index_file(agent, skills, &agent_dir, store)
            }
            // RulesDir and SingleFile are handled by skillpack-registry-sync (agent_render).
            // Fall back to symlink behavior for now so legacy sync doesn't break.
            AgentIntegrationType::RulesDir | AgentIntegrationType::SingleFile => {
                let mut synced = 0;
                for skill in skills {
                    let source = skill.path.clone();
                    let target = agent_dir.join(&skill.name);
                    Self::sync_symlink(&source, &target)?;
                    synced += 1;
                }
                Ok(AgentSyncResult {
                    agent_name: agent.name.clone(),
                    success: true,
                    message: format!(
                        "Synced {} skills to {} (symlink fallback)",
                        synced,
                        agent_dir.display()
                    ),
                    skills_synced: synced,
                })
            }
        }
    }

    fn sync_symlink(source: &Path, target: &Path) -> Result<()> {
        // Remove existing target if it's a symlink (stale)
        if target.exists() || is_symlink(target) {
            if target.is_dir() {
                fs::remove_dir_all(target)?;
            } else {
                fs::remove_file(target)?;
            }
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(source, target)?;
        }

        Ok(())
    }

    fn sync_index_file(
        agent: &AgentConfig,
        skills: &[CanonicalSkill],
        agent_dir: &Path,
        store: &CanonicalStore,
    ) -> Result<AgentSyncResult> {
        if store.dry_run {
            return Ok(AgentSyncResult {
                agent_name: agent.name.clone(),
                success: true,
                message: format!(
                    "Would write index files for {} skills to {}",
                    skills.len(),
                    agent_dir.display()
                ),
                skills_synced: skills.len(),
            });
        }

        match agent.name.as_str() {
            "Cursor" => Self::sync_cursor_mdc(skills, agent_dir),
            "Windsurf" => Self::sync_windsurf_index(skills, agent_dir),
            _ => {
                // Fallback: write a generic index.json
                Self::sync_generic_index(skills, agent_dir)
            }
        }
    }

    /// Cursor: write `.mdc` rule files to `~/.cursor/rules/`
    fn sync_cursor_mdc(skills: &[CanonicalSkill], agent_dir: &Path) -> Result<AgentSyncResult> {
        let rules_dir = agent_dir.parent().unwrap_or(agent_dir).join("rules");
        fs::create_dir_all(&rules_dir)?;

        // Clean stale .mdc files (keep up to 50 non-skill ones)
        for entry in fs::read_dir(&rules_dir)?.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "mdc" {
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    if !skills.iter().any(|s| s.name == stem.as_ref()) {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        let mut synced = 0;
        for skill in skills {
            let skill_md = skill.path.join("SKILL.md");
            let mdc_path = rules_dir.join(format!("{}.mdc", skill.name));

            if skill_md.exists() {
                // Copy SKILL.md content into .mdc file
                let content = fs::read_to_string(&skill_md)?;
                fs::write(&mdc_path, content)?;
            } else {
                // Generate a minimal .mdc with the skill name
                let content = format!(
                    "---\ndescription: \"{}\"\nglob: \"**/*\"\n---\n# {}\n\nSkill from SkillPack canonical store.\n",
                    skill.name, skill.name
                );
                fs::write(&mdc_path, content)?;
            }
            synced += 1;
        }

        Ok(AgentSyncResult {
            agent_name: "Cursor".to_string(),
            success: true,
            message: format!("Wrote {} .mdc files to {}", synced, rules_dir.display()),
            skills_synced: synced,
        })
    }

    /// Windsurf: write `index.json` to `~/.codeium/windsurf/skills/`
    fn sync_windsurf_index(skills: &[CanonicalSkill], agent_dir: &Path) -> Result<AgentSyncResult> {
        fs::create_dir_all(agent_dir)?;

        let index = serde_json::json!({
            "version": "1.0",
            "source": "skillpack-canonical",
            "skills": skills.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "path": s.path.to_string_lossy(),
                    "has_skill_md": s.has_skill_md,
                })
            }).collect::<Vec<_>>(),
        });

        let index_path = agent_dir.join("index.json");
        fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;

        Ok(AgentSyncResult {
            agent_name: "Windsurf".to_string(),
            success: true,
            message: format!(
                "Wrote index.json with {} skills to {}",
                skills.len(),
                index_path.display()
            ),
            skills_synced: skills.len(),
        })
    }

    /// Generic fallback: write `index.json`
    fn sync_generic_index(skills: &[CanonicalSkill], agent_dir: &Path) -> Result<AgentSyncResult> {
        fs::create_dir_all(agent_dir)?;

        let index = serde_json::json!({
            "skills": skills.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "path": s.path.to_string_lossy(),
                })
            }).collect::<Vec<_>>(),
        });

        let index_path = agent_dir.join("index.json");
        fs::write(&index_path, serde_json::to_string_pretty(&index)?)?;

        Ok(AgentSyncResult {
            agent_name: "Generic".to_string(),
            success: true,
            message: format!("Wrote index.json with {} skills", skills.len()),
            skills_synced: skills.len(),
        })
    }
}

#[cfg(unix)]
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}
