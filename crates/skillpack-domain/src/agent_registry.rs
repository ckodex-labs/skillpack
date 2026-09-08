//! Agent registry: built-in and custom agent configurations.

use crate::canonical_store::{AgentConfig, AgentIntegrationType};
use std::path::PathBuf;

/// Registry of all supported agents.
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: Vec<AgentConfig>,
}

impl AgentRegistry {
    /// Create the default registry: the 19 built-in AI coding assistants plus
    /// any custom harnesses declared in the `SKILLPACK_CUSTOM_AGENTS` env var
    /// (`name1=/path/to/skills,name2=/other/skills`). The env var is the
    /// zero-recompile onboarding path for a new harness, and because it is
    /// inherited by the server process, CLI and server agree on the fleet.
    pub fn default_registry() -> Self {
        let mut registry = Self::builtin_registry();
        if let Ok(spec) = std::env::var("SKILLPACK_CUSTOM_AGENTS") {
            registry = registry.with_custom_agents(parse_custom_agents(&spec));
        }
        registry
    }

    /// The built-in registry with all 19 supported AI coding assistants, with
    /// no environment influence — the stable base other layers extend.
    pub fn builtin_registry() -> Self {
        let agents = vec![
            // --- Symlink agents: skills dir mirrors canonical store via symlinks ---
            AgentConfig {
                name: "Agents (generic)".to_string(),
                default_dir: PathBuf::from("~/.agents/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "AGENTS_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Claude Code".to_string(),
                default_dir: PathBuf::from("~/.claude/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "CLAUDE_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Codex".to_string(),
                default_dir: PathBuf::from("~/.codex/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "CODEX_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "OpenClaw".to_string(),
                default_dir: PathBuf::from("~/.openclaw/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "OPENCLAW_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Aider".to_string(),
                default_dir: PathBuf::from("~/.aider/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "AIDER_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "OpenCode".to_string(),
                default_dir: PathBuf::from("~/.config/opencode/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "OPENCODE_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Zed".to_string(),
                default_dir: PathBuf::from("~/.config/zed/prompt_overrides"),
                fallback_dirs: vec![PathBuf::from("~/.zed/rules")],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "ZED_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Amp".to_string(),
                default_dir: PathBuf::from("~/.amp/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "AMP_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Gemini CLI".to_string(),
                default_dir: PathBuf::from("~/.gemini/skills"),
                fallback_dirs: vec![PathBuf::from("~/.gemini/config/skills")],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "GEMINI_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Google Antigravity".to_string(),
                default_dir: PathBuf::from("~/.antigravity/skills"),
                fallback_dirs: vec![PathBuf::from("~/.antigravitycli/skills")],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "ANTIGRAVITY_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Pi".to_string(),
                default_dir: PathBuf::from("~/.pi/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "PI_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Hermes".to_string(),
                default_dir: PathBuf::from("~/.hermes/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "HERMES_SKILLS_DIR".to_string(),
            },
            // VS Code (Copilot) has no native "skills" dir — it reads prompt /
            // instruction files from Code/User/prompts. The RulesDir render writes
            // per-skill files there; a full skills→.instructions.md bridge is a
            // follow-up. Point at the real location so it is at least correct.
            AgentConfig {
                name: "VS Code".to_string(),
                default_dir: PathBuf::from("~/Library/Application Support/Code/User/prompts"),
                fallback_dirs: vec![PathBuf::from("~/.config/Code/User/prompts")],
                integration_type: AgentIntegrationType::RulesDir,
                env_override: "VSCODE_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Crush".to_string(),
                default_dir: PathBuf::from("~/.config/crush/skills"),
                fallback_dirs: vec![PathBuf::from("~/.crush/skills")],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "CRUSH_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Slate".to_string(),
                default_dir: PathBuf::from("~/.slate/skills"),
                fallback_dirs: vec![PathBuf::from("~/.config/slate/skills")],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "SLATE_SKILLS_DIR".to_string(),
            },
            // Devin is a cloud harness with no local skills directory: it consumes
            // skills from a remote repository. The env override lets an operator
            // point at a locally-synced mirror; otherwise share to Devin's repo
            // via `scripts/skill-share.sh`. Kept here so the fleet is complete.
            AgentConfig {
                name: "Devin".to_string(),
                default_dir: PathBuf::from("~/.devin/skills"),
                fallback_dirs: vec![],
                integration_type: AgentIntegrationType::Symlink,
                env_override: "DEVIN_SKILLS_DIR".to_string(),
            },
            // --- IndexFile agents: skills aggregated into catalog files ---
            AgentConfig {
                name: "Cursor".to_string(),
                default_dir: PathBuf::from("~/.cursor/rules"),
                fallback_dirs: vec![PathBuf::from("~/.cursor/skills")],
                integration_type: AgentIntegrationType::IndexFile,
                env_override: "CURSOR_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Windsurf".to_string(),
                default_dir: PathBuf::from("~/.windsurf/rules"),
                fallback_dirs: vec![PathBuf::from("~/.codeium/windsurf/skills")],
                integration_type: AgentIntegrationType::IndexFile,
                env_override: "WINDSURF_SKILLS_DIR".to_string(),
            },
            // --- RulesDir agents: individual rule files in a flat directory ---
            AgentConfig {
                name: "Roo Code".to_string(),
                default_dir: PathBuf::from("~/.roo/rules"),
                fallback_dirs: vec![PathBuf::from("~/.roo/skills")],
                integration_type: AgentIntegrationType::RulesDir,
                env_override: "ROO_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Cline".to_string(),
                default_dir: PathBuf::from("~/Documents/Cline/Rules"),
                fallback_dirs: vec![PathBuf::from("~/.cline/rules")],
                integration_type: AgentIntegrationType::RulesDir,
                env_override: "CLINE_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Continue".to_string(),
                default_dir: PathBuf::from("~/.continue/rules"),
                fallback_dirs: vec![PathBuf::from("~/.continue/skills")],
                integration_type: AgentIntegrationType::RulesDir,
                env_override: "CONTINUE_SKILLS_DIR".to_string(),
            },
            AgentConfig {
                name: "Augment".to_string(),
                default_dir: PathBuf::from("~/.augment/rules"),
                fallback_dirs: vec![PathBuf::from("~/.augment/skills")],
                integration_type: AgentIntegrationType::RulesDir,
                env_override: "AUGMENT_SKILLS_DIR".to_string(),
            },
            // --- SingleFile agents: all skills folded into one instructions file ---
            AgentConfig {
                name: "GitHub Copilot".to_string(),
                default_dir: PathBuf::from("~/.github/instructions"),
                fallback_dirs: vec![PathBuf::from("~/.github/copilot/skills")],
                integration_type: AgentIntegrationType::SingleFile,
                env_override: "COPILOT_SKILLS_DIR".to_string(),
            },
        ];
        Self { agents }
    }

    /// List all registered agents.
    pub fn agents(&self) -> &[AgentConfig] {
        &self.agents
    }

    /// Register a custom agent harness. If an agent with the same name
    /// (case-insensitive) already exists, its directory is updated in place
    /// rather than duplicated — so config re-declaring a built-in agent
    /// relocates it instead of double-syncing.
    pub fn register(&mut self, agent: AgentConfig) {
        let lower = agent.name.to_lowercase();
        if let Some(existing) = self
            .agents
            .iter_mut()
            .find(|a| a.name.to_lowercase() == lower)
        {
            existing.default_dir = agent.default_dir;
            existing.fallback_dirs = agent.fallback_dirs;
            existing.integration_type = agent.integration_type;
            existing.env_override = agent.env_override;
        } else {
            self.agents.push(agent);
        }
    }

    /// Builder-style merge of custom harnesses (name → skills dir), typically
    /// sourced from config. Each becomes a symlink-integration agent.
    pub fn with_custom_agents<I, N, P>(mut self, extras: I) -> Self
    where
        I: IntoIterator<Item = (N, P)>,
        N: Into<String>,
        P: Into<PathBuf>,
    {
        for (name, dir) in extras {
            self.register(AgentConfig::custom_symlink(name, dir));
        }
        self
    }

    /// Find an agent by name (case-insensitive).
    pub fn find(&self, name: &str) -> Option<&AgentConfig> {
        let lower = name.to_lowercase();
        self.agents.iter().find(|a| a.name.to_lowercase() == lower)
    }
}

/// Parse a `SKILLPACK_CUSTOM_AGENTS` spec: comma-separated `name=dir` pairs.
/// Blank entries and entries without `=` or with an empty side are skipped.
fn parse_custom_agents(spec: &str) -> Vec<(String, PathBuf)> {
    spec.split(',')
        .filter_map(|entry| {
            let (name, dir) = entry.split_once('=')?;
            let name = name.trim();
            let dir = dir.trim();
            if name.is_empty() || dir.is_empty() {
                None
            } else {
                Some((name.to_string(), PathBuf::from(dir)))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_custom_agents_handles_pairs_and_junk() {
        let parsed = parse_custom_agents("foo=/a/b, bar = /c/d ,,baz,=/x,qux=");
        assert_eq!(
            parsed,
            vec![
                ("foo".to_string(), PathBuf::from("/a/b")),
                ("bar".to_string(), PathBuf::from("/c/d")),
            ]
        );
    }
}
