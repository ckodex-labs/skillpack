import Foundation
import ArgumentParser
import SkillsCore

struct BundleCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "bundle",
        abstract: "Manage skill bundles",
        subcommands: [BundleCreateCommand.self, BundleListCommand.self,
                      BundleShowCommand.self, BundleRemoveCommand.self,
                      BundleInstallCommand.self],
        defaultSubcommand: BundleListCommand.self
    )
}

private let bundleDir: URL = {
    let home = FileManager.default.homeDirectoryForCurrentUser
    return home.appendingPathComponent(".config/agent-skill-manager/bundles")
}()

struct SkillBundle: Codable {
    var name: String
    var description: String
    var createdAt: String
    var skills: [String]
}

struct BundleCreateCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "create", abstract: "Create a new bundle from selected skills")

    @Argument(help: "Bundle name")
    var name: String

    @Option(name: .shortAndLong, help: "Bundle description")
    var description: String = ""

    @Option(name: .shortAndLong, help: "Comma-separated skill names to include")
    var skills: String = ""

    func run() throws {
        let skillList = skills.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
        if skillList.isEmpty {
            fputs("error: provide --skills skill1,skill2,...\n", stderr)
            throw ExitCode.failure
        }
        let bundle = SkillBundle(name: name, description: description, createdAt: ISO8601DateFormatter().string(from: Date()), skills: skillList)
        try saveBundleFile(bundle)
        print("✓ Created bundle '\(name)' with \(skillList.count) skill(s).")
    }
}

struct BundleListCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "list", abstract: "List all saved bundles")

    func run() throws {
        let bundles = try loadAllBundles()
        if bundles.isEmpty { print("  No bundles saved."); return }
        print("  \u{001B}[1mBundles\u{001B}[0m")
        for b in bundles { print("  • \(b.name.padding(toLength: 20, withPad: " ", startingAt: 0))  \(b.skills.count) skills  \(b.description)") }
    }
}

struct BundleShowCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "show", abstract: "Show details of a bundle")

    @Argument var name: String

    func run() throws {
        let bundles = try loadAllBundles()
        guard let b = bundles.first(where: { $0.name == name }) else {
            fputs("error: bundle '\(name)' not found\n", stderr); throw ExitCode.failure
        }
        print("\u{001B}[1m\(b.name)\u{001B}[0m  \(b.createdAt)")
        if !b.description.isEmpty { print(b.description) }
        print("Skills:")
        for s in b.skills { print("  • \(s)") }
    }
}

struct BundleRemoveCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "remove", abstract: "Remove a bundle")

    @Argument var name: String

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation")
    var yes: Bool = false

    func run() throws {
        let file = bundleDir.appendingPathComponent("\(name).json")
        guard FileManager.default.fileExists(atPath: file.path) else {
            fputs("error: bundle '\(name)' not found\n", stderr); throw ExitCode.failure
        }
        if !yes {
            print("Remove bundle '\(name)'? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }
        try FileManager.default.removeItem(at: file)
        print("✓ Removed bundle '\(name)'.")
    }
}

struct BundleInstallCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "install", abstract: "Install all skills in a bundle")

    @Argument var name: String

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation")
    var yes: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let bundles = try loadAllBundles()
        guard let bundle = bundles.first(where: { $0.name == name }) else {
            fputs("error: bundle '\(name)' not found\n", stderr); throw ExitCode.failure
        }
        print("Installing \(bundle.skills.count) skill(s) from bundle '\(name)':")
        for skill in bundle.skills { print("  • \(skill)") }
        if !yes {
            print("Continue? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }
        let installer = SkillInstaller(noColor: noColor, verbose: false)
        for skill in bundle.skills {
            do {
                try installer.install(source: skill, tool: nil, yes: true)
                print("  ✓ \(skill)")
            } catch {
                fputs("  ✗ \(skill): \(error.localizedDescription)\n", stderr)
            }
        }
    }
}

// MARK: - Helpers

private func saveBundleFile(_ bundle: SkillBundle) throws {
    let fm = FileManager.default
    try fm.createDirectory(at: bundleDir, withIntermediateDirectories: true)
    let file = bundleDir.appendingPathComponent("\(bundle.name).json")
    let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted]
    try enc.encode(bundle).write(to: file)
}

private func loadAllBundles() throws -> [SkillBundle] {
    let fm = FileManager.default
    guard fm.fileExists(atPath: bundleDir.path) else { return [] }
    let files = try fm.contentsOfDirectory(at: bundleDir, includingPropertiesForKeys: nil)
        .filter { $0.pathExtension == "json" }
    return files.compactMap { try? JSONDecoder().decode(SkillBundle.self, from: Data(contentsOf: $0)) }
        .sorted { $0.name < $1.name }
}
