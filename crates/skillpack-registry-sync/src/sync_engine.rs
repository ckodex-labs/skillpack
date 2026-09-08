//! Central sync orchestrator.
//!
//! Sequence:
//!   1. discover   — scan all agent paths + shared roots
//!   2. dedup      — collapse identical content to one canonical entry
//!   3. push       — (optional) upload to OCI registry
//!   4. render     — write to each agent-specific path

use anyhow::Result;
use skillpack_domain::{AgentRegistry, CanonicalSkill};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::agent_render::render_for_agent;
use crate::dedup::{DedupReport, dedup};
use crate::multi_source::{DiscoveredSkill, discover_all, discover_shared};
use crate::oci_sync::{RegistryConfig, push_skill};

/// Input for a sync run.
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Canonical store root (e.g. `~/skills/shared`).
    pub canonical_root: PathBuf,
    /// Additional roots to scan (e.g. `~/.agents/skills`).
    pub extra_roots: Vec<PathBuf>,
    /// OCI registry config — if None, OCI push is skipped.
    pub oci: Option<RegistryConfig>,
    /// Only render for this agent (None = all agents).
    pub only_agent: Option<String>,
    /// Dry-run: discover and report without writing.
    pub dry_run: bool,
}

impl SyncOptions {
    pub fn default_with_root(canonical_root: PathBuf) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self {
            canonical_root,
            extra_roots: vec![home.join(".agents").join("skills")],
            oci: None,
            only_agent: None,
            dry_run: false,
        }
    }
}

/// Summary of a sync run.
#[derive(Debug, Default)]
pub struct SyncSummary {
    pub skills_discovered: usize,
    pub skills_after_dedup: usize,
    pub duplicates_removed: usize,
    pub agents_synced: usize,
    pub agents_skipped: usize,
    pub oci_pushed: usize,
}

/// Run the full sync pipeline.
pub async fn run_sync(opts: &SyncOptions) -> Result<SyncSummary> {
    let mut summary = SyncSummary::default();

    // 1. Discover
    let mut raw: Vec<DiscoveredSkill> = discover_all();
    raw.extend(discover_shared(&opts.extra_roots));
    raw.extend(discover_shared(std::slice::from_ref(&opts.canonical_root)));
    summary.skills_discovered = raw.len();
    info!(total = raw.len(), "discovered skills from all agent paths");

    // Sort for stable canonical choice (alphabetical by name, then by path).
    raw.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));

    // 2. Dedup
    let skill_pairs: Vec<(String, PathBuf)> = raw
        .iter()
        .map(|s| (s.name.clone(), s.path.clone()))
        .collect();
    let report: DedupReport = dedup(skill_pairs);
    summary.duplicates_removed = report.duplicate_count;
    summary.skills_after_dedup = report.unique.len();

    if report.duplicate_count > 0 {
        warn!(
            duplicates = report.duplicate_count,
            "collapsed duplicate skills"
        );
    }

    // Build canonical skill list
    let canonical: Vec<CanonicalSkill> = report
        .unique
        .iter()
        .map(|ds| CanonicalSkill {
            name: ds.canonical_name.clone(),
            path: ds.canonical_path.clone(),
            has_skill_md: ds.canonical_path.join("SKILL.md").exists(),
            content_hash: Some(ds.content_hash.clone()),
            source_agent: None,
        })
        .collect();

    // 3. OCI push (optional)
    if let Some(ref oci) = opts.oci
        && !opts.dry_run
    {
        for ds in &report.unique {
            match push_skill(ds, oci).await {
                Ok(_) => summary.oci_pushed += 1,
                Err(e) => warn!(skill = %ds.canonical_name, error = %e, "OCI push failed"),
            }
        }
    }

    // 4. Render per agent
    if opts.dry_run {
        info!("dry-run: would render {} skills to agents", canonical.len());
        return Ok(summary);
    }

    let registry = AgentRegistry::default_registry();
    for agent in registry.agents() {
        if let Some(ref only) = opts.only_agent
            && agent.name.to_lowercase() != only.to_lowercase()
        {
            continue;
        }

        match render_for_agent(agent, &canonical, &opts.canonical_root) {
            Ok(n) => {
                info!(agent = %agent.name, skills_rendered = n, "rendered");
                summary.agents_synced += 1;
            }
            Err(e) => {
                warn!(agent = %agent.name, error = %e, "render skipped");
                summary.agents_skipped += 1;
            }
        }
    }

    Ok(summary)
}
