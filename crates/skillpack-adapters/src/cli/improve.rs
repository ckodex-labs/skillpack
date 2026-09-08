//! Assessment-driven remediation.
//!
//! `skillpack improve` applies the *mechanically-fixable* findings the
//! assessor raises — missing version, missing CHANGELOG, references the
//! SKILL.md body points at but that don't exist — then re-assesses. It never
//! fabricates content-level quality (a thin description, a missing worked
//! example): those are reported for the author, not faked. Stubbed files
//! carry a `TODO(skillpack):` marker so the gap stays honest.

use std::path::Path;

/// One remediation the improve pass considered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// True when the change was written (false under `--dry-run`).
    pub applied: bool,
    /// Human-readable description of the change.
    pub description: String,
}

/// Ensure the SKILL.md frontmatter declares a `version`. Adds `version: 0.1.0`
/// at `metadata.version` when a metadata mapping exists, else at the flat top
/// level. Returns `None` when a version already exists or SKILL.md is
/// missing/malformed.
pub fn ensure_version(skill_dir: &Path, dry_run: bool) -> Option<Fix> {
    let skill_md = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md).ok()?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }
    let mut fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim()).ok()?;

    let has_version = fm.get("version").and_then(|v| v.as_str()).is_some()
        || fm
            .get("metadata")
            .and_then(|m| m.get("version"))
            .and_then(|v| v.as_str())
            .is_some();
    if has_version {
        return None;
    }

    if !dry_run {
        let key = serde_yaml::Value::String("version".into());
        let val = serde_yaml::Value::String("0.1.0".into());
        if let Some(meta) = fm.get_mut("metadata").and_then(|m| m.as_mapping_mut()) {
            meta.insert(key, val);
        } else if let Some(root) = fm.as_mapping_mut() {
            root.insert(key, val);
        } else {
            return None;
        }
        let new_fm = serde_yaml::to_string(&fm).ok()?;
        std::fs::write(&skill_md, format!("---\n{}---\n{}", new_fm, parts[2])).ok()?;
    }
    Some(Fix {
        applied: !dry_run,
        description: "added version: 0.1.0 to frontmatter".into(),
    })
}

/// Create a `CHANGELOG.md` stub when none exists. Returns `None` if present.
pub fn ensure_changelog(skill_dir: &Path, dry_run: bool) -> Option<Fix> {
    let path = skill_dir.join("CHANGELOG.md");
    if path.exists() {
        return None;
    }
    if !dry_run {
        std::fs::write(&path, "# Changelog\n\n## Unreleased\n\n- Initial entry.\n").ok()?;
    }
    Some(Fix {
        applied: !dry_run,
        description: "created CHANGELOG.md".into(),
    })
}

/// Create `TODO(skillpack):`-marked stub files for `references/…`, `scripts/…`,
/// etc. paths the SKILL.md body links to but that do not exist — resolving the
/// compatibility checker's broken-link penalty honestly. One `Fix` per file.
pub fn stub_broken_links(skill_dir: &Path, dry_run: bool) -> Vec<Fix> {
    let skill_md = skill_dir.join("SKILL.md");
    let Ok(content) = std::fs::read_to_string(&skill_md) else {
        return Vec::new();
    };
    let body = frontmatter_body(&content);

    let mut fixes = Vec::new();
    for rel in referenced_paths(&body) {
        let target = skill_dir.join(&rel);
        if target.exists() {
            continue;
        }
        if !dry_run {
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(
                &target,
                format!(
                    "<!-- TODO(skillpack): stub for {} — fill in the referenced content -->\n",
                    rel
                ),
            );
        }
        fixes.push(Fix {
            applied: !dry_run,
            description: format!("stubbed missing reference {}", rel),
        });
    }
    fixes
}

/// The SKILL.md body (content after the frontmatter block; whole file if none).
fn frontmatter_body(content: &str) -> String {
    match content.strip_prefix("---\n") {
        Some(s) => match s.find("\n---") {
            Some(end) => s[end + 4..].to_string(),
            None => content.to_string(),
        },
        None => content.to_string(),
    }
}

/// Paths under known resource dirs referenced from the body. Mirrors the
/// compatibility checker's internal-link scan (kept local: the checker's copy
/// is private and its semantics may diverge).
fn referenced_paths(body: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn skill_with(frontmatter: &str, body: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("SKILL.md"),
            format!("---\n{}\n---\n\n{}", frontmatter, body),
        )
        .unwrap();
        dir
    }

    #[test]
    fn ensure_version_adds_flat_version_when_missing() {
        let dir = skill_with("name: t\ndescription: d", "# T\nbody");
        let fix = ensure_version(dir.path(), false).expect("should fix");
        assert!(fix.applied);
        let content = fs::read_to_string(dir.path().join("SKILL.md")).unwrap();
        assert!(content.contains("version: 0.1.0"));
    }

    #[test]
    fn ensure_version_noop_when_present() {
        let dir = skill_with("name: t\nversion: 2.0.0\ndescription: d", "# T\nbody");
        assert!(ensure_version(dir.path(), false).is_none());
    }

    #[test]
    fn ensure_version_dry_run_does_not_write() {
        let dir = skill_with("name: t\ndescription: d", "# T\nbody");
        let fix = ensure_version(dir.path(), true).expect("should report");
        assert!(!fix.applied);
        let content = fs::read_to_string(dir.path().join("SKILL.md")).unwrap();
        assert!(!content.contains("version:"));
    }

    #[test]
    fn ensure_changelog_creates_when_absent() {
        let dir = skill_with("name: t", "# T");
        let fix = ensure_changelog(dir.path(), false).expect("should fix");
        assert!(fix.applied);
        assert!(dir.path().join("CHANGELOG.md").exists());
    }

    #[test]
    fn ensure_changelog_noop_when_present() {
        let dir = skill_with("name: t", "# T");
        fs::write(dir.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
        assert!(ensure_changelog(dir.path(), false).is_none());
    }

    #[test]
    fn stub_broken_links_creates_marked_stubs() {
        let dir = skill_with(
            "name: t",
            "# T\n\nSee references/guide.md and scripts/run.sh for details.",
        );
        let fixes = stub_broken_links(dir.path(), false);
        assert_eq!(fixes.len(), 2, "expected 2 stubs, got {:?}", fixes);
        let guide = fs::read_to_string(dir.path().join("references/guide.md")).unwrap();
        assert!(guide.contains("TODO(skillpack)"));
        assert!(dir.path().join("scripts/run.sh").exists());
    }

    #[test]
    fn stub_broken_links_skips_existing() {
        let dir = skill_with("name: t", "# T\n\nSee references/guide.md.");
        fs::create_dir_all(dir.path().join("references")).unwrap();
        fs::write(dir.path().join("references/guide.md"), "real content").unwrap();
        assert!(stub_broken_links(dir.path(), false).is_empty());
        // The real file is untouched.
        assert_eq!(
            fs::read_to_string(dir.path().join("references/guide.md")).unwrap(),
            "real content"
        );
    }
}
