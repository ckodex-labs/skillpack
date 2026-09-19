import Foundation

/// T-07 (§4.6): STX publisher projections — REST surface over the existing gRPC backend.
///
/// Architecture: the `SkillsDaemon` already exposes port 50051 (gRPC).
/// `STXPublisher` adds a lightweight HTTP REST facade that wraps the same
/// lifecycle and skill-listing operations. Configured via environment variables:
///   SKILLPACK_API_URL    — base URL of the REST endpoint (default http://localhost:50051)
///   SKILLPACK_TOKEN      — optional Bearer token for authenticated deployments
///
/// All methods are synchronous (semaphore-backed) to stay consistent with the
/// existing RustFSClient / FoundationDBClient patterns in this module.
/// CLIENT-SPEC.md §2.3: maximum protocol version this client supports.
let SUPPORTED_PROTOCOL_VERSION = 1

public struct HealthResult {
    public let healthy: Bool
    public let protocolVersion: Int?
}

public struct STXPublisher {
    public let endpoint: URL
    private let session: URLSession
    private let logger: (String) -> Void
    let bearerToken: String?

    public init(
        endpointURL: String? = nil,
        session: URLSession = .shared,
        logger: @escaping (String) -> Void = { _ in }
    ) {
        let envEndpoint = ProcessInfo.processInfo.environment["SKILLPACK_API_URL"] ?? "http://localhost:50051"
        let urlStr = endpointURL ?? envEndpoint
        self.endpoint = URL(string: urlStr) ?? URL(string: "http://localhost:50051")!
        self.session = session
        self.logger = logger
        // CLIENT-SPEC.md §6.1: env var primary, Keychain secondary
        self.bearerToken = ProcessInfo.processInfo.environment["SKILLPACK_TOKEN"] ?? KeychainTokenStore.load()
    }

    /// Internal init for testing transport contract without env vars.
    init(endpoint: URL, token: String?, session: URLSession = .shared) {
        self.endpoint = endpoint
        self.session = session
        self.logger = { _ in }
        self.bearerToken = token
    }

    // MARK: - Health

    /// CLIENT-SPEC.md §3.1: Health probe with 2s timeout + capability negotiation.
    public func checkHealth() -> HealthResult {
        let url = endpoint.appendingPathComponent("health")
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.timeoutInterval = 2.0
        applyAuth(to: &request)

        let semaphore = DispatchSemaphore(value: 0)
        var result = HealthResult(healthy: false, protocolVersion: nil)
        session.dataTask(with: request) { data, response, _ in
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                semaphore.signal()
                return
            }
            if let data = data,
               let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let pv = json["protocolVersion"] as? Int {
                result = HealthResult(
                    healthy: pv <= SUPPORTED_PROTOCOL_VERSION,
                    protocolVersion: pv
                )
            } else {
                result = HealthResult(healthy: true, protocolVersion: nil)
            }
            semaphore.signal()
        }.resume()
        _ = semaphore.wait(timeout: .now() + .seconds(2))
        return result
    }

    // MARK: - Skill listing

    /// GET /skills → array of skill name strings.
    public func listSkills() throws -> [String] {
        let url = endpoint.appendingPathComponent("skills")
        let data = try syncGET(url: url)
        guard let names = try? JSONDecoder().decode([String].self, from: data) else {
            throw STXError.unexpectedResponseShape("GET /skills expected [String]")
        }
        return names
    }

    // MARK: - Lifecycle events

    /// POST /lifecycle/events — publish a LifecycleDecisionBundle.
    @discardableResult
    public func publishLifecycleEvent(_ bundle: LifecycleDecisionBundle) throws -> Int {
        let url = endpoint.appendingPathComponent("lifecycle/events")
        let body = try JSONEncoder().encode(bundle)
        return try syncPOST(url: url, body: body)
    }

    /// POST /lifecycle/promote — trigger a promote request on the STX side.
    @discardableResult
    public func promote(skillName: String) throws -> Int {
        let url = endpoint.appendingPathComponent("lifecycle/promote")
        let payload = try JSONEncoder().encode(["skill_name": skillName])
        return try syncPOST(url: url, body: payload)
    }

    /// POST /lifecycle/demote — trigger a demote/supersede request on the STX side.
    @discardableResult
    public func demote(skillName: String, supersededBy: String? = nil) throws -> Int {
        let url = endpoint.appendingPathComponent("lifecycle/demote")
        var dict: [String: String] = ["skill_name": skillName]
        if let s = supersededBy { dict["superseded_by"] = s }
        let payload = try JSONEncoder().encode(dict)
        return try syncPOST(url: url, body: payload)
    }

    // MARK: - Bundle manifest

    /// GET /bundles/{skillName} → raw bundle.json string.
    public func getBundle(skillName: String) throws -> String? {
        let url = endpoint.appendingPathComponent("bundles/\(skillName)")
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        applyAuth(to: &request)

        let semaphore = DispatchSemaphore(value: 0)
        var result: Data?
        var statusCode = 0
        session.dataTask(with: request) { data, response, _ in
            statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            result = data
            semaphore.signal()
        }.resume()
        semaphore.wait()

        if statusCode == 404 { return nil }
        guard (200..<300).contains(statusCode), let data = result else {
            throw STXError.httpError(statusCode)
        }
        return String(data: data, encoding: .utf8)
    }

    // MARK: - Internals

    private func syncGET(url: URL) throws -> Data {
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.timeoutInterval = 10.0
        applyAuth(to: &request)

        let semaphore = DispatchSemaphore(value: 0)
        var resultData: Data?
        var resultStatus = 0
        var resultError: Error?

        session.dataTask(with: request) { data, response, error in
            resultError = error
            resultStatus = (response as? HTTPURLResponse)?.statusCode ?? 0
            resultData = data
            semaphore.signal()
        }.resume()
        semaphore.wait()

        if let e = resultError { throw e }
        guard (200..<300).contains(resultStatus), let data = resultData else {
            throw STXError.httpError(resultStatus)
        }
        return data
    }

    @discardableResult
    private func syncPOST(url: URL, body: Data) throws -> Int {
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.httpBody = body
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 10.0
        applyAuth(to: &request)

        let semaphore = DispatchSemaphore(value: 0)
        var statusCode = 0
        var resultError: Error?

        session.dataTask(with: request) { _, response, error in
            resultError = error
            statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            semaphore.signal()
        }.resume()
        semaphore.wait()

        if let e = resultError { throw e }
        guard (200..<300).contains(statusCode) else { throw STXError.httpError(statusCode) }
        return statusCode
    }

    func applyAuth(to request: inout URLRequest) {
        if let token = bearerToken {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
    }
}

/// T-07: Errors from STXPublisher.
public enum STXError: LocalizedError {
    case httpError(Int)
    case unexpectedResponseShape(String)

    public var errorDescription: String? {
        switch self {
        case .httpError(let code): return "STX endpoint returned HTTP \(code)"
        case .unexpectedResponseShape(let msg): return "STX response shape unexpected: \(msg)"
        }
    }
}
