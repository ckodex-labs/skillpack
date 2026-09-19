import Foundation
import ArgumentParser
import SkillsCore

struct IndexCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "index",
        abstract: "Manage skill index",
        subcommands: [IndexIngestCommand.self, IndexSearchCommand.self, IndexListCommand.self],
        defaultSubcommand: IndexListCommand.self
    )
}

private let indexDir: URL = {
    let home = FileManager.default.homeDirectoryForCurrentUser
    return home.appendingPathComponent(".config/agent-skill-manager/skill-index")
}()

struct SkillIndexEntry: Codable {
    var dirName: String
    var name: String
    var description: String
    var provider: String
    var version: String
    var path: String
    var tokenCount: Int?
    var ingestedAt: String
}

struct IndexIngestCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "ingest", abstract: "Index all discovered skills")

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let scanner = SkillScanner()
        let skills = scanner.scan(options: opts.scanOptions)
        let fm = FileManager.default
        try fm.createDirectory(at: indexDir, withIntermediateDirectories: true)

        let entries = skills.map { s in
            SkillIndexEntry(
                dirName: s.dirName,
                name: s.name,
                description: s.description,
                provider: s.provider,
                version: s.version,
                path: s.path,
                tokenCount: s.tokenCount,
                ingestedAt: ISO8601DateFormatter().string(from: Date())
            )
        }

        let indexFile = indexDir.appendingPathComponent("index.json")
        let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted]
        try enc.encode(entries).write(to: indexFile)
        print("✓ Indexed \(entries.count) skill(s) to \(indexFile.path)")
    }
}

struct IndexSearchCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "search", abstract: "Search the skill index")

    @Argument(help: "Search query")
    var query: String

    func run() throws {
        let indexFile = indexDir.appendingPathComponent("index.json")
        guard let data = try? Data(contentsOf: indexFile) else {
            fputs("error: index not found — run 'skills-cli index ingest' first\n", stderr)
            throw ExitCode.failure
        }
        let entries = try JSONDecoder().decode([SkillIndexEntry].self, from: data)
        let q = query.lowercased()
        let matches = entries.filter {
            $0.name.lowercased().contains(q) || $0.description.lowercased().contains(q) || $0.dirName.lowercased().contains(q)
        }
        if matches.isEmpty { print("  No results for '\(query)'."); return }
        for m in matches {
            print("  \u{001B}[1m\(m.name)\u{001B}[0m  [\(m.provider)]  \(m.version)")
            if !m.description.isEmpty { print("    \(String(m.description.prefix(80)))") }
        }
    }
}

struct IndexListCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "list", abstract: "List all entries in the skill index")

    func run() throws {
        let indexFile = indexDir.appendingPathComponent("index.json")
        guard let data = try? Data(contentsOf: indexFile) else {
            print("  Index is empty — run 'skills-cli index ingest' first.")
            return
        }
        let entries = try JSONDecoder().decode([SkillIndexEntry].self, from: data)
        print("  \(entries.count) skill(s) in index:")
        for e in entries.sorted(by: { $0.name < $1.name }) {
            print("  • \(e.name.padding(toLength: 30, withPad: " ", startingAt: 0))  [\(e.provider)]")
        }
    }
}
