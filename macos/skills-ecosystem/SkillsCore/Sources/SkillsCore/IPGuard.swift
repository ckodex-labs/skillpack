import Foundation

public enum IPGuardError: LocalizedError {
    case boundaryViolation(String)
    
    public var errorDescription: String? {
        switch self {
        case .boundaryViolation(let message):
            return "REFUSED: \(message)"
        }
    }
}

public struct IPGuard {
    private static let thalesPattern = "thales|cortaix-csr|ppt-thales|ip-pending"
    
    private static let thalesRegex: NSRegularExpression = {
        try! NSRegularExpression(pattern: thalesPattern, options: .caseInsensitive)
    }()
    
    public static func guardPath(_ url: URL) throws {
        let path = url.path
        let range = NSRange(location: 0, length: path.utf16.count)
        if thalesRegex.firstMatch(in: path, options: [], range: range) != nil {
            throw IPGuardError.boundaryViolation("'\(path)' matches Thales/IP-pending pattern. Resolve IP counsel clearance and use a different path.")
        }
    }
    
    public static func guardSkillName(_ name: String) throws {
        let range = NSRange(location: 0, length: name.utf16.count)
        if thalesRegex.firstMatch(in: name, options: [], range: range) != nil {
            throw IPGuardError.boundaryViolation("Skill '\(name)' matches Thales pattern. Remove from canonical store.")
        }
    }
    
    public static func scanForThales(in rootURL: URL, maxDepth: Int = 3) throws {
        let fileManager = FileManager.default
        let resourceKeys: [URLResourceKey] = [.isDirectoryKey]
        
        guard let enumerator = fileManager.enumerator(
            at: rootURL,
            includingPropertiesForKeys: resourceKeys,
            options: [.skipsHiddenFiles],
            errorHandler: { _, _ in false }
        ) else { return }
        
        var hits: [String] = []
        
        for case let fileURL as URL in enumerator {
            let relativePath = fileURL.path.replacingOccurrences(of: rootURL.path, with: "")
            let depth = relativePath.split(separator: "/").count
            if depth > maxDepth {
                enumerator.skipDescendants()
                continue
            }
            
            let pathStr = fileURL.lastPathComponent
            let range = NSRange(location: 0, length: pathStr.utf16.count)
            if thalesRegex.firstMatch(in: pathStr, options: [], range: range) != nil {
                hits.append(fileURL.path)
            }
        }
        
        if !hits.isEmpty {
            let hitsString = hits.joined(separator: "\n         ")
            throw IPGuardError.boundaryViolation("\(rootURL.path) contains Thales/IP-pending paths:\n         \(hitsString)\n         move them out before syncing.")
        }
    }
}
