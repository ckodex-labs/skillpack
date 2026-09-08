//! SkillPack Adapters - Infrastructure Layer
//!
//! Implements ports defined in domain layer.

pub mod behavioral;
pub mod canonical;
pub mod checkers;
pub mod cli;
pub mod evidence;
pub mod filesystem;
pub mod grpc_client;
pub mod lock;
pub mod reporters;
pub mod skill_migration;

// Generated canonical client model (from client-model.schema.json)
pub mod generated {
    pub mod client_model;
}

// Remote source adapters
pub mod git;
pub mod oci;
pub mod s3;
pub mod sftp;
pub mod source_factory;

// Discovery
pub mod discovery;

// CLI configuration
pub mod config;

// Remote registry client
pub mod registry_client;

// MCP server
pub mod mcp;

// Index persistence
pub mod index_repository;

// Persistence layer (LanceDB + DuckDB)
pub mod persistence;

// Re-export reader alias for CLI
pub mod reader {
    pub use crate::filesystem::FilesystemReader as FsSkillReader;
}

pub use discovery::{DiscoveredSkill, SkillTxt, SkillsDiscovery};
pub use filesystem::FilesystemReader;
pub use git::GitReader;
pub use index_repository::JsonIndexRepository;
pub use mcp::{McpResource, McpServerConfig, McpTool};
pub use oci::OciReader;
pub use persistence::{
    AgentSyncState, CanonicalSkillRecord, CanonicalSkillUpsert, DuckDbRepository, IndexStats,
};
pub use registry_client::{FederationCoordinator, RegistryClient, RemoteRegistry, TrustLevel};
pub use s3::S3Reader;
pub use sftp::SftpReader;
pub use source_factory::{SkillsReader, SourceBuilder};
