import Foundation
import SkillsCore

struct LockFileManager {
    private static let lockFileURL: URL = {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return home.appendingPathComponent(".config/agent-skill-manager/.skill-lock.json")
    }()

    static func load() -> SkillLockFile {
        guard let data = try? Data(contentsOf: lockFileURL),
              let file = try? JSONDecoder().decode(SkillLockFile.self, from: data) else {
            return SkillLockFile()
        }
        return file
    }

    static func save(_ file: SkillLockFile) throws {
        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try enc.encode(file)
        let dir = lockFileURL.deletingLastPathComponent()
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try data.write(to: lockFileURL)
    }

    static func writeLockEntry(dirName: String, source: String, sourceType: String,
                                commitHash: String?, provider: String) throws {
        var file = load()
        file.skills[dirName] = SkillLockEntry(
            source: source,
            sourceType: sourceType,
            commitHash: commitHash,
            provider: provider,
            installedAt: ISO8601DateFormatter().string(from: Date())
        )
        try save(file)
    }

    static func removeLockEntry(dirName: String) throws {
        var file = load()
        file.skills.removeValue(forKey: dirName)
        try save(file)
    }
}
