import Foundation
import ArgumentParser
import SkillsCore

struct StatsCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "stats",
        abstract: "Show aggregate skill metrics dashboard"
    )

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let scanner = SkillScanner()
        let skills = scanner.scan(options: opts.scanOptions)

        // Detect duplicates for stats
        let byDirName = Dictionary(grouping: skills, by: { $0.dirName })
        let dupCount = byDirName.values.filter { vals in
            Set(vals.map { $0.location }).count >= 2
        }.count

        let fmt = opts.formatter

        if opts.json || opts.machine {
            struct StatsResult: Codable {
                var total: Int
                var global: Int
                var project: Int
                var disabled: Int
                var duplicates: Int
                var totalTokens: Int
                var byProvider: [String: Int]
            }
            let byProvider = Dictionary(grouping: skills, by: { $0.provider }).mapValues { $0.count }
            let result = StatsResult(
                total: skills.count,
                global: skills.filter { $0.scope == "global" }.count,
                project: skills.filter { $0.scope == "project" }.count,
                disabled: skills.filter { $0.disabled }.count,
                duplicates: dupCount,
                totalTokens: skills.compactMap { $0.tokenCount }.reduce(0, +),
                byProvider: byProvider
            )
            let output = try fmt.toJSON(result)
            print(opts.machine ? machineEnvelope(data: output, command: "stats") : output)
        } else {
            print(fmt.formatStats(skills: skills, duplicateCount: dupCount))
        }
    }
}
