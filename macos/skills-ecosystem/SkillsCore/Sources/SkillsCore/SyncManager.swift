import Foundation

public struct SyncOptions {
    public var dryRun: Bool
    public var doIndex: Bool
    public var statusOnly: Bool
    public var onlyAgent: String?
    
    public init(dryRun: Bool = false, doIndex: Bool = true, statusOnly: Bool = false, onlyAgent: String? = nil) {
        self.dryRun = dryRun
        self.doIndex = doIndex
        self.statusOnly = statusOnly
        self.onlyAgent = onlyAgent
    }
}

public class SyncManager {
    public let skillsRoot: URL
    private let fileManager = FileManager.default
    private var logger: (String) -> Void
    
    public init(skillsRootPath: String? = nil, logger: @escaping (String) -> Void = { print($0) }) {
        let path = skillsRootPath ?? ProcessInfo.processInfo.environment["SKILLS_ROOT"] ?? "~/Skills/shared"
        self.skillsRoot = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
        self.logger = logger
    }
    
    public func execute(options: SyncOptions) throws {
        logger("\n=== validating canonical store: \(skillsRoot.path) ===")
        try IPGuard.guardPath(skillsRoot)
        
        var isDir: ObjCBool = false
        if !fileManager.fileExists(atPath: skillsRoot.path, isDirectory: &isDir) || !isDir.boolValue {
            logger("  ✗ \(skillsRoot.path) does not exist. Create it and copy your skills in first.")
            throw SkillsError.rootDirectoryMissing(path: skillsRoot.path)
        }
        try IPGuard.scanForThales(in: skillsRoot)
        
        let backend = ProcessInfo.processInfo.environment["STORAGE_BACKEND"] ?? "local"
        if backend.lowercased() == "distributed" {
            logger("\n=== synchronizing from distributed storage to local cache ===")
            let provider = DistributedSkillStorageProvider()
            let distributedSkills = try provider.listSkills()
            logger("  ✓ found \(distributedSkills.count) skills in distributed metadata index")
            
            var distributedSkillNames = Set<String>()
            for ds in distributedSkills {
                distributedSkillNames.insert(ds.name)
                let skillContent = try provider.getSkillContent(named: ds.name)
                
                let localSkillDir = skillsRoot.appendingPathComponent(ds.name)
                try fileManager.createDirectory(at: localSkillDir, withIntermediateDirectories: true, attributes: nil)
                let localSkillMdURL = localSkillDir.appendingPathComponent("SKILL.md")
                
                let currentContent = try? String(contentsOf: localSkillMdURL, encoding: .utf8)
                if currentContent != skillContent {
                    try skillContent.write(to: localSkillMdURL, atomically: true, encoding: .utf8)
                    logger("    ↓ synced \(ds.name)")
                } else {
                    logger("    · \(ds.name) is up-to-date")
                }
            }
            
            // Prune local active folders not in FDB
            let localDirs = try fileManager.contentsOfDirectory(at: skillsRoot, includingPropertiesForKeys: [.isDirectoryKey], options: [.skipsHiddenFiles])
            for localDir in localDirs {
                let name = localDir.lastPathComponent
                if name == "candidates" || name == "manifest.yaml" || name == ".codegraph" {
                    continue
                }
                var isItemDir: ObjCBool = false
                if fileManager.fileExists(atPath: localDir.path, isDirectory: &isItemDir), isItemDir.boolValue {
                    if !distributedSkillNames.contains(name) {
                        try fileManager.removeItem(at: localDir)
                        logger("    ✗ pruned local stale cache: \(name)")
                    }
                }
            }
        }
        
        let discovery = SkillDiscovery()
        let skills = try discovery.discoverSkills(in: skillsRoot)
        if skills.isEmpty {
            logger("  ✗ No skills found under \(skillsRoot.path)")
            throw SkillsError.noSkillsFound(path: skillsRoot.path)
        }
        
        logger("  ✓ found \(skills.count) skills")
        for skill in skills {
            try IPGuard.guardSkillName(skill.name)
            try Self.checkForbiddenTuples(skill: skill)
            logger("    \(skill.name)")
        }
        
        logger("\n=== regenerating manifest.yaml ===")
        let manifestURL = skillsRoot.appendingPathComponent("manifest.yaml")
        let bundleJsonURL = skillsRoot.appendingPathComponent("bundle.json")
        if options.dryRun || options.statusOnly {
            logger("  · would write \(manifestURL.path)")
            logger("  · would write \(bundleJsonURL.path)")
        } else {
            let manifestGen = ManifestGenerator(skillsRoot: skillsRoot)
            try manifestGen.generateManifest(skills: skills, outputURL: manifestURL)
            logger("  ✓ wrote \(manifestURL.path)")
            try manifestGen.generateBundleJson(skills: skills, manifestURL: manifestURL, outputURL: bundleJsonURL)
            logger("  ✓ wrote \(bundleJsonURL.path)")
        }
        
        let syncer = AgentSyncer(skillsRoot: skillsRoot, logger: logger)
        let activeAgents = AgentRegistry.shared.agents.filter { 
            options.onlyAgent == nil || $0.name.lowercased() == options.onlyAgent?.lowercased()
        }
        
        for agent in activeAgents {
            guard let resolved = agent.resolvedURL else {
                logger("\n=== \(agent.name) → no resolved directory (skipping) ===")
                continue
            }
            logger("\n=== \(agent.name) → \(resolved.path) ===")
            try syncer.sync(agent: agent, skills: skills, options: options)
        }
        
        if options.doIndex && !options.statusOnly {
            logger("\n=== refreshing CodeGraph index over canonical store ===")
            let runner = CodeGraphRunner(logger: logger)
            runner.runCodeGraph(at: skillsRoot)
        } else if !options.doIndex {
            logger("  · --no-index: skipping CodeGraph refresh")
        }
        
        logger("\n=== summary ===")
        logger("Canonical store     : \(skillsRoot.path)  (\(skills.count) skills)")
        logger("Manifest            : \(manifestURL.path)")
        if options.statusOnly || options.dryRun {
            let mode = options.statusOnly ? "status" : "dry-run"
            logger("\n(no changes applied — \(mode) mode)")
        }
    }
    
    public func importSkill(from sourcePath: String) throws {
        let importer = SkillImporter(skillsRoot: skillsRoot, logger: logger)
        try importer.importSkill(from: sourcePath)
        
        let backend = ProcessInfo.processInfo.environment["STORAGE_BACKEND"] ?? "local"
        if backend.lowercased() == "distributed" {
            let skillName = URL(fileURLWithPath: sourcePath).lastPathComponent
            let provider = DistributedSkillStorageProvider()
            let skillDir = skillsRoot.appendingPathComponent(skillName)
            let skillMdURL = skillDir.appendingPathComponent("SKILL.md")
            
            if fileManager.fileExists(atPath: skillMdURL.path) {
                let content = try String(contentsOf: skillMdURL, encoding: .utf8)
                let skill = Skill(url: skillDir)
                let metadata = SkillMetadata(name: skill.name, sizeBytes: skill.sizeBytes, description: skill.description)
                
                try provider.storeSkill(named: skillName, metadata: metadata, content: content)
                logger("  ✓ Uploaded imported skill '\(skillName)' to distributed storage.")
            }
        }
    }
    
    public func scanForPhysicalSkills() throws -> [DiscoveredPhysicalSkill] {
        let discovery = SkillDiscovery()
        return try discovery.scanForPhysicalSkills()
    }
    
    public func migrateAll() throws -> [MigrateResult] {
        let strategy = MigrateAllStrategy(skillsRoot: skillsRoot, logger: logger)
        let results = strategy.migrateAll()
        
        let migrated = results.filter { $0.outcome == .migrated || $0.outcome == .replacedWithLink }
        if !migrated.isEmpty {
            logger("\nBatch migration complete! Running global sync to refresh all agent link graphs...")
            try execute(options: SyncOptions(dryRun: false, doIndex: true, statusOnly: false, onlyAgent: nil))
        }
        
        MigrateAllStrategy.printSummary(results, logger: logger)
        return results
    }

    /// T-04: CLAUDE.md §10 forbidden-tuple checks.
    /// Three combinations are blocked regardless of other settings:
    ///   1. anti ∧ execute   — an ASC-anti skill cannot have execute in ASC
    ///   2. negative ∧ promotion — negative-valence + promotion intent not allowed
    ///   3. retired ∧ execute — a retired skill must not be re-executed
    static func checkForbiddenTuples(skill: Skill) throws {
        let asc = skill.asc.map { $0.lowercased() }

        // Rule 1: anti ∧ execute
        if asc.contains("anti") && asc.contains("execute") {
            throw SkillsError.forbiddenTupleViolation(
                skillName: skill.name,
                reason: "asc contains both 'anti' and 'execute' (anti ∧ execute forbidden)"
            )
        }

        // Rule 2: negative ∧ promotion
        if asc.contains("negative") && asc.contains("promotion") {
            throw SkillsError.forbiddenTupleViolation(
                skillName: skill.name,
                reason: "asc contains both 'negative' and 'promotion' (negative ∧ promotion forbidden)"
            )
        }

        // Rule 3: retired ∧ execute
        if skill.lifecycleStatus == .retired && asc.contains("execute") {
            throw SkillsError.forbiddenTupleViolation(
                skillName: skill.name,
                reason: "lifecycle is 'retired' but asc contains 'execute' (retired ∧ execute forbidden)"
            )
        }
    }

    public func craftSkill(from path: String, apiKey: String) throws -> String {
        let crafter = SkillCrafter(skillsRoot: skillsRoot, apiKey: apiKey, logger: logger)
        return try crafter.craftSkill(from: path)
    }

    public func promoteSkill(named candidateName: String) throws {
        let name = candidateName.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        try IPGuard.guardSkillName(name)
        
        let candidatesURL = skillsRoot.appendingPathComponent("candidates")
        let candidateSkillURL = candidatesURL.appendingPathComponent(name)
        let targetURL = skillsRoot.appendingPathComponent(name)
        
        var isDir: ObjCBool = false
        guard fileManager.fileExists(atPath: candidateSkillURL.path, isDirectory: &isDir), isDir.boolValue else {
            logger("  ✗ Candidate skill '\(name)' does not exist at \(candidateSkillURL.path)")
            throw SkillsError.invalidSourceDirectory(path: candidateSkillURL.path)
        }

        // T-04: retired ∧ promotion forbidden
        let candidateSkill = Skill(url: candidateSkillURL)
        if candidateSkill.lifecycleStatus == .retired {
            throw SkillsError.forbiddenTupleViolation(
                skillName: name,
                reason: "lifecycle is 'retired'; promoting a retired skill is forbidden (retired ∧ promotion)"
            )
        }
        
        if fileManager.fileExists(atPath: targetURL.path) {
            logger("  ✗ Skill '\(name)' already exists in active canonical store at \(targetURL.path)")
            throw SkillsError.skillAlreadyExists(name: name, path: targetURL.path)
        }
        
        logger("\n=== promoting candidate skill \(name) to active store ===")
        
        let backend = ProcessInfo.processInfo.environment["STORAGE_BACKEND"] ?? "local"
        if backend.lowercased() == "distributed" {
            logger("  · [Storage] Active backend: distributed. Uploading to FoundationDB/RustFS...")
            let candidateSkill = Skill(url: candidateSkillURL)
            let metadata = SkillMetadata(name: name, sizeBytes: candidateSkill.sizeBytes, description: candidateSkill.description)
            
            let skillMdURL = candidateSkillURL.appendingPathComponent("SKILL.md")
            let content = try String(contentsOf: skillMdURL, encoding: .utf8)
            
            let provider = DistributedSkillStorageProvider()
            try provider.storeSkill(named: name, metadata: metadata, content: content)
            logger("  ✓ Saved skill '\(name)' and its metadata to FoundationDB/RustFS.")
            
            try fileManager.removeItem(at: candidateSkillURL)
            logger("  ✓ Removed local candidate folder.")
        } else {
            // Move candidate to active canonical store
            try fileManager.moveItem(at: candidateSkillURL, to: targetURL)
            logger("  ✓ Moved candidate skill directory to \(targetURL.path)")
        }
        
        // Trigger a sync execution to fan out the promoted skill to all agents!
        logger("  ✓ Triggering sync sweep to distribute newly active skill...")
        try execute(options: SyncOptions(dryRun: false, doIndex: true, statusOnly: false, onlyAgent: nil))

        // T-08: publish LifecycleDecisionBundle
        let promotedSkill = Skill(url: targetURL)
        let bundle = LifecycleDecisionBundle(
            skillName: name,
            decision: .promoted,
            newStatus: SkillLifecycleStatus.active.rawValue,
            previousStatus: promotedSkill.lifecycleStatus.rawValue,
            actorUrn: "urn:ckodex:actor:cli:skills-cli"
        )
        LifecyclePublisher(skillsRoot: skillsRoot, logger: logger).publish(bundle)

        logger("  ✓ Successfully promoted skill '\(name)'!")
    }
}
