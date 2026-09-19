import Foundation
import GRPC
import NIO
import SkillsCore

// MARK: - Daemon IPC Client with XPC → gRPC → Local fallback chain

/// Queries the daemon's current status.
/// Falls back: XPC (native, fast) → gRPC (cross-platform) → local scan (daemon offline).
func queryDaemonStatus(skillsRoot: String) -> (root: String, skillCount: Int, agents: [String], transport: String)? {
    // 1. Try XPC (fastest, native macOS)
    if let result = queryStatusViaXPC() {
        return (result.root, result.skillCount, result.agents, "XPC")
    }

    // 2. Try gRPC (daemon running but XPC unavailable)
    if let result = queryStatusViaGRPC() {
        return (result.root, result.skillCount, result.agents, "gRPC")
    }

    // 3. Fall back to local scan (daemon offline entirely)
    return nil
}

/// Triggers a sync via XPC → gRPC → local fallback.
/// Returns (success, message, skillsCount, transport).
func triggerDaemonSync(
    dryRun: Bool,
    noIndex: Bool,
    onlyAgent: String?
) -> (success: Bool, message: String, count: Int, transport: String)? {
    // 1. XPC
    if let result = syncViaXPC(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent) {
        return (result.success, result.message, result.count, "XPC")
    }
    // 2. gRPC
    if let result = syncViaGRPC(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent) {
        return (result.success, result.message, result.count, "gRPC")
    }
    return nil
}

// MARK: - XPC Helpers

private struct XPCStatusResult {
    let root: String
    let skillCount: Int
    let agents: [String]
}

private func queryStatusViaXPC() -> XPCStatusResult? {
    let connection = NSXPCConnection(machServiceName: "com.ckodex.skillsdaemon", options: [])
    connection.remoteObjectInterface = NSXPCInterface(with: SkillsDaemonXPCProtocol.self)
    connection.resume()
    defer { connection.invalidate() }

    var result: XPCStatusResult?
    let semaphore = DispatchSemaphore(value: 0)

    guard let proxy = connection.remoteObjectProxyWithErrorHandler({ _ in
        semaphore.signal()
    }) as? SkillsDaemonXPCProtocol else {
        return nil
    }

    proxy.getStatus { root, count, agents in
        result = XPCStatusResult(root: root, skillCount: Int(count), agents: agents)
        semaphore.signal()
    }

    let timeout = DispatchTime.now() + .seconds(3)
    if semaphore.wait(timeout: timeout) == .timedOut {
        return nil
    }
    return result
}

private struct XPCSyncResult {
    let success: Bool
    let message: String
    let count: Int
}

private func syncViaXPC(dryRun: Bool, noIndex: Bool, onlyAgent: String?) -> XPCSyncResult? {
    let connection = NSXPCConnection(machServiceName: "com.ckodex.skillsdaemon", options: [])
    connection.remoteObjectInterface = NSXPCInterface(with: SkillsDaemonXPCProtocol.self)
    connection.resume()
    defer { connection.invalidate() }

    var result: XPCSyncResult?
    let semaphore = DispatchSemaphore(value: 0)

    guard let proxy = connection.remoteObjectProxyWithErrorHandler({ _ in
        semaphore.signal()
    }) as? SkillsDaemonXPCProtocol else {
        return nil
    }

    proxy.syncSkills(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent) { success, message, count in
        result = XPCSyncResult(success: success, message: message, count: Int(count))
        semaphore.signal()
    }

    let timeout = DispatchTime.now() + .seconds(30)
    if semaphore.wait(timeout: timeout) == .timedOut {
        return nil
    }
    return result
}

// MARK: - gRPC Helpers

private func makeGRPCChannel() -> GRPCChannel? {
    let group = PlatformSupport.makeEventLoopGroup(loopCount: 1)
    let channel = try? GRPCChannelPool.with(
        target: .hostAndPort("127.0.0.1", 50051),
        transportSecurity: .plaintext,
        eventLoopGroup: group
    )
    return channel
}

private func queryStatusViaGRPC() -> XPCStatusResult? {
    guard let channel = makeGRPCChannel() else { return nil }
    defer { try? channel.close().wait() }

    let client = Skills_SkillsServiceNIOClient(channel: channel)
    let request = Skills_StatusRequest()

    do {
        let response = try client.getStatus(request).response.wait()
        return XPCStatusResult(
            root: response.canonicalRoot,
            skillCount: Int(response.totalSkills),
            agents: response.activeAgents
        )
    } catch {
        return nil
    }
}

private func syncViaGRPC(dryRun: Bool, noIndex: Bool, onlyAgent: String?) -> XPCSyncResult? {
    guard let channel = makeGRPCChannel() else { return nil }
    defer { try? channel.close().wait() }

    let client = Skills_SkillsServiceNIOClient(channel: channel)
    var request = Skills_SyncRequest()
    request.dryRun = dryRun
    request.noIndex = noIndex
    request.onlyAgent = onlyAgent ?? ""

    do {
        let response = try client.syncSkills(request).response.wait()
        return XPCSyncResult(
            success: response.success,
            message: response.message,
            count: Int(response.skillsProcessed)
        )
    } catch {
        return nil
    }
}
