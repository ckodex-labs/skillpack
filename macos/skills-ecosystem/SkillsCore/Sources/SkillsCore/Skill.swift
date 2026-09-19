import Foundation
import CryptoKit

/// Lifecycle status values parsed from SKILL.md frontmatter `lifecycle:` key.
/// Used by T-04 forbidden-tuple checks in SyncManager.
public enum SkillLifecycleStatus: String {
    case active
    case deprecated
    case retired
    case experimental
    case unknown
}

public struct Skill {
    public let name: String
    public let url: URL
    public let sizeBytes: Int
    public let description: String
    /// ASC pattern array parsed from SKILL.md frontmatter `asc:` key (e.g. ["anti", "execute"]).
    public let asc: [String]
    /// Lifecycle status parsed from SKILL.md frontmatter `lifecycle:` key.
    public let lifecycleStatus: SkillLifecycleStatus
    /// T-02: Semantic version string parsed from SKILL.md frontmatter `version:` key.
    public let version: String?
    /// T-02: SHA-256 hex digest of the SKILL.md file content, prefixed "sha256:".
    public let contentHash: String?
    /// T-02: Deployment tier parsed from SKILL.md frontmatter `tier:` key.
    public let tier: String?
    /// T-02: Ontology URN parsed from SKILL.md frontmatter `ontology_ref:` key.
    public let ontologyRef: String?

    public init(url: URL) {
        self.url = url
        self.name = url.lastPathComponent

        // Calculate size of directory
        self.sizeBytes = Self.calculateSize(of: url)

        // Parse all frontmatter fields from SKILL.md
        let skillMdURL = url.appendingPathComponent("SKILL.md")
        let parsed = Self.parseFrontmatter(from: skillMdURL)
        self.description = parsed.description
        self.asc = parsed.asc
        self.lifecycleStatus = parsed.lifecycleStatus
        self.version = parsed.version
        self.tier = parsed.tier
        self.ontologyRef = parsed.ontologyRef

        // T-02: SHA-256 of SKILL.md content
        if let data = try? Data(contentsOf: skillMdURL) {
            let digest = CryptoKit.SHA256.hash(data: data)
            self.contentHash = "sha256:" + digest.map { String(format: "%02x", $0) }.joined()
        } else {
            self.contentHash = nil
        }
    }
    
    public static func calculateSize(of url: URL) -> Int {
        let fileManager = FileManager.default
        guard let enumerator = fileManager.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey]) else { return 0 }
        
        var totalSize = 0
        for case let fileURL as URL in enumerator {
            if let resourceValues = try? fileURL.resourceValues(forKeys: [.fileSizeKey]),
               let fileSize = resourceValues.fileSize {
                totalSize += fileSize
            }
        }
        return totalSize
    }
    
    private struct FrontmatterResult {
        var description: String
        var asc: [String]
        var lifecycleStatus: SkillLifecycleStatus
        var version: String?
        var tier: String?
        var ontologyRef: String?
    }

    /// Parse YAML frontmatter from SKILL.md extracting description, asc, lifecycle, version, tier, and ontologyRef.
    private static func parseFrontmatter(from url: URL) -> FrontmatterResult {
        var result = FrontmatterResult(description: "", asc: [], lifecycleStatus: .unknown, version: nil, tier: nil, ontologyRef: nil)
        guard let content = try? String(contentsOf: url, encoding: .utf8) else {
            return result
        }

        let lines = content.components(separatedBy: .newlines)
        var inFrontmatter = false
        var frontmatterClosed = false
        var gotDescription = false

        for line in lines {
            if line.trimmingCharacters(in: .whitespaces) == "---" {
                if !inFrontmatter {
                    inFrontmatter = true
                } else {
                    frontmatterClosed = true
                    inFrontmatter = false
                }
                continue
            }

            if inFrontmatter {
                if line.starts(with: "description:") {
                    let raw = line.replacingOccurrences(of: "description:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "\"", with: "")
                    result.description = String(raw.prefix(200))
                        .replacingOccurrences(of: "\"", with: "\\\"")
                        .replacingOccurrences(of: "\r", with: "")
                    gotDescription = true
                } else if line.starts(with: "asc:") {
                    // Inline array: asc: [anti, execute] or asc: ["anti","execute"]
                    let raw = line.replacingOccurrences(of: "asc:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "[", with: "")
                                  .replacingOccurrences(of: "]", with: "")
                                  .replacingOccurrences(of: "\"", with: "")
                    let tokens = raw.components(separatedBy: ",")
                        .map { $0.trimmingCharacters(in: .whitespaces).lowercased() }
                        .filter { !$0.isEmpty }
                    result.asc = tokens
                } else if line.starts(with: "lifecycle:") {
                    let raw = line.replacingOccurrences(of: "lifecycle:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "\"", with: "")
                                  .lowercased()
                    result.lifecycleStatus = SkillLifecycleStatus(rawValue: raw) ?? .unknown
                } else if line.starts(with: "version:") {
                    let raw = line.replacingOccurrences(of: "version:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "\"", with: "")
                    result.version = raw.isEmpty ? nil : raw
                } else if line.starts(with: "tier:") {
                    let raw = line.replacingOccurrences(of: "tier:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "\"", with: "")
                                  .lowercased()
                    result.tier = raw.isEmpty ? nil : raw
                } else if line.starts(with: "ontology_ref:") {
                    let raw = line.replacingOccurrences(of: "ontology_ref:", with: "")
                                  .trimmingCharacters(in: .whitespaces)
                                  .replacingOccurrences(of: "\"", with: "")
                    result.ontologyRef = raw.isEmpty ? nil : raw
                }
            } else if frontmatterClosed && !gotDescription && !line.isEmpty && !line.starts(with: "#") {
                result.description = String(line.trimmingCharacters(in: .whitespaces).prefix(200))
                    .replacingOccurrences(of: "\"", with: "\\\"")
                    .replacingOccurrences(of: "\r", with: "")
            }
        }

        return result
    }
}
