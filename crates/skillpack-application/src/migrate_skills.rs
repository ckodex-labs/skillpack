//! Migrate Skills Use Case
//!
//! Orchestrates batch migration of physical skills from agent directories
//! into the canonical store, replacing originals with symlinks.

use anyhow::Result;
use skillpack_domain::{
    AgentRegistry, IPGuard, IPGuardError, MigrateAllRequest, MigrateOutcome, MigrateResult,
    MigrateSummary,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Use case for migrating all physical skills to the canonical store.
pub struct MigrateSkillsUseCase {
    ip_guard: Arc<IPGuard>,
}

impl Default for MigrateSkillsUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrateSkillsUseCase {
    pub fn new() -> Self {
        Self {
            ip_guard: Arc::new(IPGuard::new()),
        }
    }

    pub fn with_ip_guard(ip_guard: Arc<IPGuard>) -> Self {
        Self { ip_guard }
    }

    pub fn execute(&self, request: MigrateAllRequest) -> Result<Vec<MigrateResult>> {
        let canonical_root = request.canonical_root;
        let registry = AgentRegistry::default_registry();
        let mut results = Vec::new();

        for agent in registry.agents() {
            let agent_dir = match agent.resolved_dir() {
                Some(d) => d,
                None => continue,
            };

            let entries = match fs::read_dir(&agent_dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let skill_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let result = self.migrate_single_skill(&path, &skill_name, agent, &canonical_root);
                results.push(result);
            }
        }

        Ok(results)
    }

    fn migrate_single_skill(
        &self,
        source_path: &Path,
        skill_name: &str,
        agent: &skillpack_domain::AgentConfig,
        canonical_root: &Path,
    ) -> MigrateResult {
        // Already a symlink — nothing to do
        if Self::is_symlink(source_path) {
            return MigrateResult::new(
                &agent.name,
                skill_name,
                source_path,
                MigrateOutcome::SkippedAlreadyLinked,
            );
        }

        // IP boundary check
        if let Err(IPGuardError::BoundaryViolation(msg)) = self.ip_guard.guard_path(source_path) {
            return MigrateResult::new(
                &agent.name,
                skill_name,
                source_path,
                MigrateOutcome::SkippedIPViolation(msg),
            );
        }
        if let Err(IPGuardError::BoundaryViolation(msg)) =
            self.ip_guard.guard_skill_name(skill_name)
        {
            return MigrateResult::new(
                &agent.name,
                skill_name,
                source_path,
                MigrateOutcome::SkippedIPViolation(msg),
            );
        }

        let target_path = canonical_root.join(skill_name);

        // Case 2: canonical already has this skill — replace physical with symlink
        if target_path.exists() {
            match Self::replace_with_symlink(source_path, &target_path) {
                Ok(_) => MigrateResult::new(
                    &agent.name,
                    skill_name,
                    source_path,
                    MigrateOutcome::ReplacedWithLink,
                ),
                Err(e) => MigrateResult::new(
                    &agent.name,
                    skill_name,
                    source_path,
                    MigrateOutcome::Failed(e.to_string()),
                ),
            }
        } else {
            // Case 3: new skill — copy to canonical, remove original, symlink back
            match self.copy_and_symlink(source_path, skill_name, &target_path) {
                Ok(_) => MigrateResult::new(
                    &agent.name,
                    skill_name,
                    source_path,
                    MigrateOutcome::Migrated,
                ),
                Err(e) => MigrateResult::new(
                    &agent.name,
                    skill_name,
                    source_path,
                    MigrateOutcome::Failed(e.to_string()),
                ),
            }
        }
    }

    fn is_symlink(path: &Path) -> bool {
        fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn replace_with_symlink(source: &Path, target: &Path) -> Result<()> {
        fs::remove_dir_all(source)?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, source)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, source)?;
        }
        Ok(())
    }

    fn copy_and_symlink(&self, source: &Path, skill_name: &str, target: &Path) -> Result<()> {
        // Scan contents for IP violations before copying
        if let Err(IPGuardError::BoundaryViolation(msg)) = self.ip_guard.scan_directory(source, 3) {
            return Err(anyhow::anyhow!(msg));
        }

        fs::create_dir_all(target.parent().unwrap_or(Path::new(".")))?;
        Self::copy_dir_all(source, target)?;

        // Ensure SKILL.md exists
        let skill_md = target.join("SKILL.md");
        if !skill_md.exists() {
            let default_template = format!(
                r#"---
description: "Migrated skill for {}."
---
# {}

Migrated from {} on {}.
"#,
                skill_name,
                skill_name,
                source.display(),
                chrono::Utc::now().to_rfc3339()
            );
            fs::write(&skill_md, default_template)?;
        }

        fs::remove_dir_all(source)?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, source)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, source)?;
        }

        Ok(())
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                Self::copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}

/// Summarize migration results for presentation.
pub fn summarize(results: &[MigrateResult]) -> MigrateSummary {
    MigrateSummary::from_results(results)
}
