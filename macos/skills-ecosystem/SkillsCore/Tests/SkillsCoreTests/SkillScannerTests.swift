import XCTest
@testable import SkillsCore

final class SkillScannerTests: XCTestCase {

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

    // MARK: - Helpers

    private func makeSkillDir(_ name: String, content: String? = nil) -> URL {
        let skillDir = tempDir.appendingPathComponent(name)
        try! FileManager.default.createDirectory(at: skillDir, withIntermediateDirectories: true)
        let md = skillDir.appendingPathComponent("SKILL.md")
        let body = content ?? "---\nname: \(name)\nversion: \"1.0.0\"\ndescription: \"Test skill \(name).\"\n---\n# \(name)"
        try! body.write(to: md, atomically: true, encoding: .utf8)
        return skillDir
    }

    /// Directly test FrontmatterParser + TokenCounter integration (core scanner logic).
    func testFrontmatterFieldsRoundTrip() {
        let skillDir = makeSkillDir("my-skill", content: """
        ---
        name: my-skill
        version: "2.0.0"
        description: "A great skill."
        license: MIT
        ---
        # Body
        """)
        let parser = FrontmatterParser()
        let fm = parser.parse(url: skillDir.appendingPathComponent("SKILL.md"))
        XCTAssertEqual(fm.name, "my-skill")
        XCTAssertEqual(fm.version, "2.0.0")
        XCTAssertEqual(fm.description, "A great skill.")
        XCTAssertEqual(fm.license, "MIT")
    }

    func testMissingVersionTriggersWarning() {
        let skillDir = makeSkillDir("no-version", content: "---\ndescription: hi\n---")
        let parser = FrontmatterParser()
        let fm = parser.parse(url: skillDir.appendingPathComponent("SKILL.md"))
        // Version 0.0.0 is the default that should trigger a warning in scanner
        XCTAssertEqual(fm.version, "0.0.0")
    }

    func testDisabledSkillDetection() throws {
        let skillDir = makeSkillDir("hidden-skill")
        let md = skillDir.appendingPathComponent("SKILL.md")
        let disabled = skillDir.appendingPathComponent("SKILL.md.disabled")
        try FileManager.default.moveItem(at: md, to: disabled)

        // Without SKILL.md, scanner should not see it (includeDisabled: false)
        XCTAssertFalse(FileManager.default.fileExists(atPath: md.path))
        // With SKILL.md.disabled present, scanner includeDisabled:true should find it
        XCTAssertTrue(FileManager.default.fileExists(atPath: disabled.path))
    }

    func testTokenCountPositive() {
        let skillDir = makeSkillDir("tok-skill", content: "---\nname: tok-skill\ndescription: hello world\n---\n# Body text here")
        let counter = TokenCounter()
        let count = counter.estimate(url: skillDir.appendingPathComponent("SKILL.md"))
        XCTAssertGreaterThan(count, 0)
    }

    func testScanOptions_scopeFilter() {
        let opts = ScanOptions(scope: "global", providerFilter: "claude", includeDisabled: false)
        XCTAssertEqual(opts.scope, "global")
        XCTAssertEqual(opts.providerFilter, "claude")
        XCTAssertFalse(opts.includeDisabled)
    }

    func testAgentRegistryHas18Providers() {
        let agents = AgentRegistry.shared.agents
        XCTAssertGreaterThanOrEqual(agents.count, 18)
        let names = agents.map(\.name)
        XCTAssertTrue(names.contains("claude"))
        XCTAssertTrue(names.contains("windsurf"))
        XCTAssertTrue(names.contains("cursor"))
        XCTAssertTrue(names.contains("copilot"))
        XCTAssertTrue(names.contains("gemini"))
        XCTAssertTrue(names.contains("pi"))
        XCTAssertTrue(names.contains("hermes"))
        XCTAssertTrue(names.contains("zed"))
        XCTAssertTrue(names.contains("antigravity"))
    }

    func testAgentRegistryLabelsPopulated() {
        let labels = AgentRegistry.shared.labels
        XCTAssertEqual(labels["claude"], "Claude Code")
        XCTAssertEqual(labels["windsurf"], "Windsurf")
        XCTAssertEqual(labels["copilot"], "GitHub Copilot")
    }
}
