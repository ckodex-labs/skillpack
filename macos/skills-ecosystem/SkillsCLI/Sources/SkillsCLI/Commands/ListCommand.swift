import Foundation
import ArgumentParser
import SkillsCore

struct ListCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "list",
        abstract: "List all discovered skills across all providers"
    )

    @OptionGroup var opts: SharedOptions
    @Option(name: .long, help: "Maximum number of skills to display")
    var limit: Int = 0

    func run() throws {
        let scanner = SkillScanner()
        var skills = scanner.scan(options: opts.scanOptions)

        if limit > 0 {
            skills = Array(skills.prefix(limit))
        }

        let fmt = opts.formatter
        if opts.json || opts.machine {
            let output = try fmt.toJSON(skills)
            if opts.machine {
                print(machineEnvelope(data: output, command: "list"))
            } else {
                print(output)
            }
        } else {
            print(fmt.formatSkillTable(skills, flat: opts.flat, sortBy: opts.sort))
        }
    }
}
