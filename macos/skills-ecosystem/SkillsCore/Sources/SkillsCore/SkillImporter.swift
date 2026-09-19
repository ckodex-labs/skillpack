import Foundation

public struct SkillImporter {
    private let fileManager = FileManager.default
    private let skillsRoot: URL
    private let logger: (String) -> Void
    
    public init(skillsRoot: URL, logger: @escaping (String) -> Void) {
        self.skillsRoot = skillsRoot
        self.logger = logger
    }
    
    public func importSkill(from sourcePath: String) throws {
        let sourceURL = URL(fileURLWithPath: (sourcePath as NSString).expandingTildeInPath)
        
        // Check if already a symlink
        if let attrs = try? fileManager.attributesOfItem(atPath: sourceURL.path),
           attrs[.type] as? FileAttributeType == .typeSymbolicLink {
            logger("  · '\(sourceURL.lastPathComponent)' is already a symbolic link. Skipping import.")
            return
        }
        
        // 1. Validate source exists
        var isDir: ObjCBool = false
        guard fileManager.fileExists(atPath: sourceURL.path, isDirectory: &isDir), isDir.boolValue else {
            logger("  ✗ Source path \(sourceURL.path) does not exist or is not a directory.")
            throw SkillsError.invalidSourceDirectory(path: sourceURL.path)
        }
        
        // 2. Run IPGuard checks
        try IPGuard.guardPath(sourceURL)
        try IPGuard.guardSkillName(sourceURL.lastPathComponent)
        try IPGuard.scanForThales(in: sourceURL)
        
        // 3. Compute target canonical URL
        let skillName = sourceURL.lastPathComponent
        let targetURL = skillsRoot.appendingPathComponent(skillName)
        
        if fileManager.fileExists(atPath: targetURL.path) {
            // Canonical already has this skill — replace physical copy with symlink
            logger("  · Skill '\(skillName)' already exists in canonical store at \(targetURL.path). Replacing source with symlink.")
            try fileManager.removeItem(at: sourceURL)
            try fileManager.createSymbolicLink(at: sourceURL, withDestinationURL: targetURL)
            logger("  ✓ Replaced physical copy with symlink: \(sourceURL.path) → \(targetURL.path)")
            return
        }
        
        logger("\n=== importing \(skillName) to canonical store ===")
        
        // 4. Copy to canonical
        try fileManager.copyItem(at: sourceURL, to: targetURL)
        logger("  ✓ Copied \(sourceURL.path) to \(targetURL.path)")
        
        // 5. Ensure SKILL.md exists
        let skillMdURL = targetURL.appendingPathComponent("SKILL.md")
        if !fileManager.fileExists(atPath: skillMdURL.path) {
            let defaultSkillTemplate = """
            ---
            description: "Imported skill for \(skillName)."
            ---
            # \(skillName)
            
            Imported from \(sourceURL.path) on \(ISO8601DateFormatter().string(from: Date())).
            """
            try defaultSkillTemplate.write(to: skillMdURL, atomically: true, encoding: .utf8)
            logger("  ✓ Created default SKILL.md in canonical store")
        }
        
        // 6. Remove original physical folder
        try fileManager.removeItem(at: sourceURL)
        logger("  ✓ Removed original physical folder at \(sourceURL.path)")
        
        // 7. Symlink back to original path
        try fileManager.createSymbolicLink(at: sourceURL, withDestinationURL: targetURL)
        logger("  ✓ Created symlink back: \(sourceURL.path) -> \(targetURL.path)")
        
        logger("  ✓ Successfully imported/migrated \(skillName)!")
    }
}
