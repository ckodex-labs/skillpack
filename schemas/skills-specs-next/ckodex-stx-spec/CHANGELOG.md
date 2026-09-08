# Changelog — CKODEX STX Spec

## [0.1.0] — 2026-05-25

Initial release.

### Added

- **Domain model** (`schemas/stx-domain.v1.schema.json`) — single typed graph that all three transports project. Eight node types: Harness, Session, SkillState, TierTransition, EvidenceBundle, AttestationChain, PolicyGate, CapabilityDescriptor.
- **Event stream contract** (`schemas/stx-event.v1.schema.json`) — per-session monotonic sequence; nine event types.
- **Query response envelope** (`schemas/stx-query-response.v1.schema.json`) — discriminated-union `GraphNode` array, pagination, structured errors, redacted-paths list.
- **JSON-LD context** (`schemas/stx-context-v1.jsonld`) at `https://ckodex.org/stx/v1/`.
- **REST transport** (`protocols/openapi.yaml`) — OpenAPI 3.1. Includes SSE stream for events.
- **gRPC transport** (`protocols/stx.proto`) — proto3. Server-streaming `StreamEvents`.
- **GraphQL transport** (`protocols/schema.graphql`) — SDL with `Query` + `Subscription` roots and union `GraphNodeUnion`.
- **Reference publisher** (`reference/publisher.py`) — stdlib state container, event emission with monotonic sequence, privacy-mode-aware redaction at the boundary.
- **Reference consumer** (`reference/consumer.py`) — envelope + node validator, privacy-mode invariant enforcement, consumer-side redaction utility.
- **Conformance suite** — 10 vectors (P1..P5 publisher; C1..C5 consumer). All exercised against the reference modules.
- **Six worked examples** — handshake, events, queries.
- **Four reference chapters** — SPEC, publisher, consumer, transports.

### Key normative decisions

- `defaultPrivacyMode == "audit-private"` is a schema `const` (vector P1).
- Canonical skill ID is `sha256(name ⊕ namespace ⊕ issuer)`, NOT the bare name. Defends against squatting (vector C4).
- Event sequence is per-session, NOT global. Consumers MUST track `last_seen[sessionId]`.
- GAL ≥ 3 ⇒ every `TierTransition` carries an `evidenceBundleId` (vector P3).
- `public-anchor` mode redacts `skillName` (unless allowlisted), `score`, `turnsSinceLastReference`, `tokenCost`, `locked` (vector P4).
- Capability handshake is unauthenticated. Capability discovery is public; auth happens on actual data requests.
- The domain schema is the source of truth. When OpenAPI/proto/SDL disagree with the schema, the schema wins.

### Compatibility

- v1 publishers and consumers MUST set `apiVersion: "ckodex.org/stx/v1"`.
- A v1 consumer reading a v2 publisher MUST reject mismatched `apiVersion` rather than guess.
- Adding new event types or graph node types is a minor version bump. Removing or renaming fields is a major version bump.

### Out of scope for v0.1

- Production-ready grpcio / FastAPI / strawberry adapters (the reference module is stdlib-only).
- Persistence layer (the reference publisher is in-memory).
- Signing implementation for `regulated-export` mode (cosign / dilithium3 hooks defined; implementation deferred).
- Rekor anchoring (the field exists; uploading and verifying is left to integrators).
