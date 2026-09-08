//! Compatibility Checker
//!
//! Assesses runtime compatibility declarations and dependency management.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct CompatibilityChecker;

impl DimensionChecker for CompatibilityChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Compatibility
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            return check_agentskills_compat(reader, path);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // 1. SKILL.md compatibility front-matter with actual declarations (50 pts)
        if let Ok(content) = reader.read_file(path, "SKILL.md") {
            if let Some(fm) = extract_frontmatter(&content) {
                if skill_md_compatibility_valid(fm) {
                    score += 50.0;
                } else {
                    issues.push(Issue {
                        dimension: DimensionId::Compatibility,
                        severity: Severity::Note,
                        message:
                            "SKILL.md compatibility front-matter missing runtime or platforms list"
                                .into(),
                        file: Some("SKILL.md".into()),
                        line: None,
                    });
                    score += 15.0;
                }
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Compatibility,
                    severity: Severity::Warning,
                    message: "SKILL.md missing YAML frontmatter".into(),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
        } else {
            issues.push(Issue {
                dimension: DimensionId::Compatibility,
                severity: Severity::Warning,
                message: "SKILL.md not found".into(),
                file: None,
                line: None,
            });
        }

        // 2. MCP server configuration with valid JSON (30 pts)
        let mut mcp_found = false;
        let mut mcp_valid = false;
        for p in ["mcp.json", "capacities/mcp/server.json"] {
            if let Ok(content) = reader.read_file(path, p) {
                mcp_found = true;
                if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                    mcp_valid = true;
                    break;
                }
            }
        }
        if mcp_valid {
            score += 30.0;
        } else if mcp_found {
            issues.push(Issue {
                dimension: DimensionId::Compatibility,
                severity: Severity::Warning,
                message: "MCP config file found but not valid JSON".into(),
                file: Some("mcp.json".into()),
                line: None,
            });
            score += 10.0;
        }

        // 3. Dependency manifest with version constraints (20 pts)
        let mut deps_found = false;
        let mut deps_pinned = false;
        if let Ok(content) = reader.read_file(path, "requirements.txt") {
            deps_found = true;
            if requirements_has_versions(&content) {
                deps_pinned = true;
            }
        } else if let Ok(content) = reader.read_file(path, "package.json") {
            deps_found = true;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if json.get("dependencies").is_some() || json.get("devDependencies").is_some() {
                    deps_pinned = true;
                }
            }
        } else if let Ok(content) = reader.read_file(path, "Cargo.toml") {
            deps_found = true;
            if content.contains("[dependencies]") {
                deps_pinned = true;
            }
        }
        if deps_pinned {
            score += 20.0;
        } else if deps_found {
            issues.push(Issue {
                dimension: DimensionId::Compatibility,
                severity: Severity::Note,
                message: "Dependency manifest found but missing version constraints".into(),
                file: None,
                line: None,
            });
            score += 5.0;
        }

        (Score::dimension(score), issues)
    }
}

/// AgentSkills rubric: compatibility means the skill loads cleanly —
/// parseable frontmatter, internal resource links that resolve, and a body
/// inside the progressive-disclosure budget (long bodies burn agent context).
fn check_agentskills_compat(reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
    let mut score = 0.0f64;
    let mut issues = Vec::new();

    let issue = |severity: Severity, message: String| Issue {
        dimension: DimensionId::Compatibility,
        severity,
        message,
        file: Some("SKILL.md".into()),
        line: None,
    };

    // 1. Frontmatter parses (30 pts)
    if common::frontmatter_map(reader, path).is_some() {
        score += 30.0;
    } else {
        issues.push(issue(
            Severity::Error,
            "SKILL.md frontmatter missing or unparseable — skill will not load".into(),
        ));
    }

    // 2. Internal resource links resolve (up to 50 pts)
    let body = common::skill_body(reader, path).unwrap_or_default();
    let refs = internal_refs(&body);
    if refs.is_empty() {
        score += 30.0; // nothing to break, but no disclosure depth either
    } else {
        let resolved = refs.iter().filter(|r| reader.file_exists(path, r)).count();
        score += 50.0 * (resolved as f64 / refs.len() as f64);
        for r in refs.iter().filter(|r| !reader.file_exists(path, r)) {
            issues.push(issue(
                Severity::Warning,
                format!("SKILL.md points to `{}` but the file does not exist", r),
            ));
        }
    }

    // 3. Disclosure budget: body under ~500 lines (20 pts)
    let lines = body.lines().count();
    if lines <= 500 {
        score += 20.0;
    } else {
        score += 5.0;
        issues.push(issue(
            Severity::Warning,
            format!(
                "SKILL.md body is {} lines — move depth into references/ (budget ~500)",
                lines
            ),
        ));
    }

    (Score::dimension(score.min(100.0)), issues)
}

/// Paths under known resource dirs referenced from the body, e.g.
/// `references/format.md` in links or inline code.
fn internal_refs(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for dir in [
        "references/",
        "scripts/",
        "assets/",
        "examples/",
        "templates/",
    ] {
        for (idx, _) in body.match_indices(dir) {
            let rest = &body[idx..];
            let end = rest
                .find(|c: char| c.is_whitespace() || "()[]`'\"<>,;".contains(c))
                .unwrap_or(rest.len());
            let candidate = rest[..end].trim_end_matches(['.', ':', '*']);
            if candidate.len() > dir.len() && !out.contains(&candidate.to_string()) {
                out.push(candidate.to_string());
            }
        }
    }
    out
}

fn extract_frontmatter(content: &str) -> Option<&str> {
    let s = content.strip_prefix("---\n")?;
    let end = s.find("\n---")?;
    Some(&s[..end])
}

fn skill_md_compatibility_valid(fm: &str) -> bool {
    let lower = fm.to_lowercase();
    (lower.contains("runtime:") || lower.contains("runtimes:"))
        && (lower.contains("platforms:") || lower.contains("platform:"))
}

fn requirements_has_versions(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && (trimmed.contains("==") || trimmed.contains(">=") || trimmed.contains("<="))
    })
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
    fn skill_md_compatibility_valid_requires_runtime_and_platforms() {
        let fm = "runtime: nodejs\nplatforms: [linux, macos]";
        assert!(skill_md_compatibility_valid(fm));
        assert!(!skill_md_compatibility_valid("runtime: nodejs"));
    }

    #[test]
    fn requirements_has_versions_detects_pinned() {
        assert!(requirements_has_versions("requests==2.31.0\n"));
        assert!(!requirements_has_versions("requests\n"));
        assert!(!requirements_has_versions("# comment\n"));
    }

    #[test]
    fn checker_full_score_with_all_manifests() {
        let reader = MockReader {
            files: [
                (
                    "SKILL.md".into(),
                    "---\nruntime: nodejs\nplatforms: [linux, macos]\n---\n# Skill\n".into(),
                ),
                ("mcp.json".into(), r#"{"name":"test"}"#.into()),
                ("requirements.txt".into(), "requests==2.31.0\n".into()),
            ]
            .into(),
        };
        let checker = CompatibilityChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 100.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn checker_partial_for_missing_platforms() {
        let reader = MockReader {
            files: [
                (
                    "SKILL.md".into(),
                    "---\nruntime: nodejs\n---\n# Skill\n".into(),
                ),
                ("mcp.json".into(), r#"{"name":"test"}"#.into()),
                ("requirements.txt".into(), "requests==2.31.0\n".into()),
            ]
            .into(),
        };
        let checker = CompatibilityChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 55.0); // 15 skill + 30 mcp + 20 deps
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("missing runtime or platforms"))
        );
    }

    #[test]
    fn checker_warns_unpinned_requirements() {
        let reader = MockReader {
            files: [
                (
                    "SKILL.md".into(),
                    "---\nruntime: nodejs\nplatforms: [linux]\n---\n# Skill\n".into(),
                ),
                ("requirements.txt".into(), "requests\nflask\n".into()),
            ]
            .into(),
        };
        let checker = CompatibilityChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 55.0);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("missing version constraints"))
        );
    }
}
