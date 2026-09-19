import Foundation

public struct SkillCrafter {
    private let fileManager = FileManager.default
    private let skillsRoot: URL
    private let apiKey: String
    private let session: URLSession
    private let logger: (String) -> Void
    
    public init(skillsRoot: URL, apiKey: String, session: URLSession = .shared, logger: @escaping (String) -> Void) {
        self.skillsRoot = skillsRoot
        self.apiKey = apiKey
        self.session = session
        self.logger = logger
    }
    
    private struct CraftedSkillJSON: Decodable {
        let skillName: String
        let description: String
        let skillMarkdown: String
    }
    
    public func craftSkill(from targetPath: String) throws -> String {
        let targetURL = URL(fileURLWithPath: (targetPath as NSString).expandingTildeInPath)
        
        // 1. Guard Target Path
        try IPGuard.guardPath(targetURL)
        try IPGuard.scanForThales(in: targetURL)
        
        var isDir: ObjCBool = false
        guard fileManager.fileExists(atPath: targetURL.path, isDirectory: &isDir), isDir.boolValue else {
            throw SkillsError.invalidSourceDirectory(path: targetURL.path)
        }
        
        logger("\n=== crafting skill candidate from directory: \(targetURL.path) ===")
        
        // 2. Index target repo with CodeGraph
        let cgRunner = CodeGraphRunner(logger: logger)
        cgRunner.runCodeGraph(at: targetURL)
        
        // 3. Extract metadata
        var files: [String] = []
        var nodes: [String] = []
        
        let dbURL = targetURL.appendingPathComponent(".codegraph/codegraph.db")
        if fileManager.fileExists(atPath: dbURL.path) {
            let queryRunner = CodeGraphQueryRunner(logger: logger)
            let metadata = queryRunner.queryMetadata(dbURL: dbURL)
            files = metadata.files
            nodes = metadata.nodes
        } else {
            logger("  ⚠ CodeGraph database not found at \(dbURL.path); falling back to file tree scan...")
            // Fallback directory scan
            let resourceKeys: [URLResourceKey] = [.fileSizeKey, .isDirectoryKey]
            if let enumerator = fileManager.enumerator(at: targetURL, includingPropertiesForKeys: resourceKeys, options: [.skipsHiddenFiles]) {
                for case let fileURL as URL in enumerator {
                    let relPath = fileURL.path.replacingOccurrences(of: targetURL.path + "/", with: "")
                    let depth = relPath.components(separatedBy: "/").count
                    if depth > 3 {
                        enumerator.skipDescendants()
                        continue
                    }
                    
                    var isItemDir: ObjCBool = false
                    if fileManager.fileExists(atPath: fileURL.path, isDirectory: &isItemDir) {
                        if isItemDir.boolValue {
                            files.append("- Directory: \(relPath)")
                        } else {
                            if let resourceValues = try? fileURL.resourceValues(forKeys: [.fileSizeKey]),
                               let fileSize = resourceValues.fileSize {
                                files.append("- File: \(relPath) (Size: \(fileSize) bytes)")
                            }
                        }
                    }
                    
                    if files.count > 100 { break }
                }
            }
        }
        
        // 4. Build prompt
        var prompt = """
        You are an expert software engineer and AI skill architect. Your goal is to analyze the following target codebase context and craft a comprehensive, production-grade agent instruction skill (SKILL.md) for it.
        
        Here is the context of the target repository:
        Path: \(targetURL.path)
        """
        
        if !files.isEmpty {
            prompt += "\n\n=== FILE LIST ===\n" + files.prefix(40).joined(separator: "\n")
        }
        if !nodes.isEmpty {
            prompt += "\n\n=== CODE SYMBOLS AND STRUCTURE ===\n" + nodes.prefix(50).joined(separator: "\n")
        }
        
        prompt += """
        
        Please generate a structured JSON response containing:
        1. "skillName": A concise, lowercase, hyphen-separated name for the skill based on the repository content (e.g., "swiftui-layout-expert", "grpc-service-handler").
        2. "description": A high-level, clear 1-2 sentence description of this skill, used in the YAML frontmatter.
        3. "skillMarkdown": A detailed, beautiful markdown representation of the skill guidelines, instructions, rules, patterns, references, and directories that an AI agent should follow when working in or with this repository.
        
        The JSON response MUST match this exact schema:
        {
          "skillName": "string",
          "description": "string",
          "skillMarkdown": "string"
        }
        
        Ensure the markdown is thorough, practical, and highly aligned with the target codebase conventions. Do not use generic placeholders. Specify the actual languages, framework versions, directory paths, and architecture principles identified from the files and symbols.
        """
        
        // 5. Invoke LLM (On-device with Cloud fallback)
        let appleClient = AppleFoundationModelsClient(logger: logger)
        let rawResponse: String
        if appleClient.isAvailable {
            logger("  ✓ [Apple Intelligence] On-device foundation model is available.")
            rawResponse = try appleClient.generateContent(prompt: prompt)
        } else {
            logger("  · [Apple Intelligence] On-device model unavailable; falling back to Cloud Gemini API...")
            let client = LLMClient(apiKey: apiKey, session: session, logger: logger)
            rawResponse = try client.generateContent(prompt: prompt)
        }
        
        // 6. Parse and Clean response
        let cleaned = cleanJSONString(rawResponse)
        guard let data = cleaned.data(using: .utf8),
              let response = try? JSONDecoder().decode(CraftedSkillJSON.self, from: data) else {
            logger("  ✗ JSON response parsing failed.")
            throw SkillsError.parsingFailed(reason: "API did not return JSON matching schema. Raw: \(rawResponse)")
        }
        
        let skillName = response.skillName.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        
        // 7. Enforce IPGuard on generated name
        try IPGuard.guardSkillName(skillName)
        
        // 8. Register under candidates
        let candidatesURL = skillsRoot.appendingPathComponent("candidates")
        let candidateSkillURL = candidatesURL.appendingPathComponent(skillName)
        
        try fileManager.createDirectory(at: candidateSkillURL, withIntermediateDirectories: true, attributes: nil)
        
        let skillMdURL = candidateSkillURL.appendingPathComponent("SKILL.md")
        let skillContent = """
        ---
        description: "\(response.description.replacingOccurrences(of: "\"", with: "\\\""))"
        ---
        \(response.skillMarkdown)
        """
        
        try skillContent.write(to: skillMdURL, atomically: true, encoding: .utf8)
        
        logger("  ✓ Registered crafted skill candidate: \(skillName)")
        logger("  ✓ Saved under candidates store: \(skillMdURL.path)")
        
        return skillName
    }
    
    private func cleanJSONString(_ raw: String) -> String {
        var cleaned = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        if cleaned.hasPrefix("```json") {
            cleaned = String(cleaned.dropFirst(7))
        } else if cleaned.hasPrefix("```") {
            cleaned = String(cleaned.dropFirst(3))
        }
        if cleaned.hasSuffix("```") {
            cleaned = String(cleaned.dropLast(3))
        }
        return cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
