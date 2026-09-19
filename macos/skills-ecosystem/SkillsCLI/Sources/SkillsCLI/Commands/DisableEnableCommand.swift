import Foundation
import ArgumentParser
import SkillsCore

struct DisableCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "disable",
        abstract: "Disable skill(s) without uninstalling (renames SKILL.md → SKILL.md.disabled)"
    )

    @Argument(help: "Skill name, dirName, or 'all' to disable all")
    var target: String

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Filter to specific provider")
    var tool: String = ""

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let scanner = SkillScanner()
        let opts = ScanOptions(scope: "both", providerFilter: tool.isEmpty ? nil : tool, includeDisabled: false)
        let all = scanner.scan(options: opts)

        let matches: [SkillInfo]
        if target.lowercased() == "all" {
            matches = all
        } else {
            let t = target.lowercased()
            matches = all.filter { $0.name.lowercased() == t || $0.dirName.lowercased() == t }
        }

        if matches.isEmpty {
            fputs("No active skills found matching '\(target)'\n", stderr)
            throw ExitCode.failure
        }

        let mgr = SkillStateManager()
        let affected = try mgr.disable(skills: matches)
        for a in affected { print("  ✓ Disabled \(a)") }
    }
}

struct EnableCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "enable",
        abstract: "Re-enable disabled skill(s)"
    )

    @Argument(help: "Skill name, dirName, or 'all' to enable all disabled")
    var target: String

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Filter to specific provider")
    var tool: String = ""

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let scanner = SkillScanner()
        let opts = ScanOptions(scope: "both", providerFilter: tool.isEmpty ? nil : tool, includeDisabled: true)
        let all = scanner.scan(options: opts)

        let disabledSkills = all.filter { $0.disabled }
        let matches: [SkillInfo]
        if target.lowercased() == "all" {
            matches = disabledSkills
        } else {
            let t = target.lowercased()
            matches = disabledSkills.filter { $0.name.lowercased() == t || $0.dirName.lowercased() == t }
        }

        if matches.isEmpty {
            fputs("No disabled skills found matching '\(target)'\n", stderr)
            throw ExitCode.failure
        }

        let mgr = SkillStateManager()
        let affected = try mgr.enable(skills: matches)
        for a in affected { print("  ✓ Enabled \(a)") }
    }
}
