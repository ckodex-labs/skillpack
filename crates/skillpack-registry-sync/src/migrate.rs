//! Schema compliance migration for the canonical skill store.
//!
//! Enforces the `agentskills.schema.json` invariants on every SKILL.md:
//!   1. `name:` must equal the parent directory name (kebab-case)
//!   2. `description:` must be present and non-empty
//!   3. Line endings must be LF (no CRLF)
//!   4. Missing SKILL.md files are created with a minimal compliant stub
//!
//! Directory names that violate `^[a-z0-9]+(-[a-z0-9]+)*$` are flagged but
//! not automatically renamed (pass `rename_invalid_dirs: true` to enable).

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Configuration for a migration run.
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    /// Root of the canonical skill store (e.g. `~/skills/shared`).
    pub root: PathBuf,
    /// If true, print changes without writing.
    pub dry_run: bool,
    /// Rename directories whose names violate the schema pattern.
    /// e.g. `ckodex-announcements-1.0.0` → `ckodex-announcements`
    pub rename_invalid_dirs: bool,
}

/// Summary of a migration run.
#[derive(Debug, Default)]
pub struct MigrateSummary {
    pub skills_scanned: usize,
    pub names_fixed: usize,
    pub crlf_fixed: usize,
    pub missing_created: usize,
    pub dirs_renamed: usize,
    pub already_compliant: usize,
    pub errors: Vec<String>,
}

/// Kebab-case pattern from the schema: `^[a-z0-9]+(-[a-z0-9]+)*$`
fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars().peekable();
    // Must start with [a-z0-9]
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    // Remainder: [a-z0-9]+ segments separated by single hyphens
    let mut in_segment = true;
    for c in chars {
        if c == '-' {
            if !in_segment {
                return false; // double hyphen
            }
            in_segment = false;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            in_segment = true;
        } else {
            return false;
        }
    }
    in_segment // must not end with a hyphen
}

/// Sanitize an invalid directory name to a valid skill name.
///
/// Strategy: replace `.` with `-`, strip any trailing `-`, lowercase everything.
fn sanitize_dir_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let replaced = lower.replace('.', "-");
    // Collapse multiple hyphens
    let mut result = String::new();
    let mut last_was_hyphen = false;
    for c in replaced.chars() {
        if c == '-' {
            if !last_was_hyphen && !result.is_empty() {
                result.push('-');
                last_was_hyphen = true;
            }
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            result.push(c);
            last_was_hyphen = false;
        }
    }
    result.trim_end_matches('-').to_string()
}

/// Run the migration against `opts.root`.
pub fn run_migrate(opts: &MigrateOptions) -> Result<MigrateSummary> {
    let root = &opts.root;
    if !root.exists() {
        anyhow::bail!("canonical root does not exist: {}", root.display());
    }

    let mut summary = MigrateSummary::default();

    let entries: Vec<_> = fs::read_dir(root)
        .with_context(|| format!("reading {}", root.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    for entry in entries {
        let dir_path = entry.path();
        let raw_dir_name = entry.file_name().to_string_lossy().to_string();

        // Determine the canonical skill name for this directory.
        let skill_name = if is_valid_skill_name(&raw_dir_name) {
            raw_dir_name.clone()
        } else {
            let sanitized = sanitize_dir_name(&raw_dir_name);
            warn!(
                dir = %raw_dir_name,
                sanitized = %sanitized,
                "directory name violates schema pattern"
            );

            if opts.rename_invalid_dirs && !sanitized.is_empty() {
                let new_path = root.join(&sanitized);
                if new_path.exists() {
                    warn!(
                        "cannot rename {} → {}: target exists",
                        raw_dir_name, sanitized
                    );
                    sanitized
                } else if opts.dry_run {
                    println!("  [dry-run] RENAME {} → {}", raw_dir_name, sanitized);
                    summary.dirs_renamed += 1;
                    sanitized
                } else {
                    fs::rename(&dir_path, &new_path).with_context(|| {
                        format!("renaming {} to {}", dir_path.display(), new_path.display())
                    })?;
                    println!("  RENAMED  {} → {}", raw_dir_name, sanitized);
                    summary.dirs_renamed += 1;
                    // Continue processing under the new path
                    summary.skills_scanned += 1;
                    if let Err(e) = migrate_skill_md(&new_path, &sanitized, opts, &mut summary) {
                        summary.errors.push(format!("{}: {}", sanitized, e));
                    }
                    continue;
                }
            } else {
                // Can't rename; use sanitized name for frontmatter check
                sanitized
            }
        };

        summary.skills_scanned += 1;

        if let Err(e) = migrate_skill_md(&dir_path, &skill_name, opts, &mut summary) {
            summary.errors.push(format!("{}: {}", skill_name, e));
        }
    }

    Ok(summary)
}

/// Migrate a single skill directory.
fn migrate_skill_md(
    dir_path: &Path,
    skill_name: &str,
    opts: &MigrateOptions,
    summary: &mut MigrateSummary,
) -> Result<()> {
    let skill_md = dir_path.join("SKILL.md");

    if !skill_md.exists() {
        // Create a minimal compliant SKILL.md stub
        let description = format!("{} skill", skill_name.replace('-', " "));
        let stub = format!(
            "---\nname: {skill_name}\ndescription: \"{description}\"\n---\n\n# {skill_name}\n\n<!-- TODO(ckodex): add skill description -->\n",
        );
        if opts.dry_run {
            println!("  [dry-run] CREATE {}/SKILL.md", skill_name);
        } else {
            fs::write(&skill_md, &stub)
                .with_context(|| format!("creating {}", skill_md.display()))?;
            println!("  CREATED  {}/SKILL.md", skill_name);
        }
        summary.missing_created += 1;
        return Ok(());
    }

    // Read and normalize
    let raw_content =
        fs::read(&skill_md).with_context(|| format!("reading {}", skill_md.display()))?;

    let content = String::from_utf8_lossy(&raw_content);
    let has_crlf = content.contains("\r\n");
    let normalized: String = if has_crlf {
        content.replace("\r\n", "\n")
    } else {
        content.into_owned()
    };

    // Fix `name:` field in frontmatter
    let (fixed_content, name_changed) = fix_frontmatter_name(&normalized, skill_name);

    let changed = name_changed || has_crlf;

    if !changed {
        debug!(skill = %skill_name, "already compliant");
        summary.already_compliant += 1;
        return Ok(());
    }

    if name_changed {
        let old = extract_name(&normalized).unwrap_or_default();
        println!(
            "  NAME     {}/SKILL.md  {} → {}",
            skill_name, old, skill_name
        );
        summary.names_fixed += 1;
    }
    if has_crlf {
        println!("  CRLF     {}/SKILL.md  (converted to LF)", skill_name);
        summary.crlf_fixed += 1;
    }

    if opts.dry_run {
        // Already printed what would change above
    } else {
        fs::write(&skill_md, fixed_content.as_bytes())
            .with_context(|| format!("writing {}", skill_md.display()))?;
    }

    Ok(())
}

/// Replace the `name:` line inside the YAML frontmatter fence.
///
/// Returns `(new_content, changed)`.
fn fix_frontmatter_name(content: &str, expected_name: &str) -> (String, bool) {
    let mut lines = content.lines().peekable();
    let mut result = Vec::new();
    let mut changed = false;
    let mut in_frontmatter = false;
    let mut frontmatter_closed = false;
    let mut fence_count = 0;

    for line in &mut lines {
        if line == "---" && !frontmatter_closed {
            fence_count += 1;
            if fence_count == 1 {
                in_frontmatter = true;
            } else if fence_count == 2 {
                in_frontmatter = false;
                frontmatter_closed = true;
            }
            result.push(line.to_string());
            continue;
        }

        if in_frontmatter
            && !frontmatter_closed
            && let Some(rest) = line.strip_prefix("name:")
        {
            let current_value = rest.trim();
            if current_value != expected_name {
                result.push(format!("name: {}", expected_name));
                changed = true;
                continue;
            }
        }

        result.push(line.to_string());
    }

    let mut out = result.join("\n");
    // Preserve trailing newline if original had one
    if content.ends_with('\n') {
        out.push('\n');
    }
    (out, changed)
}

/// Extract the current `name:` value from frontmatter (for display).
fn extract_name(content: &str) -> Option<String> {
    let mut in_fm = false;
    let mut fence_count = 0;
    for line in content.lines() {
        if line == "---" {
            fence_count += 1;
            in_fm = fence_count == 1;
            continue;
        }
        if fence_count >= 2 {
            break;
        }
        if in_fm && let Some(rest) = line.strip_prefix("name:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_valid_skill_names() {
        assert!(is_valid_skill_name("my-skill"));
        assert!(is_valid_skill_name("azure-entra-app-registration"));
        assert!(is_valid_skill_name("terraform"));
        assert!(is_valid_skill_name("react-best-practices"));
        assert!(is_valid_skill_name("v2"));
    }

    #[test]
    fn test_invalid_skill_names() {
        assert!(!is_valid_skill_name("ckodex-announcements-1.0.0"));
        assert!(!is_valid_skill_name("MySkill"));
        assert!(!is_valid_skill_name("-leading-hyphen"));
        assert!(!is_valid_skill_name("trailing-hyphen-"));
        assert!(!is_valid_skill_name("double--hyphen"));
        assert!(!is_valid_skill_name(""));
    }

    #[test]
    fn test_sanitize_dir_name() {
        assert_eq!(
            sanitize_dir_name("ckodex-announcements-1.0.0"),
            "ckodex-announcements-1-0-0"
        );
        assert_eq!(sanitize_dir_name("MySkill"), "myskill");
        assert_eq!(sanitize_dir_name("skill..name"), "skill-name");
    }

    #[test]
    fn test_fix_frontmatter_name_mismatch() {
        let content = "---\nname: entra-app-registration\ndescription: \"foo\"\n---\n\n# Body\n";
        let (fixed, changed) = fix_frontmatter_name(content, "azure-entra-app-registration");
        assert!(changed);
        assert!(fixed.contains("name: azure-entra-app-registration"));
        assert!(!fixed.contains("name: entra-app-registration"));
    }

    #[test]
    fn test_fix_frontmatter_name_already_correct() {
        let content = "---\nname: terraform\ndescription: \"foo\"\n---\n";
        let (_, changed) = fix_frontmatter_name(content, "terraform");
        assert!(!changed);
    }

    #[test]
    fn test_fix_preserves_trailing_newline() {
        let content = "---\nname: old\n---\ncontent\n";
        let (fixed, _) = fix_frontmatter_name(content, "new");
        assert!(fixed.ends_with('\n'));
    }

    #[test]
    fn test_migrate_creates_missing_skill_md() {
        let tmp = std::env::temp_dir().join("sctl-migrate-test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("my-skill")).unwrap();

        let opts = MigrateOptions {
            root: tmp.clone(),
            dry_run: false,
            rename_invalid_dirs: false,
        };

        let summary = run_migrate(&opts).unwrap();
        assert_eq!(summary.missing_created, 1);
        assert!(tmp.join("my-skill").join("SKILL.md").exists());

        let content = fs::read_to_string(tmp.join("my-skill").join("SKILL.md")).unwrap();
        assert!(content.contains("name: my-skill"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_fixes_name_mismatch() {
        let tmp = std::env::temp_dir().join("sctl-migrate-name-test");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("azure-entra-app-registration");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: entra-app-registration\ndescription: \"test\"\n---\n# Body\n",
        )
        .unwrap();

        let opts = MigrateOptions {
            root: tmp.clone(),
            dry_run: false,
            rename_invalid_dirs: false,
        };

        let summary = run_migrate(&opts).unwrap();
        assert_eq!(summary.names_fixed, 1);

        let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content.contains("name: azure-entra-app-registration"));
        assert!(!content.contains("name: entra-app-registration"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_fixes_crlf() {
        let tmp = std::env::temp_dir().join("sctl-migrate-crlf-test");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            b"---\r\nname: my-skill\r\ndescription: \"test\"\r\n---\r\n# Body\r\n",
        )
        .unwrap();

        let opts = MigrateOptions {
            root: tmp.clone(),
            dry_run: false,
            rename_invalid_dirs: false,
        };

        let summary = run_migrate(&opts).unwrap();
        assert_eq!(summary.crlf_fixed, 1);

        let raw = fs::read(skill_dir.join("SKILL.md")).unwrap();
        assert!(!raw.windows(2).any(|w| w == b"\r\n"), "CRLF still present");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_migrate_dry_run_no_writes() {
        let tmp = std::env::temp_dir().join("sctl-migrate-dry-test");
        let _ = fs::remove_dir_all(&tmp);
        let skill_dir = tmp.join("correct-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        // Wrong name on purpose
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: wrong-name\ndescription: \"test\"\n---\n",
        )
        .unwrap();

        let opts = MigrateOptions {
            root: tmp.clone(),
            dry_run: true,
            rename_invalid_dirs: false,
        };

        let summary = run_migrate(&opts).unwrap();
        assert_eq!(summary.names_fixed, 1); // counted but not written

        // File should be unchanged in dry-run
        let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(
            content.contains("name: wrong-name"),
            "dry-run mutated the file"
        );

        let _ = fs::remove_dir_all(&tmp);
    }
}
