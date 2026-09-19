import XCTest
@testable import SkillsCore

final class FrontmatterParserTests: XCTestCase {

    let parser = FrontmatterParser()

    // MARK: - Basic parsing

    func testParsesAllStandardFields() {
        let content = """
        ---
        name: my-skill
        version: "1.2.3"
        description: "A test skill for unit tests."
        creator: "Alice"
        license: "MIT"
        compatibility: "claude,codex"
        tools: [bash, read_file]
        effort: "low"
        ---
        # Body
        """
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.name, "my-skill")
        XCTAssertEqual(fm.version, "1.2.3")
        XCTAssertEqual(fm.description, "A test skill for unit tests.")
        XCTAssertEqual(fm.creator, "Alice")
        XCTAssertEqual(fm.license, "MIT")
        XCTAssertEqual(fm.compatibility, "claude,codex")
        XCTAssertEqual(fm.allowedTools, ["bash", "read_file"])
        XCTAssertEqual(fm.effort, "low")
    }

    func testDefaultVersionWhenMissing() {
        let content = "---\ndescription: hi\n---"
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.version, "0.0.0")
    }

    func testEmptyContentReturnsDefaults() {
        let fm = parser.parse(content: "")
        XCTAssertEqual(fm.version, "0.0.0")
        XCTAssertTrue(fm.description.isEmpty)
        XCTAssertTrue(fm.allowedTools.isEmpty)
    }

    func testMultilineToolsArray() {
        let content = """
        ---
        tools:
          - bash
          - read_file
          - write_to_file
        ---
        """
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.allowedTools, ["bash", "read_file", "write_to_file"])
    }

    func testInlineToolsArray() {
        let content = "---\ntools: [bash, read_file]\n---"
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.allowedTools, ["bash", "read_file"])
    }

    func testAscInlineArray() {
        let content = "---\nasc: [anti, execute]\n---"
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.asc, ["anti", "execute"])
    }

    func testLifecycleParsed() {
        let content = "---\nlifecycle: retired\n---"
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.lifecycle, "retired")
    }

    func testDescriptionTruncatedAt1024() {
        let longDesc = String(repeating: "x", count: 1100)
        let content = "---\ndescription: \"\(longDesc)\"\n---"
        let fm = parser.parse(content: content)
        XCTAssertEqual(fm.description.count, 1024)
    }

    func testNoFrontmatterReturnsEmpty() {
        let content = "# Just a heading\nSome body text."
        let fm = parser.parse(content: content)
        XCTAssertTrue(fm.name.isEmpty)
        XCTAssertEqual(fm.version, "0.0.0")
    }
}
