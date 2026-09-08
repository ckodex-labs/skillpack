//! Skill Fabricator - Unified CKODEX Skill Domain Types
//!
//! Re-exports canonical domain types, ports, and OCI adapters from the
//! SkillPack workspace so downstream CKODEX tooling (sctl, SkillIQ, etc.)
//! can depend on a single facade crate.

// Domain types (canonical source of truth)
pub use skillpack_domain::{
    AgentDefinition, BundleAssessment, BundleMetadata, BundleReader, BundleReaderError,
    EvidenceEnvelope, GovernanceConfig, Issue, LifecycleConfig, LifecycleHook, OperationsConfig,
    SbomConfig, Score, Severity, SigningConfig, SkillAssessmentSummary, SkillBundle,
    SkillDefinition, SkillDependency, SkillIdentity, SkillReader, SkillReaderError,
};

// OCI infrastructure
pub use skillpack_adapters::oci::{OciPushResult, OciReader, SignatureVerification, media_types};

// Re-export OciSource from domain for consistency
pub use skillpack_domain::OciSource;
