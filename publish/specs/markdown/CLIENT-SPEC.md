# Unified Client Design Brief & Specification

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.1.0                                                                |
| Authors       | Ckodex Labs                                                          |
| Scope         | All SkillPack client surfaces — consistent interaction model         |
| Target spec   | SkillPack v1.0, CKODEX v16.0                                         |
| Date          | 2026-05-28                                                           |
| Supersedes    | Per-client ad-hoc type definitions                                   |
| Related       | `client-model.schema.json`, `client-api-contract.md`, RFC-001      |

---

## 0. Abstract

SkillPack exposes functionality through six distinct client surfaces: CLI, VS Code Extension, Web Dashboard, macOS Ecosystem (Menu Bar + Daemon + Finder), CNI (Container/Edge Interface), and MCP Server. Today each surface reinvents its own data model, transport layer, error handling, and state management, producing drift, duplicate effort, and inconsistent UX.

This specification defines a **unified client interaction model** — a single canonical data model (JSON Schema), a transport contract with layered fallback, a structured error taxonomy with per-client rendering hints, a state/reactivity protocol, and a feature matrix declaring which operations each client MUST, SHOULD, MAY, or MUST NOT support.

All clients conforming to this spec share:
- The same type definitions (generated from `client-model.schema.json`)
- The same error codes and severity semantics
- The same cache invalidation and optimistic-update behavior
- The same authentication credential model

---

## 1. Client Surface Inventory

| ID | Client | Tech Stack | Transport | Location | Status |
|----|--------|-----------|-----------|----------|--------|
| C-01 | **CLI** (`skillpack`) | Rust (clap) | In-process domain + gRPC fallback | `crates/skillpack-adapters/src/cli/` | Operational |
| C-02 | **VS Code Extension** | TypeScript (VS Code API) | Connect-RPC Node → gRPC | `vscode-extension/` | Operational |
| C-03 | **Web Dashboard** | Next.js 15 + React 19 | Connect-Web → gRPC (REST fallback) | `dashboard/` | Stub — mock client |
| C-04 | **macOS Menu Bar** | SwiftUI | XPC → Daemon → gRPC | `macos/skills-ecosystem/SkillsUI/` | Operational |
| C-05 | **macOS Daemon** | Swift (Foundation) | gRPC + XPC server | `macos/skills-ecosystem/SkillsDaemon/` | Operational |
| C-06 | **macOS Finder** | Swift (Finder Sync) | XPC → Daemon | `macos/skills-ecosystem/SkillsFinder/` | Operational |
| C-07 | **macOS CLI** (`skills-cli`) | Swift (ArgumentParser) | Direct + gRPC | `macos/skills-ecosystem/SkillsCLI/` | **Deprecated** — use `skillpack store <cmd>` |
| C-08 | **CNI** (Container/Edge Interface) | Rust/Go/Shell | gRPC (REST fallback) | `crates/skillpack-cni/` (planned) | **Planned** — headless CI/CD client |
| C-09 | **MCP Server** | Rust | MCP protocol over stdio/sse | `crates/skillpack-adapters/src/mcp/` (planned) | **Planned** — exposes skills as MCP resources |
| C-10 | **Docs Site** | RSPress | Static (informational only) | `docs-site/` | Operational — no runtime API |

### 1.1 CNI Definition

The **Container/Node Interface (CNI)** is a headless, programmatic client designed for CI/CD pipelines, Kubernetes operators, and automated infrastructure. It is non-interactive: all input arrives via environment variables, command-line flags, or stdin JSON; all output is emitted as structured JSON or exit codes. The CNI does not render UI — it is a quality gate and distribution automation layer.

### 1.2 MCP Server Definition

The **Model Context Protocol (MCP) Server** exposes SkillPack skills as MCP resources and tools. It acts as a bridge: AI agents (Claude, Copilot, etc.) discover skills through the MCP protocol and invoke them via the standard skill lifecycle (RFC-001 §3). The MCP server is both a client (reads from SkillPack API) and a server (exposes to MCP consumers).

---

## 2. Canonical Data Model

The shared data model is defined in `client-model.schema.json`. All client types MUST be generated from this single schema.

### 2.1 Core Types

| Type | Purpose | Used By |
|------|---------|---------|
| `SkillSummary` | Lightweight skill list item (name, version, grade, tier) | All clients — list views |
| `SkillDetail` | Full skill with manifest, assessment, lifecycle state | Detail views, editing |
| `AssessmentResult` | Dimension scores, issues, grade, timestamp | Assessment display |
| `SyncStatus` | Daemon health, agent sync state, storage backend | Status panels |
| `RegistryEntry` | Search result, install candidate from remote | Registry browser |
| `LifecycleEvent` | Promote/demote/supersede decision bundle | Audit log, notifications |
| `UserPreference` | Client-agnostic settings | Settings panels |
| `ClientError` | Structured error with code, category, severity, hints | Error rendering |

### 2.2 Type Generation Pipeline

```
client-model.schema.json (single source of truth)
  ├── Rust    → schemars + serde_derive (cargo generate)
  ├── TypeScript → json-schema-to-typescript (npm script)
  └── Swift   → quicktype or custom generator (make rule)
```

**Invariant**: If a type changes in the schema, all generated bindings MUST be regenerated before merge. CI enforces this via `make check-schema-sync`.

### 2.3 Versioning

The schema carries a top-level `schemaVersion` field (integer, starting at 1). Clients MUST reject responses with a schema version greater than their supported maximum, and SHOULD degrade gracefully for versions within their supported range.

---

## 3. Transport Contract

See `client-api-contract.md` for full detail. Summary:

| Layer | Protocol | Port | Clients | Fallback |
|-------|----------|------|---------|----------|
| **Primary** | gRPC/Connect (protobuf) | 50051 | VS Code, Dashboard, macOS Daemon, CNI, MCP | REST on 50052 |
| **Secondary** | REST/JSON | 50052 | Web Dashboard (CORS), CNI (curl), CLI (quick checks) | gRPC on 50051 |
| **Local** | In-process Rust traits | — | CLI only | gRPC to localhost:50051 |
| **IPC** | XPC (macOS) | — | macOS UI, Finder → SkillsDaemon | Direct gRPC to 50051 |

### 3.1 Connection Establishment

All network clients follow this sequence:

1. **Discovery**: Check `SKILLPACK_API_URL` env var; default to `http://localhost:50051`
2. **Health probe**: `GET /health` (REST) or `GetStatus` (gRPC); timeout 2s
3. **Capability negotiation**: Server returns supported protocol version; client selects highest mutual version
4. **Auth**: Bearer token from `SKILLPACK_TOKEN` or platform credential store
5. **Keepalive**: gRPC ping every 30s; REST clients poll `/health` every 60s

### 3.2 Streaming

Long-running operations use gRPC server-streaming:

| Operation | Stream Type | Events |
|-----------|-------------|--------|
| `Assess` | `AssessStream` | `started`, `dimension_complete`, `issue_found`, `completed` |
| `SyncAgents` | `SyncAgentsStream` | `started`, `agent_synced`, `completed`, `failed` |
| `LifecycleWatch` | `LifecycleEventStream` | `promoted`, `demoted`, `superseded` |

REST clients without streaming support receive a single response with a `jobId` and poll `GET /jobs/{jobId}` for progress.

---

## 4. State Management & Reactivity

See `client-state-protocol.md` for full detail. Summary:

### 4.1 Cache Layers

| Data | TTL | Invalidation Trigger |
|------|-----|---------------------|
| Skill list | 5 min | `skill_created`, `skill_deleted`, `skill_updated` |
| Skill detail | 2 min | `skill_updated` for that skill |
| Assessment result | 1 min | `assessment_completed` for that skill |
| Sync status | 30s | `sync_started`, `sync_completed` |
| Registry search | 10 min | manual refresh only |
| User preferences | session | `preference_changed` event |

### 4.2 Optimistic Updates

User-initiated mutations apply optimistically:

1. Client updates local cache immediately
2. Client emits mutation request
3. On success: server response reconciles (usually no-op)
4. On failure: rollback to pre-mutation state + show error

**Conflict resolution**: Server-wins for assessments (computed data); last-write-wins for preferences (user data).

### 4.3 Event Delivery

Clients subscribe to server events via one of:
- **gRPC streaming** (preferred): `LifecycleEventStream`, `SyncStatusStream`
- **Server-Sent Events** (REST): `GET /events` with `text/event-stream`
- **Polling** (fallback): `GET /status` every 30s

---

## 5. Error Taxonomy

See `client-error-codes.md` for full code registry. Summary:

### 5.1 Error Structure

```json
{
  "code": "SKILLPACK_IP_VIOLATION",
  "category": "security",
  "severity": "error",
  "message": "Skill name contains restricted pattern",
  "details": { "violations": ["thales"] },
  "clientHint": {
    "cli": "Print red text with --force flag suggestion",
    "vscode": "Show error notification with 'View Details' button",
    "web": "Display inline banner with dismiss action",
    "macos": "Show alert sheet with 'Override' option",
    "cni": "Emit JSON to stderr with exit code 78 (configuration error)"
  }
}
```

### 5.2 Severity Levels

| Level | Meaning | CLI | VS Code | Web | macOS | CNI |
|-------|---------|-----|---------|-----|-------|-----|
| `fatal` | Unrecoverable; abort | Exit 1 + red | Notification + disable | Full-screen error | Alert modal | Exit 1 + JSON |
| `error` | Operation failed | Red text | Notification | Inline banner | Alert sheet | Exit 78 + JSON |
| `warning` | Degraded but continues | Yellow text | Warning notification | Inline warning | Badge color | Exit 0 + warning JSON |
| `info` | Informational | Gray text | Status bar item | Toast | Log entry | Stdout JSON |

### 5.3 Category Codes

- `VALIDATION` — schema, manifest, or input validation failure
- `SECURITY` — IP boundary, policy, or permission violation
- `NETWORK` — connection, timeout, or transport failure
- `STORAGE` — filesystem, database, or OCI registry error
- `AUTH` — authentication or authorization failure
- `INTERNAL` — unexpected server or client error

---

## 6. Authentication

### 6.1 Credential Model

| Client | Storage | Source | Rotation |
|--------|---------|--------|----------|
| CLI | Env var | `SKILLPACK_TOKEN` | Manual |
| VS Code | VS Code SecretStorage | User prompt → encrypted | Manual |
| Web | Browser localStorage (dev) / HTTP-only cookie (prod) | Login flow | JWT refresh |
| macOS | macOS Keychain | User prompt → keychain | Manual |
| CNI | Env var or mounted secret | `SKILLPACK_TOKEN` or k8s secret | CI rotation |
| MCP | Env var | `SKILLPACK_MCP_TOKEN` | Manual |

### 6.2 Token Format

Bearer tokens are opaque strings. The server validates them against a token store (DuckDB or external identity provider). No client-side token parsing.

---

## 7. Feature Matrix

See `client-feature-matrix.md` for full detail with rationale. Summary:

### 7.1 Operations by Client

| Operation | CLI | VS Code | Web | macOS | CNI | MCP |
|-----------|:---:|:-------:|:---:|:-----:|:---:|:---:|
| Assess skill | M | M | M | S | M | N |
| Grade skill | M | M | M | S | N | N |
| Create skill | M | M | S | N | N | N |
| Edit skill | N | M | S | N | N | N |
| Publish to registry | M | M | N | S | M | N |
| Install from registry | M | M | M | M | M | S |
| Sync to agents | M | S | N | M | M | N |
| View status | M | M | M | M | M | S |
| Lifecycle promote | M | S | S | S | M | N |
| Lifecycle demote | M | S | S | S | M | N |
| IP boundary check | M | M | M | M | M | N |
| Auto-assess on save | N | M | N | N | N | N |
| AI-assisted authoring | N | M | S | N | N | N |
| Registry search | M | M | M | N | M | N |
| Skill detail view | M | M | M | N | N | S |
| MCP resource exposure | N | N | N | N | N | M |

**Legend**: M = MUST, S = SHOULD, O = MAY, N = MUST NOT

### 7.2 Viewport & Interaction Constraints

| Client | Viewport | Primary Input | Offline Strategy |
|--------|----------|---------------|-----------------|
| CLI | 80-char terminal | Keyboard | Queue commands; retry on reconnect |
| VS Code | 300px sidebar, editor | Keyboard + mouse | Cache last assessment; show stale badge |
| Web | Responsive (320px–4K) | Mouse + touch | Service Worker cache; offline page |
| macOS Menu Bar | 320px fixed window | Mouse | Daemon state survives; UI reconnects |
| macOS Finder | Badge + context menu | Mouse | Badge from last sync; manual refresh |
| CNI | None (headless) | stdin/env | Fail-fast; exit non-zero |
| MCP | N/A (protocol) | MCP client | Fail-fast; expose availability |

---

## 8. Accessibility

All graphical clients (VS Code, Web, macOS) MUST support:
- **Screen reader**: All interactive elements have ARIA labels / accessibility descriptions
- **Colorblind-safe**: Error/warning states use icons + text, not color alone
- **Motion-reduced**: Respect `prefers-reduced-motion`; disable spinners/animations
- **Keyboard navigation**: All actions reachable without pointer (Tab/Enter/Escape)

The CLI provides `--no-color` and `--quiet` flags. The CNI emits structured JSON regardless of TTY.

---

## 9. Implementation Roadmap

| Phase | Deliverable | Files | Est. Duration |
|-------|------------|-------|---------------|
| 1 | Schema layer | `client-model.schema.json` + generation scripts | 1 week |
| 2 | Transport contract | `client-api-contract.md` + REST fallback server | 1 week |
| 3 | Error taxonomy + feature matrix | `client-error-codes.md`, `client-feature-matrix.md` | 3 days |
| 4 | State protocol | `client-state-protocol.md` + SSE endpoint | 1 week |
| 5 | Client refactors | Update VS Code, Dashboard, macOS to conform | 2 weeks |
| 6 | CNI implementation | `crates/skillpack-cni/` binary | 1 week |
| 7 | MCP server | `crates/skillpack-adapters/src/mcp/` | 2 weeks |

---

## 10. Appendix A: Type Mapping Reference

| Schema Type | Rust | TypeScript | Swift |
|-------------|------|------------|-------|
| `string` | `String` | `string` | `String` |
| `integer` | `i64` | `number` | `Int` |
| `number` | `f64` | `number` | `Double` |
| `boolean` | `bool` | `boolean` | `Bool` |
| `array` | `Vec<T>` | `T[]` | `[T]` |
| `object` | `struct` | `interface` | `struct` / `class` |
| `enum` | `enum` | union + const | `enum` (raw String) |
| `date-time` | `DateTime<Utc>` | `string` (ISO 8601) | `Date` (ISO 8601) |
| `uuid` | `Uuid` | `string` | `UUID` |

## 11. Appendix B: Glossary

| Term | Definition |
|------|-----------|
| **CNI** | Container/Node Interface — headless programmatic client for CI/CD |
| **MCP** | Model Context Protocol — Anthropic protocol for AI tool/resource exposure |
| **Connect** | Connect-RPC — gRPC-compatible RPC over HTTP/1 and HTTP/2 |
| **XPC** | macOS Inter-Process Communication — secure IPC between app extensions |
| **OCI** | Open Container Initiative — registry standard for artifact distribution |
| **Skill** | A directory containing `SKILL.md`, manifest, and optional resources |
| **Assessment** | 9-dimension quality evaluation of a skill |
| **Canonical Store** | `~/Skills/shared` — central skill repository |
| **Agent** | An AI coding assistant (Claude, Cursor, Copilot, etc.) |
