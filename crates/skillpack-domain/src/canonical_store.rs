//! Canonical Store Domain Model
//!
//! Defines the canonical skill store, agent configurations, sync policies,
//! and the structures needed to orchestrate skill distribution.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Integration type for an AI coding assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentIntegrationType {
    /// Skills distributed via symbolic links into the agent skills directory.
    Symlink,
    /// Skills aggregated into a single structured index file (e.g., Cursor `.mdc`, Windsurf `index.json`).
    IndexFile,
    /// Skills rendered into a single markdown instructions file (e.g., GitHub Copilot `copilot-instructions.md`).
    SingleFile,
    /// Skills rendered as individual rule files in a flat rules directory (e.g., Roo Code `.roo/rules/`).
    RulesDir,
}

/// Configuration for a supported AI coding assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Human-readable agent name (e.g., "Claude", "Cursor").
    pub name: String,
    /// Default directory where the agent expects skills.
    pub default_dir: PathBuf,
    /// Fallback directories to try if the default doesn't exist.
    pub fallback_dirs: Vec<PathBuf>,
    /// How skills are integrated into the agent environment.
    pub integration_type: AgentIntegrationType,
    /// Environment variable that overrides the default directory.
    pub env_override: String,
}

impl AgentConfig {
    /// Build a custom symlink-integration agent for a harness not in the
    /// built-in registry. The env override is derived from the name so the
    /// dir stays relocatable. This is the onboarding path for a new agent
    /// harness: register once, consume the registry by reference forever.
    pub fn custom_symlink(name: impl Into<String>, dir: impl Into<PathBuf>) -> Self {
        let name = name.into();
        let env_override = format!(
            "{}_SKILLS_DIR",
            name.to_uppercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>()
        );
        Self {
            name,
            default_dir: dir.into(),
            fallback_dirs: vec![],
            integration_type: AgentIntegrationType::Symlink,
            env_override,
        }
    }

    /// Resolve all candidate directories for this agent (env override > primary > fallbacks).
    pub fn resolved_dirs(&self) -> Vec<PathBuf> {
        if let Ok(override_path) = std::env::var(&self.env_override) {
            return vec![PathBuf::from(override_path)];
        }
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let expand = |p: &PathBuf| {
            let s = p.to_string_lossy();
            if s.starts_with('~') {
                PathBuf::from(s.replacen("~", &home.to_string_lossy(), 1))
            } else {
                p.clone()
            }
        };
        let mut dirs = vec![expand(&self.default_dir)];
        for fb in &self.fallback_dirs {
            dirs.push(expand(fb));
        }
        dirs
    }

    /// Resolve the first existing candidate directory for this agent.
    pub fn resolved_dir(&self) -> Option<PathBuf> {
        self.resolved_dirs().into_iter().find(|p| p.exists())
    }
}

/// The canonical store root and policy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalStore {
    /// Root directory of the canonical store.
    pub root: PathBuf,
    /// Whether to run in dry-run mode (no filesystem changes).
    pub dry_run: bool,
    /// Whether to refresh CodeGraph indexing after sync.
    pub do_index: bool,
    /// If set, sync only this specific agent.
    pub only_agent: Option<String>,
    /// Namespace/tenant for multi-tenancy (e.g. "default", "team-alpha").
    pub namespace: Option<String>,
    /// OCI registry endpoint for pushing/pulling skill artifacts (e.g. "localhost:5000").
    pub oci_registry: Option<String>,
    /// OCI repository prefix within the registry (e.g. "skills").
    pub oci_repository: Option<String>,
    /// Whether to push skills to OCI registry after discovering them.
    pub oci_push: bool,
}

impl CanonicalStore {
    /// Create a canonical store with the given root.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            dry_run: false,
            do_index: true,
            only_agent: None,
            namespace: None,
            oci_registry: None,
            oci_repository: Some("skills".to_string()),
            oci_push: false,
        }
    }

    /// Enable OCI registry backend for this store.
    pub fn with_oci(mut self, registry: impl Into<String>) -> Self {
        self.oci_registry = Some(registry.into());
        self.oci_push = true;
        self
    }

    /// Resolve the canonical root, expanding `~` and appending namespace if set.
    pub fn resolved_root(&self) -> PathBuf {
        let root_str = self.root.to_string_lossy();
        let base = if root_str.starts_with("~") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
            PathBuf::from(root_str.replacen("~", &home.to_string_lossy(), 1))
        } else {
            self.root.clone()
        };
        match &self.namespace {
            Some(ns) => base.join(ns),
            None => base,
        }
    }
}

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

/// Result of syncing a single agent.
#[derive(Debug, Clone)]
pub struct AgentSyncResult {
    pub agent_name: String,
    pub success: bool,
    pub message: String,
    pub skills_synced: usize,
}

/// Result of a full canonical store sync.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub skills_processed: usize,
    pub agent_results: Vec<AgentSyncResult>,
    pub skills_discovered: Vec<CanonicalSkill>,
}

/// A discovered skill in the canonical store.
#[derive(Debug, Clone)]
pub struct CanonicalSkill {
    pub name: String,
    pub path: PathBuf,
    pub has_skill_md: bool,
    /// SHA-256 hash of the SKILL.md body (after stripping frontmatter). Used for deduplication.
    pub content_hash: Option<String>,
    /// Agent that originally surfaced this skill (e.g. "Claude Code", "Gemini CLI").
    pub source_agent: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registry_has_all_agents() {
        // builtin_registry is env-independent, so this count is stable.
        let registry = AgentRegistry::builtin_registry();
        assert_eq!(registry.agents().len(), 23);
        assert!(registry.find("Claude Code").is_some());
        assert!(registry.find("cursor").is_some());
        assert!(registry.find("windsurf").is_some());
        assert!(registry.find("Gemini CLI").is_some());
        assert!(registry.find("GitHub Copilot").is_some());
        assert!(registry.find("Agents (generic)").is_some());
        // Newly registered harnesses.
        assert!(registry.find("VS Code").is_some());
        assert!(registry.find("Crush").is_some());
        assert!(registry.find("Slate").is_some());
        assert!(registry.find("Devin").is_some());
    }

    #[test]
    fn register_adds_new_and_relocates_existing() {
        let mut registry = AgentRegistry::builtin_registry();
        let before = registry.agents().len();

        // A brand-new harness is appended.
        registry.register(AgentConfig::custom_symlink(
            "MyHarness",
            "~/.myharness/skills",
        ));
        assert_eq!(registry.agents().len(), before + 1);
        assert!(registry.find("MyHarness").is_some());

        // Re-registering an existing agent relocates it, not duplicates it.
        registry.register(AgentConfig::custom_symlink("Claude Code", "/custom/claude"));
        assert_eq!(registry.agents().len(), before + 1);
        assert_eq!(
            registry.find("Claude Code").unwrap().default_dir,
            PathBuf::from("/custom/claude")
        );
    }

    #[test]
    fn custom_symlink_derives_env_override() {
        let a = AgentConfig::custom_symlink("Cool Harness", "~/x");
        assert_eq!(a.env_override, "COOL_HARNESS_SKILLS_DIR");
        assert_eq!(a.integration_type, AgentIntegrationType::Symlink);
    }

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

    #[test]
    fn with_custom_agents_merges() {
        let registry = AgentRegistry::builtin_registry()
            .with_custom_agents([("Harness One", "~/h1"), ("Harness Two", "~/h2")]);
        assert!(registry.find("Harness One").is_some());
        assert!(registry.find("Harness Two").is_some());
    }

    #[test]
    fn test_agent_integration_types_coverage() {
        let registry = AgentRegistry::default_registry();
        let symlinks: Vec<_> = registry
            .agents()
            .iter()
            .filter(|a| a.integration_type == AgentIntegrationType::Symlink)
            .collect();
        let rules_dirs: Vec<_> = registry
            .agents()
            .iter()
            .filter(|a| a.integration_type == AgentIntegrationType::RulesDir)
            .collect();
        let single_files: Vec<_> = registry
            .agents()
            .iter()
            .filter(|a| a.integration_type == AgentIntegrationType::SingleFile)
            .collect();
        assert!(!symlinks.is_empty(), "must have symlink agents");
        assert!(!rules_dirs.is_empty(), "must have rules-dir agents");
        assert!(!single_files.is_empty(), "must have single-file agents");
    }

    #[test]
    fn test_agent_config_resolved_dirs_no_tilde() {
        let config = AgentConfig {
            name: "Test".to_string(),
            default_dir: PathBuf::from("~/test"),
            fallback_dirs: vec![PathBuf::from("~/fallback")],
            integration_type: AgentIntegrationType::Symlink,
            env_override: "TEST_DIR".to_string(),
        };
        let dirs = config.resolved_dirs();
        for d in &dirs {
            assert!(
                !d.to_string_lossy().starts_with('~'),
                "tilde not expanded: {:?}",
                d
            );
        }
    }

    #[test]
    fn test_canonical_store_resolved_root() {
        let store = CanonicalStore::new("~/Skills/shared");
        let resolved = store.resolved_root();
        assert!(!resolved.to_string_lossy().starts_with('~'));
    }
}
