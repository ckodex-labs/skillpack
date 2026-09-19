import Foundation
import ArgumentParser
import SkillsCore

struct InspectCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "inspect",
        abstract: "Show detailed info for a skill"
    )

    @Argument(help: "Skill name or dirName")
    var skillName: String

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let scanner = SkillScanner()
        let all = scanner.scan(options: ScanOptions(scope: opts.scope, providerFilter: opts.tool.isEmpty ? nil : opts.tool, includeDisabled: true))
        let target = skillName.lowercased()
        let matches = all.filter { $0.name.lowercased() == target || $0.dirName.lowercased() == target }

        if matches.isEmpty {
            fputs("error: skill '\(skillName)' not found\n", stderr)
            throw ExitCode.failure
        }

        let fmt = opts.formatter
        if opts.json || opts.machine {
            let output = try fmt.toJSON(matches)
            print(opts.machine ? machineEnvelope(data: output, command: "inspect") : output)
        } else {
            for s in matches {
                print(fmt.formatSkillInspect(s))
            }
        }
    }
}
