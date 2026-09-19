import Foundation

/// Bidirectional WebSocket client for the Rust skillpack-server `/ws` endpoint.
/// Forwards real-time ServerEvents to a Swift AsyncStream and accepts commands.
@MainActor
public final class WebSocketEventStream: ObservableObject, Sendable {
    public enum ConnectionState: Sendable, Equatable {
        case disconnected
        case connecting
        case connected
        case error(String)

        public static func == (lhs: ConnectionState, rhs: ConnectionState) -> Bool {
            switch (lhs, rhs) {
            case (.disconnected, .disconnected),
                 (.connecting, .connecting),
                 (.connected, .connected):
                return true
            case (.error(let a), .error(let b)):
                return a == b
            default:
                return false
            }
        }
    }

    @Published public private(set) var state: ConnectionState = .disconnected
    @Published public private(set) var lastEvent: ServerEvent?

    private let url: URL
    private var task: Task<Void, Never>?
    private var webSocketTask: URLSessionWebSocketTask?

    public init(baseURL: URL) {
        var components = URLComponents(url: baseURL, resolvingAgainstBaseURL: true)!
        components.scheme = components.scheme == "https" ? "wss" : "ws"
        components.path = "/ws"
        self.url = components.url!
    }

    /// Start the WebSocket connection.
    public func connect() {
        guard state == .disconnected else { return }
        state = .connecting

        let session = URLSession(configuration: .default)
        let request = URLRequest(url: url)
        let wsTask = session.webSocketTask(with: request)
        self.webSocketTask = wsTask

        task = Task { [weak self] in
            self?.receiveLoop(task: wsTask)
        }
        wsTask.resume()
    }

    /// Gracefully close the connection.
    public func disconnect() {
        webSocketTask?.cancel(with: .normalClosure, reason: nil)
        webSocketTask = nil
        task?.cancel()
        task = nil
        state = .disconnected
    }

    /// Send a text command to the backend.
    public func send(_ text: String) async {
        guard let ws = webSocketTask else { return }
        do {
            try await ws.send(.string(text))
        } catch {
            state = .error(error.localizedDescription)
        }
    }

    // MARK: - Receive Loop

    private func receiveLoop(task wsTask: URLSessionWebSocketTask) {
        wsTask.receive { [weak self] result in
            guard let self else { return }
            Task { @MainActor in
                switch result {
                case .failure(let error):
                    self.state = .error(error.localizedDescription)
                case .success(let message):
                    switch message {
                    case .string(let text):
                        self.state = .connected
                        if let event = try? JSONDecoder().decode(ServerEvent.self, from: Data(text.utf8)) {
                            self.lastEvent = event
                        }
                    case .data:
                        break
                    @unknown default:
                        break
                    }
                    // Continue receiving
                    self.receiveLoop(task: wsTask)
                }
            }
        }
    }
}

// MARK: - ServerEvent Model

/// Mirrors `skillpack_domain::cache::ServerEvent` from the Rust workspace.
public struct ServerEvent: Codable, Sendable, Equatable {
    public let kind: String
    public let payload: ServerEventPayload

    enum CodingKeys: String, CodingKey {
        case kind
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        kind = try container.decode(String.self, forKey: .kind)
        payload = try ServerEventPayload(from: decoder)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(kind, forKey: .kind)
        try payload.encode(to: encoder)
    }
}

public struct ServerEventPayload: Codable, Sendable, Equatable {
    public let skillRef: String?
    public let skillPath: String?
    public let agentName: String?
    public let skillName: String?
    public let status: String?
    public let current: Int?
    public let total: Int?
    public let skillsProcessed: Int?
    public let key: String?
    public let rating: String?

    enum CodingKeys: String, CodingKey {
        case skillRef = "skill_ref"
        case skillPath = "skill_path"
        case agentName = "agent_name"
        case skillName = "skill_name"
        case status
        case current
        case total
        case skillsProcessed = "skills_processed"
        case key
        case rating
    }
}
