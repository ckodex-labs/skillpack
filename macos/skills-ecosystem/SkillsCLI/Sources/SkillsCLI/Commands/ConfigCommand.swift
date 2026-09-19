import Foundation
import ArgumentParser
import SkillsCore

struct ConfigCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "config",
        abstract: "Manage skills-cli configuration",
        subcommands: [ConfigShowCommand.self, ConfigPathCommand.self,
                      ConfigResetCommand.self, ConfigEditCommand.self],
        defaultSubcommand: ConfigShowCommand.self
    )
}

struct AppConfig: Codable {
    var version: Int
    var providers: [ProviderEntry]
    var customPaths: [CustomPath]

    struct ProviderEntry: Codable {
        var name: String
        var enabled: Bool
    }

    struct CustomPath: Codable {
        var label: String
        var path: String
        var scope: String
    }

    static var defaults: AppConfig {
        let providers = AgentRegistry.shared.agents.map {
            ProviderEntry(name: $0.name, enabled: true)
        }
        return AppConfig(version: 1, providers: providers, customPaths: [])
    }
}

private let configDir: URL = FileManager.default.homeDirectoryForCurrentUser
    .appendingPathComponent(".config/agent-skill-manager")

private let configFile: URL = configDir.appendingPathComponent("config.json")

func loadAppConfig() -> AppConfig {
    guard let data = try? Data(contentsOf: configFile),
          let cfg = try? JSONDecoder().decode(AppConfig.self, from: data) else {
        return AppConfig.defaults
    }
    return cfg
}

func saveAppConfig(_ cfg: AppConfig) throws {
    try FileManager.default.createDirectory(at: configDir, withIntermediateDirectories: true)
    let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
    try enc.encode(cfg).write(to: configFile)
}

struct ConfigShowCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "show", abstract: "Print current config")

    func run() throws {
        let cfg = loadAppConfig()
        let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        if let data = try? enc.encode(cfg), let str = String(data: data, encoding: .utf8) {
            print(str)
        }
    }
}

struct ConfigPathCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "path", abstract: "Print config file path")

    func run() {
        print(configFile.path)
    }
}

struct ConfigResetCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "reset", abstract: "Reset config to defaults")

    @Flag(name: [.customShort("y"), .customLong("yes")], help: "Skip confirmation")
    var yes: Bool = false

    func run() throws {
        if !yes {
            print("Reset config to defaults? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }
        try saveAppConfig(AppConfig.defaults)
        print("✓ Config reset to defaults at \(configFile.path)")
    }
}

struct ConfigEditCommand: ParsableCommand {
    static var configuration = CommandConfiguration(commandName: "edit", abstract: "Open config in $EDITOR")

    func run() throws {
        // Ensure file exists
        if !FileManager.default.fileExists(atPath: configFile.path) {
            try saveAppConfig(AppConfig.defaults)
        }
        let editor = ProcessInfo.processInfo.environment["EDITOR"] ?? "vi"
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        proc.arguments = [editor, configFile.path]
        try proc.run()
        proc.waitUntilExit()
    }
}
