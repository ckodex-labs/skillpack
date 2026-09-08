//! Lifecycle Checker
//!
//! Assesses whether bundle defines standard lifecycle hooks with
//! *content-validated* bodies (non-empty commands/scripts/recipes).

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct LifecycleChecker;

impl DimensionChecker for LifecycleChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Lifecycle
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            // AgentSkills rubric: install/uninstall is copy-in/copy-out, so
            // lifecycle reduces to update traceability.
            let mut score = 0.0f64;
            let mut issues = Vec::new();
            let fm = common::frontmatter_map(reader, path);
            if fm.is_some() {
                score += 30.0; // loadable skill; lifecycle largely N/A by design
            }
            if fm
                .as_ref()
                .map(|m| common::fm_str(m, "version").is_some())
                .unwrap_or(false)
            {
                score += 35.0;
            }
            if reader.file_exists(path, "CHANGELOG.md") {
                score += 35.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Lifecycle,
                    severity: Severity::Note,
                    message: "no CHANGELOG.md — updates are untraceable".into(),
                    file: None,
                    line: None,
                });
            }
            return (Score::dimension(score.min(100.0)), issues);
        }

        let mut score = 0u32;
        let mut issues = Vec::new();

        // Check for CNSB bundle with lifecycle
        let bundles = reader.list_files(path, r"\.cnsb\.json$");
        if bundles.is_empty() {
            // Check for Makefile with lifecycle targets that have recipes
            if reader.file_exists(path, "Makefile") {
                let makefile = reader.read_file(path, "Makefile").unwrap_or_default();
                if makefile_has_target_with_recipe(&makefile, "install") {
                    score += 15;
                } else if makefile.contains("install:") {
                    issues.push(empty_recipe_issue("Makefile", "install"));
                }
                if makefile_has_target_with_recipe(&makefile, "uninstall") {
                    score += 15;
                } else if makefile.contains("uninstall:") {
                    issues.push(empty_recipe_issue("Makefile", "uninstall"));
                }
                if makefile_has_target_with_recipe(&makefile, "upgrade") {
                    score += 10;
                } else if makefile.contains("upgrade:") {
                    issues.push(empty_recipe_issue("Makefile", "upgrade"));
                }
                if makefile_has_target_with_recipe(&makefile, "verify") {
                    score += 10;
                } else if makefile.contains("verify:") {
                    issues.push(empty_recipe_issue("Makefile", "verify"));
                }
                if makefile_has_target_with_recipe(&makefile, "pack")
                    || makefile_has_target_with_recipe(&makefile, "build")
                {
                    score += 10;
                } else if makefile.contains("pack:") || makefile.contains("build:") {
                    issues.push(empty_recipe_issue("Makefile", "pack/build"));
                }
            }

            // Check for package.json scripts with non-empty values
            if reader.file_exists(path, "package.json") {
                let pkg = reader.read_file(path, "package.json").unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&pkg) {
                    if json_script_has_body(&json, "install")
                        || json_script_has_body(&json, "postinstall")
                    {
                        score += 15;
                    } else if pkg.contains("\"install\"") || pkg.contains("\"postinstall\"") {
                        issues.push(empty_script_issue("package.json", "install"));
                    }
                    if json_script_has_body(&json, "build") {
                        score += 10;
                    } else if pkg.contains("\"build\"") {
                        issues.push(empty_script_issue("package.json", "build"));
                    }
                    if json_script_has_body(&json, "test") {
                        score += 10;
                    } else if pkg.contains("\"test\"") {
                        issues.push(empty_script_issue("package.json", "test"));
                    }
                } else {
                    // Unparseable package.json — fall back to string checks
                    if pkg.contains("\"install\"") || pkg.contains("\"postinstall\"") {
                        score += 15;
                    }
                    if pkg.contains("\"build\"") {
                        score += 10;
                    }
                    if pkg.contains("\"test\"") {
                        score += 10;
                    }
                }
            }

            // Check for Cargo.toml (Rust lifecycle)
            if reader.file_exists(path, "Cargo.toml") {
                score += 20; // Cargo provides built-in lifecycle
            }

            if score == 0 {
                issues.push(Issue {
                    dimension: DimensionId::Lifecycle,
                    severity: Severity::Warning,
                    message: "No lifecycle operations defined. Add Makefile, package.json scripts, or CNSB lifecycle hooks.".to_string(),
                    file: None,
                    line: None,
                });
            }
        } else {
            // Parse CNSB bundle for lifecycle section with body validation
            for bundle in &bundles {
                if let Ok(content) = reader.read_file(path, bundle)
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
                {
                    let lifecycle = json.get("lifecycle").or_else(|| {
                        // Fallback: check spec.skills[0].lifecycle (legacy nested schema)
                        json.get("spec")
                            .and_then(|s| s.get("skills"))
                            .and_then(|s| s.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|skill| skill.get("lifecycle"))
                    });
                    if let Some(lifecycle) = lifecycle {
                        // Required hooks (20 points each)
                        if hook_has_body(lifecycle, "install") {
                            score += 20;
                        } else if lifecycle.get("install").is_some() {
                            issues.push(empty_hook_issue(bundle, "install"));
                        }
                        if hook_has_body(lifecycle, "uninstall") {
                            score += 20;
                        } else if lifecycle.get("uninstall").is_some() {
                            issues.push(empty_hook_issue(bundle, "uninstall"));
                        }
                        // Recommended hooks (15 points each)
                        if hook_has_body(lifecycle, "upgrade") {
                            score += 15;
                        } else if lifecycle.get("upgrade").is_some() {
                            issues.push(empty_hook_issue(bundle, "upgrade"));
                        }
                        if hook_has_body(lifecycle, "verify") {
                            score += 15;
                        } else if lifecycle.get("verify").is_some() {
                            issues.push(empty_hook_issue(bundle, "verify"));
                        }
                        // Nice-to-have hooks (10 points each)
                        if hook_has_body(lifecycle, "pack") {
                            score += 10;
                        } else if lifecycle.get("pack").is_some() {
                            issues.push(empty_hook_issue(bundle, "pack"));
                        }
                        if hook_has_body(lifecycle, "unpack") {
                            score += 10;
                        } else if lifecycle.get("unpack").is_some() {
                            issues.push(empty_hook_issue(bundle, "unpack"));
                        }
                        if hook_has_body(lifecycle, "preInstall") {
                            score += 5;
                        } else if lifecycle.get("preInstall").is_some() {
                            issues.push(empty_hook_issue(bundle, "preInstall"));
                        }
                        if hook_has_body(lifecycle, "postInstall") {
                            score += 5;
                        } else if lifecycle.get("postInstall").is_some() {
                            issues.push(empty_hook_issue(bundle, "postInstall"));
                        }
                    } else {
                        issues.push(Issue {
                            dimension: DimensionId::Lifecycle,
                            severity: Severity::Warning,
                            message: format!("CNSB bundle {} missing lifecycle section", bundle),
                            file: Some(bundle.clone()),
                            line: None,
                        });
                        score += 20; // Base score for having CNSB at all
                    }
                }
            }
        }

        // Cap CNSB sub-score at 90 to preserve headroom for lock-file bonus
        let mut score = score.min(90);

        // Bonus: Check for lock file (reproducibility)
        if reader.file_exists(path, "skill.lock") {
            score += 10;
        } else if reader.file_exists(path, "package-lock.json")
            || reader.file_exists(path, "Cargo.lock")
        {
            score += 5; // Partial credit for ecosystem lock files
        } else {
            issues.push(Issue {
                dimension: DimensionId::Lifecycle,
                severity: Severity::Note,
                message: "No skill.lock file. Run 'skillpack lock' for reproducible installs."
                    .to_string(),
                file: None,
                line: None,
            });
        }

        // Bonus: Check for .well-known discovery
        if reader.file_exists(path, ".well-known/skills.json") {
            score += 5;
        }

        (Score::dimension(score.min(100) as f64), issues)
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Validate that a CNSB lifecycle hook has a non-empty command or script body.
fn hook_has_body(lifecycle: &serde_json::Value, key: &str) -> bool {
    lifecycle
        .get(key)
        .and_then(|v| v.get("command").or_else(|| v.get("script")))
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Validate that a Makefile target has at least one non-empty recipe line.
fn makefile_has_target_with_recipe(makefile: &str, target: &str) -> bool {
    let target_line = format!("{}:", target);
    let mut in_recipe = false;
    for line in makefile.lines() {
        let trimmed = line.trim_start();
        if line.trim() == target_line {
            in_recipe = true;
            continue;
        }
        if in_recipe {
            // Next non-empty, non-comment, non-target line must be a recipe (tab-indented)
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if line.starts_with('\t') && !trimmed.is_empty() {
                return true;
            }
            // Another target or variable definition — recipe block ended
            if trimmed.ends_with(':') || trimmed.contains('=') {
                return false;
            }
        }
    }
    false
}

/// Validate that a package.json script has a non-empty string value.
fn json_script_has_body(json: &serde_json::Value, script: &str) -> bool {
    json.get("scripts")
        .and_then(|s| s.get(script))
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

fn empty_hook_issue(bundle: &str, hook: &str) -> Issue {
    Issue {
        dimension: DimensionId::Lifecycle,
        severity: Severity::Warning,
        message: format!(
            "CNSB bundle {} lifecycle hook '{}' declared but has empty command/script",
            bundle, hook
        ),
        file: Some(bundle.to_string()),
        line: None,
    }
}

fn empty_recipe_issue(file: &str, target: &str) -> Issue {
    Issue {
        dimension: DimensionId::Lifecycle,
        severity: Severity::Warning,
        message: format!(
            "Makefile target '{}' declared in {} but has empty recipe",
            target, file
        ),
        file: Some(file.to_string()),
        line: None,
    }
}

fn empty_script_issue(file: &str, script: &str) -> Issue {
    Issue {
        dimension: DimensionId::Lifecycle,
        severity: Severity::Warning,
        message: format!(
            "package.json script '{}' declared in {} but has empty value",
            script, file
        ),
        file: Some(file.to_string()),
        line: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_has_body_detects_nonempty_command() {
        let lifecycle = serde_json::json!({
            "install": {"command": "echo hello"}
        });
        assert!(hook_has_body(&lifecycle, "install"));
    }

    #[test]
    fn hook_has_body_rejects_empty_command() {
        let lifecycle = serde_json::json!({
            "install": {"command": ""}
        });
        assert!(!hook_has_body(&lifecycle, "install"));
    }

    #[test]
    fn hook_has_body_rejects_missing_key() {
        let lifecycle = serde_json::json!({"install": {}});
        assert!(!hook_has_body(&lifecycle, "install"));
    }

    #[test]
    fn hook_has_body_accepts_script_instead_of_command() {
        let lifecycle = serde_json::json!({
            "verify": {"script": "test -f file"}
        });
        assert!(hook_has_body(&lifecycle, "verify"));
    }

    #[test]
    fn makefile_target_with_recipe_detects_nonempty_recipe() {
        let makefile = "install:\n\techo hello\n";
        assert!(makefile_has_target_with_recipe(makefile, "install"));
    }

    #[test]
    fn makefile_target_with_recipe_rejects_empty_recipe() {
        let makefile = "install:\n\nverify:\n\techo ok\n";
        assert!(!makefile_has_target_with_recipe(makefile, "install"));
    }

    #[test]
    fn makefile_target_with_recipe_rejects_no_target() {
        let makefile = "all:\n\techo hello\n";
        assert!(!makefile_has_target_with_recipe(makefile, "install"));
    }

    #[test]
    fn json_script_has_body_detects_nonempty_script() {
        let json = serde_json::json!({"scripts": {"build": "tsc"}});
        assert!(json_script_has_body(&json, "build"));
    }

    #[test]
    fn json_script_has_body_rejects_empty_script() {
        let json = serde_json::json!({"scripts": {"build": ""}});
        assert!(!json_script_has_body(&json, "build"));
    }

    #[test]
    fn json_script_has_body_rejects_missing_script() {
        let json = serde_json::json!({"scripts": {}});
        assert!(!json_script_has_body(&json, "build"));
    }
}
