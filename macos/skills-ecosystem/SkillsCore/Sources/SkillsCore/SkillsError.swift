import Foundation

/// CLIENT-SPEC.md §5: Unified error taxonomy.
/// SkillsError is the local Swift error enum. For network/API errors,
/// use ClientError (from Generated/ClientModel.swift) and render via clientHint.macos.
public enum SkillsError: LocalizedError {
    case rootDirectoryMissing(path: String)
    case noSkillsFound(path: String)
    case skillAlreadyExists(name: String, path: String)
    case invalidSourceDirectory(path: String)
    case codeGraphFailed(exitCode: Int32, message: String)
    case geminiApiKeyMissing
    case llmRequestFailed(reason: String)
    case parsingFailed(reason: String)
    case forbiddenTupleViolation(skillName: String, reason: String)
    /// Wraps a canonical ClientError for macOS rendering.
    case clientError(ClientError)
    case custom(String)

    public var errorDescription: String? {
        switch self {
        case .rootDirectoryMissing(let path):
            return "\(path) does not exist. Create it and add your skills first."
        case .noSkillsFound(let path):
            return "No skills found under \(path)"
        case .skillAlreadyExists(let name, let path):
            return "Skill '\(name)' already exists at \(path). Merge manually."
        case .invalidSourceDirectory(let path):
            return "Source path \(path) does not exist or is not a directory."
        case .codeGraphFailed(let exitCode, let message):
            return "CodeGraph failed (exit \(exitCode)): \(message)"
        case .geminiApiKeyMissing:
            return "Please set the GEMINI_API_KEY environment variable to use the skill crafting feature."
        case .llmRequestFailed(let reason):
            return "Gemini API request failed: \(reason)"
        case .parsingFailed(let reason):
            return "Failed to parse crafted skill output: \(reason)"
        case .forbiddenTupleViolation(let skillName, let reason):
            return "Skill '\(skillName)' violates CLAUDE.md §10 forbidden-tuple rule: \(reason)"
        case .clientError(let clientError):
            return clientError.message
        case .custom(let reason):
            return reason
        }
    }

    /// CLIENT-SPEC.md §5.2: Render per-client hint for macOS.
    public var macosHint: String? {
        switch self {
        case .clientError(let ce):
            return ce.clientHint?.macos
        default:
            return nil
        }
    }
}
