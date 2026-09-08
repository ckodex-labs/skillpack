//! Skill Migration Domain Model
//!
//! Types for migrating physical (non-symlinked) skills from agent
//! directories into the canonical store.

use std::path::PathBuf;

/// Outcome of migrating a single physical skill to the canonical store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrateOutcome {
    /// Skill was copied into the canonical store and original replaced with symlink.
    Migrated,
    /// Physical copy removed and replaced with a symlink to the existing canonical version.
    ReplacedWithLink,
    /// Source was already a symlink — nothing to do.
    SkippedAlreadyLinked,
    /// Source path or name matched an IP boundary pattern — refused.
    SkippedIPViolation(String),
    /// Migration failed for an unexpected reason.
    Failed(String),
}

/// Result of a single skill migration attempt.
#[derive(Debug, Clone)]
pub struct MigrateResult {
    pub agent_name: String,
    pub skill_name: String,
    pub source_path: PathBuf,
    pub outcome: MigrateOutcome,
}

impl MigrateResult {
    pub fn new(
        agent_name: impl Into<String>,
        skill_name: impl Into<String>,
        source_path: impl Into<PathBuf>,
        outcome: MigrateOutcome,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            skill_name: skill_name.into(),
            source_path: source_path.into(),
            outcome,
        }
    }
}

/// Summary statistics for a batch migration.
#[derive(Debug, Clone, Default)]
pub struct MigrateSummary {
    pub migrated: usize,
    pub replaced: usize,
    pub skipped_linked: usize,
    pub skipped_ip: usize,
    pub failed: usize,
    pub total: usize,
}

impl MigrateSummary {
    pub fn from_results(results: &[MigrateResult]) -> Self {
        let mut summary = MigrateSummary {
            total: results.len(),
            ..Default::default()
        };
        for r in results {
            match r.outcome {
                MigrateOutcome::Migrated => summary.migrated += 1,
                MigrateOutcome::ReplacedWithLink => summary.replaced += 1,
                MigrateOutcome::SkippedAlreadyLinked => summary.skipped_linked += 1,
                MigrateOutcome::SkippedIPViolation(_) => summary.skipped_ip += 1,
                MigrateOutcome::Failed(_) => summary.failed += 1,
            }
        }
        summary
    }
}

/// Request to migrate all physical skills into the canonical store.
#[derive(Debug, Clone)]
pub struct MigrateAllRequest {
    pub canonical_root: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_summary_counts() {
        let results = vec![
            MigrateResult::new("Claude", "skill-a", "/a", MigrateOutcome::Migrated),
            MigrateResult::new("Claude", "skill-b", "/b", MigrateOutcome::ReplacedWithLink),
            MigrateResult::new(
                "Cursor",
                "skill-c",
                "/c",
                MigrateOutcome::SkippedAlreadyLinked,
            ),
            MigrateResult::new(
                "Cursor",
                "skill-d",
                "/d",
                MigrateOutcome::SkippedIPViolation("thales".to_string()),
            ),
            MigrateResult::new(
                "Windsurf",
                "skill-e",
                "/e",
                MigrateOutcome::Failed("io error".to_string()),
            ),
        ];

        let summary = MigrateSummary::from_results(&results);
        assert_eq!(summary.migrated, 1);
        assert_eq!(summary.replaced, 1);
        assert_eq!(summary.skipped_linked, 1);
        assert_eq!(summary.skipped_ip, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total, 5);
    }
}
