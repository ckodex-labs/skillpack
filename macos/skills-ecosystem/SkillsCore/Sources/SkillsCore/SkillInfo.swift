import Foundation

// MARK: - Enums (proper typed replacements for ClientModel's Grade/Tier typealiases)

/// Assessment grade — typed enum replacing `typealias Grade = String` in ClientModel.swift
public enum SkillGrade: String, Codable, CaseIterable, Comparable {
    case sPlus = "S+"
    case s     = "S"
    case a     = "A"
    case b     = "B"
    case c     = "C"
    case d     = "D"
    case f     = "F"

    private var order: Int {
        switch self {
        case .sPlus: return 0; case .s: return 1; case .a: return 2
        case .b:     return 3; case .c: return 4; case .d: return 5; case .f: return 6
        }
    }
    public static func < (lhs: SkillGrade, rhs: SkillGrade) -> Bool { lhs.order < rhs.order }
}

/// Runtime FSM tier — use STXTier from Generated/SkillSchema.swift (canonical source).
public typealias SkillFSMTier = STXTier

// Note: AssessmentResult, DimensionScore, DimensionID, Issue, Severity, LifecycleState,
// LifecycleStatus are defined in Generated/ClientModel.swift — use those directly.

// MARK: - SkillWarning

public struct SkillWarning: Codable, Equatable {
    public let category: String
    public let message: String

    public init(category: String, message: String) {
        self.category = category
        self.message = message
    }
}

// MARK: - SkillInfo (runtime scan record)

public struct SkillInfo: Codable {
    // --- Identity (v1 metadata) ---
    public var name: String
    public var displayName: String?
    public var version: String
    public var description: String
    public var synopsis: String?
    public var creator: String
    public var authors: [String]
    public var license: String
    public var keywords: [String]
    public var compatibility: String
    public var allowedTools: [String]

    // --- Filesystem / provider ---
    public var dirName: String
    public var path: String
    public var originalPath: String
    public var location: String
    public var scope: String
    public var provider: String
    public var providerLabel: String
    public var isSymlink: Bool
    public var symlinkTarget: String?
    public var realPath: String
    public var contentHash: String?

    // --- Scan metadata ---
    public var tokenCount: Int?
    public var fileCount: Int?
    public var warnings: [SkillWarning]
    public var disabled: Bool

    // --- Schema blocks (v1/v1.1) ---
    public var tier: String?                     // deployment tier: experimental|beta|stable|deprecated|retired
    public var fsmTier: SkillFSMTier?            // runtime FSM tier: L0-L4X
    public var governance: SkillGovernance?
    public var policy: SkillPolicy?
    public var supplyChain: SkillSupplyChain?
    public var ontology: SkillOntology?
    public var runtime: SkillRuntime?
    public var capabilities: SkillCapabilities?

    // --- Assessment / lifecycle (ClientModel.swift types) ---
    public var grade: SkillGrade?
    public var assessment: AssessmentResult?    // ClientModel.AssessmentResult
    public var lifecycle: LifecycleState?       // ClientModel.LifecycleState
    public var registryUrl: String?

    // --- Legacy asm-compat ---
    public var marketplace: String?
    public var effort: String?

    public init(
        name: String,
        version: String,
        description: String,
        creator: String,
        license: String,
        compatibility: String,
        allowedTools: [String],
        dirName: String,
        path: String,
        originalPath: String,
        location: String,
        scope: String,
        provider: String,
        providerLabel: String,
        isSymlink: Bool,
        symlinkTarget: String?,
        realPath: String,
        tokenCount: Int?,
        warnings: [SkillWarning],
        disabled: Bool = false,
        displayName: String? = nil,
        synopsis: String? = nil,
        authors: [String] = [],
        keywords: [String] = [],
        contentHash: String? = nil,
        fileCount: Int? = nil,
        tier: String? = nil,
        fsmTier: SkillFSMTier? = nil,
        governance: SkillGovernance? = nil,
        policy: SkillPolicy? = nil,
        supplyChain: SkillSupplyChain? = nil,
        ontology: SkillOntology? = nil,
        runtime: SkillRuntime? = nil,
        capabilities: SkillCapabilities? = nil,
        grade: SkillGrade? = nil,
        assessment: AssessmentResult? = nil,
        lifecycle: LifecycleState? = nil,
        registryUrl: String? = nil,
        marketplace: String? = nil,
        effort: String? = nil
    ) {
        self.name = name; self.version = version; self.description = description
        self.creator = creator; self.license = license; self.compatibility = compatibility
        self.allowedTools = allowedTools; self.dirName = dirName; self.path = path
        self.originalPath = originalPath; self.location = location; self.scope = scope
        self.provider = provider; self.providerLabel = providerLabel
        self.isSymlink = isSymlink; self.symlinkTarget = symlinkTarget; self.realPath = realPath
        self.tokenCount = tokenCount; self.warnings = warnings; self.disabled = disabled
        self.displayName = displayName; self.synopsis = synopsis
        self.authors = authors; self.keywords = keywords; self.contentHash = contentHash
        self.fileCount = fileCount; self.tier = tier; self.fsmTier = fsmTier
        self.governance = governance; self.policy = policy; self.supplyChain = supplyChain
        self.ontology = ontology; self.runtime = runtime; self.capabilities = capabilities
        self.grade = grade; self.assessment = assessment; self.lifecycle = lifecycle
        self.registryUrl = registryUrl; self.marketplace = marketplace; self.effort = effort
    }
}

// MARK: - Lock file (skill-lock.schema.json)

public struct SkillLockEntry: Codable {
    public var source: String
    public var sourceType: String
    public var commitHash: String?
    public var provider: String
    public var installedAt: String
    public var integrity: String?      // SRI hash (sha256-<base64>)
    public var resolved: String?       // Full resolved ref (OCI digest etc.)

    public init(source: String, sourceType: String, commitHash: String? = nil,
                provider: String, installedAt: String,
                integrity: String? = nil, resolved: String? = nil) {
        self.source = source; self.sourceType = sourceType; self.commitHash = commitHash
        self.provider = provider; self.installedAt = installedAt
        self.integrity = integrity; self.resolved = resolved
    }
}

public struct SkillLockFileMeta: Codable {
    public var generated: String?
    public var generator: String?
    public var checksum: String?
    public init(generated: String? = nil, generator: String? = nil, checksum: String? = nil) {
        self.generated = generated; self.generator = generator; self.checksum = checksum
    }
}

public struct SkillLockFile: Codable {
    public var lockVersion: Int
    public var metadata: SkillLockFileMeta?
    public var skills: [String: SkillLockEntry]

    public init(lockVersion: Int = 1, metadata: SkillLockFileMeta? = nil,
                skills: [String: SkillLockEntry] = [:]) {
        self.lockVersion = lockVersion; self.metadata = metadata; self.skills = skills
    }
}
