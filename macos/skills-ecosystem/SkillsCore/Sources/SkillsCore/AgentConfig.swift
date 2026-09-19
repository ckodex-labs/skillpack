import Foundation

public enum IntegrationType {
    case symlink
    case indexFile
}

public struct AgentConfig {
    public let name: String
    public let defaultPath: String
    public let fallbackPaths: [String]
    public let envOverrideKey: String
    public let integrationType: IntegrationType
    
    public init(name: String, defaultPath: String, fallbackPaths: [String] = [], envOverrideKey: String, integrationType: IntegrationType) {
        self.name = name
        self.defaultPath = defaultPath
        self.fallbackPaths = fallbackPaths
        self.envOverrideKey = envOverrideKey
        self.integrationType = integrationType
    }
    
    public var resolvedURL: URL? {
        let paths: [String]
        if let envVal = getenv(envOverrideKey) {
            let envPath = String(cString: envVal)
            paths = [envPath]
        } else {
            paths = [defaultPath] + fallbackPaths
        }
        for path in paths {
            let expanded = (path as NSString).expandingTildeInPath
            let url = URL(fileURLWithPath: expanded)
            if FileManager.default.fileExists(atPath: url.path) {
                return url
            }
        }
        // Return primary default even if not found, so callers can create it
        return URL(fileURLWithPath: (paths.first! as NSString).expandingTildeInPath)
    }
}

public struct AgentRegistry {
    public static let shared = AgentRegistry()
    
    public let agents: [AgentConfig]
    
    public let labels: [String: String]

    private init() {
        self.agents = [
            // ── Priority providers ──
            AgentConfig(name: "claude",      defaultPath: "~/.claude/skills",              envOverrideKey: "CLAUDE_SKILLS_DIR",      integrationType: .symlink),
            AgentConfig(name: "codex",       defaultPath: "~/.codex/skills",               envOverrideKey: "CODEX_SKILLS_DIR",       integrationType: .symlink),
            AgentConfig(name: "opencode",    defaultPath: "~/.config/opencode/skills",     envOverrideKey: "OPENCODE_SKILLS_DIR",    integrationType: .symlink),
            AgentConfig(name: "pi",          defaultPath: "~/.pi/skills",                  envOverrideKey: "PI_SKILLS_DIR",          integrationType: .symlink),
            AgentConfig(name: "hermes",      defaultPath: "~/.hermes/skills",              envOverrideKey: "HERMES_SKILLS_DIR",      integrationType: .symlink),
            AgentConfig(name: "openclaw",    defaultPath: "~/.openclaw/skills",            envOverrideKey: "OPENCLAW_SKILLS_DIR",    integrationType: .symlink),
            AgentConfig(name: "agents",      defaultPath: "~/.agents/skills",              envOverrideKey: "AGENTS_SKILLS_DIR",      integrationType: .symlink),
            // ── Standard providers ──
            AgentConfig(name: "cursor",      defaultPath: "~/.cursor/rules",               envOverrideKey: "CURSOR_SKILLS_DIR",      integrationType: .indexFile),
            AgentConfig(name: "copilot",     defaultPath: "~/.github/instructions",        envOverrideKey: "COPILOT_SKILLS_DIR",     integrationType: .symlink),
            AgentConfig(name: "windsurf",    defaultPath: "~/.windsurf/rules",             fallbackPaths: ["~/.codeium/windsurf/rules", "~/.windsurf/skills"],
                                             envOverrideKey: "WINDSURF_SKILLS_DIR",        integrationType: .symlink),
            AgentConfig(name: "antigravity", defaultPath: "~/.antigravity/skills",         envOverrideKey: "ANTIGRAVITY_SKILLS_DIR", integrationType: .symlink),
            AgentConfig(name: "gemini",      defaultPath: "~/.gemini/skills",              fallbackPaths: ["~/.gemini/antigravity-ide/skills", "~/.gemini/config/skills"],
                                             envOverrideKey: "GEMINI_SKILLS_DIR",          integrationType: .symlink),
            AgentConfig(name: "cline",       defaultPath: "~/Documents/Cline/Rules",       envOverrideKey: "CLINE_SKILLS_DIR",       integrationType: .symlink),
            AgentConfig(name: "roo",         defaultPath: "~/.roo/rules",                  envOverrideKey: "ROO_SKILLS_DIR",         integrationType: .symlink),
            AgentConfig(name: "continue",    defaultPath: "~/.continue/rules",             envOverrideKey: "CONTINUE_SKILLS_DIR",    integrationType: .symlink),
            AgentConfig(name: "aider",       defaultPath: "~/.aider/skills",               envOverrideKey: "AIDER_SKILLS_DIR",       integrationType: .symlink),
            AgentConfig(name: "zed",         defaultPath: "~/.config/zed/prompt_overrides",envOverrideKey: "ZED_SKILLS_DIR",         integrationType: .symlink),
            AgentConfig(name: "augment",     defaultPath: "~/.augment/rules",              envOverrideKey: "AUGMENT_SKILLS_DIR",     integrationType: .symlink),
        ]

        self.labels = [
            "claude":      "Claude Code",
            "codex":       "Codex",
            "opencode":    "OpenCode",
            "pi":          "Pi",
            "hermes":      "Hermes",
            "openclaw":    "OpenClaw",
            "agents":      "Agents",
            "cursor":      "Cursor",
            "copilot":     "GitHub Copilot",
            "windsurf":    "Windsurf",
            "antigravity": "Google Antigravity",
            "gemini":      "Gemini CLI",
            "cline":       "Cline",
            "roo":         "Roo Code",
            "continue":    "Continue",
            "aider":       "Aider",
            "zed":         "Zed",
            "augment":     "Augment",
        ]
    }
    
    public func agent(named name: String) -> AgentConfig? {
        return agents.first { $0.name.lowercased() == name.lowercased() }
    }
}
