import Testing
import Foundation
@testable import SkillsCore

@Suite(.serialized) struct MigrateAllTests {
    
    private func withMigrationSandbox(run: (URL, URL) throws -> Void) throws {
        let root = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(root) }
        
        let canonicalStore = root.appendingPathComponent("canonical")
        try! FileManager.default.createDirectory(at: canonicalStore, withIntermediateDirectories: true, attributes: nil)
        
        // Create sandbox agent directories and set env overrides
        var oldEnv: [String: String?] = [:]
        for agent in AgentRegistry.shared.agents {
            let agentDir = root.appendingPathComponent("\(agent.name)-sandbox")
            try! FileManager.default.createDirectory(at: agentDir, withIntermediateDirectories: true, attributes: nil)
            oldEnv[agent.envOverrideKey] = ProcessInfo.processInfo.environment[agent.envOverrideKey]
            setenv(agent.envOverrideKey, agentDir.path, 1)
        }
        
        defer {
            for (key, val) in oldEnv {
                if let v = val {
                    setenv(key, v, 1)
                } else {
                    unsetenv(key)
                }
            }
        }
        
        try run(canonicalStore, root)
    }
    
    @Test func testMigratePhysicalSkillToCanonical() throws {
        try withMigrationSandbox { canonicalStore, root in
            // Place a physical skill in the claude sandbox
            let claudeDir = root.appendingPathComponent("claude-sandbox")
            let physicalSkill = TestFixtures.createSkillDirectory(
                in: claudeDir, name: "new-skill",
                skillMdContent: "---\ndescription: \"A new skill\"\n---"
            )
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let result = strategy.migrateSkill(from: physicalSkill, agentName: "claude")
            
            #expect(result.outcome == .migrated)
            
            // Skill should now exist in canonical
            let canonicalSkill = canonicalStore.appendingPathComponent("new-skill")
            #expect(FileManager.default.fileExists(atPath: canonicalSkill.path))
            
            // Original should be a symlink
            let attrs = try FileManager.default.attributesOfItem(atPath: physicalSkill.path)
            #expect(attrs[.type] as? FileAttributeType == .typeSymbolicLink)
        }
    }
    
    @Test func testMigrateWhenCanonicalAlreadyHasSkill() throws {
        try withMigrationSandbox { canonicalStore, root in
            // Pre-existing skill in canonical
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "existing-skill",
                skillMdContent: "---\ndescription: \"Canonical version\"\n---"
            )
            
            // Physical copy in agent dir
            let geminiDir = root.appendingPathComponent("gemini-sandbox")
            let physicalSkill = TestFixtures.createSkillDirectory(
                in: geminiDir, name: "existing-skill",
                skillMdContent: "---\ndescription: \"Agent version\"\n---"
            )
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let result = strategy.migrateSkill(from: physicalSkill, agentName: "gemini")
            
            #expect(result.outcome == .replacedWithLink)
            
            // Original should be a symlink now
            let attrs = try FileManager.default.attributesOfItem(atPath: physicalSkill.path)
            #expect(attrs[.type] as? FileAttributeType == .typeSymbolicLink)
            
            // Symlink should point to canonical
            let dest = try FileManager.default.destinationOfSymbolicLink(atPath: physicalSkill.path)
            let canonicalSkill = canonicalStore.appendingPathComponent("existing-skill")
            #expect(dest == canonicalSkill.path)
        }
    }
    
    @Test func testMigrateAlreadyLinkedSkillIsSkipped() throws {
        try withMigrationSandbox { canonicalStore, root in
            // Create canonical skill
            let canonicalSkill = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "linked-skill",
                skillMdContent: "---\ndescription: \"Linked\"\n---"
            )
            
            // Create a symlink in agent dir
            let windDir = root.appendingPathComponent("windsurf-sandbox")
            let linkURL = windDir.appendingPathComponent("linked-skill")
            try FileManager.default.createSymbolicLink(at: linkURL, withDestinationURL: canonicalSkill)
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let result = strategy.migrateSkill(from: linkURL, agentName: "windsurf")
            
            #expect(result.outcome == .skippedAlreadyLinked)
        }
    }
    
    @Test func testMigrateIPBlockedSkillIsSkipped() throws {
        try withMigrationSandbox { canonicalStore, root in
            let claudeDir = root.appendingPathComponent("claude-sandbox")
            let blockedSkill = TestFixtures.createSkillDirectory(
                in: claudeDir, name: "thales-secret",
                skillMdContent: "---\ndescription: \"Blocked\"\n---"
            )
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let result = strategy.migrateSkill(from: blockedSkill, agentName: "claude")
            
            if case .skippedIPViolation = result.outcome {
                // expected
            } else {
                Issue.record("Expected skippedIPViolation but got \(result.outcome)")
            }
            
            // Physical skill should still exist untouched
            #expect(FileManager.default.fileExists(atPath: blockedSkill.path))
        }
    }
    
    @Test func testMigrateAllBatchProcesses() throws {
        try withMigrationSandbox { canonicalStore, root in
            // Put physical skills in multiple agent dirs
            let claudeDir = root.appendingPathComponent("claude-sandbox")
            _ = TestFixtures.createSkillDirectory(in: claudeDir, name: "skill-a", skillMdContent: "# A")
            
            let codexDir = root.appendingPathComponent("codex-sandbox")
            _ = TestFixtures.createSkillDirectory(in: codexDir, name: "skill-b", skillMdContent: "# B")
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let results = strategy.migrateAll()
            
            // Should have processed both skills
            let migrated = results.filter { $0.outcome == .migrated }
            #expect(migrated.count == 2)
            
            // Both should now exist in canonical
            #expect(FileManager.default.fileExists(atPath: canonicalStore.appendingPathComponent("skill-a").path))
            #expect(FileManager.default.fileExists(atPath: canonicalStore.appendingPathComponent("skill-b").path))
        }
    }
    
    @Test func testMigrateAllCreatesDefaultSkillMdIfMissing() throws {
        try withMigrationSandbox { canonicalStore, root in
            let aiderDir = root.appendingPathComponent("aider-sandbox")
            let noMdSkill = aiderDir.appendingPathComponent("no-md-skill")
            try FileManager.default.createDirectory(at: noMdSkill, withIntermediateDirectories: true, attributes: nil)
            
            let strategy = MigrateAllStrategy(skillsRoot: canonicalStore, logger: { _ in })
            let result = strategy.migrateSkill(from: noMdSkill, agentName: "aider")
            
            #expect(result.outcome == .migrated)
            
            let skillMd = canonicalStore.appendingPathComponent("no-md-skill").appendingPathComponent("SKILL.md")
            #expect(FileManager.default.fileExists(atPath: skillMd.path))
            
            let content = try String(contentsOf: skillMd, encoding: .utf8)
            #expect(content.contains("Migrated skill for no-md-skill"))
        }
    }
    
    @Test func testSyncManagerMigrateAll() throws {
        try withMigrationSandbox { canonicalStore, root in
            // Need a skill in canonical for SyncManager.execute to work
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "base-skill",
                skillMdContent: "---\ndescription: \"Base\"\n---"
            )
            
            let claudeDir = root.appendingPathComponent("claude-sandbox")
            _ = TestFixtures.createSkillDirectory(in: claudeDir, name: "migrate-me", skillMdContent: "# Migrate")
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            let results = try manager.migrateAll()
            
            let migrated = results.filter { $0.outcome == .migrated }
            #expect(migrated.count >= 1)
            #expect(migrated.contains(where: { $0.skillName == "migrate-me" }))
        }
    }
    
    @Test func testMigrateSummaryOutput() throws {
        let results = [
            MigrateResult(agentName: "claude", skillName: "a", sourcePath: "/tmp/a", outcome: .migrated),
            MigrateResult(agentName: "codex", skillName: "b", sourcePath: "/tmp/b", outcome: .replacedWithLink),
            MigrateResult(agentName: "gemini", skillName: "c", sourcePath: "/tmp/c", outcome: .skippedAlreadyLinked),
            MigrateResult(agentName: "claude", skillName: "d", sourcePath: "/tmp/d", outcome: .skippedIPViolation("blocked")),
        ]
        
        var logs: [String] = []
        MigrateAllStrategy.printSummary(results) { logs.append($0) }
        
        let output = logs.joined(separator: "\n")
        #expect(output.contains("Migrated (new)         : 1"))
        #expect(output.contains("Replaced with link     : 1"))
        #expect(output.contains("Skipped (already linked): 1"))
        #expect(output.contains("Skipped (IP violation)  : 1"))
        #expect(output.contains("Total processed        : 4"))
    }
}
