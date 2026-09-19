import Foundation

public protocol AgentSyncStrategy {
    func sync(agent: AgentConfig, skills: [Skill], options: SyncOptions, skillsRoot: URL, logger: (String) -> Void) throws
}

public struct SymlinkSyncStrategy: AgentSyncStrategy {
    private let fileManager = FileManager.default
    
    public init() {}
    
    public func sync(agent: AgentConfig, skills: [Skill], options: SyncOptions, skillsRoot: URL, logger: (String) -> Void) throws {
        guard let targetDir = agent.resolvedURL else {
            logger("  · \(agent.name) has no resolved directory (skipping)")
            return
        }
        
        if !fileManager.fileExists(atPath: targetDir.path) {
            if options.dryRun || options.statusOnly {
                logger("  · \(agent.name) dir not found (skipping or would create)")
                return
            } else {
                try? fileManager.createDirectory(at: targetDir, withIntermediateDirectories: true, attributes: nil)
            }
        }
        
        if !fileManager.fileExists(atPath: targetDir.path) {
            logger("  · \(agent.name) dir not creatable: \(targetDir.path) (skipping)")
            return
        }
        
        // Pass 1: ensure links exist
        for skill in skills {
            let linkURL = targetDir.appendingPathComponent(skill.name)
            
            if fileManager.fileExists(atPath: linkURL.path) {
                // Check if symlink
                let attrs = try fileManager.attributesOfItem(atPath: linkURL.path)
                if attrs[.type] as? FileAttributeType == .typeSymbolicLink {
                    let dest = try fileManager.destinationOfSymbolicLink(atPath: linkURL.path)
                    if dest == skill.url.path {
                        logger("  · \(agent.name)/\(skill.name) already correct")
                        continue
                    } else {
                        if !options.dryRun && !options.statusOnly {
                            try fileManager.removeItem(at: linkURL)
                            try fileManager.createSymbolicLink(at: linkURL, withDestinationURL: skill.url)
                        }
                        logger("  ✓ \(agent.name)/\(skill.name) relinked → \(skill.url.path)")
                    }
                } else {
                    logger("  ⚠ \(agent.name)/\(skill.name) exists and is NOT a symlink — skipping (manual merge required)")
                    continue
                }
            } else {
                if !options.dryRun && !options.statusOnly {
                    try fileManager.createSymbolicLink(at: linkURL, withDestinationURL: skill.url)
                }
                logger("  ✓ \(agent.name)/\(skill.name) linked → \(skill.url.path)")
            }
        }
        
        // Pass 2: Prune dangling
        let existingItems = try fileManager.contentsOfDirectory(at: targetDir, includingPropertiesForKeys: nil, options: [.skipsHiddenFiles])
        for item in existingItems {
            let attrs = try fileManager.attributesOfItem(atPath: item.path)
            if attrs[.type] as? FileAttributeType == .typeSymbolicLink {
                let dest = try fileManager.destinationOfSymbolicLink(atPath: item.path)
                if !fileManager.fileExists(atPath: dest) {
                    if !options.dryRun && !options.statusOnly {
                        try fileManager.removeItem(at: item)
                    }
                    logger("  ✓ \(agent.name)/\(item.lastPathComponent) pruned (canonical missing)")
                }
            }
        }
    }
}

public struct CursorIndexSyncStrategy: AgentSyncStrategy {
    private let fileManager = FileManager.default
    
    public init() {}
    
    public func sync(agent: AgentConfig, skills: [Skill], options: SyncOptions, skillsRoot: URL, logger: (String) -> Void) throws {
        guard let targetDir = agent.resolvedURL else {
            logger("  · \(agent.name) has no resolved directory (skipping)")
            return
        }
        
        if !fileManager.fileExists(atPath: targetDir.path) {
            if options.statusOnly {
                logger("  · no \(targetDir.path); \(agent.name) likely not configured (skipping)")
                return
            }
        }
        
        if !options.dryRun && !options.statusOnly {
            try? fileManager.createDirectory(at: targetDir, withIntermediateDirectories: true, attributes: nil)
        }
        
        let indexURL = targetDir.appendingPathComponent("skills-index.mdc")
        
        var content = """
        ---
        description: "Personal skill catalog (canonical store: \(skillsRoot.path)). Auto-generated, do not hand-edit."
        globs: ["**/*"]
        alwaysApply: false
        ---
        
        # Available Skills
        
        Canonical store: `\(skillsRoot.path)`
        
        Query via CodeGraph MCP (if configured) or read the SKILL.md directly at the path below.
        
        | Skill | Path | Description |
        |---|---|---|
        """
        content += "\n"
        
        for skill in skills {
            let safeDesc = skill.description.replacingOccurrences(of: "|", with: "\\|")
            content += "| `\(skill.name)` | `\(skill.url.path)` | \(safeDesc) |\n"
        }
        
        content += """
        \n## Boundary
        
        Skills under any path matching `/thales/i` or marked `IP-pending` are **not** indexed here. Refuse and surface any request to cross-reference them under personal scope.
        \n
        """
        
        if options.dryRun || options.statusOnly {
            logger("  · would write \(indexURL.path)")
        } else {
            try safeAtomicWrite(content: content, to: indexURL)
            logger("  ✓ wrote \(indexURL.path) with \(skills.count) entries")
        }
    }
    
    private func safeAtomicWrite(content: String, to url: URL) throws {
        if fileManager.fileExists(atPath: url.path) {
            let tempURL = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString)
            try content.write(to: tempURL, atomically: true, encoding: .utf8)
            _ = try fileManager.replaceItemAt(url, withItemAt: tempURL)
        } else {
            try content.write(to: url, atomically: true, encoding: .utf8)
        }
    }
}

public struct AgentSyncer {
    private let skillsRoot: URL
    private let logger: (String) -> Void
    
    public init(skillsRoot: URL, logger: @escaping (String) -> Void) {
        self.skillsRoot = skillsRoot
        self.logger = logger
    }
    
    public func sync(agent: AgentConfig, skills: [Skill], options: SyncOptions) throws {
        let strategy: AgentSyncStrategy
        switch agent.integrationType {
        case .symlink:
            strategy = SymlinkSyncStrategy()
        case .indexFile:
            strategy = CursorIndexSyncStrategy()
        }
        try strategy.sync(agent: agent, skills: skills, options: options, skillsRoot: skillsRoot, logger: logger)
    }
}
