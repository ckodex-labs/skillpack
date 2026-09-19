// Auto-generated from the full CKODEX schema ecosystem.
// Sources:
//   schemas/common/types.schema.json
//   schemas/cnsb/v1/cnsb.schema.json
//   schemas/cnsb/v1/skill-lock.schema.json
//   schemas/skills-specs-next/ckodex-skill-lifecycle-rfc001/schemas/pca-lifecycle.v1.schema.json
//   schemas/skills-specs-next/ckodex-skill-lifecycle-rfc001/schemas/skill-runtime-capabilities.v1.schema.json
//   schemas/skills-specs-next/ckodex-stx-spec/schemas/stx-domain.v1.schema.json
//   schemas/skills-specs-next/ckodex-skill-spec-v1.1/schemas/skillsbundle.v1.schema.json
// Do not edit manually. Run `make generate-types` in macos/skills-ecosystem to regenerate.

import Foundation

// MARK: - common/types.schema.json

/// Loose URN — any urn: prefix.
public typealias Urn = String

/// Strict CKODEX URN: urn:ckodex:<type>:<id>
public typealias CkodexUrn = String

/// Governance Autonomy Level 0–5.
public typealias GAL = Int

/// Atomic Security Control symbolic identifier (e.g. "ASC_SLSA4").
public typealias ASC = String

public enum ActorType: String, Codable {
    case human   = "Human"
    case agent   = "Agent"
    case service = "Service"
    case system  = "System"
}

public struct CkodexActor: Codable {
    public let urn: CkodexUrn
    public let type: ActorType
    public let displayName: String?
    public let tenantUrn: Urn?
    public let labels: [String: String]?

    public init(urn: CkodexUrn, type: ActorType, displayName: String? = nil,
                tenantUrn: Urn? = nil, labels: [String: String]? = nil) {
        self.urn = urn; self.type = type; self.displayName = displayName
        self.tenantUrn = tenantUrn; self.labels = labels
    }
}

public struct CkodexSignature: Codable {
    public let alg: String
    public let value: String       // base64-encoded raw bytes
    public let keyId: Urn
    public let createdAt: String?

    public init(alg: String, value: String, keyId: Urn, createdAt: String? = nil) {
        self.alg = alg; self.value = value; self.keyId = keyId; self.createdAt = createdAt
    }
}

public struct ResourceMetadata: Codable {
    public let name: String
    public let urn: CkodexUrn
    public let version: String
    public let description: String?
    public let labels: [String: String]?
    public let annotations: [String: String]?

    public init(name: String, urn: CkodexUrn, version: String, description: String? = nil,
                labels: [String: String]? = nil, annotations: [String: String]? = nil) {
        self.name = name; self.urn = urn; self.version = version
        self.description = description; self.labels = labels; self.annotations = annotations
    }
}

public struct EvidenceRef: Codable {
    public let id: CkodexUrn
    public let kind: String?
    public init(id: CkodexUrn, kind: String? = nil) { self.id = id; self.kind = kind }
}

// MARK: - cnsb/v1/cnsb.schema.json  (Cloud-Native Skills Bundle)

public struct CNSBManifest: Codable {
    public let schemaVersion: Int              // const: 1
    public let apiVersion: String              // const: "cnsb.ckodex.org/v1"
    public let kind: String                    // const: "SkillBundle"
    public let metadata: ResourceMetadata
    public let skills: [CNSBSkill]
    public let dependencies: [Urn]?
    public let bundleLayer: CNSBBundleLayer?
}

public struct CNSBBundleLayer: Codable {
    public let mediaType: String               // const: "application/vnd.ai.skill.bundle.v1.tar+zstd"
    public let digest: String                  // sha256:<hex>
    public let size: Int?
}

public struct CNSBSkillParam: Codable {
    public let name: String
    public let type: String?
    public let description: String?
    public let required: Bool?
    public let schemaRef: String?
}

public struct CNSBSkillExample: Codable {
    public let name: String
    public let description: String?
    public let input: [String: AnyCodable]?
    public let expectedOutput: [String: AnyCodable]?
}

public struct CNSBSkillLifecycle: Codable {
    public let status: CNSBLifecycleStatus?
    public let deprecatedAt: String?
    public let retiredAt: String?
    public let supersededBy: String?
    public let hooks: [String: String]?
}

public enum CNSBLifecycleStatus: String, Codable {
    case active       = "active"
    case deprecated   = "deprecated"
    case retired      = "retired"
    case experimental = "experimental"
}

public struct CNSBSkill: Codable {
    public let id: String
    public let entry: String
    public let description: String?
    public let galMin: GAL
    public let galMax: GAL
    public let asc: [ASC]?
    public let inputs: [CNSBSkillParam]?
    public let outputs: [CNSBSkillParam]?
    public let examples: [CNSBSkillExample]?
    public let labels: [String: String]?
    public let lifecycle: CNSBSkillLifecycle?
}

// MARK: - cnsb/v1/skill-lock.schema.json

public enum LockRegistryType: String, Codable {
    case oci = "oci"; case npm = "npm"; case git = "git"
}

public enum LockRegistryAuth: String, Codable {
    case none = "none"; case token = "token"; case oidc = "oidc"
}

public struct LockRegistrySignature: Codable {
    public let keyId: String?
    public let algorithm: String?
    public let value: String?
}

public struct LockedDependency: Codable {
    public let version: String
    public let resolved: String
    public let integrity: String    // SRI: sha256-<base64>
    public let registry: String?
    public let signature: LockRegistrySignature?
    public let dependencies: [String]?   // transitive URNs
}

public struct LockRegistry: Codable {
    public let url: String
    public let type: LockRegistryType?
    public let auth: LockRegistryAuth?
}

public struct CNSBLockFile: Codable {
    public let lockVersion: Int               // const: 1
    public let metadata: SkillLockFileMeta?
    public let dependencies: [String: LockedDependency]
    public let registries: [String: LockRegistry]?
}

// MARK: - pca-lifecycle.v1.schema.json  (Proof-Carrying Action lifecycle bundle)

public enum PCAPrivacyMode: String, Codable {
    case debugLocal      = "debug-local"
    case auditPrivate    = "audit-private"
    case publicAnchor    = "public-anchor"
    case regulatedExport = "regulated-export"
}

public enum LifecycleTrigger: String, Codable {
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

public struct SignalWeights: Codable {
    public let lex: Double?   // lexical
    public let sem: Double?   // semantic
    public let ont: Double?   // ontological
    public let co: Double?    // co-occurrence
    public let rec: Double?   // recency
    public let usr: Double?   // user signal
}

public struct SignalComponents: Codable {
    public let lex: Double?
    public let sem: Double?
    public let ont: Double?
    public let co: Double?
    public let rec: Double?
    public let usr: Double?
}

public struct PCALifecycleDecision: Codable {
    public let skillId: String              // sha256:<hex> canonical skill identity
    public let skillName: String?           // redacted in non-debug modes
    public let fromTier: STXTier
    public let toTier: STXTier
    public let trigger: LifecycleTrigger
    public let score: Double?
    public let components: SignalComponents?
    public let weights: SignalWeights?
    public let drift: Double?
    public let theta: Double?
    public let idleTurns: Int?
    public let capability: String?          // for trigger == capabilityDenied
}

public struct PCALifecycleBundle: Codable {
    public let predicate: String            // const: "ckodex/skill-lifecycle@v1"
    public let subject: String              // urn:ckodex:harness:<id>:turn:<uuid>
    public let timestamp: String
    public let privacyMode: PCAPrivacyMode
    public let decisions: [PCALifecycleDecision]
    public let contextPoolTokensBefore: Int?
    public let contextPoolTokensAfter: Int?
    public let budgetMax: Int?
    public let policyVersion: String        // "ckodex-skill-lifecycle/v<M>.<N>"
    public let weights: SignalWeights?
    public let harnessCapabilityDescriptorHash: String?
}

// MARK: - skill-runtime-capabilities.v1.schema.json

public struct SkillThresholds: Codable {
    public let tauSynLoad: Double?
    public let tauSynKeep: Double?
    public let tauFullLoad: Double?
    public let tauFullKeep: Double?
    public let tauFloor: Double?
    public let thetaDrift: Double?

    enum CodingKeys: String, CodingKey {
        case tauSynLoad  = "tau_syn_load"
        case tauSynKeep  = "tau_syn_keep"
        case tauFullLoad = "tau_full_load"
        case tauFullKeep = "tau_full_keep"
        case tauFloor    = "tau_floor"
        case thetaDrift  = "theta_drift"
    }
}

public struct SkillRuntimeCapabilities: Codable {
    public let supportsSynopsisTier: Bool
    public let supportsOntologyScoring: Bool?
    public let supportsHysteresis: Bool?
    public let supportsEviction: Bool?
    public let supportsLifecyclePca: Bool?
    public let supportsL4xSandbox: Bool?
    public let pcaPrivacyModes: [PCAPrivacyMode]?
    public let maxSkillPoolTokens: Int?
    public let defaultWeights: SignalWeights?
    public let defaultThresholds: SkillThresholds?

    enum CodingKeys: String, CodingKey {
        case supportsSynopsisTier    = "supports_synopsis_tier"
        case supportsOntologyScoring = "supports_ontology_scoring"
        case supportsHysteresis      = "supports_hysteresis"
        case supportsEviction        = "supports_eviction"
        case supportsLifecyclePca    = "supports_lifecycle_pca"
        case supportsL4xSandbox      = "supports_l4x_sandbox"
        case pcaPrivacyModes         = "pca_privacy_modes"
        case maxSkillPoolTokens      = "max_skill_pool_tokens"
        case defaultWeights          = "default_weights"
        case defaultThresholds       = "default_thresholds"
    }
}

// MARK: - stx-domain.v1.schema.json  (Skills Transparency Exchange graph)

/// Tier L0–L4X as used in the STX graph (canonical source for the enum).
public enum STXTier: String, Codable, CaseIterable {
    case l0 = "L0"; case l1 = "L1"; case l2 = "L2"
    case l3 = "L3"; case l4r = "L4R"; case l4x = "L4X"
}

public enum STXTransport: String, Codable {
    case rest = "rest"; case grpc = "grpc"; case graphql = "graphql"
}

public struct STXCapabilityDescriptor: Codable {
    public let stxVersion: String              // const: "v1"
    public let transports: [STXTransport]
    public let privacyModesSupported: [PCAPrivacyMode]
    public let defaultPrivacyMode: PCAPrivacyMode
    public let maxEventsPerSecond: Int?
    public let maxQueryDepth: Int?
    public let harnessId: CkodexUrn?
    public let harnessVersion: String?
    public let frameworkVersion: String?
}

public struct STXHarness: Codable {
    public let id: CkodexUrn
    public let frameworkVersion: String
    public let capability: STXCapabilityDescriptor
    public let sessionIds: [CkodexUrn]?
}

public struct STXSession: Codable {
    public let id: CkodexUrn
    public let harnessId: CkodexUrn
    public let startedAt: String
    public let endedAt: String?
    public let currentGal: GAL
    public let privacyMode: PCAPrivacyMode
    public let skillPoolTokensBudget: Int?
    public let skillPoolTokensCurrent: Int?
    public let skillStateIds: [CkodexUrn]?
}

public struct STXSkillState: Codable {
    public let id: CkodexUrn
    public let sessionId: CkodexUrn
    public let skillId: String              // sha256:<hex>
    public let skillName: String?
    public let skillVersion: String?
    public let tier: STXTier
    public let score: Double
    public let turnsSinceLastReference: Int?
    public let tokenCost: Int?
    public let locked: Bool?
    public let transitionIds: [CkodexUrn]?
}

public struct STXTierTransition: Codable {
    public let id: CkodexUrn
    public let skillStateId: CkodexUrn
    public let timestamp: String
    public let fromTier: STXTier
    public let toTier: STXTier
    public let trigger: LifecycleTrigger
    public let score: Double?
    public let evidenceBundleId: CkodexUrn?
}

public enum STXSignatureType: String, Codable {
    case cosign    = "cosign"
    case pgp       = "pgp"
    case x509      = "x509"
    case dilithium3 = "dilithium3"
}

public struct STXSignatureRef: Codable {
    public let type: STXSignatureType
    public let ref: String
}

public struct STXEvidenceBundle: Codable {
    public let id: CkodexUrn
    public let predicate: String            // const: "ckodex/skill-lifecycle@v1"
    public let subject: CkodexUrn
    public let timestamp: String
    public let privacyMode: PCAPrivacyMode
    public let policyVersion: String?
    public let digest: String               // sha256:<hex>
    public let rekorAnchor: String?
    public let transitionIds: [CkodexUrn]?
    public let attestationChainId: CkodexUrn?
}

public struct STXAttestationChain: Codable {
    public let id: CkodexUrn
    public let issuer: CkodexUrn
    public let signedAt: String
    public let signatures: [STXSignatureRef]
    public let evidenceBundleIds: [CkodexUrn]?
}

public enum STXPolicyGateName: String, Codable {
    case preExec   = "pre_exec"
    case runtime   = "runtime"
    case postExec  = "post_exec"
    case promotion = "promotion"
    case emergency = "emergency"
}

public enum STXPolicyDecision: String, Codable {
    case allow = "allow"; case deny = "deny"; case warn = "warn"
}

public struct STXPolicyGate: Codable {
    public let id: CkodexUrn
    public let name: STXPolicyGateName
    public let decision: STXPolicyDecision
    public let evaluatedAt: String
    public let reason: String?
    public let skillStateId: CkodexUrn?
}

public struct STXDomain: Codable {
    public let apiVersion: String           // const: "ckodex.org/stx/v1"
    public let kind: String                 // const: "StxDomain"
    public let harness: STXHarness?
    public let sessions: [STXSession]?
    public let skillStates: [STXSkillState]?
    public let transitions: [STXTierTransition]?
    public let evidence: [STXEvidenceBundle]?
    public let attestations: [STXAttestationChain]?
    public let policyGates: [STXPolicyGate]?
}

// MARK: - skillsbundle.v1.schema.json

public enum BundleSignatureType: String, Codable {
    case cosign = "cosign"; case pgp = "pgp"; case x509 = "x509"
}

public struct BundleSignatureRef: Codable {
    public let type: BundleSignatureType
    public let ref: String
}

public struct SkillsBundleIntegrity: Codable {
    public let digest: String?
    public let sizeBytes: Int?
}

public struct SkillsBundleSupplyChain: Codable {
    public let sbomRef: String?
    public let provenanceRef: String?
    public let signatures: [BundleSignatureRef]?
}

public struct SkillsBundleEntrypoints: Codable {
    public let skillMd: String              // const: "SKILL.md"
    public let manifest: String?            // "skill.json"
}

public struct SkillsBundleSkill: Codable {
    public let name: String
    public let version: String?
    public let path: String                 // "skills/<name>/"
    public let entrypoints: SkillsBundleEntrypoints
    public let integrity: SkillsBundleIntegrity?
    public let supplyChain: SkillsBundleSupplyChain?
}

public enum BundleSandboxPolicy: String, Codable {
    case required    = "required"
    case recommended = "recommended"
    case none        = "none"
}

public enum BundleUserConfirmation: String, Codable {
    case always       = "always"
    case dangerousOnly = "dangerous-only"
    case never        = "never"
}

public enum BundleProofType: String, Codable {
    case pca  = "PCA"; case uca  = "UCA"; case tip  = "TIP"
    case zkp  = "ZKP"; case dca  = "DCA"; case tkp  = "TKP"
    case wcag = "WCAG"; case clp = "CLP"; case ufp  = "UFP"
}

public enum BundleEnforcementPoint: String, Codable {
    case preExec   = "pre_exec"
    case runtime   = "runtime"
    case postExec  = "post_exec"
    case promotion = "promotion"
    case emergency = "emergency"
}

public struct SkillsBundleGovernanceDefaults: Codable {
    public let galMinimum: Int?
    public let proofRequired: Bool?
    public let proofTypes: [BundleProofType]?
    public let enforcementPoints: [BundleEnforcementPoint]?
    public let emergencyProtocols: [String]?  // "EP-NNN"
}

public struct SkillsBundlePolicyDefaults: Codable {
    public let sandbox: BundleSandboxPolicy?
    public let userConfirmation: BundleUserConfirmation?
}

public struct SkillsBundleDefaults: Codable {
    public let policy: SkillsBundlePolicyDefaults?
    public let allowedToolsRaw: String?
    public let governance: SkillsBundleGovernanceDefaults?
}

public struct SkillsBundleMetadata: Codable {
    public let name: String
    public let version: String
    public let license: String?
    public let authors: [String]?
    public let labels: [String: String]?
}

public struct SkillsBundle: Codable {
    public let apiVersion: String           // const: "ckodex.org/skillsbundle/v1"
    public let kind: String                 // const: "SkillsBundle"
    public let metadata: SkillsBundleMetadata
    public let defaults: SkillsBundleDefaults?
    public let skills: [SkillsBundleSkill]
    public let bundleSupplyChain: SkillsBundleSupplyChain?
}

// MARK: - AnyCodable (lightweight shim for mixed-type JSON objects)

public struct AnyCodable: Codable {
    public let value: Any

    public init(_ value: Any) { self.value = value }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let v = try? container.decode(Bool.self)   { value = v; return }
        if let v = try? container.decode(Int.self)    { value = v; return }
        if let v = try? container.decode(Double.self) { value = v; return }
        if let v = try? container.decode(String.self) { value = v; return }
        if let v = try? container.decode([String: AnyCodable].self) { value = v; return }
        if let v = try? container.decode([AnyCodable].self) { value = v; return }
        value = NSNull()
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch value {
        case let v as Bool:   try container.encode(v)
        case let v as Int:    try container.encode(v)
        case let v as Double: try container.encode(v)
        case let v as String: try container.encode(v)
        case let v as [String: AnyCodable]: try container.encode(v)
        case let v as [AnyCodable]:         try container.encode(v)
        default: try container.encodeNil()
        }
    }
}
