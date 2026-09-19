import SwiftUI
import SkillsCore

@main
struct SkillsUIApp: App {
    @StateObject private var viewModel = SkillsViewModel()

    var body: some Scene {
        MenuBarExtra {
            ContentView()
                .environmentObject(viewModel)
        } label: {
            Image(systemName: viewModel.state.systemImage)
                .symbolRenderingMode(.hierarchical)
        }
        .menuBarExtraStyle(.window)
    }
}

// MARK: - State

enum AppState: Equatable {
    case idle
    case syncing
    case error(String)

    var systemImage: String {
        switch self {
        case .idle:    return "arrow.triangle.2.circlepath"
        case .syncing: return "arrow.triangle.2.circlepath.circle.fill"
        case .error:   return "exclamationmark.triangle.fill"
        }
    }

    var label: String {
        switch self {
        case .idle:           return "Idle"
        case .syncing:        return "Syncing…"
        case .error(let msg): return "Error: \(msg)"
        }
    }
}

@MainActor
final class SkillsViewModel: ObservableObject {
    @Published var state: AppState = .idle
    @Published var status: DaemonStatus?
    @Published var physicalSkills: [DiscoveredPhysicalSkill] = []
    @Published var lastSync: Date?
    @Published var lastSyncSummary: String?
    @Published var dryRun: Bool = true
    @Published var skipIndex: Bool = false

    // MARK: - Backend Connectivity
    @Published var backendSkills: [BackendSkillSummary] = []
    @Published var backendConnected: Bool = false
    @Published var backendEvents: [ServerEvent] = []
    @Published var backendError: String?

    private let manager = SyncManager()
    private let backendClient = SkillPackBackendClient.fromEnvironment()
    private var eventStream: WebSocketEventStream?

    init() {
        Task { await self.refreshStatus() }
        Task { await self.connectBackend() }
    }

    func refreshStatus() async {
        let root = manager.skillsRoot.path
        let result: DaemonStatus? = await Task.detached { @Sendable in
            MenuBarDaemonClient.queryStatus(skillsRoot: root)
        }.value

        if let result {
            self.status = result
        } else {
            self.status = await localStatusSnapshot(root: root)
        }

        let discovered = await Task.detached { @Sendable in
            (try? SyncManager().scanForPhysicalSkills()) ?? []
        }.value
        self.physicalSkills = discovered
    }

    private func localStatusSnapshot(root: String) async -> DaemonStatus {
        return await Task.detached { @Sendable in
            let fm = FileManager.default
            let url = URL(fileURLWithPath: root)
            var count = 0
            if let entries = try? fm.contentsOfDirectory(at: url, includingPropertiesForKeys: nil) {
                for entry in entries where (try? entry.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true {
                    let skill = entry.appendingPathComponent("SKILL.md")
                    if fm.fileExists(atPath: skill.path) { count += 1 }
                }
            }
            let agents = AgentRegistry.shared.agents.map { $0.name }.sorted()
            
            let backend = ProcessInfo.processInfo.environment["STORAGE_BACKEND"] ?? "local"
            var fdbStatus = "n/a"
            var rustStatus = "n/a"
            if backend.lowercased() == "distributed" {
                fdbStatus = FoundationDBClient().checkHealth() ? "online" : "offline"
                rustStatus = RustFSClient().checkHealth() ? "online" : "offline"
            }
            
            return DaemonStatus(
                root: root,
                skillCount: count,
                agents: agents,
                transport: "local",
                storageBackend: backend,
                fdbStatus: fdbStatus,
                rustFSStatus: rustStatus
            )
        }.value
    }

    func performSync() async {
        self.state = .syncing
        self.lastSyncSummary = nil
        let dryRun = self.dryRun
        let noIndex = self.skipIndex

        let daemon: DaemonSyncResult? = await Task.detached { @Sendable in
            MenuBarDaemonClient.triggerSync(dryRun: dryRun, noIndex: noIndex, onlyAgent: nil)
        }.value

        if let daemon {
            self.lastSync = Date()
            self.lastSyncSummary = "[\(daemon.transport)] \(daemon.message) — \(daemon.count) skill(s)"
            self.state = daemon.success ? .idle : .error(daemon.message)
            await self.refreshStatus()
            return
        }

        do {
            let opts = SyncOptions(dryRun: dryRun, doIndex: !noIndex, statusOnly: false, onlyAgent: nil)
            let mgr = self.manager
            try await Task.detached { @Sendable in
                try mgr.execute(options: opts)
            }.value
            self.lastSync = Date()
            self.lastSyncSummary = "[local] sync complete"
            self.state = .idle
            await self.refreshStatus()
        } catch {
            self.state = .error(error.localizedDescription)
            self.lastSyncSummary = "[local] failed: \(error.localizedDescription)"
        }
    }

    func performMigrateAll() async {
        self.state = .syncing
        self.lastSyncSummary = nil

        do {
            let mgr = self.manager
            let results = try await Task.detached { @Sendable in
                try mgr.migrateAll()
            }.value

            self.lastSync = Date()
            let migratedCount = results.filter { $0.outcome == .migrated || $0.outcome == .replacedWithLink }.count
            let blockedCount = results.filter { if case .skippedIPViolation = $0.outcome { return true }; return false }.count

            self.lastSyncSummary = "[local] migrated \(migratedCount) skills (\(blockedCount) blocked)"
            self.state = .idle
            await self.refreshStatus()
        } catch {
            self.state = .error(error.localizedDescription)
            self.lastSyncSummary = "[local] migration failed: \(error.localizedDescription)"
        }
    }

    // MARK: - Backend Integration

    func connectBackend() async {
        do {
            _ = try await backendClient.health()
            self.backendConnected = true
            self.backendError = nil

            // Start WebSocket for real-time events
            let stream = WebSocketEventStream(baseURL: backendClient.baseURL)
            self.eventStream = stream
            stream.connect()

            // Subscribe to events
            Task {
                for await _ in stream.$lastEvent.values {
                    if let event = stream.lastEvent {
                        self.backendEvents.insert(event, at: 0)
                        if self.backendEvents.count > 50 { self.backendEvents.removeLast() }
                    }
                }
            }

            await fetchBackendSkills()
        } catch {
            self.backendConnected = false
            self.backendError = error.localizedDescription
        }
    }

    func fetchBackendSkills() async {
        do {
            let skills = try await backendClient.listSkills()
            self.backendSkills = skills
        } catch {
            self.backendError = error.localizedDescription
        }
    }

    func disconnectBackend() {
        eventStream?.disconnect()
        eventStream = nil
        backendConnected = false
    }
}

// MARK: - View

struct ContentView: View {
    @EnvironmentObject var viewModel: SkillsViewModel
    @State private var isInventoryExpanded: Bool = false
    @State private var isRotating: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            
            Divider()
                .background(Color.secondary.opacity(0.2))
            
            statusBlock
            
            if viewModel.backendConnected {
                Divider()
                    .background(Color.secondary.opacity(0.2))
                backendBlock
            }

            if let s = viewModel.status, s.storageBackend.lowercased() == "distributed" {
                Divider()
                    .background(Color.secondary.opacity(0.2))
                distributedStorageBlock(status: s)
            }

            if !viewModel.physicalSkills.isEmpty {
                Divider()
                    .background(Color.secondary.opacity(0.2))
                inventoryBlock
            }
            
            Divider()
                .background(Color.secondary.opacity(0.2))
            optionsBlock
            
            Divider()
                .background(Color.secondary.opacity(0.2))
            actions
        }
        .padding(14)
        .frame(width: 320)
    }

    private var backendBlock: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("SkillPack Backend")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundStyle(.secondary)
                Spacer()
                Circle()
                    .fill(Color.green)
                    .frame(width: 6, height: 6)
                    .shadow(color: .green, radius: 2)
                Text("LIVE")
                    .font(.system(size: 8, weight: .bold, design: .monospaced))
                    .padding(.horizontal, 4)
                    .padding(.vertical, 1)
                    .background(Capsule().fill(Color.green.opacity(0.15)))
                    .foregroundStyle(.green)
            }

            if !viewModel.backendSkills.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("\(viewModel.backendSkills.count) skill(s) in canonical store")
                        .font(.system(size: 10))
                        .foregroundStyle(.secondary)
                    ForEach(viewModel.backendSkills.prefix(5)) { skill in
                        HStack {
                            Text(skill.skillRef)
                                .font(.system(size: 10, design: .monospaced))
                                .lineLimit(1)
                            Spacer()
                            if let grade = skill.grade {
                                Text(grade)
                                    .font(.system(size: 9, weight: .bold))
                                    .padding(.horizontal, 4)
                                    .padding(.vertical, 1)
                                    .background(Capsule().fill(Color.blue.opacity(0.15)))
                                    .foregroundStyle(.blue)
                            }
                        }
                    }
                }
                .padding(6)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.secondary.opacity(0.04))
                )
            }

            if let last = viewModel.backendEvents.first {
                HStack(spacing: 6) {
                    Image(systemName: "bolt.fill")
                        .font(.system(size: 8))
                        .foregroundStyle(.yellow)
                    Text("Last event: \(last.kind)")
                        .font(.system(size: 9, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                    Spacer()
                }
                .padding(6)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.yellow.opacity(0.06))
                )
            }
        }
    }

    private func distributedStorageBlock(status: DaemonStatus) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Distributed Storage System")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundStyle(.secondary)
            
            HStack(spacing: 8) {
                // FoundationDB Metadata
                HStack(spacing: 6) {
                    Circle()
                        .fill(status.fdbStatus == "online" ? Color.green : Color.red)
                        .frame(width: 6, height: 6)
                        .shadow(color: status.fdbStatus == "online" ? .green : .red, radius: 2)
                    
                    VStack(alignment: .leading, spacing: 1) {
                        Text("Metadata Index")
                            .font(.system(size: 8))
                            .foregroundStyle(.secondary)
                        Text("FoundationDB")
                            .font(.system(size: 9, weight: .semibold, design: .monospaced))
                    }
                    
                    Spacer()
                    
                    Text(status.fdbStatus.uppercased())
                        .font(.system(size: 8, weight: .bold, design: .monospaced))
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(
                            Capsule().fill(status.fdbStatus == "online" ? Color.green.opacity(0.15) : Color.red.opacity(0.15))
                        )
                        .foregroundStyle(status.fdbStatus == "online" ? .green : .red)
                }
                .padding(6)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.secondary.opacity(0.04))
                )
                .frame(maxWidth: .infinity)
                
                // RustFS Payloads
                HStack(spacing: 6) {
                    Circle()
                        .fill(status.rustFSStatus == "online" ? Color.green : Color.red)
                        .frame(width: 6, height: 6)
                        .shadow(color: status.rustFSStatus == "online" ? .green : .red, radius: 2)
                    
                    VStack(alignment: .leading, spacing: 1) {
                        Text("Payload Store")
                            .font(.system(size: 8))
                            .foregroundStyle(.secondary)
                        Text("RustFS S3")
                            .font(.system(size: 9, weight: .semibold, design: .monospaced))
                    }
                    
                    Spacer()
                    
                    Text(status.rustFSStatus.uppercased())
                        .font(.system(size: 8, weight: .bold, design: .monospaced))
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(
                            Capsule().fill(status.rustFSStatus == "online" ? Color.green.opacity(0.15) : Color.red.opacity(0.15))
                        )
                        .foregroundStyle(status.rustFSStatus == "online" ? .green : .red)
                }
                .padding(6)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color.secondary.opacity(0.04))
                )
                .frame(maxWidth: .infinity)
            }
        }
    }

    private var header: some View {
        HStack {
            Image(systemName: viewModel.state.systemImage)
                .font(.title3)
                .foregroundStyle(headerTint)
                .rotationEffect(.degrees(isRotating ? 360 : 0))
                .animation(viewModel.state == .syncing ? .linear(duration: 1.5).repeatForever(autoreverses: false) : .default, value: isRotating)
                .onAppear {
                    if viewModel.state == .syncing {
                        isRotating = true
                    }
                }
                .onChange(of: viewModel.state) { newState in
                    if newState == .syncing {
                        isRotating = true
                    } else {
                        isRotating = false
                    }
                }
            
            VStack(alignment: .leading, spacing: 2) {
                Text("Ckodex Skills")
                    .font(.headline)
                    .fontWeight(.bold)
                Text(viewModel.state.label)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            
            Spacer()
            
            // Status pill
            Text(viewModel.status?.transport.uppercased() ?? "OFFLINE")
                .font(.system(size: 9, weight: .bold, design: .monospaced))
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(
                    Capsule()
                        .fill(viewModel.status?.transport == "XPC" ? Color.green.opacity(0.15) : Color.orange.opacity(0.15))
                )
                .foregroundStyle(viewModel.status?.transport == "XPC" ? .green : .orange)
        }
    }

    private var headerTint: Color {
        switch viewModel.state {
        case .idle:    return .blue
        case .syncing: return .orange
        case .error:   return .red
        }
    }

    @ViewBuilder
    private var statusBlock: some View {
        if let s = viewModel.status {
            VStack(alignment: .leading, spacing: 6) {
                row("Store Root", abbreviateHome(s.root), mono: true)
                row("Strategy", s.storageBackend.uppercased())
                row("Canonical", "\(s.skillCount) active skills")
                row("Registry", "\(s.agents.count) agents configured")
            }
            .font(.system(size: 11))
        } else {
            HStack(spacing: 8) {
                ProgressView()
                    .controlSize(.small)
                Text("Querying daemon status…")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func row(_ label: String, _ value: String, mono: Bool = false) -> some View {
        HStack(alignment: .top) {
            Text(label)
                .frame(width: 80, alignment: .leading)
                .foregroundStyle(.secondary)
                .font(.system(size: 11, weight: .medium))
            Text(value)
                .font(mono ? .system(size: 11, design: .monospaced) : .system(size: 11))
                .lineLimit(1)
                .truncationMode(.middle)
                .foregroundStyle(.primary)
        }
    }

    private var inventoryBlock: some View {
        DisclosureGroup(isExpanded: $isInventoryExpanded) {
            VStack(alignment: .leading, spacing: 6) {
                ScrollView(.vertical, showsIndicators: true) {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(viewModel.physicalSkills, id: \.path) { skill in
                            HStack {
                                Image(systemName: "folder")
                                    .foregroundStyle(.orange)
                                    .font(.system(size: 10))
                                Text(skill.skillName)
                                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                                    .lineLimit(1)
                                Spacer()
                                Text(skill.agentName)
                                    .font(.system(size: 10))
                                    .padding(.horizontal, 4)
                                    .padding(.vertical, 1)
                                    .background(Capsule().fill(Color.gray.opacity(0.15)))
                                    .foregroundStyle(.secondary)
                            }
                            .padding(.vertical, 2)
                        }
                    }
                    .frame(maxHeight: 80)
                }
                
                Button {
                    Task { await viewModel.performMigrateAll() }
                } label: {
                    HStack {
                        Spacer()
                        Image(systemName: "tray.and.arrow.down.fill")
                        Text("Migrate All to Canonical")
                        Spacer()
                    }
                    .font(.system(size: 11, weight: .semibold))
                    .padding(.vertical, 4)
                }
                .buttonStyle(.borderedProminent)
                .tint(.orange)
                .disabled(viewModel.state == .syncing)
            }
            .padding(.top, 4)
        } label: {
            HStack {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                Text("\(viewModel.physicalSkills.count) Uncentralized Skill(s)")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(.orange)
                Spacer()
            }
        }
        .accentColor(.orange)
    }

    private var optionsBlock: some View {
        VStack(alignment: .leading, spacing: 6) {
            Toggle(isOn: $viewModel.dryRun) {
                HStack {
                    Image(systemName: "doc.text.magnifyingglass")
                        .foregroundStyle(.secondary)
                    Text("Dry-run (preview only)")
                }
            }
            Toggle(isOn: $viewModel.skipIndex) {
                HStack {
                    Image(systemName: "magnifyingglass.circle")
                        .foregroundStyle(.secondary)
                    Text("Skip CodeGraph re-index")
                }
            }
        }
        .toggleStyle(.checkbox)
        .font(.system(size: 11))
    }

    private var actions: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let summary = viewModel.lastSyncSummary {
                Text(summary)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(summary.contains("failed") || summary.contains("Error") ? .red : .secondary)
                    .lineLimit(2)
                    .padding(6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(
                        RoundedRectangle(cornerRadius: 6)
                            .fill(summary.contains("failed") || summary.contains("Error") ? Color.red.opacity(0.08) : Color.secondary.opacity(0.05))
                    )
            }
            
            HStack {
                Button {
                    Task { await viewModel.performSync() }
                } label: {
                    HStack {
                        Image(systemName: "arrow.triangle.2.circlepath")
                        Text(viewModel.dryRun ? "Preview Sync" : "Sync Now")
                    }
                    .font(.system(size: 12, weight: .bold))
                }
                .buttonStyle(.borderedProminent)
                .tint(.blue)
                .disabled(viewModel.state == .syncing)

                Button {
                    Task { await viewModel.refreshStatus() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 12, weight: .bold))
                }
                .buttonStyle(.bordered)

                Spacer()

                Button("Quit") { NSApplication.shared.terminate(nil) }
                    .buttonStyle(.bordered)
                    .font(.system(size: 11))
            }
        }
    }

    private func abbreviateHome(_ path: String) -> String {
        let home = NSHomeDirectory()
        return path.hasPrefix(home) ? "~" + path.dropFirst(home.count) : path
    }
}
