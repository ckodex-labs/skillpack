import Testing
import Foundation
@testable import SkillsCore

struct ImportTests {
    
    private func withImportSandbox(run: (URL, URL) throws -> Void) throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let canonicalStore = tempDir.appendingPathComponent("canonical")
        let sourceDir = tempDir.appendingPathComponent("external-source")
        
        try! FileManager.default.createDirectory(at: canonicalStore, withIntermediateDirectories: true, attributes: nil)
        try! FileManager.default.createDirectory(at: sourceDir, withIntermediateDirectories: true, attributes: nil)
        
        try run(canonicalStore, sourceDir)
    }
    
    @Test func testImportValidSkill() throws {
        try withImportSandbox { canonicalStore, sourceDir in
            let mySkill = TestFixtures.createSkillDirectory(in: sourceDir, name: "new-skill", skillMdContent: "# my new skill")
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: Never.self) {
                try manager.importSkill(from: mySkill.path)
            }
            
            // 1. Physical folder must be copied to canonical
            let targetURL = canonicalStore.appendingPathComponent("new-skill")
            #expect(FileManager.default.fileExists(atPath: targetURL.path))
            #expect(FileManager.default.fileExists(atPath: targetURL.appendingPathComponent("SKILL.md").path))
            
            // 2. Original path should be replaced with a symlink pointing to the canonical path
            let attrs = try FileManager.default.attributesOfItem(atPath: mySkill.path)
            #expect(attrs[.type] as? FileAttributeType == .typeSymbolicLink)
            
            let dest = try FileManager.default.destinationOfSymbolicLink(atPath: mySkill.path)
            #expect(dest == targetURL.path)
        }
    }
    
    @Test func testImportCreatesDefaultSkillMdIfMissing() throws {
        try withImportSandbox { canonicalStore, sourceDir in
            let mySkill = sourceDir.appendingPathComponent("no-md-skill")
            try! FileManager.default.createDirectory(at: mySkill, withIntermediateDirectories: true, attributes: nil)
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.importSkill(from: mySkill.path)
            
            let targetURL = canonicalStore.appendingPathComponent("no-md-skill")
            let skillMd = targetURL.appendingPathComponent("SKILL.md")
            #expect(FileManager.default.fileExists(atPath: skillMd.path))
            
            let content = try String(contentsOf: skillMd, encoding: .utf8)
            #expect(content.contains("Imported skill for no-md-skill"))
        }
    }
    
    @Test func testImportSkillAlreadyExistsReplacesWithLink() throws {
        try withImportSandbox { canonicalStore, sourceDir in
            let mySkill = TestFixtures.createSkillDirectory(in: sourceDir, name: "duplicate-skill", skillMdContent: "# Duplicate")
            
            // Pre-create the duplicate skill in canonical store
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "duplicate-skill", skillMdContent: "# Pre-existing")
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            // Should NOT throw — instead it replaces the physical copy with a symlink
            #expect(throws: Never.self) {
                try manager.importSkill(from: mySkill.path)
            }
            
            // Source should now be a symlink pointing to canonical
            let attrs = try FileManager.default.attributesOfItem(atPath: mySkill.path)
            #expect(attrs[.type] as? FileAttributeType == .typeSymbolicLink)
            
            let dest = try FileManager.default.destinationOfSymbolicLink(atPath: mySkill.path)
            let targetURL = canonicalStore.appendingPathComponent("duplicate-skill")
            #expect(dest == targetURL.path)
        }
    }
    
    @Test func testImportSkillSymlinkSourceIsSkipped() throws {
        try withImportSandbox { canonicalStore, sourceDir in
            let sourceSkill = sourceDir.appendingPathComponent("already-linked-skill")
            let canonicalSkill = canonicalStore.appendingPathComponent("already-linked-skill")
            
            try! FileManager.default.createDirectory(at: canonicalSkill, withIntermediateDirectories: true, attributes: nil)
            try! FileManager.default.createSymbolicLink(at: sourceSkill, withDestinationURL: canonicalSkill)
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            
            // Should complete silently without throwing errors since it's already symlinked
            #expect(throws: Never.self) {
                try manager.importSkill(from: sourceSkill.path)
            }
        }
    }
    
    @Test func testImportSkillIPBlockedThrows() throws {
        try withImportSandbox { canonicalStore, sourceDir in
            // Skill folder has a blocked name
            let blockedSkill = TestFixtures.createSkillDirectory(in: sourceDir, name: "thales-skills", skillMdContent: "# Blocked")
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: IPGuardError.self) {
                try manager.importSkill(from: blockedSkill.path)
            }
        }
    }
}
