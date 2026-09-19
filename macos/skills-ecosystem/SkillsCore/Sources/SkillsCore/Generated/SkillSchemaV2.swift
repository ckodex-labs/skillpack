// Auto-generated from schemas/skill.v2.schema.json
// Unified CKODEX Skill Manifest v2 — consolidates all prior schema layers.
// Do not edit manually. Run `make generate-types` in macos/skills-ecosystem to regenerate.

import Foundation

// MARK: - Primitives

public typealias V2Urn         = String   // urn: prefix (loose)
public typealias V2CkodexUrn   = String   // urn:ckodex:<type>:<id>
public typealias V2Sha256      = String   // sha256:<hex64>
public typealias V2Timestamp   = String   // ISO 8601 / RFC 3339
public typealias V2GAL         = Int      // 0–5
public typealias V2ASC         = String   // e.g. "ASC_SLSA4"

// MARK: - Core enums

public enum V2DeploymentTier: String, Codable, CaseIterable {
    case experimental = "experimental"
    case beta         = "beta"
    case stable       = "stable"
    case deprecated   = "deprecated"
    case retired      = "retired"
}

public enum V2RuntimeTier: String, Codable, CaseIterable {
    case l0 = "L0"; case l1 = "L1"; case l2 = "L2"
    case l3 = "L3"; case l4r = "L4R"; case l4x = "L4X"
}

public enum V2ProofType: String, Codable, CaseIterable {
    case pca = "PCA"; case uca = "UCA"; case tip = "TIP"
    case zkp = "ZKP"; case dca = "DCA"; case tkp = "TKP"
    case wcag = "WCAG"; case clp = "CLP"; case ufp = "UFP"
}

public enum V2EnforcementPoint: String, Codable {
    case preExec   = "pre_exec"
    case runtime   = "runtime"
    case postExec  = "post_exec"
    case promotion = "promotion"
    case emergency = "emergency"
}

public enum V2PrivacyMode: String, Codable {
    case debugLocal      = "debug-local"
    case auditPrivate    = "audit-private"
    case publicAnchor    = "public-anchor"
    case regulatedExport = "regulated-export"
}

public enum V2Grade: String, Codable, CaseIterable, Comparable {
    case sPlus = "S+"; case s = "S"; case a = "A"
    case b = "B"; case c = "C"; case d = "D"; case f = "F"
    private var rank: Int {
        switch self {
        case .sPlus: return 0; case .s: return 1; case .a: return 2
        case .b: return 3; case .c: return 4; case .d: return 5; case .f: return 6
        }
    }
    public static func < (lhs: V2Grade, rhs: V2Grade) -> Bool { lhs.rank < rhs.rank }
}

public enum V2LifecycleStatus: String, Codable {
    case draft = "draft"; case active = "active"
    case deprecated = "deprecated"; case retired = "retired"
    case experimental = "experimental"; case superseded = "superseded"
}

public enum V2LifecycleTrigger: String, Codable {
    case scoreThreshold    = "score_threshold"
    case policyPreFilter   = "policy_pre_filter"
    case intentToInvoke    = "intent_to_invoke"
    case topicDrift        = "topic_drift"
    case idleTimeout       = "idle_timeout"
    case scoreDecay        = "score_decay"
    case budgetPressure    = "budget_pressure"
    case taskComplete      = "task_complete"
    case mutex             = "mutex"
    case frameworkBreak    = "framework_break"
    case capabilityDenied  = "capability_denied"
    case emergencyEvict    = "emergency_evict"
    case explicitUserSignal = "explicit_user_signal"
    case resourceLoad      = "resource_load"
    case executionComplete = "execution_complete"
    case policyRevoked     = "policy_revoked"
}

public enum V2SignatureType: String, Codable {
    case cosign = "cosign"; case pgp = "pgp"
    case x509 = "x509"; case dilithium3 = "dilithium3"
}

public enum V2ActorType: String, Codable {
    case human = "Human"; case agent = "Agent"
    case service = "Service"; case system = "System"
}

public enum V2ToolRisk: String, Codable {
    case low = "low"; case medium = "medium"; case high = "high"
}

public enum V2SandboxPolicy: String, Codable {
    case required = "required"; case recommended = "recommended"; case none = "none"
}

public enum V2UserConfirmation: String, Codable {
    case always = "always"; case dangerousOnly = "dangerous-only"; case never = "never"
}

// MARK: - Shared value types

public struct V2SignalWeights: Codable, Equatable {
    public var lex: Double?; public var sem: Double?; public var ont: Double?
    public var co: Double?;  public var rec: Double?; public var usr: Double?
    public init(lex: Double? = nil, sem: Double? = nil, ont: Double? = nil,
                co: Double? = nil, rec: Double? = nil, usr: Double? = nil) {
        self.lex = lex; self.sem = sem; self.ont = ont
        self.co = co; self.rec = rec; self.usr = usr
    }
}

public struct V2SignatureRef: Codable {
    public let type: V2SignatureType
    public let ref: String
    public init(type: V2SignatureType, ref: String) { self.type = type; self.ref = ref }
}

public struct V2Actor: Codable {
    public let urn: V2CkodexUrn
    public let type: V2ActorType
    public let displayName: String?
    public let tenantUrn: V2Urn?
    public let labels: [String: String]?
    public init(urn: V2CkodexUrn, type: V2ActorType, displayName: String? = nil,
                tenantUrn: V2Urn? = nil, labels: [String: String]? = nil) {
        self.urn = urn; self.type = type; self.displayName = displayName
        self.tenantUrn = tenantUrn; self.labels = labels
    }
}

// MARK: - §metadata

public struct V2Metadata: Codable {
    public var name: String
    public var displayName: String?
    public var description: String
    public var synopsis: String?
    public var synopsisHash: V2Sha256?
    public var version: String?
    public var deploymentTier: V2DeploymentTier?
    public var license: String?
    public var authors: [String]
    public var keywords: [String]
    public var labels: [String: String]
    public var annotations: [String: String]
    public var registryUrl: String?
    public var urn: V2CkodexUrn?

    public init(name: String, description: String, version: String? = nil,
                displayName: String? = nil, synopsis: String? = nil,
                synopsisHash: V2Sha256? = nil, deploymentTier: V2DeploymentTier? = nil,
                license: String? = nil, authors: [String] = [], keywords: [String] = [],
                labels: [String: String] = [:], annotations: [String: String] = [:],
                registryUrl: String? = nil, urn: V2CkodexUrn? = nil) {
        self.name = name; self.description = description; self.version = version
        self.displayName = displayName; self.synopsis = synopsis; self.synopsisHash = synopsisHash
        self.deploymentTier = deploymentTier; self.license = license
        self.authors = authors; self.keywords = keywords; self.labels = labels
        self.annotations = annotations; self.registryUrl = registryUrl; self.urn = urn
    }
}

// MARK: - §entrypoints

public struct V2Entrypoints: Codable {
    public let skillMd: String          // const: "SKILL.md"
    public let manifest: String?        // "skill.json"
    public let readme: String?
    public init(skillMd: String = "SKILL.md", manifest: String? = nil, readme: String? = nil) {
        self.skillMd = skillMd; self.manifest = manifest; self.readme = readme
    }
}

// MARK: - §capabilities

public struct V2CapabilityTool: Codable {
    public var name: String
    public var scope: String?
    public var risk: V2ToolRisk?
    public init(name: String, scope: String? = nil, risk: V2ToolRisk? = nil) {
        self.name = name; self.scope = scope; self.risk = risk
    }
}

public struct V2Capabilities: Codable {
    public var requiresNetwork: Bool
    public var requiresFilesystem: Bool
    public var requiresGpu: Bool
    public var os: [String]
    public var arch: [String]
    public var tools: [V2CapabilityTool]
    public init(requiresNetwork: Bool = false, requiresFilesystem: Bool = true,
                requiresGpu: Bool = false, os: [String] = [], arch: [String] = [],
                tools: [V2CapabilityTool] = []) {
        self.requiresNetwork = requiresNetwork; self.requiresFilesystem = requiresFilesystem
        self.requiresGpu = requiresGpu; self.os = os; self.arch = arch; self.tools = tools
    }
}

public struct V2AIPackCapabilities: Codable {
    public var read: Bool; public var write: Bool; public var network: Bool
    public var filesystem: Bool; public var execution: Bool
    public init(read: Bool = true, write: Bool = false, network: Bool = false,
                filesystem: Bool = true, execution: Bool = false) {
        self.read = read; self.write = write; self.network = network
        self.filesystem = filesystem; self.execution = execution
    }
}

// MARK: - §policy

public struct V2DataHandling: Codable {
    public var pii: Bool
    public var secrets: Bool
    public var allowedOutputs: [String]
    public init(pii: Bool = false, secrets: Bool = true, allowedOutputs: [String] = []) {
        self.pii = pii; self.secrets = secrets; self.allowedOutputs = allowedOutputs
    }
}

public struct V2Policy: Codable {
    public var sandbox: V2SandboxPolicy
    public var userConfirmation: V2UserConfirmation
    public var dataHandling: V2DataHandling?
    public init(sandbox: V2SandboxPolicy = .recommended,
                userConfirmation: V2UserConfirmation = .dangerousOnly,
                dataHandling: V2DataHandling? = nil) {
        self.sandbox = sandbox; self.userConfirmation = userConfirmation
        self.dataHandling = dataHandling
    }
}

// MARK: - §governance

public struct V2Governance: Codable {
    public var galMinimum: V2GAL?
    public var galMax: V2GAL?
    public var proofRequired: Bool?
    public var proofTypes: [V2ProofType]
    public var enforcementPoints: [V2EnforcementPoint]
    public var emergencyProtocols: [String]   // "EP-NNN"
    public var asc: [V2ASC]
    public init(galMinimum: V2GAL? = nil, galMax: V2GAL? = nil, proofRequired: Bool? = nil,
                proofTypes: [V2ProofType] = [], enforcementPoints: [V2EnforcementPoint] = [],
                emergencyProtocols: [String] = [], asc: [V2ASC] = []) {
        self.galMinimum = galMinimum; self.galMax = galMax; self.proofRequired = proofRequired
        self.proofTypes = proofTypes; self.enforcementPoints = enforcementPoints
        self.emergencyProtocols = emergencyProtocols; self.asc = asc
    }
}

// MARK: - §ontology

public struct V2Ontology: Codable {
    public var context: String          // const: "https://ckodex.org/skill-context/v1"
    public var subjects: [String]
    public var produces: [String]
    public var consumes: [String]
    public var related: [String]
    public var exclusiveOf: [String]
    public var triggers: [String]
    public var galMinimum: V2GAL?
    public var proofTypes: [V2ProofType]

    enum CodingKeys: String, CodingKey {
        case context = "@context"
        case subjects, produces, consumes, related, exclusiveOf, triggers, galMinimum, proofTypes
    }

    public init(context: String = "https://ckodex.org/skill-context/v1",
                subjects: [String] = [], produces: [String] = [], consumes: [String] = [],
                related: [String] = [], exclusiveOf: [String] = [], triggers: [String] = [],
                galMinimum: V2GAL? = nil, proofTypes: [V2ProofType] = []) {
        self.context = context; self.subjects = subjects; self.produces = produces
        self.consumes = consumes; self.related = related; self.exclusiveOf = exclusiveOf
        self.triggers = triggers; self.galMinimum = galMinimum; self.proofTypes = proofTypes
    }
}

// MARK: - §runtime

public struct V2Runtime: Codable {
    public var implicitInvocation: Bool
    public var minTier: V2RuntimeTier?
    public var maxTier: V2RuntimeTier?
    public var l4xAllowed: Bool
    public init(implicitInvocation: Bool = true, minTier: V2RuntimeTier? = nil,
                maxTier: V2RuntimeTier? = nil, l4xAllowed: Bool = false) {
        self.implicitInvocation = implicitInvocation; self.minTier = minTier
        self.maxTier = maxTier; self.l4xAllowed = l4xAllowed
    }
}

// MARK: - §supplyChain

public struct V2SupplyChainSource: Codable {
    public var repo: String?; public var path: String?; public var revision: String?
    public init(repo: String? = nil, path: String? = nil, revision: String? = nil) {
        self.repo = repo; self.path = path; self.revision = revision
    }
}

public struct V2SupplyChainSBOM: Codable {
    public var spdx: String?; public var cyclonedx: String?
    public init(spdx: String? = nil, cyclonedx: String? = nil) {
        self.spdx = spdx; self.cyclonedx = cyclonedx
    }
}

public struct V2SupplyChainProvenance: Codable {
    public var inToto: String?; public var slsaLevel: Int?
    public init(inToto: String? = nil, slsaLevel: Int? = nil) {
        self.inToto = inToto; self.slsaLevel = slsaLevel
    }
}

public struct V2SupplyChain: Codable {
    public var source: V2SupplyChainSource?
    public var digest: V2Sha256?
    public var signatures: [V2SignatureRef]
    public var sbom: V2SupplyChainSBOM?
    public var provenance: V2SupplyChainProvenance?
    public init(source: V2SupplyChainSource? = nil, digest: V2Sha256? = nil,
                signatures: [V2SignatureRef] = [], sbom: V2SupplyChainSBOM? = nil,
                provenance: V2SupplyChainProvenance? = nil) {
        self.source = source; self.digest = digest; self.signatures = signatures
        self.sbom = sbom; self.provenance = provenance
    }
}

// MARK: - §inputs / §outputs / §examples

public struct V2SkillParam: Codable {
    public var name: String; public var type: String?
    public var description: String?; public var required: Bool?; public var schemaRef: String?
    public init(name: String, type: String? = nil, description: String? = nil,
                required: Bool? = nil, schemaRef: String? = nil) {
        self.name = name; self.type = type; self.description = description
        self.required = required; self.schemaRef = schemaRef
    }
}

public struct V2SkillExample: Codable {
    public var name: String; public var description: String?
    public init(name: String, description: String? = nil) {
        self.name = name; self.description = description
    }
}

// MARK: - §lifecycleHooks

public struct V2LifecycleHook: Codable {
    public var command: String?; public var script: String?; public var timeout: String?
    public var retries: Int?; public var env: [String: String]?
    public init(command: String? = nil, script: String? = nil, timeout: String? = nil,
                retries: Int? = nil, env: [String: String]? = nil) {
        self.command = command; self.script = script; self.timeout = timeout
        self.retries = retries; self.env = env
    }
}

public struct V2LifecycleHooks: Codable {
    public var pack: V2LifecycleHook?; public var unpack: V2LifecycleHook?
    public var install: V2LifecycleHook?; public var upgrade: V2LifecycleHook?
    public var uninstall: V2LifecycleHook?; public var verify: V2LifecycleHook?
    public var preInstall: V2LifecycleHook?; public var postInstall: V2LifecycleHook?
    public var preInvoke: String?; public var postInvoke: String?; public var onDeprecate: String?
}

// MARK: - §lifecycleState

public struct V2LifecycleState: Codable {
    public var status: V2LifecycleStatus?
    public var promotedAt: V2Timestamp?; public var demotedAt: V2Timestamp?
    public var deprecatedAt: V2Timestamp?; public var retiredAt: V2Timestamp?
    public var supersededBy: String?; public var actorUrn: V2CkodexUrn?
    public init(status: V2LifecycleStatus? = nil, promotedAt: V2Timestamp? = nil,
                demotedAt: V2Timestamp? = nil, deprecatedAt: V2Timestamp? = nil,
                retiredAt: V2Timestamp? = nil, supersededBy: String? = nil,
                actorUrn: V2CkodexUrn? = nil) {
        self.status = status; self.promotedAt = promotedAt; self.demotedAt = demotedAt
        self.deprecatedAt = deprecatedAt; self.retiredAt = retiredAt
        self.supersededBy = supersededBy; self.actorUrn = actorUrn
    }
}

// MARK: - §assessment

public struct V2DimensionScore: Codable {
    public var dimensionId: String; public var dimensionName: String
    public var score: Double; public var maxScore: Double; public var weight: Double
    public var findings: [String]
    public init(dimensionId: String, dimensionName: String, score: Double,
                maxScore: Double, weight: Double, findings: [String] = []) {
        self.dimensionId = dimensionId; self.dimensionName = dimensionName
        self.score = score; self.maxScore = maxScore; self.weight = weight; self.findings = findings
    }
}

public struct V2AssessmentIssue: Codable {
    public var severity: String; public var dimensionId: String; public var message: String
    public var path: String?; public var line: Int?
    public init(severity: String, dimensionId: String, message: String,
                path: String? = nil, line: Int? = nil) {
        self.severity = severity; self.dimensionId = dimensionId; self.message = message
        self.path = path; self.line = line
    }
}

public struct V2AssessmentResult: Codable {
    public var totalScore: Double; public var grade: V2Grade
    public var dimensions: [V2DimensionScore]; public var issues: [V2AssessmentIssue]
    public var bonusPoints: Int; public var stubCount: Int?; public var assessedAt: V2Timestamp
    public init(totalScore: Double, grade: V2Grade, dimensions: [V2DimensionScore] = [],
                issues: [V2AssessmentIssue] = [], bonusPoints: Int = 0,
                stubCount: Int? = nil, assessedAt: V2Timestamp = "") {
        self.totalScore = totalScore; self.grade = grade; self.dimensions = dimensions
        self.issues = issues; self.bonusPoints = bonusPoints; self.stubCount = stubCount
        self.assessedAt = assessedAt
    }
}

// MARK: - §runtimeCapabilities

public struct V2RuntimeCapabilities: Codable {
    public var supportsSynopsisTier: Bool?; public var supportsOntologyScoring: Bool?
    public var supportsHysteresis: Bool?; public var supportsEviction: Bool?
    public var supportsLifecyclePca: Bool?; public var supportsL4xSandbox: Bool?
    public var maxSkillPoolTokens: Int?; public var defaultWeights: V2SignalWeights?
}

// MARK: - §pca  (Proof-Carrying Action)

public struct V2PCADecision: Codable {
    public var skillId: V2Sha256; public var skillName: String?
    public var fromTier: V2RuntimeTier; public var toTier: V2RuntimeTier
    public var trigger: V2LifecycleTrigger; public var score: Double?
    public var drift: Double?; public var theta: Double?; public var idleTurns: Int?
    public var capability: String?; public var weights: V2SignalWeights?
    public init(skillId: V2Sha256, fromTier: V2RuntimeTier, toTier: V2RuntimeTier,
                trigger: V2LifecycleTrigger, skillName: String? = nil, score: Double? = nil,
                drift: Double? = nil, theta: Double? = nil, idleTurns: Int? = nil,
                capability: String? = nil, weights: V2SignalWeights? = nil) {
        self.skillId = skillId; self.skillName = skillName; self.fromTier = fromTier
        self.toTier = toTier; self.trigger = trigger; self.score = score; self.drift = drift
        self.theta = theta; self.idleTurns = idleTurns; self.capability = capability
        self.weights = weights
    }
}

public struct V2PCA: Codable {
    public var predicate: String    // const: "ckodex/skill-lifecycle@v1"
    public var subject: V2CkodexUrn
    public var timestamp: V2Timestamp
    public var privacyMode: V2PrivacyMode
    public var policyVersion: String
    public var decisions: [V2PCADecision]
}

// MARK: - §stx  (Skills Transparency Exchange reference)

public struct V2STXRef: Codable {
    public var skillId: V2Sha256?
    public var harnessId: V2CkodexUrn?; public var sessionId: V2CkodexUrn?
    public var currentTier: V2RuntimeTier?
    public var score: Double?; public var tokenCost: Int?; public var locked: Bool?
}

// MARK: - §lock

public struct V2LockedDependency: Codable {
    public var version: String; public var resolved: String; public var integrity: String
    public var registry: String?; public var signatures: [V2SignatureRef]?
    public var dependencies: [V2Urn]?
}

public struct V2Lock: Codable {
    public var lockVersion: Int   // const: 1
    public var integrity: String?; public var resolved: String?
    public var dependencies: [String: V2LockedDependency]?
}

// MARK: - AIPACK v0.1.1 identity types

public enum V2ArtifactFamily: String, Codable, CaseIterable {
    case model      = "model"
    case capability = "capability"
    case control    = "control"
    case knowledge  = "knowledge"
    case assurance  = "assurance"
    case composite  = "composite"
}

public enum V2AipackArtifactCode: String, Codable, CaseIterable {
    case a1 = "A1"; case a2 = "A2"; case a3 = "A3"; case a4 = "A4"
    case a5 = "A5"; case a6 = "A6"; case a7 = "A7"; case a8 = "A8"
    case a9 = "A9"; case a10 = "A10"; case a11 = "A11"; case a12 = "A12"
    case a13 = "A13"; case a14 = "A14"; case c1 = "C1"
}

public enum V2AipackSchemaCompat: String, Codable {
    case aipackV010  = "aipack.v0.1.0"
    case aipackV011  = "aipack.v0.1.1"
    case ckodexSkillV2 = "ckodex.skill.v2"
}

public struct V2PluginProjectionEntry: Codable {
    public var enabled: Bool?
    public var pluginJson: String?
    public var skillsPath: String?
    public var pluginRootToken: String?
    public init(enabled: Bool? = false, pluginJson: String? = nil,
                skillsPath: String? = nil, pluginRootToken: String? = nil) {
        self.enabled = enabled; self.pluginJson = pluginJson
        self.skillsPath = skillsPath; self.pluginRootToken = pluginRootToken
    }
}

public struct V2AipackSkillProjection: Codable {
    public var vscodeAgentPlugin: V2PluginProjectionEntry?
    public var copilotCliPlugin: V2PluginProjectionEntry?
    public var claudePlugin: V2PluginProjectionEntry?
    public var openPlugin: V2PluginProjectionEntry?
}

/// Canonical AIPACK identity block. `code` == "A4", `kind` == "Skill", `family` == "capability".
public struct V2AipackSkillIdentity: Codable {
    public let code: String              // const: "A4"
    public let kind: String              // const: "Skill"
    public let family: String            // const: "capability"
    public let manifestMediaType: String // const: "application/vnd.ai.skill.v1+json"
    public let bundleMediaType: String   // const: "application/vnd.ai.skill.bundle.v1.tar+zstd"
    public var subjectDigest: V2Sha256?
    public var ociRef: String?
    public var schemaCompatibility: [V2AipackSchemaCompat]?
    public var projections: V2AipackSkillProjection?

    public init(
        subjectDigest: V2Sha256? = nil,
        ociRef: String? = nil,
        schemaCompatibility: [V2AipackSchemaCompat]? = nil,
        projections: V2AipackSkillProjection? = nil
    ) {
        self.code = "A4"; self.kind = "Skill"; self.family = "capability"
        self.manifestMediaType = "application/vnd.ai.skill.v1+json"
        self.bundleMediaType   = "application/vnd.ai.skill.bundle.v1.tar+zstd"
        self.subjectDigest = subjectDigest; self.ociRef = ociRef
        self.schemaCompatibility = schemaCompatibility; self.projections = projections
    }
}

public enum V2PackageLifecycleState: String, Codable, CaseIterable {
    case authored    = "authored"
    case packed      = "packed"
    case signed      = "signed"
    case published   = "published"
    case fetched     = "fetched"
    case unpacked    = "unpacked"
    case verified    = "verified"
    case planned     = "planned"
    case approved    = "approved"
    case installed   = "installed"
    case enabled     = "enabled"
    case active      = "active"
    case disabled    = "disabled"
    case upgraded    = "upgraded"
    case rolledBack  = "rolled_back"
    case quarantined = "quarantined"
    case deprecated  = "deprecated"
    case retired     = "retired"
    case uninstalled = "uninstalled"
    case removed     = "removed"
}

public enum V2PackagePolicyDecision: String, Codable {
    case allow            = "allow"
    case ask              = "ask"
    case deny             = "deny"
    case quarantine       = "quarantine"
    case allowWithSandbox = "allow-with-sandbox"
    case allowReadonly    = "allow-readonly"
}

public struct V2PackageLifecycleRecord: Codable {
    public var state: V2PackageLifecycleState?
    public var previousState: V2PackageLifecycleState?
    public var changedAt: V2Timestamp?
    public var changedBy: V2Actor?
    public var reason: String?
    public var receiptDigest: V2Sha256?
    public var policyDecision: V2PackagePolicyDecision?
    public init(state: V2PackageLifecycleState? = nil, previousState: V2PackageLifecycleState? = nil,
                changedAt: V2Timestamp? = nil, changedBy: V2Actor? = nil,
                reason: String? = nil, receiptDigest: V2Sha256? = nil,
                policyDecision: V2PackagePolicyDecision? = nil) {
        self.state = state; self.previousState = previousState; self.changedAt = changedAt
        self.changedBy = changedBy; self.reason = reason; self.receiptDigest = receiptDigest
        self.policyDecision = policyDecision
    }
}

// MARK: - §agentCompatibility  (asm / skills-cli back-compat)

public struct V2AgentCompatibility: Codable {
    public var compatibility: String?
    public var allowedTools: [String]
    public var allowedToolsRaw: String?
    public var effort: String?
    public var marketplace: String?
    public init(compatibility: String? = nil, allowedTools: [String] = [],
                allowedToolsRaw: String? = nil, effort: String? = nil,
                marketplace: String? = nil) {
        self.compatibility = compatibility; self.allowedTools = allowedTools
        self.allowedToolsRaw = allowedToolsRaw; self.effort = effort
        self.marketplace = marketplace
    }
}

// MARK: - Root: SkillManifestV2

/// The unified v2 skill manifest. `schemaVersion` is always 2, `apiVersion` is always
/// "ckodex.org/skill/v2", `kind` is always "Skill".
public struct SkillManifestV2: Codable {
    public let schemaVersion: Int               // const: 2
    public let apiVersion: String               // const: "ckodex.org/skill/v2"
    public let kind: String                     // const: "Skill"

    public var metadata: V2Metadata
    public var entrypoints: V2Entrypoints

    // Capabilities
    public var capabilities: V2Capabilities?
    public var aipackCapabilities: V2AIPackCapabilities?

    // Content
    public var resources: V2Resources?
    public var inputs: [V2SkillParam]?
    public var outputs: [V2SkillParam]?
    public var examples: [V2SkillExample]?

    // Governance / policy
    public var policy: V2Policy?
    public var governance: V2Governance?
    public var ontology: V2Ontology?
    public var runtime: V2Runtime?

    // Supply chain
    public var supplyChain: V2SupplyChain?

    // Lifecycle
    public var lifecycleHooks: V2LifecycleHooks?
    public var lifecycleState: V2LifecycleState?

    // Assessment
    public var assessment: V2AssessmentResult?

    // Runtime / transparency
    public var runtimeCapabilities: V2RuntimeCapabilities?
    public var pca: V2PCA?
    public var stx: V2STXRef?
    public var lock: V2Lock?

    // AIPACK identity (optional, non-breaking)
    public var family: String?               // const "capability" when present
    public var aipack: V2AipackSkillIdentity?
    public var packageLifecycle: V2PackageLifecycleRecord?

    // Legacy asm back-compat
    public var agentCompatibility: V2AgentCompatibility?

    public init(
        metadata: V2Metadata,
        entrypoints: V2Entrypoints = V2Entrypoints(),
        capabilities: V2Capabilities? = nil,
        aipackCapabilities: V2AIPackCapabilities? = nil,
        resources: V2Resources? = nil,
        inputs: [V2SkillParam]? = nil,
        outputs: [V2SkillParam]? = nil,
        examples: [V2SkillExample]? = nil,
        policy: V2Policy? = nil,
        governance: V2Governance? = nil,
        ontology: V2Ontology? = nil,
        runtime: V2Runtime? = nil,
        supplyChain: V2SupplyChain? = nil,
        lifecycleHooks: V2LifecycleHooks? = nil,
        lifecycleState: V2LifecycleState? = nil,
        assessment: V2AssessmentResult? = nil,
        runtimeCapabilities: V2RuntimeCapabilities? = nil,
        pca: V2PCA? = nil,
        stx: V2STXRef? = nil,
        lock: V2Lock? = nil,
        agentCompatibility: V2AgentCompatibility? = nil,
        family: String? = nil,
        aipack: V2AipackSkillIdentity? = nil,
        packageLifecycle: V2PackageLifecycleRecord? = nil
    ) {
        self.schemaVersion = 2
        self.apiVersion = "ckodex.org/skill/v2"
        self.kind = "Skill"
        self.metadata = metadata
        self.entrypoints = entrypoints
        self.capabilities = capabilities
        self.aipackCapabilities = aipackCapabilities
        self.resources = resources
        self.inputs = inputs; self.outputs = outputs; self.examples = examples
        self.policy = policy; self.governance = governance
        self.ontology = ontology; self.runtime = runtime
        self.supplyChain = supplyChain
        self.lifecycleHooks = lifecycleHooks; self.lifecycleState = lifecycleState
        self.assessment = assessment
        self.runtimeCapabilities = runtimeCapabilities
        self.pca = pca; self.stx = stx; self.lock = lock
        self.agentCompatibility = agentCompatibility
        self.family = family
        self.aipack = aipack
        self.packageLifecycle = packageLifecycle
    }
}

// MARK: - §resources (extracted here to avoid forward-ref issues)

public struct V2Resources: Codable {
    public var scripts: [String]
    public var references: [String]
    public var assets: [String]
    public init(scripts: [String] = [], references: [String] = [], assets: [String] = []) {
        self.scripts = scripts; self.references = references; self.assets = assets
    }
}
