import Foundation

/// Thin HTTP client for the Rust skillpack-server REST API.
/// All canonical business logic lives in the Rust workspace;
/// this client is responsible for transport only.
public actor SkillPackBackendClient {
    public let baseURL: URL
    private let session: URLSession
    private let decoder = JSONDecoder()

    public init(baseURL: URL, session: URLSession = .shared) {
        self.baseURL = baseURL
        self.session = session
        self.decoder.dateDecodingStrategy = .iso8601
    }

    /// Create a client using environment variables (SKILLPACK_SERVER_HOST, SKILLPACK_SERVER_PORT).
    public static func fromEnvironment() -> SkillPackBackendClient {
        let host = ProcessInfo.processInfo.environment["SKILLPACK_SERVER_HOST"] ?? "127.0.0.1"
        let port = ProcessInfo.processInfo.environment["SKILLPACK_SERVER_PORT"] ?? "8080"
        let url = URL(string: "http://\(host):\(port)")!
        return SkillPackBackendClient(baseURL: url)
    }

    // MARK: - W3C Trace Context

    private func generateTraceparent() -> String {
        let traceId = Self.randomHex(32)
        let spanId = Self.randomHex(16)
        return "00-\(traceId)-\(spanId)-01"
    }

    private static func randomHex(_ length: Int) -> String {
        let chars = Array("0123456789abcdef")
        return String((0..<length).map { _ in chars[Int.random(in: 0..<16)] })
    }

    private func request(for path: String) -> URLRequest {
        var req = URLRequest(url: baseURL.appendingPathComponent(path))
        req.setValue(generateTraceparent(), forHTTPHeaderField: "traceparent")
        req.setValue("application/json", forHTTPHeaderField: "Accept")
        return req
    }

    // MARK: - REST Proxies

    /// GET /health — server health check
    public func health() async throws -> HealthResponse {
        let (data, resp) = try await session.data(for: request(for: "health"))
        try checkStatus(response: resp)
        return try decoder.decode(HealthResponse.self, from: data)
    }

    /// GET /api/v1/skills — list all skills
    public func listSkills() async throws -> [BackendSkillSummary] {
        let (data, resp) = try await session.data(for: request(for: "api/v1/skills"))
        try checkStatus(response: resp)
        let wrapper = try decoder.decode(SkillsListResponse.self, from: data)
        return wrapper.skills
    }

    /// GET /api/v1/skills/{ref}/rating — get skill rating
    public func getRating(skillRef: String) async throws -> BackendSkillRating {
        let path = "api/v1/skills/\(skillRef)/rating"
        let (data, resp) = try await session.data(for: request(for: path))
        try checkStatus(response: resp)
        return try decoder.decode(BackendSkillRating.self, from: data)
    }

    /// GET /api/v1/skills/search — search skills
    public func searchSkills(query: String? = nil, minScore: Double? = nil) async throws -> [BackendSkillSummary] {
        var components = URLComponents(url: baseURL.appendingPathComponent("api/v1/skills/search"), resolvingAgainstBaseURL: true)!
        var items: [URLQueryItem] = []
        if let q = query { items.append(URLQueryItem(name: "text", value: q)) }
        if let ms = minScore { items.append(URLQueryItem(name: "min_score", value: String(ms))) }
        components.queryItems = items

        var req = URLRequest(url: components.url!)
        req.setValue(generateTraceparent(), forHTTPHeaderField: "traceparent")
        req.setValue("application/json", forHTTPHeaderField: "Accept")

        let (data, resp) = try await session.data(for: req)
        try checkStatus(response: resp)
        let result = try decoder.decode(SearchResponse.self, from: data)
        return result.skills
    }

    // MARK: - Helpers

    private func checkStatus(response: URLResponse) throws {
        guard let http = response as? HTTPURLResponse else { return }
        if http.statusCode >= 400 {
            throw BackendError.httpStatus(http.statusCode)
        }
    }
}

// MARK: - Models

public struct HealthResponse: Decodable, Sendable {
    public let status: String
    public let protocolVersion: String

    enum CodingKeys: String, CodingKey {
        case status
        case protocolVersion = "protocol_version"
    }
}

public struct BackendSkillSummary: Decodable, Sendable, Identifiable {
    public let skillRef: String
    public let name: String?
    public let grade: String?
    public let score: Double?
    public let moodysRating: String?
    public let reputationTier: String?

    public var id: String { skillRef }

    enum CodingKeys: String, CodingKey {
        case skillRef = "skill_ref"
        case name
        case grade
        case score
        case moodysRating = "moodys_rating"
        case reputationTier = "reputation_tier"
    }
}

public struct BackendSkillRating: Decodable, Sendable {
    public let skillRef: String
    public let rating: String?
    public let grade: String?
    public let score: Double?

    enum CodingKeys: String, CodingKey {
        case skillRef = "skill_ref"
        case rating
        case grade
        case score
    }
}

private struct SkillsListResponse: Decodable {
    let skills: [BackendSkillSummary]
}

private struct SearchResponse: Decodable {
    let skills: [BackendSkillSummary]
}

public enum BackendError: Error, Sendable {
    case httpStatus(Int)
    case decodingError(String)
}
