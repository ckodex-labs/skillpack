import Foundation

public struct DistributedSkillStorageProvider: SkillStorageProvider {
    private let fdbClient: FoundationDBClient
    private let rustClient: RustFSClient
    private let bucket: String
    
    public init(fdbClient: FoundationDBClient = FoundationDBClient(),
                rustClient: RustFSClient = RustFSClient(),
                bucket: String = "skills") {
        self.fdbClient = fdbClient
        self.rustClient = rustClient
        self.bucket = bucket
    }
    
    public func listSkills() throws -> [SkillMetadata] {
        let prefix = "skills/metadata/"
        let range = try fdbClient.getrange(prefix: prefix)
        
        var result: [SkillMetadata] = []
        let decoder = JSONDecoder()
        
        for (_, jsonString) in range {
            guard let data = jsonString.data(using: .utf8) else { continue }
            if let metadata = try? decoder.decode(SkillMetadata.self, from: data) {
                result.append(metadata)
            }
        }
        
        return result.sorted { $0.name < $1.name }
    }
    
    public func getSkillContent(named name: String) throws -> String {
        let key = "\(name)/SKILL.md"
        guard let data = try rustClient.getObject(bucket: bucket, key: key) else {
            throw SkillsError.custom("Skill payload not found in distributed storage: \(name)")
        }
        guard let content = String(data: data, encoding: .utf8) else {
            throw SkillsError.custom("Skill payload is not valid UTF-8 for skill: \(name)")
        }
        return content
    }
    
    public func storeSkill(named name: String, metadata: SkillMetadata, content: String) throws {
        // 1. Store metadata in FDB
        let encoder = JSONEncoder()
        let metadataData = try encoder.encode(metadata)
        guard let metadataJSON = String(data: metadataData, encoding: .utf8) else {
            throw SkillsError.custom("Failed to serialize metadata for skill: \(name)")
        }
        
        let fdbKey = "skills/metadata/\(name)"
        try fdbClient.set(key: fdbKey, value: metadataJSON)
        
        // 2. Store content in RustFS
        let key = "\(name)/SKILL.md"
        let contentData = Data(content.utf8)
        try rustClient.putObject(bucket: bucket, key: key, data: contentData)
    }
    
    public func deleteSkill(named name: String) throws {
        // 1. Delete metadata from FDB
        let fdbKey = "skills/metadata/\(name)"
        try fdbClient.clear(key: fdbKey)
        
        // 2. Delete content from RustFS
        let key = "\(name)/SKILL.md"
        try rustClient.deleteObject(bucket: bucket, key: key)
    }

    // MARK: - T-03 extensions

    public func getManifest(named name: String) throws -> String? {
        let key = "skills/manifest/\(name)/bundle.json"
        guard let data = try rustClient.getObject(bucket: bucket, key: key) else { return nil }
        return String(data: data, encoding: .utf8)
    }

    public func getEvidence(named name: String, predicateType: String?) throws -> [SkillEvidenceRecord] {
        let prefix = "skills/evidence/\(name)/"
        let range = try fdbClient.getrange(prefix: prefix)
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        var records: [SkillEvidenceRecord] = []
        for (_, jsonString) in range {
            guard let data = jsonString.data(using: .utf8),
                  let record = try? decoder.decode(SkillEvidenceRecord.self, from: data) else { continue }
            if let filter = predicateType, record.predicateType != filter { continue }
            records.append(record)
        }
        return records.sorted { $0.recordedAt > $1.recordedAt }
    }

    public func listVersions(named name: String) throws -> [String] {
        let prefix = "skills/versions/\(name)/"
        let range = try fdbClient.getrange(prefix: prefix)
        let versions = range.map { $0.value }
            .filter { !$0.isEmpty }
        return versions.sorted { $0 > $1 }
    }
}
