import Foundation

/// T-08 (§4.6): Decision event emitted by SyncManager on every promote/demote.
/// Published over the STX projection layer (REST or gRPC) and optionally persisted
/// to the local `.lifecycle-log/` directory for audit purposes.
public struct LifecycleDecisionBundle: Codable {
    /// Logical action performed.
    public enum Decision: String, Codable {
        case promoted
        case demoted
        case superseded
        case retired
    }

    public let skillName: String
    public let decision: Decision
    /// Lifecycle status after the transition (per SkillLifecycleStatus).
    public let newStatus: String
    /// Lifecycle status before the transition, if known.
    public let previousStatus: String?
    /// RFC 3339 UTC timestamp.
    public let decidedAt: String
    /// Optional: name of the skill this was superseded by.
    public let supersededBy: String?
    /// Optional: actor URN (e.g. "urn:ckodex:actor:cli:skills-cli").
    public let actorUrn: String?

    public init(
        skillName: String,
        decision: Decision,
        newStatus: String,
        previousStatus: String? = nil,
        decidedAt: Date = Date(),
        supersededBy: String? = nil,
        actorUrn: String? = nil
    ) {
        self.skillName = skillName
        self.decision = decision
        self.newStatus = newStatus
        self.previousStatus = previousStatus
        self.decidedAt = ISO8601DateFormatter().string(from: decidedAt)
        self.supersededBy = supersededBy
        self.actorUrn = actorUrn
    }
}

/// T-08: Publishes LifecycleDecisionBundle events.
/// Primary sink: local `.lifecycle-log/` directory (always).
/// Secondary sink: STX REST endpoint (if SKILLS_STX_ENDPOINT env var is set).
public struct LifecyclePublisher {
    private let skillsRoot: URL
    private let logger: (String) -> Void
    private let fileManager = FileManager.default

    public init(skillsRoot: URL, logger: @escaping (String) -> Void = { _ in }) {
        self.skillsRoot = skillsRoot
        self.logger = logger
    }

    /// Persist and optionally push the bundle.
    public func publish(_ bundle: LifecycleDecisionBundle) {
        // 1. Write to local audit log (non-fatal)
        do {
            try persistLocally(bundle)
        } catch {
            logger("  ⚠ lifecycle-log write failed: \(error.localizedDescription)")
        }

        // 2. Push to STX REST endpoint if configured
        if let endpoint = ProcessInfo.processInfo.environment["SKILLS_STX_ENDPOINT"],
           let url = URL(string: "\(endpoint)/lifecycle/events") {
            pushREST(bundle, to: url)
        }
    }

    // MARK: - Private

    private func persistLocally(_ bundle: LifecycleDecisionBundle) throws {
        let logDir = skillsRoot.appendingPathComponent(".lifecycle-log")
        try fileManager.createDirectory(at: logDir, withIntermediateDirectories: true)

        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(bundle)

        let filename = "\(bundle.decidedAt.replacingOccurrences(of: ":", with: "-"))_\(bundle.skillName)_\(bundle.decision.rawValue).json"
        let fileURL = logDir.appendingPathComponent(filename)
        try data.write(to: fileURL, options: .atomic)
        logger("  ✓ lifecycle-log → \(fileURL.lastPathComponent)")
    }

    /// Non-blocking fire-and-forget POST to the STX REST endpoint.
    private func pushREST(_ bundle: LifecycleDecisionBundle, to url: URL) {
        guard let data = try? JSONEncoder().encode(bundle) else { return }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.httpBody = data
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 5.0

        URLSession.shared.dataTask(with: request) { [logger] _, response, error in
            if let error = error {
                logger("  ⚠ STX REST push failed: \(error.localizedDescription)")
            } else if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
                logger("  ⚠ STX REST push HTTP \(http.statusCode)")
            } else {
                logger("  ✓ STX lifecycle event pushed to \(url.absoluteString)")
            }
        }.resume()
    }
}
