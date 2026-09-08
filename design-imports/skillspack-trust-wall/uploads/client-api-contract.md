# Client API Contract

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.1.0                                                                |
| Scope         | Transport layer, endpoint mapping, protocol fallback                 |
| Target        | All SkillPack clients (C-01 through C-09)                          |

---

## 1. Protocol Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CLIENT SURFACES                                  │
│  CLI   VS Code   Web Dashboard   macOS UI   macOS Finder   CNI   MCP    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      TRANSPORT LAYER (MULTI-PROTOCOL)                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────┐  │
│  │ gRPC/Connect │   │ REST/JSON    │   │ XPC (macOS)  │   │ In-proc  │  │
│  │ Port 50051   │   │ Port 50052   │   │ Mach IPC     │   │ Rust     │  │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                         SERVER (skillpack-server)                      │
│  SkillPackService │ IndexService │ RatingsService │ QueryService       │
│  CanonicalStoreService │ LifecycleService │ HealthService            │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. gRPC/Connect (Primary) — Port 50051

### 2.1 Services

All services are defined in `proto/skillpack/v1/`:

| Service | File | Purpose |
|---------|------|---------|
| `SkillPackService` | `skillpack.proto` | Assess, Grade, Report, AssessStream |
| `IndexService` | `index.proto` | CreateIndex, GetIndex, ListIndices, AddConstituent, CalculateValue, GetHistory |
| `RatingsService` | `ratings.proto` | GetRating, GetRatingHistory, CompareRatings |
| `QueryService` | `query.proto` | Search, GetSkillProfile, GetCompatibility, GetCostEstimate |
| `CanonicalStoreService` | `canonical.proto` | SyncAgents, CheckBoundary, MigrateAll, ImportSkill, PromoteSkill, GetStatus, SyncAgentsStream |
| `LifecycleService` | *(new)* | WatchLifecycleEvents, PublishLifecycleEvent |
| `HealthService` | *(new)* | Check, Watch |

### 2.2 Connect-RPC Conformance

- **HTTP/1.1 fallback**: Connect supports unary over HTTP/1.1 for environments that cannot use HTTP/2
- **Binary protobuf** for efficiency; JSON protobuf for debugging
- **Compression**: gzip and brotli supported; client advertises via `Accept-Encoding`
- **Timeouts**: Unary RPCs default 30s; streaming RPCs default 5m

### 2.3 Connection Lifecycle

```
CONNECT SEQUENCE:
1. Resolve endpoint (env > config > default localhost:50051)
2. TCP connect with 2s timeout
3. Health Check RPC (HealthService.Check)
4. If health OK → proceed
5. If health FAIL → fallback to REST port 50052
6. If REST also FAIL → enter degraded mode (cached data + queue mutations)
```

### 2.4 Keepalive

```
gRPC keepalive:
  client → server: ping every 30s
  server → client: ping every 30s
  timeout: 10s (connection closed if no ack)

REST fallback:
  poll /health every 60s
  timeout: 5s
```

---

## 3. REST/JSON (Secondary) — Port 50052

### 3.1 Design Principles

- **Read-heavy**: All read operations fully supported
- **Write-limited**: Mutations require explicit `X-SkillPack-Client-Id` header
- **Streaming polyfill**: Long-running ops return `JobStatus` with `jobId`; client polls `GET /jobs/{jobId}`
- **CORS**: Enabled for `http://localhost:*` in development; configurable whitelist in production

### 3.2 Endpoint Mapping

| gRPC Method | REST Equivalent | Method | Request Body |
|-------------|-----------------|--------|--------------|
| `SkillPackService.Assess` | `/api/v1/assess` | POST | `{ skillPath, minScore?, skillName?, skillVersion? }` |
| `SkillPackService.Grade` | `/api/v1/grade` | POST | `{ skillPath, minimumGrade }` |
| `SkillPackService.Report` | `/api/v1/report` | POST | `{ skillPath, format }` |
| `IndexService.Search` | `/api/v1/skills/search` | GET | Query params: `q`, `limit`, `offset` |
| `QueryService.GetSkillProfile` | `/api/v1/skills/{id}` | GET | — |
| `CanonicalStoreService.GetStatus` | `/api/v1/status` | GET | — |
| `CanonicalStoreService.SyncAgents` | `/api/v1/sync` | POST | `{ dryRun, noIndex, onlyAgent? }` |
| `CanonicalStoreService.CheckBoundary` | `/api/v1/boundary-check` | POST | `{ targetPath, skillName? }` |
| `LifecycleService.PublishLifecycleEvent` | `/api/v1/lifecycle` | POST | `LifecycleEvent` JSON |
| `HealthService.Check` | `/health` | GET | — |
| *(any streaming)* | `/api/v1/jobs/{jobId}` | GET | Poll for progress |

### 3.3 REST Response Format

All REST responses wrap in a consistent envelope:

```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "requestId": "uuid",
    "timestamp": "2026-05-28T19:00:00Z",
    "schemaVersion": 1
  }
}
```

Error envelope:

```json
{
  "success": false,
  "error": {
    "code": "SKILLPACK_IP_VIOLATION",
    "category": "security",
    "severity": "error",
    "message": "Skill name contains restricted pattern",
    "details": { "violations": ["thales"] }
  },
  "meta": { "requestId": "uuid", "timestamp": "...", "schemaVersion": 1 }
}
```

### 3.4 REST Authentication

```
Authorization: Bearer <token>
X-SkillPack-Client-Id: <client-id>   // Required for mutations
X-SkillPack-Client-Version: <semver> // Optional, for metrics
```

---

## 4. XPC (macOS IPC)

### 4.1 Architecture

```
SkillsUI (Menu Bar) ──XPC──→ SkillsDaemon ──gRPC──→ skillpack-server
SkillsFinder (Finder) ──XPC──→ SkillsDaemon ──gRPC──→ skillpack-server
```

### 4.2 XPC Protocol

Defined in `macos/skills-ecosystem/SkillsCore/Sources/SkillsCore/SkillsDaemonXPCProtocol.swift`:

```swift
@objc protocol SkillsDaemonXPCProtocol {
    func syncSkills(dryRun: Bool, noIndex: Bool, reply: @escaping (Bool, String) -> Void)
    func getStatus(reply: @escaping (Data?) -> Void)   // Encoded SyncStatus
    func getExtendedStatus(reply: @escaping (Data?) -> Void)
    func checkBoundary(targetPath: String, skillName: String?, reply: @escaping (Bool, [String]) -> Void)
}
```

### 4.3 XPC → gRPC Bridge

The SkillsDaemon acts as a bridge:
- Receives XPC calls from UI/Finder
- Forwards to `skillpack-server` via gRPC (localhost:50051)
- Returns results via XPC reply blocks
- Caches `GetStatus` for 30s to reduce gRPC load

---

## 5. In-Process (CLI)

### 5.1 Direct Domain Access

The Rust CLI (`skillpack`) links directly against `skillpack-domain` and `skillpack-application` crates. No network hop for local operations.

### 5.2 gRPC Fallback

When `--remote` flag is passed or the canonical store is configured for distributed mode:
- CLI uses `CanonicalStoreClient` (gRPC client in `skillpack-adapters/src/grpc_client.rs`)
- Connects to `SKILLPACK_API_URL` (default localhost:50051)

---

## 6. MCP Protocol

### 6.1 Transport

MCP supports two transports:
- **stdio**: MCP client spawns `skillpack-mcp` binary; JSON-RPC over stdin/stdout
- **SSE**: HTTP server mode; MCP client connects to `http://localhost:3001/sse`

### 6.2 Resource Exposure

Each skill is exposed as an MCP resource:

```
resource://skillpack/skills/{skillName}
  → Returns SKILL.md content

resource://skillpack/assessments/{skillName}
  → Returns latest AssessmentResult JSON
```

### 6.3 Tool Exposure

```
tool://skillpack/assess
  → Input: { skillPath }
  → Output: AssessmentResult

tool://skillpack/install
  → Input: { ociRef }
  → Output: { success, installedPath }
```

---

## 7. Health & Discovery

### 7.1 Health Endpoint

```protobuf
service HealthService {
  rpc Check(HealthCheckRequest) returns (HealthCheckResponse);
  rpc Watch(HealthCheckRequest) returns (stream HealthCheckResponse);
}

message HealthCheckRequest {
  string service = 1;  // "" = all services
}

message HealthCheckResponse {
  enum ServingStatus {
    UNKNOWN = 0;
    SERVING = 1;
    NOT_SERVING = 2;
  }
  ServingStatus status = 1;
  repeated ServiceStatus services = 2;
}
```

### 7.2 Server Info Endpoint

`GET /api/v1/info` (REST) or `GetServerInfo` (gRPC):

```json
{
  "name": "skillpack-server",
  "version": "1.0.0",
  "protocolVersion": 1,
  "supportedTransports": ["gRPC", "REST"],
  "features": ["assessment", "grading", "registry", "lifecycle"]
}
```

---

## 8. Rate Limiting

| Client Type | Burst | Sustained | Notes |
|-------------|-------|-----------|-------|
| CLI | 100/min | 30/min | Local ops exempt |
| VS Code | 60/min | 20/min | Auto-assess throttled |
| Web | 120/min | 40/min | Per-session |
| macOS | 60/min | 20/min | Daemon batches requests |
| CNI | 300/min | 100/min | Higher for CI pipelines |
| MCP | 60/min | 20/min | Per-connection |

Rate limit response: HTTP 429 / gRPC `RESOURCE_EXHAUSTED` with `Retry-After` header.

---

## 9. Deprecation Policy

- gRPC methods: deprecated for 2 minor versions, then removed
- REST endpoints: deprecated for 4 minor versions (longer for external consumers)
- Deprecated methods return `Deprecation` header with sunset date
- Clients MUST warn users when calling deprecated endpoints
