import Foundation
import ArgumentParser
import SkillsCore

struct InitCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "init",
        abstract: "Scaffold a new skill with SKILL.md template"
    )

    @Argument(help: "Skill name (kebab-case recommended)")
    var name: String

    @Option(name: .shortAndLong, help: "Directory to create the skill in (default: current dir)")
    var output: String = "."

    func run() throws {
        let skillName = name.lowercased()
            .replacingOccurrences(of: " ", with: "-")
            .components(separatedBy: CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "-_")).inverted)
            .joined()

        let baseDir = URL(fileURLWithPath: (output as NSString).expandingTildeInPath)
        let skillDir = baseDir.appendingPathComponent(skillName)
        let fm = FileManager.default

        if fm.fileExists(atPath: skillDir.path) {
            fputs("error: '\(skillDir.path)' already exists\n", stderr)
            throw ExitCode.failure
        }

        try fm.createDirectory(at: skillDir, withIntermediateDirectories: true)

        let template = """
        ---
        name: \(skillName)
        version: "0.1.0"
        description: "A short description of what this skill does."
        creator: ""
        license: ""
        compatibility: ""
        tools: []
        ---

        # \(skillName)

        Describe when and how to use this skill.

        ## Usage

        Provide usage examples and context here.
        """

        let mdURL = skillDir.appendingPathComponent("SKILL.md")
        try template.write(to: mdURL, atomically: true, encoding: .utf8)

        print("✓ Scaffolded skill at \(skillDir.path)")
        print("  Edit SKILL.md to complete the skill definition.")
    }
}
