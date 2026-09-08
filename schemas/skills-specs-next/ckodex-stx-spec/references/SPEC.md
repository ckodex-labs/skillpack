# CKODEX STX v1 — Narrative Specification

| Field      | Value                                                |
|------------|-------------------------------------------------------|
| Status     | Draft                                                 |
| Version    | 1.0.0                                                 |
| API ID     | `ckodex.org/stx/v1`                                   |
| Domain $id | `https://schemas.ckodex.org/stx-domain.v1.schema.json`|
| Authors    | Ckodex Labs                                           |
| Framework  | CKODEX v16.0                                          |

---

## 1. Scope and goals

This specification defines the Skills Transparency Exchange (STX), a
symmetric protocol for publishing and querying the runtime state of a
CKODEX skill pool.

**Goals:**

1. One typed graph domain that all three transports project: REST, gRPC, GraphQL.
2. Symmetric — every conformant implementation passes the publisher vectors when acting as a harness AND the consumer vectors when acting as a client.
3. Privacy-mode-aware by default. `audit-private` is the v1 default; `debug-local`, `public-anchor`, `regulated-export` are negotiated via the capability handshake.
4. Bridges static manifest (`ckodex-skill-spec-v1.1`) and runtime FSM (`ckodex-skill-lifecycle-rfc001`). STX is the wire format that exposes both at runtime.

**Non-goals:**

- Replacing OpenTelemetry. STX is skill-specific; OTel covers everything else.
- Mandating a specific identity/auth scheme. Bearer JWTs are illustrative.
- Defining the storage layer behind a publisher.

## 2. Domain model

The single source of truth is `schemas/stx-domain.v1.schema.json`. Every
type used by any transport is defined there. The protocol files
(`openapi.yaml`, `stx.proto`, `schema.graphql`) are renderings of the
schema — when they disagree with the schema, the schema wins.

Eight types form the graph:

| Type                  | Cardinality from parent             | Notes                                                                 |
|-----------------------|--------------------------------------|------------------------------------------------------------------------|
| `Harness`             | root                                 | one per publisher                                                      |
| `Session`             | Harness → Session (1..n)            | URN-addressable, tied to one GAL + privacyMode                         |
| `SkillState`          | Session → SkillState (0..n)         | canonical skillId = `sha256(name ⊕ namespace ⊕ issuer)`               |
| `TierTransition`      | SkillState → TierTransition (0..n)  | strictly time-ordered                                                  |
| `EvidenceBundle`      | TierTransition → Evidence (0..1)    | present iff `GAL ≥ 3` OR `privacyMode != debug-local`                  |
| `AttestationChain`    | EvidenceBundle → Attestation (0..1) | present iff `GAL ≥ 4`                                                  |
| `PolicyGate`          | Session → PolicyGate (0..n)         | one per gate evaluation                                                |
| `CapabilityDescriptor`| handshake artifact                  | unauthenticated; always returns `defaultPrivacyMode = audit-private`   |

## 3. Capability handshake (normative)

Before any other request, a consumer MUST call `GetCapability` (gRPC) /
`GET /capability` (REST) / `query { capability { ... } }` (GraphQL).
The handshake is unauthenticated and reveals:

- `stxVersion` — MUST be `"v1"`.
- `transports` — at least one of `rest`, `grpc`, `graphql`. Conformant implementations SHOULD advertise all three.
- `privacyModesSupported` — superset of `["audit-private"]`. Other modes are optional.
- `defaultPrivacyMode` — MUST be `"audit-private"` for v1.
- `maxEventsPerSecond` — publisher's event-stream rate ceiling.
- `maxQueryDepth` — GraphQL-only; depth limit for nested selection.
- `harnessId`, `harnessVersion`, `frameworkVersion` — identity strings.

A publisher MUST refuse any request whose negotiated `privacyMode`
is not in `privacyModesSupported`. The publisher MAY downgrade a
request from a stricter mode to `audit-private` and reflect that in
the response's `privacyMode` field (with the affected fields listed in
`redacted`).

## 4. Privacy modes (normative)

The four modes inherited from RFC-001 §9, applied to STX transport:

| Mode               | What's exposed                                                              | What's redacted                                          |
|--------------------|------------------------------------------------------------------------------|------------------------------------------------------------|
| `debug-local`      | everything                                                                   | nothing                                                    |
| `audit-private`    | structure + identities + scores + transitions + digests (default)            | `tokenCost` (cost model is internal)                       |
| `public-anchor`    | structure + digests + transition counts                                      | `skillName` (unless allowlisted), `score`, `turns*`, `tokenCost`, `locked` |
| `regulated-export` | full visibility, but the response MUST be signed (cosign or dilithium3)     | nothing redacted at the data layer; signing is mandatory   |

A publisher MUST list every redacted field path in the response's
`redacted` array so privileged consumers can detect what was withheld
without leaking what was redacted.

## 5. Symmetric conformance (normative)

A "publisher-conformant" implementation passes vectors **P1..P5**.
A "consumer-conformant" implementation passes vectors **C1..C5**.
A "symmetric-conformant" implementation passes **all 10**.

Vector P3 (`gal-evidence-invariant`) is binding: at GAL ≥ 3, every
`TierTransition` MUST carry an `evidenceBundleId`. Vector C3
(`privacy-mode-invariant`) is binding: `public-anchor` responses MUST
NOT carry `skillName` outside the publisher's public allowlist.

Conformance vectors live in `conformance/vectors.json` and are
exercised by `conformance/run.py`.

## 6. Transports

### 6.1 REST (OpenAPI 3.1)

Source of truth: `protocols/openapi.yaml`.

- All payloads JSON. Negotiated mode passed via `Stx-Privacy-Mode` header.
- Event stream uses Server-Sent Events (SSE) at `GET /events`.
- Pagination via `cursor` query parameter and `pageInfo.cursor` response field.

### 6.2 gRPC (proto3)

Source of truth: `protocols/stx.proto`.

- Server-side streaming RPC for `StreamEvents`.
- All other RPCs are unary.
- Metadata key `stx-privacy-mode` carries the negotiated mode.

### 6.3 GraphQL

Source of truth: `protocols/schema.graphql`.

- `Query` root for reads, `Subscription` root for the event stream.
- The capability handshake is exposed as `Query.capability`.
- `maxQueryDepth` from the handshake enforces a server-side depth limit.

## 7. Event stream invariants (normative)

- **Per-session monotonic sequence.** Consumers MUST reject any event from a session whose `sequence` is ≤ the highest `sequence` already observed for that session.
- **Evidence precedes its referencing transition.** When `GAL ≥ 3` (or `privacyMode != debug-local`), a publisher MUST emit `evidence_emitted` for an `EvidenceBundle` BEFORE emitting the `tier_transition` that references it via `evidenceBundleId`. This prevents the consumer from observing a transition that names an unseen evidence bundle. Vector P2 is binding.
- **Global timestamp ordering MAY be violated.** Cross-session events from different clocks need not be monotonic globally.
- **Replay.** A consumer MAY request replay from a specific sequence number; the publisher SHOULD support a reasonable replay window (RFC: ≥ 1 hour).
- **Backpressure.** A publisher MUST drop events rather than block its lifecycle FSM. Dropped events SHOULD be summarized in a follow-up event of type `capability_updated` (publishers add a `droppedSince` counter).

## 8. Backward and forward compatibility

- A v2 consumer reading a v1 publisher MUST gracefully ignore unknown fields.
- A v1 consumer reading a v2 publisher MUST reject responses where `apiVersion` is not `"ckodex.org/stx/v1"`.
- Adding a new `eventType` is a minor version bump. Removing one is a major version bump.

## 9. Security and threat model

| Threat                                                       | Mitigation                                                              |
|--------------------------------------------------------------|--------------------------------------------------------------------------|
| Information leak via skill names                             | `public-anchor` mode + allowlist                                         |
| Sequence-injection (forged event with high `sequence`)       | Authenticated transport (mTLS / bearer); per-session HMAC RECOMMENDED   |
| Replay against rate-limit budgets                            | Sequence numbers + replay-window check on the consumer side             |
| Cross-tenant correlation                                     | URNs MUST be tenant-scoped; `harnessId` is the trust boundary           |
| Ontology / capability poisoning via injected handshake       | `defaultPrivacyMode = audit-private` is a schema `const`                |
| Side-channel via digests                                     | Digests are over redacted-friendly canonical bundles, never raw prompts |

## 10. Examples

Six bundled in `examples/`:

- `handshake-request.json` + `handshake-response.json`
- `event-tier-promotion.json` + `event-evidence-emit.json`
- `query-skill-pool.json` + `query-skill-pool-response.json`

## 11. References

- RFC-001 Progressive Skill Lifecycle (companion pack `ckodex-skill-lifecycle-rfc001`)
- CKODEX Skill Spec v1.1 (companion pack `ckodex-skill-spec-v1.1`)
- JSON Schema 2020-12: `https://json-schema.org/draft/2020-12/schema`
- OpenAPI 3.1: `https://spec.openapis.org/oas/v3.1.0`
- GraphQL Specification: `https://spec.graphql.org/`
- gRPC proto3: `https://protobuf.dev/programming-guides/proto3/`

## 12. License

Apache-2.0.
