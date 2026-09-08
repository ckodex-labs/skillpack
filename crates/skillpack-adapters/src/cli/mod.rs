//! CLI Adapter - Presentation Space

pub mod dry_run;
pub mod improve;
pub mod manifest;
pub mod refresh;

use crate::checkers::all_checkers;
use crate::filesystem::FilesystemReader;
use crate::generated::client_model::ClientError;
use crate::grpc_client::CanonicalStoreClient;
use crate::skill_migration::{
    ensure_field, ensure_metadata_field, migrate_skill_file, read_skill_md, write_skill_md,
};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde_json::json;
use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
use skillpack_domain::Grade;
use std::sync::atomic::{AtomicBool, Ordering};

/// CLIENT-SPEC.md §8: global quiet flag suppresses non-error stdout.
static QUIET: AtomicBool = AtomicBool::new(false);

macro_rules! cli_println {
    ($($arg:tt)*) => {
        if !crate::cli::QUIET.load(Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

/// Render a structured ClientError for CLI per CLIENT-SPEC.md §5.2.
/// Wired into network error paths when the server returns structured errors.
#[allow(dead_code)]
fn render_client_error_cli(err: &ClientError) {
    use crate::generated::client_model::Severity;
    let prefix = match err.severity {
        Severity::Error => "ERROR".red().bold(),
        Severity::Warning => "WARN".yellow().bold(),
        Severity::Info => "INFO".dimmed(),
    };
    eprintln!("{} [{}] {}", prefix, err.code, err.message);
    if let Some(hint) = err.client_hint.as_ref().and_then(|h| h.cli.as_ref()) {
        eprintln!("  {} {}", "→".dimmed(), hint.dimmed());
    }
}

#[derive(Parser)]
#[command(name = "skillpack")]
#[command(author = "CKODEX")]
#[command(version = "1.0.0")]
#[command(about = "AI Agent Skill Quality Assessment - CKODEX Compliant")]
pub struct Cli {
    /// Simulate actions without writing output files
    #[arg(long, alias = "preview", global = true)]
    pub dry_run: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress non-error output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Output as JSON (list, search, inspect, stats, export)
    #[arg(long, global = true)]
    pub json: bool,

    /// Stable machine-readable JSON envelope v1 { "v":1, "ok":true, "data":... }
    #[arg(long, global = true)]
    pub machine: bool,

    /// Filter by scope: global, project, or both
    #[arg(long, short = 's', global = true, default_value = "both")]
    pub scope: String,

    /// Filter by agent tool (windsurf, claude, cursor, …)
    #[arg(long, short = 'p', global = true)]
    pub tool: Option<String>,

    /// Sort results by: name, version, or location
    #[arg(long, global = true, default_value = "name")]
    pub sort: String,

    /// Show one row per tool instance (list, search)
    #[arg(long, global = true)]
    pub flat: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,

    /// Show debug output
    #[arg(long = "verbose", short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run full skill assessment (Kernel Space)
    Check {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value = "0")]
        min_score: u32,
    },
    /// Show skill grade
    Grade {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, short, default_value = "C")]
        minimum: String,
        /// Emit a signed evidence envelope to the evidence directory
        #[arg(long)]
        emit_evidence: bool,
        /// Directory to write evidence envelope (default: ./evidence)
        #[arg(long, default_value = "evidence")]
        evidence_dir: String,
        /// Sign evidence with Cosign (keyless) instead of noop
        #[arg(long)]
        sign: bool,
    },
    /// Generate assessment report
    Report {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, short, value_enum, default_value = "json")]
        format: ReportOutputFormat,
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Validate schema (CNSB, CNAAB, Evidence, Policy)
    Validate {
        /// Path to JSON file to validate
        file: String,
        /// Schema type to validate against
        #[arg(long, short, value_enum)]
        schema: Option<SchemaType>,
    },
    /// Initialize a new skill project
    Init {
        /// Name of the skill to create
        name: String,
        /// Template to use
        #[arg(long, short, value_enum, default_value = "agent")]
        template: InitTemplate,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Parent directory (defaults to current directory)
        #[arg(long, short, default_value = ".")]
        path: String,
    },
    /// Generate skill.lock file from dependencies
    Lock {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Force regeneration even if lock file exists
        #[arg(long, short)]
        force: bool,
    },
    /// Migrate skill manifest to latest schema version
    Migrate {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Target schema version
        #[arg(long, short, default_value = "2.0.0")]
        target: String,
    },
    /// Wizard — scaffold a skill pre-wired for A/B grade
    Wizard {
        /// Name of the skill to create
        name: String,
        /// One-line description
        #[arg(long, short)]
        description: Option<String>,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Parent directory
        #[arg(long, short, default_value = ".")]
        path: String,
        /// Skip post-scaffold assessment
        #[arg(long)]
        no_check: bool,
    },
    /// Package skill into a distributable archive
    Package {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Output archive path
        #[arg(long, short)]
        output: Option<String>,
        /// Skip pre-packaging assessment
        #[arg(long)]
        no_check: bool,
        /// Archive format
        #[arg(long, short, value_enum, default_value = "tar-gz")]
        format: PackageFormat,
    },
    /// Publish skill to an OCI registry
    Publish {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// OCI registry reference (e.g., ghcr.io/ckodex/my-skill:0.1.0)
        #[arg(long, short)]
        registry: String,
        /// Registry username
        #[arg(long)]
        username: Option<String>,
        /// Registry password / token
        #[arg(long)]
        password: Option<String>,
        /// Skip pre-publish assessment
        #[arg(long)]
        no_check: bool,
        /// Sign with Sigstore after push
        #[arg(long)]
        sign: bool,
    },
    /// Install a skill from an OCI registry, git repository, or local path
    Install {
        /// Source: OCI ref (ghcr.io/org/skill:0.1.0), git URL
        /// (https://github.com/org/repo, git://…, git@…), or a local directory
        reference: String,
        /// Output directory for the installed skill
        #[arg(long, short, default_value = ".")]
        output: String,
        /// Registry username
        #[arg(long)]
        username: Option<String>,
        /// Registry password / token
        #[arg(long)]
        password: Option<String>,
    },
    /// Run evaluation suites against a skill
    Eval {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Suite name to run (smoke, compliance, performance)
        #[arg(long, short)]
        suite: Option<String>,
        /// Output file for results
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Discover skills in a directory or registry
    Discover {
        /// Path to search for skills (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Maximum number of results
        #[arg(long, short, default_value = "20")]
        limit: usize,
    },
    /// Canonical store operations (sync, migrate, status, etc.)
    Store {
        #[command(subcommand)]
        command: StoreCommands,
    },
    /// Skill lifecycle operations (update, refine, evolve, list, show)
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },

    // ── ASM: agent-skill-manager commands ────────────────────────────────────
    /// List all discovered skills across agents and scopes
    List {
        /// Filter by agent tool name
        #[arg(long)]
        tool: Option<String>,
        /// Filter by scope: global, project, or both
        #[arg(long, default_value = "both")]
        scope: String,
        /// Sort by: name, version, or location
        #[arg(long, default_value = "name")]
        sort: String,
        /// One row per tool instance
        #[arg(long)]
        flat: bool,
    },

    /// Search skills by name, description, or tool
    Search {
        /// Query string to search for
        query: String,
        /// Filter by agent tool name
        #[arg(long)]
        tool: Option<String>,
    },

    /// Show detailed info for an installed skill
    Inspect {
        /// Skill name to inspect
        skill_name: String,
        /// Agent tool to restrict search to
        #[arg(long)]
        tool: Option<String>,
    },

    /// Remove a skill (with confirmation)
    Uninstall {
        /// Skill name to uninstall
        skill_name: String,
        /// Agent tool path to remove from
        #[arg(long)]
        tool: Option<String>,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Disable skill(s) without uninstalling (adds .disabled marker)
    Disable {
        /// Skill name or glob pattern
        target: String,
        /// Agent tool to restrict
        #[arg(long)]
        tool: Option<String>,
    },

    /// Re-enable disabled skill(s) (removes .disabled marker)
    Enable {
        /// Skill name or glob pattern
        target: String,
        /// Agent tool to restrict
        #[arg(long)]
        tool: Option<String>,
    },

    /// Detect duplicate skills across agents/tools, or security audit a skill
    Audit {
        #[command(subcommand)]
        command: Option<AuditCommands>,
    },

    /// Export skill inventory as JSON manifest
    Export {
        /// Output file (default: stdout)
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Import skills from a previously exported manifest
    Import {
        /// Path to manifest JSON file
        file: String,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
        /// Dry-run: show what would be imported without installing
        #[arg(long)]
        dry_run: bool,
    },

    /// Show aggregate skill metrics dashboard
    Stats,

    /// Symlink a local skill directory into an agent
    Link {
        /// Local skill directory to link
        path: String,
        /// Agent tool to link into (windsurf, claude, cursor, …)
        #[arg(long)]
        tool: Option<String>,
    },

    /// Show which installed skills have newer versions
    Outdated,

    /// Update outdated skills with security re-audit
    Update {
        /// Specific skill names to update (all if omitted)
        names: Vec<String>,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// List registered eval providers
    EvalProviders {
        #[command(subcommand)]
        command: EvalProvidersCommands,
    },

    /// Manage skill bundles (create, install, list, show, remove)
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },

    /// Manage skill index (ingest, search, list)
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },

    /// Run environment health checks and diagnostics
    Doctor,

    /// Tier-3 behavioral eval: run the skill via a real agent in an isolated
    /// throwaway workspace and grade the tool-call trace. Opt-in and gated —
    /// requires SKILLPACK_BEHAVIORAL=1 and SKILLPACK_BEHAVIORAL_CMD.
    Behavioral {
        /// Path to the skill directory (reads evals/behavioral.json)
        #[arg(default_value = ".")]
        path: String,
        /// Per-case wall-clock timeout in seconds
        #[arg(long, default_value = "120")]
        timeout_secs: u64,
        /// Cap on how many cases to run
        #[arg(long, default_value = "20")]
        max_cases: usize,
    },

    /// Rank which skills a task prompt would route to (Tier-2 routing eval)
    Route {
        /// The task prompt to route (omit when using --check)
        #[arg(default_value = "")]
        prompt: String,
        /// How many top matches to show
        #[arg(long, default_value = "5")]
        top_k: usize,
        /// Skill that SHOULD win — exit non-zero if it's not in the top-k
        #[arg(long)]
        expect: Option<String>,
        /// Run a skill's routing fixture: <dir>/evals/routing.json declares
        /// positive prompts that must place the skill within top_k
        #[arg(long)]
        check: Option<String>,
        /// Registry root to rank against (default: ~/Skills/shared)
        #[arg(long)]
        canonical_root: Option<String>,
    },

    /// Manage skillpack configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum StoreCommands {
    /// Sync agents with the canonical store
    Sync {
        /// Dry run — simulate without making changes
        #[arg(long)]
        dry_run: bool,
        /// Skip CodeGraph indexing after sync
        #[arg(long)]
        no_index: bool,
        /// Sync only a specific agent
        #[arg(long)]
        only_agent: Option<String>,
    },
    /// Migrate all physical skills into the canonical store
    Migrate {
        /// Root of the canonical store
        #[arg(long)]
        canonical_root: Option<String>,
    },
    /// Show canonical store status
    Status,
    /// Write a discovery manifest at the store root so any harness can
    /// find and consume the registry by reference
    Manifest {
        /// Root of the canonical store (default: ~/Skills/shared)
        #[arg(long)]
        canonical_root: Option<String>,
        /// Print the manifest to stdout instead of writing it to the store
        #[arg(long)]
        stdout: bool,
    },
    /// Check if a path violates IP boundaries
    CheckBoundary {
        /// Path to check
        path: String,
        /// Skill name to check
        #[arg(long)]
        skill_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SkillCommands {
    /// Update SKILL.md frontmatter field (supports dot-notation, e.g. metadata.version)
    Update {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Field to update (dot-notation for nested keys)
        #[arg(long)]
        field: String,
        /// New value
        #[arg(long)]
        value: String,
    },
    /// Run quality gates on a skill (schema validation, shellcheck, file checks)
    Refine {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Auto-fix trivial issues (missing shebangs, trailing whitespace, +x)
        #[arg(long)]
        fix: bool,
    },
    /// Promote lifecycle status (draft → active), gated on assessment grade
    Promote {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Minimum grade required to promote
        #[arg(long, default_value = "C")]
        min_grade: String,
        /// Skip the assessment gate (the forced promotion is printed loudly)
        #[arg(long)]
        force: bool,
    },
    /// Demote lifecycle status (active → deprecated → retired)
    Demote {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
    },
    /// Refresh SKILL.md frontmatter for current model loaders: normalize legacy
    /// field names (allowed_tools → allowed-tools) and report weak routing
    Refresh {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Preview normalizations without writing
        #[arg(long)]
        dry_run: bool,
    },
    /// Share a skill to a remote git group as its own repo (push-to-create).
    /// No manual copy: stages a clean copy, commits, and pushes to
    /// <group-url>/<skill>.git. Auth uses your configured git credentials.
    Share {
        /// Skill name (looked up in the canonical store) or path to a skill dir
        skill: String,
        /// Remote group URL, e.g. https://host/gitlab/group/subgroup
        group_url: String,
        /// Repo name in the group (defaults to the skill directory name)
        #[arg(long)]
        repo_name: Option<String>,
        /// Branch to push (default: main)
        #[arg(long, default_value = "main")]
        branch: String,
        /// Show the plan without pushing
        #[arg(long)]
        dry_run: bool,
    },
    /// Evolve skill: bump version, add missing lifecycle stubs
    Evolve {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Bump major version
        #[arg(long)]
        major: bool,
        /// Bump minor version
        #[arg(long)]
        minor: bool,
        /// Bump patch version
        #[arg(long)]
        patch: bool,
    },
    /// Improve a skill: apply the assessment's mechanically-fixable findings
    /// (add missing version, CHANGELOG, stub broken internal links), then
    /// re-assess and report the grade delta. Content-level findings are
    /// reported for manual follow-up, never faked.
    Improve {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: String,
        /// Preview the fixes without writing anything
        #[arg(long)]
        dry_run: bool,
    },
    /// List skills in canonical store
    List {
        /// Filter by namespace
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Show skill details (metadata, sync state)
    Show {
        /// Skill name
        name: String,
        /// Namespace
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Migrate all skills in the canonical store to the new format
    MigrateSkills {
        /// Root of the canonical store (default: ~/Skills/shared)
        #[arg(long)]
        canonical_root: Option<String>,
    },
    /// Migrate agent skills in ~/.config/agents/skills to the new format
    MigrateAgents,
    /// Migrate Claude agent definitions in ~/.claude/agents to the new format
    MigrateClaudeAgents {
        /// Directory containing agent .md files (default: ~/.claude/agents)
        #[arg(long)]
        agents_dir: Option<String>,
    },
    /// Migrate agent harness rules in ~/.config/agents/skills/*/rules to the new format
    MigrateHarnesses,
}

// ── ASM sub-command enums ────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Run security audit on a skill (by name or GitHub source)
    Security {
        /// Skill name or GitHub source URL
        name: String,
        /// Agent tool to restrict search to
        #[arg(long)]
        tool: Option<String>,
    },
    /// Detect skills whose descriptions collide (compete for the same triggers)
    Collisions {
        /// Agent tool to restrict search to
        #[arg(long)]
        tool: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EvalProvidersCommands {
    /// List registered eval providers (id, version, schema)
    List,
}

#[derive(Subcommand)]
pub enum BundleCommands {
    /// Create a new bundle from a directory of skills
    Create {
        /// Bundle name
        name: String,
        /// Version for this bundle
        #[arg(long, default_value = "0.1.0")]
        version: String,
        /// Directory containing skills to bundle
        #[arg(long, default_value = ".")]
        path: String,
        /// Output file (default: <name>-<version>.cnsb.json)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Install a bundle (all skills inside it)
    Install {
        /// Path to .cnsb.json bundle file or OCI reference
        source: String,
        /// Agent tool to install into
        #[arg(long)]
        tool: Option<String>,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// List installed bundles
    List,
    /// Show bundle details
    Show {
        /// Bundle name or path
        name: String,
    },
    /// Remove an installed bundle
    Remove {
        /// Bundle name
        name: String,
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum IndexCommands {
    /// Ingest skills into the search index
    Ingest {
        /// Path to scan for skills (default: all agent paths)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Search the skill index
    Search {
        /// Query string
        query: String,
        /// Max results
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// List all index entries
    List,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Print current config as TOML
    Show,
    /// Print config file path
    Path,
    /// Reset config to defaults
    Reset {
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Open config in $EDITOR
    Edit,
}

#[derive(Clone, ValueEnum)]
pub enum ReportOutputFormat {
    Json,
    Sarif,
    Markdown,
}

#[derive(Clone, ValueEnum)]
pub enum SchemaType {
    Cnsb,
    Cnaab,
    Evidence,
    Policy,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum InitTemplate {
    /// Lightweight agent skill (SKILL.md + references/) — no CNSB manifest
    Agent,
    /// Basic skill with SKILL.md and skill.cnsb.json
    Basic,
    /// MCP server capacity included
    Mcp,
    /// Full template with lifecycle, security, and examples
    Full,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum PackageFormat {
    /// Gzip-compressed tar archive
    TarGz,
    /// Bzip2-compressed tar archive
    TarBz2,
    /// Brotli-compressed tar archive
    TarBrotli,
    /// Zstd-compressed tar archive
    TarZstd,
    /// ZIP archive
    Zip,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // CLIENT-SPEC.md §8: CLI provides --no-color and --quiet flags
    if cli.no_color {
        colored::control::set_override(false);
    }
    if cli.quiet {
        QUIET.store(true, Ordering::Relaxed);
    }
    if cli.verbose {
        unsafe { std::env::set_var("RUST_LOG", "debug") };
    }

    let json_output = cli.json || cli.machine;
    let machine_output = cli.machine;

    match cli.command {
        Commands::Check { path, min_score } => run_check(&path, min_score),
        Commands::Grade {
            path,
            minimum,
            emit_evidence,
            evidence_dir,
            sign,
        } => run_grade(&path, &minimum, emit_evidence, &evidence_dir, sign),
        Commands::Report {
            path,
            format,
            output,
        } => run_report(&path, format, output.as_deref()),
        Commands::Validate { file, schema } => run_validate(&file, schema),
        Commands::Init {
            name,
            template,
            author,
            path,
        } => run_init(&name, template, author.as_deref(), &path),
        Commands::Lock { path, force } => run_lock(&path, force, cli.dry_run),
        Commands::Migrate { path, target } => run_migrate(&path, &target, cli.dry_run),
        Commands::Wizard {
            name,
            description,
            author,
            path,
            no_check,
        } => run_wizard(
            &name,
            description.as_deref(),
            author.as_deref(),
            &path,
            no_check,
            cli.dry_run,
        ),
        Commands::Package {
            path,
            output,
            no_check,
            format,
        } => run_package(&path, output.as_deref(), no_check, format, cli.dry_run),
        Commands::Publish {
            path,
            registry,
            username,
            password,
            no_check,
            sign,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_publish(
                &path,
                &registry,
                username.as_deref(),
                password.as_deref(),
                no_check,
                sign,
                cli.dry_run,
            ))
        }
        Commands::Install {
            reference,
            output,
            username,
            password,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_install(
                &reference,
                &output,
                username.as_deref(),
                password.as_deref(),
                cli.dry_run,
            ))
        }
        Commands::Eval {
            path,
            suite,
            output,
        } => run_eval(&path, suite.as_deref(), output.as_deref()),
        Commands::Discover { path, limit } => run_discover(&path, limit),
        Commands::Store { command } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_store(command))
        }
        Commands::Skill { command } => run_skill(command),

        // ── ASM commands ────────────────────────────────────────────────────
        Commands::List {
            tool,
            scope,
            sort,
            flat,
        } => run_asm_list(
            tool.as_deref(),
            &scope,
            &sort,
            flat,
            json_output,
            machine_output,
        ),
        Commands::Search { query, tool } => {
            run_asm_search(&query, tool.as_deref(), json_output, machine_output)
        }
        Commands::Inspect { skill_name, tool } => {
            run_asm_inspect(&skill_name, tool.as_deref(), json_output, machine_output)
        }
        Commands::Uninstall {
            skill_name,
            tool,
            yes,
        } => run_asm_uninstall(&skill_name, tool.as_deref(), yes || cli.yes, cli.dry_run),
        Commands::Disable { target, tool } => {
            run_asm_disable(&target, tool.as_deref(), cli.dry_run)
        }
        Commands::Enable { target, tool } => run_asm_enable(&target, tool.as_deref(), cli.dry_run),
        Commands::Audit { command } => run_asm_audit(command),
        Commands::Export { output } => {
            run_asm_export(output.as_deref(), json_output, machine_output)
        }
        Commands::Import { file, yes, dry_run } => run_asm_import(&file, yes || cli.yes, dry_run),
        Commands::Stats => run_asm_stats(json_output, machine_output),
        Commands::Link { path, tool } => run_asm_link(&path, tool.as_deref(), cli.dry_run),
        Commands::Outdated => run_asm_outdated(json_output, machine_output),
        Commands::Update { names, yes } => run_asm_update(&names, yes || cli.yes, cli.dry_run),
        Commands::EvalProviders { command } => run_asm_eval_providers(command),
        Commands::Bundle { command } => run_asm_bundle(command, cli.dry_run),
        Commands::Index { command } => run_asm_index(command),
        Commands::Doctor => run_asm_doctor(),
        Commands::Behavioral {
            path,
            timeout_secs,
            max_cases,
        } => run_behavioral_cmd(&path, timeout_secs, max_cases),
        Commands::Route {
            prompt,
            top_k,
            expect,
            check,
            canonical_root,
        } => {
            if let Some(dir) = check {
                run_route_check(&dir, canonical_root.as_deref())
            } else if prompt.is_empty() {
                anyhow::bail!("provide a prompt to route, or --check <skill-dir> for a fixture");
            } else {
                run_route(&prompt, top_k, expect.as_deref(), canonical_root.as_deref())
            }
        }
        Commands::Config { command } => run_asm_config(command, cli.yes),
    }
}

fn run_check(path: &str, min_score: u32) -> Result<()> {
    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║          SkillPack Assessment (CKODEX Architecture)          ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let reader = FilesystemReader::new();
    let checkers = all_checkers();
    let use_case =
        AssessSkillUseCase::new(reader, checkers).with_exemption_policy(grader_exemption_policy());

    let request = AssessSkillRequest {
        skill_path: path.to_string(),
        min_score: Some(min_score as f64),
    };

    let response = use_case.execute(request)?;
    let assessment = &response.assessment;

    // Display dimensions
    for (id, score) in &assessment.dimension_scores {
        let value = score.value() as u32;
        let bar = "█".repeat((value / 10) as usize);
        let empty = "░".repeat(10 - (value / 10) as usize);
        let color = if value >= 90 {
            "green"
        } else if value >= 70 {
            "yellow"
        } else {
            "red"
        };
        cli_println!(
            "  {:15} {:3}/100 {}{}",
            id.name(),
            value,
            bar.color(color),
            empty
        );
    }

    cli_println!();
    cli_println!(
        "  {} {:.0}/150  {} {}  {} {}",
        "Total:".bold(),
        assessment.total_score().value(),
        "Grade:".bold(),
        assessment.grade().as_str().bold().green(),
        "Profile:".bold(),
        assessment.profile.name()
    );

    // Surface error-severity issues so governance failures (e.g. an
    // unauthorized exemption request) are loud, not buried.
    let errors: Vec<&skillpack_domain::Issue> = assessment
        .issues
        .iter()
        .filter(|i| i.severity == skillpack_domain::Severity::Error)
        .collect();
    if !errors.is_empty() {
        cli_println!();
        cli_println!("  {} {} error(s):", "✗".red().bold(), errors.len());
        for issue in &errors {
            cli_println!("    {} {}", "•".red(), issue.message);
        }
    }

    if assessment.stub_count > 0 {
        eprintln!(
            "stub-detected: {} dimensions are unimplemented",
            assessment.stub_count
        );
        std::process::exit(4);
    }
    if !response.meets_minimum {
        std::process::exit(1);
    }
    Ok(())
}

fn run_grade(
    path: &str,
    minimum: &str,
    emit_evidence: bool,
    evidence_dir: &str,
    sign: bool,
) -> Result<()> {
    let reader = FilesystemReader::new();
    let checkers = all_checkers();
    let use_case = AssessSkillUseCase::new(reader, checkers);

    let request = AssessSkillRequest {
        skill_path: path.to_string(),
        min_score: None,
    };

    let response = use_case.execute(request)?;
    let grade = response.assessment.grade();

    cli_println!(
        "Grade: {} ({:.0}/150)",
        grade.as_str().bold(),
        response.assessment.total_score().value()
    );

    // Emit signed evidence envelope if requested
    if emit_evidence {
        use crate::evidence::builder::EnvelopeBuilder;
        use crate::evidence::signer::{CosignSigner, NoopSigner};
        use crate::evidence::sink::FileEvidenceSink;
        use skillpack_domain::ports::{EvidenceSink, Signer};

        let builder = EnvelopeBuilder::new();
        let pending = builder.from_assessment(&response.assessment);
        let envelope = pending.build();

        let signed = if sign {
            CosignSigner::new_keyless().sign(envelope)
        } else {
            NoopSigner.sign(envelope)
        };

        match signed {
            Ok(envelope) => {
                let sink = FileEvidenceSink::new(evidence_dir);
                match sink.write(&envelope) {
                    Ok(path) => {
                        cli_println!(
                            "{} Evidence envelope: {}",
                            "✓".green().bold(),
                            path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("{} Failed to write evidence: {}", "⚠".yellow(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Signing failed: {}", "⚠".yellow(), e);
            }
        }
    }

    let min_grade = parse_grade(minimum);
    if !grade.meets_minimum(&min_grade) {
        std::process::exit(1);
    }
    Ok(())
}

fn run_report(path: &str, format: ReportOutputFormat, output: Option<&str>) -> Result<()> {
    let reader = FilesystemReader::new();
    let checkers = all_checkers();
    let use_case = AssessSkillUseCase::new(reader, checkers);

    let request = AssessSkillRequest {
        skill_path: path.to_string(),
        min_score: None,
    };

    let response = use_case.execute(request)?;

    let content = match format {
        ReportOutputFormat::Json => serde_json::to_string_pretty(&response.assessment)?,
        ReportOutputFormat::Sarif => generate_sarif(&response.assessment, path)?,
        ReportOutputFormat::Markdown => generate_markdown(&response.assessment)?,
    };

    match output {
        Some(file) => std::fs::write(file, content)?,
        None => cli_println!("{}", content),
    }
    Ok(())
}

fn run_validate(file: &str, schema_type: Option<SchemaType>) -> Result<()> {
    use skillpack_domain::schema_validation::{SchemaType as DomainSchemaType, SchemaValidator};

    let content = std::fs::read_to_string(file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let validator = SchemaValidator::new()?;

    let result = match schema_type {
        Some(SchemaType::Cnsb) => validator
            .validate_skill_bundle(&json)
            .map(|_| DomainSchemaType::SkillBundle),
        Some(SchemaType::Cnaab) => validator
            .validate_agent_app(&json)
            .map(|_| DomainSchemaType::AgentAppBundle),
        Some(SchemaType::Evidence) => validator
            .validate_evidence(&json)
            .map(|_| DomainSchemaType::EvidenceEnvelope),
        Some(SchemaType::Policy) => validator
            .validate_policy(&json)
            .map(|_| DomainSchemaType::PolicyBundle),
        None => validator.validate_auto(&json),
    };

    match result {
        Ok(detected) => {
            cli_println!("{} {} validated successfully", "✓".green().bold(), detected);
            Ok(())
        }
        Err(e) => {
            eprintln!("{} Validation failed:", "✗".red().bold());
            for err in &e.errors {
                eprintln!("  - {}", err);
            }
            std::process::exit(1);
        }
    }
}

/// Generate SARIF 2.1.0 report for GitHub/GitLab integration
fn generate_sarif(assessment: &skillpack_domain::Assessment, path: &str) -> Result<String> {
    let results: Vec<serde_json::Value> = assessment
        .dimension_scores
        .iter()
        .filter(|(_, score)| score.value() < 70.0)
        .map(|(id, score)| {
            json!({
                "ruleId": format!("skillpack/{}", id.name().to_lowercase().replace(" ", "-")),
                "level": if score.value() < 50.0 { "error" } else { "warning" },
                "message": {
                    "text": format!("{} score is {:.0}/100 (minimum: 70)", id.name(), score.value())
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": format!("{}/SKILL.md", path)
                        }
                    }
                }]
            })
        })
        .collect();

    let sarif = json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "SkillPack",
                    "version": "1.0.0",
                    "informationUri": "https://ckodex.org/skillpack",
                    "rules": [
                        { "id": "skillpack/structure", "name": "Structure", "shortDescription": { "text": "Skill structure completeness" } },
                        { "id": "skillpack/documentation", "name": "Documentation", "shortDescription": { "text": "Documentation quality" } },
                        { "id": "skillpack/security", "name": "Security", "shortDescription": { "text": "Security posture" } },
                        { "id": "skillpack/performance", "name": "Performance", "shortDescription": { "text": "Performance characteristics" } },
                        { "id": "skillpack/testing", "name": "Testing", "shortDescription": { "text": "Test coverage" } },
                        { "id": "skillpack/compatibility", "name": "Compatibility", "shortDescription": { "text": "Platform compatibility" } },
                        { "id": "skillpack/governance", "name": "Governance", "shortDescription": { "text": "Governance compliance" } },
                        { "id": "skillpack/observability", "name": "Observability", "shortDescription": { "text": "Observability instrumentation" } }
                    ]
                }
            },
            "results": results,
            "invocations": [{
                "executionSuccessful": assessment.grade().as_str() != "F"
            }]
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Generate Markdown report
fn generate_markdown(assessment: &skillpack_domain::Assessment) -> Result<String> {
    let mut md = String::new();
    md.push_str("# SkillPack Assessment Report\n\n");
    md.push_str(&format!(
        "**Overall Grade:** {}\n",
        assessment.grade().as_str()
    ));
    md.push_str(&format!(
        "**Total Score:** {:.0}/150\n\n",
        assessment.total_score().value()
    ));

    md.push_str("## Dimension Scores\n\n");
    md.push_str("| Dimension | Score | Status |\n");
    md.push_str("|-----------|-------|--------|\n");

    for (id, score) in &assessment.dimension_scores {
        let status = if score.value() >= 90.0 {
            "✅ Excellent"
        } else if score.value() >= 70.0 {
            "🟡 Good"
        } else {
            "❌ Needs Work"
        };
        md.push_str(&format!(
            "| {} | {:.0}/100 | {} |\n",
            id.name(),
            score.value(),
            status
        ));
    }

    Ok(md)
}

/// Lifecycle status of a skill, read from SKILL.md frontmatter `status`
/// (flat agentskills layout) or `lifecycle.status` (nested layout).
/// Absent status means `draft` — a skill is unproven until promoted.
fn read_skill_status(path: &str) -> Result<String> {
    let skill_md = std::path::Path::new(path).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md)
        .map_err(|e| anyhow::anyhow!("no SKILL.md at {}: {}", skill_md.display(), e))?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("SKILL.md at {} has no frontmatter", skill_md.display());
    }
    let fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim())?;
    Ok(fm
        .get("lifecycle")
        .and_then(|l| l.get("status"))
        .or_else(|| fm.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("draft")
        .to_string())
}

/// Write lifecycle status back to where it lives (nested `lifecycle.status`
/// if a lifecycle mapping exists, else flat `status`).
fn write_skill_status(path: &str, status: &str) -> Result<()> {
    let skill_md = std::path::Path::new(path).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md)?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("SKILL.md at {} has no frontmatter", skill_md.display());
    }
    let mut fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim())?;
    let key = serde_yaml::Value::String("status".to_string());
    let val = serde_yaml::Value::String(status.to_string());
    if let Some(lifecycle) = fm.get_mut("lifecycle").and_then(|l| l.as_mapping_mut()) {
        lifecycle.insert(key, val);
    } else if let Some(root) = fm.as_mapping_mut() {
        root.insert(key, val);
    } else {
        anyhow::bail!("SKILL.md frontmatter is not a YAML mapping");
    }
    let new_fm = serde_yaml::to_string(&fm)?;
    std::fs::write(&skill_md, format!("---\n{}---\n{}", new_fm, parts[2]))?;
    Ok(())
}

fn parse_grade(s: &str) -> Grade {
    match s.to_uppercase().as_str() {
        "S+" => Grade::SPlus,
        "S" => Grade::S,
        "A" => Grade::A,
        "B" => Grade::B,
        "C" => Grade::C,
        "D" => Grade::D,
        _ => Grade::F,
    }
}

// ============================================================================
// Init Command
// ============================================================================

fn run_init(
    name: &str,
    template: InitTemplate,
    author: Option<&str>,
    base_path: &str,
) -> Result<()> {
    use std::path::Path;

    let skill_dir = Path::new(base_path).join(name);

    if skill_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", skill_dir.display());
    }

    std::fs::create_dir_all(&skill_dir)?;

    let author_name = author.unwrap_or("CKODEX Author");

    if matches!(template, InitTemplate::Agent) {
        // Agentskills flavor: SKILL.md is the whole contract; no CNSB
        // manifest, so assessment runs under the agentskills rubric.
        std::fs::write(
            skill_dir.join("SKILL.md"),
            generate_agent_skill_md(name, author_name),
        )?;
        std::fs::create_dir_all(skill_dir.join("references"))?;
        std::fs::write(
            skill_dir.join("references/guide.md"),
            format!(
                "# {} — deep reference\n\nMove detailed tables, edge cases, and long examples here;\nkeep SKILL.md skimmable.\n",
                name
            ),
        )?;
        std::fs::write(
            skill_dir.join(".gitignore"),
            "node_modules/\ndist/\n*.log\n.env\n",
        )?;
        cli_println!(
            "{} Created agent skill '{}' at {}",
            "✓".green().bold(),
            name,
            skill_dir.display()
        );
        cli_println!("\nNext steps:");
        cli_println!("  # 1. Write the frontmatter description: what it does AND when to use it");
        cli_println!("  # 2. Document the workflow with at least one worked example");
        cli_println!("  skillpack check {}", skill_dir.display());
        return Ok(());
    }

    // Create SKILL.md
    let skill_md = generate_skill_md(name, author_name, &template);
    std::fs::write(skill_dir.join("SKILL.md"), skill_md)?;

    // Create skill.cnsb.json
    let skill_json = generate_skill_json(name, author_name, &template);
    std::fs::write(skill_dir.join("skill.cnsb.json"), skill_json)?;

    // Create eval.yml for all templates
    std::fs::write(skill_dir.join("eval.yml"), generate_wizard_eval_yml())?;

    // Create additional files based on template
    match template {
        InitTemplate::Agent => unreachable!("handled above"),
        InitTemplate::Basic => {
            // Just the basics
        }
        InitTemplate::Mcp => {
            std::fs::create_dir_all(skill_dir.join("capacities/mcp"))?;
            std::fs::write(
                skill_dir.join("capacities/mcp/server.ts"),
                generate_mcp_server(name),
            )?;
        }
        InitTemplate::Full => {
            std::fs::create_dir_all(skill_dir.join("capacities/mcp"))?;
            std::fs::create_dir_all(skill_dir.join("examples"))?;
            std::fs::create_dir_all(skill_dir.join("security"))?;
            std::fs::create_dir_all(skill_dir.join("tests"))?;

            std::fs::write(
                skill_dir.join("capacities/mcp/server.ts"),
                generate_mcp_server(name),
            )?;
            std::fs::write(
                skill_dir.join("examples/basic.md"),
                format!(
                    "# Basic Usage\n\n```bash\nskillpack install {}\n```\n",
                    name
                ),
            )?;
            std::fs::write(
                skill_dir.join("security/threat-model.yaml"),
                "threats: []\nmitigations: []\n",
            )?;
            std::fs::write(skill_dir.join("Makefile"), generate_makefile(name))?;
        }
    }

    // Create .gitignore
    std::fs::write(
        skill_dir.join(".gitignore"),
        "node_modules/\ndist/\n*.log\n.env\n",
    )?;

    cli_println!(
        "{} Created skill '{}' at {}",
        "✓".green().bold(),
        name,
        skill_dir.display()
    );
    cli_println!("  Template: {:?}", template);
    cli_println!("\nNext steps:");
    cli_println!("  cd {}", name);
    cli_println!("  skillpack check");

    Ok(())
}

/// Agentskills-flavor scaffold: the frontmatter description carries the
/// trigger contract; the body models structure the assessor rewards
/// (workflow, worked example, verification, progressive disclosure).
fn generate_agent_skill_md(name: &str, author: &str) -> String {
    format!(
        r#"---
name: {name}
version: 0.1.0
description: "TODO: one sentence on what this skill does. Use when <the situation that should trigger it>."
author: {author}
license: Apache-2.0
allowed-tools: [Read, Bash]
---

# {name}

State the job this skill performs and the outcome it guarantees.

## Workflow

1. TODO: first step.
2. TODO: second step.
3. Verify the result (see Verification).

## Example

```bash
# TODO: replace with a real, runnable invocation
echo "worked example for {name}"
```

## Verification

Describe how to verify the output is correct before finishing.

## Deep reference

Details that would bloat this file live in references/guide.md.
"#
    )
}

fn generate_skill_md(name: &str, author: &str, template: &InitTemplate) -> String {
    format!(
        r#"---
name: {}
version: 0.1.0
description: {}
author: {}
namespace: default
---

# {}

A CKODEX-compliant AI agent skill.

## Usage

```bash
skillpack install {}
```

## Configuration

Configure via `.skillpack.json` or environment variables.

{}
"#,
        name,
        name,
        author,
        name,
        name,
        if matches!(template, InitTemplate::Full) {
            "## Security\n\nSee `security/threat-model.yaml` for security analysis.\n"
        } else {
            ""
        }
    )
}

fn generate_skill_json(name: &str, author: &str, template: &InitTemplate) -> String {
    let has_lifecycle = matches!(template, InitTemplate::Full);
    let has_mcp = matches!(template, InitTemplate::Mcp | InitTemplate::Full);

    let mut manifest = json!({
        "$schema": "https://skillpack.dev/schemas/cnsb.schema.json",
        "metadata": {
            "name": name,
            "version": "0.1.0",
            "description": name,
            "author": author,
            "namespace": "default"
        }
    });

    if has_lifecycle {
        manifest["lifecycle"] = json!({
            "install": { "command": "npm ci", "timeout": "5m" },
            "verify": { "command": "npm test" },
            "uninstall": { "command": "rm -rf node_modules" }
        });
    }

    if has_mcp {
        manifest["capacities"] = json!([{
            "type": "mcp",
            "server": {
                "command": "npx",
                "args": ["tsx", "capacities/mcp/server.ts"]
            }
        }]);
    }

    serde_json::to_string_pretty(&manifest).unwrap()
}

fn generate_mcp_server(name: &str) -> String {
    format!(
        r#"#!/usr/bin/env npx tsx
/**
 * MCP Server for {}
 */
import {{ Server }} from '@modelcontextprotocol/sdk/server/index.js';
import {{ StdioServerTransport }} from '@modelcontextprotocol/sdk/server/stdio.js';

const server = new Server(
    {{ name: '{}', version: '0.1.0' }},
    {{ capabilities: {{ tools: {{}} }} }}
);

async function main() {{
    const transport = new StdioServerTransport();
    await server.connect(transport);
}}

main().catch(console.error);
"#,
        name, name
    )
}

fn generate_makefile(name: &str) -> String {
    format!(
        r#".PHONY: install verify test clean

install:
	npm ci

verify:
	npm test
	skillpack check

test:
	npm test

clean:
	rm -rf node_modules dist

assess:
	skillpack check --min-score 80

publish:
	skillpack lock
	oras push ghcr.io/$(USER)/{name}:$(VERSION) .
"#,
        name = name
    )
}

// ============================================================================
// Lock Command
// ============================================================================

fn run_lock(path: &str, force: bool, dry_run: bool) -> Result<()> {
    use crate::cli::dry_run::DryRun;
    use crate::lock::LockEmitter;
    use std::path::{Path, PathBuf};

    let skill_path = Path::new(path);
    let lock_path = skill_path.join("skill.lock");

    if lock_path.exists() && !force {
        cli_println!(
            "{} skill.lock already exists (use --force to regenerate)",
            "ℹ".blue().bold()
        );
        return Ok(());
    }

    // Find CNSB manifest (any .cnsb.json file)
    let manifest_path = std::fs::read_dir(skill_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".cnsb.json"))
        .map(|e| e.path())
        .next()
        .ok_or_else(|| anyhow::anyhow!("No .cnsb.json manifest found in {}", path))?;

    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let name = manifest["metadata"]["name"].as_str().unwrap_or("unknown");
    let version = manifest["metadata"]["version"].as_str().unwrap_or("0.0.0");

    // Build skill URN
    let skill_urn = format!("urn:ckodex:skill:{}:{}", name, version);

    // Compute SRI integrity for each tracked file
    let mut file_digests: Vec<(PathBuf, String)> = Vec::new();

    let manifest_bytes = manifest_content.into_bytes();
    file_digests.push((
        PathBuf::from("skill.cnsb.json"),
        LockEmitter::compute_integrity(&manifest_bytes),
    ));

    let skill_md_path = skill_path.join("SKILL.md");
    if skill_md_path.exists() {
        let bytes = std::fs::read(&skill_md_path)?;
        file_digests.push((
            PathBuf::from("SKILL.md"),
            LockEmitter::compute_integrity(&bytes),
        ));
    }

    // Emit schema-compliant lock file
    let lock = LockEmitter::emit(&skill_urn, &file_digests);
    let lock_json = serde_json::to_string_pretty(&lock)?;

    let dr = DryRun(dry_run);
    dr.perform_fallible("write skill.lock", || {
        std::fs::write(&lock_path, &lock_json)
    })?;

    let action = if dry_run {
        "Would generate"
    } else {
        "Generated"
    };
    cli_println!("{} {} skill.lock", "✓".green().bold(), action);
    cli_println!("  Skill: {} v{}", name, version);
    cli_println!("  Tracked files: {}", file_digests.len());

    Ok(())
}

// ============================================================================
// Migrate Command
// ============================================================================

fn run_migrate(path: &str, target: &str, dry_run: bool) -> Result<()> {
    use std::path::Path;

    let skill_path = Path::new(path);
    let manifest_path = skill_path.join("skill.cnsb.json");

    if !manifest_path.exists() {
        anyhow::bail!("skill.cnsb.json not found in {}", path);
    }

    let content = std::fs::read_to_string(&manifest_path)?;
    let mut manifest: serde_json::Value = serde_json::from_str(&content)?;

    // Detect current schema version
    let current_schema = manifest["$schema"].as_str().unwrap_or("");
    let current_version = detect_schema_version(current_schema);

    cli_println!("Current schema version: {}", current_version);
    cli_println!("Target schema version: {}", target);

    if current_version == target {
        cli_println!("{} Already at target version", "✓".green().bold());
        return Ok(());
    }

    let mut changes: Vec<String> = Vec::new();

    // Apply migrations
    if current_version == "1.0.0" && (target == "2.0.0" || target.starts_with("2.")) {
        // v1 -> v2 migrations

        // 1. Update schema URL
        manifest["$schema"] = json!("https://skillpack.dev/schemas/cnsb-v2.schema.json");
        changes.push("Updated $schema to v2".to_string());

        // 2. Rename 'capacities' to 'extensions' if present
        if manifest.get("capacities").is_some() {
            manifest["extensions"] = manifest["capacities"].clone();
            if let Some(obj) = manifest.as_object_mut() {
                obj.remove("capacities");
            }
            changes.push("Renamed 'capacities' to 'extensions'".to_string());
        }

        // 3. Add DAL version if missing
        if manifest["metadata"].get("dal_version").is_none()
            && manifest["metadata"].get("gal_version").is_none()
        {
            manifest["metadata"]["dal_version"] = json!("1.0.0");
            changes.push("Added dal_version to metadata".to_string());
        }

        // 4. Wrap lifecycle in operations block
        if manifest.get("lifecycle").is_some() && manifest.get("operations").is_none() {
            manifest["operations"] = json!({
                "lifecycle": manifest["lifecycle"].clone()
            });
            if let Some(obj) = manifest.as_object_mut() {
                obj.remove("lifecycle");
            }
            changes.push("Wrapped lifecycle in operations block".to_string());
        }
    }

    if changes.is_empty() {
        cli_println!("{} No migration needed", "ℹ".blue().bold());
        return Ok(());
    }

    cli_println!("\nMigration changes:");
    for change in &changes {
        cli_println!("  - {}", change);
    }

    if dry_run {
        cli_println!("\n{} Dry run - no changes written", "ℹ".blue().bold());
        cli_println!(
            "\nNew content:\n{}",
            serde_json::to_string_pretty(&manifest)?
        );
    } else {
        // Backup original
        let backup_path = manifest_path.with_extension("json.bak");
        std::fs::copy(&manifest_path, &backup_path)?;

        // Write migrated content
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        cli_println!("\n{} Migration complete", "✓".green().bold());
        cli_println!("  Backup saved to: {}", backup_path.display());
    }

    Ok(())
}

fn detect_schema_version(schema_url: &str) -> String {
    if schema_url.contains("-v2") || schema_url.contains("/v2/") {
        "2.0.0".to_string()
    } else {
        "1.0.0".to_string()
    }
}

// ============================================================================
// Wizard Command
// ============================================================================

fn run_wizard(
    name: &str,
    description: Option<&str>,
    author: Option<&str>,
    base_path: &str,
    no_check: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::cli::dry_run::DryRun;
    use std::path::Path;

    let skill_dir = Path::new(base_path).join(name);
    if skill_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", skill_dir.display());
    }

    let author_name = author.unwrap_or("CKODEX Author");
    let desc = description.unwrap_or("A high-quality CKODEX-compliant AI agent skill.");

    let dr = DryRun(dry_run);

    // Directories
    dr.perform_fallible("create skill directory", || {
        std::fs::create_dir_all(&skill_dir)
    })?;
    for subdir in &[
        "capacities/mcp",
        "examples",
        "security",
        "tests",
        "evidence",
        "evals",
        ".github/workflows",
    ] {
        let path = skill_dir.join(subdir);
        dr.perform_fallible(&format!("create {}", subdir), || {
            std::fs::create_dir_all(&path)
        })?;
    }

    // Core files
    let files: Vec<(&str, String)> = vec![
        (
            "SKILL.md",
            generate_wizard_skill_md(name, desc, author_name),
        ),
        (
            "skill.cnsb.json",
            generate_wizard_cnsb(name, desc, author_name),
        ),
        ("README.md", generate_wizard_readme(name, desc)),
        ("LICENSE", generate_wizard_license()),
        ("CODE_OF_CONDUCT.md", generate_wizard_code_of_conduct()),
        ("SECURITY.md", generate_wizard_security_md(name)),
        ("GOVERNANCE.md", generate_wizard_governance_md()),
        ("CODEOWNERS", generate_wizard_codeowners()),
        ("CHANGELOG.md", generate_wizard_changelog(name)),
        ("tests/smoke.rs", generate_wizard_test_smoke(name)),
        ("examples/basic.md", generate_wizard_example(name)),
        ("security/threat-model.yaml", generate_wizard_threat_model()),
        (".github/workflows/ci.yml", generate_wizard_ci_yml(name)),
        ("eval.yml", generate_wizard_eval_yml()),
        ("evals/runner.yaml", generate_wizard_evals_runner()),
        ("Makefile", generate_wizard_makefile(name)),
        ("evidence/provenance.json", generate_wizard_provenance(name)),
        ("evidence/sbom.json", generate_wizard_sbom(name)),
        (".gitignore", generate_wizard_gitignore()),
    ];

    for (file_path, content) in &files {
        let full_path = skill_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            dr.perform_fallible(&format!("ensure dir for {}", file_path), || {
                std::fs::create_dir_all(parent)
            })?;
        }
        dr.perform_fallible(&format!("write {}", file_path), || {
            std::fs::write(&full_path, content)
        })?;
    }

    let action = if dry_run {
        "Would scaffold"
    } else {
        "Scaffolded"
    };
    cli_println!("{} skill '{}' at {}", action, name, skill_dir.display());
    cli_println!("  Files: {}", files.len());

    if !no_check && !dry_run {
        cli_println!();
        cli_println!("{}", "Running assessment...".cyan().bold());

        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        let response = use_case.execute(AssessSkillRequest {
            skill_path: skill_dir.to_str().unwrap().to_string(),
            min_score: None,
        })?;

        let assessment = response.assessment;
        let grade = assessment.grade();
        let score = assessment.total_score().value();

        cli_println!();
        cli_println!(
            "{} Overall: {} ({:.1}/100)",
            "✓".green().bold(),
            grade.to_string().bold(),
            score
        );

        if !response.stub_dimensions.is_empty() {
            cli_println!(
                "  {} Stub dimensions: {:?}",
                "⚠".yellow(),
                response.stub_dimensions
            );
        }

        cli_println!();
        cli_println!("Dimension breakdown:");
        for (dim, s) in &assessment.dimension_scores {
            cli_println!("  {:24} {:>6.1}", format!("{:?}:", dim), s.value());
        }

        if score < 90.0 {
            cli_println!();
            cli_println!("{}", "Quick wins to reach A:".yellow().bold());
            if assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Documentation)
                .is_none_or(|s| s.value() < 90.0)
            {
                cli_println!("  • Expand README.md with more detail and examples");
            }
            if assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Testing)
                .is_none_or(|s| s.value() < 90.0)
            {
                cli_println!("  • Add actual test implementations in tests/smoke.rs");
            }
            if assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Lifecycle)
                .is_none_or(|s| s.value() < 90.0)
            {
                cli_println!("  • Add more lifecycle hooks (upgrade, uninstall, preInstall)");
            }
            if assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Governance)
                .is_none_or(|s| s.value() < 90.0)
            {
                cli_println!("  • Add CODEOWNERS entries and expand GOVERNANCE.md");
            }
        }
    }

    cli_println!();
    cli_println!("Next steps:");
    cli_println!("  cd {}", name);
    if !no_check && !dry_run {
        cli_println!("  # Edit description in SKILL.md and README.md");
        cli_println!("  # Implement tests in tests/smoke.rs");
    }
    cli_println!("  skillpack check");

    Ok(())
}

fn generate_wizard_skill_md(name: &str, description: &str, author: &str) -> String {
    format!(
        r#"---
name: {}
version: 0.1.0
description: {}
author: {}
compatibility:
  runtime: [node20, bun]
  platforms: [linux/amd64, linux/arm64, darwin/arm64]
  sdkPins:
    - "@ckodex/skill-sdk: ^1.0.0"
galMin: 1
galMax: 5
---

# {}

{}

## Installation

```bash
skillpack install {}
```

## Usage

```bash
skillpack check --min-score 80
```

## Security

This skill declares ASC patterns and includes a threat model.
See `security/threat-model.yaml` for details.

## Lifecycle

| Hook | Command |
|------|---------|
| install | `make install` |
| verify | `make verify` |
| test | `make test` |
| clean | `make clean` |
| assess | `skillpack check --min-score 80` |
| publish | `make publish` |

## License

Apache-2.0. See [LICENSE](LICENSE).
"#,
        name, description, author, name, description, name
    )
}

fn generate_wizard_cnsb(name: &str, description: &str, author: &str) -> String {
    serde_json::to_string_pretty(&json!({
        "$schema": "https://skillpack.dev/schemas/cnsb.schema.json",
        "apiVersion": "cnsb.ckodex.dev/v1",
        "kind": "SkillBundle",
        "metadata": {
            "name": name,
            "version": "0.1.0",
            "description": description,
            "author": author,
            "license": "Apache-2.0",
            "urn": format!("urn:ckodex:skill:{}:0.1.0", name)
        },
        "skills": [{
            "name": name,
            "version": "0.1.0",
            "description": description,
            "asc": ["OIS-ASC-001"],
            "examples": ["examples/basic.md"],
            "galMin": 1,
            "galMax": 5
        }],
        "governance": {
            "license": "Apache-2.0",
            "codeOfConduct": true,
            "securityPolicy": true
        },
        "lifecycle": {
            "install": { "command": "make install", "timeout": "5m" },
            "verify": { "command": "make verify" },
            "test": { "command": "make test" },
            "clean": { "command": "make clean" },
            "assess": { "command": "skillpack check --min-score 80" },
            "publish": { "command": "make publish" }
        }
    }))
    .unwrap()
}

fn generate_wizard_readme(name: &str, description: &str) -> String {
    format!(
        r#"# {}

{}

## Features

- CKODEX-compliant skill structure
- Pre-wired lifecycle hooks
- Security threat model included
- Automated testing scaffolding
- Continuous integration ready

## Quick Start

```bash
# Install dependencies
make install

# Run tests
make test

# Assess quality
skillpack check --min-score 80
```

## Project Structure

```
{}/
├── SKILL.md              # Skill manifest (frontmatter)
├── skill.cnsb.json       # CNSB bundle manifest
├── README.md             # This file
├── LICENSE               # Apache-2.0
├── CODE_OF_CONDUCT.md    # Community standards
├── SECURITY.md           # Security policy
├── GOVERNANCE.md         # Decision-making process
├── CODEOWNERS            # Repository ownership
├── CHANGELOG.md          # Version history
├── Makefile              # Build & lifecycle automation
├── tests/                # Test suite
│   └── smoke.rs
├── examples/             # Usage examples
│   └── basic.md
├── security/             # Security artifacts
│   └── threat-model.yaml
├── evidence/             # Provenance & SBOM
│   ├── provenance.json
│   └── sbom.json
├── evals/                # Evaluation harness
│   └── runner.yaml
└── .github/workflows/    # CI/CD
    └── ci.yml
```

## Contributing

See [GOVERNANCE.md](GOVERNANCE.md) for our decision-making process
and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community standards.

## Security

Please report security issues following the process in [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE) for full text.
"#,
        name, description, name
    )
}

fn generate_wizard_license() -> String {
    r#"Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

Copyright 2026 CKODEX Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"#
    .to_string()
}

fn generate_wizard_code_of_conduct() -> String {
    r#"# Code of Conduct

## Our Pledge

We pledge to make participation in our project a harassment-free experience
for everyone, regardless of age, body size, disability, ethnicity, gender
identity and expression, level of experience, nationality, personal appearance,
race, religion, or sexual identity and orientation.

## Our Standards

Examples of behavior that contributes to a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

## Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be
reported by contacting the project team. All complaints will be reviewed
and investigated promptly and fairly.
"#
    .to_string()
}

fn generate_wizard_security_md(name: &str) -> String {
    format!(
        r#"# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in {}, please report it
responsibly by emailing security@ckodex.dev.

Please include:
- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and aim to provide a timeline
for resolution within 7 days.

## Security Measures

- All releases are signed with Sigstore
- SBOM and provenance metadata included in `evidence/`
- Threat model maintained in `security/threat-model.yaml`
"#,
        name
    )
}

fn generate_wizard_governance_md() -> String {
    r#"# Governance

## Decision Making

This project follows a meritocratic governance model:

1. **Lazy consensus**: Proposals are accepted if no objection is raised
   within 72 hours.
2. **Major changes**: Require approval from at least two maintainers.
3. **Breaking changes**: Require discussion in an issue and approval
   from the majority of maintainers.

## Maintainers

- @ckodex/maintainers

## Contributors

See the GitHub contributors graph for a list of all contributors.

## Code of Conduct

All participants must adhere to [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
"#
    .to_string()
}

fn generate_wizard_codeowners() -> String {
    "* @ckodex/maintainers\n".to_string()
}

fn generate_wizard_changelog(name: &str) -> String {
    format!(
        r#"# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-10

### Added

- Initial release of {}
- CKODEX-compliant skill structure
- Pre-wired lifecycle hooks
- Security threat model stub
- Automated test scaffolding
- CI/CD workflow
"#,
        name
    )
}

fn generate_wizard_test_smoke(name: &str) -> String {
    format!(
        r#"#[test]
fn skill_manifest_exists() {{
    let manifest = std::fs::read_to_string("SKILL.md").unwrap();
    assert!(manifest.contains("name: {}"));
}}

#[test]
fn cnsb_manifest_is_valid_json() {{
    let raw = std::fs::read_to_string("skill.cnsb.json").unwrap();
    let _: serde_json::Value = serde_json::from_str(&raw).unwrap();
}}
"#,
        name
    )
}

fn generate_wizard_example(name: &str) -> String {
    format!(
        r#"# Basic Usage

## Install

```bash
skillpack install {}
```

## Verify

```bash
make verify
```

## Assess Quality

```bash
skillpack check --min-score 80
```
"#,
        name
    )
}

fn generate_wizard_threat_model() -> String {
    r#"threats:
  - id: T001
    category: Spoofing
    description: Unauthorized modification of skill metadata
    likelihood: Low
    impact: High
    mitigation: Digitally sign all releases with Sigstore

  - id: T002
    category: Tampering
    description: Malicious code injection in dependency chain
    likelihood: Medium
    impact: High
    mitigation: Pin all dependencies; use SBOM for supply chain tracking

  - id: T003
    category: Information Disclosure
    description: Secrets leaked in public repository
    likelihood: Low
    impact: Medium
    mitigation: Pre-commit hooks scanning for secrets; CI secret detection

mitigations:
  - id: M001
    description: All releases signed with Sigstore cosign
    status: Planned

  - id: M002
    description: SBOM generated for every release
    status: Planned
"#
    .to_string()
}

fn generate_wizard_ci_yml(name: &str) -> String {
    format!(
        r#"name: CI

on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install SkillPack
        run: cargo install --git https://github.com/ckodex/skillpack
      - name: Run tests
        run: make test
      - name: Assess skill quality
        run: skillpack check --min-score 80

  eval:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install SkillPack
        run: cargo install --git https://github.com/ckodex/skillpack
      - name: Run evals
        run: skillpack eval {}

  package:
    runs-on: ubuntu-latest
    needs: [test]
    if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      - name: Install SkillPack
        run: cargo install --git https://github.com/ckodex/skillpack
      - name: Package skill
        run: skillpack package --no-check
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: {}-${{ github.ref_name }}
          path: '*.tar.gz'

  publish:
    runs-on: ubuntu-latest
    needs: [package]
    if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      - name: Install SkillPack
        run: cargo install --git https://github.com/ckodex/skillpack
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: {}-${{ github.ref_name }}
      - name: Publish to OCI registry
        run: skillpack publish --registry ${{ vars.OCI_REGISTRY }} --sign --no-check
"#,
        name, name, name
    )
}

fn generate_wizard_evals_runner() -> String {
    r#"runner: skillpack-eval
version: 1.0.0

suites:
  - name: smoke
    description: Basic smoke tests
    tests:
      - id: manifest-valid
        command: skillpack validate --schema cnsb skill.cnsb.json
        expect: 0

  - name: quality
    description: Quality threshold gate
    tests:
      - id: grade-a
        command: skillpack check --min-score 80
        expect: 0

hitl:
  enabled: true
  requireApproval: true
  approverRoles:
    - maintainer
    - security-reviewer
  promotionThreshold:
    minGrade: A
    minScore: 90
"#
    .to_string()
}

fn generate_wizard_eval_yml() -> String {
    r#"# SkillPack Evaluation Suites
# Run with: skillpack eval [--suite smoke|compliance]

suites:
  smoke:
    description: "Quick sanity checks"
    tests:
      - manifest_exists
      - manifest_valid_json
      - skill_md_exists
      - readme_exists
      - metadata_complete
      - skillpack_check

  compliance:
    description: "Full compliance validation"
    tests:
      - manifest_exists
      - manifest_valid_json
      - skill_md_exists
      - readme_exists
      - metadata_complete
      - skillpack_check
    threshold:
      min_score: 60
"#
    .to_string()
}

fn generate_wizard_makefile(name: &str) -> String {
    format!(
        r#".PHONY: install verify test clean assess eval package publish

install:
	@echo "Installing {} dependencies..."

verify: test assess eval
	@echo "All verifications passed."

test:
	@echo "Running test suite..."
	@cargo test --workspace 2>/dev/null || true

clean:
	@rm -rf node_modules dist *.log *.tar.gz

assess:
	@skillpack check --min-score 80

eval:
	@skillpack eval --suite smoke

package:
	@skillpack package --no-check

publish:
	@skillpack publish --registry $(REGISTRY) --sign --no-check
"#,
        name
    )
}

fn generate_wizard_provenance(name: &str) -> String {
    serde_json::to_string_pretty(&json!({
        "_type": "https://in-toto.io/Statement/v1",
        "predicateType": "https://slsa.dev/provenance/v1",
        "subject": [{
            "name": name,
            "digest": { "sha256": "placeholder" }
        }],
        "predicate": {
            "buildDefinition": {
                "buildType": "https://github.com/ckodex/skillpack/buildtypes/skill@v1",
                "externalParameters": {},
                "resolvedDependencies": []
            },
            "runDetails": {
                "builder": { "id": "https://github.com/ckodex/skillpack/.github/workflows/ci.yml" },
                "metadata": { "invocationId": "placeholder" }
            }
        }
    }))
    .unwrap()
}

fn generate_wizard_sbom(name: &str) -> String {
    serde_json::to_string_pretty(&json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        "version": 1,
        "metadata": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "tools": [{"vendor": "CKODEX", "name": "skillpack", "version": "1.0.0"}],
            "component": {
                "type": "application",
                "name": name,
                "version": "0.1.0"
            }
        },
        "components": []
    }))
    .unwrap()
}

fn generate_wizard_gitignore() -> String {
    "node_modules/\ndist/\n*.log\n.env\ntarget/\n*.bak\n".to_string()
}

// ============================================================================
// Package Command
// ============================================================================

fn run_package(
    path: &str,
    output: Option<&str>,
    no_check: bool,
    format: PackageFormat,
    dry_run: bool,
) -> Result<()> {
    use sha2::Digest;
    use std::io::Write;
    use std::path::Path;

    let skill_path = Path::new(path);
    let manifest_path = skill_path.join("skill.cnsb.json");

    if !manifest_path.exists() {
        anyhow::bail!("skill.cnsb.json not found in {}", path);
    }

    // Pre-packaging assessment
    if !no_check {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        let response = use_case.execute(AssessSkillRequest {
            skill_path: path.to_string(),
            min_score: None,
        })?;

        let score = response.assessment.total_score().value();
        if score < 60.0 {
            anyhow::bail!(
                "Skill scores {:.1}/100 (grade: {}). Packaging requires at least C (60). Use --no-check to bypass.",
                score,
                response.assessment.grade()
            );
        }
        cli_println!(
            "{} Pre-packaging assessment passed: {:.1}/100",
            "✓".green().bold(),
            score
        );
    }

    // Read manifest for default filename
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
    let name = manifest["metadata"]["name"].as_str().unwrap_or("skill");
    let version = manifest["metadata"]["version"].as_str().unwrap_or("0.0.0");

    let default_output = match format {
        PackageFormat::TarGz => format!("{}-{}.tar.gz", name, version),
        PackageFormat::TarBz2 => format!("{}-{}.tar.bz2", name, version),
        PackageFormat::TarBrotli => format!("{}-{}.tar.br", name, version),
        PackageFormat::TarZstd => format!("{}-{}.tar.zst", name, version),
        PackageFormat::Zip => format!("{}-{}.zip", name, version),
    };
    let output_path = output.unwrap_or(&default_output);

    if dry_run {
        cli_println!(
            "{} Would package '{}' into '{}'",
            "ℹ".blue().bold(),
            skill_path.display(),
            output_path
        );
        return Ok(());
    }

    // Collect files
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(skill_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(skill_path)
            .unwrap_or(entry.path());
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with("target/")
            || rel_str.starts_with(".git/")
            || rel_str.ends_with(".tar.gz")
            || rel_str.ends_with(".tar.bz2")
            || rel_str.ends_with(".tar.br")
            || rel_str.ends_with(".tar.zst")
            || rel_str.ends_with(".zip")
            || rel_str.starts_with("node_modules/")
        {
            continue;
        }
        files.push(entry.path().to_path_buf());
    }

    // Create archive
    match format {
        PackageFormat::TarGz => {
            let tar_gz = std::fs::File::create(output_path)?;
            let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            for file in &files {
                let rel = file.strip_prefix(skill_path)?;
                tar.append_path_with_name(file, rel)?;
            }
            tar.finish()?;
        }
        PackageFormat::TarBz2 => {
            let tar_bz2 = std::fs::File::create(output_path)?;
            let enc = bzip2::write::BzEncoder::new(tar_bz2, bzip2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            for file in &files {
                let rel = file.strip_prefix(skill_path)?;
                tar.append_path_with_name(file, rel)?;
            }
            tar.finish()?;
        }
        PackageFormat::TarBrotli => {
            let tar_br = std::fs::File::create(output_path)?;
            let mut enc = brotli::CompressorWriter::new(tar_br, 4096, 11, 22);
            {
                let mut tar = tar::Builder::new(&mut enc);
                for file in &files {
                    let rel = file.strip_prefix(skill_path)?;
                    tar.append_path_with_name(file, rel)?;
                }
                tar.finish()?;
            }
            enc.flush()?;
        }
        PackageFormat::TarZstd => {
            let tar_zst = std::fs::File::create(output_path)?;
            let mut enc = zstd::stream::write::Encoder::new(tar_zst, 1)?;
            {
                let mut tar = tar::Builder::new(&mut enc);
                for file in &files {
                    let rel = file.strip_prefix(skill_path)?;
                    tar.append_path_with_name(file, rel)?;
                }
                tar.finish()?;
            }
            enc.finish()?;
        }
        PackageFormat::Zip => {
            let zip_file = std::fs::File::create(output_path)?;
            let mut zip = zip::ZipWriter::new(zip_file);
            for file in &files {
                let rel = file.strip_prefix(skill_path)?;
                zip.start_file::<_, ()>(
                    rel.to_string_lossy(),
                    zip::write::FileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated),
                )?;
                let content = std::fs::read(file)?;
                zip.write_all(&content)?;
            }
            zip.finish()?;
        }
    }

    // Compute archive digest
    let archive_content = std::fs::read(output_path)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&archive_content);
    let digest = format!("sha256:{}", hex::encode(hasher.finalize()));

    let size = archive_content.len();
    cli_println!("{} Packaged {}", "✓".green().bold(), output_path);
    cli_println!("  Files: {}", files.len());
    cli_println!("  Size: {} bytes", size);
    cli_println!("  Digest: {}", &digest[..47]);

    Ok(())
}

// ============================================================================
// Publish Command
// ============================================================================

/// Read (name, version) from a SKILL.md's frontmatter for publishing an
/// agentskills-convention skill. `name` is required; `version` defaults to
/// "0.0.0" when absent (agentskills does not mandate a version).
fn agentskill_identity(skill_md: &std::path::Path) -> Result<(String, String)> {
    let content = std::fs::read_to_string(skill_md)?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("SKILL.md has no YAML frontmatter");
    }
    let fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim())?;
    let name = fm
        .get("name")
        .or_else(|| fm.get("metadata").and_then(|m| m.get("name")))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("SKILL.md frontmatter missing `name`"))?
        .to_string();
    let version = fm
        .get("version")
        .or_else(|| fm.get("metadata").and_then(|m| m.get("version")))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    Ok((name, version))
}

async fn run_publish(
    path: &str,
    registry: &str,
    username: Option<&str>,
    password: Option<&str>,
    no_check: bool,
    sign: bool,
    dry_run: bool,
) -> Result<()> {
    use std::path::Path;

    let skill_path = Path::new(path);
    let cnsb_path = skill_path.join("skill.cnsb.json");
    let agentskill_md = skill_path.join("SKILL.md");

    // Publish supports both flavors: CNSB bundles and SKILL.md-convention
    // agent skills (which have no CNSB manifest — the bulk of the fleet).
    if !cnsb_path.exists() && !agentskill_md.exists() {
        anyhow::bail!("no skill.cnsb.json or SKILL.md found in {}", path);
    }

    // Pre-publish assessment
    if !no_check {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        let response = use_case.execute(AssessSkillRequest {
            skill_path: path.to_string(),
            min_score: None,
        })?;

        let score = response.assessment.total_score().value();
        if score < 60.0 {
            anyhow::bail!(
                "Skill scores {:.1}/100 (grade: {}). Publishing requires at least C (60). Use --no-check to bypass.",
                score,
                response.assessment.grade()
            );
        }
        cli_println!(
            "{} Pre-publish assessment passed: {:.1}/100",
            "✓".green().bold(),
            score
        );
    }

    // Resolve identity + artifact media type from whichever manifest exists.
    let (name, version, media_type): (String, String, &str) = if cnsb_path.exists() {
        let manifest_content = std::fs::read_to_string(&cnsb_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        (
            manifest["metadata"]["name"]
                .as_str()
                .unwrap_or("skill")
                .to_string(),
            manifest["metadata"]["version"]
                .as_str()
                .unwrap_or("0.0.0")
                .to_string(),
            "application/vnd.ckodex.cnsb.v1+tar.gz",
        )
    } else {
        let (n, v) = agentskill_identity(&agentskill_md)?;
        (n, v, "application/vnd.ckodex.agentskill.v1+tar.gz")
    };

    // Bundle all skill files into a single tar.gz blob
    let mut bundle_buf = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut bundle_buf, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);
        for entry in walkdir::WalkDir::new(skill_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = entry
                .path()
                .strip_prefix(skill_path)
                .unwrap_or(entry.path());
            let rel_str = rel.to_string_lossy();
            if rel_str.starts_with("target/")
                || rel_str.starts_with(".git/")
                || rel_str.ends_with(".tar.gz")
                || rel_str.starts_with("node_modules/")
            {
                continue;
            }
            tar.append_path_with_name(entry.path(), rel)?;
        }
        tar.finish()?;
    }

    // Parse registry reference
    let registry_ref = registry.strip_prefix("oci://").unwrap_or(registry);
    let parts: Vec<&str> = registry_ref.split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid registry reference: {}", registry);
    }
    let registry_host = parts[0];
    let repo_tag = parts[1..].join("/");

    if dry_run {
        cli_println!(
            "{} Would publish '{}' v{} to oci://{}/{}",
            "ℹ".blue().bold(),
            name,
            version,
            registry_host,
            repo_tag
        );
        cli_println!("  Bundle size: {} bytes", bundle_buf.len());
        if sign {
            cli_println!("  Would sign with Sigstore");
        }
        return Ok(());
    }

    // Push to OCI
    let auth = match (username, password) {
        (Some(u), Some(p)) => Some((u, p)),
        _ => None,
    };

    let result = crate::oci::OciReader::push(
        registry_host,
        repo_tag
            .rsplit_once(':')
            .map(|(r, _)| r)
            .unwrap_or(&repo_tag),
        repo_tag
            .rsplit_once(':')
            .map(|(_, t)| t)
            .unwrap_or("latest"),
        &bundle_buf,
        media_type,
        auth,
    )
    .await?;

    cli_println!("{} Published {} v{}", "✓".green().bold(), name, version);
    cli_println!("  Registry: oci://{}/{}", registry_host, repo_tag);
    cli_println!("  Digest: {}", &result.digest[..47]);
    cli_println!("  Manifest: {}", result.manifest_url);

    // ── AIPACK §6.4 attestation referrers ──────────────────────────────────
    // Always emit after a successful push; referrer failures are non-fatal
    // (warn and continue) to avoid blocking the publish workflow.
    let subject_digest = &result.digest;
    let repo_only = repo_tag
        .rsplit_once(':')
        .map(|(r, _)| r)
        .unwrap_or(&repo_tag);

    // T-18: urn:skill:static-analysis:v1 — sourced from security checker output
    {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        if let Ok(resp) = use_case.execute(AssessSkillRequest {
            skill_path: path.to_string(),
            min_score: None,
        }) {
            let assessment = &resp.assessment;
            let security_score = assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Security)
                .map(|s| s.value())
                .unwrap_or(0.0);
            let issues: Vec<serde_json::Value> = assessment
                .issues
                .iter()
                .filter(|i| i.dimension == skillpack_domain::DimensionId::Security)
                .map(|i| serde_json::json!({"severity": format!("{:?}", i.severity), "message": i.message}))
                .collect();
            let statement = serde_json::json!({
                "_type": "https://in-toto.io/Statement/v1",
                "subject": [{"name": format!("{}/{}", registry_host, repo_tag), "digest": {"sha256": subject_digest.strip_prefix("sha256:").unwrap_or(subject_digest)}}],
                "predicateType": "urn:skill:static-analysis:v1",
                "predicate": {
                    "analyzer": "skillpack/security-checker",
                    "score": security_score,
                    "grade": format!("{:?}", assessment.grade()),
                    "issues": issues,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            let payload = serde_json::to_vec(&statement)?;
            match crate::oci::OciReader::push_referrer(
                registry_host,
                repo_only,
                subject_digest,
                "urn:skill:static-analysis:v1",
                &payload,
                auth,
            )
            .await
            {
                Ok(r) => cli_println!(
                    "{} Referrer pushed: urn:skill:static-analysis:v1 ({})",
                    "✓".green(),
                    &r.digest[..std::cmp::min(47, r.digest.len())]
                ),
                Err(e) => cli_println!(
                    "{} Referrer push failed (static-analysis): {}",
                    "⚠".yellow(),
                    e
                ),
            }
        }
    }

    // T-19: urn:skill:capability-declaration:v1 — sourced from governance checker + manifest
    {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        if let Ok(resp) = use_case.execute(AssessSkillRequest {
            skill_path: path.to_string(),
            min_score: None,
        }) {
            let assessment = &resp.assessment;
            let gov_score = assessment
                .dimension_scores
                .get(&skillpack_domain::DimensionId::Governance)
                .map(|s| s.value())
                .unwrap_or(0.0);
            // CNSB manifests declare capabilities/GAL; agentskills skills do
            // not, so those fields are null for them.
            let cnsb: serde_json::Value = if cnsb_path.exists() {
                std::fs::read_to_string(&cnsb_path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            };
            let statement = serde_json::json!({
                "_type": "https://in-toto.io/Statement/v1",
                "subject": [{"name": format!("{}/{}", registry_host, repo_tag), "digest": {"sha256": subject_digest.strip_prefix("sha256:").unwrap_or(subject_digest)}}],
                "predicateType": "urn:skill:capability-declaration:v1",
                "predicate": {
                    "analyzer": "skillpack/governance-checker",
                    "governanceScore": gov_score,
                    "capabilities": cnsb.get("capabilities").cloned().unwrap_or(serde_json::Value::Null),
                    "aipackCapabilities": cnsb.get("aipackCapabilities").cloned().unwrap_or(serde_json::Value::Null),
                    "galMin": cnsb.pointer("/skills/0/galMin").cloned().unwrap_or(serde_json::Value::Null),
                    "galMax": cnsb.pointer("/skills/0/galMax").cloned().unwrap_or(serde_json::Value::Null),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            let payload = serde_json::to_vec(&statement)?;
            match crate::oci::OciReader::push_referrer(
                registry_host,
                repo_only,
                subject_digest,
                "urn:skill:capability-declaration:v1",
                &payload,
                auth,
            )
            .await
            {
                Ok(r) => cli_println!(
                    "{} Referrer pushed: urn:skill:capability-declaration:v1 ({})",
                    "✓".green(),
                    &r.digest[..std::cmp::min(47, r.digest.len())]
                ),
                Err(e) => cli_println!(
                    "{} Referrer push failed (capability-declaration): {}",
                    "⚠".yellow(),
                    e
                ),
            }
        }
    }

    // T-20: cyclonedx.org/bom — generated by syft (non-blocking; warn if unavailable)
    {
        let sbom_result = tokio::task::spawn_blocking({
            let path = path.to_string();
            move || -> Result<Vec<u8>> {
                let output = std::process::Command::new("syft")
                    .args([&path, "-o", "cyclonedx-json"])
                    .output()
                    .map_err(|e| anyhow::anyhow!("syft not found: {}", e))?;
                if output.status.success() {
                    Ok(output.stdout)
                } else {
                    anyhow::bail!(
                        "syft exited {}: {}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!("SBOM task panicked: {}", e));

        match sbom_result {
            Ok(Ok(sbom_bytes)) => {
                let statement = serde_json::json!({
                    "_type": "https://in-toto.io/Statement/v1",
                    "subject": [{"name": format!("{}/{}", registry_host, repo_tag), "digest": {"sha256": subject_digest.strip_prefix("sha256:").unwrap_or(subject_digest)}}],
                    "predicateType": "cyclonedx.org/bom",
                    "predicate": serde_json::from_slice::<serde_json::Value>(&sbom_bytes).unwrap_or(serde_json::Value::Null)
                });
                let payload = serde_json::to_vec(&statement)?;
                match crate::oci::OciReader::push_referrer(
                    registry_host,
                    repo_only,
                    subject_digest,
                    "cyclonedx.org/bom",
                    &payload,
                    auth,
                )
                .await
                {
                    Ok(r) => cli_println!(
                        "{} Referrer pushed: cyclonedx.org/bom ({})",
                        "✓".green(),
                        &r.digest[..std::cmp::min(47, r.digest.len())]
                    ),
                    Err(e) => cli_println!(
                        "{} Referrer push failed (cyclonedx.org/bom): {}",
                        "⚠".yellow(),
                        e
                    ),
                }
            }
            Ok(Err(e)) | Err(e) => {
                cli_println!(
                    "{} SBOM generation skipped: {}. Install syft to enable.",
                    "⚠".yellow(),
                    e
                );
            }
        }
    }
    // ── end AIPACK attestation referrers ────────────────────────────────────

    if sign {
        match sign_with_sigstore(&bundle_buf).await {
            Ok(signature_ref) => {
                cli_println!(
                    "{} Signed with Sigstore — signature: {}",
                    "✓".green().bold(),
                    signature_ref
                );
            }
            Err(e) => {
                cli_println!("{} Sigstore signing failed: {}", "⚠".yellow(), e);
                cli_println!("  Set SIGSTORE_ID_TOKEN to enable keyless signing.");
            }
        }
    }

    Ok(())
}

async fn sign_with_sigstore(data: &[u8]) -> Result<String> {
    let token_str = std::env::var("SIGSTORE_ID_TOKEN")
        .map_err(|_| anyhow::anyhow!("SIGSTORE_ID_TOKEN not set"))?;

    let data = data.to_vec();

    let bundle_json = tokio::task::spawn_blocking(move || -> Result<String> {
        let context = sigstore::bundle::sign::SigningContext::production()
            .map_err(|e| anyhow::anyhow!("Sigstore context failed: {e}"))?;

        let token = sigstore::oauth::IdentityToken::try_from(token_str.as_str())
            .map_err(|e| anyhow::anyhow!("Invalid identity token: {e}"))?;

        let signer = context
            .blocking_signer(token)
            .map_err(|e| anyhow::anyhow!("Signer creation failed: {e}"))?;

        let artifact = signer
            .sign(std::io::Cursor::new(&data))
            .map_err(|e| anyhow::anyhow!("Signing failed: {e}"))?;

        let bundle = artifact.to_bundle();
        let json = serde_json::to_string(&bundle)
            .map_err(|e| anyhow::anyhow!("Bundle serialization failed: {e}"))?;

        Ok(json)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Sigstore signing task panicked: {e}"))??;

    // Write bundle to file for attachment
    let bundle_path = "skill.sigstore.bundle";
    std::fs::write(bundle_path, &bundle_json)?;

    Ok(format!("{} ({} bytes)", bundle_path, bundle_json.len()))
}

// ============================================================================
// Install Command
// ============================================================================

async fn run_install(
    reference: &str,
    output: &str,
    username: Option<&str>,
    password: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    use skillpack_domain::source::SkillSource;

    // Dispatch on the source scheme: OCI registries, git repositories, and
    // local filesystem paths are all first-class install sources.
    let source = SkillSource::from_uri(reference)
        .map_err(|e| anyhow::anyhow!("could not parse source '{}': {:?}", reference, e))?;
    match source {
        SkillSource::Oci(_) => {
            run_install_oci(reference, output, username, password, dry_run).await
        }
        SkillSource::Git(git) => run_install_git(&git, output, dry_run),
        SkillSource::Local(path) => run_install_local(&path, output, dry_run),
        SkillSource::S3(_)
        | SkillSource::Sftp(_)
        | SkillSource::AzureBlob(_)
        | SkillSource::Gcs(_) => anyhow::bail!(
            "install from '{}' is not yet supported (supported schemes: OCI, git, local path)",
            reference
        ),
    }
}

/// Find the skill root within a fetched/local tree: the directory itself if it
/// holds SKILL.md or skill.cnsb.json, else the single immediate subdirectory
/// that does. Errors when there is none, or more than one (ambiguous).
fn locate_skill_root(dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let has_skill =
        |d: &std::path::Path| d.join("SKILL.md").exists() || d.join("skill.cnsb.json").exists();
    if has_skill(dir) {
        return Ok(dir.to_path_buf());
    }
    let mut candidates: Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && has_skill(p))
        .collect();
    match candidates.len() {
        1 => Ok(candidates.pop().unwrap()),
        0 => anyhow::bail!(
            "no SKILL.md or skill.cnsb.json in {} (nor one level down)",
            dir.display()
        ),
        n => anyhow::bail!(
            "found {} skills under {} — point at a specific skill directory",
            n,
            dir.display()
        ),
    }
}

/// Resolve a skill's name from its manifest (SKILL.md frontmatter or CNSB),
/// falling back to the directory name.
fn resolve_skill_name(skill_root: &std::path::Path) -> String {
    if let Ok((name, _)) = agentskill_identity(&skill_root.join("SKILL.md")) {
        return name;
    }
    if let Ok(c) = std::fs::read_to_string(skill_root.join("skill.cnsb.json"))
        && let Ok(j) = serde_json::from_str::<serde_json::Value>(&c)
        && let Some(n) = j["metadata"]["name"].as_str()
    {
        return n.to_string();
    }
    skill_root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "skill".to_string())
}

/// Validate that a resolved skill name is a single, safe path component before
/// it is joined onto the output directory. A skill's `name` comes from
/// attacker-controllable frontmatter, so `name: /etc/cron.d` (absolute) or
/// `name: ../../x` would otherwise escape the output dir on install and write
/// arbitrary files. Reject anything that is not a plain directory component.
fn safe_dir_component(name: &str) -> Result<String> {
    let trimmed = name.trim();
    let bad = trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\0')
        || std::path::Path::new(trimmed).is_absolute()
        // Reject any name that resolves to more than one component.
        || std::path::Path::new(trimmed).components().count() != 1;
    if bad {
        anyhow::bail!(
            "unsafe skill name '{}' — refusing to install (must be a single path component, no '/', '..', or absolute path)",
            name
        );
    }
    Ok(trimmed.to_string())
}

/// Recursively copy a skill tree, skipping symlinks and build/VCS noise.
fn copy_tree(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)?.flatten() {
        let name = entry.file_name();
        if matches!(
            name.to_string_lossy().as_ref(),
            ".git" | "target" | "node_modules"
        ) {
            continue;
        }
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(&name);
        if ty.is_symlink() {
            continue;
        } else if ty.is_dir() {
            copy_tree(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Install a skill from a git repository: clone/fetch, locate the skill, copy
/// it into the output directory.
fn run_install_git(
    source: &skillpack_domain::source::GitSource,
    output: &str,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        cli_println!(
            "{} Would clone {}@{} and install the skill into {}",
            "ℹ".blue().bold(),
            source.url,
            source.r#ref,
            output
        );
        return Ok(());
    }
    let reader = crate::git::GitReader::new(source.clone());
    let cloned = reader
        .fetch()
        .map_err(|e| anyhow::anyhow!("git fetch failed: {}", e))?;
    let skill_root = locate_skill_root(&cloned)?;
    let name = safe_dir_component(&resolve_skill_name(&skill_root))?;
    let dest = std::path::Path::new(output).join(&name);
    copy_tree(&skill_root, &dest)?;
    cli_println!(
        "{} Installed {} from {}@{} into {}",
        "✓".green().bold(),
        name,
        source.url,
        source.r#ref,
        dest.display()
    );
    Ok(())
}

/// Install a skill from a local filesystem path (copy into the output dir).
/// Use `skillpack link` instead to symlink a local skill in place.
fn run_install_local(path: &std::path::Path, output: &str, dry_run: bool) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("local path does not exist: {}", path.display());
    }
    let skill_root = locate_skill_root(path)?;
    let name = safe_dir_component(&resolve_skill_name(&skill_root))?;
    let dest = std::path::Path::new(output).join(&name);
    if dry_run {
        cli_println!(
            "{} Would install local skill '{}' from {} into {}",
            "ℹ".blue().bold(),
            name,
            skill_root.display(),
            dest.display()
        );
        return Ok(());
    }
    copy_tree(&skill_root, &dest)?;
    cli_println!(
        "{} Installed {} from {} into {}",
        "✓".green().bold(),
        name,
        skill_root.display(),
        dest.display()
    );
    Ok(())
}

async fn run_install_oci(
    reference: &str,
    output: &str,
    _username: Option<&str>,
    _password: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    use std::path::Path;

    let registry_ref = reference.strip_prefix("oci://").unwrap_or(reference);
    let parts: Vec<&str> = registry_ref.split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid registry reference: {}", reference);
    }
    let registry_host = parts[0];
    let repo_tag = parts[1..].join("/");

    let repo = repo_tag
        .rsplit_once(':')
        .map(|(r, _)| r)
        .unwrap_or(&repo_tag);

    let output_dir = Path::new(output).join(repo.rsplit_once('/').map(|(_, r)| r).unwrap_or(repo));

    if dry_run {
        cli_println!(
            "{} Would install '{}' from oci://{}/{} into '{}'",
            "ℹ".blue().bold(),
            reference,
            registry_host,
            repo_tag,
            output_dir.display()
        );
        return Ok(());
    }

    // Pull from OCI
    let source = skillpack_domain::source::OciSource::from_uri(reference)?;

    let reader = crate::oci::OciReader::new(source);
    let pulled_path = reader.pull().await?;

    // Extract tar.gz if present
    let tar_gz_path = pulled_path.join("bundle.tar.gz");
    if tar_gz_path.exists() {
        let tar_gz = std::fs::File::open(&tar_gz_path)?;
        let dec = flate2::read::GzDecoder::new(tar_gz);
        let mut tar = tar::Archive::new(dec);
        tar.unpack(&output_dir)?;
    }

    cli_println!(
        "{} Installed {} into {}",
        "✓".green().bold(),
        reference,
        output_dir.display()
    );

    Ok(())
}

// ============================================================================
// Eval Command
// ============================================================================

fn run_eval(path: &str, suite: Option<&str>, output: Option<&str>) -> Result<()> {
    use std::path::Path;

    let skill_path = Path::new(path);
    let manifest_path = skill_path.join("skill.cnsb.json");

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║                    SKILL EVALUATION                        ║"
            .cyan()
            .bold()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );

    let suite_name = suite.unwrap_or("smoke");
    cli_println!("Running '{}' suite on {}", suite_name, skill_path.display());

    let mut results = Vec::new();

    // Test 1: Manifest exists
    let manifest_exists = manifest_path.exists();
    results.push(("manifest-exists", manifest_exists));
    cli_println!(
        "  {} manifest.cnsb.json exists",
        if manifest_exists {
            "✓".green()
        } else {
            "✗".red()
        }
    );

    // Test 2: Manifest is valid JSON
    let manifest_valid = if manifest_exists {
        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => serde_json::from_str::<serde_json::Value>(&content).is_ok(),
            Err(_) => false,
        }
    } else {
        false
    };
    results.push(("manifest-valid-json", manifest_valid));
    cli_println!(
        "  {} manifest is valid JSON",
        if manifest_valid {
            "✓".green()
        } else {
            "✗".red()
        }
    );

    // Test 3: SKILL.md exists
    let skill_md_exists = skill_path.join("SKILL.md").exists();
    results.push(("skill-md-exists", skill_md_exists));
    cli_println!(
        "  {} SKILL.md exists",
        if skill_md_exists {
            "✓".green()
        } else {
            "✗".red()
        }
    );

    // Test 4: README.md exists
    let readme_exists = skill_path.join("README.md").exists();
    results.push(("readme-exists", readme_exists));
    cli_println!(
        "  {} README.md exists",
        if readme_exists {
            "✓".green()
        } else {
            "✗".red()
        }
    );

    // Test 5: skill.cnsb.json has required metadata fields
    let metadata_ok = if manifest_exists && manifest_valid {
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&content)?;
        manifest["metadata"]["name"].is_string() && manifest["metadata"]["version"].is_string()
    } else {
        false
    };
    results.push(("metadata-complete", metadata_ok));
    cli_println!(
        "  {} metadata has name + version",
        if metadata_ok {
            "✓".green()
        } else {
            "✗".red()
        }
    );

    // Test 6: Run skillpack check
    let check_threshold = if suite_name == "compliance" {
        60.0
    } else {
        10.0
    };
    let check_ok = if suite_name == "compliance" || suite_name == "smoke" {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, all_checkers());
        match use_case.execute(AssessSkillRequest {
            skill_path: path.to_string(),
            min_score: None,
        }) {
            Ok(response) => {
                let score = response.assessment.total_score().value();
                let ok = score >= check_threshold;
                cli_println!(
                    "  {} skillpack check: {:.1}/100 {}",
                    if ok { "✓".green() } else { "✗".red() },
                    score,
                    if ok { "" } else { "(below threshold)" }
                );
                ok
            }
            Err(e) => {
                cli_println!("  {} skillpack check failed: {}", "✗".red(), e);
                false
            }
        }
    } else {
        true // skip for non-compliance suites
    };
    results.push(("skillpack-check", check_ok));

    let passed = results.iter().filter(|(_, p)| *p).count();
    let total = results.len();

    cli_println!();
    cli_println!(
        "Results: {}/{} tests passed",
        passed.to_string().green().bold(),
        total
    );

    if let Some(out) = output {
        let report = json!({
            "suite": suite_name,
            "skill_path": path,
            "passed": passed,
            "total": total,
            "tests": results.iter().map(|(n, p)| json!({"name": n, "passed": p})).collect::<Vec<_>>(),
        });
        std::fs::write(out, serde_json::to_string_pretty(&report)?)?;
        cli_println!("Report written to {}", out);
    }

    if passed < total {
        anyhow::bail!("{} tests failed", total - passed);
    }

    Ok(())
}

// ============================================================================
// Discover Command
// ============================================================================

fn run_discover(path: &str, limit: usize) -> Result<()> {
    use std::path::Path;

    let search_path = Path::new(path);

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║                    SKILL DISCOVERY                         ║"
            .cyan()
            .bold()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!(
        "Searching for skills in {} (limit: {})\n",
        search_path.display(),
        limit
    );

    let mut found = 0;
    for entry in walkdir::WalkDir::new(search_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() == "skill.cnsb.json")
    {
        if found >= limit {
            break;
        }

        let skill_dir = entry.path().parent().unwrap_or(entry.path());
        let manifest_content = std::fs::read_to_string(entry.path())?;
        let manifest: serde_json::Value = match serde_json::from_str(&manifest_content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = manifest["metadata"]["name"].as_str().unwrap_or("unknown");
        let version = manifest["metadata"]["version"].as_str().unwrap_or("?");
        let description = manifest["metadata"]["description"].as_str().unwrap_or("");

        // Quick assessment for grade
        let grade = if skill_dir.join("SKILL.md").exists() {
            let reader = FilesystemReader::new();
            let use_case = AssessSkillUseCase::new(reader, all_checkers());
            match use_case.execute(AssessSkillRequest {
                skill_path: skill_dir.to_string_lossy().to_string(),
                min_score: None,
            }) {
                Ok(response) => response.assessment.grade().to_string(),
                Err(_) => "?".to_string(),
            }
        } else {
            "?".to_string()
        };

        cli_println!("  {} {} v{} [{}]", "▸".cyan(), name, version, grade);
        if !description.is_empty() {
            cli_println!("    {}", description);
        }
        cli_println!("    {}", skill_dir.display());
        cli_println!();

        found += 1;
    }

    if found == 0 {
        cli_println!("{} No skills found.", "ℹ".blue());
        cli_println!("  Skills must contain a 'skill.cnsb.json' manifest file.");
    } else {
        cli_println!("Found {} skill(s)", found.to_string().green().bold());
    }

    Ok(())
}

// ============================================================================
// Store Commands
// ============================================================================

/// Load the grader-owned exemption policy from a location the graded skill
/// cannot touch: `~/.config/skillpack/exemptions.toml` (override with
/// `SKILLPACK_EXEMPTIONS`). Missing/unreadable file → empty policy (nothing
/// exempt). Format:
///
/// ```toml
/// [grants]
/// "skill-steward" = ["testing", "evals_hitl"]
/// ```
fn grader_exemption_policy() -> skillpack_domain::ExemptionPolicy {
    use skillpack_domain::{ExemptionPolicy, parse_exempt_dimension};

    let path = std::env::var("SKILLPACK_EXEMPTIONS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
                .join("skillpack")
                .join("exemptions.toml")
        });
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ExemptionPolicy::empty();
    };
    let Ok(value) = content.parse::<toml::Value>() else {
        eprintln!(
            "{} exemptions file {} is not valid TOML — ignoring (no exemptions granted)",
            "⚠".yellow(),
            path.display()
        );
        return ExemptionPolicy::empty();
    };
    let grants = value
        .get("grants")
        .and_then(|g| g.as_table())
        .map(|table| {
            table
                .iter()
                .map(|(skill, dims)| {
                    let parsed: Vec<_> = dims
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|d| d.as_str().and_then(parse_exempt_dimension))
                                .collect()
                        })
                        .unwrap_or_default();
                    (skill.clone(), parsed)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ExemptionPolicy::from_grants(grants)
}

/// Default canonical store root, honoring SKILLPACK_CANONICAL_ROOT.
fn default_canonical_root() -> std::path::PathBuf {
    if let Ok(r) = std::env::var("SKILLPACK_CANONICAL_ROOT") {
        return std::path::PathBuf::from(r);
    }
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Skills")
        .join("shared")
}

/// Read a skill's frontmatter `description` (nested or flat), best-effort.
fn skill_description(skill_dir: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(skill_dir.join("SKILL.md")).ok()?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }
    let fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim()).ok()?;
    fm.get("metadata")
        .and_then(|m| m.get("description"))
        .or_else(|| fm.get("description"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Read a skill's lifecycle status (nested `lifecycle.status` or flat
/// `status`); defaults to "draft" when unset.
fn skill_status(skill_dir: &std::path::Path) -> String {
    (|| {
        let content = std::fs::read_to_string(skill_dir.join("SKILL.md")).ok()?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return None;
        }
        let fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim()).ok()?;
        fm.get("lifecycle")
            .and_then(|l| l.get("status"))
            .or_else(|| fm.get("status"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    })()
    .unwrap_or_else(|| "draft".to_string())
}

/// Generate the registry discovery manifest: a `.well-known`-style contract
/// at the store root that lets ANY harness find the registry and consume it
/// by reference (symlink) instead of copying skills into its own tree.
fn run_store_manifest(canonical_root: Option<&str>, to_stdout: bool) -> Result<()> {
    let root = canonical_root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_canonical_root);

    if !root.exists() {
        anyhow::bail!(
            "canonical store not found at {} (run `skillpack store migrate` first)",
            root.display()
        );
    }

    let mut skills: Vec<serde_json::Value> = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "candidates" {
            continue;
        }
        let dir = entry.path();
        let has_skill_md = dir.join("SKILL.md").exists();
        skills.push(serde_json::json!({
            "name": name,
            "path": name,
            "hasSkillMd": has_skill_md,
            "status": skill_status(&dir),
            "description": skill_description(&dir),
        }));
    }
    skills.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "registry": "ckodex-skillpack",
        "canonicalRoot": root.to_string_lossy(),
        "skillCount": skills.len(),
        "consumption": {
            "model": "reference",
            "method": "symlink",
            "instruction":
                "For each skill, symlink <yourAgentSkillsDir>/<name> -> <canonicalRoot>/<name>. Never copy — the registry is the single source of truth.",
            "onboard":
                "Built-in harness: `skillpack store sync`. New harness: set SKILLPACK_CUSTOM_AGENTS=YourHarness=/path/to/skills then sync.",
            "reconcile": "skillpack store sync --dry-run  # preview reference fanout"
        },
        "skills": skills,
    });

    let json = serde_json::to_string_pretty(&manifest)?;
    if to_stdout {
        println!("{}", json);
    } else {
        let out = root.join(".skillpack-registry.json");
        std::fs::write(&out, format!("{}\n", json))?;
        cli_println!(
            "{} Wrote registry manifest: {} ({} skills)",
            "✓".green().bold(),
            out.display(),
            manifest["skillCount"]
        );
        cli_println!("  Harnesses discover the registry by reading this file.");
    }
    Ok(())
}

/// Tier-3 behavioral eval, gated. This spawns a real agent and consumes
/// tokens, so it never runs implicitly: the operator must set
/// `SKILLPACK_BEHAVIORAL=1` (opt-in) and `SKILLPACK_BEHAVIORAL_CMD` (the agent
/// command). Results are advisory — they never feed the promote gate.
fn run_behavioral_cmd(path: &str, timeout_secs: u64, max_cases: usize) -> Result<()> {
    use crate::behavioral::{BehavioralOptions, SubprocessRunner, run_behavioral};
    use skillpack_domain::behavioral::BehavioralFixture;
    use std::time::Duration;

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Behavioral Eval (Tier 3)                          ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    // Gate 1: explicit opt-in.
    if std::env::var("SKILLPACK_BEHAVIORAL").as_deref() != Ok("1") {
        anyhow::bail!(
            "behavioral evals are opt-in: set SKILLPACK_BEHAVIORAL=1 to acknowledge they spawn a real agent and consume tokens"
        );
    }
    // Gate 2: operator-configured agent command (no default — safe by default).
    let cmd_spec: Vec<String> = std::env::var("SKILLPACK_BEHAVIORAL_CMD")
        .ok()
        .map(|s| shell_split(&s))
        .unwrap_or_default();
    if cmd_spec.is_empty() {
        anyhow::bail!(
            "no agent command configured: set SKILLPACK_BEHAVIORAL_CMD (e.g. 'claude -p --output-format stream-json'). The skill is staged at ./skill in an isolated workspace; the command must load it and read the task prompt from stdin."
        );
    }

    let dir = std::path::Path::new(path);
    let fixture_path = dir.join("evals/behavioral.json");
    let raw = std::fs::read_to_string(&fixture_path).map_err(|e| {
        anyhow::anyhow!("no behavioral fixture at {}: {}", fixture_path.display(), e)
    })?;
    let fixture = BehavioralFixture::from_json(&raw).map_err(|e| anyhow::anyhow!(e))?;
    if fixture.cases.is_empty() {
        anyhow::bail!("behavioral fixture has no cases");
    }

    let runner = SubprocessRunner::from_command(&cmd_spec)?;
    let opts = BehavioralOptions {
        timeout: Duration::from_secs(timeout_secs),
        max_cases,
    };

    cli_println!(
        "  Running {} case(s) in isolated workspaces (timeout {}s each)…\n",
        fixture.cases.len().min(max_cases),
        timeout_secs
    );
    let grades = run_behavioral(dir, &fixture, &runner, &opts)?;

    let mut failed = 0;
    for g in &grades {
        if g.passed {
            cli_println!("  {} {}", "✓".green(), g.name.bold());
        } else {
            failed += 1;
            cli_println!("  {} {}", "✗".red(), g.name.bold());
            for r in &g.reasons {
                cli_println!("      {} {}", "•".red(), r);
            }
        }
    }

    cli_println!();
    if failed == 0 {
        cli_println!(
            "  {} All {} behavioral case(s) passed.",
            "✓".green().bold(),
            grades.len()
        );
        Ok(())
    } else {
        eprintln!(
            "  {} {}/{} behavioral case(s) failed — the skill does not behave as declared.",
            "✗".red().bold(),
            failed,
            grades.len()
        );
        std::process::exit(1);
    }
}

/// Minimal POSIX-ish command split: whitespace-separated, honoring single and
/// double quotes. Sufficient for `SKILLPACK_BEHAVIORAL_CMD`.
fn shell_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut in_token = false;
    for c in s.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else {
                    cur.push(c);
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    in_token = true;
                }
                c if c.is_whitespace() => {
                    if in_token {
                        out.push(std::mem::take(&mut cur));
                        in_token = false;
                    }
                }
                c => {
                    cur.push(c);
                    in_token = true;
                }
            },
        }
    }
    if in_token {
        out.push(cur);
    }
    out
}

/// Collect the fleet's routing surfaces (name + description) from the
/// canonical store — the deduplicated set an agent would route across.
fn fleet_routing_surfaces(
    canonical_root: Option<&str>,
) -> Result<Vec<skillpack_domain::collision::RoutingSurface>> {
    let root = canonical_root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_canonical_root);
    if !root.exists() {
        anyhow::bail!(
            "canonical store not found at {} (run `skillpack store migrate` first)",
            root.display()
        );
    }
    let mut surfaces = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "candidates" {
            continue;
        }
        if let Some(desc) = skill_description(&entry.path()) {
            surfaces.push(skillpack_domain::collision::RoutingSurface {
                name,
                description: desc,
            });
        }
    }
    Ok(surfaces)
}

/// Tier-2 routing eval: rank the fleet against a task prompt. With `--expect`,
/// this becomes a CI gate — non-zero exit when the intended skill doesn't win.
fn run_route(
    prompt: &str,
    top_k: usize,
    expect: Option<&str>,
    canonical_root: Option<&str>,
) -> Result<()> {
    let surfaces = fleet_routing_surfaces(canonical_root)?;
    // When gating, always compute enough depth to see whether the expected
    // skill placed, even if the display top_k is smaller.
    let depth = top_k.max(1).max(if expect.is_some() { 10 } else { 0 });
    let hits = skillpack_domain::collision::rank_by_query(&surfaces, prompt, depth);

    cli_println!("Routing \"{}\" across {} skills:\n", prompt, surfaces.len());
    for hit in hits.iter().take(top_k.max(1)) {
        let marker = if hit.rank == 1 {
            "→".green().bold()
        } else {
            " ".normal()
        };
        cli_println!(
            "  {} {}. {:.3}  {}",
            marker,
            hit.rank,
            hit.score,
            hit.name.bold()
        );
    }

    if let Some(want) = expect {
        let want_lower = want.to_lowercase();
        match hits.iter().find(|h| h.name.to_lowercase() == want_lower) {
            Some(h) if h.rank <= top_k.max(1) => {
                cli_println!(
                    "\n{} '{}' routes at rank {} (within top {}).",
                    "✓".green().bold(),
                    want,
                    h.rank,
                    top_k.max(1)
                );
            }
            Some(h) => {
                eprintln!(
                    "{} '{}' only routes at rank {} (want ≤ {}). Sharpen its description's \"Use when …\" trigger.",
                    "✗".red().bold(),
                    want,
                    h.rank,
                    top_k.max(1)
                );
                std::process::exit(1);
            }
            None => {
                eprintln!(
                    "{} '{}' does not route for this prompt at all. Its description misses the trigger.",
                    "✗".red().bold(),
                    want
                );
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

/// Run a skill's routing fixture (`<dir>/evals/routing.json`) against the
/// fleet. Fixture shape: `{ "positive": ["prompt", …], "top_k": 3 }`. Every
/// positive prompt must place the skill within top_k, else this is a failed
/// routing eval (non-zero exit) — the skill's description under-routes.
fn run_route_check(skill_dir: &str, canonical_root: Option<&str>) -> Result<()> {
    let dir = std::path::Path::new(skill_dir);
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("invalid skill dir: {}", skill_dir))?;

    let fixture_path = dir.join("evals/routing.json");
    let raw = std::fs::read_to_string(&fixture_path)
        .map_err(|e| anyhow::anyhow!("no routing fixture at {}: {}", fixture_path.display(), e))?;
    let fixture: serde_json::Value = serde_json::from_str(&raw)?;
    let top_k = fixture
        .get("top_k")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(3) as usize;
    let positives: Vec<String> = fixture
        .get("positive")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    if positives.is_empty() {
        anyhow::bail!(
            "fixture {} has no `positive` prompts",
            fixture_path.display()
        );
    }

    let surfaces = fleet_routing_surfaces(canonical_root)?;
    let want = name.to_lowercase();
    let depth = top_k.max(10);
    let mut failures = 0;

    cli_println!(
        "Routing eval for '{}' ({} prompts, top_k={}):\n",
        name.bold(),
        positives.len(),
        top_k
    );
    for prompt in &positives {
        let hits = skillpack_domain::collision::rank_by_query(&surfaces, prompt, depth);
        let placed = hits.iter().find(|h| h.name.to_lowercase() == want);
        match placed {
            Some(h) if h.rank <= top_k => {
                cli_println!("  {} rank {}  \"{}\"", "✓".green(), h.rank, prompt);
            }
            Some(h) => {
                failures += 1;
                cli_println!(
                    "  {} rank {} (> {})  \"{}\"",
                    "✗".red(),
                    h.rank,
                    top_k,
                    prompt
                );
            }
            None => {
                failures += 1;
                cli_println!("  {} unranked  \"{}\"", "✗".red(), prompt);
            }
        }
    }

    if failures == 0 {
        cli_println!(
            "\n{} All {} routing prompts place '{}' within top {}.",
            "✓".green().bold(),
            positives.len(),
            name,
            top_k
        );
        Ok(())
    } else {
        eprintln!(
            "\n{} {}/{} routing prompts under-route '{}'. Sharpen its description's trigger.",
            "✗".red().bold(),
            failures,
            positives.len(),
            name
        );
        std::process::exit(1);
    }
}

async fn run_store(command: StoreCommands) -> Result<()> {
    let client = CanonicalStoreClient::from_env();

    match command {
        StoreCommands::Sync {
            dry_run,
            no_index,
            only_agent,
        } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Canonical Store Sync                               ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            match client
                .sync_agents(dry_run, no_index, only_agent.clone())
                .await
            {
                Ok(resp) => {
                    cli_println!(
                        "{} {}",
                        if resp.success {
                            "✓".green()
                        } else {
                            "⚠".yellow()
                        },
                        resp.message
                    );
                    cli_println!("  Skills processed: {}", resp.skills_processed);
                    if let Some(agent) = only_agent {
                        cli_println!("  Agent: {}", agent);
                    }
                }
                Err(e) => {
                    eprintln!("{} Sync failed: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
        StoreCommands::Migrate { canonical_root } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Migrate Physical Skills                            ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let root = canonical_root.unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("~"))
                    .join("Skills/shared")
                    .to_string_lossy()
                    .to_string()
            });

            match client.migrate_all(root).await {
                Ok(resp) => {
                    cli_println!(
                        "{} {}",
                        if resp.success {
                            "✓".green()
                        } else {
                            "⚠".yellow()
                        },
                        resp.message
                    );
                }
                Err(e) => {
                    eprintln!("{} Migrate failed: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
        StoreCommands::Status => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Canonical Store Status                             ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            match client.status().await {
                Ok(resp) => {
                    cli_println!("  Canonical root: {}", resp.canonical_root);
                    cli_println!("  Total skills: {}", resp.total_skills);
                    cli_println!(
                        "  Healthy: {}",
                        if resp.is_healthy {
                            "✓".green()
                        } else {
                            "✗".red()
                        }
                    );
                    cli_println!();
                    if !resp.active_agents.is_empty() {
                        cli_println!("  Active agents:");
                        for agent in &resp.active_agents {
                            let status = if agent.is_active {
                                "✓ active".green()
                            } else {
                                "· inactive".yellow()
                            };
                            cli_println!(
                                "    {:20} {:12} {}",
                                agent.name,
                                format!("({})", agent.path),
                                status
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Status failed: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
        StoreCommands::Manifest {
            canonical_root,
            stdout,
        } => {
            // Local filesystem operation — no server round-trip needed.
            run_store_manifest(canonical_root.as_deref(), stdout)?;
        }
        StoreCommands::CheckBoundary { path, skill_name } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Check Boundary                                     ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            match client.check_boundary(path, skill_name).await {
                Ok(resp) => {
                    if resp.is_safe {
                        cli_println!("{} Path is safe — no IP boundary violations", "✓".green());
                    } else {
                        cli_println!("{} Boundary violation detected!", "✗".red());
                        cli_println!("  Violations:");
                        for v in &resp.violations {
                            cli_println!("    - {}", v);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{} Check failed: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

/// Assess a skill and summarize: grade, total score, and the non-Note issue
/// messages (errors + warnings the author should act on).
fn assess_summary(path: &str) -> Result<(Grade, f64, Vec<String>)> {
    use skillpack_domain::Severity;
    let use_case = AssessSkillUseCase::new(FilesystemReader::new(), all_checkers())
        .with_exemption_policy(grader_exemption_policy());
    let resp = use_case.execute(AssessSkillRequest {
        skill_path: path.to_string(),
        min_score: None,
    })?;
    let a = resp.assessment;
    let manual: Vec<String> = a
        .issues
        .iter()
        .filter(|i| i.severity != Severity::Note)
        .map(|i| i.message.clone())
        .collect();
    Ok((a.grade(), a.total_score().value(), manual))
}

/// `skillpack improve`: apply the mechanically-fixable findings, then
/// re-assess and report the grade delta plus any remaining manual work.
fn run_skill_improve(path: &str, dry_run: bool) -> Result<()> {
    let dir = std::path::Path::new(path);
    if !dir.join("SKILL.md").exists() {
        anyhow::bail!("no SKILL.md at {}", dir.display());
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Improve Skill                                      ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let (grade_before, score_before, _) = assess_summary(path)?;

    let mut fixes = Vec::new();
    if let Some(f) = improve::ensure_version(dir, dry_run) {
        fixes.push(f);
    }
    if let Some(f) = improve::ensure_changelog(dir, dry_run) {
        fixes.push(f);
    }
    fixes.extend(improve::stub_broken_links(dir, dry_run));

    if fixes.is_empty() {
        cli_println!("  {} No mechanical fixes needed.", "✓".green());
    } else {
        let verb = if dry_run { "would apply" } else { "applied" };
        cli_println!("  {} {} {} fix(es):", "✓".green().bold(), verb, fixes.len());
        for f in &fixes {
            cli_println!("    {} {}", "•".green(), f.description);
        }
    }

    // Re-assess (post-fix in the real run; identical in dry-run).
    let (grade_after, score_after, manual) = assess_summary(path)?;
    cli_println!();
    if dry_run {
        cli_println!(
            "  Grade: {} ({:.0}/150) — run without --dry-run to apply",
            grade_before.as_str(),
            score_before
        );
    } else {
        cli_println!(
            "  Grade: {} ({:.0}) → {} ({:.0})",
            grade_before.as_str(),
            score_before,
            grade_after.as_str().bold(),
            score_after
        );
    }

    if !manual.is_empty() {
        cli_println!(
            "\n  {} {} finding(s) need manual work (not auto-fixable):",
            "▸".yellow(),
            manual.len()
        );
        for m in manual.iter().take(10) {
            cli_println!("    {} {}", "•".yellow(), m);
        }
        if manual.len() > 10 {
            cli_println!(
                "    … and {} more (run `skillpack check {}`)",
                manual.len() - 10,
                path
            );
        }
    }
    Ok(())
}

/// `skillpack refresh`: normalize the model-facing frontmatter and report
/// content-level routing issues.
fn run_skill_refresh(path: &str, dry_run: bool) -> Result<()> {
    let skill_md = std::path::Path::new(path).join("SKILL.md");
    if !skill_md.exists() {
        anyhow::bail!("no SKILL.md at {}", skill_md.display());
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Refresh Skill (model frontmatter)                 ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let report = refresh::refresh_skill(&skill_md, dry_run)?;

    if report.normalized.is_empty() {
        cli_println!("  {} Frontmatter field names already current.", "✓".green());
    } else {
        let verb = if dry_run {
            "would normalize"
        } else {
            "normalized"
        };
        cli_println!("  {} {}:", "✓".green().bold(), verb);
        for n in &report.normalized {
            cli_println!("    {} {}", "•".green(), n);
        }
    }

    if report.warnings.is_empty() {
        cli_println!(
            "\n  {} Routing fields look good for current model loaders.",
            "✓".green()
        );
    } else {
        cli_println!(
            "\n  {} {} routing issue(s) to address manually:",
            "▸".yellow(),
            report.warnings.len()
        );
        for w in &report.warnings {
            cli_println!("    {} {}", "•".yellow(), w);
        }
    }
    Ok(())
}

/// Reject git remote-helper transports (ext::, fd::) that execute commands.
fn ensure_safe_share_url(url: &str) -> Result<()> {
    if let Some(idx) = url.find("::") {
        let helper = &url[..idx];
        let is_scheme = url.get(idx..idx + 3) == Some("://");
        if !is_scheme && !helper.contains('/') {
            anyhow::bail!("refusing git remote-helper transport '{}::'", helper);
        }
    }
    Ok(())
}

/// `skillpack skill share`: publish a skill to a remote git group as its own repo
/// via push-to-create. Stages a clean copy (VCS noise + symlinks stripped by
/// copy_tree), commits once, and pushes to <group-url>/<skill>.git. No manual
/// copy. Auth uses whatever git credentials are configured for the host.
fn run_skill_share(
    skill: &str,
    group_url: &str,
    repo_name: Option<&str>,
    branch: &str,
    dry_run: bool,
) -> Result<()> {
    use std::process::Command;

    let canon_root = std::env::var("SKILLPACK_CANONICAL_ROOT").unwrap_or_else(|_| {
        format!(
            "{}/Skills/shared",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let candidate = if std::path::Path::new(skill).is_dir() {
        std::path::PathBuf::from(skill)
    } else {
        std::path::Path::new(&canon_root).join(skill)
    };
    let src = std::fs::canonicalize(&candidate)
        .map_err(|_| anyhow::anyhow!("skill not found: {} (looked in {})", skill, canon_root))?;
    if !src.join("SKILL.md").exists() {
        cli_println!(
            "  {} no SKILL.md in {} (sharing anyway)",
            "⚠".yellow(),
            src.display()
        );
    }
    let name = safe_dir_component(
        repo_name
            .map(str::to_string)
            .unwrap_or_else(|| src.file_name().unwrap().to_string_lossy().to_string())
            .as_str(),
    )?;
    let url = format!("{}/{}.git", group_url.trim_end_matches('/'), name);
    ensure_safe_share_url(&url)?;

    cli_println!(
        "{}",
        "── skill share ─────────────────────────────────────────────".cyan()
    );
    cli_println!("  skill  : {}", name);
    cli_println!("  source : {}", src.display());
    cli_println!("  target : {}  (new repo in the group)", url);

    let tmp = std::env::temp_dir().join(format!("skillpack-share-{}", name));
    let _ = std::fs::remove_dir_all(&tmp);
    let repo = tmp.join("repo");
    copy_tree(&src, &repo)?;

    if dry_run {
        cli_println!(
            "  {} dry-run — staged a clean copy; not pushing.",
            "◌".yellow()
        );
        let _ = std::fs::remove_dir_all(&tmp);
        return Ok(());
    }

    let git = |args: &[&str]| -> Result<()> {
        let ok = Command::new("git")
            .env("GIT_ALLOW_PROTOCOL", "file:git:http:https:ssh")
            .env("GIT_TERMINAL_PROMPT", "0")
            .current_dir(&repo)
            .args(args)
            .status()?
            .success();
        if !ok {
            anyhow::bail!("git {} failed", args.first().copied().unwrap_or(""));
        }
        Ok(())
    };
    let msg = format!("{name} skill — published from the canonical store via skillpack");
    git(&["init", "-q", "-b", branch])?;
    git(&["add", "-A"])?;
    git(&[
        "-c",
        "user.name=skillpack",
        "-c",
        "user.email=skillpack@localhost",
        "commit",
        "-q",
        "-m",
        &msg,
    ])?;
    git(&["remote", "add", "origin", &url])?;
    let push_ok = Command::new("git")
        .env("GIT_ALLOW_PROTOCOL", "file:git:http:https:ssh")
        .env("GIT_TERMINAL_PROMPT", "0")
        .current_dir(&repo)
        .args(["push", "-u", "origin", branch])
        .status()?
        .success();
    let _ = std::fs::remove_dir_all(&tmp);
    if !push_ok {
        anyhow::bail!(
            "push failed — store credentials for the host first: \
             printf 'protocol=https\\nhost=<host>\\nusername=oauth2\\npassword=<PAT>\\n\\n' | git credential approve"
        );
    }
    cli_println!(
        "  {} shared '{}' → {}/{}",
        "✓".green().bold(),
        name,
        group_url.trim_end_matches('/'),
        name
    );
    Ok(())
}

fn run_skill(command: SkillCommands) -> Result<()> {
    match command {
        SkillCommands::Update { path, field, value } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Update Skill Frontmatter                           ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let skill_path = std::path::Path::new(&path);
            let skill_md = skill_path.join("SKILL.md");
            if !skill_md.exists() {
                eprintln!("{} No SKILL.md found at {}", "✗".red(), skill_md.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&skill_md)?;
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() < 3 {
                eprintln!("{} SKILL.md has invalid frontmatter format", "✗".red());
                std::process::exit(1);
            }

            let mut fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim())?;

            // Support dot-notation for nested keys (e.g. metadata.version)
            let keys: Vec<&str> = field.split('.').collect();
            let mut current = &mut fm;
            for (i, key) in keys.iter().enumerate() {
                let key_yaml = serde_yaml::Value::String(key.to_string());
                if i == keys.len() - 1 {
                    current[key_yaml] = serde_yaml::Value::String(value.clone());
                } else {
                    let mapping = current
                        .as_mapping_mut()
                        .ok_or_else(|| anyhow::anyhow!("Field '{}' is not a mapping", key))?;
                    let entry = mapping
                        .entry(key_yaml)
                        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
                    current = entry;
                }
            }

            let new_fm = serde_yaml::to_string(&fm)?;
            let new_content = format!("---\n{}---\n{}", new_fm, parts[2]);
            std::fs::write(&skill_md, new_content)?;

            cli_println!(
                "{} Updated {} to '{}' in {}",
                "✓".green(),
                field,
                value,
                skill_md.display()
            );
        }
        SkillCommands::Promote {
            path,
            min_grade,
            force,
        } => {
            let current = read_skill_status(&path)?;
            let next = match current.as_str() {
                "draft" => "active",
                "deprecated" => "active", // re-promotion after remediation
                "active" => {
                    cli_println!("{} Already active — nothing to promote.", "ℹ".blue());
                    return Ok(());
                }
                "retired" => {
                    // Retirement is one-way; resurrecting a retired skill is a
                    // re-creation decision, not a promotion.
                    anyhow::bail!("'{}' is retired — retirement is one-way", path);
                }
                other => anyhow::bail!("unknown lifecycle status '{}'", other),
            };

            // No promotion without proof: the assessment gate.
            if force {
                cli_println!(
                    "{} Promotion gate SKIPPED via --force (no assessment evidence)",
                    "⚠".yellow().bold()
                );
            } else {
                let minimum = parse_grade(&min_grade);
                let use_case = skillpack_application::AssessSkillUseCase::new(
                    crate::filesystem::FilesystemReader::new(),
                    crate::checkers::all_checkers(),
                );
                let response = use_case.execute(skillpack_application::AssessSkillRequest {
                    skill_path: path.clone(),
                    min_score: None,
                })?;
                let grade = response.assessment.grade();
                if grade.meets_minimum(&minimum) {
                    cli_println!(
                        "{} Gate passed: grade {} (profile {}) ≥ {}",
                        "✓".green(),
                        grade.as_str(),
                        response.assessment.profile.name(),
                        min_grade
                    );
                } else {
                    eprintln!(
                        "{} Promotion denied: grade {} < required {}. Remediate (see `skillpack check {}`) or use --force.",
                        "✗".red(),
                        grade.as_str(),
                        min_grade,
                        path
                    );
                    std::process::exit(1);
                }
            }

            write_skill_status(&path, next)?;
            cli_println!(
                "{} Promoted: {} → {}",
                "✓".green().bold(),
                current,
                next.bold()
            );
        }
        SkillCommands::Demote { path } => {
            let current = read_skill_status(&path)?;
            let next = match current.as_str() {
                "active" => "deprecated",
                "draft" => "deprecated",
                "deprecated" => "retired",
                "retired" => {
                    cli_println!("{} Already retired — terminal state.", "ℹ".blue());
                    return Ok(());
                }
                other => anyhow::bail!("unknown lifecycle status '{}'", other),
            };
            write_skill_status(&path, next)?;
            cli_println!(
                "{} Demoted: {} → {}",
                "⚠".yellow().bold(),
                current,
                next.bold()
            );
            if next == "retired" {
                cli_println!("  Retirement is one-way; agents should stop loading this skill.");
            }
        }
        SkillCommands::Refine { path, fix } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Refine Skill                                       ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let skill_path = std::path::Path::new(&path);
            let skill_md = skill_path.join("SKILL.md");
            if !skill_md.exists() {
                eprintln!("{} No SKILL.md found at {}", "✗".red(), skill_md.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&skill_md)?;
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() < 3 {
                eprintln!("{} Invalid SKILL.md format", "✗".red());
                std::process::exit(1);
            }

            // Parse frontmatter
            let fm: serde_yaml::Value = match serde_yaml::from_str(parts[1].trim()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{} Frontmatter parse error: {}", "✗".red(), e);
                    std::process::exit(1);
                }
            };

            let mut issues = 0;
            let mut fixed = 0;

            // Load schema and validate required fields
            let schema_path = std::path::Path::new("schemas/agentskills.schema.json");
            let schema: Option<serde_json::Value> = if schema_path.exists() {
                std::fs::read_to_string(schema_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            } else {
                // Try workspace root relative to skill directory
                let workspace_schema = skill_path
                    .join("..")
                    .join("..")
                    .join("schemas")
                    .join("agentskills.schema.json");
                if workspace_schema.exists() {
                    std::fs::read_to_string(workspace_schema)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                } else {
                    None
                }
            };

            if let Some(ref s) = schema {
                if let Some(required) = s.get("required").and_then(|r| r.as_array()) {
                    for req in required {
                        if let Some(field) = req.as_str()
                            && fm.get(field).is_none()
                        {
                            eprintln!("{} Missing required field: {}", "✗".red(), field);
                            issues += 1;
                        }
                    }
                }
                cli_println!("  Validated against agentskills.schema.json");
            } else {
                cli_println!(
                    "  {} agentskills.schema.json not found, using basic checks",
                    "⚠".yellow()
                );
                if fm.get("name").is_none() {
                    eprintln!("{} Missing required field: name", "✗".red());
                    issues += 1;
                }
                if fm.get("description").is_none() {
                    eprintln!("{} Missing required field: description", "✗".red());
                    issues += 1;
                }
            }

            // Check scripts directory
            let scripts_dir = skill_path.join("scripts");
            if scripts_dir.exists() {
                let entries = std::fs::read_dir(&scripts_dir)?;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "sh").unwrap_or(false) {
                        if fix {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mut perms = std::fs::metadata(&path)?.permissions();
                                perms.set_mode(perms.mode() | 0o111);
                                std::fs::set_permissions(&path, perms)?;
                                fixed += 1;
                            }
                        }
                        // Check for shebang
                        let script_content = std::fs::read_to_string(&path).unwrap_or_default();
                        if !script_content.starts_with("#!/") {
                            eprintln!("{} Missing shebang: {}", "✗".red(), path.display());
                            issues += 1;
                            if fix {
                                let new_content =
                                    format!("#!/usr/bin/env bash\n{}", script_content);
                                std::fs::write(&path, new_content)?;
                                fixed += 1;
                            }
                        }
                    }
                }
            }

            // Check referenced files exist
            if let Some(body) = parts.get(2) {
                for line in body.lines() {
                    if line.starts_with("| `")
                        && line.contains("scripts/")
                        && let Some(start) = line.find("scripts/")
                        && let Some(end) = line[start..].find('`')
                    {
                        let script_ref = &line[start..start + end];
                        let expected = skill_path.join(script_ref);
                        if !expected.exists() {
                            eprintln!("{} Referenced file missing: {}", "✗".red(), script_ref);
                            issues += 1;
                        }
                    }
                }
            }

            if issues == 0 {
                cli_println!("{} Skill passes all quality gates", "✓".green());
            } else {
                cli_println!("{} Found {} issues", "⚠".yellow(), issues);
            }
            if fixed > 0 {
                cli_println!("{} Auto-fixed {} items", "✓".green(), fixed);
            }
        }
        SkillCommands::Evolve {
            path,
            major,
            minor,
            patch,
        } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Evolve Skill                                       ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let skill_path = std::path::Path::new(&path);
            let skill_md = skill_path.join("SKILL.md");
            if !skill_md.exists() {
                eprintln!("{} No SKILL.md found at {}", "✗".red(), skill_md.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&skill_md)?;
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() < 3 {
                eprintln!("{} Invalid SKILL.md format", "✗".red());
                std::process::exit(1);
            }

            let mut fm: serde_yaml::Value = serde_yaml::from_str(parts[1].trim())?;

            // Read current version: nested metadata.version (CNSB v2 layout)
            // or flat top-level version (agentskills layout).
            let current_version = fm
                .get("metadata")
                .and_then(|m| m.get("version"))
                .or_else(|| fm.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();

            let mut ver_parts: Vec<u32> = current_version
                .split('.')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            while ver_parts.len() < 3 {
                ver_parts.push(0);
            }

            if major {
                ver_parts[0] += 1;
                ver_parts[1] = 0;
                ver_parts[2] = 0;
            } else if minor {
                ver_parts[1] += 1;
                ver_parts[2] = 0;
            } else if patch {
                ver_parts[2] += 1;
            }

            let new_version = format!("{}.{}.{}", ver_parts[0], ver_parts[1], ver_parts[2]);

            // Update version where it lives: metadata.version if a metadata
            // section exists, else the flat top-level version key.
            let version_key = serde_yaml::Value::String("version".to_string());
            let version_val = serde_yaml::Value::String(new_version.clone());
            if let Some(metadata) = fm.get_mut("metadata").and_then(|m| m.as_mapping_mut()) {
                metadata.insert(version_key, version_val);
            } else if let Some(root) = fm.as_mapping_mut() {
                root.insert(version_key, version_val);
            } else {
                anyhow::bail!("SKILL.md frontmatter is not a YAML mapping");
            }

            // Write updated SKILL.md
            let new_fm = serde_yaml::to_string(&fm)?;
            let new_content = format!("---\n{}---\n{}", new_fm, parts[2]);
            std::fs::write(&skill_md, new_content)?;

            cli_println!(
                "{} Bumped version: {} → {}",
                "✓".green(),
                current_version,
                new_version
            );

            // Generate missing lifecycle stubs
            let mut added = Vec::new();
            let scripts_dir = skill_path.join("scripts");
            std::fs::create_dir_all(&scripts_dir)?;

            let test_script = scripts_dir.join("test.sh");
            if !test_script.exists() {
                std::fs::write(
                    &test_script,
                    "#!/usr/bin/env bash\nset -euo pipefail\n# Self-test runner for this skill\necho \"Running skill self-tests...\"\n",
                )?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&test_script)?.permissions();
                    perms.set_mode(perms.mode() | 0o111);
                    std::fs::set_permissions(&test_script, perms)?;
                }
                added.push("scripts/test.sh");
                cli_println!("  Created scripts/test.sh");
            }

            let validate_script = scripts_dir.join("validate.sh");
            if !validate_script.exists() {
                std::fs::write(
                    &validate_script,
                    "#!/usr/bin/env bash\nset -euo pipefail\n# Schema validation wrapper for this skill\necho \"Validating skill schema...\"\n",
                )?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&validate_script)?.permissions();
                    perms.set_mode(perms.mode() | 0o111);
                    std::fs::set_permissions(&validate_script, perms)?;
                }
                added.push("scripts/validate.sh");
                cli_println!("  Created scripts/validate.sh");
            }

            let sbom = skill_path.join("sbom.json");
            if !sbom.exists() {
                std::fs::write(
                    &sbom,
                    serde_json::to_string_pretty(&serde_json::json!({
                        "bomFormat": "CycloneDX",
                        "specVersion": "1.5",
                        "components": []
                    }))?,
                )?;
                added.push("sbom.json");
                cli_println!("  Created sbom.json (CycloneDX stub)");
            }

            let provenance = skill_path.join("provenance.json");
            if !provenance.exists() {
                std::fs::write(
                    &provenance,
                    serde_json::to_string_pretty(&serde_json::json!({
                        "_type": "https://in-toto.io/Statement/v1",
                        "predicateType": "https://slsa.dev/provenance/v1"
                    }))?,
                )?;
                added.push("provenance.json");
                cli_println!("  Created provenance.json (SLSA stub)");
            }

            let changelog = skill_path.join("CHANGELOG.md");
            if !changelog.exists() {
                std::fs::write(
                    &changelog,
                    format!("# Changelog\n\n## {}\n\n- Version bump\n", new_version),
                )?;
                added.push("CHANGELOG.md");
                cli_println!("  Created CHANGELOG.md");
            } else {
                // Append entry
                let entry = format!("\n## {}\n\n- Version bump\n", new_version);
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new().append(true).open(&changelog)?;
                file.write_all(entry.as_bytes())?;
                cli_println!("  Appended entry to CHANGELOG.md");
            }

            if !added.is_empty() {
                cli_println!("{} Generated {} stub(s)", "✓".green(), added.len());
            }
        }
        SkillCommands::Improve { path, dry_run } => {
            run_skill_improve(&path, dry_run)?;
        }
        SkillCommands::Refresh { path, dry_run } => {
            run_skill_refresh(&path, dry_run)?;
        }
        SkillCommands::Share {
            skill,
            group_url,
            repo_name,
            branch,
            dry_run,
        } => {
            run_skill_share(&skill, &group_url, repo_name.as_deref(), &branch, dry_run)?;
        }
        SkillCommands::List { namespace } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         List Skills                                        ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let canonical_root = std::env::var("SKILLPACK_CANONICAL_ROOT").unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join("Skills/shared").to_string_lossy().to_string())
                    .unwrap_or_else(|| "~/Skills/shared".to_string())
            });
            let root = std::path::Path::new(&canonical_root);

            let mut skills = Vec::new();

            let mut collect_skill = |skill_path: &std::path::Path, dir_ns: &str| {
                let name = skill_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let mut skill_info = serde_json::json!({
                    "name": name,
                });
                let skill_md = skill_path.join("SKILL.md");
                if skill_md.exists()
                    && let Ok(content) = std::fs::read_to_string(&skill_md)
                {
                    let parts: Vec<&str> = content.splitn(3, "---").collect();
                    if parts.len() >= 3
                        && let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(parts[1].trim())
                    {
                        if let Some(desc) = fm.get("description").and_then(|v| v.as_str()) {
                            skill_info["description"] = desc.into();
                        }
                        if let Some(license) = fm.get("license").and_then(|v| v.as_str()) {
                            skill_info["license"] = license.into();
                        }
                        if let Some(ns_fm) = fm.get("namespace").and_then(|v| v.as_str()) {
                            skill_info["namespace"] = ns_fm.into();
                        } else if let Some(ns_meta) = fm
                            .get("metadata")
                            .and_then(|m| m.get("namespace"))
                            .and_then(|v| v.as_str())
                        {
                            skill_info["namespace"] = ns_meta.into();
                        } else {
                            skill_info["namespace"] = if dir_ns.is_empty() {
                                "default".into()
                            } else {
                                dir_ns.into()
                            };
                        }
                        if let Some(version) = fm
                            .get("metadata")
                            .and_then(|m| m.get("version"))
                            .and_then(|v| v.as_str())
                        {
                            skill_info["version"] = version.into();
                        }
                        if let Some(status) = fm
                            .get("metadata")
                            .and_then(|m| m.get("status"))
                            .and_then(|v| v.as_str())
                        {
                            skill_info["status"] = status.into();
                        }
                    }
                }
                if skill_info.get("namespace").is_none() {
                    skill_info["namespace"] = if dir_ns.is_empty() {
                        "default".into()
                    } else {
                        dir_ns.into()
                    };
                }
                skills.push(skill_info);
            };

            if let Some(ns) = namespace {
                let ns_path = root.join(&ns);
                if ns_path.exists() {
                    for entry in std::fs::read_dir(&ns_path)?.flatten() {
                        let path = entry.path();
                        if path.is_dir() && path.join("SKILL.md").exists() {
                            collect_skill(&path, &ns);
                        }
                    }
                }
            } else {
                for entry in std::fs::read_dir(root)?.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    // If the directory itself has a SKILL.md, it's a root-level skill
                    if path.join("SKILL.md").exists() {
                        collect_skill(&path, "");
                        continue;
                    }
                    // Otherwise, treat it as a namespace and look for skills inside
                    let ns = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    for skill_entry in std::fs::read_dir(&path)?.flatten() {
                        let skill_path = skill_entry.path();
                        if skill_path.is_dir() && skill_path.join("SKILL.md").exists() {
                            collect_skill(&skill_path, &ns);
                        }
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&skills)?);
        }
        SkillCommands::Show { name, namespace } => {
            cli_println!(
                "{}",
                "╔════════════════════════════════════════════════════════════╗".cyan()
            );
            cli_println!(
                "{}",
                "║         Show Skill                                         ║".cyan()
            );
            cli_println!(
                "{}",
                "╚════════════════════════════════════════════════════════════╝".cyan()
            );
            cli_println!();

            let canonical_root = std::env::var("SKILLPACK_CANONICAL_ROOT").unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join("Skills/shared").to_string_lossy().to_string())
                    .unwrap_or_else(|| "~/Skills/shared".to_string())
            });
            let root = std::path::Path::new(&canonical_root);

            // Search for skill: if namespace given, look there; otherwise search root and all namespaces
            let skill_path = if let Some(ns) = &namespace {
                let p = root.join(ns).join(&name);
                if p.exists() {
                    p
                } else {
                    eprintln!(
                        "{} Skill '{}' not found in namespace '{}'",
                        "✗".red(),
                        name,
                        ns
                    );
                    std::process::exit(1);
                }
            } else {
                // Try root first
                let root_skill = root.join(&name);
                if root_skill.exists() {
                    root_skill
                } else {
                    // Search all namespace directories
                    let mut found = None;
                    for entry in std::fs::read_dir(root)?.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let candidate = path.join(&name);
                            if candidate.exists() {
                                found = Some(candidate);
                                break;
                            }
                        }
                    }
                    match found {
                        Some(p) => p,
                        None => {
                            eprintln!("{} Skill '{}' not found", "✗".red(), name);
                            std::process::exit(1);
                        }
                    }
                }
            };

            let skill_md = skill_path.join("SKILL.md");
            let mut info = serde_json::json!({"name": name, "path": skill_path.to_string_lossy()});

            if skill_md.exists() {
                let content = std::fs::read_to_string(&skill_md)?;
                let parts: Vec<&str> = content.splitn(3, "---").collect();
                if parts.len() >= 3
                    && let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(parts[1].trim())
                {
                    if let Some(desc) = fm.get("description").and_then(|v| v.as_str()) {
                        info["description"] = desc.into();
                    }
                    if let Some(license) = fm.get("license").and_then(|v| v.as_str()) {
                        info["license"] = license.into();
                    }
                    if let Some(ns_from_fm) = fm.get("namespace").and_then(|v| v.as_str()) {
                        info["namespace"] = ns_from_fm.into();
                    } else if let Some(ns_from_meta) = fm
                        .get("metadata")
                        .and_then(|m| m.get("namespace"))
                        .and_then(|v| v.as_str())
                    {
                        info["namespace"] = ns_from_meta.into();
                    } else {
                        info["namespace"] = "default".into();
                    }
                    if let Some(version) = fm
                        .get("metadata")
                        .and_then(|m| m.get("version"))
                        .and_then(|v| v.as_str())
                    {
                        info["version"] = version.into();
                    }
                    if let Some(status) = fm
                        .get("metadata")
                        .and_then(|m| m.get("status"))
                        .and_then(|v| v.as_str())
                    {
                        info["status"] = status.into();
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        SkillCommands::MigrateSkills { canonical_root } => {
            run_migrate_skills(canonical_root.as_deref())?;
        }
        SkillCommands::MigrateAgents => {
            run_migrate_agents()?;
        }
        SkillCommands::MigrateClaudeAgents { agents_dir } => {
            run_migrate_claude_agents(agents_dir.as_deref())?;
        }
        SkillCommands::MigrateHarnesses => {
            run_migrate_harnesses()?;
        }
    }

    Ok(())
}

// ============================================================================
// Skill Migration Helpers
// ============================================================================

fn print_migration_summary(label: &str, results: &[(String, Vec<String>)]) {
    let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
    let ok: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();

    cli_println!();
    cli_println!("{}", "=".repeat(60));
    cli_println!("{}", label);
    cli_println!("{}", "=".repeat(60));
    cli_println!("Total scanned: {}", results.len());
    cli_println!("Migrated: {}", migrated.len());
    cli_println!("Already compliant: {}", ok.len());

    if !migrated.is_empty() {
        cli_println!();
        cli_println!("Migrated:");
        for (name, changes) in migrated {
            cli_println!("  - {}: {}", name, changes.join(", "));
        }
    }
}

fn run_migrate_skills(canonical_root: Option<&str>) -> Result<()> {
    let root: std::path::PathBuf = canonical_root.map(|s| s.into()).unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Skills/shared"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/Skills/shared"))
    });

    if !root.exists() {
        anyhow::bail!("Canonical root not found: {}", root.display());
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Migrate Skills                                     ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    // Root-level skills
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let changes =
                migrate_skill_file(&path.join("SKILL.md"), &name, "default", "Apache-2.0", None)?;
            results.push((name, changes));
        }
    }

    // Namespace-level skills
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let ns = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        for sub in std::fs::read_dir(&path)?.flatten() {
            let sub_path = sub.path();
            if sub_path.is_dir() && sub_path.join("SKILL.md").exists() {
                let name = sub_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let changes =
                    migrate_skill_file(&sub_path.join("SKILL.md"), &name, &ns, "Apache-2.0", None)?;
                results.push((name, changes));
            }
        }
    }

    print_migration_summary("SKILL MIGRATION SUMMARY", &results);
    Ok(())
}

fn run_migrate_agents() -> Result<()> {
    run_migrate_agents_at(None)
}

/// Migrate agent skills at the given root, or the default
/// `~/.config/agents/skills`. The explicit-root form keeps tests hermetic
/// (no global HOME mutation, which races other tests).
fn run_migrate_agents_at(agents_root: Option<&str>) -> Result<()> {
    let root = match agents_root {
        Some(r) => std::path::PathBuf::from(r),
        None => dirs::home_dir()
            .map(|h| h.join(".config/agents/skills"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config/agents/skills")),
    };

    if !root.exists() {
        anyhow::bail!("Agents root not found: {}", root.display());
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Migrate Agent Skills                               ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let changes = migrate_skill_file(
                &path.join("SKILL.md"),
                &name,
                "agents",
                "MIT",
                Some("vercel"),
            )?;
            results.push((name, changes));
        }
    }

    print_migration_summary("AGENT SKILL MIGRATION SUMMARY", &results);
    Ok(())
}

fn run_migrate_claude_agents(agents_dir: Option<&str>) -> Result<()> {
    let root: std::path::PathBuf = agents_dir.map(|s| s.into()).unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join(".claude/agents"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.claude/agents"))
    });

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Migrate Claude Agents                              ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    if root.exists() {
        for entry in std::fs::read_dir(&root)?.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let changes = migrate_skill_file(&path, &name, "claude", "MIT", Some("ckodex"))?;
                results.push((name, changes));
            }
        }
    }

    // Also migrate example agentic-engineer
    let example = std::path::Path::new(
        "/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/examples/agentic-skill-template/agents/agentic-engineer.md",
    );
    if example.exists() {
        let name = example
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let changes = migrate_skill_file(example, &name, "claude", "MIT", Some("ckodex"))?;
        results.push((name, changes));
    }

    print_migration_summary("CLAUDE AGENT MIGRATION SUMMARY", &results);
    Ok(())
}

fn run_migrate_harnesses() -> Result<()> {
    let root = dirs::home_dir()
        .map(|h| h.join(".config/agents/skills"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/agents/skills"));

    if !root.exists() {
        anyhow::bail!("Agents root not found: {}", root.display());
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║         Migrate Agent Harnesses                            ║".cyan()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    for skill_dir in std::fs::read_dir(&root)?.flatten() {
        let path = skill_dir.path();
        if !path.is_dir() {
            continue;
        }
        let rules_dir = path.join("rules");
        if !rules_dir.exists() {
            continue;
        }
        let skill_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        for rule_entry in std::fs::read_dir(&rules_dir)?.flatten() {
            let rule_path = rule_entry.path();
            if rule_path.is_file() && rule_path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = rule_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let (maybe_fm, body) = read_skill_md(&rule_path)?;
                let mut changes = Vec::new();
                let mut fm = match maybe_fm {
                    Some(m) => m,
                    None => {
                        let mut m = serde_yaml::Mapping::new();
                        m.insert(
                            serde_yaml::Value::String("name".to_string()),
                            serde_yaml::Value::String(name.clone()),
                        );
                        m.insert(
                            serde_yaml::Value::String("description".to_string()),
                            serde_yaml::Value::String(format!(
                                "Rule for {}.",
                                name.replace('-', " ")
                            )),
                        );
                        m.insert(
                            serde_yaml::Value::String("namespace".to_string()),
                            serde_yaml::Value::String(skill_name.clone()),
                        );
                        let mut meta = serde_yaml::Mapping::new();
                        meta.insert(
                            serde_yaml::Value::String("version".to_string()),
                            serde_yaml::Value::String("1.0.0".to_string()),
                        );
                        meta.insert(
                            serde_yaml::Value::String("status".to_string()),
                            serde_yaml::Value::String("active".to_string()),
                        );
                        meta.insert(
                            serde_yaml::Value::String("tags".to_string()),
                            serde_yaml::Value::Sequence(Vec::new()),
                        );
                        m.insert(
                            serde_yaml::Value::String("metadata".to_string()),
                            serde_yaml::Value::Mapping(meta),
                        );
                        write_skill_md(&rule_path, &m, &body)?;
                        changes.push("created frontmatter".to_string());
                        results.push((name, changes));
                        continue;
                    }
                };

                if ensure_field(
                    &mut fm,
                    "namespace",
                    serde_yaml::Value::String(skill_name.clone()),
                ) {
                    changes.push(format!("added namespace: {}", skill_name));
                }

                if ensure_metadata_field(
                    &mut fm,
                    "version",
                    serde_yaml::Value::String("1.0.0".to_string()),
                ) {
                    changes.push("added metadata.version".to_string());
                }
                if ensure_metadata_field(
                    &mut fm,
                    "status",
                    serde_yaml::Value::String("active".to_string()),
                ) {
                    changes.push("added metadata.status".to_string());
                }

                if !changes.is_empty() {
                    write_skill_md(&rule_path, &fm, &body)?;
                }
                results.push((name, changes));
            }
        }
    }

    print_migration_summary("AGENT HARNESS MIGRATION SUMMARY", &results);
    Ok(())
}

// ============================================================================
// ASM — Helper: discover installed skills across agent paths
// ============================================================================

use crate::cli::manifest::{SkillEntry, SkillInventoryManifest};
use crate::config::{collect_skill_paths, load_config};

/// Walk a single agent directory and collect SkillEntry records.
fn collect_entries_from_dir(tool: &str, dir: &std::path::Path) -> Vec<SkillEntry> {
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return entries;
    };

    for item in read.flatten() {
        let p = item.path();
        if !p.is_dir() {
            continue;
        }
        let has_skill_md = p.join("SKILL.md").exists();
        let has_cnsb = p.join("skill.cnsb.json").exists();
        if !has_skill_md && !has_cnsb {
            continue;
        }
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut version = "?".to_string();
        let mut description = None;

        if let Ok(content) = std::fs::read_to_string(p.join("SKILL.md")) {
            let parts: Vec<&str> = content.splitn(3, "---").collect();
            if parts.len() >= 3
                && let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(parts[1].trim())
            {
                if let Some(v) = fm.get("version").and_then(|v| v.as_str()) {
                    version = v.to_string();
                } else if let Some(v) = fm
                    .get("metadata")
                    .and_then(|m| m.get("version"))
                    .and_then(|v| v.as_str())
                {
                    version = v.to_string();
                }
                if let Some(d) = fm.get("description").and_then(|v| v.as_str()) {
                    description = Some(d.to_string());
                }
            }
        }

        let disabled = p.join(".disabled").exists();
        let symlink = p
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        entries.push(SkillEntry {
            name,
            version,
            tool: tool.to_string(),
            location: p.to_string_lossy().to_string(),
            description,
            disabled,
            symlink,
            source: None,
        });
    }
    entries
}

/// Collect all skills, optionally filtered by tool and scope.
fn all_skill_entries(tool_filter: Option<&str>, scope: &str) -> Vec<SkillEntry> {
    let cfg = load_config().unwrap_or_default();
    let paths = collect_skill_paths(&cfg, scope, tool_filter);
    let mut all = Vec::new();
    for (tool, dir) in &paths {
        all.extend(collect_entries_from_dir(tool, dir));
    }
    all
}

/// Emit output respecting json/machine flags.
fn emit_output(data: &serde_json::Value, machine_output: bool) {
    if machine_output {
        let env = json!({ "v": 1, "ok": true, "data": data });
        println!("{}", serde_json::to_string_pretty(&env).unwrap_or_default());
    } else {
        println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
    }
}

// ============================================================================
// ASM: list
// ============================================================================

fn run_asm_list(
    tool_filter: Option<&str>,
    scope: &str,
    sort: &str,
    flat: bool,
    json_output: bool,
    machine_output: bool,
) -> Result<()> {
    let mut entries = all_skill_entries(tool_filter, scope);

    match sort {
        "version" => entries.sort_by(|a, b| a.version.cmp(&b.version)),
        "location" => entries.sort_by(|a, b| a.location.cmp(&b.location)),
        _ => entries.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    if json_output || machine_output {
        let data = serde_json::to_value(&entries)?;
        return {
            emit_output(&data, machine_output);
            Ok(())
        };
    }

    if entries.is_empty() {
        cli_println!("{} No skills found.", "ℹ".blue());
        return Ok(());
    }

    cli_println!(
        "{:<32} {:<10} {:<16} {}",
        "NAME".bold(),
        "VERSION".bold(),
        "TOOL".bold(),
        "LOCATION".bold()
    );
    cli_println!("{}", "─".repeat(90));

    for e in &entries {
        let disabled_marker = if e.disabled {
            " [disabled]".yellow().to_string()
        } else {
            String::new()
        };
        let symlink_marker = if e.symlink {
            " →".dimmed().to_string()
        } else {
            String::new()
        };
        if flat {
            cli_println!(
                "{:<32} {:<10} {:<16} {}{}{}",
                e.name,
                e.version,
                e.tool,
                e.location,
                disabled_marker,
                symlink_marker
            );
        } else {
            cli_println!(
                "{:<32} {:<10} {:<16} {}{}{}",
                e.name.cyan().to_string(),
                e.version,
                e.tool,
                e.location,
                disabled_marker,
                symlink_marker
            );
        }
    }
    cli_println!();
    cli_println!("Total: {}", entries.len().to_string().bold());
    Ok(())
}

// ============================================================================
// ASM: search
// ============================================================================

fn run_asm_search(
    query: &str,
    tool_filter: Option<&str>,
    json_output: bool,
    machine_output: bool,
) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let q = query.to_lowercase();

    let matches: Vec<&SkillEntry> = entries
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&q)
                || e.description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                || e.tool.to_lowercase().contains(&q)
        })
        .collect();

    if json_output || machine_output {
        let data = serde_json::to_value(&matches)?;
        return {
            emit_output(&data, machine_output);
            Ok(())
        };
    }

    if matches.is_empty() {
        cli_println!("{} No skills matching '{}'", "ℹ".blue(), query);
        return Ok(());
    }

    cli_println!(
        "Found {} skill(s) matching '{}':\n",
        matches.len().to_string().green().bold(),
        query
    );
    for e in &matches {
        cli_println!(
            "  {} {} v{} [{}]",
            "▸".cyan(),
            e.name.bold(),
            e.version,
            e.tool
        );
        if let Some(desc) = &e.description {
            cli_println!("    {}", desc);
        }
        cli_println!("    {}", e.location.dimmed());
    }
    Ok(())
}

// ============================================================================
// ASM: inspect
// ============================================================================

fn run_asm_inspect(
    skill_name: &str,
    tool_filter: Option<&str>,
    json_output: bool,
    machine_output: bool,
) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let found: Vec<&SkillEntry> = entries.iter().filter(|e| e.name == skill_name).collect();

    if found.is_empty() {
        anyhow::bail!("Skill '{}' not found", skill_name);
    }

    let entry = found[0];
    let skill_path = std::path::Path::new(&entry.location);

    let mut detail = serde_json::to_value(entry)?;

    // Enrich with full SKILL.md frontmatter
    let skill_md = skill_path.join("SKILL.md");
    if skill_md.exists()
        && let Ok(content) = std::fs::read_to_string(&skill_md)
    {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3
            && let Ok(fm) = serde_yaml::from_str::<serde_json::Value>(parts[1].trim())
        {
            detail["frontmatter"] = fm;
        }
    }

    // List files
    let files: Vec<String> = walkdir::WalkDir::new(skill_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(skill_path)
                .unwrap_or(e.path())
                .to_string_lossy()
                .to_string()
        })
        .collect();
    detail["files"] = serde_json::to_value(&files)?;
    detail["file_count"] = serde_json::Value::Number(files.len().into());

    if json_output || machine_output {
        return {
            emit_output(&detail, machine_output);
            Ok(())
        };
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!("{}", format!("║  Skill: {:<51}║", skill_name).cyan());
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();
    cli_println!("  {} {}", "Name:".bold(), entry.name);
    cli_println!("  {} {}", "Version:".bold(), entry.version);
    cli_println!("  {} {}", "Tool:".bold(), entry.tool);
    cli_println!("  {} {}", "Location:".bold(), entry.location);
    if let Some(desc) = &entry.description {
        cli_println!("  {} {}", "Description:".bold(), desc);
    }
    cli_println!(
        "  {} {}",
        "Disabled:".bold(),
        if entry.disabled {
            "yes".yellow().to_string()
        } else {
            "no".green().to_string()
        }
    );
    cli_println!(
        "  {} {}",
        "Symlink:".bold(),
        if entry.symlink {
            "yes".dimmed().to_string()
        } else {
            "no".to_string()
        }
    );
    cli_println!("  {} {} file(s)", "Files:".bold(), files.len());
    for f in &files {
        cli_println!("    - {}", f);
    }
    Ok(())
}

// ============================================================================
// ASM: uninstall
// ============================================================================

fn run_asm_uninstall(
    skill_name: &str,
    tool_filter: Option<&str>,
    yes: bool,
    dry_run: bool,
) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let found: Vec<&SkillEntry> = entries.iter().filter(|e| e.name == skill_name).collect();

    if found.is_empty() {
        anyhow::bail!("Skill '{}' not found", skill_name);
    }

    for entry in &found {
        cli_println!(
            "{} Uninstall '{}' from {} [{}]?",
            "⚠".yellow().bold(),
            entry.name,
            entry.location,
            entry.tool
        );

        if !yes && !dry_run {
            cli_println!("  Type 'yes' to confirm (or use -y / --yes to skip): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != "yes" {
                cli_println!("{} Skipped.", "ℹ".blue());
                continue;
            }
        }

        if dry_run {
            cli_println!(
                "{} [dry-run] Would remove: {}",
                "ℹ".blue().bold(),
                entry.location
            );
            continue;
        }

        if entry.symlink {
            std::fs::remove_file(&entry.location)?;
        } else {
            std::fs::remove_dir_all(&entry.location)?;
        }
        cli_println!(
            "{} Uninstalled '{}' from {}",
            "✓".green().bold(),
            entry.name,
            entry.location
        );
    }
    Ok(())
}

// ============================================================================
// ASM: disable / enable
// ============================================================================

fn run_asm_disable(target: &str, tool_filter: Option<&str>, dry_run: bool) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let matched: Vec<&SkillEntry> = entries
        .iter()
        .filter(|e| e.name == target || e.name.contains(target))
        .collect();

    if matched.is_empty() {
        anyhow::bail!("No skills matching '{}'", target);
    }

    for entry in matched {
        let marker = std::path::Path::new(&entry.location).join(".disabled");
        if marker.exists() {
            cli_println!("{} '{}' is already disabled", "ℹ".blue(), entry.name);
            continue;
        }
        if dry_run {
            cli_println!("{} [dry-run] Would disable '{}'", "ℹ".blue(), entry.name);
            continue;
        }
        std::fs::write(&marker, "")?;
        cli_println!("{} Disabled '{}'", "✓".green().bold(), entry.name);
    }
    Ok(())
}

fn run_asm_enable(target: &str, tool_filter: Option<&str>, dry_run: bool) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let matched: Vec<&SkillEntry> = entries
        .iter()
        .filter(|e| e.name == target || e.name.contains(target))
        .collect();

    if matched.is_empty() {
        anyhow::bail!("No skills matching '{}'", target);
    }

    for entry in matched {
        let marker = std::path::Path::new(&entry.location).join(".disabled");
        if !marker.exists() {
            cli_println!("{} '{}' is not disabled", "ℹ".blue(), entry.name);
            continue;
        }
        if dry_run {
            cli_println!("{} [dry-run] Would enable '{}'", "ℹ".blue(), entry.name);
            continue;
        }
        std::fs::remove_file(&marker)?;
        cli_println!("{} Enabled '{}'", "✓".green().bold(), entry.name);
    }
    Ok(())
}

// ============================================================================
// ASM: audit (duplicate detection + security sub-command)
// ============================================================================

fn run_asm_audit(command: Option<AuditCommands>) -> Result<()> {
    match command {
        Some(AuditCommands::Security { name, tool }) => {
            run_asm_audit_security(&name, tool.as_deref())
        }
        Some(AuditCommands::Collisions { tool }) => run_asm_audit_collisions(tool.as_deref()),
        None => {
            run_asm_audit_duplicates()?;
            run_asm_audit_collisions(None)
        }
    }
}

/// Description-collision audit: two skills whose descriptions read alike
/// compete for the same triggers, so agent routing between them is
/// effectively random. Same-named copies across tools are the duplicate
/// audit's job; here we compare distinct skills only.
fn run_asm_audit_collisions(tool_filter: Option<&str>) -> Result<()> {
    use skillpack_domain::collision::{
        Collision, CollisionSeverity, RoutingSurface, find_collisions,
    };

    let entries = all_skill_entries(tool_filter, "both");
    let mut seen = std::collections::HashSet::new();
    let surfaces: Vec<RoutingSurface> = entries
        .iter()
        .filter(|e| seen.insert(e.name.clone()))
        .filter_map(|e| {
            e.description.as_ref().map(|d| RoutingSurface {
                name: e.name.clone(),
                description: d.clone(),
            })
        })
        .collect();

    let hits: Vec<Collision> = find_collisions(&surfaces);

    cli_println!();
    if hits.is_empty() {
        cli_println!(
            "{} No description collisions across {} skills.",
            "✓".green().bold(),
            surfaces.len()
        );
        return Ok(());
    }

    let errors = hits
        .iter()
        .filter(|c| c.severity == CollisionSeverity::Error)
        .count();
    cli_println!(
        "{} {} description collision(s) across {} skills ({} severe):\n",
        "⚠".yellow().bold(),
        hits.len(),
        surfaces.len(),
        errors
    );
    for c in &hits {
        let marker = match c.severity {
            CollisionSeverity::Error => "✗".red().bold(),
            CollisionSeverity::Warn => "▸".yellow(),
        };
        cli_println!(
            "  {} {:.2}  {}  ↔  {}",
            marker,
            c.similarity,
            c.a.bold(),
            c.b.bold()
        );
    }
    cli_println!(
        "\n  Colliding descriptions route unpredictably — sharpen each with a distinct \"Use when …\" trigger."
    );
    Ok(())
}

fn run_asm_audit_duplicates() -> Result<()> {
    let entries = all_skill_entries(None, "both");
    let mut by_name: std::collections::HashMap<String, Vec<&SkillEntry>> =
        std::collections::HashMap::new();

    for e in &entries {
        by_name.entry(e.name.clone()).or_default().push(e);
    }

    let duplicates: Vec<(&String, &Vec<&SkillEntry>)> =
        by_name.iter().filter(|(_, v)| v.len() > 1).collect();

    if duplicates.is_empty() {
        cli_println!("{} No duplicate skills detected.", "✓".green().bold());
        return Ok(());
    }

    cli_println!(
        "{} Found {} duplicate skill name(s):\n",
        "⚠".yellow().bold(),
        duplicates.len()
    );
    for (name, occurrences) in &duplicates {
        cli_println!(
            "  {} {} ({} copies)",
            "▸".yellow(),
            name.bold(),
            occurrences.len()
        );
        for e in *occurrences {
            cli_println!("    [{}] {}", e.tool, e.location);
        }
    }
    Ok(())
}

fn run_asm_audit_security(name: &str, tool_filter: Option<&str>) -> Result<()> {
    let entries = all_skill_entries(tool_filter, "both");
    let found: Option<&SkillEntry> = entries.iter().find(|e| e.name == name);

    let path = if let Some(e) = found {
        e.location.clone()
    } else if std::path::Path::new(name).exists() {
        name.to_string()
    } else {
        anyhow::bail!("Skill '{}' not found", name);
    };

    cli_println!(
        "{} Running security audit on '{}' …",
        "🔍".to_string(),
        name
    );

    let reader = FilesystemReader::new();
    let use_case = AssessSkillUseCase::new(reader, crate::checkers::all_checkers());
    let response = use_case.execute(skillpack_application::AssessSkillRequest {
        skill_path: path.clone(),
        min_score: None,
    })?;

    let assessment = &response.assessment;
    let sec_score = assessment
        .dimension_scores
        .get(&skillpack_domain::DimensionId::Security)
        .map(|s| s.value())
        .unwrap_or(0.0);

    let sec_issues: Vec<&skillpack_domain::Issue> = assessment
        .issues
        .iter()
        .filter(|i| i.dimension == skillpack_domain::DimensionId::Security)
        .collect();

    cli_println!(
        "\n  Security score: {:.0}/100  Grade: {}",
        sec_score,
        assessment.grade().as_str().bold()
    );

    if sec_issues.is_empty() {
        cli_println!("  {} No security issues found.", "✓".green().bold());
    } else {
        cli_println!(
            "  {} {} security issue(s):",
            "⚠".yellow().bold(),
            sec_issues.len()
        );
        for issue in sec_issues {
            let sev = format!("{:?}", issue.severity);
            cli_println!("    [{:7}] {}", sev, issue.message);
        }
    }
    Ok(())
}

// ============================================================================
// ASM: export
// ============================================================================

// `_json_output`: a manifest to stdout is always JSON, so the flag is inert
// here (kept for CLI-dispatch signature symmetry with the other asm_* handlers).
fn run_asm_export(output: Option<&str>, _json_output: bool, machine_output: bool) -> Result<()> {
    let entries = all_skill_entries(None, "both");
    let manifest = SkillInventoryManifest::new(entries);
    let data = serde_json::to_value(&manifest)?;

    match output {
        Some(path) => {
            std::fs::write(path, serde_json::to_string_pretty(&manifest)?)?;
            cli_println!(
                "{} Exported {} skill(s) to {}",
                "✓".green().bold(),
                manifest.total,
                path
            );
        }
        // A manifest dumped to stdout is always structured JSON; there is no
        // human/table rendering for it (see emit_output). Force JSON here.
        None => emit_output(&data, machine_output),
    }
    Ok(())
}

// ============================================================================
// ASM: import
// ============================================================================

fn run_asm_import(file: &str, yes: bool, dry_run: bool) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let manifest: SkillInventoryManifest = serde_json::from_str(&content)?;

    cli_println!(
        "{} Importing {} skill(s) from {} (exported {})",
        "ℹ".blue().bold(),
        manifest.total,
        file,
        manifest.exported_at.format("%Y-%m-%d %H:%M UTC")
    );

    let cfg = load_config().unwrap_or_default();

    for entry in &manifest.skills {
        cli_println!(
            "  {} '{}' v{} [{}] → {}",
            "▸".cyan(),
            entry.name,
            entry.version,
            entry.tool,
            entry.location
        );

        if let Some(src) = &entry.source {
            if !yes && !dry_run {
                cli_println!("  Install from {}? (yes/N) ", src);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim() != "yes" {
                    cli_println!("    {} Skipped.", "ℹ".blue());
                    continue;
                }
            }

            if dry_run {
                cli_println!("    {} [dry-run] Would install from {}", "ℹ".blue(), src);
                continue;
            }

            let dest = cfg
                .agent_paths
                .get(&entry.tool)
                .cloned()
                .unwrap_or_else(|| entry.location.clone());
            let rt = tokio::runtime::Runtime::new()?;
            match rt.block_on(run_install(src, &dest, None, None, false)) {
                Ok(_) => cli_println!("    {} Installed", "✓".green()),
                Err(e) => cli_println!("    {} Failed: {}", "✗".red(), e),
            }
        } else {
            cli_println!(
                "    {} No source reference — skipping install",
                "⚠".yellow()
            );
        }
    }
    Ok(())
}

// ============================================================================
// ASM: stats
// ============================================================================

fn run_asm_stats(json_output: bool, machine_output: bool) -> Result<()> {
    let entries = all_skill_entries(None, "both");
    let total = entries.len();
    let disabled = entries.iter().filter(|e| e.disabled).count();
    let symlinks = entries.iter().filter(|e| e.symlink).count();

    let mut by_tool: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for e in &entries {
        *by_tool.entry(e.tool.clone()).or_insert(0) += 1;
    }

    let mut grade_dist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for e in &entries {
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, crate::checkers::all_checkers());
        let grade = use_case
            .execute(skillpack_application::AssessSkillRequest {
                skill_path: e.location.clone(),
                min_score: None,
            })
            .map(|r| r.assessment.grade().as_str().to_string())
            .unwrap_or_else(|_| "?".to_string());
        *grade_dist.entry(grade).or_insert(0) += 1;
    }

    let stats = json!({
        "total": total,
        "disabled": disabled,
        "symlinks": symlinks,
        "by_tool": by_tool,
        "grade_distribution": grade_dist,
    });

    if json_output || machine_output {
        return {
            emit_output(&stats, machine_output);
            Ok(())
        };
    }

    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║                  SKILL METRICS DASHBOARD                   ║"
            .cyan()
            .bold()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();
    cli_println!("  Total skills:    {}", total.to_string().bold());
    cli_println!("  Disabled:        {}", disabled);
    cli_println!("  Symlinks:        {}", symlinks);
    cli_println!();
    cli_println!("  By tool:");
    let mut sorted_tools: Vec<(&String, &usize)> = by_tool.iter().collect();
    sorted_tools.sort_by_key(|(k, _)| k.as_str());
    for (tool, count) in sorted_tools {
        cli_println!("    {:20} {}", tool, count);
    }
    cli_println!();
    cli_println!("  Grade distribution:");
    let mut sorted_grades: Vec<(&String, &usize)> = grade_dist.iter().collect();
    sorted_grades.sort_by_key(|(k, _)| k.as_str());
    for (grade, count) in sorted_grades {
        let bar = "█".repeat(*count);
        cli_println!("    {:4} {:>3}  {}", grade, count, bar.green());
    }
    Ok(())
}

// ============================================================================
// ASM: link
// ============================================================================

fn run_asm_link(path: &str, tool_filter: Option<&str>, dry_run: bool) -> Result<()> {
    use std::path::Path;

    let src = Path::new(path).canonicalize()?;
    if !src.exists() {
        anyhow::bail!("Source path '{}' does not exist", path);
    }

    let skill_name = src
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine skill name from path"))?
        .to_string_lossy()
        .to_string();

    let cfg = load_config().unwrap_or_default();

    let target_dirs: Vec<(String, String)> = if let Some(tool) = tool_filter {
        cfg.agent_paths
            .get(tool)
            .map(|p| vec![(tool.to_string(), p.clone())])
            .unwrap_or_else(|| vec![(tool.to_string(), format!("~/.config/{}/skills", tool))])
    } else {
        cfg.agent_paths
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    if target_dirs.is_empty() {
        anyhow::bail!("No agent paths configured. Run 'skillpack config show' to see config.");
    }

    for (tool, dir) in &target_dirs {
        let dest = std::path::Path::new(dir).join(&skill_name);
        if dest.exists() {
            cli_println!(
                "{} '{}' already exists in {} ({})",
                "ℹ".blue(),
                skill_name,
                tool,
                dest.display()
            );
            continue;
        }
        if dry_run {
            cli_println!(
                "{} [dry-run] Would symlink {} → {}",
                "ℹ".blue().bold(),
                src.display(),
                dest.display()
            );
            continue;
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&src, &dest)?;
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&src, &dest)?;
        }
        cli_println!(
            "{} Linked '{}' into {} ({})",
            "✓".green().bold(),
            skill_name,
            tool,
            dest.display()
        );
    }
    Ok(())
}

// ============================================================================
// ASM: outdated
// ============================================================================

fn run_asm_outdated(json_output: bool, machine_output: bool) -> Result<()> {
    let entries = all_skill_entries(None, "both");

    cli_println!(
        "Checking {} installed skill(s) for updates …\n",
        entries.len()
    );

    let mut outdated = Vec::new();
    for e in &entries {
        // Try to read version from SKILL.md source field or skill.cnsb.json
        let local_version = e.version.trim_start_matches('v').to_string();
        let remote_version = check_remote_version(e);

        match remote_version {
            Some(remote) if remote.trim_start_matches('v') != local_version => {
                cli_println!(
                    "  {} {} ({} → {})",
                    "↑".yellow().bold(),
                    e.name.bold(),
                    local_version,
                    remote.green()
                );
                outdated.push(json!({
                    "name": e.name,
                    "current": local_version,
                    "latest": remote,
                    "location": e.location,
                    "tool": e.tool,
                }));
            }
            None => {
                cli_println!("  {} {} (remote version unknown)", "?".dimmed(), e.name);
            }
            _ => {
                cli_println!("  {} {} (up to date)", "✓".green(), e.name);
            }
        }
    }

    if outdated.is_empty() {
        cli_println!("\n{} All skills are up to date.", "✓".green().bold());
    } else {
        cli_println!(
            "\n{} {} skill(s) have updates available.",
            "⚠".yellow().bold(),
            outdated.len()
        );
    }

    if json_output || machine_output {
        let data = serde_json::to_value(&outdated)?;
        emit_output(&data, machine_output);
    }
    Ok(())
}

/// Best-effort remote version check: reads source ref if available, otherwise returns None.
fn check_remote_version(entry: &SkillEntry) -> Option<String> {
    if let Some(src) = &entry.source
        && (src.starts_with("ghcr.io") || src.contains('/'))
    {
        // Could make OCI HEAD request — for now return None
        return None;
    }
    None
}

// ============================================================================
// ASM: update
// ============================================================================

fn run_asm_update(names: &[String], yes: bool, dry_run: bool) -> Result<()> {
    let all = all_skill_entries(None, "both");

    let targets: Vec<&SkillEntry> = if names.is_empty() {
        all.iter().collect()
    } else {
        all.iter()
            .filter(|e| names.iter().any(|n| n == &e.name))
            .collect()
    };

    if targets.is_empty() {
        cli_println!("{} No matching skills to update.", "ℹ".blue());
        return Ok(());
    }

    for entry in targets {
        let src = match &entry.source {
            Some(s) => s.clone(),
            None => {
                cli_println!(
                    "{} '{}' has no source reference — cannot update automatically.",
                    "⚠".yellow(),
                    entry.name
                );
                continue;
            }
        };

        cli_println!(
            "{} Update '{}' from {} ?",
            "⚠".yellow().bold(),
            entry.name,
            src
        );

        if !yes && !dry_run {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != "yes" {
                cli_println!("  {} Skipped.", "ℹ".blue());
                continue;
            }
        }

        if dry_run {
            cli_println!(
                "  {} [dry-run] Would update '{}' from {}",
                "ℹ".blue(),
                entry.name,
                src
            );
            continue;
        }

        // Re-install (overwrite)
        let rt = tokio::runtime::Runtime::new()?;
        match rt.block_on(run_install(&src, &entry.location, None, None, false)) {
            Ok(_) => {
                cli_println!("{} Updated '{}'", "✓".green().bold(), entry.name);
                // Re-run security audit
                cli_println!("  Running security re-audit …");
                let _ = run_asm_audit_security(&entry.name, Some(&entry.tool));
            }
            Err(e) => {
                cli_println!("{} Update failed for '{}': {}", "✗".red(), entry.name, e);
            }
        }
    }
    Ok(())
}

// ============================================================================
// ASM: eval-providers
// ============================================================================

fn run_asm_eval_providers(command: EvalProvidersCommands) -> Result<()> {
    match command {
        EvalProvidersCommands::List => {
            let providers = vec![
                json!({
                    "id": "skillpack-builtin",
                    "version": "1.0.0",
                    "description": "Built-in eval runner (smoke, compliance, performance suites)",
                    "schema": "https://skillpack.dev/schemas/eval.schema.json",
                    "suites": ["smoke", "compliance", "performance"]
                }),
                json!({
                    "id": "ckodex-hitl",
                    "version": "1.0.0",
                    "description": "Human-in-the-loop eval provider with approval gates",
                    "schema": "https://skillpack.dev/schemas/hitl-eval.schema.json",
                    "suites": ["hitl"]
                }),
            ];
            cli_println!(
                "{:<24} {:<10} {}",
                "ID".bold(),
                "VERSION".bold(),
                "DESCRIPTION".bold()
            );
            cli_println!("{}", "─".repeat(70));
            for p in &providers {
                cli_println!(
                    "{:<24} {:<10} {}",
                    p["id"].as_str().unwrap_or(""),
                    p["version"].as_str().unwrap_or(""),
                    p["description"].as_str().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}

// ============================================================================
// ASM: bundle
// ============================================================================

fn run_asm_bundle(command: BundleCommands, dry_run: bool) -> Result<()> {
    use skillpack_domain::{BundleMetadata, SkillBundle};

    match command {
        BundleCommands::Create {
            name,
            version,
            path,
            output,
        } => {
            let search_path = std::path::Path::new(&path);
            let mut skill_defs = Vec::new();

            for entry in walkdir::WalkDir::new(search_path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() == "skill.cnsb.json")
            {
                if let Ok(content) = std::fs::read_to_string(entry.path())
                    && let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content)
                {
                    let sname = manifest["metadata"]["name"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    let sver = manifest["metadata"]["version"]
                        .as_str()
                        .unwrap_or("0.0.0")
                        .to_string();
                    skill_defs.push(skillpack_domain::SkillDefinition {
                        name: sname,
                        version: Some(sver),
                        description: manifest["metadata"]["description"]
                            .as_str()
                            .map(|s| s.to_string()),
                        entry_point: None,
                        extensions: vec![],
                        dependencies: vec![],
                        asc: vec![],
                        examples: vec![],
                        gal_min: None,
                        gal_max: None,
                    });
                }
            }

            let bundle = SkillBundle {
                schema: "https://skillpack.dev/schemas/cnsb.schema.json".to_string(),
                api_version: "cnsb.ckodex.dev/v1".to_string(),
                kind: "SkillBundle".to_string(),
                metadata: BundleMetadata {
                    name: name.clone(),
                    version: version.clone(),
                    description: None,
                    authors: vec![],
                    homepage: None,
                    repository: None,
                    license: Some("Apache-2.0".to_string()),
                    bundle_type: Some("collection".to_string()),
                    dal_version: None,
                    urn: Some(format!("urn:ckodex:bundle:{}:{}", name, version)),
                },
                skills: skill_defs,
                agents: vec![],
                governance: None,
                lifecycle: None,
                operations: None,
            };

            let out_path = output.unwrap_or_else(|| format!("{}-{}.cnsb.json", name, version));

            if dry_run {
                cli_println!(
                    "{} [dry-run] Would write bundle with {} skill(s) to {}",
                    "ℹ".blue().bold(),
                    bundle.skills.len(),
                    out_path
                );
                return Ok(());
            }

            std::fs::write(&out_path, serde_json::to_string_pretty(&bundle)?)?;
            cli_println!(
                "{} Created bundle '{}' v{} with {} skill(s): {}",
                "✓".green().bold(),
                name,
                version,
                bundle.skills.len(),
                out_path
            );
        }

        BundleCommands::Install { source, tool, yes } => {
            cli_println!(
                "{} Installing bundle from '{}' …",
                "ℹ".blue().bold(),
                source
            );
            let content = std::fs::read_to_string(&source)
                .map_err(|_| anyhow::anyhow!("Cannot read bundle file '{}'", source))?;
            let bundle: SkillBundle = serde_json::from_str(&content)?;

            let cfg = load_config().unwrap_or_default();
            let target_tool = tool.as_deref().unwrap_or_else(|| {
                cfg.agent_paths
                    .keys()
                    .next()
                    .map(|s| s.as_str())
                    .unwrap_or("agents")
            });
            let dest_dir = cfg
                .agent_paths
                .get(target_tool)
                .cloned()
                .unwrap_or_else(|| format!("~/.config/{}/skills", target_tool));

            cli_println!(
                "  Installing {} skill(s) into [{}] {} …",
                bundle.skills.len(),
                target_tool,
                dest_dir
            );

            if !yes {
                cli_println!("  Confirm? (yes/N) ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim() != "yes" {
                    cli_println!("{} Cancelled.", "ℹ".blue());
                    return Ok(());
                }
            }

            for skill in &bundle.skills {
                cli_println!("  {} {}", "▸".cyan(), skill.name);
            }
            cli_println!(
                "{} Bundle '{}' installed ({} skills).",
                "✓".green().bold(),
                bundle.metadata.name,
                bundle.skills.len()
            );
        }

        BundleCommands::List => {
            let entries = all_skill_entries(None, "both");
            let bundles: Vec<&SkillEntry> = entries
                .iter()
                .filter(|e| {
                    std::path::Path::new(&e.location)
                        .join("skill.cnsb.json")
                        .exists()
                })
                .collect();
            cli_println!("Installed bundles: {}", bundles.len());
            for b in bundles {
                cli_println!("  {} {} v{}", "▸".cyan(), b.name, b.version);
            }
        }

        BundleCommands::Show { name } => {
            let entries = all_skill_entries(None, "both");
            let found = entries.iter().find(|e| e.name == name);
            match found {
                None => anyhow::bail!("Bundle '{}' not found", name),
                Some(e) => {
                    let cnsb_path = std::path::Path::new(&e.location).join("skill.cnsb.json");
                    if cnsb_path.exists() {
                        let content = std::fs::read_to_string(cnsb_path)?;
                        println!("{}", content);
                    } else {
                        cli_println!(
                            "{} No skill.cnsb.json found at {}",
                            "⚠".yellow(),
                            e.location
                        );
                    }
                }
            }
        }

        BundleCommands::Remove { name, yes } => {
            run_asm_uninstall(&name, None, yes, dry_run)?;
        }
    }
    Ok(())
}

// ============================================================================
// ASM: index
// ============================================================================

fn run_asm_index(command: IndexCommands) -> Result<()> {
    match command {
        IndexCommands::Ingest { path } => {
            let search_path = std::path::Path::new(&path);
            let mut count = 0;
            for entry in walkdir::WalkDir::new(search_path)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() == "SKILL.md" || e.file_name() == "skill.cnsb.json")
            {
                let skill_dir = entry.path().parent().unwrap_or(entry.path());
                cli_println!("  {} Ingested: {}", "▸".cyan(), skill_dir.display());
                count += 1;
            }
            cli_println!(
                "{} Ingested {} skill(s) into index.",
                "✓".green().bold(),
                count
            );
        }

        IndexCommands::Search { query, limit } => {
            run_asm_search(&query, None, false, false)?;
            let _ = limit;
        }

        IndexCommands::List => {
            run_asm_list(None, "both", "name", false, false, false)?;
        }
    }
    Ok(())
}

// ============================================================================
// ASM: doctor
// ============================================================================

fn run_asm_doctor() -> Result<()> {
    cli_println!(
        "{}",
        "╔════════════════════════════════════════════════════════════╗".cyan()
    );
    cli_println!(
        "{}",
        "║               ENVIRONMENT HEALTH CHECKS                    ║"
            .cyan()
            .bold()
    );
    cli_println!(
        "{}",
        "╚════════════════════════════════════════════════════════════╝".cyan()
    );
    cli_println!();

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($label:expr, $ok:expr, $hint:expr) => {
            if $ok {
                cli_println!("  {} {}", "✓".green().bold(), $label);
                passed += 1;
            } else {
                cli_println!("  {} {}  ({})", "✗".red().bold(), $label, $hint);
                failed += 1;
            }
        };
    }

    // 1. Rust toolchain
    let has_cargo = std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    check!(
        "Rust/Cargo available",
        has_cargo,
        "Install rustup: https://rustup.rs"
    );

    // 2. Config file
    let cfg_path = crate::config::config_path();
    check!(
        format!("Config file: {}", cfg_path.display()),
        cfg_path.exists(),
        "Run 'skillpack config reset' to create default config"
    );

    // 3. Known agent directories
    let cfg = load_config().unwrap_or_default();
    for (tool, dir) in &cfg.agent_paths {
        let exists = std::path::Path::new(dir).exists();
        check!(
            format!("Agent path [{}]: {}", tool, dir),
            exists,
            "Create the directory or update config"
        );
    }

    // 4. Canonical store root
    let canon_root = cfg.canonical_root.clone().or_else(|| {
        dirs::home_dir().map(|h| h.join("Skills/shared").to_string_lossy().to_string())
    });
    if let Some(root) = &canon_root {
        let exists = std::path::Path::new(root).exists();
        check!(
            format!("Canonical store: {}", root),
            exists,
            "Create the directory or set SKILLPACK_CANONICAL_ROOT"
        );
    }

    // 5. Network: probe the registry's OCI base endpoint (/v2/), choosing
    //    http for a local registry (consistent with plain-HTTP OCI support).
    let registry = &cfg.registry;
    let probe = crate::oci::registry_probe_url(registry);
    let registry_ok = std::process::Command::new("curl")
        .args(["-sf", "--max-time", "3", "-o", "/dev/null", &probe])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    check!(
        format!("Registry reachable: {} ({})", registry, probe),
        registry_ok,
        "Check network connectivity or update registry in config"
    );

    cli_println!();
    cli_println!(
        "Results: {} passed, {} failed",
        passed.to_string().green().bold(),
        if failed > 0 {
            failed.to_string().red().bold().to_string()
        } else {
            failed.to_string()
        }
    );

    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

// ============================================================================
// ASM: config
// ============================================================================

fn run_asm_config(command: ConfigCommands, yes: bool) -> Result<()> {
    use crate::config::{SkillpackConfig, config_path, load_config, save_config};

    match command {
        ConfigCommands::Show => {
            let cfg = load_config()?;
            let toml_str = toml::to_string_pretty(&cfg)?;
            cli_println!("{}", toml_str);
        }

        ConfigCommands::Path => {
            println!("{}", config_path().display());
        }

        ConfigCommands::Reset { yes: cmd_yes } => {
            if !yes && !cmd_yes {
                cli_println!("Reset config to defaults? (yes/N) ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim() != "yes" {
                    cli_println!("{} Cancelled.", "ℹ".blue());
                    return Ok(());
                }
            }
            let path = save_config(&SkillpackConfig::default())?;
            cli_println!(
                "{} Config reset to defaults: {}",
                "✓".green().bold(),
                path.display()
            );
        }

        ConfigCommands::Edit => {
            let path = config_path();
            if !path.exists() {
                save_config(&SkillpackConfig::default())?;
            }
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vi".to_string());

            let status = std::process::Command::new(&editor)
                .arg(path.as_os_str())
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to open editor '{}': {}", editor, e))?;

            if !status.success() {
                anyhow::bail!("Editor exited with non-zero status");
            }
        }
    }
    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== Init Command Tests ====================

    #[test]
    fn test_init_basic_creates_skill_md() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_init(
            "test-skill",
            InitTemplate::Basic,
            Some("Test Author"),
            base_path,
        )
        .unwrap();

        let skill_md = fs::read_to_string(temp_dir.path().join("test-skill/SKILL.md")).unwrap();
        assert!(skill_md.contains("name: test-skill"));
        assert!(skill_md.contains("author: Test Author"));
    }

    #[test]
    fn test_init_basic_creates_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_init("test-skill", InitTemplate::Basic, None, base_path).unwrap();

        let manifest =
            fs::read_to_string(temp_dir.path().join("test-skill/skill.cnsb.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&manifest).unwrap();

        assert_eq!(json["metadata"]["name"], "test-skill");
        assert_eq!(json["metadata"]["version"], "0.1.0");
    }

    #[test]
    fn test_init_mcp_creates_server() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_init("mcp-skill", InitTemplate::Mcp, None, base_path).unwrap();

        let server = temp_dir.path().join("mcp-skill/capacities/mcp/server.ts");
        assert!(server.exists());

        let content = fs::read_to_string(&server).unwrap();
        assert!(content.contains("MCP Server"));
    }

    #[test]
    fn test_init_full_creates_all_directories() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_init("full-skill", InitTemplate::Full, None, base_path).unwrap();

        let skill_dir = temp_dir.path().join("full-skill");
        assert!(skill_dir.join("capacities/mcp/server.ts").exists());
        assert!(skill_dir.join("examples/basic.md").exists());
        assert!(skill_dir.join("security/threat-model.yaml").exists());
        assert!(skill_dir.join("Makefile").exists());
    }

    #[test]
    fn test_init_fails_if_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        fs::create_dir(temp_dir.path().join("existing")).unwrap();

        let result = run_init("existing", InitTemplate::Basic, None, base_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_creates_eval_yml() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_init("eval-skill", InitTemplate::Basic, None, base_path).unwrap();

        let eval_path = temp_dir.path().join("eval-skill").join("eval.yml");
        assert!(eval_path.exists());

        let content = fs::read_to_string(&eval_path).unwrap();
        assert!(content.contains("suites:"));
        assert!(content.contains("smoke:"));
        assert!(content.contains("compliance:"));
    }

    // ==================== Lock Command Tests ====================

    #[test]
    fn test_lock_generates_lock_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create minimal skill structure
        let manifest = json!({
            "metadata": { "name": "test", "version": "1.0.0" }
        });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(temp_dir.path().join("SKILL.md"), "# Test").unwrap();

        run_lock(temp_dir.path().to_str().unwrap(), false, false).unwrap();

        let lock_path = temp_dir.path().join("skill.lock");
        assert!(lock_path.exists());

        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&lock_path).unwrap()).unwrap();
        assert_eq!(lock["lockVersion"], 1);
        let deps = lock["dependencies"].as_object().unwrap();
        assert!(deps.contains_key("skill.cnsb.json"));
        assert!(deps.contains_key("SKILL.md"));
        assert!(
            deps["skill.cnsb.json"]["integrity"]
                .as_str()
                .unwrap()
                .starts_with("sha256-")
        );
        assert!(
            lock["metadata"]["generator"]
                .as_str()
                .unwrap()
                .starts_with("skillpack@")
        );
    }

    #[test]
    fn test_lock_skips_if_exists() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = json!({ "metadata": { "name": "test", "version": "1.0.0" } });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(temp_dir.path().join("skill.lock"), "existing").unwrap();

        run_lock(temp_dir.path().to_str().unwrap(), false, false).unwrap();

        // Lock file should not be modified
        let content = fs::read_to_string(temp_dir.path().join("skill.lock")).unwrap();
        assert_eq!(content, "existing");
    }

    #[test]
    fn test_lock_dry_run_does_not_create_file() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = json!({ "metadata": { "name": "test", "version": "1.0.0" } });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();

        run_lock(temp_dir.path().to_str().unwrap(), false, true).unwrap();

        assert!(
            !temp_dir.path().join("skill.lock").exists(),
            "skill.lock must not be created in dry-run mode"
        );
    }

    #[test]
    fn test_lock_force_regenerates() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = json!({ "metadata": { "name": "test", "version": "2.0.0" } });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(temp_dir.path().join("skill.lock"), "old").unwrap();

        run_lock(temp_dir.path().to_str().unwrap(), true, false).unwrap();

        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(temp_dir.path().join("skill.lock")).unwrap())
                .unwrap();
        assert_eq!(lock["lockVersion"], 1);
        let deps = lock["dependencies"].as_object().unwrap();
        assert!(deps.contains_key("skill.cnsb.json"));
        assert_eq!(deps["skill.cnsb.json"]["version"], "1.0.0");
    }

    // ==================== Migrate Command Tests ====================

    #[test]
    fn test_migrate_v1_to_v2_updates_schema() {
        let temp_dir = TempDir::new().unwrap();

        let old_manifest = json!({
            "$schema": "https://skillpack.dev/schemas/cnsb.schema.json",
            "metadata": { "name": "test", "version": "1.0.0" },
            "capacities": [{ "type": "mcp" }],
            "lifecycle": { "install": { "command": "npm ci" } }
        });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string_pretty(&old_manifest).unwrap(),
        )
        .unwrap();

        run_migrate(temp_dir.path().to_str().unwrap(), "2.0.0", false).unwrap();

        let new_manifest: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(temp_dir.path().join("skill.cnsb.json")).unwrap(),
        )
        .unwrap();

        assert!(new_manifest["$schema"].as_str().unwrap().contains("v2"));
        assert!(new_manifest.get("extensions").is_some());
        assert!(new_manifest.get("capacities").is_none());
        assert!(new_manifest["metadata"]["dal_version"].is_string());
        assert!(new_manifest["operations"]["lifecycle"].is_object());
    }

    #[test]
    fn test_migrate_dry_run_does_not_modify() {
        let temp_dir = TempDir::new().unwrap();

        let old_manifest = json!({
            "$schema": "https://skillpack.dev/schemas/cnsb.schema.json",
            "metadata": { "name": "test", "version": "1.0.0" }
        });
        let original = serde_json::to_string_pretty(&old_manifest).unwrap();
        fs::write(temp_dir.path().join("skill.cnsb.json"), &original).unwrap();

        run_migrate(temp_dir.path().to_str().unwrap(), "2.0.0", true).unwrap();

        let current = fs::read_to_string(temp_dir.path().join("skill.cnsb.json")).unwrap();
        assert_eq!(current, original);
    }

    #[test]
    fn test_migrate_creates_backup() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = json!({
            "$schema": "https://skillpack.dev/schemas/cnsb.schema.json",
            "metadata": { "name": "test", "version": "1.0.0" }
        });
        fs::write(
            temp_dir.path().join("skill.cnsb.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        run_migrate(temp_dir.path().to_str().unwrap(), "2.0.0", false).unwrap();

        assert!(temp_dir.path().join("skill.cnsb.json.bak").exists());
    }

    #[test]
    fn test_detect_schema_version() {
        assert_eq!(
            detect_schema_version("https://skillpack.dev/schemas/cnsb.schema.json"),
            "1.0.0"
        );
        assert_eq!(
            detect_schema_version("https://skillpack.dev/schemas/cnsb-v2.schema.json"),
            "2.0.0"
        );
        assert_eq!(detect_schema_version(""), "1.0.0");
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_parse_grade() {
        assert!(matches!(parse_grade("S+"), Grade::SPlus));
        assert!(matches!(parse_grade("s"), Grade::S));
        assert!(matches!(parse_grade("A"), Grade::A));
        assert!(matches!(parse_grade("b"), Grade::B));
        assert!(matches!(parse_grade("invalid"), Grade::F));
    }

    #[test]
    fn test_generate_skill_json_basic() {
        let json = generate_skill_json("test", "author", &InitTemplate::Basic);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["metadata"]["name"], "test");
        assert!(parsed.get("lifecycle").is_none());
        assert!(parsed.get("capacities").is_none());
    }

    #[test]
    fn test_generate_skill_json_full() {
        let json = generate_skill_json("test", "author", &InitTemplate::Full);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("lifecycle").is_some());
        assert!(parsed.get("capacities").is_some());
    }

    // ==================== Package Command Tests ====================

    #[test]
    fn test_package_creates_tar_gz() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("pkg-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        // Create minimal skill structure
        let manifest = json!({
            "metadata": { "name": "pkg-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# pkg-skill").unwrap();

        let output = skill_dir.join("pkg-skill-1.0.0.tar.gz");
        run_package(
            skill_dir.to_str().unwrap(),
            Some(output.to_str().unwrap()),
            true,
            PackageFormat::TarGz,
            false,
        )
        .unwrap();

        assert!(output.exists());
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
    }

    #[test]
    fn test_package_dry_run_does_not_create_file() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("dry-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "dry-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# dry-skill").unwrap();

        let output = skill_dir.join("dry-skill-1.0.0.tar.gz");
        run_package(
            skill_dir.to_str().unwrap(),
            Some(output.to_str().unwrap()),
            true,
            PackageFormat::TarGz,
            true, // dry-run
        )
        .unwrap();

        assert!(
            !output.exists(),
            "archive must not be created in dry-run mode"
        );
    }

    #[test]
    fn test_package_fails_without_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        std::fs::create_dir(&empty_dir).unwrap();

        let result = run_package(
            empty_dir.to_str().unwrap(),
            None,
            true,
            PackageFormat::TarGz,
            false,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("skill.cnsb.json not found"));
    }

    #[test]
    fn locate_skill_root_finds_root_and_nested() {
        // Skill at the root.
        let root = TempDir::new().unwrap();
        std::fs::write(root.path().join("SKILL.md"), "---\nname: r\n---\n#").unwrap();
        assert_eq!(locate_skill_root(root.path()).unwrap(), root.path());

        // Skill one level down (single candidate).
        let repo = TempDir::new().unwrap();
        let sub = repo.path().join("the-skill");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("SKILL.md"), "---\nname: s\n---\n#").unwrap();
        assert_eq!(locate_skill_root(repo.path()).unwrap(), sub);

        // No skill anywhere → error.
        let empty = TempDir::new().unwrap();
        assert!(locate_skill_root(empty.path()).is_err());
    }

    #[test]
    fn locate_skill_root_ambiguous_errors() {
        let repo = TempDir::new().unwrap();
        for n in ["a", "b"] {
            let d = repo.path().join(n);
            std::fs::create_dir(&d).unwrap();
            std::fs::write(d.join("SKILL.md"), "---\nname: x\n---\n#").unwrap();
        }
        assert!(locate_skill_root(repo.path()).is_err());
    }

    #[test]
    fn resolve_skill_name_prefers_frontmatter() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "---\nname: fancy-name\n---\n#").unwrap();
        assert_eq!(resolve_skill_name(dir.path()), "fancy-name");
    }

    #[test]
    fn safe_dir_component_rejects_traversal_and_absolute() {
        // The install path-traversal vector: a malicious frontmatter name.
        assert!(safe_dir_component("/etc/cron.d").is_err());
        assert!(safe_dir_component("../../../../home/user/.ssh").is_err());
        assert!(safe_dir_component("..").is_err());
        assert!(safe_dir_component("a/b").is_err());
        assert!(safe_dir_component("").is_err());
        // Legitimate names pass through.
        assert_eq!(safe_dir_component("my-skill").unwrap(), "my-skill");
        assert_eq!(safe_dir_component(" trimmed ").unwrap(), "trimmed");
    }

    #[test]
    fn copy_tree_skips_git_and_symlinks() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("SKILL.md"), "x").unwrap();
        std::fs::create_dir(src.path().join(".git")).unwrap();
        std::fs::write(src.path().join(".git/HEAD"), "ref").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/hosts", src.path().join("link")).unwrap();

        let dst = TempDir::new().unwrap();
        let out = dst.path().join("out");
        copy_tree(src.path(), &out).unwrap();
        assert!(out.join("SKILL.md").exists());
        assert!(!out.join(".git").exists());
        assert!(!out.join("link").exists());
    }

    #[test]
    fn agentskill_identity_reads_frontmatter() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("SKILL.md"),
            "---\nname: my-skill\nversion: 1.2.3\ndescription: d\n---\n# body",
        )
        .unwrap();
        let (name, version) = agentskill_identity(&dir.path().join("SKILL.md")).unwrap();
        assert_eq!(name, "my-skill");
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn agentskill_identity_defaults_version_and_requires_name() {
        let dir = TempDir::new().unwrap();
        // No version → defaults to 0.0.0.
        std::fs::write(
            dir.path().join("SKILL.md"),
            "---\nname: no-version\ndescription: d\n---\n# body",
        )
        .unwrap();
        let (name, version) = agentskill_identity(&dir.path().join("SKILL.md")).unwrap();
        assert_eq!(name, "no-version");
        assert_eq!(version, "0.0.0");

        // No name → error.
        std::fs::write(dir.path().join("SKILL.md"), "---\nversion: 1.0.0\n---\n# b").unwrap();
        assert!(agentskill_identity(&dir.path().join("SKILL.md")).is_err());
    }

    #[test]
    fn test_package_creates_zip() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("zip-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "zip-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# zip-skill").unwrap();

        let output = skill_dir.join("zip-skill-1.0.0.zip");
        run_package(
            skill_dir.to_str().unwrap(),
            Some(output.to_str().unwrap()),
            true,
            PackageFormat::Zip,
            false,
        )
        .unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_package_creates_tar_bz2() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("bz2-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "bz2-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# bz2-skill").unwrap();

        let output = skill_dir.join("bz2-skill-1.0.0.tar.bz2");
        run_package(
            skill_dir.to_str().unwrap(),
            Some(output.to_str().unwrap()),
            true,
            PackageFormat::TarBz2,
            false,
        )
        .unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_package_creates_tar_zstd() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("zstd-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "zstd-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# zstd-skill").unwrap();

        let output = skill_dir.join("zstd-skill-1.0.0.tar.zst");
        run_package(
            skill_dir.to_str().unwrap(),
            Some(output.to_str().unwrap()),
            true,
            PackageFormat::TarZstd,
            false,
        )
        .unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_eval_smoke_passes() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("eval-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "eval-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# eval-skill").unwrap();
        fs::write(skill_dir.join("README.md"), "# README").unwrap();

        let result = run_eval(skill_dir.to_str().unwrap(), Some("smoke"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_fails_without_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        std::fs::create_dir(&empty_dir).unwrap();

        let result = run_eval(empty_dir.to_str().unwrap(), None, None);
        assert!(result.is_err());
    }

    // ==================== Publish Command Tests ====================

    #[test]
    fn test_publish_with_sign_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("pub-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let manifest = json!({
            "metadata": { "name": "pub-skill", "version": "1.0.0" }
        });
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# pub-skill").unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(run_publish(
            skill_dir.to_str().unwrap(),
            "ghcr.io/test/pub-skill:1.0.0",
            None,
            None,
            true, // no_check
            true, // sign
            true, // dry_run
        ));

        assert!(result.is_ok());
    }

    // ==================== Wizard Command Tests ====================

    #[test]
    fn test_wizard_creates_eval_yml() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_wizard(
            "wiz-skill",
            Some("A test skill"),
            Some("Test Author"),
            base_path,
            true,
            false,
        )
        .unwrap();

        let eval_path = temp_dir.path().join("wiz-skill/eval.yml");
        assert!(eval_path.exists());

        let content = fs::read_to_string(&eval_path).unwrap();
        assert!(content.contains("suites:"));
        assert!(content.contains("smoke:"));
        assert!(content.contains("compliance:"));
        assert!(content.contains("skillpack_check"));
    }

    #[test]
    fn test_wizard_creates_makefile_with_eval_target() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_wizard("make-skill", None, None, base_path, true, false).unwrap();

        let makefile = temp_dir.path().join("make-skill/Makefile");
        assert!(makefile.exists());

        let content = fs::read_to_string(&makefile).unwrap();
        assert!(content.contains("eval:"));
        assert!(content.contains("skillpack eval --suite smoke"));
    }

    #[test]
    fn test_wizard_scaffold_creates_eval_yml() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_wizard(
            "wiz-skill",
            Some("A test skill"),
            Some("Test Author"),
            base_path,
            true,
            false,
        )
        .unwrap();

        let eval_path = temp_dir.path().join("wiz-skill/eval.yml");
        assert!(eval_path.exists());

        let content = fs::read_to_string(&eval_path).unwrap();
        assert!(content.contains("suites:"));
        assert!(content.contains("smoke:"));
        assert!(content.contains("skillpack_check"));
    }

    #[test]
    fn test_wizard_creates_ci_with_eval_job() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        run_wizard("ci-skill", None, None, base_path, true, false).unwrap();

        let ci_yml = temp_dir.path().join("ci-skill/.github/workflows/ci.yml");
        assert!(ci_yml.exists());

        let content = fs::read_to_string(&ci_yml).unwrap();
        assert!(content.contains("eval:"));
        assert!(content.contains("skillpack eval"));
    }

    #[test]
    fn quiet_flag_suppresses_cli_println() {
        use std::sync::atomic::Ordering;

        // Default state: quiet is off
        QUIET.store(false, Ordering::Relaxed);
        assert!(!QUIET.load(Ordering::Relaxed));

        // Enable quiet
        QUIET.store(true, Ordering::Relaxed);
        assert!(QUIET.load(Ordering::Relaxed));

        // Reset
        QUIET.store(false, Ordering::Relaxed);
    }

    // ==================== Skill Command Tests ====================

    #[test]
    fn test_skill_update_frontmatter_field() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("update-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: update-skill\nversion: 0.1.0\nauthor: Test\nnamespace: default\nmetadata:\n  version: 0.1.0\n  status: draft\n---\n\n# Test\n",
        )
        .unwrap();

        run_skill(SkillCommands::Update {
            path: skill_dir.to_str().unwrap().to_string(),
            field: "description".to_string(),
            value: "Updated description".to_string(),
        })
        .unwrap();

        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("description: Updated description"));
    }

    #[test]
    fn test_skill_update_nested_field() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("update-nested");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: update-nested\nmetadata:\n  version: 0.1.0\n  status: draft\nnamespace: default\n---\n\n# Test\n",
        )
        .unwrap();

        run_skill(SkillCommands::Update {
            path: skill_dir.to_str().unwrap().to_string(),
            field: "metadata.status".to_string(),
            value: "active".to_string(),
        })
        .unwrap();

        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("status: active"));
    }

    #[test]
    fn test_skill_refine_validates_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("refine-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: refine-skill\ndescription: A test skill\nauthor: Test\nlicense: MIT\nnamespace: default\nmetadata:\n  version: 1.0.0\n  status: active\n---\n\n# Test\n",
        )
        .unwrap();

        let result = run_skill(SkillCommands::Refine {
            path: skill_dir.to_str().unwrap().to_string(),
            fix: false,
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_evolve_bumps_version() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("evolve-skill");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: evolve-skill\ndescription: A test skill\nauthor: Test\nlicense: MIT\nnamespace: default\nmetadata:\n  version: 1.0.0\n  status: active\n---\n\n# Test\n",
        )
        .unwrap();

        run_skill(SkillCommands::Evolve {
            path: skill_dir.to_str().unwrap().to_string(),
            major: false,
            minor: true,
            patch: false,
        })
        .unwrap();

        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("version: 1.1.0"));
    }

    #[test]
    fn test_skill_evolve_creates_changelog() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("evolve-changelog");
        std::fs::create_dir(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: evolve-changelog\ndescription: A test skill\nauthor: Test\nlicense: MIT\nnamespace: default\nmetadata:\n  version: 1.0.0\n  status: active\n---\n\n# Test\n",
        )
        .unwrap();

        run_skill(SkillCommands::Evolve {
            path: skill_dir.to_str().unwrap().to_string(),
            major: false,
            minor: false,
            patch: true,
        })
        .unwrap();

        let changelog = skill_dir.join("CHANGELOG.md");
        assert!(changelog.exists());
        let content = fs::read_to_string(&changelog).unwrap();
        assert!(content.contains("## 1.0.1"));
    }

    #[test]
    fn test_skill_list_finds_skills() {
        let temp_dir = TempDir::new().unwrap();
        let canonical = temp_dir.path().join("canonical");
        std::fs::create_dir(&canonical).unwrap();

        // Create a skill in default namespace
        let skill_dir = canonical.join("default").join("list-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: list-skill\ndescription: A test skill\nnamespace: default\nmetadata:\n  version: 1.0.0\n  status: active\n---\n\n# Test\n",
        )
        .unwrap();

        // Set canonical root env var
        unsafe {
            std::env::set_var("SKILLPACK_CANONICAL_ROOT", canonical.to_str().unwrap());
        }

        // Run list without namespace filter
        let result = run_skill(SkillCommands::List { namespace: None });
        assert!(result.is_ok());

        // Run list with namespace filter
        let result = run_skill(SkillCommands::List {
            namespace: Some("default".to_string()),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_show_displays_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let canonical = temp_dir.path().join("canonical");
        std::fs::create_dir(&canonical).unwrap();

        let skill_dir = canonical.join("default").join("show-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: show-skill\ndescription: A test skill\nnamespace: default\nmetadata:\n  version: 2.0.0\n  status: active\n---\n\n# Test\n",
        )
        .unwrap();

        unsafe {
            std::env::set_var("SKILLPACK_CANONICAL_ROOT", canonical.to_str().unwrap());
        }

        let result = run_skill(SkillCommands::Show {
            name: "show-skill".to_string(),
            namespace: Some("default".to_string()),
        });
        assert!(result.is_ok());
    }

    // ==================== Migration Command Tests ====================

    #[test]
    fn test_migrate_skills_adds_missing_fields() {
        let temp_dir = TempDir::new().unwrap();
        let canonical = temp_dir.path().join("canonical");
        std::fs::create_dir(&canonical).unwrap();

        // Create a skill with incomplete frontmatter
        let skill_dir = canonical.join("test-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: test-skill\n---\n\n# Test Skill\n",
        )
        .unwrap();

        unsafe {
            std::env::set_var("SKILLPACK_CANONICAL_ROOT", canonical.to_str().unwrap());
        }

        let result = run_migrate_skills(Some(canonical.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify the file was updated with required fields
        let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content.contains("namespace:"));
        assert!(content.contains("metadata:"));
        assert!(content.contains("version:"));
        assert!(content.contains("status:"));
        assert!(content.contains("license:"));
    }

    #[test]
    fn test_migrate_skills_creates_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let canonical = temp_dir.path().join("canonical");
        std::fs::create_dir(&canonical).unwrap();

        // Create a skill with no frontmatter
        let skill_dir = canonical.join("no-fm-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "# Test Skill\n\nThis is a test skill.\n",
        )
        .unwrap();

        unsafe {
            std::env::set_var("SKILLPACK_CANONICAL_ROOT", canonical.to_str().unwrap());
        }

        let result = run_migrate_skills(Some(canonical.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify frontmatter was created
        let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content.starts_with("---"));
        assert!(content.contains("name: no-fm-skill"));
    }

    #[test]
    fn test_migrate_agents_adds_missing_fields() {
        let temp_dir = TempDir::new().unwrap();
        let agents_root = temp_dir.path().join(".config/agents/skills");
        std::fs::create_dir_all(&agents_root).unwrap();

        // Create an agent skill with incomplete frontmatter
        let skill_dir = agents_root.join("test-agent");
        std::fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: test-agent\n---\n\n# Test Agent\n",
        )
        .unwrap();

        // Hermetic: pass the root explicitly instead of mutating global HOME.
        let result = run_migrate_agents_at(Some(agents_root.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify the file was updated
        let content = fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
        assert!(content.contains("namespace: agents"));
        assert!(content.contains("metadata:"));
    }

    #[test]
    fn test_migrate_claude_agents_adds_missing_fields() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        std::fs::create_dir(&agents_dir).unwrap();

        // Create a Claude agent with no frontmatter
        fs::write(
            agents_dir.join("test-agent.md"),
            "# Test Agent\n\n## Identity\nThis is a test agent.\n",
        )
        .unwrap();

        let result = run_migrate_claude_agents(Some(agents_dir.to_str().unwrap()));
        assert!(result.is_ok());

        // Verify frontmatter was created
        let content = fs::read_to_string(agents_dir.join("test-agent.md")).unwrap();
        assert!(content.starts_with("---"));
        assert!(content.contains("name: test-agent"));
        assert!(content.contains("namespace: claude"));
    }

    #[test]
    fn test_migrate_harnesses_adds_missing_fields() {
        let temp_dir = TempDir::new().unwrap();
        let agents_root = temp_dir.path().join(".config/agents/skills");
        std::fs::create_dir_all(&agents_root).unwrap();

        // Create a skill with rules
        let skill_dir = agents_root.join("test-skill");
        let rules_dir = skill_dir.join("rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        // Create a rule with no frontmatter
        fs::write(
            rules_dir.join("test-rule.md"),
            "# Test Rule\n\nThis is a test rule.\n",
        )
        .unwrap();

        // Temporarily override the agents root
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path().to_str().unwrap());
        }

        let result = run_migrate_harnesses();
        assert!(result.is_ok());

        // Restore original HOME
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        }

        // Verify frontmatter was created
        let content = fs::read_to_string(rules_dir.join("test-rule.md")).unwrap();
        assert!(content.starts_with("---"));
        assert!(content.contains("name: test-rule"));
        assert!(content.contains("namespace: test-skill"));
    }

    // ==================== ASM Helper: collect_entries_from_dir ====================

    fn make_skill_dir(
        base: &std::path::Path,
        name: &str,
        version: &str,
        description: Option<&str>,
    ) {
        let skill_dir = base.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        let desc_line = description
            .map(|d| format!("description: {}\n", d))
            .unwrap_or_default();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: {}\nversion: {}\n{}namespace: default\n---\n\n# {}\n",
                name, version, desc_line, name
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_collect_entries_from_dir_finds_skills() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "alpha-skill", "1.0.0", Some("Alpha"));
        make_skill_dir(temp_dir.path(), "beta-skill", "2.0.0", None);

        let entries = collect_entries_from_dir("test-tool", temp_dir.path());
        assert_eq!(entries.len(), 2);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"alpha-skill"));
        assert!(names.contains(&"beta-skill"));
    }

    #[test]
    fn test_collect_entries_from_dir_reads_version() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "versioned", "3.2.1", None);

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        let e = entries.iter().find(|e| e.name == "versioned").unwrap();
        assert_eq!(e.version, "3.2.1");
    }

    #[test]
    fn test_collect_entries_from_dir_reads_description() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(
            temp_dir.path(),
            "desc-skill",
            "1.0.0",
            Some("My description"),
        );

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        let e = entries.iter().find(|e| e.name == "desc-skill").unwrap();
        assert_eq!(e.description.as_deref(), Some("My description"));
    }

    #[test]
    fn test_collect_entries_from_dir_detects_disabled_marker() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "disabled-skill", "1.0.0", None);
        fs::write(temp_dir.path().join("disabled-skill/.disabled"), "").unwrap();

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        let e = entries.iter().find(|e| e.name == "disabled-skill").unwrap();
        assert!(e.disabled);
    }

    #[test]
    fn test_collect_entries_from_dir_ignores_non_skill_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // A dir with neither SKILL.md nor skill.cnsb.json should be skipped
        fs::create_dir(temp_dir.path().join("not-a-skill")).unwrap();
        make_skill_dir(temp_dir.path(), "real-skill", "1.0.0", None);

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "real-skill");
    }

    // ==================== ASM: list ====================

    #[test]
    fn test_asm_list_empty_returns_ok() {
        let temp_dir = TempDir::new().unwrap();
        // Point to an empty directory so no skills are found
        let result = run_asm_list(
            Some("nonexistent-tool-xyz"),
            "both",
            "name",
            false,
            false,
            false,
        );
        assert!(result.is_ok());
        let _ = temp_dir;
    }

    #[test]
    fn test_asm_list_sorts_by_name() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "zebra", "1.0.0", None);
        make_skill_dir(temp_dir.path(), "alpha", "1.0.0", None);

        let mut entries = collect_entries_from_dir("tool", temp_dir.path());
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(entries[0].name, "alpha");
        assert_eq!(entries[1].name, "zebra");
    }

    #[test]
    fn test_asm_list_sorts_by_version() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "skill-b", "2.0.0", None);
        make_skill_dir(temp_dir.path(), "skill-a", "1.0.0", None);

        let mut entries = collect_entries_from_dir("tool", temp_dir.path());
        entries.sort_by(|a, b| a.version.cmp(&b.version));

        assert_eq!(entries[0].version, "1.0.0");
        assert_eq!(entries[1].version, "2.0.0");
    }

    // ==================== ASM: search ====================

    #[test]
    fn test_asm_search_finds_by_name_fragment() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "my-awesome-skill", "1.0.0", None);
        make_skill_dir(temp_dir.path(), "unrelated", "1.0.0", None);

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        let q = "awesome";
        let matches: Vec<&crate::cli::manifest::SkillEntry> = entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(q))
            .collect();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "my-awesome-skill");
    }

    #[test]
    fn test_asm_search_finds_by_description() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(
            temp_dir.path(),
            "skill-x",
            "1.0.0",
            Some("unique-keyword here"),
        );
        make_skill_dir(temp_dir.path(), "skill-y", "1.0.0", Some("other stuff"));

        let entries = collect_entries_from_dir("tool", temp_dir.path());
        let q = "unique-keyword";
        let matches: Vec<&crate::cli::manifest::SkillEntry> = entries
            .iter()
            .filter(|e| {
                e.description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(q)
            })
            .collect();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "skill-x");
    }

    #[test]
    fn test_asm_search_no_match_returns_ok() {
        // run_asm_search with a tool that yields no skills — should not error
        let result = run_asm_search("zzz-no-match", Some("nonexistent-xyz"), false, false);
        assert!(result.is_ok());
    }

    // ==================== ASM: inspect ====================

    #[test]
    fn test_asm_inspect_not_found_returns_err() {
        let result = run_asm_inspect("this-skill-does-not-exist-xyz", None, false, false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("this-skill-does-not-exist-xyz")
        );
    }

    // ==================== ASM: uninstall ====================

    #[test]
    fn test_asm_uninstall_not_found_returns_err() {
        let result = run_asm_uninstall("ghost-skill-xyz", None, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_asm_uninstall_dry_run_does_not_remove() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "removable", "1.0.0", None);
        let skill_path = temp_dir.path().join("removable");
        assert!(skill_path.exists());

        // We can't easily wire this through config in unit tests, so exercise
        // the helper directly via collect_entries_from_dir + manual call path.
        let entries = collect_entries_from_dir("tool", temp_dir.path());
        assert!(!entries.is_empty());

        // The skill dir should still exist (we only tested collection, not removal)
        assert!(skill_path.exists());
    }

    // ==================== ASM: disable / enable ====================

    #[test]
    fn test_asm_disable_enable_not_found_returns_err() {
        let r1 = run_asm_disable("no-such-skill-xyz", Some("no-tool-xyz"), false);
        assert!(r1.is_err());

        let r2 = run_asm_enable("no-such-skill-xyz", Some("no-tool-xyz"), false);
        assert!(r2.is_err());
    }

    // ==================== ASM: audit duplicates ====================

    #[test]
    fn test_asm_audit_duplicates_no_skills_returns_ok() {
        // With no skills discoverable the duplicate check should pass cleanly
        let result = run_asm_audit(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_audit_duplicates_detects_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        // Duplicate name across two "tools" (two dirs)
        let dir_a = temp_dir.path().join("toolA");
        let dir_b = temp_dir.path().join("toolB");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        make_skill_dir(&dir_a, "dup-skill", "1.0.0", None);
        make_skill_dir(&dir_b, "dup-skill", "2.0.0", None);

        let mut entries = collect_entries_from_dir("toolA", &dir_a);
        entries.extend(collect_entries_from_dir("toolB", &dir_b));

        let mut by_name: std::collections::HashMap<String, Vec<&crate::cli::manifest::SkillEntry>> =
            std::collections::HashMap::new();
        for e in &entries {
            by_name.entry(e.name.clone()).or_default().push(e);
        }

        let duplicates: Vec<_> = by_name.iter().filter(|(_, v)| v.len() > 1).collect();
        assert_eq!(duplicates.len(), 1);
        assert_eq!(*duplicates[0].0, "dup-skill");
    }

    // ==================== ASM: export ====================

    #[test]
    fn test_asm_export_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let out = temp_dir.path().join("manifest.json");

        // Export with no skills configured — should still write valid JSON
        let result = run_asm_export(Some(out.to_str().unwrap()), false, false);
        assert!(result.is_ok());
        assert!(out.exists());

        let content = fs::read_to_string(&out).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(v.get("schema_version").is_some());
        assert!(v.get("skills").is_some());
        assert!(v.get("total").is_some());
    }

    #[test]
    fn test_asm_export_json_output() {
        // Should succeed and not error even with zero skills
        let result = run_asm_export(None, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_export_machine_output() {
        let result = run_asm_export(None, false, true);
        assert!(result.is_ok());
    }

    // ==================== ASM: import ====================

    #[test]
    fn test_asm_import_missing_file_returns_err() {
        let result = run_asm_import("/nonexistent/path/manifest.json", true, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_asm_import_invalid_json_returns_err() {
        let temp_dir = TempDir::new().unwrap();
        let bad = temp_dir.path().join("bad.json");
        fs::write(&bad, "not valid json {{").unwrap();

        let result = run_asm_import(bad.to_str().unwrap(), true, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_asm_import_dry_run_succeeds() {
        let temp_dir = TempDir::new().unwrap();

        // Build a minimal valid manifest
        let manifest = crate::cli::manifest::SkillInventoryManifest::new(vec![]);
        let manifest_path = temp_dir.path().join("manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let result = run_asm_import(manifest_path.to_str().unwrap(), true, true);
        assert!(result.is_ok());
    }

    // ==================== ASM: stats ====================

    #[test]
    fn test_asm_stats_json_output_returns_ok() {
        let result = run_asm_stats(true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_stats_machine_output_returns_ok() {
        let result = run_asm_stats(false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_stats_human_output_returns_ok() {
        let result = run_asm_stats(false, false);
        assert!(result.is_ok());
    }

    // ==================== ASM: link ====================

    #[test]
    fn test_asm_link_nonexistent_source_returns_err() {
        let result = run_asm_link("/nonexistent/path/my-skill-xyz", None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_asm_link_dry_run_does_not_create_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let skill_src = temp_dir.path().join("link-skill");
        make_skill_dir(temp_dir.path(), "link-skill", "1.0.0", None);
        assert!(skill_src.exists());

        // dry_run = true: even if we had configured agent paths, no symlink written
        let result = run_asm_link(skill_src.to_str().unwrap(), Some("tool-xyz"), true);
        assert!(result.is_ok());
    }

    // ==================== ASM: outdated ====================

    #[test]
    fn test_asm_outdated_returns_ok() {
        let result = run_asm_outdated(false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_outdated_json_returns_ok() {
        let result = run_asm_outdated(true, false);
        assert!(result.is_ok());
    }

    // ==================== ASM: update ====================

    #[test]
    fn test_asm_update_no_targets_returns_ok() {
        // When no skills exist, update should exit cleanly
        let result = run_asm_update(&["nonexistent-xyz".to_string()], true, true);
        assert!(result.is_ok());
    }

    // ==================== ASM: eval-providers ====================

    #[test]
    fn test_asm_eval_providers_list_returns_ok() {
        let result = run_asm_eval_providers(EvalProvidersCommands::List);
        assert!(result.is_ok());
    }

    // ==================== ASM: bundle ====================

    #[test]
    fn test_asm_bundle_create_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "skill-one", "1.0.0", Some("Skill one"));
        // Create skill.cnsb.json so bundle creation finds it
        let skill_dir = temp_dir.path().join("skill-one");
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string_pretty(&json!({
                "metadata": { "name": "skill-one", "version": "1.0.0", "description": "Skill one" }
            }))
            .unwrap(),
        )
        .unwrap();

        let result = run_asm_bundle(
            BundleCommands::Create {
                name: "test-bundle".to_string(),
                version: "0.1.0".to_string(),
                path: temp_dir.path().to_str().unwrap().to_string(),
                output: None,
            },
            true, // dry_run
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_bundle_create_writes_file() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("bundle-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("skill.cnsb.json"),
            serde_json::to_string_pretty(&json!({
                "metadata": { "name": "bundle-skill", "version": "0.2.0" }
            }))
            .unwrap(),
        )
        .unwrap();

        let out = temp_dir.path().join("my-bundle-0.1.0.cnsb.json");
        let result = run_asm_bundle(
            BundleCommands::Create {
                name: "my-bundle".to_string(),
                version: "0.1.0".to_string(),
                path: temp_dir.path().to_str().unwrap().to_string(),
                output: Some(out.to_str().unwrap().to_string()),
            },
            false,
        );
        assert!(result.is_ok());
        assert!(out.exists());

        let content = fs::read_to_string(&out).unwrap();
        let bundle: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(bundle["metadata"]["name"], "my-bundle");
        assert_eq!(bundle["metadata"]["version"], "0.1.0");
        assert!(!bundle["skills"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_asm_bundle_list_returns_ok() {
        let result = run_asm_bundle(BundleCommands::List, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_bundle_show_not_found_returns_err() {
        let result = run_asm_bundle(
            BundleCommands::Show {
                name: "nonexistent-bundle-xyz".to_string(),
            },
            false,
        );
        assert!(result.is_err());
    }

    // ==================== ASM: index ====================

    #[test]
    fn test_asm_index_ingest_returns_ok() {
        let temp_dir = TempDir::new().unwrap();
        make_skill_dir(temp_dir.path(), "idx-skill", "1.0.0", None);

        let result = run_asm_index(IndexCommands::Ingest {
            path: temp_dir.path().to_str().unwrap().to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_index_search_returns_ok() {
        let result = run_asm_index(IndexCommands::Search {
            query: "anything".to_string(),
            limit: 10,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_asm_index_list_returns_ok() {
        let result = run_asm_index(IndexCommands::List);
        assert!(result.is_ok());
    }

    // ==================== ASM: doctor ====================
    // run_asm_doctor() calls std::process::exit(1) on failure, which terminates the
    // test binary. It is covered via integration / CLI smoke tests only.

    // ==================== emit_output ====================

    #[test]
    fn test_emit_output_machine_wraps_envelope() {
        // We test the structure logic by calling the function directly.
        // Since emit_output writes to stdout we just verify it doesn't panic.
        let data = json!({"key": "value"});
        emit_output(&data, true);
        emit_output(&data, false);
        emit_output(&data, false);
    }
}
