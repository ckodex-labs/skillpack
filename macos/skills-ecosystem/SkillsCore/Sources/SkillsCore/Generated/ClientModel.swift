// Auto-generated from schemas/skills-specs-next/client-model.schema.json
// Generated at: 2026-07-12T20:00:00Z  (update this timestamp when manually syncing)
// Do not edit manually. Run `make generate-types` in macos/skills-ecosystem to regenerate.

import Foundation

// MARK: - Primitive Type Aliases

public typealias Grade = String
public typealias Tier = String

// MARK: - Core Types

public struct SkillSummary: Codable {
    public let id: UUID
    public let name: String
    public let displayName: String?
    public let version: String
    public let grade: Grade
    public let tier: Tier
    public let updatedAt: Date
    public let isLocal: Bool?
    public let hasAssessment: Bool?
}

public struct SkillDetail: Codable {
    public let id: UUID
    public let name: String
    public let displayName: String?
    public let version: String
    public let grade: Grade
    public let tier: Tier
    public let manifest: SkillManifest
    public let assessment: AssessmentResult?
    public let lifecycle: LifecycleState?
    public let contentHash: String?
    public let path: String?
    public let updatedAt: Date
    public let installedAt: Date?
    public let registryUrl: String?
}

public struct SkillManifest: Codable {
    public let apiVersion: ApiVersion
    public let kind: String
    public let metadata: ManifestMetadata
    public let ontology: Ontology?
    public let runtime: Runtime?
    public let governance: Governance?
}

public enum ApiVersion: String, Codable {
    case v1 = "ckodex.org/skill/v1"
    case v1_1 = "ckodex.org/skill/v1.1"
}

public struct ManifestMetadata: Codable {
    public let name: String
    public let description: String
    public let synopsis: String?
    public let version: String
    public let tags: [String]?
}

public struct Ontology: Codable {
    public let subjects: [String]?
    public let produces: [String]?
    public let consumes: [String]?
    public let related: [String]?
    public let triggers: [String]?
    public let exclusiveOf: [String]?
}

public struct Runtime: Codable {
    public let implicitInvocation: Bool?
    public let minTier: Tier?
    public let maxTier: Tier?
    public let l4xAllowed: Bool?
}

public struct Governance: Codable {
    public let scope: GovernanceScope?
    public let requiresTipForCrossTenant: Bool?
}

public struct GovernanceScope: Codable {
    public let tenant: String?
    public let environment: [String]?
    public let workspace: String?
}

public struct AssessmentResult: Codable {
    public let id: UUID
    public let skillId: UUID
    public let totalScore: Double
    public let grade: Grade
    public let dimensions: [DimensionScore]
    public let issues: [Issue]
    public let bonusPoints: Int?
    public let stubCount: Int?
    public let assessedAt: Date
    public let profile: AssessmentProfile?
}

/// Rubric the skill was assessed under. cnsb = strict supply-chain bar
/// (CNSB manifest present); agentskills = SKILL.md content-quality bar.
public enum AssessmentProfile: String, Codable {
    case cnsb = "cnsb"
    case agentskills = "agentskills"
}

public struct DimensionScore: Codable {
    public let dimensionId: DimensionID
    public let dimensionName: String
    public let score: Double
    public let maxScore: Double
    public let weight: Double
    public let findings: [String]?
}

public enum DimensionID: String, Codable {
    case identity = "identity"
    case security = "security"
    case provenance = "provenance"
    case documentation = "documentation"
    case testing = "testing"
    case compatibility = "compatibility"
    case lifecycle = "lifecycle"
    case governance = "governance"
    case evalsHitl = "evals_hitl"
}

public struct Issue: Codable {
    public let severity: Severity
    public let dimensionId: String
    public let message: String
    public let path: String?
    public let line: Int?
}

public enum Severity: String, Codable {
    case error = "error"
    case warning = "warning"
    case info = "info"
}

public struct SyncStatus: Codable {
    public let root: String
    public let skillCount: Int
    public let agents: [AgentStatus]
    public let transport: Transport
    public let storageBackend: StorageBackend
    public let fdbStatus: FDBStatus?
    public let rustFSStatus: RustFSStatus?
    public let lastSyncAt: Date?
    public let isSyncing: Bool?
}

public enum Transport: String, Codable {
    case local = "local"
    case xpc = "XPC"
    case grpc = "gRPC"
    case rest = "REST"
}

public enum StorageBackend: String, Codable {
    case local = "local"
    case distributed = "distributed"
}

public enum FDBStatus: String, Codable {
    case online = "online"
    case offline = "offline"
    case notApplicable = "n/a"
}

public enum RustFSStatus: String, Codable {
    case online = "online"
    case offline = "offline"
    case notApplicable = "n/a"
}

public struct AgentStatus: Codable {
    public let name: String
    public let type: AgentType
    public let directory: String
    public let synced: Bool
    public let skillCount: Int?
}

public enum AgentType: String, Codable {
    case symlink = "symlink"
    case indexFile = "indexFile"
    case copy = "copy"
}

public struct RegistryEntry: Codable {
    public let name: String
    public let displayName: String?
    public let version: String
    public let description: String?
    public let registry: String
    public let ociRef: String
    public let grade: Grade?
    public let publishedAt: Date?
    public let downloadCount: Int?
    public let author: String?
}

public struct LifecycleEvent: Codable {
    public let eventId: UUID
    public let skillName: String
    public let skillId: UUID?
    public let decision: Decision
    public let newStatus: LifecycleStatus
    public let previousStatus: LifecycleStatus?
    public let decidedAt: Date
    public let supersededBy: String?
    public let actorUrn: String?
    public let scoreAtDecision: Double?
    public let privacyMode: PrivacyMode?
}

public enum Decision: String, Codable {
    case promoted = "promoted"
    case demoted = "demoted"
    case superseded = "superseded"
    case retired = "retired"
}

public enum LifecycleStatus: String, Codable {
    case draft = "draft"
    case active = "active"
    case deprecated = "deprecated"
    case retired = "retired"
}

public enum PrivacyMode: String, Codable {
    case debugLocal = "debug-local"
    case auditPrivate = "audit-private"
    case publicAnchor = "public-anchor"
    case regulatedExport = "regulated-export"
}

public struct UserPreference: Codable {
    public let schemaVersion: Int?
    public let autoAssess: Bool?
    public let minimumGrade: Grade?
    public let aiProvider: AIProvider?
    public let showWizardOnNewWorkspace: Bool?
    public let serverUrl: String?
    public let preferredTransport: PreferredTransport?
    public let theme: Theme?
    public let reducedMotion: Bool?
}

public enum AIProvider: String, Codable {
    case builtin = "builtin"
    case copilot = "copilot"
    case custom = "custom"
}

public enum PreferredTransport: String, Codable {
    case grpc = "gRPC"
    case rest = "REST"
    case auto = "auto"
}

public enum Theme: String, Codable {
    case system = "system"
    case light = "light"
    case dark = "dark"
}

public struct ClientError: Codable {
    public let code: String
    public let category: ErrorCategory
    public let severity: ErrorSeverity
    public let message: String
    public let details: [String: String]?
    public let clientHint: ClientHint?
    public let traceId: UUID?
}

public enum ErrorCategory: String, Codable {
    case validation = "validation"
    case security = "security"
    case network = "network"
    case storage = "storage"
    case auth = "auth"
    case `internal` = "internal"
}

public enum ErrorSeverity: String, Codable {
    case fatal = "fatal"
    case error = "error"
    case warning = "warning"
    case info = "info"
}

public struct ClientHint: Codable {
    public let cli: String?
    public let vscode: String?
    public let web: String?
    public let macos: String?
    public let cni: String?
    public let mcp: String?
}

public struct JobStatus: Codable {
    public let jobId: UUID
    public let status: JobStatusState
    public let progressPercent: Int?
    public let result: [String: String]?
    public let error: ClientError?
    public let createdAt: Date
    public let completedAt: Date?
}

public enum JobStatusState: String, Codable {
    case pending = "pending"
    case running = "running"
    case completed = "completed"
    case failed = "failed"
    case cancelled = "cancelled"
}

public struct LifecycleState: Codable {
    public let status: LifecycleStatus
    public let promotedAt: Date
    public let demotedAt: Date?
    public let supersededBy: String?
    public let actorUrn: String?
}

public struct ClientModel: Codable {
    public let schemaVersion: Int
}
