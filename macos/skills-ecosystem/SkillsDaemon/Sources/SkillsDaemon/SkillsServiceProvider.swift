import Foundation
import GRPC
import NIO
import SkillsCore

public class SkillsServiceProvider: Skills_SkillsServiceProvider {
    public var interceptors: Skills_SkillsServiceServerInterceptorFactoryProtocol? {
        return nil
    }

    private let resolvedPath: String

    public init(resolvedPath: String) {
        self.resolvedPath = resolvedPath
    }

    public func syncSkills(
        request: Skills_SyncRequest,
        context: StatusOnlyCallContext
    ) -> EventLoopFuture<Skills_SyncResponse> {
        let manager = SyncManager(skillsRootPath: resolvedPath) { log in
            NSLog("SkillsDaemonGRPC [SyncLog]: \(log)")
        }

        var response = Skills_SyncResponse()
        do {
            try manager.execute(options: SyncOptions(
                dryRun: request.dryRun,
                doIndex: !request.noIndex,
                statusOnly: false,
                onlyAgent: request.onlyAgent.isEmpty ? nil : request.onlyAgent
            ))

            let skills = try SkillDiscovery().discoverSkills(in: URL(fileURLWithPath: resolvedPath))
            response.success = true
            response.message = "Sync successful via gRPC"
            response.skillsProcessed = Int32(skills.count)
        } catch {
            response.success = false
            response.message = "Sync failed via gRPC: \(error.localizedDescription)"
            response.skillsProcessed = 0
        }

        return context.eventLoop.makeSucceededFuture(response)
    }

    public func getStatus(
        request: Skills_StatusRequest,
        context: StatusOnlyCallContext
    ) -> EventLoopFuture<Skills_StatusResponse> {
        var response = Skills_StatusResponse()
        response.canonicalRoot = resolvedPath
        do {
            let skills = try SkillDiscovery().discoverSkills(in: URL(fileURLWithPath: resolvedPath))
            response.totalSkills = Int32(skills.count)
            response.activeAgents = AgentRegistry.shared.agents.map { $0.name }
        } catch {
            response.totalSkills = 0
            response.activeAgents = []
        }
        return context.eventLoop.makeSucceededFuture(response)
    }

    public func checkBoundary(
        request: Skills_BoundaryRequest,
        context: StatusOnlyCallContext
    ) -> EventLoopFuture<Skills_BoundaryResponse> {
        var response = Skills_BoundaryResponse()
        let url = URL(fileURLWithPath: request.targetPath)
        do {
            try IPGuard.guardPath(url)
            response.isSafe = true
            response.message = "Path is safe"
        } catch let IPGuardError.boundaryViolation(msg) {
            response.isSafe = false
            response.message = msg
        } catch {
            response.isSafe = false
            response.message = error.localizedDescription
        }
        return context.eventLoop.makeSucceededFuture(response)
    }
}
