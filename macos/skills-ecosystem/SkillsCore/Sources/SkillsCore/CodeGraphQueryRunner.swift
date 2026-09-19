import Foundation
import SQLite3

public struct CodeGraphQueryRunner {
    private let logger: (String) -> Void
    
    public init(logger: @escaping (String) -> Void = { _ in }) {
        self.logger = logger
    }
    
    public func queryMetadata(dbURL: URL) -> (files: [String], nodes: [String]) {
        let dbPath = dbURL.path
        logger("  ✓ Opening CodeGraph database at \(dbPath)...")
        
        var db: OpaquePointer?
        guard sqlite3_open(dbPath, &db) == SQLITE_OK else {
            logger("  ⚠ Failed to open CodeGraph database at \(dbPath)")
            return ([], [])
        }
        defer { sqlite3_close(db) }
        
        var files: [String] = []
        var nodes: [String] = []
        
        // 1. Query top files
        let fileQuery = "SELECT path, language, size FROM files ORDER BY size DESC LIMIT 40;"
        var stmt: OpaquePointer?
        if sqlite3_prepare_v2(db, fileQuery, -1, &stmt, nil) == SQLITE_OK {
            while sqlite3_step(stmt) == SQLITE_ROW {
                if let pathCStr = sqlite3_column_text(stmt, 0),
                   let langCStr = sqlite3_column_text(stmt, 1) {
                    let path = String(cString: pathCStr)
                    let lang = String(cString: langCStr)
                    let size = sqlite3_column_int(stmt, 2)
                    files.append("- File: \(path) (Language: \(lang), Size: \(size) bytes)")
                }
            }
            sqlite3_finalize(stmt)
        } else {
            logger("  ⚠ Failed to prepare files query")
        }
        
        // 2. Query nodes with signatures or docstrings
        let nodeQuery = """
        SELECT kind, name, file_path, language, signature, docstring 
        FROM nodes 
        WHERE (docstring IS NOT NULL AND length(docstring) > 0) OR (signature IS NOT NULL AND length(signature) > 0)
        ORDER BY length(docstring) DESC 
        LIMIT 60;
        """
        if sqlite3_prepare_v2(db, nodeQuery, -1, &stmt, nil) == SQLITE_OK {
            while sqlite3_step(stmt) == SQLITE_ROW {
                let kind = String(cString: sqlite3_column_text(stmt, 0))
                let name = String(cString: sqlite3_column_text(stmt, 1))
                let filePath = String(cString: sqlite3_column_text(stmt, 2))
                let language = String(cString: sqlite3_column_text(stmt, 3))
                
                var signature = ""
                if let sigCStr = sqlite3_column_text(stmt, 4) {
                    signature = String(cString: sigCStr)
                }
                
                var docstring = ""
                if let docCStr = sqlite3_column_text(stmt, 5) {
                    docstring = String(cString: docCStr)
                }
                
                var nodeDesc = "- \(kind) `\(name)` in \(filePath) (Language: \(language))"
                if !signature.isEmpty {
                    nodeDesc += "\n  Signature: `\(signature)`"
                }
                if !docstring.isEmpty {
                    nodeDesc += "\n  Docstring: \"\(docstring)\""
                }
                nodes.append(nodeDesc)
            }
            sqlite3_finalize(stmt)
        } else {
            logger("  ⚠ Failed to prepare nodes query")
        }
        
        logger("  ✓ Extracted \(files.count) files and \(nodes.count) code definitions from CodeGraph index")
        return (files, nodes)
    }
}
