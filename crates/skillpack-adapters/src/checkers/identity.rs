//! Identity & Manifest Checker
//!
//! Validates SKILL.md frontmatter and CNSB metadata.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct IdentityChecker;

impl DimensionChecker for IdentityChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::IdentityAndManifest
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            return self.check_agentskills_profile(reader, path);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();
        let agentskills_ok = self.check_agentskills(reader, path, &mut issues);
        let cnsb_ok = self.check_cnsb(reader, path, &mut issues);

        if agentskills_ok {
            score += 50.0;
        }
        if cnsb_ok {
            score += 50.0;
        }
        if !agentskills_ok && !cnsb_ok {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Error,
                message: "Neither valid SKILL.md frontmatter nor CNSB metadata found".into(),
                file: None,
                line: None,
            });
        }

        (Score::dimension(score), issues)
    }
}

impl IdentityChecker {
    /// AgentSkills rubric: identity lives entirely in SKILL.md frontmatter.
    /// `name` + `description` are the contract; `version` is a bonus, not a
    /// requirement (the agentskills convention does not define one).
    fn check_agentskills_profile(
        &self,
        reader: &dyn SkillReader,
        path: &Path,
    ) -> (Score, Vec<Issue>) {
        let mut score = 0.0f64;
        let mut issues = Vec::new();

        let issue = |severity: Severity, message: &str| Issue {
            dimension: DimensionId::IdentityAndManifest,
            severity,
            message: message.into(),
            file: Some("SKILL.md".into()),
            line: None,
        };

        let Some(fm) = common::frontmatter_map(reader, path) else {
            issues.push(issue(
                Severity::Error,
                "SKILL.md missing or has no parseable YAML frontmatter",
            ));
            return (Score::dimension(0.0), issues);
        };

        match common::fm_str(&fm, "name") {
            Some(name) => {
                score += 30.0;
                if valid_skill_name(&name) {
                    score += 10.0;
                } else {
                    issues.push(issue(
                        Severity::Warning,
                        "frontmatter `name` should be lowercase-hyphenated, <=64 chars",
                    ));
                }
            }
            None => issues.push(issue(Severity::Error, "frontmatter missing `name`")),
        }

        match common::fm_str(&fm, "description") {
            Some(desc) => {
                score += 30.0;
                let q = common::description_quality(&desc);
                score += q * 20.0;
                if q < 0.6 {
                    issues.push(issue(
                        Severity::Warning,
                        "description is thin — state what the skill does AND when to use it (\"Use when …\")",
                    ));
                }
            }
            None => issues.push(issue(Severity::Error, "frontmatter missing `description`")),
        }

        if common::fm_str(&fm, "version").is_some() {
            score += 10.0;
        } else {
            issues.push(issue(
                Severity::Note,
                "optional `version` not declared (helps update tracking)",
            ));
        }

        (Score::dimension(score.min(100.0)), issues)
    }

    fn check_agentskills(
        &self,
        reader: &dyn SkillReader,
        path: &Path,
        issues: &mut Vec<Issue>,
    ) -> bool {
        let Ok(content) = reader.read_file(path, "SKILL.md") else {
            return false;
        };
        let Some(fm) = extract_frontmatter(&content) else {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Error,
                message: "SKILL.md missing or malformed YAML frontmatter".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
            return false;
        };
        let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(fm) else {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Error,
                message: "SKILL.md frontmatter not parseable YAML".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
            return false;
        };
        let m = match parsed.as_mapping() {
            Some(m) => m,
            None => return false,
        };
        let has_name = m
            .get(serde_yaml::Value::String("name".into()))
            .and_then(|v| v.as_str())
            .is_some();
        let has_version = m
            .get(serde_yaml::Value::String("version".into()))
            .and_then(|v| v.as_str())
            .is_some();
        let has_desc = m
            .get(serde_yaml::Value::String("description".into()))
            .and_then(|v| v.as_str())
            .is_some();
        if !has_name {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Error,
                message: "SKILL.md frontmatter missing `name`".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
        }
        if !has_version {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Error,
                message: "SKILL.md frontmatter missing `version`".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
        }
        if !has_desc {
            issues.push(Issue {
                dimension: DimensionId::IdentityAndManifest,
                severity: Severity::Warning,
                message: "SKILL.md frontmatter missing `description`".into(),
                file: Some("SKILL.md".into()),
                line: None,
            });
        }
        has_name && has_version
    }

    fn check_cnsb(&self, reader: &dyn SkillReader, path: &Path, issues: &mut Vec<Issue>) -> bool {
        let cnsb_files = reader.list_files(path, r"\.cnsb\.json$");
        if cnsb_files.is_empty() {
            return false;
        }
        let mut all_ok = true;
        for f in &cnsb_files {
            let Ok(content) = reader.read_file(path, f) else {
                all_ok = false;
                continue;
            };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
                issues.push(Issue {
                    dimension: DimensionId::IdentityAndManifest,
                    severity: Severity::Error,
                    message: format!("{} not parseable JSON", f),
                    file: Some(f.into()),
                    line: None,
                });
                all_ok = false;
                continue;
            };
            for k in ["apiVersion", "kind"] {
                if json.get(k).and_then(serde_json::Value::as_str).is_none() {
                    issues.push(Issue {
                        dimension: DimensionId::IdentityAndManifest,
                        severity: Severity::Error,
                        message: format!("{} missing `{}`", f, k),
                        file: Some(f.into()),
                        line: None,
                    });
                    all_ok = false;
                }
            }
            for k in ["name", "urn", "version"] {
                if json
                    .pointer(&format!("/metadata/{}", k))
                    .and_then(serde_json::Value::as_str)
                    .is_none()
                {
                    issues.push(Issue {
                        dimension: DimensionId::IdentityAndManifest,
                        severity: Severity::Error,
                        message: format!("{} missing metadata.{}", f, k),
                        file: Some(f.into()),
                        line: None,
                    });
                    all_ok = false;
                }
            }
        }
        all_ok
    }
}

fn valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

fn extract_frontmatter(content: &str) -> Option<&str> {
    let s = content.strip_prefix("---\n")?;
    let end = s.find("\n---")?;
    Some(&s[..end])
}
