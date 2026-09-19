import Foundation

/// Run a process and return trimmed stdout, throwing on non-zero exit.
@discardableResult
func shell(_ args: String...) throws -> String {
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    proc.arguments = args
    let pipe = Pipe()
    proc.standardOutput = pipe
    proc.standardError = Pipe()
    try proc.run()
    proc.waitUntilExit()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    let output = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
    guard proc.terminationStatus == 0 else {
        throw NSError(domain: "ShellHelper", code: Int(proc.terminationStatus),
                      userInfo: [NSLocalizedDescriptionKey: "Command failed: \(args.joined(separator: " "))"])
    }
    return output
}

/// Build a machine-readable JSON envelope (v1), matching asm's --machine format.
func machineEnvelope(data: String, command: String) -> String {
    let ts = ISO8601DateFormatter().string(from: Date())
    return """
    {
      "v": 1,
      "command": "\(command)",
      "timestamp": "\(ts)",
      "data": \(data)
    }
    """
}
