//! Security Checker
//!
//! Validates license, ASC patterns, secrets, and dangerous lifecycle commands.

use crate::checkers::common;
use crate::checkers::secrets::SecretScanner;
use skillpack_domain::{
    AscPattern, DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader,
};
use std::path::Path;

pub struct SecurityChecker;

impl DimensionChecker for SecurityChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Security
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // 1. License declaration (25 pts)
        if license_in_frontmatter(reader, path) || reader.file_exists(path, "LICENSE") {
            score += 25.0;
        } else {
            issues.push(Issue {
                dimension: DimensionId::Security,
                severity: Severity::Warning,
                message: "No license found in SKILL.md frontmatter or LICENSE file".into(),
                file: None,
                line: None,
            });
        }

        // 2. ASC pattern in CNSB metadata or skills (25 pts)
        let cnsb_files = reader.list_files(path, r"\.cnsb\.json$");
        let mut asc_ok = false;
        for f in &cnsb_files {
            if let Ok(content) = reader.read_file(path, f)
                && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
            {
                // Check legacy metadata.annotations.asc string
                if let Some(asc) = json
                    .pointer("/metadata/annotations/asc")
                    .and_then(serde_json::Value::as_str)
                {
                    if AscPattern::is_valid(asc) {
                        asc_ok = true;
                    } else {
                        issues.push(Issue {
                            dimension: DimensionId::Security,
                            severity: Severity::Error,
                            message: format!("Invalid ASC pattern: {}", asc),
                            file: Some(f.into()),
                            line: None,
                        });
                    }
                }
                // Check skills[].asc array (canonical CNSB v1/v2 schema)
                if let Some(skills) = json.pointer("/skills").and_then(|v| v.as_array()) {
                    for skill in skills {
                        if let Some(asc_array) = skill.get("asc").and_then(|v| v.as_array()) {
                            for asc in asc_array {
                                if let Some(asc_str) = asc.as_str()
                                    && AscPattern::is_valid(asc_str)
                                {
                                    asc_ok = true;
                                }
                            }
                        }
                        if let Some(asc) = skill.get("asc").and_then(|v| v.as_str()) {
                            if AscPattern::is_valid(asc) {
                                asc_ok = true;
                            } else {
                                issues.push(Issue {
                                    dimension: DimensionId::Security,
                                    severity: Severity::Error,
                                    message: format!("Invalid ASC pattern: {}", asc),
                                    file: Some(f.into()),
                                    line: None,
                                });
                            }
                        }
                    }
                }
                // Check spec.skills[].asc (legacy nested schema)
                if let Some(spec) = json.get("spec")
                    && let Some(skills) = spec.get("skills").and_then(|v| v.as_array())
                {
                    for skill in skills {
                        if let Some(asc_array) = skill.get("asc").and_then(|v| v.as_array()) {
                            for asc in asc_array {
                                if let Some(asc_str) = asc.as_str()
                                    && AscPattern::is_valid(asc_str)
                                {
                                    asc_ok = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if asc_ok {
            score += 25.0;
        } else if common::is_agentskills(reader, path) {
            // AgentSkills rubric: ASC patterns live in CNSB manifests, which
            // these skills do not carry. Capability discipline stands in:
            // declared tool scope + a clean (or absent) scripts/ surface.
            let declares_tools = common::frontmatter_map(reader, path)
                .map(|m| {
                    common::fm_has_any(
                        &m,
                        &["allowed-tools", "allowed_tools", "allowedTools", "tools"],
                    )
                })
                .unwrap_or(false);
            if declares_tools {
                score += 15.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Security,
                    severity: Severity::Note,
                    message: "no tool scope declared (allowed-tools) — skill runs with ambient capability".into(),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
            let script_files = reader.list_files(path, r"scripts/.+");
            let mut scripts_dirty = false;
            for f in &script_files {
                if let Ok(content) = reader.read_file(path, f)
                    && ((content.contains("curl") && content.contains("| sh"))
                        || (content.contains("curl") && content.contains("| bash"))
                        || content.contains("rm -rf /"))
                {
                    scripts_dirty = true;
                    issues.push(Issue {
                        dimension: DimensionId::Security,
                        severity: Severity::Error,
                        message: format!("dangerous pattern in {}", f),
                        file: Some(f.clone()),
                        line: None,
                    });
                }
            }
            if !scripts_dirty {
                score += 10.0;
            }
        }

        // 3. Secrets scan (25 pts)
        let scanner = SecretScanner::new();
        let mut secrets_found = false;
        for f in reader.list_files(path, r"\.(rs|py|js|ts|yaml|yml|json|md|tf|sh)$") {
            if let Ok(content) = reader.read_file(path, &f) {
                let hits = scanner.scan(&content);
                if !hits.is_empty() {
                    secrets_found = true;
                    for hit in hits {
                        issues.push(Issue {
                            dimension: DimensionId::Security,
                            severity: Severity::Error,
                            message: format!(
                                "{} secret detected on line {}: {}",
                                hit.kind, hit.line, hit.matched
                            ),
                            file: Some(f.clone()),
                            line: Some(hit.line as u32),
                        });
                    }
                }
            }
        }
        if !secrets_found {
            score += 25.0;
        }

        // 4. No dangerous lifecycle commands (25 pts)
        if lifecycle_has_dangerous_commands(reader, path, &mut issues) {
            issues.push(Issue {
                dimension: DimensionId::Security,
                severity: Severity::Error,
                message: "Dangerous lifecycle commands detected".into(),
                file: None,
                line: None,
            });
        } else {
            score += 25.0;
        }

        (Score::dimension(score), issues)
    }
}

fn license_in_frontmatter(reader: &dyn SkillReader, path: &Path) -> bool {
    let Ok(content) = reader.read_file(path, "SKILL.md") else {
        return false;
    };
    let Some(fm) = extract_frontmatter(&content) else {
        return false;
    };
    fm.contains("license:")
}

fn extract_frontmatter(content: &str) -> Option<&str> {
    let s = content.strip_prefix("---\n")?;
    let end = s.find("\n---")?;
    Some(&s[..end])
}

fn lifecycle_has_dangerous_commands(
    reader: &dyn SkillReader,
    path: &Path,
    issues: &mut Vec<Issue>,
) -> bool {
    let mut found = false;
    let cnsb_files = reader.list_files(path, r"\.cnsb\.json$");
    for f in &cnsb_files {
        let Ok(content) = reader.read_file(path, f) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        if let Some(lifecycle) = json.get("lifecycle") {
            for cmd in [
                "install",
                "uninstall",
                "upgrade",
                "verify",
                "pack",
                "unpack",
            ] {
                if let Some(val) = lifecycle.get(cmd).and_then(serde_json::Value::as_str) {
                    if val.contains("curl") && val.contains("|") {
                        issues.push(Issue {
                            dimension: DimensionId::Security,
                            severity: Severity::Error,
                            message: format!("{} contains 'curl | sh' pipe", cmd),
                            file: Some(f.into()),
                            line: None,
                        });
                        found = true;
                    }
                    if val.contains("eval $(") || val.contains("`") && val.contains("$") {
                        issues.push(Issue {
                            dimension: DimensionId::Security,
                            severity: Severity::Error,
                            message: format!("{} contains dangerous eval", cmd),
                            file: Some(f.into()),
                            line: None,
                        });
                        found = true;
                    }
                    if val.contains("rm -rf /") {
                        issues.push(Issue {
                            dimension: DimensionId::Security,
                            severity: Severity::Error,
                            message: format!("{} contains 'rm -rf /'", cmd),
                            file: Some(f.into()),
                            line: None,
                        });
                        found = true;
                    }
                }
            }
        }
    }
    found
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
            Ok(SkillIdentity {
                name: "test".into(),
                version: "1.0.0".into(),
                path: "".into(),
            })
        }
        fn file_exists(&self, _path: &Path, relative: &str) -> bool {
            self.files.contains_key(relative)
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
    fn extract_frontmatter_parses_yaml_block() {
        let content = "---\nlicense: MIT\n---\n# Skill\n";
        let fm = extract_frontmatter(content);
        assert_eq!(fm, Some("license: MIT"));
    }

    #[test]
    fn extract_frontmatter_returns_none_without_prefix() {
        let content = "# Skill\nNo frontmatter here.\n";
        assert_eq!(extract_frontmatter(content), None);
    }

    #[test]
    fn extract_frontmatter_returns_none_with_empty_yaml() {
        let content = "---\n---\n# Skill\n";
        let fm = extract_frontmatter(content);
        // After stripping prefix, the remaining string starts with "---" directly;
        // find("\n---") does not match, so None is returned.
        assert_eq!(fm, None);
    }

    #[test]
    fn license_in_frontmatter_detects_license_field() {
        let reader = MockReader {
            files: [(
                "SKILL.md".into(),
                "---\nlicense: Apache-2.0\n---\n# My Skill\n".into(),
            )]
            .into(),
        };
        assert!(license_in_frontmatter(&reader, Path::new(".")));
    }

    #[test]
    fn license_in_frontmatter_missing_skill_md() {
        let reader = MockReader {
            files: HashMap::new(),
        };
        assert!(!license_in_frontmatter(&reader, Path::new(".")));
    }

    #[test]
    fn lifecycle_has_dangerous_commands_detects_curl_pipe() {
        let reader = MockReader {
            files: [(
                "test.cnsb.json".into(),
                r#"{"lifecycle":{"install":"curl https://example.com | sh"}}"#.into(),
            )]
            .into(),
        };
        let mut issues = Vec::new();
        assert!(lifecycle_has_dangerous_commands(
            &reader,
            Path::new("."),
            &mut issues
        ));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("curl | sh"));
    }

    #[test]
    fn lifecycle_has_dangerous_commands_detects_rm_rf_root() {
        let reader = MockReader {
            files: [(
                "test.cnsb.json".into(),
                r#"{"lifecycle":{"uninstall":"rm -rf /"}}"#.into(),
            )]
            .into(),
        };
        let mut issues = Vec::new();
        assert!(lifecycle_has_dangerous_commands(
            &reader,
            Path::new("."),
            &mut issues
        ));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("rm -rf /"));
    }

    #[test]
    fn lifecycle_has_dangerous_commands_allows_safe_commands() {
        let reader = MockReader {
            files: [(
                "test.cnsb.json".into(),
                r#"{"lifecycle":{"install":"echo hello"}}"#.into(),
            )]
            .into(),
        };
        let mut issues = Vec::new();
        assert!(!lifecycle_has_dangerous_commands(
            &reader,
            Path::new("."),
            &mut issues
        ));
        assert!(issues.is_empty());
    }
}
