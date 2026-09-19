import Cocoa
import FinderSync
import SkillsCore

class FinderSync: FIFinderSync {

    override init() {
        super.init()
        
        NSLog("FinderSync() launched")
        
        // Default watch folder is home directory or skills root if defined
        let path = ProcessInfo.processInfo.environment["SKILLS_ROOT"] ?? "~/Skills/shared"
        let resolvedPath = (path as NSString).expandingTildeInPath
        let watchURL = URL(fileURLWithPath: resolvedPath)
        
        FIFinderSyncController.default().directoryURLs = [watchURL]
        
        // Set badges with correct forBadgeIdentifier signature
        FIFinderSyncController.default().setBadgeImage(NSImage(named: NSImage.statusAvailableName)!, label: "Safe Skill", forBadgeIdentifier: "0")
        FIFinderSyncController.default().setBadgeImage(NSImage(named: NSImage.statusUnavailableName)!, label: "IP Boundary Breach", forBadgeIdentifier: "1")
    }
    
    // MARK: - Primary Finder Sync API
    
    override func beginObservingDirectory(at url: URL) {
        NSLog("beginObservingDirectory: \(url.path)")
    }
    
    override func endObservingDirectory(at url: URL) {
        NSLog("endObservingDirectory: \(url.path)")
    }
    
    override func requestBadgeIdentifier(for url: URL) {
        // Run swift-native IP Guard verification to badge files!
        do {
            try IPGuard.guardPath(url)
            // If it's inside the skills store and contains SKILL.md, it's a verified safe skill
            let skillMd = url.appendingPathComponent("SKILL.md")
            if FileManager.default.fileExists(atPath: skillMd.path) {
                FIFinderSyncController.default().setBadgeIdentifier("0", for: url)
            }
        } catch {
            // Boundary violation
            FIFinderSyncController.default().setBadgeIdentifier("1", for: url)
        }
    }
    
    // MARK: - Context Menu Construction
    
    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        let menu = NSMenu(title: "")
        
        // We only append items if we are right-clicking files or inside a directory
        menu.addItem(withTitle: "Sync All Ckodex Skills", action: #selector(syncSkills(_:)), keyEquivalent: "")
        
        if menuKind == .contextualMenuForItems {
            if let selectedURLs = FIFinderSyncController.default().selectedItemURLs(),
               let url = selectedURLs.first,
               getPhysicalAgentSkill(for: url) != nil {
                menu.addItem(withTitle: "Migrate Skill to Canonical Store", action: #selector(migrateSelectedSkill(_:)), keyEquivalent: "")
            } else {
                menu.addItem(withTitle: "Promote Folder to Skill", action: #selector(promoteFolder(_:)), keyEquivalent: "")
            }
        }
        
        return menu
    }
    
    // MARK: - Helper Methods
    
    private func getPhysicalAgentSkill(for url: URL) -> (agentName: String, skillName: String)? {
        let path = url.path
        let fileManager = FileManager.default
        
        // Ensure it's a directory and not a symlink
        var isDir: ObjCBool = false
        guard fileManager.fileExists(atPath: path, isDirectory: &isDir), isDir.boolValue else {
            return nil
        }
        if let attrs = try? fileManager.attributesOfItem(atPath: path),
           attrs[.type] as? FileAttributeType == .typeSymbolicLink {
            return nil
        }
        
        // Check if it is inside any agent's directory
        for agent in AgentRegistry.shared.agents {
            guard agent.integrationType == .symlink else { continue }
            let agentPath = agent.resolvedURL.path
            
            // Check prefix. It must be inside the agent directory, but not the agent directory itself.
            if path.hasPrefix(agentPath) && path != agentPath {
                // The skill name is the path component right under the agent directory
                let relative = path.replacingOccurrences(of: agentPath + "/", with: "")
                let parts = relative.split(separator: "/")
                if let firstPart = parts.first {
                    let skillName = String(firstPart)
                    let skillURL = agent.resolvedURL.appendingPathComponent(skillName)
                    // Double check that the folder at skillURL is not a symlink
                    if let skillAttrs = try? fileManager.attributesOfItem(atPath: skillURL.path),
                       skillAttrs[.type] as? FileAttributeType != .typeSymbolicLink {
                        return (agent.name, skillName)
                    }
                }
            }
        }
        
        return nil
    }
    
    // MARK: - Context Actions
    
    @objc func syncSkills(_ sender: AnyObject?) {
        NSLog("FinderSync: syncSkills triggered")
        let manager = SyncManager()
        do {
            try manager.execute(options: SyncOptions(dryRun: false, doIndex: true, statusOnly: false, onlyAgent: nil))
            NSLog("FinderSync: Skills Sync Complete. Successfully synchronized canonical store to all agents.")
        } catch {
            NSLog("FinderSync: Skills Sync Failed: \(error.localizedDescription)")
        }
    }
    
    @objc func migrateSelectedSkill(_ sender: AnyObject?) {
        guard let targetURLs = FIFinderSyncController.default().selectedItemURLs(), let url = targetURLs.first else {
            return
        }
        
        guard let info = getPhysicalAgentSkill(for: url) else {
            NSLog("FinderSync: Selected folder is not a valid physical agent skill.")
            return
        }
        
        NSLog("FinderSync: Migrating physical skill '\(info.skillName)' from agent '\(info.agentName)'")
        
        let path = ProcessInfo.processInfo.environment["SKILLS_ROOT"] ?? "~/Skills/shared"
        let resolvedPath = (path as NSString).expandingTildeInPath
        let skillsRoot = URL(fileURLWithPath: resolvedPath)
        
        let strategy = MigrateAllStrategy(skillsRoot: skillsRoot) { log in
            NSLog("FinderSync [MigrateLog]: \(log)")
        }
        
        let result = strategy.migrateSkill(from: url, agentName: info.agentName)
        switch result.outcome {
        case .migrated, .replacedWithLink:
            NSLog("FinderSync: Migration successful! Triggering sync...")
            syncSkills(nil)
        case .skippedAlreadyLinked:
            NSLog("FinderSync: Skill is already linked.")
        case .skippedIPViolation(let msg):
            NSLog("FinderSync: Migration blocked by IPGuard: \(msg)")
        }
    }
    
    @objc func promoteFolder(_ sender: AnyObject?) {
        guard let targetURLs = FIFinderSyncController.default().selectedItemURLs(), let url = targetURLs.first else {
            return
        }
        
        // First check IP Guard
        do {
            try IPGuard.guardPath(url)
            try IPGuard.guardSkillName(url.lastPathComponent)
        } catch {
            NSLog("FinderSync: IP Boundary Blocked: \(error.localizedDescription)")
            return
        }
        
        let skillMdURL = url.appendingPathComponent("SKILL.md")
        if FileManager.default.fileExists(atPath: skillMdURL.path) {
            NSLog("FinderSync: Already a Skill. This folder already contains a SKILL.md.")
            return
        }
        
        let defaultSkillTemplate = """
        ---
        description: "Scaffolded skill for \(url.lastPathComponent)."
        ---
        # \(url.lastPathComponent)
        
        Add your capabilities, instructions, or workflow rules here.
        """
        
        do {
            try defaultSkillTemplate.write(to: skillMdURL, atomically: true, encoding: .utf8)
            NSLog("FinderSync: Promoted to Skill. Created SKILL.md in \(url.lastPathComponent). Triggering Sync...")
            syncSkills(nil)
        } catch {
            NSLog("FinderSync: Promotion Failed. Could not write SKILL.md: \(error.localizedDescription)")
        }
    }
}
