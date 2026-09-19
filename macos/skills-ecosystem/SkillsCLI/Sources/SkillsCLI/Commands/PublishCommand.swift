import Foundation
import ArgumentParser
import SkillsCore

struct PublishCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "publish",
        abstract: "Validate, audit, and submit a skill to the registry"
    )

    @Argument(help: "Path to skill directory (default: current directory)")
    var path: String = "."

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let skillURL = URL(fileURLWithPath: (path as NSString).expandingTildeInPath).resolvingSymlinksInPath()
        let fm = FileManager.default

        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: skillURL.path, isDirectory: &isDir), isDir.boolValue else {
            fputs("error: '\(path)' is not a directory\n", stderr); throw ExitCode.failure
        }

        let mdURL = skillURL.appendingPathComponent("SKILL.md")
        guard fm.fileExists(atPath: mdURL.path) else {
            fputs("error: no SKILL.md found — run 'skills-cli init <name>' first\n", stderr); throw ExitCode.failure
        }

        let parser = FrontmatterParser()
        let fm2 = parser.parse(url: mdURL)
        let skillName = skillURL.lastPathComponent

        print("\n  \u{001B}[1mPublish checklist for '\(skillName)'\u{001B}[0m")
        print("  " + String(repeating: "─", count: 50))

        var issues: [String] = []

        let checks: [(String, Bool)] = [
            ("SKILL.md exists",            true),
            ("description present",        !fm2.description.isEmpty),
            ("version present",            !fm2.version.isEmpty && fm2.version != "0.0.0"),
            ("creator present",            !fm2.creator.isEmpty),
            ("license present",            !fm2.license.isEmpty),
            ("name is kebab-case",         skillName == skillName.lowercased() && !skillName.contains(" "))
        ]

        for (label, ok) in checks {
            let icon = ok ? "\u{001B}[32m✓\u{001B}[0m" : "\u{001B}[31m✗\u{001B}[0m"
            print("  \(icon)  \(label)")
            if !ok { issues.append(label) }
        }

        if !issues.isEmpty {
            print("\n  \u{001B}[31mFix \(issues.count) issue(s) before publishing.\u{001B}[0m")
            throw ExitCode.failure
        }

        print("\n  \u{001B}[32m✓ Skill passes all publish checks.\u{001B}[0m")
        print("\n  \u{001B}[1mSubmission instructions:\u{001B}[0m")
        print("  1. Fork https://github.com/your-org/skills-registry")
        print("  2. Copy '\(skillURL.path)' to skills/\(skillName)/")
        print("  3. Open a Pull Request with title: 'Add skill: \(skillName)'")
        print("  4. Fill in the PR template and request review.")
    }
}
