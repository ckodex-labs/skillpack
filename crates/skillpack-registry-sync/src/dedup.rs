//! Skill deduplication engine.
//!
//! Deduplication key: SHA-256 of the SKILL.md body after stripping YAML frontmatter.
//! Two skills with identical body hashes are considered duplicates regardless of name.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// A deduplicated skill slot: canonical name + set of aliases (duplicate names).
#[derive(Debug, Clone)]
pub struct DeduplicatedSkill {
    /// Canonical name (first seen wins).
    pub canonical_name: String,
    /// Aliases pointing at the same content (slug → original path).
    pub aliases: Vec<(String, std::path::PathBuf)>,
    /// SHA-256 hex digest of the normalized SKILL.md body.
    pub content_hash: String,
    /// Filesystem path of the canonical copy.
    pub canonical_path: std::path::PathBuf,
}

/// Result of a dedup pass.
#[derive(Debug, Default)]
pub struct DedupReport {
    pub unique: Vec<DeduplicatedSkill>,
    pub duplicate_count: usize,
    pub total_scanned: usize,
}

/// Strip YAML frontmatter from skill content and return the body.
fn strip_frontmatter(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return trimmed;
    }
    // Find the closing `---`
    let after_open = &trimmed[3..];
    if let Some(close) = after_open.find("\n---") {
        after_open[close + 4..].trim_start()
    } else {
        trimmed
    }
}

/// Compute SHA-256 of the normalized skill body.
pub fn content_hash(skill_md_path: &Path) -> anyhow::Result<String> {
    let raw = std::fs::read_to_string(skill_md_path)?;
    let body = strip_frontmatter(&raw);
    let digest = Sha256::digest(body.as_bytes());
    Ok(hex::encode(digest))
}

/// Run deduplication across a list of (name, path) pairs.
///
/// Pairs are expected to be sorted so stable ordering determines canonical choice.
pub fn dedup(skills: Vec<(String, std::path::PathBuf)>) -> DedupReport {
    let total_scanned = skills.len();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut unique: Vec<DeduplicatedSkill> = Vec::new();

    for (name, path) in skills {
        let skill_md = path.join("SKILL.md");
        let hash = if skill_md.exists() {
            content_hash(&skill_md).unwrap_or_else(|_| slug_hash(&name))
        } else {
            slug_hash(&name)
        };

        if let Some(idx) = seen.get(&hash) {
            unique[*idx].aliases.push((name, path));
        } else {
            seen.insert(hash.clone(), unique.len());
            unique.push(DeduplicatedSkill {
                canonical_name: name,
                aliases: Vec::new(),
                content_hash: hash,
                canonical_path: path,
            });
        }
    }

    let duplicate_count = total_scanned - unique.len();
    DedupReport {
        unique,
        duplicate_count,
        total_scanned,
    }
}

/// Fallback hash when no SKILL.md is present: hash the normalized slug.
fn slug_hash(name: &str) -> String {
    let normalized = name.to_lowercase().replace([' ', '_'], "-");
    hex::encode(Sha256::digest(normalized.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter_removes_yaml_block() {
        let input = "---\nname: foo\n---\n# Body\nContent here.";
        let body = strip_frontmatter(input);
        assert_eq!(body, "# Body\nContent here.");
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let input = "# Plain\nNo frontmatter.";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn test_dedup_collapses_identical_slugs() {
        // Two skills with the same slug-hash (no SKILL.md) collapse to one.
        let skills = vec![
            (
                "my-skill".to_string(),
                std::path::PathBuf::from("/a/my-skill"),
            ),
            (
                "my-skill".to_string(),
                std::path::PathBuf::from("/b/my-skill"),
            ),
        ];
        let report = dedup(skills);
        assert_eq!(report.unique.len(), 1);
        assert_eq!(report.duplicate_count, 1);
        assert_eq!(report.unique[0].aliases.len(), 1);
    }

    #[test]
    fn test_dedup_keeps_distinct_skills() {
        let skills = vec![
            ("alpha".to_string(), std::path::PathBuf::from("/a/alpha")),
            ("beta".to_string(), std::path::PathBuf::from("/b/beta")),
        ];
        let report = dedup(skills);
        assert_eq!(report.unique.len(), 2);
        assert_eq!(report.duplicate_count, 0);
    }
}
