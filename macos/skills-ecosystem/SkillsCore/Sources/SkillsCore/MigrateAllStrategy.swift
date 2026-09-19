import Foundation

/// Outcome of migrating a single physical skill to the canonical store.
public enum MigrateOutcome: Equatable {
    /// Skill was copied into the canonical store and original replaced with symlink.
    case migrated
    /// Physical copy removed and replaced with a symlink to the existing canonical version.
    case replacedWithLink
    /// Source was already a symlink — nothing to do.
    case skippedAlreadyLinked
    /// Source path or name matched an IP boundary pattern — refused.
    case skippedIPViolation(String)
}

/// Result of a single skill migration attempt.
public struct MigrateResult {
    public let agentName: String
    public let skillName: String
    public let sourcePath: String
    public let outcome: MigrateOutcome
    
    public init(agentName: String, skillName: String, sourcePath: String, outcome: MigrateOutcome) {
        self.agentName = agentName
        self.skillName = skillName
        self.sourcePath = sourcePath
        self.outcome = outcome
    }
}

/// Orchestrates batch migration of all physical (non-symlinked) skills across
/// agent directories into the canonical store, then symlinks them back.
public struct MigrateAllStrategy {
    private let fileManager = FileManager.default
    private let skillsRoot: URL
    private let logger: (String) -> Void
    
    public init(skillsRoot: URL, logger: @escaping (String) -> Void) {
        self.skillsRoot = skillsRoot
        self.logger = logger
    }
    
    /// Migrate a single physical skill directory into the canonical store.
    ///
    /// Three codepaths:
    /// 1. Source is already a symlink → skip.
    /// 2. Canonical already has the skill → remove the physical copy, symlink back.
    /// 3. Canonical does not have the skill → copy in, remove original, symlink back.
    public func migrateSkill(from sourceURL: URL, agentName: String) -> MigrateResult {
        let skillName = sourceURL.lastPathComponent
        
        // Already a symlink — nothing to do
        if let attrs = try? fileManager.attributesOfItem(atPath: sourceURL.path),
           attrs[.type] as? FileAttributeType == .typeSymbolicLink {
            logger("  · \(agentName)/\(skillName) is already a symbolic link. Skipping.")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedAlreadyLinked)
        }
        
        // IP boundary check
        do {
            try IPGuard.guardPath(sourceURL)
            try IPGuard.guardSkillName(skillName)
        } catch let IPGuardError.boundaryViolation(msg) {
            logger("  ✗ \(agentName)/\(skillName) blocked by IPGuard: \(msg)")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedIPViolation(msg))
        } catch {
            logger("  ✗ \(agentName)/\(skillName) unexpected error: \(error.localizedDescription)")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedIPViolation(error.localizedDescription))
        }
        
        let targetURL = skillsRoot.appendingPathComponent(skillName)
        
        // Case 2: canonical already has this skill — replace physical with symlink
        if fileManager.fileExists(atPath: targetURL.path) {
            do {
                try fileManager.removeItem(at: sourceURL)
                try fileManager.createSymbolicLink(at: sourceURL, withDestinationURL: targetURL)
                logger("  ✓ \(agentName)/\(skillName) replaced physical copy with symlink → \(targetURL.path)")
                return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .replacedWithLink)
            } catch {
                logger("  ✗ \(agentName)/\(skillName) failed to replace: \(error.localizedDescription)")
                return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedIPViolation(error.localizedDescription))
            }
        }
        
        // Case 3: new skill — copy to canonical, remove original, symlink back
        do {
            // Scan contents for IP violations before copying
            try IPGuard.scanForThales(in: sourceURL)
            
            try fileManager.copyItem(at: sourceURL, to: targetURL)
            logger("  ✓ Copied \(sourceURL.path) to \(targetURL.path)")
            
            // Ensure SKILL.md exists
            let skillMdURL = targetURL.appendingPathComponent("SKILL.md")
            if !fileManager.fileExists(atPath: skillMdURL.path) {
                let defaultSkillTemplate = """
                ---
                description: "Migrated skill for \(skillName)."
                ---
                # \(skillName)
                
                Migrated from \(sourceURL.path) on \(ISO8601DateFormatter().string(from: Date())).
                """
                try defaultSkillTemplate.write(to: skillMdURL, atomically: true, encoding: .utf8)
                logger("  ✓ Created default SKILL.md in canonical store")
            }
            
            try fileManager.removeItem(at: sourceURL)
            try fileManager.createSymbolicLink(at: sourceURL, withDestinationURL: targetURL)
            logger("  ✓ \(agentName)/\(skillName) migrated and symlinked → \(targetURL.path)")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .migrated)
        } catch let IPGuardError.boundaryViolation(msg) {
            logger("  ✗ \(agentName)/\(skillName) IP violation in contents: \(msg)")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedIPViolation(msg))
        } catch {
            logger("  ✗ \(agentName)/\(skillName) migration failed: \(error.localizedDescription)")
            return MigrateResult(agentName: agentName, skillName: skillName, sourcePath: sourceURL.path, outcome: .skippedIPViolation(error.localizedDescription))
        }
    }
    
    /// Discover and migrate all physical skills across all agent directories.
    public func migrateAll() -> [MigrateResult] {
        let discovery = SkillDiscovery()
        
        guard let physicalSkills = try? discovery.scanForPhysicalSkills() else {
            logger("  ✗ Failed to scan for physical skills")
            return []
        }
        
        if physicalSkills.isEmpty {
            logger("\nNo physical skills found to migrate!")
            return []
        }
        
        logger("\nFound \(physicalSkills.count) physical skills to migrate. Starting batch migration...")
        
        var results: [MigrateResult] = []
        for skill in physicalSkills {
            logger("\n>>> Centralizing '\(skill.skillName)' from \(skill.agentName)...")
            let sourceURL = URL(fileURLWithPath: skill.path)
            let result = migrateSkill(from: sourceURL, agentName: skill.agentName)
            results.append(result)
        }
        
        return results
    }
    
    /// Print a summary table of migration results.
    public static func printSummary(_ results: [MigrateResult], logger: (String) -> Void) {
        if results.isEmpty {
            logger("\nNothing to migrate — all skills are already centralized.")
            return
        }
        
        let migrated = results.filter { $0.outcome == .migrated }
        let replaced = results.filter { $0.outcome == .replacedWithLink }
        let skippedLinked = results.filter { $0.outcome == .skippedAlreadyLinked }
        let skippedIP = results.filter { if case .skippedIPViolation = $0.outcome { return true }; return false }
        
        logger("\n=== Migration Summary ===")
        logger("  Migrated (new)         : \(migrated.count)")
        logger("  Replaced with link     : \(replaced.count)")
        logger("  Skipped (already linked): \(skippedLinked.count)")
        logger("  Skipped (IP violation)  : \(skippedIP.count)")
        logger("  Total processed        : \(results.count)")
        
        if !skippedIP.isEmpty {
            logger("\n  IP-blocked skills:")
            for r in skippedIP {
                logger("    ✗ \(r.agentName)/\(r.skillName)")
            }
        }
    }
}
