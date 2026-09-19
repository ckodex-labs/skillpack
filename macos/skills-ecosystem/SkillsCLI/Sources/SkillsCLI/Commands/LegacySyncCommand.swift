import Foundation
import ArgumentParser
import SkillsCore

/// Backward-compatible legacy subcommand wrapping the old canonical-store sync operations.
struct LegacySyncCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "sync",
        abstract: "Synchronize canonical skill store to all agents (legacy)"
    )

    @Flag(name: .long, help: "Show what would change without applying")
    var dryRun: Bool = false

    @Flag(name: .long, inversion: .prefixedNo, help: "Skip CodeGraph re-index")
    var index: Bool = true

    @Flag(name: .long, help: "Report current state, no changes")
    var status: Bool = false

    @Option(name: .customLong("only"), help: "Fan out to one agent only")
    var onlyAgent: String?

    @Option(name: .customLong("import"), help: "Import a skill folder into the canonical store")
    var importPath: String?

    @Option(name: .customLong("promote"), help: "Promote a candidate skill to active status")
    var promoteName: String?

    @Flag(name: .customLong("inventory"), help: "Print inventory of physical (non-symlinked) skills")
    var inventory: Bool = false

    @Flag(name: .customLong("migrate-all"), help: "Migrate all physical skills into the canonical store")
    var migrateAll: Bool = false

    mutating func run() throws {
        let skillsRootPath = ProcessInfo.processInfo.environment["SKILLS_ROOT"] ?? "~/Skills/shared"
        let manager = SyncManager(skillsRootPath: skillsRootPath)

        if inventory {
            let physicalSkills = try manager.scanForPhysicalSkills()
            if physicalSkills.isEmpty {
                print("\nNo physical skills found — everything is symlinked. (Clean state)")
                return
            }
            print("\nPhysical skills in agent folders (not symlinked):")
            print(String(repeating: "─", count: 80))
            for skill in physicalSkills {
                let sizeStr = String(format: "%.1f KB", Double(skill.sizeBytes) / 1024.0)
                let displayPath = skill.path.replacingOccurrences(of: NSHomeDirectory(), with: "~")
                print(String(format: "  %-12s  %-24s  %-10s  %s",
                             skill.agentName, skill.skillName, sizeStr, displayPath))
            }
            print(String(repeating: "─", count: 80))
            return
        }

        if migrateAll {
            _ = try manager.migrateAll()
            return
        }

        if let path = importPath {
            try manager.importSkill(from: path)
            return
        }

        if let name = promoteName {
            try manager.promoteSkill(named: name)
            return
        }

        let opts = SyncOptions(dryRun: dryRun, doIndex: index, statusOnly: status, onlyAgent: onlyAgent)
        try manager.execute(options: opts)
    }
}
