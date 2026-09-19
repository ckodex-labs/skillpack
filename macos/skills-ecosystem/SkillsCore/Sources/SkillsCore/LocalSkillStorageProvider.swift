import Foundation

public struct LocalSkillStorageProvider: SkillStorageProvider {
    private let skillsRoot: URL
    private let fileManager = FileManager.default
    
    public init(skillsRoot: URL) {
        self.skillsRoot = skillsRoot
    }
    
    public func listSkills() throws -> [SkillMetadata] {
        let discovery = SkillDiscovery()
        let discovered = try discovery.discoverSkills(in: skillsRoot)
        return discovered.map { skill in
            SkillMetadata(
                name: skill.name,
                sizeBytes: skill.sizeBytes,
                description: skill.description,
                version: skill.version,
                contentHash: skill.contentHash,
                tier: skill.tier,
                ontologyRef: skill.ontologyRef
            )
        }
    }
    
    public func getSkillContent(named name: String) throws -> String {
        let skillMdURL = skillsRoot.appendingPathComponent(name).appendingPathComponent("SKILL.md")
        guard fileManager.fileExists(atPath: skillMdURL.path) else {
            throw SkillsError.invalidSourceDirectory(path: skillMdURL.path)
        }
        return try String(contentsOf: skillMdURL, encoding: .utf8)
    }
    
    public func storeSkill(named name: String, metadata: SkillMetadata, content: String) throws {
        let skillDir = skillsRoot.appendingPathComponent(name)
        try fileManager.createDirectory(at: skillDir, withIntermediateDirectories: true, attributes: nil)
        let skillMdURL = skillDir.appendingPathComponent("SKILL.md")
        try content.write(to: skillMdURL, atomically: true, encoding: .utf8)
    }
    
    public func deleteSkill(named name: String) throws {
        let skillDir = skillsRoot.appendingPathComponent(name)
        if fileManager.fileExists(atPath: skillDir.path) {
            try fileManager.removeItem(at: skillDir)
        }
    }

    // MARK: - T-03 extensions

    public func getManifest(named name: String) throws -> String? {
        let bundleJsonURL = skillsRoot.appendingPathComponent(name).appendingPathComponent("bundle.json")
        guard fileManager.fileExists(atPath: bundleJsonURL.path) else { return nil }
        return try String(contentsOf: bundleJsonURL, encoding: .utf8)
    }

    public func getEvidence(named name: String, predicateType: String?) throws -> [SkillEvidenceRecord] {
        let evidenceDir = skillsRoot.appendingPathComponent(name).appendingPathComponent(".evidence")
        guard fileManager.fileExists(atPath: evidenceDir.path) else { return [] }
        let files = try fileManager.contentsOfDirectory(at: evidenceDir, includingPropertiesForKeys: nil)
            .filter { $0.pathExtension == "json" }
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        var records: [SkillEvidenceRecord] = []
        for file in files {
            guard let data = try? Data(contentsOf: file),
                  let record = try? decoder.decode(SkillEvidenceRecord.self, from: data) else { continue }
            if let filter = predicateType, record.predicateType != filter { continue }
            records.append(record)
        }
        return records.sorted { $0.recordedAt > $1.recordedAt }
    }

    public func listVersions(named name: String) throws -> [String] {
        let versionsDir = skillsRoot.appendingPathComponent(name).appendingPathComponent(".versions")
        guard fileManager.fileExists(atPath: versionsDir.path) else { return [] }
        let files = try fileManager.contentsOfDirectory(at: versionsDir, includingPropertiesForKeys: nil)
            .filter { $0.pathExtension == "md" }
            .map { $0.deletingPathExtension().lastPathComponent }
            .filter { !$0.isEmpty }
        return files.sorted { $0 > $1 }
    }
}
