import Foundation
import ArgumentParser
import SkillsCore

struct OutdatedCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "outdated",
        abstract: "Show which installed skills have newer versions available"
    )

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let lockFile = LockFileManager.load()
        let scanner = SkillScanner()
        let skills = scanner.scan(options: opts.scanOptions)

        var entries: [[String: String]] = []
        var outdatedCount = 0

        for skill in skills {
            guard let lock = lockFile.skills[skill.dirName] else {
                entries.append([
                    "name": skill.name,
                    "dirName": skill.dirName,
                    "status": "untracked",
                    "installedCommit": "",
                    "latestCommit": "",
                    "source": ""
                ])
                continue
            }

            var latestCommit = ""
            var status = "up-to-date"

            if lock.sourceType == "github", let commit = lock.commitHash {
                latestCommit = (try? fetchRemoteHead(source: lock.source)) ?? ""
                if !latestCommit.isEmpty && !commit.isEmpty && !latestCommit.hasPrefix(commit) {
                    status = "outdated"
                    outdatedCount += 1
                }
            } else {
                status = "untracked"
            }

            entries.append([
                "name": skill.name,
                "dirName": skill.dirName,
                "status": status,
                "installedCommit": lock.commitHash ?? "",
                "latestCommit": String(latestCommit.prefix(8)),
                "source": lock.source
            ])
        }

        let fmt = opts.formatter
        if opts.json || opts.machine {
            struct OutdatedResult: Codable {
                var outdatedCount: Int
                var entries: [[String: String]]
            }
            let result = OutdatedResult(outdatedCount: outdatedCount, entries: entries)
            let output = try fmt.toJSON(result)
            print(opts.machine ? machineEnvelope(data: output, command: "outdated") : output)
        } else {
            if entries.isEmpty {
                print("  No tracked skills found.")
                return
            }
            print(String(format: "  %-30s  %-10s  %-8s  %s", "Name", "Status", "Installed", "Latest"))
            print("  " + String(repeating: "─", count: 70))
            for e in entries.sorted(by: { ($0["name"] ?? "") < ($1["name"] ?? "") }) {
                let statusStr: String
                switch e["status"] {
                case "outdated":   statusStr = "\u{001B}[33moutdated\u{001B}[0m"
                case "up-to-date": statusStr = "\u{001B}[32mup-to-date\u{001B}[0m"
                default:           statusStr = "\u{001B}[90muntracked\u{001B}[0m"
                }
                let inst = String((e["installedCommit"] ?? "").prefix(8))
                let latest = String((e["latestCommit"] ?? "").prefix(8))
                print(String(format: "  %-30s  %s  %-8s  %s", e["name"] ?? "", statusStr, inst, latest))
            }
            print("\n  \(outdatedCount) outdated / \(entries.count) total")
        }
    }

    private func fetchRemoteHead(source: String) throws -> String {
        let url: String
        if source.hasPrefix("https://") {
            url = source
        } else {
            let parts = source.split(separator: "/")
            guard parts.count >= 2 else { return "" }
            url = "https://github.com/\(source)"
        }
        let result = try shell("git", "ls-remote", url, "HEAD")
        return String(result.split(separator: "\t").first ?? "").trimmingCharacters(in: .whitespaces)
    }
}

struct UpdateCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "update",
        abstract: "Update outdated skills"
    )

    @Argument(help: "Specific skill names to update (omit for all outdated)")
    var names: [String] = []

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompts")
    var yes: Bool = false

    @Flag(name: .long, help: "Output as JSON")
    var json: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let lockFile = LockFileManager.load()
        let scanner = SkillScanner()
        let allSkills = scanner.scan(options: ScanOptions(scope: "global"))

        let targets: [SkillInfo]
        if names.isEmpty {
            targets = allSkills.filter { lockFile.skills[$0.dirName]?.sourceType == "github" }
        } else {
            let lower = names.map { $0.lowercased() }
            targets = allSkills.filter { lower.contains($0.dirName.lowercased()) || lower.contains($0.name.lowercased()) }
        }

        if targets.isEmpty {
            print("  No skills to update.")
            return
        }

        print("Skills to update: \(targets.map(\.name).joined(separator: ", "))")
        if !yes {
            print("Continue? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }

        for skill in targets {
            guard let lock = lockFile.skills[skill.dirName] else { continue }
            let installer = SkillInstaller(noColor: noColor, verbose: false)
            do {
                // Re-install from source (atomic: install to temp, validate, replace)
                print("  Updating \(skill.name) from \(lock.source)...")
                try installer.install(source: lock.source, tool: lock.provider, yes: true)
                print("  ✓ \(skill.name) updated")
            } catch {
                fputs("  ✗ Failed to update \(skill.name): \(error.localizedDescription)\n", stderr)
            }
        }
    }
}
