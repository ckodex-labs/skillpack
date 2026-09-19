import Foundation
import SkillsCore
import GRPC
import NIO

@main
struct SkillsDaemon {
    static func main() {
        NSLog("SkillsDaemon: Background agent service starting up (thin client mode)...")

        let path = ProcessInfo.processInfo.environment["SKILLS_ROOT"] ?? "~/Skills/shared"
        let resolvedPath = (path as NSString).expandingTildeInPath
        let skillsRoot = URL(fileURLWithPath: resolvedPath)

        // Ensure folder exists
        try? FileManager.default.createDirectory(at: skillsRoot, withIntermediateDirectories: true, attributes: nil)

        // 1. Start FSEventStream DirectoryWatcher
        //    On change, notify the Rust skillpack-server to sync.
        NSLog("SkillsDaemon: Watching \(skillsRoot.path) for changes...")
        let watcher = DirectoryWatcher(url: skillsRoot) {
            NSLog("SkillsDaemon: Change detected in canonical store! Notifying Rust server...")
            do {
                let client = try SkillPackGRPCClient()
                let result = try client.syncAgents(dryRun: false, noIndex: false, onlyAgent: nil)
                if result.success {
                    NSLog("SkillsDaemon: Auto-sync via Rust server successful. \(result.skillsProcessed) skills processed.")
                } else {
                    NSLog("SkillsDaemon [Warn]: Rust server sync returned failure: \(result.message)")
                }
            } catch {
                NSLog("SkillsDaemon [Error]: Failed to notify Rust server: \(error.localizedDescription)")
            }
        }
        watcher.start()

        // 2. Start XPC Listener (local macOS IPC for UI and CLI)
        //    Delegates to the Rust skillpack-server instead of handling locally.
        NSLog("SkillsDaemon: Starting XPC Listener (com.ckodex.skillsdaemon)...")
        let xpcDelegate = XPCListenerDelegate(resolvedPath: resolvedPath)
        let xpcListener = NSXPCListener(machServiceName: "com.ckodex.skillsdaemon")
        xpcListener.delegate = xpcDelegate
        xpcListener.resume()

        // Run forever
        RunLoop.current.run()
    }
}

class XPCListenerDelegate: NSObject, NSXPCListenerDelegate {
    private let resolvedPath: String

    init(resolvedPath: String) {
        self.resolvedPath = resolvedPath
        super.init()
    }

    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        newConnection.exportedInterface = NSXPCInterface(with: SkillsDaemonXPCProtocol.self)
        newConnection.exportedObject = SkillsDaemonXPCService(resolvedPath: resolvedPath)
        newConnection.resume()
        NSLog("SkillsDaemonXPC: Accepted new incoming client connection.")
        return true
    }
}
