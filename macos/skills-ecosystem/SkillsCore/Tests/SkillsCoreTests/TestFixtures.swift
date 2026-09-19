import Foundation

struct TestFixtures {
    static func createTempDirectory() -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("com.ckodex.skills.test-\(UUID().uuidString)")
        try! FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true, attributes: nil)
        return tempDir
    }

    static func removeTempDirectory(_ url: URL) {
        try? FileManager.default.removeItem(at: url)
    }

    static func createSkillDirectory(in root: URL, name: String, skillMdContent: String?) -> URL {
        let skillDir = root.appendingPathComponent(name)
        try! FileManager.default.createDirectory(at: skillDir, withIntermediateDirectories: true, attributes: nil)
        if let content = skillMdContent {
            let fileURL = skillDir.appendingPathComponent("SKILL.md")
            try! content.write(to: fileURL, atomically: true, encoding: .utf8)
        }
        return skillDir
    }
}
