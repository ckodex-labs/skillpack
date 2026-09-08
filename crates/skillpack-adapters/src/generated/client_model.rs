// Auto-generated from schemas/skills-specs-next/client-model.schema.json
// Generated at: 2026-07-12T19:42:31Z
// Do not edit manually. Run `make generate-types` to regenerate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Apiversion {
    #[serde(rename = "ckodex.org/skill/v1")]
    CkodexOrgSkillV1,
    #[serde(rename = "ckodex.org/skill/v1.1")]
    CkodexOrgSkillV11,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Profile {
    #[serde(rename = "cnsb")]
    Cnsb,
    #[serde(rename = "agentskills")]
    Agentskills,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Dimensionid {
    #[serde(rename = "identity")]
    Identity,
    #[serde(rename = "security")]
    Security,
    #[serde(rename = "provenance")]
    Provenance,
    #[serde(rename = "documentation")]
    Documentation,
    #[serde(rename = "testing")]
    Testing,
    #[serde(rename = "compatibility")]
    Compatibility,
    #[serde(rename = "lifecycle")]
    Lifecycle,
    #[serde(rename = "governance")]
    Governance,
    #[serde(rename = "evals_hitl")]
    EvalsHitl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transport {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "XPC")]
    Xpc,
    #[serde(rename = "gRPC")]
    Grpc,
    #[serde(rename = "REST")]
    Rest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Storagebackend {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "distributed")]
    Distributed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Fdbstatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "n/a")]
    NA,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Rustfsstatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "n/a")]
    NA,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    #[serde(rename = "symlink")]
    Symlink,
    #[serde(rename = "indexFile")]
    Indexfile,
    #[serde(rename = "copy")]
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    #[serde(rename = "promoted")]
    Promoted,
    #[serde(rename = "demoted")]
    Demoted,
    #[serde(rename = "superseded")]
    Superseded,
    #[serde(rename = "retired")]
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Newstatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "retired")]
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Previousstatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "retired")]
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Privacymode {
    #[serde(rename = "debug-local")]
    DebugLocal,
    #[serde(rename = "audit-private")]
    AuditPrivate,
    #[serde(rename = "public-anchor")]
    PublicAnchor,
    #[serde(rename = "regulated-export")]
    RegulatedExport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Minimumgrade {
    #[serde(rename = "S+")]
    Splus,
    #[serde(rename = "S")]
    S,
    #[serde(rename = "A")]
    A,
    #[serde(rename = "B")]
    B,
    #[serde(rename = "C")]
    C,
    #[serde(rename = "D")]
    D,
    #[serde(rename = "F")]
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Aiprovider {
    #[serde(rename = "builtin")]
    Builtin,
    #[serde(rename = "copilot")]
    Copilot,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Preferredtransport {
    #[serde(rename = "gRPC")]
    Grpc,
    #[serde(rename = "REST")]
    Rest,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Category {
    #[serde(rename = "validation")]
    Validation,
    #[serde(rename = "security")]
    Security,
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "storage")]
    Storage,
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "internal")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Grade {
    #[serde(rename = "S+")]
    Splus,
    #[serde(rename = "S")]
    S,
    #[serde(rename = "A")]
    A,
    #[serde(rename = "B")]
    B,
    #[serde(rename = "C")]
    C,
    #[serde(rename = "D")]
    D,
    #[serde(rename = "F")]
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tier {
    #[serde(rename = "L0")]
    L0,
    #[serde(rename = "L1")]
    L1,
    #[serde(rename = "L2")]
    L2,
    #[serde(rename = "L3")]
    L3,
    #[serde(rename = "L4R")]
    L4r,
    #[serde(rename = "L4X")]
    L4x,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub grade: Grade,
    pub tier: Tier,
    pub updated_at: DateTime<Utc>,
    pub is_local: Option<bool>,
    pub has_assessment: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub grade: Grade,
    pub tier: Tier,
    pub manifest: SkillManifest,
    pub assessment: Option<AssessmentResult>,
    pub lifecycle: Option<LifecycleState>,
    pub content_hash: Option<String>,
    pub path: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub installed_at: Option<DateTime<Utc>>,
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub api_version: Apiversion,
    pub kind: String,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub ontology: Option<serde_json::Map<String, serde_json::Value>>,
    pub runtime: Option<serde_json::Map<String, serde_json::Value>>,
    pub governance: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub total_score: f64,
    pub grade: Grade,
    pub dimensions: Vec<DimensionScore>,
    pub issues: Vec<Issue>,
    pub bonus_points: Option<i64>,
    pub stub_count: Option<i64>,
    pub assessed_at: DateTime<Utc>,
    pub profile: Option<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub dimension_id: Dimensionid,
    pub dimension_name: String,
    pub score: f64,
    pub max_score: f64,
    pub weight: f64,
    pub findings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub dimension_id: String,
    pub message: String,
    pub path: Option<String>,
    pub line: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub root: String,
    pub skill_count: i64,
    pub agents: Vec<AgentStatus>,
    pub transport: Transport,
    pub storage_backend: Storagebackend,
    pub fdb_status: Option<Fdbstatus>,
    pub rust_f_s_status: Option<Rustfsstatus>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub is_syncing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub name: String,
    pub r#type: Type,
    pub directory: String,
    pub synced: bool,
    pub skill_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub registry: String,
    pub oci_ref: String,
    pub grade: Option<Grade>,
    pub published_at: Option<DateTime<Utc>>,
    pub download_count: Option<i64>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub event_id: Uuid,
    pub skill_name: String,
    pub skill_id: Option<Uuid>,
    pub decision: Decision,
    pub new_status: Newstatus,
    pub previous_status: Option<Previousstatus>,
    pub decided_at: DateTime<Utc>,
    pub superseded_by: Option<String>,
    pub actor_urn: Option<String>,
    pub score_at_decision: Option<f64>,
    pub privacy_mode: Option<Privacymode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub schema_version: Option<i64>,
    pub auto_assess: Option<bool>,
    pub minimum_grade: Option<Minimumgrade>,
    pub ai_provider: Option<Aiprovider>,
    pub show_wizard_on_new_workspace: Option<bool>,
    pub server_url: Option<String>,
    pub preferred_transport: Option<Preferredtransport>,
    pub theme: Option<Theme>,
    pub reduced_motion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientError {
    pub code: String,
    pub category: Category,
    pub severity: Severity,
    pub message: String,
    pub details: Option<serde_json::Map<String, serde_json::Value>>,
    pub client_hint: Option<ClientHint>,
    pub trace_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHint {
    pub cli: Option<String>,
    pub vscode: Option<String>,
    pub web: Option<String>,
    pub macos: Option<String>,
    pub cni: Option<String>,
    pub mcp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: Uuid,
    pub status: Status,
    pub progress_percent: Option<i64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<ClientError>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleState {
    pub status: Status,
    pub promoted_at: DateTime<Utc>,
    pub demoted_at: Option<DateTime<Utc>>,
    pub superseded_by: Option<String>,
    pub actor_urn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientModel {
    pub schema_version: i64,
}
