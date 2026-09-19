import Foundation
import SkillsCore

/// XPC service that delegates all canonical store operations to the Rust skillpack-server.
@objc(SkillsDaemonXPCService)
public class SkillsDaemonXPCService: NSObject, SkillsDaemonXPCProtocol {
    private let resolvedPath: String

    public init(resolvedPath: String) {
        self.resolvedPath = resolvedPath
        super.init()
    }

    public func syncSkills(
        dryRun: Bool,
        noIndex: Bool,
        onlyAgent: String?,
        withReply reply: @escaping (Bool, String, Int32) -> Void
    ) {
        do {
            let client = try SkillPackGRPCClient()
            let result = try client.syncAgents(dryRun: dryRun, noIndex: noIndex, onlyAgent: onlyAgent)
            reply(result.success, result.message, Int32(result.skillsProcessed))
        } catch {
            reply(false, "Sync failed via XPC → Rust: \(error.localizedDescription)", 0)
        }
    }

    public func getStatus(withReply reply: @escaping (String, Int32, [String]) -> Void) {
        do {
            let client = try SkillPackGRPCClient()
            let result = try client.getStatus()
            reply(result.canonicalRoot, Int32(result.totalSkills), ["Rust server: \(result.isHealthy ? "online" : "offline")"])
        } catch {
            reply(resolvedPath, 0, ["Rust server: unreachable"])
        }
    }

    public func getExtendedStatus(withReply reply: @escaping (String, Int32, [String], String, String, String) -> Void) {
        do {
            let client = try SkillPackGRPCClient()
            let result = try client.getStatus()
            let rustStatus = result.isHealthy ? "online" : "offline"
            reply(result.canonicalRoot, Int32(result.totalSkills), ["Rust server: \(rustStatus)"], "rust", rustStatus, rustStatus)
        } catch {
            reply(resolvedPath, 0, ["Rust server: unreachable"], "rust", "offline", "offline")
        }
    }

    public func checkBoundary(
        targetPath: String,
        withReply reply: @escaping (Bool, String) -> Void
    ) {
        do {
            let client = try SkillPackGRPCClient()
            let result = try client.checkBoundary(targetPath: targetPath)
            reply(result.isSafe, result.message)
        } catch {
            reply(false, "Boundary check failed: \(error.localizedDescription)")
        }
    }
}
