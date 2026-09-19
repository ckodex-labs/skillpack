import Foundation

/// Mirrors asm's `SkillStateFile` and `skill-state.ts`.
/// Persists disabled-skill state to ~/.config/agent-skill-manager/skill-state.json
/// and renames SKILL.md ↔ SKILL.md.disabled on disk.
public struct SkillStateManager {

    private static let configDir: URL = {
        let home = FileManager.default.homeDirectoryForCurrentUser
        return home.appendingPathComponent(".config/agent-skill-manager")
    }()

    private static var stateFile: URL {
        configDir.appendingPathComponent("skill-state.json")
    }

    public struct StateFile: Codable {
        public var version: Int
        /// dirName → provider → scope → true
        public var disabled: [String: [String: [String: Bool]]]

        public init() {
            self.version = 1
            self.disabled = [:]
        }
    }

    public init() {}

    // MARK: - Load / Save

    public func loadState() -> StateFile {
        guard let data = try? Data(contentsOf: Self.stateFile),
              let state = try? JSONDecoder().decode(StateFile.self, from: data) else {
            return StateFile()
        }
        return state
    }

    public func saveState(_ state: StateFile) throws {
        let fm = FileManager.default
        try fm.createDirectory(at: Self.configDir, withIntermediateDirectories: true)
        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try enc.encode(state)
        try data.write(to: Self.stateFile)
    }

    // MARK: - Disable

    /// Rename SKILL.md → SKILL.md.disabled and persist state.
    /// `targets` is a list of SkillInfo to disable.
    public func disable(skills: [SkillInfo]) throws -> [String] {
        let fm = FileManager.default
        var state = loadState()
        var affected: [String] = []

        for skill in skills {
            let mdPath = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md")
            let disabledPath = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md.disabled")

            if fm.fileExists(atPath: mdPath.path) {
                try fm.moveItem(at: mdPath, to: disabledPath)
            }

            // Record in state
            if state.disabled[skill.dirName] == nil { state.disabled[skill.dirName] = [:] }
            if state.disabled[skill.dirName]![skill.provider] == nil {
                state.disabled[skill.dirName]![skill.provider] = [:]
            }
            state.disabled[skill.dirName]![skill.provider]![skill.scope] = true
            affected.append("\(skill.provider)/\(skill.dirName)")
        }

        try saveState(state)
        return affected
    }

    // MARK: - Enable

    /// Rename SKILL.md.disabled → SKILL.md and clear state.
    public func enable(skills: [SkillInfo]) throws -> [String] {
        let fm = FileManager.default
        var state = loadState()
        var affected: [String] = []

        for skill in skills {
            let mdPath = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md")
            let disabledPath = URL(fileURLWithPath: skill.path).appendingPathComponent("SKILL.md.disabled")

            if fm.fileExists(atPath: disabledPath.path) {
                try fm.moveItem(at: disabledPath, to: mdPath)
            }

            // Clear from state
            state.disabled[skill.dirName]?[skill.provider]?.removeValue(forKey: skill.scope)
            if state.disabled[skill.dirName]?[skill.provider]?.isEmpty == true {
                state.disabled[skill.dirName]?.removeValue(forKey: skill.provider)
            }
            if state.disabled[skill.dirName]?.isEmpty == true {
                state.disabled.removeValue(forKey: skill.dirName)
            }
            affected.append("\(skill.provider)/\(skill.dirName)")
        }

        try saveState(state)
        return affected
    }

    // MARK: - Query

    public func isDisabled(dirName: String, provider: String, scope: String) -> Bool {
        let state = loadState()
        return state.disabled[dirName]?[provider]?[scope] == true
    }
}
