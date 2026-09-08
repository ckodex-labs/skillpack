//! skillpack-registry-sync — centralized OCI 1.2 skill registry
//!
//! Provides:
//! - Multi-source discovery across all 19 AI coding assistants
//! - SHA-256 content deduplication
//! - OCI push/pull via Zot (local) or any OCI 1.2 registry
//! - Per-agent rendering (Symlink, IndexFile, RulesDir, SingleFile)

pub mod agent_render;
pub mod dedup;
pub mod migrate;
pub mod multi_source;
pub mod oci_sync;
pub mod sync_engine;
