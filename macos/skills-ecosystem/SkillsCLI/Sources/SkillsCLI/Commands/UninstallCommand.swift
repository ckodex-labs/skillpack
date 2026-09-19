import Foundation
import ArgumentParser
import SkillsCore

struct UninstallCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "uninstall",
        abstract: "Remove a skill (with confirmation)"
    )

    @Argument(help: "Skill name or dirName to remove")
    var skillName: String

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Remove from specific provider only")
    var tool: String = ""

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompt")
    var yes: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let scanner = SkillScanner()
        let opts = ScanOptions(scope: "both", providerFilter: tool.isEmpty ? nil : tool, includeDisabled: true)
        let all = scanner.scan(options: opts)
        let target = skillName.lowercased()
        let matches = all.filter { $0.name.lowercased() == target || $0.dirName.lowercased() == target }

        if matches.isEmpty {
            fputs("error: skill '\(skillName)' not found\n", stderr)
            throw ExitCode.failure
        }

        let fmt = OutputFormatter(noColor: noColor)

        print("The following instances will be removed:")
        for m in matches {
            print("  • [\(m.provider)/\(m.scope)]  \(fmt.shorten(m.path))")
        }

        if !yes {
            print("\nContinue? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else {
                print("Aborted.")
                return
            }
        }

        let fm = FileManager.default
        for m in matches {
            let url = URL(fileURLWithPath: m.path)
            do {
                try fm.removeItem(at: url)
                print("  ✓ Removed \(fmt.shorten(m.path))")
            } catch {
                fputs("  ✗ Failed to remove \(m.path): \(error.localizedDescription)\n", stderr)
            }
        }
        print("Done.")
    }
}
