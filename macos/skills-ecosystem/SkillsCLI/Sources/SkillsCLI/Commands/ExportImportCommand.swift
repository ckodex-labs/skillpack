import Foundation
import ArgumentParser
import SkillsCore

struct ExportCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "export",
        abstract: "Export skill inventory as JSON manifest"
    )

    @OptionGroup var opts: SharedOptions

    @Option(name: .shortAndLong, help: "Output file path (default: stdout)")
    var output: String = ""

    func run() throws {
        let scanner = SkillScanner()
        let skills = scanner.scan(options: opts.scanOptions)
        let manifest = SkillManifest(
            exportedAt: ISO8601DateFormatter().string(from: Date()),
            skillCount: skills.count,
            skills: skills
        )

        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try enc.encode(manifest)
        let jsonStr = String(data: data, encoding: .utf8) ?? ""

        if output.isEmpty {
            print(jsonStr)
        } else {
            let url = URL(fileURLWithPath: (output as NSString).expandingTildeInPath)
            try data.write(to: url)
            print("✓ Exported \(skills.count) skill(s) to \(url.path)")
        }
    }
}

struct ImportManifestCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "import",
        abstract: "Import skills from a previously exported manifest"
    )

    @Argument(help: "Path to the manifest JSON file")
    var file: String

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompts")
    var yes: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let url = URL(fileURLWithPath: (file as NSString).expandingTildeInPath)
        guard let data = try? Data(contentsOf: url) else {
            fputs("error: cannot read '\(file)'\n", stderr)
            throw ExitCode.failure
        }
        let manifest = try JSONDecoder().decode(SkillManifest.self, from: data)

        print("Manifest contains \(manifest.skillCount) skill(s) exported at \(manifest.exportedAt).")
        if !yes {
            print("Install all to their original paths? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }

        let fm = FileManager.default
        var succeeded = 0, skipped = 0

        for skill in manifest.skills {
            let dest = URL(fileURLWithPath: skill.path)
            if fm.fileExists(atPath: dest.path) {
                print("  · Skipping '\(skill.name)' — already exists at \(skill.path)")
                skipped += 1
                continue
            }
            // For symlink skills we can only note the missing target
            if skill.isSymlink {
                print("  · Skipping '\(skill.name)' — was a symlink, re-install from source")
                skipped += 1
                continue
            }
            print("  (Import of physical skills requires the canonical store. Use 'install' instead for '\(skill.name)')")
            skipped += 1
        }

        print("\nDone. Installed: \(succeeded)  Skipped: \(skipped)")
    }
}

// MARK: - Manifest model

struct SkillManifest: Codable {
    var exportedAt: String
    var skillCount: Int
    var skills: [SkillInfo]
}
