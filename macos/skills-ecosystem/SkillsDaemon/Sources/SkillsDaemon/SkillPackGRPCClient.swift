import Foundation
import GRPC
import NIO

/// Thin gRPC client for the Rust skillpack-server.
/// Delegates all canonical store operations to the Rust backend.
public final class SkillPackGRPCClient {
    private let channel: GRPCChannel
    private let client: Skillpack_V1_CanonicalStoreServiceNIOClient

    /// Connect to the Rust skillpack-server at the given host/port.
    public init(host: String = "127.0.0.1", port: Int = 50051) throws {
        let group = PlatformSupport.makeEventLoopGroup(loopCount: 1)
        self.channel = try GRPCChannelPool.with(
            target: .hostAndPort(host, port),
            transportSecurity: .plaintext,
            eventLoopGroup: group
        )
        self.client = Skillpack_V1_CanonicalStoreServiceNIOClient(channel: channel)
    }

    deinit {
        try? channel.close().wait()
    }

    // MARK: - Sync

    public func syncAgents(dryRun: Bool = false, noIndex: Bool = false, onlyAgent: String? = nil) throws -> (success: Bool, message: String, skillsProcessed: Int) {
        var request = Skillpack_V1_SyncAgentsRequest()
        request.dryRun = dryRun
        request.noIndex = noIndex
        request.onlyAgent = onlyAgent ?? ""
        let response = try client.syncAgents(request).response.wait()
        return (response.success, response.message, Int(response.skillsProcessed))
    }

    // MARK: - Boundary Check

    public func checkBoundary(targetPath: String, skillName: String? = nil) throws -> (isSafe: Bool, message: String) {
        var request = Skillpack_V1_CheckBoundaryRequest()
        request.targetPath = targetPath
        request.skillName = skillName ?? ""
        let response = try client.checkBoundary(request).response.wait()
        return (response.isSafe, response.message)
    }

    // MARK: - Migrate

    public func migrateAll(canonicalRoot: String) throws -> (success: Bool, message: String, migrated: Int, replaced: Int, skippedLinked: Int, skippedIp: Int) {
        var request = Skillpack_V1_MigrateAllRequest()
        request.canonicalRoot = canonicalRoot
        let response = try client.migrateAll(request).response.wait()
        return (
            response.success,
            response.message,
            Int(response.migratedCount),
            Int(response.replacedCount),
            Int(response.skippedLinkedCount),
            Int(response.skippedIpCount)
        )
    }

    // MARK: - Import

    public func importSkill(sourcePath: String, targetName: String? = nil) throws -> (success: Bool, message: String, skillName: String) {
        var request = Skillpack_V1_ImportSkillRequest()
        request.sourcePath = sourcePath
        request.targetName = targetName ?? ""
        let response = try client.importSkill(request).response.wait()
        return (response.success, response.message, response.skillName)
    }

    // MARK: - Promote

    public func promoteSkill(candidateName: String) throws -> (success: Bool, message: String, skillName: String) {
        var request = Skillpack_V1_PromoteSkillRequest()
        request.candidateName = candidateName
        let response = try client.promoteSkill(request).response.wait()
        return (response.success, response.message, response.skillName)
    }

    // MARK: - Status

    public func getStatus() throws -> (canonicalRoot: String, totalSkills: Int, isHealthy: Bool) {
        let request = Skillpack_V1_GetStatusRequest()
        let response = try client.getStatus(request).response.wait()
        return (response.canonicalRoot, Int(response.totalSkills), response.isHealthy)
    }

    // MARK: - Frontmatter Migration

    public func migrateSkills(canonicalRoot: String? = nil) throws -> (success: Bool, message: String, totalScanned: Int, migrated: Int, compliant: Int, names: [String]) {
        var request = Skillpack_V1_MigrateSkillsRequest()
        if let root = canonicalRoot { request.canonicalRoot = root }
        let response = try client.migrateSkills(request).response.wait()
        return (response.success, response.message, Int(response.totalScanned), Int(response.migratedCount), Int(response.alreadyCompliantCount), response.migratedNames)
    }

    public func migrateAgents() throws -> (success: Bool, message: String, totalScanned: Int, migrated: Int, compliant: Int, names: [String]) {
        let request = Skillpack_V1_MigrateAgentsRequest()
        let response = try client.migrateAgents(request).response.wait()
        return (response.success, response.message, Int(response.totalScanned), Int(response.migratedCount), Int(response.alreadyCompliantCount), response.migratedNames)
    }

    public func migrateClaudeAgents(agentsDir: String? = nil) throws -> (success: Bool, message: String, totalScanned: Int, migrated: Int, compliant: Int, names: [String]) {
        var request = Skillpack_V1_MigrateClaudeAgentsRequest()
        if let dir = agentsDir { request.agentsDir = dir }
        let response = try client.migrateClaudeAgents(request).response.wait()
        return (response.success, response.message, Int(response.totalScanned), Int(response.migratedCount), Int(response.alreadyCompliantCount), response.migratedNames)
    }

    public func migrateHarnesses() throws -> (success: Bool, message: String, totalScanned: Int, migrated: Int, compliant: Int, names: [String]) {
        let request = Skillpack_V1_MigrateHarnessesRequest()
        let response = try client.migrateHarnesses(request).response.wait()
        return (response.success, response.message, Int(response.totalScanned), Int(response.migratedCount), Int(response.alreadyCompliantCount), response.migratedNames)
    }
}
