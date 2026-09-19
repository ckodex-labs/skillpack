import Foundation
import ArgumentParser
import SkillsCore

struct SearchCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "search",
        abstract: "Search skills by name, description, or tool"
    )

    @Argument(help: "Search query")
    var query: String

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let scanner = SkillScanner()
        let all = scanner.scan(options: opts.scanOptions)
        let q = query.lowercased()

        let matched = all.filter { s in
            s.name.lowercased().contains(q) ||
            s.description.lowercased().contains(q) ||
            s.dirName.lowercased().contains(q) ||
            s.allowedTools.contains(where: { $0.lowercased().contains(q) })
        }

        let fmt = opts.formatter
        if opts.json || opts.machine {
            let output = try fmt.toJSON(matched)
            if opts.machine {
                print(machineEnvelope(data: output, command: "search"))
            } else {
                print(output)
            }
        } else {
            print(fmt.formatSearchResults(matched, query: query))
        }
    }
}
