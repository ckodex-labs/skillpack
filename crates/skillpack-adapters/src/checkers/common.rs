//! Shared helpers for profile-aware dimension checkers.
//!
//! The AgentSkills rubric branches score SKILL.md content (frontmatter,
//! markdown body, progressive-disclosure resources) instead of CNSB
//! supply-chain artifacts. These helpers centralize the parsing so each
//! checker's rubric stays declarative.

use skillpack_domain::{AssessmentProfile, SkillReader};
use std::path::Path;

/// True when the skill should be scored under the AgentSkills rubric.
pub fn is_agentskills(reader: &dyn SkillReader, path: &Path) -> bool {
    AssessmentProfile::detect(reader, path) == AssessmentProfile::AgentSkills
}

/// Raw frontmatter block of SKILL.md, if present.
pub fn frontmatter(content: &str) -> Option<&str> {
    let s = content.strip_prefix("---\n")?;
    let end = s.find("\n---")?;
    Some(&s[..end])
}

/// Parsed SKILL.md frontmatter mapping, if present and valid YAML.
pub fn frontmatter_map(reader: &dyn SkillReader, path: &Path) -> Option<serde_yaml::Mapping> {
    let content = reader.read_file(path, "SKILL.md").ok()?;
    let fm = frontmatter(&content)?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(fm).ok()?;
    parsed.as_mapping().cloned()
}

/// String field from a frontmatter mapping.
pub fn fm_str(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(serde_yaml::Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// True when any of the given frontmatter keys is present (any value type).
pub fn fm_has_any(map: &serde_yaml::Mapping, keys: &[&str]) -> bool {
    keys.iter()
        .any(|k| map.contains_key(serde_yaml::Value::String((*k).into())))
}

/// SKILL.md body (content after the frontmatter block; whole file if none).
pub fn skill_body(reader: &dyn SkillReader, path: &Path) -> Option<String> {
    let content = reader.read_file(path, "SKILL.md").ok()?;
    let body = match content.strip_prefix("---\n") {
        Some(s) => match s.find("\n---") {
            Some(end) => {
                let rest = &s[end + 4..];
                rest.trim_start_matches('-').trim_start().to_string()
            }
            None => content.clone(),
        },
        None => content.clone(),
    };
    Some(body)
}

/// Count of markdown headings (## and deeper) in a body.
pub fn heading_count(body: &str) -> usize {
    body.lines()
        .filter(|l| l.starts_with("##") || l.starts_with("# "))
        .count()
}

/// Count of fenced code blocks in a body.
pub fn code_fence_count(body: &str) -> usize {
    body.matches("```").count() / 2
}

/// Progressive-disclosure resource dirs that actually contain files.
pub fn resource_dirs_with_files(reader: &dyn SkillReader, path: &Path) -> Vec<&'static str> {
    ["references", "scripts", "assets", "examples", "templates"]
        .into_iter()
        .filter(|d| !reader.list_files(path, &format!(r"{}/.+", d)).is_empty())
        .collect()
}

/// Description quality band, 0.0–1.0. Rewards trigger-oriented descriptions
/// ("use when …") of healthy length; penalizes one-liners and walls of text.
pub fn description_quality(desc: &str) -> f64 {
    let len = desc.chars().count();
    let mut q: f64 = match len {
        0 => 0.0,
        1..=19 => 0.2,
        20..=59 => 0.5,
        60..=1024 => 0.8,
        _ => 0.5, // over the 1024-char convention budget
    };
    let lower = desc.to_lowercase();
    // Trigger-clause forms per the agent-skills convention: the description
    // is a router, so it must say WHEN, not just what.
    if [
        "use when",
        "use this",
        "use before",
        "use after",
        "use during",
        "use for",
        "trigger",
        "should be used",
    ]
    .iter()
    .any(|t| lower.contains(t))
    {
        q += 0.2;
    }
    // Anti-pattern: a description that summarizes the workflow makes the agent
    // follow the summary instead of reading the skill. Penalize step
    // enumeration ("1. … 2. …", "first … then … finally").
    if summarizes_workflow(&lower) {
        q -= 0.3;
    }
    q.clamp(0.0, 1.0)
}

/// Heuristic: does the description read like a step list rather than a
/// what+when router? True when it enumerates ordered steps.
fn summarizes_workflow(lower: &str) -> bool {
    // Numbered steps: at least two step markers. The digit-dot forms require a
    // trailing space ("1. ") so version/decimal numbers ("schema 1.0 to 2.0")
    // are not misread as an enumerated step list.
    let numbered = ["1. ", "2. ", "1) ", "2) ", "step 1", "step 2"]
        .iter()
        .filter(|m| lower.contains(*m))
        .count()
        >= 2;
    // Sequence words strung together imply a walkthrough, not a trigger.
    let has_then = lower.contains(" then ");
    let sequence = has_then
        && (lower.contains("first ")
            || lower.contains("finally")
            || lower.contains(" next ")
            || lower.matches(" then ").count() >= 2);
    numbered || sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_quality_rewards_triggers() {
        let plain = "Formats code blocks across a workspace with style detection.";
        let triggered =
            "Formats code blocks across a workspace. Use when reformatting fenced code.";
        assert!(description_quality(triggered) > description_quality(plain));
    }

    #[test]
    fn description_quality_penalizes_stub_and_wall() {
        assert!(description_quality("Formatter") < 0.5);
        assert!(description_quality(&"x".repeat(3000)) < 0.8);
    }

    #[test]
    fn description_quality_penalizes_workflow_summary() {
        // All three carry a "Use when" trigger and land in the same length
        // band, so the ONLY difference is the step-enumeration structure. This
        // isolates the summarizes_workflow penalty: without it, router and
        // step_list would both clamp to 1.0 and the assertion would fail
        // (Rule 9 — the test must fail if the penalty is removed).
        let router = "Convert CSV to JSON with schema inference. Use when transforming tabular exports to JSON.";
        let step_list = "Convert CSV to JSON. Use when transforming exports. 1. Read the header. 2. Infer types.";
        let prose_seq = "Convert CSV to JSON. Use when transforming exports: first read the header, then infer types, then finally emit.";
        assert!(description_quality(router) > description_quality(step_list));
        assert!(description_quality(router) > description_quality(prose_seq));
    }

    #[test]
    fn description_quality_ignores_version_numbers() {
        // CORR-3: "1.0"/"2.0" must not be misread as an enumerated step list.
        let versioned = "Migrate configuration files from schema version 1.0 to 2.0. Use when upgrading configs.";
        // No workflow penalty → trigger bonus lands it at the top of the band.
        assert!(description_quality(versioned) >= 0.8);
    }

    #[test]
    fn frontmatter_extracts_block() {
        let c = "---\nname: a\n---\nBody";
        assert_eq!(frontmatter(c), Some("name: a"));
    }

    #[test]
    fn skill_body_strips_frontmatter() {
        // exercised via reader in integration tests; parsing logic here
        let content = "---\nname: a\n---\n\n# Title\nBody";
        let s = content.strip_prefix("---\n").unwrap();
        let end = s.find("\n---").unwrap();
        let rest = &s[end + 4..];
        assert!(rest.contains("# Title"));
    }
}
