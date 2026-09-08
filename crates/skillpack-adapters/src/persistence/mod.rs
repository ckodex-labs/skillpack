//! Persistence Adapters
//!
//! DuckDB for high-performance analytics storage.

pub mod duckdb;

pub use self::duckdb::{AgentSyncState, CanonicalSkillRecord, DuckDbRepository, IndexStats};
