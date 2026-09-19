# macOS Skills Ecosystem → Rust Migration

## Overview

The macOS Skills Ecosystem has been refactored to delegate canonical store logic to the Rust `skillpack-server`. The macOS layer is now a **thin client** responsible for:

- **Filesystem watching** (`FSEventStream` via `DirectoryWatcher`)
- **UI** (`SkillsUI` Menu Bar, `SkillsFinder` Finder Extension)
- **Local IPC** (XPC between CLI/Daemon/UI)

All business logic has been migrated to the Rust workspace at the monorepo root.

## What Moved to Rust

| Swift Module                      | File                                               | Rust Replacement                                                                         |
| --------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `SyncManager`                     | `SkillsCore/SyncManager.swift`                     | `SyncCanonicalStoreUseCase` (`crates/skillpack-application/src/sync_canonical_store.rs`) |
| `IPGuard`                         | `SkillsCore/IPGuard.swift`                         | `IPGuard` (`crates/skillpack-domain/src/ip_guard.rs`)                                    |
| `MigrateAllStrategy`              | `SkillsCore/MigrateAllStrategy.swift`              | `MigrateSkillsUseCase` (`crates/skillpack-application/src/migrate_skills.rs`)            |
| `AgentSyncer`                     | `SkillsCore/AgentSyncer.swift`                     | `AgentSyncer` adapter (`crates/skillpack-adapters/src/canonical/`)                       |
| `ManifestGenerator`               | `SkillsCore/ManifestGenerator.swift`               | `ManifestGenerator` adapter                                                              |
| `SkillCrafter`                    | `SkillsCore/SkillCrafter.swift`                    | `skillpack wizard` CLI command                                                           |
| `SkillImporter`                   | `SkillsCore/SkillImporter.swift`                   | `CanonicalStoreService::ImportSkill` gRPC method                                         |
| `LocalSkillStorageProvider`       | `SkillsCore/LocalSkillStorageProvider.swift`       | Rust filesystem adapters                                                                 |
| `DistributedSkillStorageProvider` | `SkillsCore/DistributedSkillStorageProvider.swift` | Rust persistence layer                                                                   |
| `FoundationDBClient`              | `SkillsCore/FoundationDBClient.swift`              | Removed — DuckDB is the persistence store                                                |
| `RustFSClient`                    | `SkillsCore/RustFSClient.swift`                    | Removed — superseded by gRPC client                                                      |

## New gRPC Service

The Rust server exposes `CanonicalStoreService` on `[::1]:50051`:

- `SyncAgents` — trigger canonical store sync to agents
- `CheckBoundary` — IP boundary check
- `MigrateAll` — migrate physical skills to canonical store
- `ImportSkill` — import external skill into canonical store
- `PromoteSkill` — promote candidate to active store
- `GetStatus` — get canonical store status
- `SyncAgentsStream` — streaming sync progress

Proto definition: `proto/skillpack/v1/canonical.proto`

## macOS / iOS Client Architecture

```
SkillsUI / iOS App
         │
         ├─── HTTP REST ─────┬──► skillpack-server  :8080
         │                   │    (skills, ratings, search)
         │                   │
         └─── WebSocket ─────┴──► /ws  (real-time events)
         │
         ▼
    SkillPackGRPCClient  (gRPC to Rust server)
         │
         ▼
   skillpack-server  [::1]:50051
```

### Transport Modes

| Transport | Endpoint                     | Use Case                                                         |
| --------- | ---------------------------- | ---------------------------------------------------------------- |
| HTTP REST | `SKILLPACK_SERVER_HOST:8080` | CRUD on skills, ratings, search                                  |
| WebSocket | `/ws`                        | Real-time `ServerEvent` broadcast (sync progress, skill changes) |
| gRPC      | `50051`                      | Canonical store sync, migration, import                          |

### Swift Client Types

| Type                     | File                                      | Purpose                                                       |
| ------------------------ | ----------------------------------------- | ------------------------------------------------------------- |
| `SkillPackBackendClient` | `SkillsCore/SkillPackBackendClient.swift` | `async/await` HTTP client with W3C trace context              |
| `WebSocketEventStream`   | `SkillsCore/WebSocketEventStream.swift`   | `URLSessionWebSocketTask` wrapper with `ServerEvent` decoding |

## Environment Variables

| Variable                | Default           | Purpose               |
| ----------------------- | ----------------- | --------------------- |
| `SKILLPACK_SERVER_HOST` | `127.0.0.1`       | Rust gRPC server host |
| `SKILLPACK_SERVER_PORT` | `50051`           | Rust gRPC server port |
| `SKILLS_ROOT`           | `~/Skills/shared` | Canonical store path  |

## Generating Swift Proto Stubs

```bash
cd macos/skills-ecosystem
make generate-protos
```

Requires: `protoc`, `swift-protobuf` plugin, `grpc-swift` plugin.

## Remaining macOS Responsibilities

| Component          | Responsibility                                                            |
| ------------------ | ------------------------------------------------------------------------- |
| `DirectoryWatcher` | `FSEventStream` watching canonical store, notifying Rust server on change |
| `SkillsUI`         | Menu Bar UI displaying status, triggering manual syncs                    |
| `SkillsFinder`     | Finder badges and context menu for quick sync                             |
| `SkillsDaemon`     | Filesystem watcher + XPC listener (delegates to Rust)                     |
| `SkillsCLI`        | Thin CLI wrapper calling Rust gRPC API                                    |
