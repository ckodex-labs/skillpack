//! Documentation Checker
//!
//! Assesses presence and *quality* of documentation files.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

/// Minimum content length thresholds (bytes) for each doc type.
const SKILL_MD_MIN: usize = 200;
const README_MIN: usize = 300;
const USAGE_MIN: usize = 200;
const CODE_OF_CONDUCT_MIN: usize = 100;
const CHANGELOG_MIN: usize = 100;

pub struct DocumentationChecker;

impl DimensionChecker for DocumentationChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Documentation
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            return check_agentskills_docs(reader, path);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // SKILL.md (required, up to 40 pts)
        if let Ok(content) = reader.read_file(path, "SKILL.md") {
            score += doc_score(&content, SKILL_MD_MIN, 40.0);
            if content.len() < SKILL_MD_MIN {
                issues.push(short_doc_issue("SKILL.md", SKILL_MD_MIN));
            }
            // Bonus: check for YAML frontmatter
            if !content.starts_with("---") {
                issues.push(Issue {
                    dimension: DimensionId::Documentation,
                    severity: Severity::Note,
                    message: "SKILL.md missing YAML frontmatter".to_string(),
                    file: Some("SKILL.md".into()),
                    line: Some(1),
                });
            }
        } else {
            issues.push(missing_doc_issue("SKILL.md"));
        }

        // README.md (up to 20 pts)
        if let Ok(content) = reader.read_file(path, "README.md") {
            score += doc_score(&content, README_MIN, 20.0);
            if content.len() < README_MIN {
                issues.push(short_doc_issue("README.md", README_MIN));
            }
        } else {
            issues.push(missing_doc_issue("README.md"));
        }

        // docs/USAGE.md (up to 20 pts)
        if let Ok(content) = reader.read_file(path, "docs/USAGE.md") {
            score += doc_score(&content, USAGE_MIN, 20.0);
            if content.len() < USAGE_MIN {
                issues.push(short_doc_issue("docs/USAGE.md", USAGE_MIN));
            }
        } else {
            // docs/USAGE.md is optional — no error, just a note
            issues.push(Issue {
                dimension: DimensionId::Documentation,
                severity: Severity::Note,
                message: "Optional docs/USAGE.md not found".to_string(),
                file: None,
                line: None,
            });
        }

        // CODE_OF_CONDUCT.md (up to 10 pts)
        if let Ok(content) = reader.read_file(path, "CODE_OF_CONDUCT.md") {
            score += doc_score(&content, CODE_OF_CONDUCT_MIN, 10.0);
            if content.len() < CODE_OF_CONDUCT_MIN {
                issues.push(short_doc_issue("CODE_OF_CONDUCT.md", CODE_OF_CONDUCT_MIN));
            }
        } else {
            issues.push(missing_doc_issue("CODE_OF_CONDUCT.md"));
        }

        // CHANGELOG.md (up to 10 pts)
        if let Ok(content) = reader.read_file(path, "CHANGELOG.md") {
            score += doc_score(&content, CHANGELOG_MIN, 10.0);
            if content.len() < CHANGELOG_MIN {
                issues.push(short_doc_issue("CHANGELOG.md", CHANGELOG_MIN));
            }
            // Bonus: check for version entries (e.g., "## 1.0.0" or "## [1.0.0]")
            if !content.contains("## ") {
                issues.push(Issue {
                    dimension: DimensionId::Documentation,
                    severity: Severity::Note,
                    message: "CHANGELOG.md missing version headings (## x.y.z)".to_string(),
                    file: Some("CHANGELOG.md".into()),
                    line: None,
                });
            }
        } else {
            issues.push(missing_doc_issue("CHANGELOG.md"));
        }

        (Score::dimension(score.min(100.0)), issues)
    }
}

// ---------------------------------------------------------------------------
// AgentSkills rubric
// ---------------------------------------------------------------------------

/// Documentation for a SKILL.md-convention skill lives in the body itself
/// plus progressive-disclosure resources; repo files (README, CoC) are
/// optional polish, not requirements.
fn check_agentskills_docs(reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
    let mut score = 0.0f64;
    let mut issues = Vec::new();

    let note = |message: String| Issue {
        dimension: DimensionId::Documentation,
        severity: Severity::Note,
        message,
        file: None,
        line: None,
    };

    // 1. SKILL.md body substance (up to 55 pts)
    match common::skill_body(reader, path) {
        Some(body) => {
            let len = body.chars().count();
            score += match len {
                0..=99 => 0.0,
                100..=299 => 10.0,
                300..=1199 => 20.0,
                1200..=2999 => 35.0,
                _ => 45.0,
            };
            if len < 300 {
                issues.push(Issue {
                    dimension: DimensionId::Documentation,
                    severity: Severity::Warning,
                    message: "SKILL.md body is very thin — document workflow, inputs, and outcomes"
                        .into(),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
            if common::heading_count(&body) >= 2 {
                score += 5.0;
            } else {
                issues.push(note(
                    "SKILL.md body has no section headings — structure aids skimmability".into(),
                ));
            }
            if common::code_fence_count(&body) >= 1 {
                score += 5.0;
            } else {
                issues.push(note(
                    "SKILL.md body has no code examples (``` fences)".into(),
                ));
            }
            // Discipline sections mark battle-tested skills: explicit
            // triggers, failure modes, and evidence-based verification.
            let lower = body.to_lowercase();
            if [
                "## when to use",
                "red flag",
                "rationalization",
                "## verification",
            ]
            .iter()
            .any(|s| lower.contains(s))
            {
                score += 5.0;
            } else {
                issues.push(note(
                    "consider When to Use / Red Flags / Verification sections — they encode judgment, not just workflow".into(),
                ));
            }
        }
        None => {
            issues.push(Issue {
                dimension: DimensionId::Documentation,
                severity: Severity::Error,
                message: "Missing SKILL.md".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
            return (Score::dimension(0.0), issues);
        }
    }

    // 2. Progressive disclosure (up to 25 pts)
    let dirs = common::resource_dirs_with_files(reader, path);
    if !dirs.is_empty() {
        score += 15.0;
        let body = common::skill_body(reader, path).unwrap_or_default();
        if dirs.iter().any(|d| body.contains(d)) {
            score += 10.0;
        } else {
            issues.push(note(format!(
                "resource dirs ({}) exist but SKILL.md never points to them",
                dirs.join(", ")
            )));
        }
    } else {
        issues.push(note(
            "no references/ scripts/ assets/ — consider progressive disclosure for depth".into(),
        ));
    }

    // 3. Optional repo docs (5 + 10 pts)
    if reader
        .read_file(path, "README.md")
        .map(|c| c.len() >= 200)
        .unwrap_or(false)
    {
        score += 5.0;
    }
    if reader.file_exists(path, "CHANGELOG.md") {
        score += 10.0;
    }

    (Score::dimension(score.min(100.0)), issues)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Score proportional to doc length, up to max at the threshold.
fn doc_score(content: &str, threshold: usize, max: f64) -> f64 {
    if content.len() >= threshold {
        max
    } else {
        (content.len() as f64 / threshold as f64) * max
    }
}

fn missing_doc_issue(file: &str) -> Issue {
    Issue {
        dimension: DimensionId::Documentation,
        severity: Severity::Error,
        message: format!("Missing {}", file),
        file: Some(file.into()),
        line: None,
    }
}

fn short_doc_issue(file: &str, min: usize) -> Issue {
    Issue {
        dimension: DimensionId::Documentation,
        severity: Severity::Warning,
        message: format!("{} is very short (<{} bytes)", file, min),
        file: Some(file.into()),
        line: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_score_full_at_threshold() {
        assert_eq!(doc_score("x".repeat(300).as_str(), 300, 20.0), 20.0);
    }

    #[test]
    fn doc_score_partial_below_threshold() {
        assert_eq!(doc_score("x".repeat(150).as_str(), 300, 20.0), 10.0);
    }

    #[test]
    fn doc_score_zero_when_empty() {
        assert_eq!(doc_score("", 300, 20.0), 0.0);
    }
}
