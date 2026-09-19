import Foundation
import ArgumentParser
import SkillsCore

struct AuditCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "audit",
        abstract: "Detect duplicate skills across tools",
        subcommands: [AuditSecurityCommand.self]
    )

    @OptionGroup var opts: SharedOptions

    func run() throws {
        let scanner = SkillScanner()
        let skills = scanner.scan(options: opts.scanOptions)
        let groups = detectDuplicates(skills)

        let fmt = opts.formatter
        if opts.json || opts.machine {
            struct AuditResult: Codable {
                var duplicateCount: Int
                var groups: [[String: String]]
            }
            let result = AuditResult(
                duplicateCount: groups.count,
                groups: groups.flatMap { g in
                    g.instances.map { i in
                        ["key": g.key, "reason": g.reason, "provider": i.provider, "path": i.path]
                    }
                }
            )
            let output = try fmt.toJSON(result)
            print(opts.machine ? machineEnvelope(data: output, command: "audit") : output)
        } else {
            let converted = groups.map {
                OutputFormatter.DuplicateGroup(key: $0.key, reason: $0.reason, instances: $0.instances)
            }
            print(fmt.formatAuditReport(converted))
        }
    }

    private func detectDuplicates(_ skills: [SkillInfo]) -> [(key: String, reason: String, instances: [SkillInfo])] {
        var groups: [(key: String, reason: String, instances: [SkillInfo])] = []

        // Dedupe by realPath first (symlinks pointing to same target = same skill)
        var seenReal = [String: SkillInfo]()
        var deduped: [SkillInfo] = []
        for s in skills {
            if let existing = seenReal[s.realPath] {
                if !s.isSymlink && existing.isSymlink {
                    deduped[deduped.firstIndex(where: { $0.path == existing.path })!] = s
                    seenReal[s.realPath] = s
                }
            } else {
                seenReal[s.realPath] = s
                deduped.append(s)
            }
        }

        // Rule 1: same dirName across ≥2 different locations
        let byDirName = Dictionary(grouping: deduped, by: \.dirName)
        for (dirName, members) in byDirName.sorted(by: { $0.key < $1.key }) {
            let locations = Set(members.map(\.location))
            if locations.count >= 2 {
                groups.append((key: dirName, reason: "same-dirName", instances: members))
            }
        }

        // Rule 2: same frontmatter name but different dirName
        let byName = Dictionary(grouping: deduped, by: \.name)
        for (name, members) in byName.sorted(by: { $0.key < $1.key }) {
            let dirNames = Set(members.map(\.dirName))
            if dirNames.count >= 2 {
                let alreadyCovered = groups.contains(where: { $0.instances.contains(where: { $0.name == name }) })
                if !alreadyCovered {
                    groups.append((key: name, reason: "same-name-different-dir", instances: members))
                }
            }
        }

        return groups
    }
}

struct AuditSecurityCommand: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "security",
        abstract: "Run a security audit on a skill"
    )

    @Argument(help: "Skill name or GitHub source")
    var name: String

    @Flag(name: .long, help: "Output as JSON")
    var json: Bool = false

    @Flag(name: .long, help: "Disable ANSI colors")
    var noColor: Bool = false

    func run() throws {
        let suspiciousPatterns: [String] = [
            "eval(", "exec(", "shell_exec", "child_process", "subprocess",
            "os.system", "__import__", "importlib", "socket.connect",
            "base64.decode", "atob(", "fromCharCode", "document.write",
            "innerHTML", "dangerouslySetInner"
        ]
        let scanner = SkillScanner()
        let all = scanner.scan(options: ScanOptions(scope: "both", includeDisabled: false))
        let target = name.lowercased()
        let matches = all.filter { $0.name.lowercased() == target || $0.dirName.lowercased() == target }

        var findings: [(path: String, pattern: String, line: Int)] = []
        let fm = FileManager.default

        for skill in matches {
            let skillURL = URL(fileURLWithPath: skill.path)
            guard let enumerator = fm.enumerator(at: skillURL, includingPropertiesForKeys: [.isRegularFileKey],
                                                  options: [.skipsHiddenFiles]) else { continue }
            for case let file as URL in enumerator {
                guard (try? file.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true else { continue }
                guard let content = try? String(contentsOf: file, encoding: .utf8) else { continue }
                let lines = content.components(separatedBy: .newlines)
                for (i, line) in lines.enumerated() {
                    for pattern in suspiciousPatterns {
                        if line.contains(pattern) {
                            findings.append((path: fm.displayName(atPath: file.path), pattern: pattern, line: i + 1))
                        }
                    }
                }
            }
        }

        if json {
            struct SecurityResult: Codable {
                var skillName: String
                var passed: Bool
                var findingCount: Int
                var findings: [[String: String]]
            }
            let result = SecurityResult(
                skillName: name,
                passed: findings.isEmpty,
                findingCount: findings.count,
                findings: findings.map { ["file": $0.path, "pattern": $0.pattern, "line": "\($0.line)"] }
            )
            let enc = JSONEncoder(); enc.outputFormatting = [.prettyPrinted]
            if let data = try? enc.encode(result), let str = String(data: data, encoding: .utf8) {
                print(str)
            }
        } else {
            if findings.isEmpty {
                print("  \u{001B}[32m✓\u{001B}[0m No suspicious patterns found in '\(name)'.")
            } else {
                print("  \u{001B}[33m⚠\u{001B}[0m \(findings.count) suspicious pattern(s) found in '\(name)':")
                for f in findings {
                    print("    line \(f.line)  [\(f.pattern)]  \(f.path)")
                }
            }
        }
    }
}
