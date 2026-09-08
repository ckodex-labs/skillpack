//! `skillpack refresh` — align a skill's SKILL.md frontmatter with the
//! conventions the current models use to load skills into their system prompt.
//!
//! The models route on two frontmatter fields — `name` (identity) and
//! `description` (the trigger: what the skill does AND when to use it) — and
//! read the tool scope from `allowed-tools`. Older skills use legacy field
//! spellings (`allowed_tools`, `allowedTools`, `tools`) that the current
//! loaders do not recognize. Refresh **normalizes the mechanical field names**
//! and **reports** the content-level issues (weak description, non-kebab name,
//! oversized body) for the author — it never rewrites meaning.
//!
//! This is distinct from `improve`, which adds missing files (version,
//! CHANGELOG, stub links); refresh only touches the model-facing frontmatter.

use anyhow::Result;
use std::path::Path;

/// Outcome of a refresh pass.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RefreshReport {
    /// Mechanical normalizations applied (or previewed under dry-run).
    pub normalized: Vec<String>,
    /// Content-level issues the author must address manually.
    pub warnings: Vec<String>,
    /// Whether the frontmatter changed.
    pub changed: bool,
}

/// Canonicalize legacy tool-scope field names to `allowed-tools`, in place.
/// Returns the normalizations applied.
pub fn normalize_frontmatter(fm: &mut serde_yaml::Mapping) -> Vec<String> {
    let mut applied = Vec::new();
    let canon = serde_yaml::Value::String("allowed-tools".into());
    for legacy in ["allowed_tools", "allowedTools", "tools"] {
        let lk = serde_yaml::Value::String(legacy.into());
        if !fm.contains_key(&lk) {
            continue;
        }
        if fm.contains_key(&canon) {
            // Canonical form already present — drop the legacy duplicate.
            fm.remove(&lk);
            applied.push(format!("removed duplicate legacy `{}`", legacy));
        } else if let Some(v) = fm.remove(&lk) {
            fm.insert(canon.clone(), v);
            applied.push(format!("renamed `{}` → `allowed-tools`", legacy));
        }
    }
    applied
}

/// Content-level warnings about the model-routing fields.
pub fn frontmatter_warnings(fm: &serde_yaml::Mapping, body_lines: usize) -> Vec<String> {
    let mut w = Vec::new();
    let get = |k: &str| {
        fm.get(serde_yaml::Value::String(k.into()))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    };

    match get("name") {
        None => w.push("frontmatter missing `name` — the model cannot identify the skill".into()),
        Some(name) if !is_kebab(&name) => w.push(format!(
            "`name: {}` is not lowercase-hyphenated — loaders expect kebab-case",
            name
        )),
        _ => {}
    }

    match get("description") {
        None => w.push(
            "frontmatter missing `description` — the model routes on this; add what it does AND when to use it".into(),
        ),
        Some(desc) => {
            if crate::checkers::common::description_quality(&desc) < 0.8 {
                w.push(
                    "`description` is a weak router — add a trigger clause (\"Use when …\") so the model fires the skill at the right time".into(),
                );
            }
            if desc.chars().count() > 1024 {
                w.push("`description` exceeds the 1024-char budget — tighten it".into());
            }
        }
    }

    if body_lines > 500 {
        w.push(format!(
            "SKILL.md body is {} lines — over the ~500-line budget; move depth into references/ so it stays skimmable in the system prompt",
            body_lines
        ));
    }
    w
}

/// Split a SKILL.md into (frontmatter, body). The file MUST begin with a `---`
/// line, and the closing `---` must sit at a line boundary — so a `---` inside a
/// YAML value, or a horizontal rule in the markdown body, is never mistaken for
/// a delimiter. Returns `None` when there is no complete frontmatter block.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    // Tolerate a UTF-8 BOM and CRLF line endings.
    let s = content.strip_prefix('\u{feff}').unwrap_or(content);
    let s = s
        .strip_prefix("---\n")
        .or_else(|| s.strip_prefix("---\r\n"))?;
    // Closing delimiter: a newline immediately followed by `---`.
    let end = s.find("\n---")?;
    let fm = &s[..end];
    // Body is everything after the closing delimiter's own line.
    let after = &s[end + 1..];
    let body = after.find('\n').map_or("", |nl| &after[nl + 1..]);
    Some((fm, body))
}

/// Refresh a skill's SKILL.md: normalize field names (written back unless
/// dry-run) and collect content warnings. Never rewrites a file that lacks a
/// well-formed frontmatter block — it reports that as a warning instead.
pub fn refresh_skill(skill_md: &Path, dry_run: bool) -> Result<RefreshReport> {
    let content = std::fs::read_to_string(skill_md)?;
    let Some((fm_str, body)) = split_frontmatter(&content) else {
        return Ok(RefreshReport {
            normalized: Vec::new(),
            warnings: vec![
                "SKILL.md has no complete YAML frontmatter block (must open and close with a `---` line) — the model cannot route to it".into(),
            ],
            changed: false,
        });
    };
    let body_lines = body.lines().filter(|l| !l.trim().is_empty()).count();

    let parsed: serde_yaml::Value = serde_yaml::from_str(fm_str.trim())?;
    let mut fm = parsed
        .as_mapping()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("SKILL.md frontmatter is not a YAML mapping"))?;

    let normalized = normalize_frontmatter(&mut fm);
    let warnings = frontmatter_warnings(&fm, body_lines);
    let changed = !normalized.is_empty();

    if changed && !dry_run {
        let new_fm = serde_yaml::to_string(&serde_yaml::Value::Mapping(fm))?;
        std::fs::write(skill_md, format!("---\n{}---\n{}", new_fm, body))?;
    }

    Ok(RefreshReport {
        normalized,
        warnings,
        changed,
    })
}

fn is_kebab(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn map(yaml: &str) -> serde_yaml::Mapping {
        serde_yaml::from_str::<serde_yaml::Value>(yaml)
            .unwrap()
            .as_mapping()
            .unwrap()
            .clone()
    }

    #[test]
    fn normalize_renames_legacy_tool_fields() {
        let mut fm = map("name: t\nallowed_tools: [Read, Write]");
        let applied = normalize_frontmatter(&mut fm);
        assert!(applied.iter().any(|a| a.contains("allowed-tools")));
        assert!(fm.contains_key(serde_yaml::Value::String("allowed-tools".into())));
        assert!(!fm.contains_key(serde_yaml::Value::String("allowed_tools".into())));
    }

    #[test]
    fn normalize_drops_duplicate_when_canonical_present() {
        let mut fm = map("name: t\nallowed-tools: [Read]\nallowedTools: [Write]");
        let applied = normalize_frontmatter(&mut fm);
        assert!(applied.iter().any(|a| a.contains("duplicate")));
        // Canonical value preserved (not overwritten by the legacy one).
        let v = fm
            .get(serde_yaml::Value::String("allowed-tools".into()))
            .unwrap();
        assert!(format!("{:?}", v).contains("Read"));
    }

    #[test]
    fn warnings_flag_weak_description_and_non_kebab_name() {
        let fm = map("name: Not_Kebab\ndescription: Formats things.");
        let w = frontmatter_warnings(&fm, 10);
        assert!(w.iter().any(|m| m.contains("kebab")));
        assert!(w.iter().any(|m| m.contains("trigger")));
    }

    #[test]
    fn warnings_clean_for_good_frontmatter() {
        let fm = map(
            "name: good-skill\ndescription: \"Convert CSV files into JSON records with schema inference. Use when transforming tabular exports into structured JSON.\"",
        );
        let w = frontmatter_warnings(&fm, 50);
        assert!(w.is_empty(), "unexpected warnings: {:?}", w);
    }

    #[test]
    fn warnings_flag_oversized_body() {
        let fm = map("name: x\ndescription: \"Do it. Use when needed.\"");
        assert!(
            frontmatter_warnings(&fm, 600)
                .iter()
                .any(|m| m.contains("line"))
        );
    }

    #[test]
    fn refresh_writes_normalized_frontmatter() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("SKILL.md");
        fs::write(
            &p,
            "---\nname: t\ndescription: \"Do it. Use when needed.\"\nallowedTools: [Read]\n---\n# body",
        )
        .unwrap();
        let report = refresh_skill(&p, false).unwrap();
        assert!(report.changed);
        let after = fs::read_to_string(&p).unwrap();
        assert!(after.contains("allowed-tools"));
        assert!(!after.contains("allowedTools"));
        assert!(after.contains("# body"));
    }

    #[test]
    fn refresh_dry_run_does_not_write() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("SKILL.md");
        fs::write(&p, "---\nname: t\nallowed_tools: [Read]\n---\n# b").unwrap();
        let report = refresh_skill(&p, true).unwrap();
        assert!(report.changed);
        assert!(fs::read_to_string(&p).unwrap().contains("allowed_tools"));
    }

    #[test]
    fn body_only_file_with_hr_is_not_treated_as_frontmatter() {
        // CORR-1: no frontmatter, but the body has `---` horizontal rules.
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("SKILL.md");
        let original = "# Title\n\nSection A\n\n---\n\nallowedTools: junk\n\n---\n\nSection C\n";
        fs::write(&p, original).unwrap();
        let report = refresh_skill(&p, false).unwrap();
        assert!(!report.changed, "must not rewrite a body-only file");
        assert!(report.warnings.iter().any(|w| w.contains("no complete")));
        // File is untouched.
        assert_eq!(fs::read_to_string(&p).unwrap(), original);
    }

    #[test]
    fn triple_dash_inside_value_does_not_truncate() {
        // CORR-2: `---` inside a quoted value must not be seen as the delimiter.
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("SKILL.md");
        fs::write(
            &p,
            "---\nname: t\ndescription: \"a --- b\"\nallowedTools: [Read]\n---\nbody\n",
        )
        .unwrap();
        let report = refresh_skill(&p, false).unwrap();
        assert!(report.changed);
        let after = fs::read_to_string(&p).unwrap();
        // The field after the embedded `---` survived the rewrite.
        assert!(after.contains("allowed-tools"));
        assert!(after.contains("a --- b"));
        assert!(after.contains("body"));
    }

    #[test]
    fn split_frontmatter_requires_leading_delimiter() {
        assert!(split_frontmatter("no fm here\n---\nx\n").is_none());
        assert!(split_frontmatter("---\nname: x\n(no close)").is_none());
        let (fm, body) = split_frontmatter("---\nname: x\n---\nhello\n").unwrap();
        assert_eq!(fm, "name: x");
        assert_eq!(body, "hello\n");
    }
}
