import Foundation

public struct SkillMetadata: Codable, Equatable {
    public let name: String
    public let sizeBytes: Int
    public let description: String
    /// T-02/T-03: Semantic version string (e.g. "1.2.0").
    public let version: String?
    /// T-02/T-03: SHA-256 hex digest of SKILL.md, prefixed "sha256:".
    public let contentHash: String?
    /// T-02/T-03: Deployment tier (e.g. "stable", "experimental").
    public let tier: String?
    /// T-02/T-03: Ontology URN reference for this skill.
    public let ontologyRef: String?

    public init(
        name: String,
        sizeBytes: Int,
        description: String,
        version: String? = nil,
        contentHash: String? = nil,
        tier: String? = nil,
        ontologyRef: String? = nil
    ) {
        self.name = name
        self.sizeBytes = sizeBytes
        self.description = description
        self.version = version
        self.contentHash = contentHash
        self.tier = tier
        self.ontologyRef = ontologyRef
    }
}

/// T-03: Evidence record returned by getEvidence.
public struct SkillEvidenceRecord: Codable, Equatable {
    public let skillName: String
    /// Predicate type URI (e.g. "urn:skill:static-analysis:v1").
    public let predicateType: String
    /// Raw JSON payload of the evidence.
    public let payload: String
    public let recordedAt: Date

    public init(skillName: String, predicateType: String, payload: String, recordedAt: Date = Date()) {
        self.skillName = skillName
        self.predicateType = predicateType
        self.payload = payload
        self.recordedAt = recordedAt
    }
}

public protocol SkillStorageProvider {
    func listSkills() throws -> [SkillMetadata]
    func getSkillContent(named: String) throws -> String
    func storeSkill(named: String, metadata: SkillMetadata, content: String) throws
    func deleteSkill(named: String) throws

    /// T-03: Return the stored manifest blob (e.g. bundle.json content) for a named skill.
    func getManifest(named: String) throws -> String?

    /// T-03: Return all evidence records for a named skill, optionally filtered by predicateType.
    func getEvidence(named: String, predicateType: String?) throws -> [SkillEvidenceRecord]

    /// T-03: Return all known version strings for a named skill, newest first.
    func listVersions(named: String) throws -> [String]
}
