import Foundation

public struct FoundationDBClient {
    private let fdbcliPath: String
    private let logger: (String) -> Void
    
    public init(fdbcliPath: String = "/usr/local/bin/fdbcli", logger: @escaping (String) -> Void = { _ in }) {
        self.fdbcliPath = fdbcliPath
        self.logger = logger
    }
    
    @discardableResult
    private func runFDBCommand(_ command: String) throws -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: fdbcliPath)
        process.arguments = ["--exec", command]
        
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        
        try process.run()
        process.waitUntilExit()
        
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: data, encoding: .utf8) else {
            throw SkillsError.custom("Failed to read output from fdbcli")
        }
        
        if process.terminationStatus != 0 {
            logger("  ✗ fdbcli failed with code \(process.terminationStatus): \(output)")
            throw SkillsError.custom("fdbcli command failed with code \(process.terminationStatus)")
        }
        
        return output
    }
    
    public func set(key: String, value: String) throws {
        // Guard input to prevent any command injection/parsing issues
        guard key.range(of: "^[a-zA-Z0-9_/.-]+$", options: .regularExpression) != nil else {
            throw SkillsError.custom("Invalid characters in FoundationDB key: \(key)")
        }
        
        let b64Value = Data(value.utf8).base64EncodedString()
        let command = "writemode on; set \(key) \(b64Value)"
        try runFDBCommand(command)
    }
    
    public func get(key: String) throws -> String? {
        guard key.range(of: "^[a-zA-Z0-9_/.-]+$", options: .regularExpression) != nil else {
            throw SkillsError.custom("Invalid characters in FoundationDB key: \(key)")
        }
        
        let output = try runFDBCommand("get \(key)")
        let lines = output.components(separatedBy: .newlines)
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.contains("not found") {
                return nil
            }
            if trimmed.contains(" is ") {
                let parts = trimmed.components(separatedBy: " is ")
                if parts.count == 2 {
                    let cleanedVal = cleanPart(parts[1])
                    if let decodedData = Data(base64Encoded: cleanedVal),
                       let decodedString = String(data: decodedData, encoding: .utf8) {
                        return decodedString
                    }
                }
            }
        }
        return nil
    }
    
    public func clear(key: String) throws {
        guard key.range(of: "^[a-zA-Z0-9_/.-]+$", options: .regularExpression) != nil else {
            throw SkillsError.custom("Invalid characters in FoundationDB key: \(key)")
        }
        let command = "writemode on; clear \(key)"
        try runFDBCommand(command)
    }
    
    public func getrange(prefix: String) throws -> [String: String] {
        guard prefix.range(of: "^[a-zA-Z0-9_/.-]+$", options: .regularExpression) != nil else {
            throw SkillsError.custom("Invalid characters in FoundationDB prefix: \(prefix)")
        }
        
        let output = try runFDBCommand("getrange \(prefix)")
        let lines = output.components(separatedBy: .newlines)
        var result: [String: String] = [:]
        
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.hasPrefix("`") && trimmed.contains(" is ") {
                let parts = trimmed.components(separatedBy: " is ")
                if parts.count == 2 {
                    let cleanedKey = cleanPart(parts[0])
                    let cleanedVal = cleanPart(parts[1])
                    if let decodedData = Data(base64Encoded: cleanedVal),
                       let decodedString = String(data: decodedData, encoding: .utf8) {
                        result[cleanedKey] = decodedString
                    }
                }
            }
        }
        return result
    }
    
    public func checkHealth() -> Bool {
        do {
            _ = try runFDBCommand("get healthcheck")
            return true
        } catch {
            return false
        }
    }
    
    private func cleanPart(_ s: String) -> String {
        var temp = s.trimmingCharacters(in: .whitespacesAndNewlines)
        if temp.hasPrefix("`") { temp.removeFirst() }
        if temp.hasSuffix("'") { temp.removeLast() }
        return temp
    }
}
