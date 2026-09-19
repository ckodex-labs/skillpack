import Foundation
import ArgumentParser
import SkillsCore

@main
struct SkillsCLI: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "skills-cli",
        abstract: "Agent Skill Manager — native macOS CLI for managing AI coding agent skills.",
        discussion: """
        Manages installed skills across all AI coding agent providers (Claude, Codex,
        Windsurf, Cursor, Copilot, OpenCode, Gemini, and 10+ more) from a single CLI.

        Run 'skills-cli <command> --help' for command-specific options.
        """,
        version: "1.0.0",
        subcommands: [
            ListCommand.self,
            SearchCommand.self,
            InspectCommand.self,
            InstallCommand.self,
            UninstallCommand.self,
            DisableCommand.self,
            EnableCommand.self,
            AuditCommand.self,
            ExportCommand.self,
            ImportManifestCommand.self,
            InitCommand.self,
            StatsCommand.self,
            LinkCommand.self,
            OutdatedCommand.self,
            UpdateCommand.self,
            PublishCommand.self,
            EvalCommand.self,
            BundleCommand.self,
            IndexCommand.self,
            DoctorCommand.self,
            ConfigCommand.self,
            // Legacy canonical-store commands (kept for backward compat)
            LegacySyncCommand.self,
        ],
        defaultSubcommand: ListCommand.self
    )
}

