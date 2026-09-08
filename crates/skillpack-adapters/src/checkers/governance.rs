//! Governance Checker
//!
//! Assesses governance documentation and practices with content-quality checks.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct GovernanceChecker;

impl DimensionChecker for GovernanceChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Governance
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            // AgentSkills rubric: governance is licensing, capability scope,
            // and ownership attribution — not repo policy files.
            let mut score = 0.0f64;
            let mut issues = Vec::new();
            let fm = common::frontmatter_map(reader, path);
            let has_license = reader.file_exists(path, "LICENSE")
                || fm
                    .as_ref()
                    .map(|m| common::fm_str(m, "license").is_some())
                    .unwrap_or(false);
            if has_license {
                score += 40.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Note,
                    message: "no license declared (LICENSE file or frontmatter `license`)".into(),
                    file: None,
                    line: None,
                });
            }
            if fm
                .as_ref()
                .map(|m| {
                    common::fm_has_any(
                        m,
                        &["allowed-tools", "allowed_tools", "allowedTools", "tools"],
                    )
                })
                .unwrap_or(false)
            {
                score += 30.0;
            }
            if fm
                .as_ref()
                .map(|m| {
                    common::fm_has_any(m, &["author", "authors", "owner", "source", "metadata"])
                })
                .unwrap_or(false)
            {
                score += 30.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Note,
                    message: "no author/owner attribution in frontmatter".into(),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
            return (Score::dimension(score.min(100.0)), issues);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // 1. Threat model with required sections (25 pts)
        match reader.read_file(path, "security/threat-model.yaml") {
            Ok(content) => {
                if threat_model_valid(&content) {
                    score += 25.0;
                } else {
                    issues.push(Issue {
                        dimension: DimensionId::Governance,
                        severity: Severity::Warning,
                        message: "security/threat-model.yaml missing required sections (assets, threats, mitigations)".into(),
                        file: Some("security/threat-model.yaml".into()),
                        line: None,
                    });
                    score += 10.0;
                }
            }
            Err(_) => {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Warning,
                    message: "Missing security/threat-model.yaml".into(),
                    file: None,
                    line: None,
                });
            }
        }

        // 2. SECURITY.md with contact/disclosure section (20 pts)
        match reader.read_file(path, "security/SECURITY.md") {
            Ok(content) => {
                if security_md_valid(&content) {
                    score += 20.0;
                } else {
                    issues.push(Issue {
                        dimension: DimensionId::Governance,
                        severity: Severity::Warning,
                        message: "security/SECURITY.md is too short or missing contact/disclosure section".into(),
                        file: Some("security/SECURITY.md".into()),
                        line: None,
                    });
                    score += 5.0;
                }
            }
            Err(_) => {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Warning,
                    message: "Missing security/SECURITY.md".into(),
                    file: None,
                    line: None,
                });
            }
        }

        // 3. GOVERNANCE.md with minimum content (20 pts)
        if let Ok(content) = reader.read_file(path, "security/GOVERNANCE.md") {
            if governance_md_valid(&content) {
                score += 20.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Note,
                    message: "security/GOVERNANCE.md is too short (<100 chars)".into(),
                    file: Some("security/GOVERNANCE.md".into()),
                    line: None,
                });
                score += 5.0;
            }
        }

        // 4. CODEOWNERS with valid patterns (15 pts)
        let codeowners_paths = ["CODEOWNERS", ".github/CODEOWNERS"];
        let mut codeowners_found = false;
        let mut codeowners_valid = false;
        for p in &codeowners_paths {
            if let Ok(content) = reader.read_file(path, p) {
                codeowners_found = true;
                if codeowners_valid_content(&content) {
                    codeowners_valid = true;
                    break;
                }
            }
        }
        if codeowners_valid {
            score += 15.0;
        } else if codeowners_found {
            issues.push(Issue {
                dimension: DimensionId::Governance,
                severity: Severity::Note,
                message: "CODEOWNERS file found but contains no valid ownership patterns".into(),
                file: Some("CODEOWNERS".into()),
                line: None,
            });
            score += 5.0;
        }

        // 5. CODE_OF_CONDUCT.md with minimum content (10 pts)
        if let Ok(content) = reader.read_file(path, "CODE_OF_CONDUCT.md") {
            if content.trim().len() >= 200 {
                score += 10.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Note,
                    message: "CODE_OF_CONDUCT.md is too short (<200 chars)".into(),
                    file: Some("CODE_OF_CONDUCT.md".into()),
                    line: None,
                });
                score += 3.0;
            }
        }

        // 6. LICENSE with minimum content (10 pts)
        if let Ok(content) = reader.read_file(path, "LICENSE") {
            if content.trim().len() >= 100 {
                score += 10.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Governance,
                    severity: Severity::Note,
                    message: "LICENSE file is too short (<100 chars)".into(),
                    file: Some("LICENSE".into()),
                    line: None,
                });
                score += 3.0;
            }
        }

        (Score::dimension(score), issues)
    }
}

fn threat_model_valid(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("assets:") && lower.contains("threats:") && lower.contains("mitigations:")
}

fn security_md_valid(content: &str) -> bool {
    let lower = content.to_lowercase();
    content.trim().len() >= 200
        && (lower.contains("contact") || lower.contains("disclosure") || lower.contains("report"))
}

fn governance_md_valid(content: &str) -> bool {
    content.trim().len() >= 100
}

fn codeowners_valid_content(content: &str) -> bool {
    content
        .lines()
        .any(|line| !line.trim().is_empty() && !line.starts_with('#') && line.contains('@'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::{SkillIdentity, SkillReaderError};
    use std::collections::HashMap;
    use std::path::Path;

    struct MockReader {
        files: HashMap<String, String>,
    }

    impl SkillReader for MockReader {
        fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
            Ok(SkillIdentity::new("test", "1.0.0", ""))
        }
        fn file_exists(&self, _path: &Path, relative: &str) -> bool {
            if self.files.contains_key(relative) {
                return true;
            }
            let prefix = format!("{}/", relative);
            self.files.keys().any(|k| k.starts_with(&prefix))
        }
        fn read_file(&self, _path: &Path, relative: &str) -> Result<String, SkillReaderError> {
            self.files
                .get(relative)
                .cloned()
                .ok_or_else(|| SkillReaderError::NotFound(relative.into()))
        }
        fn list_files(&self, _path: &Path, _pattern: &str) -> Vec<String> {
            self.files.keys().cloned().collect()
        }
    }

    #[test]
    fn threat_model_valid_requires_all_sections() {
        assert!(threat_model_valid(
            "assets:\n  - db\nthreats:\n  - xss\nmitigations:\n  - input validation"
        ));
        assert!(!threat_model_valid("assets:\n  - db"));
    }

    #[test]
    fn security_md_valid_requires_length_and_contact() {
        let valid = "# Security\n\nIf you discover a security issue, please contact us at security@example.com. We take all reports seriously and will respond within 48 hours.\n\n## Disclosure Policy\nWe follow responsible disclosure.";
        assert!(security_md_valid(valid));
        let short = "# Security\n";
        assert!(!security_md_valid(short));
    }

    #[test]
    fn codeowners_valid_content_requires_at_symbol() {
        assert!(codeowners_valid_content("* @alice\n"));
        assert!(!codeowners_valid_content("# comment\n"));
        assert!(!codeowners_valid_content("\n\n"));
    }

    #[test]
    fn checker_full_score_for_complete_governance() {
        let reader = MockReader {
            files: [
                (
                    "security/threat-model.yaml".into(),
                    "assets:\nthreats:\nmitigations:\n".into(),
                ),
                (
                    "security/SECURITY.md".into(),
                    "# Security\nContact: security@example.com\nDisclosure: responsible\n"
                        .repeat(5)
                        .into(),
                ),
                (
                    "security/GOVERNANCE.md".into(),
                    "# Governance\nThis project is governed by the core team. Decisions require consensus among maintainers before any major changes are approved.\n".into(),
                ),
                (".github/CODEOWNERS".into(), "* @team\n".into()),
                (
                    "CODE_OF_CONDUCT.md".into(),
                    "# Code of Conduct\nBe excellent to each other.\n"
                        .repeat(10)
                        .into(),
                ),
                (
                    "LICENSE".into(),
                    "MIT License\nCopyright (c) 2024\n".repeat(5).into(),
                ),
            ]
            .into(),
        };
        let checker = GovernanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 100.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn checker_warns_incomplete_threat_model() {
        let reader = MockReader {
            files: [(
                "security/threat-model.yaml".into(),
                "assets:\n  - db\n".into(),
            )]
            .into(),
        };
        let checker = GovernanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 10.0);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("missing required sections"))
        );
    }

    #[test]
    fn checker_warns_empty_codeowners() {
        let reader = MockReader {
            files: [(".github/CODEOWNERS".into(), "# just a comment\n".into())].into(),
        };
        let checker = GovernanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 5.0);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("no valid ownership patterns"))
        );
    }
}
