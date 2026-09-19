import Foundation

/// T-01: Outcome of a single evolution operation.
public enum EvolveOutcome: Equatable {
    case schemaBumped(from: String?, to: String)
    case contentPatched(field: String)
    case superseded(by: String)
    case noChange
}

/// T-01: Result returned by SkillEvolver for each skill processed.
public struct EvolveResult: Equatable {
    public let skillName: String
    public let outcomes: [EvolveOutcome]
    public let dryRun: Bool

    public var changed: Bool { outcomes.contains(where: { $0 != .noChange }) }
}

/// T-01 (§3.3, §4.4): Performs schema-bump, content-patch, and supersession on skills.
/// Always honours --dry-run: no filesystem writes occur when dryRun is true.
public struct SkillEvolver {
    private let skillsRoot: URL
    private let logger: (String) -> Void
    private let fileManager = FileManager.default

    public init(skillsRoot: URL, logger: @escaping (String) -> Void = { print($0) }) {
        self.skillsRoot = skillsRoot
        self.logger = logger
    }

    // MARK: - Public API

    /// Bump the `version:` frontmatter field to `newVersion` for `skillName`.
    /// Writes SKILL.md atomically unless dryRun is true.
    @discardableResult
    public func bumpVersion(skillName: String, newVersion: String, dryRun: Bool) throws -> EvolveResult {
        let (url, content, frontmatter) = try loadSkill(named: skillName)
        let oldVersion = frontmatter["version"]
        let patched = setFrontmatterKey(content: content, key: "version", value: newVersion)

        if !dryRun {
            try write(content: patched, to: url)
            logger("  ✓ [\(skillName)] version \(oldVersion ?? "nil") → \(newVersion)")
        } else {
            logger("  · [dry-run] [\(skillName)] would bump version \(oldVersion ?? "nil") → \(newVersion)")
        }

        return EvolveResult(
            skillName: skillName,
            outcomes: [.schemaBumped(from: oldVersion, to: newVersion)],
            dryRun: dryRun
        )
    }

    /// Patch a single frontmatter `field` to `value` for `skillName`.
    @discardableResult
    public func patchField(skillName: String, field: String, value: String, dryRun: Bool) throws -> EvolveResult {
        let (url, content, _) = try loadSkill(named: skillName)
        let patched = setFrontmatterKey(content: content, key: field, value: value)

        if !dryRun {
            try write(content: patched, to: url)
            logger("  ✓ [\(skillName)] patched \(field) = \(value)")
        } else {
            logger("  · [dry-run] [\(skillName)] would patch \(field) = \(value)")
        }

        return EvolveResult(
            skillName: skillName,
            outcomes: [.contentPatched(field: field)],
            dryRun: dryRun
        )
    }

    /// Mark `skillName` as superseded by `replacementName`.
    /// Sets `lifecycle: deprecated` and `superseded_by: <replacementName>` in frontmatter.
    @discardableResult
    public func supersede(skillName: String, by replacementName: String, dryRun: Bool) throws -> EvolveResult {
        let (url, content, _) = try loadSkill(named: skillName)
        var patched = setFrontmatterKey(content: content, key: "lifecycle", value: "deprecated")
        patched = setFrontmatterKey(content: patched, key: "superseded_by", value: replacementName)

        if !dryRun {
            try write(content: patched, to: url)
            logger("  ✓ [\(skillName)] superseded by \(replacementName); lifecycle → deprecated")

            // T-08: publish LifecycleDecisionBundle for this demotion
            let bundle = LifecycleDecisionBundle(
                skillName: skillName,
                decision: .superseded,
                newStatus: SkillLifecycleStatus.deprecated.rawValue,
                supersededBy: replacementName,
                actorUrn: "urn:ckodex:actor:cli:skills-cli"
            )
            LifecyclePublisher(skillsRoot: skillsRoot, logger: logger).publish(bundle)
        } else {
            logger("  · [dry-run] [\(skillName)] would supersede → \(replacementName)")
        }

        return EvolveResult(
            skillName: skillName,
            outcomes: [.superseded(by: replacementName)],
            dryRun: dryRun
        )
    }

    // MARK: - Internals

    private func loadSkill(named name: String) throws -> (url: URL, content: String, frontmatter: [String: String]) {
        let skillMdURL = skillsRoot.appendingPathComponent(name).appendingPathComponent("SKILL.md")
        guard fileManager.fileExists(atPath: skillMdURL.path) else {
            throw SkillsError.invalidSourceDirectory(path: skillMdURL.path)
        }
        let content = try String(contentsOf: skillMdURL, encoding: .utf8)
        return (skillMdURL, content, parseFrontmatter(content))
    }

    /// Parse all key: value pairs from the first YAML frontmatter block.
    private func parseFrontmatter(_ content: String) -> [String: String] {
        var result: [String: String] = [:]
        let lines = content.components(separatedBy: .newlines)
        var inFrontmatter = false
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed == "---" {
                if !inFrontmatter { inFrontmatter = true; continue }
                else { break }
            }
            if inFrontmatter, let colonIdx = trimmed.firstIndex(of: ":") {
                let key = String(trimmed[trimmed.startIndex..<colonIdx]).trimmingCharacters(in: .whitespaces)
                let val = String(trimmed[trimmed.index(after: colonIdx)...])
                    .trimmingCharacters(in: .whitespaces)
                    .replacingOccurrences(of: "\"", with: "")
                if !key.isEmpty { result[key] = val }
            }
        }
        return result
    }

    /// Set or insert `key: value` inside the YAML frontmatter block.
    /// If the key already exists it is replaced; otherwise appended before the closing `---`.
    private func setFrontmatterKey(content: String, key: String, value: String) -> String {
        var lines = content.components(separatedBy: .newlines)
        var inFrontmatter = false
        var foundKey = false

        for i in lines.indices {
            let trimmed = lines[i].trimmingCharacters(in: .whitespaces)
            if trimmed == "---" {
                if !inFrontmatter { inFrontmatter = true; continue }
                // Closing --- : insert key before it if not found
                if !foundKey { lines.insert("\(key): \"\(value)\"", at: i) }
                break
            }
            if inFrontmatter, trimmed.hasPrefix("\(key):") {
                lines[i] = "\(key): \"\(value)\""
                foundKey = true
            }
        }

        return lines.joined(separator: "\n")
    }

    private func write(content: String, to url: URL) throws {
        let tempURL = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try content.write(to: tempURL, atomically: true, encoding: .utf8)
        if fileManager.fileExists(atPath: url.path) {
            _ = try fileManager.replaceItemAt(url, withItemAt: tempURL)
        } else {
            try fileManager.moveItem(at: tempURL, to: url)
        }
    }
}
