//! EvidenceEnvelope domain type
//!
//! Signed evidence output substrate. Serializes to JSON that validates against
//! `schemas/evidence/v1/envelope.schema.json` (OpenEvidence Envelope v1).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subject of the evidence — the artifact being assessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// URN identifying the subject artifact.
    pub urn: String,
    /// Classifier (e.g. "SkillBundle", "Model", "Dataset", "AgentApp", "Run").
    pub kind: String,
    /// Optional key-value labels attached to the subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

/// Evidence statement describing what was assessed and the outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    /// Logical statement type (e.g. "SkillAssessment", "BuildProvenance").
    #[serde(rename = "type")]
    pub type_: String,
    /// Optional URI of a JSON Schema describing the payload structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Evidence payload; structure depends on `type_` and optional `schema`.
    pub payload: serde_json::Value,
}

/// Actor that produced this evidence envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Kind of actor: "agent", "human", "service", or "machine".
    /// Serializes as `type` to match the JSON schema; `kind` is accepted as a deser alias.
    #[serde(rename = "type", alias = "kind")]
    pub kind: String,
    /// Identifier of the actor (URN).
    /// Serializes as `urn` to match the JSON schema; `id` is accepted as a deser alias.
    #[serde(rename = "urn", alias = "id")]
    pub id: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Optional tenant URN.
    #[serde(skip_serializing_if = "Option::is_none", rename = "tenantUrn")]
    pub tenant_urn: Option<String>,
    /// Optional labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

/// A cryptographic signature over the envelope content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Key identifier (e.g. a cosign key ID or Rekor log ID).
    /// Serializes as `keyId` to match the JSON schema.
    #[serde(rename = "keyId")]
    pub keyid: String,
    /// Base64-encoded signature bytes.
    /// Serializes as `value` to match the JSON schema; `sig` is accepted as a deser alias.
    #[serde(rename = "value", alias = "sig")]
    pub sig: String,
    /// Signing algorithm (e.g. "ed25519", "ecdsa-p256", "dilithium3").
    /// Serializes as `alg` to match the JSON schema; `algo` is accepted as a deser alias.
    #[serde(rename = "alg", alias = "algo")]
    pub algo: String,
    /// Optional timestamp when the signature was created.
    #[serde(skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Reference to a predecessor evidence envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// URN identifying the referenced envelope.
    pub id: String,
    /// Optional classifier (e.g. "SkillAssessment", "BuildProvenance").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Chain-of-custody linking this envelope to prior evidence envelopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    /// EvidenceRef identifiers of predecessor envelopes.
    pub previous: Vec<EvidenceRef>,
}

/// OpenEvidence Envelope v1
///
/// Signed evidence output substrate. Required fields: `id`, `subject`,
/// `statement`, `issuedAt`, `actor`, `signatures`. Serialises to camelCase
/// to match `schemas/evidence/v1/envelope.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEnvelope {
    /// URN of this evidence envelope (immutable identifier).
    pub id: String,
    /// Subject artifact being described.
    pub subject: Subject,
    /// Evidence statement with payload.
    pub statement: Statement,
    /// ISO 8601 timestamp of when this envelope was issued.
    pub issued_at: chrono::DateTime<chrono::Utc>,
    /// Actor that produced the envelope.
    pub actor: Actor,
    /// Governance Autonomy Level at which this action/evidence occurred (0–5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gal: Option<u8>,
    /// Atomic Security Controls active at the time of the action.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub asc: Vec<String>,
    /// Optional chain-of-custody linking to predecessor envelopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<Chain>,
    /// Cryptographic signatures. Empty on unsigned envelopes; signing is C0.2.
    pub signatures: Vec<Signature>,
}
