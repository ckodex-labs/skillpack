import Foundation
import ArgumentParser
import SkillsCore

struct LinkCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "link",
        abstract: "Symlink a local skill directory into all enabled agent directories"
    )

    @Argument(help: "Path to the skill directory to link")
    var path: String

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Link to a specific provider only")
    var tool: String = ""

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompts")
    var yes: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let sourceURL = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
            .resolvingSymlinksInPath()
        let fm = FileManager.default

        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: sourceURL.path, isDirectory: &isDir), isDir.boolValue else {
            fputs("error: '\(path)' is not a directory\n", stderr)
            throw ExitCode.failure
        }

        let mdPath = sourceURL.appendingPathComponent("SKILL.md")
        guard fm.fileExists(atPath: mdPath.path) else {
            fputs("error: no SKILL.md found in '\(path)'\n", stderr)
            throw ExitCode.failure
        }

        let skillName = sourceURL.lastPathComponent
        let registry = AgentRegistry.shared
        let agents = tool.isEmpty ? registry.agents : registry.agents.filter { $0.name.lowercased() == tool.lowercased() }

        print("Will link '\(skillName)' into:")
        let targets = agents.compactMap { agent -> (AgentConfig, URL)? in
            guard let dir = agent.resolvedURL else { return nil }
            return (agent, dir.appendingPathComponent(skillName))
        }

        for (agent, dest) in targets {
            print("  • \(agent.name)  →  \(dest.path)")
        }

        if !yes {
            print("\nContinue? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }

        for (agent, dest) in targets {
            let parentDir = dest.deletingLastPathComponent()
            do {
                try fm.createDirectory(at: parentDir, withIntermediateDirectories: true)
                if fm.fileExists(atPath: dest.path) {
                    print("  · Skipping \(agent.name) — link already exists")
                    continue
                }
                try fm.createSymbolicLink(at: dest, withDestinationURL: sourceURL)
                print("  ✓ Linked \(agent.name)")
            } catch {
                fputs("  ✗ Failed for \(agent.name): \(error.localizedDescription)\n", stderr)
            }
        }
        print("Done.")
    }
}
