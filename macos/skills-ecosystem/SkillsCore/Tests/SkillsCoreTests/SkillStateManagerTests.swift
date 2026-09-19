import XCTest
@testable import SkillsCore

final class SkillStateManagerTests: XCTestCase {

    var tempDir: URL!

    override func setUp() {
        super.setUp()
        tempDir = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
        try! FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        try? FileManager.default.removeItem(at: tempDir)
        super.tearDown()
    }

    private func makeSkill(dirName: String, provider: String = "claude", scope: String = "global") -> SkillInfo {
        let skillDir = tempDir.appendingPathComponent(dirName)
        try! FileManager.default.createDirectory(at: skillDir, withIntermediateDirectories: true)
        let md = skillDir.appendingPathComponent("SKILL.md")
        try! "---\nname: \(dirName)\nversion: 1.0.0\n---".write(to: md, atomically: true, encoding: .utf8)

        return SkillInfo(
            name: dirName, version: "1.0.0", description: "", creator: "", license: "",
            compatibility: "", allowedTools: [], dirName: dirName,
            path: skillDir.path, originalPath: skillDir.path,
            location: "\(scope)-\(provider)", scope: scope, provider: provider,
            providerLabel: provider, isSymlink: false, symlinkTarget: nil,
            realPath: skillDir.path, tokenCount: nil, warnings: []
        )
    }

    func testDisableRenamesMd() throws {
        let skill = makeSkill(dirName: "test-skill")
        let mgr = SkillStateManager()
        _ = try mgr.disable(skills: [skill])

        let md = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md")
        let disabled = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md.disabled")
        XCTAssertFalse(FileManager.default.fileExists(atPath: md.path))
        XCTAssertTrue(FileManager.default.fileExists(atPath: disabled.path))
    }

    func testEnableRestoresMd() throws {
        let skill = makeSkill(dirName: "restore-skill")
        let mgr = SkillStateManager()

        // Disable first
        _ = try mgr.disable(skills: [skill])

        // Now re-enable with the disabled flag set
        var disabledSkill = skill
        disabledSkill.disabled = true
        _ = try mgr.enable(skills: [disabledSkill])

        let md = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md")
        XCTAssertTrue(FileManager.default.fileExists(atPath: md.path))
    }
}
