import Foundation
import SkillsCore

/// Installs skills from GitHub (owner/repo or URL) or local path into all (or a specific) provider.
struct SkillInstaller {
    let noColor: Bool
    let verbose: Bool

    init(noColor: Bool = false, verbose: Bool = false) {
        self.noColor = noColor
        self.verbose = verbose
    }

    // MARK: - Public API

    func install(source: String, tool: String?, yes: Bool) throws {
        if isLocalPath(source) {
            try installLocal(path: source, tool: tool, yes: yes)
        } else {
            try installGitHub(source: source, tool: tool, yes: yes)
        }
    }

    // MARK: - Local

    private func installLocal(path: String, tool: String?, yes: Bool) throws {
        let sourceURL = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
            .resolvingSymlinksInPath()
        let fm = FileManager.default

        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: sourceURL.path, isDirectory: &isDir), isDir.boolValue else {
            throw NSError(domain: "Installer", code: 1,
                          userInfo: [NSLocalizedDescriptionKey: "Not a directory: \(path)"])
        }

        let mdPath = sourceURL.appendingPathComponent("SKILL.md")
        guard fm.fileExists(atPath: mdPath.path) else {
            throw NSError(domain: "Installer", code: 2,
                          userInfo: [NSLocalizedDescriptionKey: "No SKILL.md found in \(path)"])
        }

        let skillName = sourceURL.lastPathComponent
        let targets = resolveTargets(tool: tool)

        print("Installing '\(skillName)' from local path to \(targets.count) provider(s):")
        for (agent, dest) in targets { print("  • \(agent.name)  →  \(dest.path)") }

        if !yes {
            print("Continue? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }

        for (agent, dest) in targets {
            try linkOrCopy(source: sourceURL, dest: dest, agentName: agent.name)
            try LockFileManager.writeLockEntry(
                dirName: skillName, source: path, sourceType: "local",
                commitHash: nil, provider: agent.name
            )
        }
        print("✓ Installed '\(skillName)'.")
    }

    // MARK: - GitHub

    private func installGitHub(source: String, tool: String?, yes: Bool) throws {
        let (repoURL, subpath) = parseGitHubSource(source)
        let skillName = subpath?.components(separatedBy: "/").last ?? repoURL.components(separatedBy: "/").last ?? "skill"

        let targets = resolveTargets(tool: tool)
        print("Installing '\(skillName)' from \(repoURL)...")

        if !yes {
            print("Install to \(targets.count) provider(s)? [y/N] ", terminator: "")
            let answer = readLine()?.lowercased().trimmingCharacters(in: .whitespaces) ?? ""
            guard answer == "y" || answer == "yes" else { print("Aborted."); return }
        }

        // Clone to temp dir
        let tmpDir = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
        let fm = FileManager.default
        defer { try? fm.removeItem(at: tmpDir) }

        try fm.createDirectory(at: tmpDir, withIntermediateDirectories: true)
        print("  Cloning \(repoURL)...")
        try shell("git", "clone", "--depth=1", repoURL, tmpDir.path)

        let commitHash = (try? shell("git", "-C", tmpDir.path, "rev-parse", "HEAD")) ?? ""

        // Find skill directory
        let skillSource: URL
        if let sub = subpath, !sub.isEmpty {
            skillSource = tmpDir.appendingPathComponent(sub)
        } else {
            // Try to find a skills/ subdir or root
            let skillsDir = tmpDir.appendingPathComponent("skills")
            if fm.fileExists(atPath: skillsDir.appendingPathComponent(skillName).appendingPathComponent("SKILL.md").path) {
                skillSource = skillsDir.appendingPathComponent(skillName)
            } else if fm.fileExists(atPath: tmpDir.appendingPathComponent("SKILL.md").path) {
                skillSource = tmpDir
            } else {
                // Try to find any directory with a SKILL.md
                let found = findSkillDir(in: tmpDir)
                if let f = found {
                    skillSource = f
                } else {
                    throw NSError(domain: "Installer", code: 3,
                                  userInfo: [NSLocalizedDescriptionKey: "No SKILL.md found in repository"])
                }
            }
        }

        guard fm.fileExists(atPath: skillSource.appendingPathComponent("SKILL.md").path) else {
            throw NSError(domain: "Installer", code: 4,
                          userInfo: [NSLocalizedDescriptionKey: "SKILL.md not found at expected path \(skillSource.path)"])
        }

        for (agent, dest) in targets {
            try linkOrCopy(source: skillSource, dest: dest, agentName: agent.name)
            try LockFileManager.writeLockEntry(
                dirName: dest.lastPathComponent, source: source, sourceType: "github",
                commitHash: String(commitHash.prefix(40)), provider: agent.name
            )
        }
        print("✓ Installed '\(skillName)' (commit: \(String(commitHash.prefix(8)))).")
    }

    // MARK: - Helpers

    private func resolveTargets(tool: String?) -> [(agent: AgentConfig, dest: URL)] {
        let registry = AgentRegistry.shared
        let agents: [AgentConfig]
        if let t = tool, !t.isEmpty {
            agents = registry.agents.filter { $0.name.lowercased() == t.lowercased() }
        } else {
            agents = registry.agents
        }
        return agents.compactMap { agent -> (AgentConfig, URL)? in
            guard let dir = agent.resolvedURL else { return nil }
            return (agent, dir.appendingPathComponent(dir.lastPathComponent == agent.name ? "" : "")
                .deletingLastPathComponent()
                .appendingPathComponent(dir.lastPathComponent))
        }.map { (agent, dir) in
            let skillDir = (agent.resolvedURL ?? dir)
            return (agent, skillDir.appendingPathComponent(skillDir.lastPathComponent == agent.name ? "" : "")
                .deletingLastPathComponent()
                .appendingPathComponent((agent.resolvedURL ?? dir).lastPathComponent))
        }.compactMap { (agent, _) -> (AgentConfig, URL)? in
            guard let dir = agent.resolvedURL else { return nil }
            // dest = <provider-dir>/<skill-name>  — we don't know skill name yet, return dir
            return (agent, dir)
        }
    }

    private func linkOrCopy(source: URL, dest: URL, agentName: String) throws {
        let fm = FileManager.default
        let skillName = source.lastPathComponent
        let target = dest.appendingPathComponent(skillName)

        try fm.createDirectory(at: dest, withIntermediateDirectories: true)

        if fm.fileExists(atPath: target.path) {
            print("  · Skipping \(agentName) — '\(skillName)' already installed")
            return
        }

        // Copy to destination (not symlink, to keep it self-contained per provider)
        try fm.copyItem(at: source, to: target)
        print("  ✓ \(agentName)")
    }

    private func findSkillDir(in root: URL) -> URL? {
        let fm = FileManager.default
        guard let enumerator = fm.enumerator(at: root, includingPropertiesForKeys: [.isDirectoryKey],
                                              options: [.skipsHiddenFiles]) else { return nil }
        for case let url as URL in enumerator {
            if (try? url.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true {
                if fm.fileExists(atPath: url.appendingPathComponent("SKILL.md").path) {
                    return url
                }
            }
        }
        return nil
    }

    private func isLocalPath(_ input: String) -> Bool {
        input.hasPrefix("/") || input.hasPrefix("./") || input.hasPrefix("../") || input.hasPrefix("~/")
    }

    private func parseGitHubSource(_ source: String) -> (String, String?) {
        // Full URL
        if source.hasPrefix("https://github.com/") {
            let urlStr = source.replacingOccurrences(of: "/tree/[^/]+", with: "", options: .regularExpression)
            return (urlStr.hasSuffix(".git") ? urlStr : urlStr + ".git", nil)
        }
        // owner/repo or owner/repo/subpath
        let parts = source.components(separatedBy: "/")
        guard parts.count >= 2 else { return (source, nil) }
        let repoURL = "https://github.com/\(parts[0])/\(parts[1]).git"
        let subpath = parts.count > 2 ? parts[2...].joined(separator: "/") : nil
        return (repoURL, subpath)
    }
}
