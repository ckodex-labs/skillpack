//! SkillPack CLI Configuration
//!
//! Persisted at `~/.config/skillpack/config.toml`

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default registry endpoint
const DEFAULT_REGISTRY: &str = "https://registry.skillpack.dev";

/// Default scope
const DEFAULT_SCOPE: &str = "both";

/// Top-level config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillpackConfig {
    /// Default registry URL
    pub registry: String,

    /// Default scope: global, project, or both
    pub scope: String,

    /// Default agent tool filter (e.g. windsurf, claude, cursor)
    pub default_tool: Option<String>,

    /// Output format preference: human or json
    pub output_format: String,

    /// Disable ANSI colors
    pub no_color: bool,

    /// Known agent skill directories (name -> path)
    pub agent_paths: std::collections::HashMap<String, String>,

    /// Canonical store root
    pub canonical_root: Option<String>,
}

impl Default for SkillpackConfig {
    fn default() -> Self {
        let mut agent_paths = std::collections::HashMap::new();

        // Populate well-known agent paths
        if let Some(home) = dirs::home_dir() {
            let candidates = [
                ("windsurf", ".codeium/windsurf/memories/skills"),
                ("claude", ".claude/skills"),
                ("cursor", ".cursor/skills"),
                ("copilot", ".config/github-copilot/skills"),
                ("agents", ".config/agents/skills"),
            ];
            for (name, rel) in candidates {
                let p = home.join(rel);
                if p.exists() {
                    agent_paths.insert(name.to_string(), p.to_string_lossy().to_string());
                }
            }
        }

        Self {
            registry: DEFAULT_REGISTRY.to_string(),
            scope: DEFAULT_SCOPE.to_string(),
            default_tool: None,
            output_format: "human".to_string(),
            no_color: false,
            agent_paths,
            canonical_root: None,
        }
    }
}

/// Return the config file path: `~/.config/skillpack/config.toml`
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("skillpack")
        .join("config.toml")
}

/// Load config from disk, returning defaults if missing or unreadable.
pub fn load_config() -> Result<SkillpackConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(SkillpackConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let cfg: SkillpackConfig =
        toml::from_str(&content).unwrap_or_else(|_| SkillpackConfig::default());
    Ok(cfg)
}

/// Save config to disk (creates parent dirs as needed).
pub fn save_config(cfg: &SkillpackConfig) -> Result<PathBuf> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Collect all known skill paths from config + well-known locations,
/// filtered by optional scope ("global", "project", "both") and tool name.
pub fn collect_skill_paths(
    cfg: &SkillpackConfig,
    scope: &str,
    tool_filter: Option<&str>,
) -> Vec<(String, PathBuf)> {
    let mut paths: Vec<(String, PathBuf)> = Vec::new();

    // Always include configured agent paths
    for (tool, dir) in &cfg.agent_paths {
        if let Some(filter) = tool_filter
            && tool != filter
        {
            continue;
        }
        let p = PathBuf::from(dir);
        if p.exists() {
            paths.push((tool.clone(), p));
        }
    }

    // Add project-local scope
    if scope == "project" || scope == "both" {
        let local = PathBuf::from(".");
        if local.join("SKILL.md").exists() || local.join("skill.cnsb.json").exists() {
            paths.push(("local".to_string(), local));
        }
    }

    // Add canonical store root if set
    if scope == "global" || scope == "both" {
        if let Some(root) = &cfg.canonical_root {
            let p = PathBuf::from(root);
            if p.exists() {
                paths.push(("canonical".to_string(), p));
            }
        } else if let Some(home) = dirs::home_dir() {
            let p = home.join("Skills/shared");
            if p.exists() {
                paths.push(("canonical".to_string(), p));
            }
        }
    }

    paths
}
