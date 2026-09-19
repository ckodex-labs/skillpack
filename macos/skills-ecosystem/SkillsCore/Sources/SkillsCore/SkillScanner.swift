import Foundation

public struct ScanOptions {
    public var scope: String
    public var providerFilter: String?
    public var includeDisabled: Bool

    public init(scope: String = "both", providerFilter: String? = nil, includeDisabled: Bool = false) {
        self.scope = scope
        self.providerFilter = providerFilter
        self.includeDisabled = includeDisabled
    }
}

public struct SkillScanner {
    private let fileManager = FileManager.default
    private let frontmatter = FrontmatterParser()
    private let tokenCounter = TokenCounter()

    public init() {}

    public func scan(options: ScanOptions = ScanOptions()) -> [SkillInfo] {
        let registry = AgentRegistry.shared
        var results: [SkillInfo] = []

        for agent in registry.agents {
            if let pf = options.providerFilter, !pf.isEmpty {
                if agent.name.lowercased() != pf.lowercased() { continue }
            }

            let label = registry.labels[agent.name] ?? agent.name

            // Global scope — use resolvedURL which honours env overrides
            if options.scope == "global" || options.scope == "both" {
                let dir = agent.resolvedURL ?? resolveDir(agent.defaultPath)
                scanDirectory(dir: dir, agent: agent, label: label, scope: "global",
                              location: "global-\(agent.name)", into: &results,
                              includeDisabled: options.includeDisabled)
            }

            // Project scope (cwd)
            if options.scope == "project" || options.scope == "both" {
                let projectDir = projectPath(for: agent)
                if let pd = projectDir {
                    scanDirectory(dir: pd, agent: agent, label: label, scope: "project",
                                  location: "project-\(agent.name)", into: &results,
                                  includeDisabled: options.includeDisabled)
                }
            }
        }

        return results.sorted { $0.name.lowercased() < $1.name.lowercased() }
    }

    // MARK: - Private

    private func resolveDir(_ path: String) -> URL {
        let expanded = (path as NSString).expandingTildeInPath
        return URL(fileURLWithPath: expanded)
    }

    private func projectPath(for agent: AgentConfig) -> URL? {
        let cwd = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        let projectSubdir: String
        switch agent.name {
        case "claude":      projectSubdir = ".claude/skills"
        case "codex":       projectSubdir = ".codex/skills"
        case "opencode":    projectSubdir = ".opencode/skills"
        case "cursor":      projectSubdir = ".cursor/rules"
        case "copilot":     projectSubdir = ".github/instructions"
        case "windsurf":    projectSubdir = ".windsurf/rules"
        case "cline":       projectSubdir = ".clinerules"
        case "roo":         projectSubdir = ".roo/rules"
        case "continue":    projectSubdir = ".continue/rules"
        case "aider":       projectSubdir = ".aider/skills"
        case "zed":         projectSubdir = ".zed/rules"
        case "augment":     projectSubdir = ".augment/rules"
        default:            projectSubdir = ".\(agent.name)/skills"
        }
        let url = cwd.appendingPathComponent(projectSubdir)
        return url
    }

    private func scanDirectory(dir: URL, agent: AgentConfig, label: String, scope: String,
                               location: String, into results: inout [SkillInfo],
                               includeDisabled: Bool) {
        var isDir: ObjCBool = false
        guard fileManager.fileExists(atPath: dir.path, isDirectory: &isDir), isDir.boolValue else {
            return
        }

        let contents: [URL]
        do {
            contents = try fileManager.contentsOfDirectory(
                at: dir, includingPropertiesForKeys: [.isDirectoryKey, .isSymbolicLinkKey],
                options: [.skipsHiddenFiles]
            )
        } catch { return }

        for item in contents {
            var itemIsDir: ObjCBool = false
            guard fileManager.fileExists(atPath: item.path, isDirectory: &itemIsDir), itemIsDir.boolValue else {
                continue
            }

            let skillMd = item.appendingPathComponent("SKILL.md")
            let skillMdDisabled = item.appendingPathComponent("SKILL.md.disabled")

            let isActive = fileManager.fileExists(atPath: skillMd.path)
            let isDisabled = !isActive && fileManager.fileExists(atPath: skillMdDisabled.path)

            if !isActive && !isDisabled { continue }
            if isDisabled && !includeDisabled { continue }

            let mdURL = isDisabled ? skillMdDisabled : skillMd
            let fm = frontmatter.parse(url: mdURL)
            let tokens = tokenCounter.estimate(url: mdURL)

            let symlinkInfo = resolveSymlink(item)
            let realPath = symlinkInfo.realPath ?? item.path

            var warnings: [SkillWarning] = []
            if fm.version == "0.0.0" || fm.version.isEmpty {
                warnings.append(SkillWarning(category: "missing-version",
                    message: "Skill has no version (or default 0.0.0) in SKILL.md frontmatter"))
            }
            if fm.description.isEmpty {
                warnings.append(SkillWarning(category: "missing-description",
                    message: "Skill has no description in SKILL.md frontmatter"))
            }
            if tokens > 50_000 {
                warnings.append(SkillWarning(category: "oversized",
                    message: "SKILL.md is very large (~\(tokens) tokens); consider splitting"))
            }

            let skillName = fm.name.isEmpty ? item.lastPathComponent : fm.name

            let info = SkillInfo(
                name: skillName,
                version: fm.version,
                description: fm.description,
                creator: fm.creator,
                license: fm.license,
                compatibility: fm.compatibility,
                allowedTools: fm.allowedTools,
                dirName: item.lastPathComponent,
                path: item.path,
                originalPath: item.path,
                location: location,
                scope: scope,
                provider: agent.name,
                providerLabel: label,
                isSymlink: symlinkInfo.isSymlink,
                symlinkTarget: symlinkInfo.target,
                realPath: realPath,
                tokenCount: tokens,
                warnings: warnings,
                disabled: isDisabled,
                synopsis: fm.synopsis.isEmpty ? nil : fm.synopsis,
                authors: fm.authors,
                keywords: fm.keywords,
                fileCount: countFiles(in: item),
                tier: fm.tier.isEmpty ? nil : fm.tier,
                governance: fm.governance.galMinimum != nil || !fm.governance.proofTypes.isEmpty ? fm.governance : nil,
                policy: fm.policy.sandbox != "recommended" ? fm.policy : nil,
                supplyChain: fm.supplyChain.repo != nil ? fm.supplyChain : nil,
                ontology: !fm.ontology.subjects.isEmpty || !fm.ontology.triggers.isEmpty ? fm.ontology : nil,
                runtime: fm.runtime.minTier != nil ? fm.runtime : nil,
                capabilities: fm.capabilities.requiresNetwork || fm.capabilities.requiresGpu ? fm.capabilities : nil,
                effort: fm.effort
            )
            results.append(info)
        }
    }

    private struct SymlinkInfo {
        var isSymlink: Bool
        var target: String?
        var realPath: String?
    }

    private func resolveSymlink(_ url: URL) -> SymlinkInfo {
        guard let attrs = try? fileManager.attributesOfItem(atPath: url.path),
              attrs[.type] as? FileAttributeType == .typeSymbolicLink else {
            return SymlinkInfo(isSymlink: false, target: nil, realPath: url.path)
        }
        let target = try? fileManager.destinationOfSymbolicLink(atPath: url.path)
        let real = try? URL(fileURLWithPath: url.path).resolvingSymlinksInPath().path
        return SymlinkInfo(isSymlink: true, target: target, realPath: real)
    }

    private func countFiles(in url: URL) -> Int {
        guard let enumerator = fileManager.enumerator(
            at: url, includingPropertiesForKeys: [.isRegularFileKey], options: [.skipsHiddenFiles]
        ) else { return 0 }
        var count = 0
        for case let file as URL in enumerator {
            if (try? file.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true {
                count += 1
            }
        }
        return count
    }
}
