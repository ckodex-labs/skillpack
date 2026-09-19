import Foundation
import ArgumentParser
import SkillsCore

struct InstallCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "install",
        abstract: "Install a skill from GitHub or local path"
    )

    @Argument(help: "Source: GitHub owner/repo, full GitHub URL, or local path")
    var source: String

    @Option(name: [.customShort("p"), .customLong("tool")], help: "Install to a specific provider only")
    var tool: String = ""

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation prompts")
    var yes: Bool = false

    @Flag(name: [.customShort("V"), .customLong("verbose")], help: "Show debug output")
    var verbose: Bool = false

    func run() throws {
        let installer = SkillInstaller(noColor: noColor, verbose: verbose)
        try installer.install(source: source, tool: tool.isEmpty ? nil : tool, yes: yes)
    }
}
