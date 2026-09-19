import Foundation

public struct ANSI {
    public static let reset    = "\u{001B}[0m"
    public static let bold     = "\u{001B}[1m"
    public static let dim      = "\u{001B}[2m"
    public static let red      = "\u{001B}[31m"
    public static let green    = "\u{001B}[32m"
    public static let yellow   = "\u{001B}[33m"
    public static let blue     = "\u{001B}[34m"
    public static let magenta  = "\u{001B}[35m"
    public static let cyan     = "\u{001B}[36m"
    public static let white    = "\u{001B}[37m"
    public static let gray     = "\u{001B}[90m"

    public static func provider(_ name: String, noColor: Bool = false) -> String {
        guard !noColor else { return name }
        switch name {
        case "claude":      return "\(cyan)\(name)\(reset)"
        case "codex":       return "\(blue)\(name)\(reset)"
        case "opencode":    return "\(green)\(name)\(reset)"
        case "cursor":      return "\(magenta)\(name)\(reset)"
        case "windsurf":    return "\(yellow)\(name)\(reset)"
        case "copilot":     return "\(white)\(name)\(reset)"
        case "gemini":      return "\(blue)\(name)\(reset)"
        default:            return "\(gray)\(name)\(reset)"
        }
    }
}

public struct OutputFormatter {
    public var noColor: Bool
    public var json: Bool
    public var machine: Bool

    public init(noColor: Bool = false, json: Bool = false, machine: Bool = false) {
        self.noColor = noColor
        self.json = json
        self.machine = machine
    }

    // MARK: - JSON

    public func toJSON<T: Encodable>(_ value: T) throws -> String {
        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try enc.encode(value)
        return String(data: data, encoding: .utf8) ?? "[]"
    }

    // MARK: - Skill table

    public func formatSkillTable(_ skills: [SkillInfo], flat: Bool = false, sortBy: String = "name") -> String {
        let sorted = sort(skills, by: sortBy)
        if flat {
            return formatFlat(sorted)
        }
        return formatTable(sorted)
    }

    private func formatTable(_ skills: [SkillInfo]) -> String {
        let col1 = max(20, skills.map { $0.name.count }.max() ?? 20)
        let col2 = 9
        let col3 = 12
        let col4 = 8
        let col5 = max(10, skills.map { $0.providerLabel.count }.max() ?? 10)

        let sep = String(repeating: "─", count: col1 + col2 + col3 + col4 + col5 + 16)
        var lines: [String] = []
        let header = pad("Name", col1) + "  " + pad("Version", col2) + "  " + pad("Provider", col5) + "  " + pad("Tokens", col4) + "  " + "Warnings"
        lines.append(bold(header))
        lines.append(color(sep, ANSI.gray))

        for s in skills {
            let tokens = s.tokenCount.map { "~\($0)" } ?? "—"
            let warnStr = s.warnings.isEmpty ? "" : color("⚠ \(s.warnings.count)", ANSI.yellow)
            let disabledMark = s.disabled ? color(" [disabled]", ANSI.gray) : ""
            let row = pad(s.name, col1) + "  " + pad(s.version, col2) + "  " + pad(s.providerLabel, col5) + "  " + pad(tokens, col4) + "  " + warnStr + disabledMark
            lines.append(row)
        }

        lines.append(color(sep, ANSI.gray))
        lines.append(color("  \(skills.count) skill(s)", ANSI.dim))
        return lines.joined(separator: "\n")
    }

    private func formatFlat(_ skills: [SkillInfo]) -> String {
        var lines: [String] = []
        let header = pad("Name", 30) + "  " + pad("Provider", 15) + "  " + pad("Scope", 8) + "  " + pad("Version", 9) + "  Path"
        lines.append(bold(header))
        lines.append(color(String(repeating: "─", count: 100), ANSI.gray))
        for s in skills {
            let row = pad(s.name, 30) + "  " + pad(s.provider, 15) + "  " + pad(s.scope, 8) + "  " + pad(s.version, 9) + "  " + shorten(s.path)
            lines.append(row)
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Inspect / Detail

    public func formatSkillInspect(_ s: SkillInfo) -> String {
        var lines: [String] = []
        lines.append("\n" + bold(s.name) + color("  \(s.version)", ANSI.gray))
        lines.append(color(String(repeating: "─", count: 60), ANSI.gray))
        lines.append(field("Description",  s.description.isEmpty ? color("(none)", ANSI.gray) : s.description))
        lines.append(field("Provider",     "\(ANSI.provider(s.provider, noColor: noColor))  (\(s.providerLabel))"))
        lines.append(field("Scope",        s.scope))
        lines.append(field("Location",     s.location))
        lines.append(field("Path",         shorten(s.path)))
        lines.append(field("Real path",    shorten(s.realPath)))
        lines.append(field("Symlink",      s.isSymlink ? (s.symlinkTarget.map { "→ \(shorten($0))" } ?? "yes") : "no"))
        lines.append(field("Tokens",       s.tokenCount.map { "~\($0)" } ?? "—"))
        lines.append(field("Files",        s.fileCount.map { "\($0)" } ?? "—"))
        if !s.license.isEmpty    { lines.append(field("License",      s.license)) }
        if !s.creator.isEmpty    { lines.append(field("Creator",      s.creator)) }
        if !s.compatibility.isEmpty { lines.append(field("Compat",    s.compatibility)) }
        if !s.allowedTools.isEmpty { lines.append(field("Tools",      s.allowedTools.joined(separator: ", "))) }
        if !s.warnings.isEmpty {
            lines.append("")
            lines.append(color("  Warnings:", ANSI.yellow))
            for w in s.warnings {
                lines.append(color("    ⚠ [\(w.category)] \(w.message)", ANSI.yellow))
            }
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Search results

    public func formatSearchResults(_ skills: [SkillInfo], query: String) -> String {
        if skills.isEmpty {
            return color("  No skills found matching '\(query)'", ANSI.gray)
        }
        var lines: [String] = [bold("  Search results for '\(query)':"), ""]
        for s in skills {
            let snippet = s.description.isEmpty ? color("(no description)", ANSI.gray) : String(s.description.prefix(80))
            lines.append("  " + bold(s.name) + color("  \(s.version)  [\(s.provider)]", ANSI.gray))
            lines.append("    " + snippet)
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Stats

    public func formatStats(skills: [SkillInfo], duplicateCount: Int) -> String {
        var lines: [String] = [bold("\n  Skills Statistics"), color("  " + String(repeating: "─", count: 40), ANSI.gray)]
        let byProvider = Dictionary(grouping: skills, by: \.provider)
            .mapValues { $0.count }
            .sorted { $0.key < $1.key }
        lines.append(field("Total skills", "\(skills.count)"))
        lines.append(field("Global",       "\(skills.filter { $0.scope == "global" }.count)"))
        lines.append(field("Project",      "\(skills.filter { $0.scope == "project" }.count)"))
        lines.append(field("Duplicates",   "\(duplicateCount)"))
        lines.append(field("Disabled",     "\(skills.filter { $0.disabled }.count)"))
        let totalTokens = skills.compactMap(\.tokenCount).reduce(0, +)
        lines.append(field("Total tokens", "~\(totalTokens)"))
        lines.append("")
        lines.append(bold("  By provider:"))
        for (provider, count) in byProvider {
            lines.append("    " + ANSI.provider(provider, noColor: noColor) + "  \(count)")
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Doctor

    public func formatDoctorReport(_ checks: [(String, Bool, String)]) -> String {
        var lines: [String] = [bold("\n  Doctor report"), color("  " + String(repeating: "─", count: 50), ANSI.gray)]
        for (name, ok, detail) in checks {
            let icon = ok ? color("✓", ANSI.green) : color("✗", ANSI.red)
            let msg  = detail.isEmpty ? "" : color("  \(detail)", ANSI.gray)
            lines.append("  \(icon)  \(name)\(msg)")
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Audit

    public struct DuplicateGroup {
        public var key: String
        public var reason: String
        public var instances: [SkillInfo]
        public init(key: String, reason: String, instances: [SkillInfo]) {
            self.key = key; self.reason = reason; self.instances = instances
        }
    }

    public func formatAuditReport(_ groups: [DuplicateGroup]) -> String {
        if groups.isEmpty {
            return color("  ✓ No duplicate skills found.", ANSI.green)
        }
        var lines: [String] = [bold("\n  Duplicate skills detected: \(groups.count) group(s)"), ""]
        for g in groups {
            lines.append(color("  ── \(g.key)  [\(g.reason)]", ANSI.yellow))
            for inst in g.instances {
                lines.append("    • \(inst.provider)  \(shorten(inst.path))")
            }
        }
        return lines.joined(separator: "\n")
    }

    // MARK: - Helpers

    private func sort(_ skills: [SkillInfo], by field: String) -> [SkillInfo] {
        switch field {
        case "version":  return skills.sorted { $0.version < $1.version }
        case "location": return skills.sorted { $0.location < $1.location }
        default:         return skills.sorted { $0.name.lowercased() < $1.name.lowercased() }
        }
    }

    private func pad(_ s: String, _ width: Int) -> String {
        if s.count >= width { return String(s.prefix(width - 1)) + "…" }
        return s + String(repeating: " ", count: width - s.count)
    }

    public func shorten(_ path: String) -> String {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        return path.replacingOccurrences(of: home, with: "~")
    }

    private func field(_ label: String, _ value: String) -> String {
        let padded = pad(label, 14)
        return "  " + color(padded, ANSI.gray) + "  " + value
    }

    private func bold(_ s: String) -> String {
        guard !noColor else { return s }
        return "\(ANSI.bold)\(s)\(ANSI.reset)"
    }

    private func color(_ s: String, _ code: String) -> String {
        guard !noColor else { return s }
        return "\(code)\(s)\(ANSI.reset)"
    }
}
