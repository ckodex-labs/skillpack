import Testing
import Foundation
@testable import SkillsCore

struct IPGuardTests {
    
    @Test func testGuardPathWithCleanPath() throws {
        let cleanURL = URL(fileURLWithPath: "/tmp/safe-skills/my-awesome-skill")
        #expect(throws: Never.self) {
            try IPGuard.guardPath(cleanURL)
        }
    }
    
    @Test func testGuardPathWithThalesPattern() throws {
        let dirtyURL1 = URL(fileURLWithPath: "/tmp/safe-skills/thales-rules")
        let dirtyURL2 = URL(fileURLWithPath: "/tmp/safe-skills/cortaix-csr-notes")
        let dirtyURL3 = URL(fileURLWithPath: "/tmp/safe-skills/ppt-thales-design")
        let dirtyURL4 = URL(fileURLWithPath: "/tmp/safe-skills/ip-pending-idea")
        
        #expect(throws: IPGuardError.self) { try IPGuard.guardPath(dirtyURL1) }
        #expect(throws: IPGuardError.self) { try IPGuard.guardPath(dirtyURL2) }
        #expect(throws: IPGuardError.self) { try IPGuard.guardPath(dirtyURL3) }
        #expect(throws: IPGuardError.self) { try IPGuard.guardPath(dirtyURL4) }
    }
    
    @Test func testGuardPathCaseInsensitive() throws {
        let dirtyURL = URL(fileURLWithPath: "/tmp/safe-skills/THALES-Project")
        #expect(throws: IPGuardError.self) {
            try IPGuard.guardPath(dirtyURL)
        }
    }
    
    @Test func testGuardSkillNameClean() throws {
        #expect(throws: Never.self) {
            try IPGuard.guardSkillName("my-skill")
        }
    }
    
    @Test func testGuardSkillNameDirty() throws {
        #expect(throws: IPGuardError.self) { try IPGuard.guardSkillName("thales") }
        #expect(throws: IPGuardError.self) { try IPGuard.guardSkillName("cortaix-csr") }
        #expect(throws: IPGuardError.self) { try IPGuard.guardSkillName("ppt-thales") }
        #expect(throws: IPGuardError.self) { try IPGuard.guardSkillName("ip-pending") }
    }
    
    @Test func testScanForThalesInEmptyDir() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        #expect(throws: Never.self) {
            try IPGuard.scanForThales(in: tempDir)
        }
    }
    
    @Test func testScanForThalesWithViolationAtDepth2() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        // Depth 2: tempDir/skill/cortaix-csr
        let skillDir = tempDir.appendingPathComponent("my-skill")
        try! FileManager.default.createDirectory(at: skillDir, withIntermediateDirectories: true, attributes: nil)
        let violationFile = skillDir.appendingPathComponent("cortaix-csr.txt")
        try! "secret data".write(to: violationFile, atomically: true, encoding: .utf8)
        
        #expect(throws: IPGuardError.self) {
            try IPGuard.scanForThales(in: tempDir)
        }
    }
    
    @Test func testScanForThalesWithViolationDeeperThanMaxDepth() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        // Depth 4: tempDir/skill/sub1/sub2/thales.txt
        let deepDir = tempDir.appendingPathComponent("my-skill").appendingPathComponent("sub1").appendingPathComponent("sub2")
        try! FileManager.default.createDirectory(at: deepDir, withIntermediateDirectories: true, attributes: nil)
        let violationFile = deepDir.appendingPathComponent("thales.txt")
        try! "ignored deep data".write(to: violationFile, atomically: true, encoding: .utf8)
        
        // maxDepth = 2 -> Should ignore depth 4 and pass cleanly
        #expect(throws: Never.self) {
            try IPGuard.scanForThales(in: tempDir, maxDepth: 2)
        }
    }
}
