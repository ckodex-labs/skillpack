import Foundation
import SkillsCore

// SkillsUI talks to the daemon over XPC only and falls back to local SyncManager.
// The gRPC fallback path used by SkillsCLI is intentionally omitted here:
//   - keeps SkillsUI free of grpc-swift / swift-nio dependencies
//   - XPC is the fast native macOS path; if it's down, gRPC almost always is too
//   - on miss, we report transport="local" so the UI is honest about it

struct DaemonStatus {
    let root: String
    let skillCount: Int
    let agents: [String]
    let transport: String        // "XPC" or "local"
    let storageBackend: String   // "local" or "distributed"
    let fdbStatus: String        // "online", "offline", "n/a"
    let rustFSStatus: String     // "online", "offline", "n/a"
}

struct DaemonSyncResult {
    let success: Bool
    let message: String
    let count: Int
    let transport: String        // "XPC" or "local"
}

enum MenuBarDaemonClient {

    static func queryStatus(skillsRoot: String) -> DaemonStatus? {
        if let xpc = queryStatusViaExtendedXPC() {
            return DaemonStatus(
                root: xpc.root,
                skillCount: xpc.skillCount,
                agents: xpc.agents,
                transport: "XPC",
                storageBackend: xpc.storageBackend,
                fdbStatus: xpc.fdbStatus,
                rustFSStatus: xpc.rustFSStatus
            )
        }
        return nil
    }

    static func triggerSync(dryRun: Bool, noIndex: Bool, onlyAgent: String?) -> DaemonSyncResult? {
        if let xpc = syncViaXPC(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent) {
            return DaemonSyncResult(success: xpc.success, message: xpc.message, count: xpc.count, transport: "XPC")
        }
        return nil
    }
}

// MARK: - XPC

private struct XPCStatus {
    let root: String
    let skillCount: Int
    let agents: [String]
    let storageBackend: String
    let fdbStatus: String
    let rustFSStatus: String
}

private struct XPCSync {
    let success: Bool
    let message: String
    let count: Int
}

private func queryStatusViaExtendedXPC() -> XPCStatus? {
    let connection = NSXPCConnection(machServiceName: "com.ckodex.skillsdaemon", options: [])
    connection.remoteObjectInterface = NSXPCInterface(with: SkillsDaemonXPCProtocol.self)
    connection.resume()
    defer { connection.invalidate() }

    var result: XPCStatus?
    let semaphore = DispatchSemaphore(value: 0)

    guard let proxy = connection.remoteObjectProxyWithErrorHandler({ _ in
        semaphore.signal()
    }) as? SkillsDaemonXPCProtocol else {
        return nil
    }

    proxy.getExtendedStatus { root, count, agents, backend, fdb, rust in
        result = XPCStatus(
            root: root,
            skillCount: Int(count),
            agents: agents,
            storageBackend: backend,
            fdbStatus: fdb,
            rustFSStatus: rust
        )
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + .seconds(3)) == .timedOut {
        return nil
    }
    return result
}

private func syncViaXPC(dryRun: Bool, noIndex: Bool, onlyAgent: String?) -> XPCSync? {
    let connection = NSXPCConnection(machServiceName: "com.ckodex.skillsdaemon", options: [])
    connection.remoteObjectInterface = NSXPCInterface(with: SkillsDaemonXPCProtocol.self)
    connection.resume()
    defer { connection.invalidate() }

    var result: XPCSync?
    let semaphore = DispatchSemaphore(value: 0)

    guard let proxy = connection.remoteObjectProxyWithErrorHandler({ _ in
        semaphore.signal()
    }) as? SkillsDaemonXPCProtocol else {
        return nil
    }

    proxy.syncSkills(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent) { success, message, count in
        result = XPCSync(success: success, message: message, count: Int(count))
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + .seconds(30)) == .timedOut {
        return nil
    }
    return result
}
