//! Evals & HITL Checker
//!
//! Assesses eval harness and human-in-the-loop gate configuration.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct EvalsHitlChecker;

impl DimensionChecker for EvalsHitlChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::EvalsHitl
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            // AgentSkills rubric: evaluability = eval fixtures, worked
            // examples an agent can imitate, and crisp trigger conditions.
            let mut score = 0.0f64;
            let mut issues = Vec::new();
            let has_evals = !reader.list_files(path, r"(evals|tests)/.+").is_empty();
            if has_evals {
                score += 40.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::EvalsHitl,
                    severity: Severity::Note,
                    message: "no evals/ or tests/ fixtures — skill behavior is unverifiable".into(),
                    file: None,
                    line: None,
                });
            }
            let body = common::skill_body(reader, path).unwrap_or_default();
            let lower = body.to_lowercase();
            if common::code_fence_count(&body) >= 2 || lower.contains("## example") {
                score += 40.0;
            } else if common::code_fence_count(&body) == 1 {
                score += 20.0;
            }
            let desc = common::frontmatter_map(reader, path)
                .and_then(|m| common::fm_str(&m, "description"))
                .unwrap_or_default();
            if common::description_quality(&desc) >= 0.8 {
                score += 20.0; // trigger-quality description is itself evaluable
            }
            return (Score::dimension(score.min(100.0)), issues);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // evals/runner.yaml (40 pts)
        if reader.file_exists(path, "evals/runner.yaml") {
            score += 40.0;
            if let Ok(content) = reader.read_file(path, "evals/runner.yaml") {
                if content.contains("hitl:") || content.contains("approver_roles:") {
                    score += 30.0;
                } else {
                    issues.push(Issue {
                        dimension: DimensionId::EvalsHitl,
                        severity: Severity::Note,
                        message: "evals/runner.yaml missing HITL gate config".to_string(),
                        file: Some("evals/runner.yaml".into()),
                        line: None,
                    });
                }
            }
        } else {
            issues.push(Issue {
                dimension: DimensionId::EvalsHitl,
                severity: Severity::Warning,
                message: "Missing evals/runner.yaml".to_string(),
                file: None,
                line: None,
            });
        }

        // CI eval workflow (30 pts)
        if reader.file_exists(path, ".github/workflows/eval.yml")
            || reader.file_exists(path, ".github/workflows/hitl.yml")
        {
            score += 30.0;
        }

        (Score::dimension(score), issues)
    }
}
