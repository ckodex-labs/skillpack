import Foundation
import ArgumentParser
import SkillsCore

struct DoctorCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "doctor",
        abstract: "Run environment health checks and diagnostics"
    )

    @Flag(name: .long, help: "Output as JSON")
    var json: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        var checks: [(name: String, ok: Bool, detail: String)] = []

        // 1. git available
        let gitOk = (try? shell("git", "--version")) != nil
        checks.append(("git available", gitOk, gitOk ? "" : "Install git via Xcode CLT: xcode-select --install"))

        // 2. Provider directories accessible
        let registry = AgentRegistry.shared
        for agent in registry.agents {
            if let url = agent.resolvedURL {
                let exists = FileManager.default.fileExists(atPath: url.path)
                checks.append(("provider: \(agent.name)", exists, exists ? url.path : "directory not found: \(url.path)"))
            }
        }

        // 3. Config dir writable
        let configDir = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/agent-skill-manager")
        let configOk: Bool
        do {
            try FileManager.default.createDirectory(at: configDir, withIntermediateDirectories: true)
            configOk = true
        } catch { configOk = false }
        checks.append(("config dir writable", configOk, configOk ? configDir.path : "cannot create config dir"))

        // 4. Lock file valid JSON (if exists)
        let lockFile = configDir.appendingPathComponent(".skill-lock.json")
        if FileManager.default.fileExists(atPath: lockFile.path) {
            let lockOk: Bool
            if let data = try? Data(contentsOf: lockFile),
               (try? JSONDecoder().decode(SkillLockFile.self, from: data)) != nil {
                lockOk = true
            } else { lockOk = false }
            checks.append(("lock file valid", lockOk, lockOk ? "" : "lock file is corrupted — delete it and re-install skills"))
        }

        // 5. Scan for orphaned symlinks across providers
        var orphans: [String] = []
        let scanner = SkillScanner()
        let skills = scanner.scan(options: ScanOptions(scope: "global"))
        for skill in skills where skill.isSymlink {
            let target = skill.symlinkTarget ?? skill.realPath
            if !FileManager.default.fileExists(atPath: target) {
                orphans.append(skill.path)
            }
        }
        checks.append(("no orphaned symlinks", orphans.isEmpty,
                        orphans.isEmpty ? "" : "\(orphans.count) orphaned: " + orphans.prefix(3).joined(separator: ", ")))

        // Output
        let fmt = OutputFormatter(noColor: noColor)
        if json {
            struct DoctorResult: Codable {
                var passed: Bool
                var checks: [[String: String]]
            }
            let allOk = checks.allSatisfy(\.ok)
            let result = DoctorResult(
                passed: allOk,
                checks: checks.map { ["name": $0.name, "ok": $0.ok ? "true" : "false", "detail": $0.detail] }
            )
            let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted]
            if let data = try? enc.encode(result), let str = String(data: data, encoding: .utf8) { print(str) }
        } else {
            print(fmt.formatDoctorReport(checks.map { ($0.name, $0.ok, $0.detail) }))
            let passed = checks.allSatisfy(\.ok)
            let summary = passed ? "\u{001B}[32m✓ All checks passed\u{001B}[0m" : "\u{001B}[33m⚠ Some checks failed\u{001B}[0m"
            print("\n  \(summary)")
        }
    }
}
