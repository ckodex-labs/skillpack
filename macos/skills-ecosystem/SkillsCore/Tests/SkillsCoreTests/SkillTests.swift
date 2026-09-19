import Testing
import Foundation
@testable import SkillsCore

struct SkillTests {
    
    @Test func testInitWithFrontmatterDescription() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let markdown = """
        ---
        description: "A super cool skill description"
        ---
        # Skill Heading
        """
        
        let skillURL = TestFixtures.createSkillDirectory(in: tempDir, name: "cool-skill", skillMdContent: markdown)
        let skill = Skill(url: skillURL)
        
        #expect(skill.name == "cool-skill")
        #expect(skill.description == "A super cool skill description")
    }
    
    @Test func testInitWithoutFrontmatterFallsback() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let markdown = """
        # Skill Heading
        
        This is the fallback paragraph description of the skill.
        Another paragraph.
        """
        
        let skillURL = TestFixtures.createSkillDirectory(in: tempDir, name: "fallback-skill", skillMdContent: markdown)
        let skill = Skill(url: skillURL)
        
        #expect(skill.description == "This is the fallback paragraph description of the skill.")
    }
    
    @Test func testInitMissingSkillMd() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let skillURL = tempDir.appendingPathComponent("empty-skill")
        try! FileManager.default.createDirectory(at: skillURL, withIntermediateDirectories: true, attributes: nil)
        
        let skill = Skill(url: skillURL)
        #expect(skill.description == "")
    }
    
    @Test func testDescriptionTruncation() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let longText = String(repeating: "A", count: 250)
        let markdown = """
        ---
        description: "\(longText)"
        ---
        """
        
        let skillURL = TestFixtures.createSkillDirectory(in: tempDir, name: "long-skill", skillMdContent: markdown)
        let skill = Skill(url: skillURL)
        
        #expect(skill.description.count == 200)
    }
    
    @Test func testDescriptionQuotesEscaped() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let markdown = """
        ---
        description: "Skill with \\"quotes\\""
        ---
        """
        
        let skillURL = TestFixtures.createSkillDirectory(in: tempDir, name: "quoted-skill", skillMdContent: markdown)
        let skill = Skill(url: skillURL)
        
        #expect(skill.description == "Skill with \\quotes\\")
    }
    
    @Test func testCalculateSize() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let file1 = tempDir.appendingPathComponent("file1.txt")
        let file2 = tempDir.appendingPathComponent("file2.txt")
        
        try! "12345".write(to: file1, atomically: true, encoding: .utf8)
        try! "abcde".write(to: file2, atomically: true, encoding: .utf8)
        
        let size = Skill.calculateSize(of: tempDir)
        #expect(size == 10)
    }
}
