//! Port Interfaces (Hexagonal Architecture)
//!
//! Defines interfaces for external adapters.

use crate::envelope::EvidenceEnvelope;
use crate::{Assessment, DimensionId, Issue, Score, SkillIdentity};
use std::path::Path;

/// Port for reading skill pack contents
pub trait SkillReader: Send + Sync {
    /// Read skill identity from SKILL.md or package metadata
    fn read_identity(&self, path: &Path) -> Result<SkillIdentity, SkillReaderError>;

    /// Check if file exists in skill
    fn file_exists(&self, path: &Path, relative: &str) -> bool;

    /// Read file contents
    fn read_file(&self, path: &Path, relative: &str) -> Result<String, SkillReaderError>;

    /// List files matching pattern
    fn list_files(&self, path: &Path, pattern: &str) -> Vec<String>;
}

/// Port for dimension checkers
pub trait DimensionChecker: Send + Sync {
    /// Dimension this checker handles
    fn dimension(&self) -> DimensionId;

    /// Run check and return score + issues
    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>);

    /// Returns true iff this checker is a placeholder with no content analysis.
    /// Phases 1–9 will flip this to false as real content checks ship.
    fn is_stub(&self) -> bool {
        false
    }
}

/// Port for report generation
pub trait ReportGenerator: Send + Sync {
    /// Generate report from assessment
    fn generate(&self, assessment: &Assessment) -> Result<String, ReportError>;

    /// Report format (e.g., "json", "sarif", "markdown")
    fn format(&self) -> &'static str;
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum SkillReaderError {
    #[error("Skill not found: {0}")]
    NotFound(String),
    #[error("Failed to read file: {0}")]
    ReadError(String),
    #[error("Invalid skill metadata: {0}")]
    InvalidMetadata(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Failed to generate report: {0}")]
    GenerationError(String),
}

/// Port for reading skill bundles (CNSB)
pub trait BundleReader: Send + Sync {
    /// Read bundle from CNSB JSON file
    fn read_bundle(&self, path: &Path) -> Result<crate::SkillBundle, BundleReaderError>;

    /// List all CNSB files in directory
    fn list_bundles(&self, path: &Path) -> Vec<String>;
}

#[derive(Debug, thiserror::Error)]
pub enum BundleReaderError {
    #[error("Bundle not found: {0}")]
    NotFound(String),
    #[error("Invalid bundle format: {0}")]
    InvalidFormat(String),
    #[error("Failed to read bundle: {0}")]
    ReadError(String),
}

/// Port for signing evidence envelopes
pub trait Signer: Send + Sync {
    fn sign(&self, envelope: EvidenceEnvelope) -> Result<EvidenceEnvelope, SignerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    #[error("OIDC unavailable: {0}")]
    OidcUnavailable(String),
    #[error("key access denied: {0}")]
    KeyAccessDenied(String),
    #[error("signing failed: {0}")]
    SigningFailed(String),
}

/// Port for emitting evidence to a sink
pub trait EvidenceSink: Send + Sync {
    fn write(&self, envelope: &EvidenceEnvelope) -> Result<std::path::PathBuf, SinkError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("write failed: {0}")]
    WriteError(String),
}
