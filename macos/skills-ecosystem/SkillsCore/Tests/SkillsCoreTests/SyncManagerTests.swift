import Testing
import Foundation
@testable import SkillsCore

@Suite(.serialized) struct SyncManagerTests {
    
    // Helper to override environment variables during a test execution
    private func withSandbox(run: (URL, URL, URL) throws -> Void) throws {
        let root = TestFixtures.createTempDirectory()
        let claudeDir = root.appendingPathComponent("claude-sandbox")
        let cursorDir = root.appendingPathComponent("cursor-sandbox")
        
        try! FileManager.default.createDirectory(at: claudeDir, withIntermediateDirectories: true, attributes: nil)
        try! FileManager.default.createDirectory(at: cursorDir, withIntermediateDirectories: true, attributes: nil)
        
        var oldEnv: [String: String?] = [:]
        for agent in AgentRegistry.shared.agents {
            let agentDir: URL
            if agent.name == "claude" {
                agentDir = claudeDir
            } else if agent.name == "cursor" {
                agentDir = cursorDir
            } else {
                agentDir = root.appendingPathComponent("\(agent.name)-sandbox")
                try! FileManager.default.createDirectory(at: agentDir, withIntermediateDirectories: true, attributes: nil)
            }
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
            TestFixtures.removeTempDirectory(root)
        }
        
        let canonicalStore = root.appendingPathComponent("canonical")
        try! FileManager.default.createDirectory(at: canonicalStore, withIntermediateDirectories: true, attributes: nil)
        
        try run(canonicalStore, claudeDir, cursorDir)
    }
    
    @Test func testExecuteValidCanonicalStore() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            // Create a physical skill
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "math-skill", skillMdContent: "---\ndescription: \"Math functions\"\n---")
            
            var logs: [String] = []
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { logs.append($0) }
            
            #expect(throws: Never.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
            }
            
            // Verify manifest generated
            let manifest = canonicalStore.appendingPathComponent("manifest.yaml")
            #expect(FileManager.default.fileExists(atPath: manifest.path))
            
            let manifestContent = try String(contentsOf: manifest, encoding: .utf8)
            #expect(manifestContent.contains("math-skill"))
            
            // Verify symlink created for Claude
            let symlink = claudeDir.appendingPathComponent("math-skill")
            #expect(FileManager.default.fileExists(atPath: symlink.path))
            
            // Verify cursor index generated
            let cursorIndex = cursorDir.appendingPathComponent("skills-index.mdc")
            #expect(FileManager.default.fileExists(atPath: cursorIndex.path))
        }
    }
    
    @Test func testExecuteDryRunDoesNotWrite() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "dry-skill", skillMdContent: "---\ndescription: \"Dry run\"\n---")
            
            var logs: [String] = []
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { logs.append($0) }
            
            try manager.execute(options: SyncOptions(dryRun: true, doIndex: false, statusOnly: false, onlyAgent: nil))
            
            // Verify no files are written
            let manifest = canonicalStore.appendingPathComponent("manifest.yaml")
            #expect(!FileManager.default.fileExists(atPath: manifest.path))
            
            let symlink = claudeDir.appendingPathComponent("dry-skill")
            #expect(!FileManager.default.fileExists(atPath: symlink.path))
            
            let cursorIndex = cursorDir.appendingPathComponent("skills-index.mdc")
            #expect(!FileManager.default.fileExists(atPath: cursorIndex.path))
            
            #expect(logs.contains(where: { $0.contains("would write") || $0.contains("would link") }))
        }
    }
    
    @Test func testExecuteStatusOnlyDoesNotWrite() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "status-skill", skillMdContent: "---\ndescription: \"Status check\"\n---")
            
            var logs: [String] = []
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { logs.append($0) }
            
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: true, onlyAgent: nil))
            
            let manifest = canonicalStore.appendingPathComponent("manifest.yaml")
            #expect(!FileManager.default.fileExists(atPath: manifest.path))
        }
    }
    
    @Test func testExecuteMissingRootDirThrows() throws {
        let missingPath = "/tmp/com.ckodex.non-existent-\(UUID().uuidString)"
        let manager = SyncManager(skillsRootPath: missingPath) { _ in }
        
        #expect(throws: SkillsError.self) {
            try manager.execute(options: SyncOptions())
        }
    }
    
    @Test func testExecuteEmptyRootDirThrows() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        
        let manager = SyncManager(skillsRootPath: tempDir.path) { _ in }
        
        #expect(throws: SkillsError.self) {
            try manager.execute(options: SyncOptions())
        }
    }
    
    @Test func testExecuteOnlyAgentFilter() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "filtered-skill", skillMdContent: "---\ndescription: \"Filtered\"\n---")
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            
            // Sync ONLY Claude
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: "claude"))
            
            let symlink = claudeDir.appendingPathComponent("filtered-skill")
            #expect(FileManager.default.fileExists(atPath: symlink.path))
            
            // Cursor should NOT be sync'd (no index file written)
            let cursorIndex = cursorDir.appendingPathComponent("skills-index.mdc")
            #expect(!FileManager.default.fileExists(atPath: cursorIndex.path))
        }
    }
    
    @Test func testSymlinkPruning() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            // Create a valid skill in canonical and link it
            _ = TestFixtures.createSkillDirectory(in: canonicalStore, name: "active-skill", skillMdContent: "---\ndescription: \"Active\"\n---")
            
            // Create a dangling symlink inside claudeDir pointing to non-existent skill
            let danglingLink = claudeDir.appendingPathComponent("stale-skill")
            try! FileManager.default.createSymbolicLink(at: danglingLink, withDestinationURL: canonicalStore.appendingPathComponent("stale-skill"))
            
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: "claude"))
            
            // Stale link should be pruned
            #expect(!FileManager.default.fileExists(atPath: danglingLink.path))
            
            // Active link should remain
            let activeLink = claudeDir.appendingPathComponent("active-skill")
            #expect(FileManager.default.fileExists(atPath: activeLink.path))
        }
    }
    
    @Test func testManifestYAMLStructure() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "yaml-skill",
                skillMdContent: "---\ndescription: \"YAML check\"\n---"
            )

            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))

            let manifest = canonicalStore.appendingPathComponent("manifest.yaml")
            let content = try String(contentsOf: manifest, encoding: .utf8)

            // Structural checks: required top-level keys must be present
            #expect(content.contains("generated_at:"))
            #expect(content.contains("canonical_root:"))
            #expect(content.contains("skills:"))
            #expect(content.contains("yaml-skill"))
        }
    }

    @Test func testSymlinkFanOutCreatesCorrectLinks() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            let skillDir = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "linked-skill",
                skillMdContent: "---\ndescription: \"Link target\"\n---"
            )

            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: "claude"))

            let link = claudeDir.appendingPathComponent("linked-skill")
            let dest = try FileManager.default.destinationOfSymbolicLink(atPath: link.path)
            // Symlink must resolve back to the canonical skill directory.
            // Normalize both sides through resolvingSymlinksInPath to absorb
            // macOS's /var → /private/var prefix (NSTemporaryDirectory crosses it).
            let resolvedDest = URL(fileURLWithPath: dest).resolvingSymlinksInPath().path
            let resolvedSkill = skillDir.resolvingSymlinksInPath().path
            #expect(resolvedDest == resolvedSkill)
        }
    }

    @Test func testExecuteDryRunPreservesExistingManifest() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "preserve-skill",
                skillMdContent: "---\ndescription: \"Preserve\"\n---"
            )

            let manifest = canonicalStore.appendingPathComponent("manifest.yaml")
            let sentinel = "# SENTINEL — must not be overwritten\nskills: []\n"
            try sentinel.write(to: manifest, atomically: true, encoding: .utf8)

            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.execute(options: SyncOptions(dryRun: true, doIndex: false, statusOnly: false, onlyAgent: nil))

            let content = try String(contentsOf: manifest, encoding: .utf8)
            #expect(content == sentinel)
        }
    }

    @Test func testExecuteDryRunDoesNotPruneStaleSymlinks() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "active-skill",
                skillMdContent: "---\ndescription: \"Active\"\n---"
            )

            let danglingLink = claudeDir.appendingPathComponent("stale-skill")
            try FileManager.default.createSymbolicLink(
                at: danglingLink,
                withDestinationURL: canonicalStore.appendingPathComponent("stale-skill")
            )

            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            try manager.execute(options: SyncOptions(dryRun: true, doIndex: false, statusOnly: false, onlyAgent: "claude"))

            // Dangling link must survive dry-run (prune is gated behind !dryRun && !statusOnly)
            let linkAttrs = try FileManager.default.attributesOfItem(atPath: danglingLink.path)
            #expect(linkAttrs[.type] as? FileAttributeType == .typeSymbolicLink)
        }
    }

    @Test func testNonSymlinkConflictLogsAndContinues() throws {
        try withSandbox { canonicalStore, claudeDir, cursorDir in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "linkable-skill",
                skillMdContent: "---\ndescription: \"Linkable\"\n---"
            )
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "blocked-skill",
                skillMdContent: "---\ndescription: \"Blocked\"\n---"
            )
            // Pre-existing non-symlink directory at the conflict location
            let blockingDir = claudeDir.appendingPathComponent("blocked-skill")
            try FileManager.default.createDirectory(at: blockingDir, withIntermediateDirectories: true)

            var logs: [String] = []
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { logs.append($0) }

            // Soft-fail contract: must not throw
            #expect(throws: Never.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: "claude"))
            }

            // Non-symlink dir is preserved untouched
            let blockedAttrs = try FileManager.default.attributesOfItem(atPath: blockingDir.path)
            #expect(blockedAttrs[.type] as? FileAttributeType == .typeDirectory)

            // Sibling skill still got linked (continue, not abort)
            let linkable = claudeDir.appendingPathComponent("linkable-skill")
            #expect(FileManager.default.fileExists(atPath: linkable.path))

            // Conflict surfaced in logs for operator visibility
            #expect(logs.contains(where: { $0.contains("blocked-skill") && $0.contains("skipping") }))
        }
    }

    // MARK: - T-04 forbidden-tuple regression tests

    @Test func testForbiddenTuple_anti_execute_throws() throws {
        try withSandbox { canonicalStore, _, _ in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "bad-anti-execute",
                skillMdContent: "---\ndescription: \"Bad\"\nasc: [anti, execute]\n---"
            )
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: SkillsError.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
            }
        }
    }

    @Test func testForbiddenTuple_negative_promotion_throws() throws {
        try withSandbox { canonicalStore, _, _ in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "bad-negative-promotion",
                skillMdContent: "---\ndescription: \"Bad\"\nasc: [negative, promotion]\n---"
            )
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: SkillsError.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
            }
        }
    }

    @Test func testForbiddenTuple_retired_execute_throws() throws {
        try withSandbox { canonicalStore, _, _ in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "bad-retired-execute",
                skillMdContent: "---\ndescription: \"Bad\"\nlifecycle: retired\nasc: [execute]\n---"
            )
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: SkillsError.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
            }
        }
    }

    @Test func testForbiddenTuple_safeSkill_doesNotThrow() throws {
        try withSandbox { canonicalStore, _, _ in
            _ = TestFixtures.createSkillDirectory(
                in: canonicalStore, name: "safe-skill",
                skillMdContent: "---\ndescription: \"Safe\"\nlifecycle: active\nasc: [execute]\nversion: \"1.0.0\"\ntier: stable\n---"
            )
            let manager = SyncManager(skillsRootPath: canonicalStore.path) { _ in }
            #expect(throws: Never.self) {
                try manager.execute(options: SyncOptions(dryRun: false, doIndex: false, statusOnly: false, onlyAgent: nil))
            }
        }
    }

    // MARK: - T-02 Skill struct extension regression tests

    @Test func testSkillParsesVersionTierOntologyRef() throws {
        let tempDir = TestFixtures.createTempDirectory()
        defer { TestFixtures.removeTempDirectory(tempDir) }
        _ = TestFixtures.createSkillDirectory(
            in: tempDir, name: "versioned-skill",
            skillMdContent: "---\ndescription: \"Versioned\"\nversion: \"2.1.0\"\ntier: beta\nontology_ref: \"urn:ckodex:ontology:core:1.0\"\n---"
        )
        let skill = Skill(url: tempDir.appendingPathComponent("versioned-skill"))
        #expect(skill.version == "2.1.0")
        #expect(skill.tier == "beta")
        #expect(skill.ontologyRef == "urn:ckodex:ontology:core:1.0")
        #expect(skill.contentHash?.hasPrefix("sha256:") == true)
        #expect(skill.contentHash?.count == 71)
    }
}
