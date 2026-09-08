//! Per-agent skill rendering.
//!
//! Each `AgentIntegrationType` has a distinct render strategy:
//!   Symlink     → create a symlink from agent dir to canonical path
//!   IndexFile   → write a catalog `.mdc` or `index.json` aggregating all skills
//!   RulesDir    → copy SKILL.md as `<skill-name>.md` into the rules directory
//!   SingleFile  → fold all skill descriptions into one `copilot-instructions.md`

use anyhow::Result;
use skillpack_domain::{AgentConfig, AgentIntegrationType, CanonicalSkill};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Render canonical skills into the agent-specific directory format.
pub fn render_for_agent(
    agent: &AgentConfig,
    skills: &[CanonicalSkill],
    canonical_root: &Path,
) -> Result<usize> {
    let Some(dir) = agent.resolved_dir() else {
        return Ok(0);
    };
    fs::create_dir_all(&dir)?;

    match agent.integration_type {
        AgentIntegrationType::Symlink => render_symlinks(skills, canonical_root, &dir),
        AgentIntegrationType::IndexFile => {
            render_index_file(&agent.name, skills, canonical_root, &dir)
        }
        AgentIntegrationType::RulesDir => render_rules_dir(skills, &dir),
        AgentIntegrationType::SingleFile => render_single_file(&agent.name, skills, &dir),
    }
}

// --- Symlink ---

fn render_symlinks(
    skills: &[CanonicalSkill],
    canonical_root: &Path,
    agent_dir: &Path,
) -> Result<usize> {
    let mut count = 0;
    for skill in skills {
        let source = canonical_root.join(&skill.name);
        let target = agent_dir.join(&skill.name);
        replace_symlink(&source, &target)?;
        count += 1;
    }
    Ok(count)
}

fn replace_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() || is_symlink(target) {
        if target.is_dir() && !is_symlink(target) {
            fs::remove_dir_all(target)?;
        } else {
            fs::remove_file(target)?;
        }
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target)?;
    Ok(())
}

// --- IndexFile (Cursor .mdc catalog, Windsurf index.json) ---

fn render_index_file(
    agent_name: &str,
    skills: &[CanonicalSkill],
    canonical_root: &Path,
    agent_dir: &Path,
) -> Result<usize> {
    match agent_name {
        "Cursor" => render_cursor_mdc(skills, canonical_root, agent_dir),
        _ => render_generic_index_json(skills, canonical_root, agent_dir),
    }
}

fn render_cursor_mdc(
    skills: &[CanonicalSkill],
    canonical_root: &Path,
    agent_dir: &Path,
) -> Result<usize> {
    let mut table = String::from("| Skill | Path | Description |\n|---|---|---|\n");
    for skill in skills {
        let description = read_description(&skill.path)
            .unwrap_or_default()
            .chars()
            .take(120)
            .collect::<String>()
            .replace('|', "\\|");
        table.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            skill.name,
            canonical_root.join(&skill.name).display(),
            description,
        ));
    }
    let content = format!(
        "---\ndescription: \"Personal skill catalog (canonical store: {}). Auto-generated, do not hand-edit.\"\nglobs: [\"**/*\"]\nalwaysApply: false\n---\n\n# Available Skills\n\nCanonical store: `{}`\n\nQuery via CodeGraph MCP (if configured) or read the SKILL.md directly at the path below.\n\n{}\n",
        canonical_root.display(),
        canonical_root.display(),
        table,
    );
    let mdc_path = agent_dir.join("skillpack-catalog.mdc");
    fs::write(&mdc_path, content)?;
    debug!(path = %mdc_path.display(), count = skills.len(), "wrote Cursor catalog");
    Ok(skills.len())
}

fn render_generic_index_json(
    skills: &[CanonicalSkill],
    canonical_root: &Path,
    agent_dir: &Path,
) -> Result<usize> {
    let index = serde_json::json!({
        "version": "1.0",
        "source": canonical_root.to_string_lossy(),
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "skills": skills.iter().map(|s| serde_json::json!({
            "name": s.name,
            "path": s.path.to_string_lossy(),
            "has_skill_md": s.has_skill_md,
        })).collect::<Vec<_>>(),
    });
    let path = agent_dir.join("skillpack-index.json");
    fs::write(&path, serde_json::to_string_pretty(&index)?)?;
    Ok(skills.len())
}

// --- RulesDir (Roo Code, Cline, Continue, Augment) ---

fn render_rules_dir(skills: &[CanonicalSkill], agent_dir: &Path) -> Result<usize> {
    let mut count = 0;
    for skill in skills {
        let skill_md = skill.path.join("SKILL.md");
        let dest = agent_dir.join(format!("{}.md", skill.name));
        if skill_md.exists() {
            fs::copy(&skill_md, &dest)?;
        } else {
            fs::write(&dest, format!("# {}\n\nSee canonical store.\n", skill.name))?;
        }
        count += 1;
    }
    Ok(count)
}

// --- SingleFile (GitHub Copilot) ---

fn render_single_file(
    agent_name: &str,
    skills: &[CanonicalSkill],
    agent_dir: &Path,
) -> Result<usize> {
    let filename = match agent_name {
        "GitHub Copilot" => "copilot-instructions.md",
        _ => "instructions.md",
    };
    let mut content = format!(
        "# Skill Catalog — {}\n\nAuto-generated by sctl. Do not hand-edit.\n\n",
        agent_name
    );
    for skill in skills {
        let desc = read_description(&skill.path).unwrap_or_default();
        content.push_str(&format!("## {}\n\n{}\n\n", skill.name, desc));
    }
    fs::write(agent_dir.join(filename), content)?;
    Ok(skills.len())
}

// --- Helpers ---

fn read_description(skill_path: &Path) -> Option<String> {
    let content = fs::read_to_string(skill_path.join("SKILL.md")).ok()?;
    // Extract `description:` from frontmatter
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("description:") {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

#[cfg(unix)]
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_symlink(path: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_rules_dir_creates_md_files() {
        let tmp = std::env::temp_dir().join("sctl-render-test");
        let _ = fs::create_dir_all(&tmp);

        let skill_path = tmp.join("my-skill");
        fs::create_dir_all(&skill_path).unwrap();
        fs::write(
            skill_path.join("SKILL.md"),
            "---\nname: my-skill\n---\n# My Skill\nContent.",
        )
        .unwrap();

        let skills = vec![CanonicalSkill {
            name: "my-skill".to_string(),
            path: skill_path,
            has_skill_md: true,
            content_hash: None,
            source_agent: None,
        }];

        let rules_dir = tmp.join("rules");
        fs::create_dir_all(&rules_dir).unwrap();
        let count = render_rules_dir(&skills, &rules_dir).unwrap();
        assert_eq!(count, 1);
        assert!(rules_dir.join("my-skill.md").exists());

        let _ = fs::remove_dir_all(&tmp);
    }
}
