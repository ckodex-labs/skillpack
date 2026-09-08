//! sctl — SkillPack Control CLI
//!
//! Centralized OCI 1.2 registry management for AI coding assistant skills.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

use skillpack_registry_sync::{
    dedup::dedup,
    migrate::{MigrateOptions, run_migrate},
    multi_source::{discover_all, discover_shared},
    oci_sync::RegistryConfig,
    sync_engine::{SyncOptions, run_sync},
};

#[derive(Parser)]
#[command(
    name = "sctl",
    version,
    about = "SkillPack Control — OCI 1.2 skill registry manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Canonical store root (default: ~/skills/shared).
    #[arg(long, global = true, env = "SKILLPACK_ROOT")]
    root: Option<PathBuf>,

    /// OCI registry endpoint for push/pull (e.g. localhost:5000).
    #[arg(long, global = true, env = "SKILLPACK_OCI_REGISTRY")]
    registry: Option<String>,

    /// Dry-run: print what would happen without making changes.
    #[arg(long, short = 'n', global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover all skills across every agent path and print a report.
    Discover,
    /// Deduplicate discovered skills and report collisions.
    Dedup,
    /// Push canonical skills to the OCI registry.
    Push,
    /// Pull skills from OCI registry to canonical root.
    Pull {
        /// Skill name to pull (omit for all).
        #[arg(long)]
        skill: Option<String>,
    },
    /// Full sync: discover → dedup → push → render to all agents.
    Sync {
        /// Only render for this agent.
        #[arg(long)]
        agent: Option<String>,
    },
    /// List all registered agents and their paths.
    Agents,
    /// Migrate all skills in the canonical store to schema compliance.
    ///
    /// Fixes name mismatches, CRLF line endings, and creates missing SKILL.md stubs.
    /// Safe to re-run: already-compliant skills are left untouched.
    Migrate {
        /// Rename directories with invalid names (e.g. dots → hyphens).
        #[arg(long)]
        rename_dirs: bool,
    },
    /// Audit canonical store and report compliance issues without fixing them.
    Audit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let root = resolve_root(cli.root);

    match cli.command {
        Commands::Discover => cmd_discover(&root),
        Commands::Dedup => cmd_dedup(&root),
        Commands::Push => cmd_push(&root, cli.registry.as_deref()).await,
        Commands::Pull { skill } => {
            cmd_pull(skill.as_deref(), &root, cli.registry.as_deref()).await
        }
        Commands::Sync { agent } => {
            cmd_sync(
                &root,
                agent.as_deref(),
                cli.registry.as_deref(),
                cli.dry_run,
            )
            .await
        }
        Commands::Agents => cmd_agents(),
        Commands::Migrate { rename_dirs } => cmd_migrate(&root, rename_dirs, cli.dry_run),
        Commands::Audit => cmd_audit(&root),
    }
}

fn resolve_root(arg: Option<PathBuf>) -> PathBuf {
    if let Some(p) = arg {
        return p;
    }
    if let Ok(v) = std::env::var("SKILLPACK_ROOT") {
        return PathBuf::from(v);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    home.join("skills").join("shared")
}

fn cmd_discover(root: &PathBuf) -> Result<()> {
    let mut skills = discover_all();
    skills.extend(discover_shared(std::slice::from_ref(root)));

    println!(
        "{}",
        format!("Discovered {} skills across all agents:", skills.len()).bold()
    );
    for s in &skills {
        println!(
            "  {} ({}) — {}",
            s.name.cyan(),
            s.source_agent.blue(),
            s.path.display()
        );
    }
    Ok(())
}

fn cmd_dedup(root: &PathBuf) -> Result<()> {
    let mut raw = discover_all();
    raw.extend(discover_shared(std::slice::from_ref(root)));
    raw.sort_by(|a, b| a.name.cmp(&b.name));

    let pairs: Vec<_> = raw.into_iter().map(|s| (s.name, s.path)).collect();
    let report = dedup(pairs);

    println!("{}", "Deduplication report:".to_string().bold());
    println!("  Total scanned:      {}", report.total_scanned);
    println!("  Unique (canonical): {}", report.unique.len());
    println!(
        "  Duplicates removed: {}",
        report.duplicate_count.to_string().yellow()
    );

    for ds in &report.unique {
        if !ds.aliases.is_empty() {
            println!(
                "  {} {} has {} alias(es):",
                "→".green(),
                ds.canonical_name.cyan(),
                ds.aliases.len()
            );
            for (alias, path) in &ds.aliases {
                println!("    {} ({})", alias.dimmed(), path.display());
            }
        }
    }
    Ok(())
}

async fn cmd_push(root: &Path, registry_arg: Option<&str>) -> Result<()> {
    let oci = registry_arg
        .map(|r| RegistryConfig {
            endpoint: r.to_string(),
            ..RegistryConfig::local_zot()
        })
        .unwrap_or_else(RegistryConfig::local_zot);

    let opts = SyncOptions {
        canonical_root: root.to_path_buf(),
        extra_roots: vec![],
        oci: Some(oci),
        only_agent: None,
        dry_run: false,
    };
    let summary = run_sync(&opts).await?;
    println!(
        "{} Pushed {} skills to OCI registry",
        "✓".green(),
        summary.oci_pushed
    );
    Ok(())
}

async fn cmd_pull(_skill: Option<&str>, root: &Path, _registry: Option<&str>) -> Result<()> {
    println!(
        "Pull: OCI pull into {} not yet wired (add --registry and skill tag map)",
        root.display()
    );
    Ok(())
}

async fn cmd_sync(
    root: &Path,
    only_agent: Option<&str>,
    registry_arg: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let oci = registry_arg.map(|r| RegistryConfig {
        endpoint: r.to_string(),
        ..RegistryConfig::local_zot()
    });

    let opts = SyncOptions {
        canonical_root: root.to_path_buf(),
        extra_roots: vec![home.join(".agents").join("skills")],
        oci,
        only_agent: only_agent.map(str::to_string),
        dry_run,
    };

    let summary = run_sync(&opts).await?;
    println!("{}", "Sync complete:".bold());
    println!("  Skills discovered:  {}", summary.skills_discovered);
    println!(
        "  After dedup:        {}",
        summary.skills_after_dedup.to_string().green()
    );
    println!(
        "  Duplicates removed: {}",
        summary.duplicates_removed.to_string().yellow()
    );
    println!(
        "  Agents synced:      {}",
        summary.agents_synced.to_string().green()
    );
    if summary.agents_skipped > 0 {
        println!(
            "  Agents skipped:     {}",
            summary.agents_skipped.to_string().dimmed()
        );
    }
    if summary.oci_pushed > 0 {
        println!(
            "  OCI pushed:         {}",
            summary.oci_pushed.to_string().cyan()
        );
    }
    Ok(())
}

fn cmd_migrate(root: &Path, rename_dirs: bool, dry_run: bool) -> Result<()> {
    let label = if dry_run { "[dry-run] " } else { "" };
    println!(
        "{}{}",
        label.yellow(),
        format!(
            "Migrating skills in {} to schema compliance…",
            root.display()
        )
        .bold()
    );

    let opts = MigrateOptions {
        root: root.to_path_buf(),
        dry_run,
        rename_invalid_dirs: rename_dirs,
    };
    let summary = run_migrate(&opts)?;

    println!();
    println!("{}", "Migration complete:".bold());
    println!("  Skills scanned:      {}", summary.skills_scanned);
    println!(
        "  Already compliant:   {}",
        summary.already_compliant.to_string().green()
    );
    if summary.names_fixed > 0 {
        println!(
            "  Names fixed:         {}",
            summary.names_fixed.to_string().yellow()
        );
    }
    if summary.crlf_fixed > 0 {
        println!(
            "  CRLF → LF:           {}",
            summary.crlf_fixed.to_string().yellow()
        );
    }
    if summary.missing_created > 0 {
        println!(
            "  SKILL.md created:    {}",
            summary.missing_created.to_string().cyan()
        );
    }
    if summary.dirs_renamed > 0 {
        println!(
            "  Dirs renamed:        {}",
            summary.dirs_renamed.to_string().cyan()
        );
    }
    if !summary.errors.is_empty() {
        println!("  {} errors:", "⚠".red());
        for e in &summary.errors {
            println!("    {} {}", "✗".red(), e);
        }
    }
    Ok(())
}

fn cmd_audit(root: &Path) -> Result<()> {
    use skillpack_registry_sync::migrate::{MigrateOptions, run_migrate};
    let opts = MigrateOptions {
        root: root.to_path_buf(),
        dry_run: true,
        rename_invalid_dirs: false,
    };
    let summary = run_migrate(&opts)?;
    let total_issues = summary.names_fixed + summary.crlf_fixed + summary.missing_created;
    println!();
    println!("{}", "Audit summary:".bold());
    println!("  Skills scanned:  {}", summary.skills_scanned);
    println!(
        "  Compliant:       {}",
        summary.already_compliant.to_string().green()
    );
    if total_issues == 0 {
        println!("  {} All skills are schema-compliant!", "✓".green());
    } else {
        println!("  {} issues found:", total_issues.to_string().red());
        if summary.names_fixed > 0 {
            println!("    Name mismatches:  {}", summary.names_fixed);
        }
        if summary.crlf_fixed > 0 {
            println!("    CRLF files:       {}", summary.crlf_fixed);
        }
        if summary.missing_created > 0 {
            println!("    Missing SKILL.md: {}", summary.missing_created);
        }
        println!("  Run `sctl migrate` to fix all issues.");
    }
    Ok(())
}

fn cmd_agents() -> Result<()> {
    use skillpack_domain::AgentRegistry;
    let registry = AgentRegistry::default_registry();
    println!(
        "{}",
        format!("Registered agents ({}):", registry.agents().len()).bold()
    );
    for agent in registry.agents() {
        let dir = agent.resolved_dir();
        let status = match dir {
            Some(ref d) => format!("✓ {}", d.display()).green().to_string(),
            None => "✗ not found".dimmed().to_string(),
        };
        println!(
            "  {:25} {:12?}  {}",
            agent.name, agent.integration_type, status
        );
    }
    Ok(())
}
