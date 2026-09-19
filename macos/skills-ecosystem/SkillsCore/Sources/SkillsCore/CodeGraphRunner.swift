import Foundation

public struct CodeGraphRunner {
    private let fileManager = FileManager.default
    private let logger: (String) -> Void
    
    public init(logger: @escaping (String) -> Void) {
        self.logger = logger
    }
    
    private struct ProcessResult {
        let exitCode: Int32
        let stdout: String
        let stderr: String
    }
    
    private func findExecutable(_ name: String) -> String? {
        let env = ProcessInfo.processInfo.environment
        if let pathVar = env["PATH"] {
            let paths = pathVar.components(separatedBy: ":")
            for path in paths {
                let potentialURL = URL(fileURLWithPath: path).appendingPathComponent(name)
                if fileManager.isExecutableFile(atPath: potentialURL.path) {
                    return potentialURL.path
                }
            }
        }
        
        let homeDir = NSHomeDirectory()
        var commonPaths = [
            "/usr/local/bin/\(name)",
            "/opt/homebrew/bin/\(name)"
        ]
        
        // Glob-based dynamic NVM node version search
        let nvmNodeVersionsDir = URL(fileURLWithPath: homeDir).appendingPathComponent(".nvm/versions/node")
        if let contents = try? fileManager.contentsOfDirectory(at: nvmNodeVersionsDir, includingPropertiesForKeys: nil, options: [.skipsHiddenFiles]) {
            let sortedVersions = contents.map { $0.lastPathComponent }.sorted(by: { $0.compare($1, options: .numeric) == .orderedDescending })
            for version in sortedVersions {
                let nodeBinPath = nvmNodeVersionsDir.appendingPathComponent(version).appendingPathComponent("bin/\(name)")
                if fileManager.isExecutableFile(atPath: nodeBinPath.path) {
                    commonPaths.append(nodeBinPath.path)
                    break
                }
            }
        }
        
        
        for path in commonPaths {
            if fileManager.isExecutableFile(atPath: path) {
                return path
            }
        }
        
        return nil
    }
    
    private func runProcess(executable: String, arguments: [String], currentDirectory: URL, environment: [String: String]) -> ProcessResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: executable)
        process.arguments = arguments
        process.currentDirectoryURL = currentDirectory
        process.environment = environment
        
        let outPipe = Pipe()
        let errPipe = Pipe()
        process.standardOutput = outPipe
        process.standardError = errPipe
        
        do {
            try process.run()
            process.waitUntilExit()
            
            let outData = outPipe.fileHandleForReading.readDataToEndOfFile()
            let errData = errPipe.fileHandleForReading.readDataToEndOfFile()
            
            let stdout = String(data: outData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            let stderr = String(data: errData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            
            return ProcessResult(exitCode: process.terminationStatus, stdout: stdout, stderr: stderr)
        } catch {
            return ProcessResult(exitCode: -1, stdout: "", stderr: error.localizedDescription)
        }
    }
    
    public func runCodeGraph(at root: URL) {
        let env = ProcessInfo.processInfo.environment
        
        guard let codegraphPath = findExecutable("codegraph") else {
            logger("  ⚠ codegraph not on PATH; skipping index refresh")
            logger("      install with: npx @colbymchenry/codegraph install --location=global")
            return
        }
        
        let dotCodegraph = root.appendingPathComponent(".codegraph")
        if !fileManager.fileExists(atPath: dotCodegraph.path) {
            logger("  ✓ Initializing CodeGraph index in \(root.path)...")
            let initRes = runProcess(executable: codegraphPath, arguments: ["init"], currentDirectory: root, environment: env)
            if initRes.exitCode != 0 {
                logger("  ⚠ codegraph init returned code \(initRes.exitCode): \(initRes.stderr)")
            }
        }
        
        logger("  ✓ Indexing CodeGraph project...")
        let syncRes = runProcess(executable: codegraphPath, arguments: ["index"], currentDirectory: root, environment: env)
        if syncRes.exitCode == 0 {
            logger("  ✓ CodeGraph index refreshed")
        } else {
            logger("  ✗ CodeGraph index failed with exit code \(syncRes.exitCode): \(syncRes.stderr)")
        }
    }
}
