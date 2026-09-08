# Publisher chapter — what a CKODEX harness MUST expose

This chapter is the publisher-side contract. A harness conforming to
STX v1 publisher role MUST satisfy everything below.

## 1. Surface

Every conformant publisher exposes:

| Surface                     | REST                              | gRPC                                       | GraphQL                                   |
|-----------------------------|------------------------------------|---------------------------------------------|-------------------------------------------|
| Capability handshake        | `GET /capability` (no auth)       | `GetCapability`                             | `query { capability { ... } }`            |
| Harness root                | `GET /harness`                    | `GetHarness`                                | `query { harness { ... } }`               |
| Session enumeration         | `GET /sessions`                   | `ListSessions`                              | `query { sessions { ... } }`              |
| Session by ID               | `GET /sessions/{id}`              | `GetSession`                                | `query { session(id) { ... } }`           |
| Skill pool by tier          | `GET /sessions/{id}/skill-states` | `ListSkillStates`                           | `Session.skillStates(tier: ...)`          |
| Tier transitions            | `GET /sessions/{id}/transitions`  | `ListTransitions`                           | `Session.transitions(since: ...)`         |
| Evidence by ID              | `GET /evidence/{id}`              | `GetEvidence`                               | `query { evidence(id) { ... } }`          |
| Event stream                | `GET /events` (SSE)               | `StreamEvents` (server-streaming)           | `subscription { events { ... } }`         |

A v1 publisher MUST advertise at least one of `rest`, `grpc`,
`graphql` in `CapabilityDescriptor.transports`. Symmetric-conformant
implementations advertise all three.

## 2. State machine invariants

These derive from RFC-001 §3 and apply at every publish point:

| Invariant                                                                          | Vector |
|------------------------------------------------------------------------------------|--------|
| `defaultPrivacyMode == "audit-private"`                                            | P1     |
| Session lifecycle emits `session_open` before any other event                      | P2     |
| Per-session event sequence is strictly increasing and unique                       | P2, P5 |
| `evidence_emitted` precedes its referencing `tier_transition` (GAL ≥ 3)            | P2     |
| GAL ≥ 3 ⇒ every TierTransition carries `evidenceBundleId`                          | P3     |
| `EvidenceBundle.predicate == "ckodex/skill-lifecycle@v1"`                          | P3     |
| `EvidenceBundle.digest` matches `^sha256:[0-9a-f]{64}$`                            | P3     |
| `public-anchor` ⇒ `skillName` redacted unless on allowlist                         | P4     |

## 3. Redaction rules (boundary, not internal)

Redaction happens at the API boundary, not in the in-memory graph. The
publisher's internal state may carry every field; what it serializes
out depends on the negotiated `privacyMode`.

Reference implementation: `reference/publisher.py::_redact_skill_state`.

The redacted-field list is returned in the `redacted` array of every
response. Consumers MAY use this to detect what was withheld; the
publisher MUST NOT lie about it.

## 4. Event ordering

Sequence numbers are per-session, not global. A consumer reading two
sessions concurrently may observe interleaved timestamps; sequence
ordering applies only within `sessionId`.

If the publisher's lifecycle FSM emits at a rate above
`maxEventsPerSecond`, the publisher MUST drop the lowest-priority
events (RFC: those with `eventType` in `["capability_updated"]`
first, then `policy_gate_evaluated`, then `skill_registered`) and
SHOULD report the drop count in a follow-up `capability_updated`
event carrying the new counter.

## 5. Authentication

STX itself is auth-scheme-agnostic. The OpenAPI document illustrates
bearer JWTs because most harnesses already use them; gRPC implementations
typically use mTLS + token metadata; GraphQL implementations typically
use the same bearer scheme as REST.

The capability handshake MUST NOT require authentication. Capability
discovery is a public operation — the harness's URN and version are
not secret. (If your deployment treats them as secret, gate the
publisher behind a network ACL rather than refusing the handshake.)

## 6. Versioning

A publisher's `frameworkVersion` MUST match the CKODEX framework
version it was authored against. Mismatch handling:

- Consumer requests with `Stx-API-Version: v1` against a publisher
  serving `apiVersion: "ckodex.org/stx/v1"` → MUST succeed.
- Consumer with `v2` against a `v1` publisher → return error
  `version_mismatch` with HTTP 426 / gRPC `FAILED_PRECONDITION`.
- Adding a new event type or graph node type → minor version bump.
- Removing or renaming a field → major version bump.

## 7. Implementation checklist

The publisher reference at `reference/publisher.py` demonstrates one
working answer. A production publisher would additionally:

- [ ] Persist sessions across restarts (the reference is in-memory).
- [ ] Index by sessionId for O(log n) lookup at scale.
- [ ] Implement rate-limiting per `maxEventsPerSecond`.
- [ ] Sign `regulated-export` responses with cosign or dilithium3.
- [ ] Anchor `EvidenceBundle.digest` in Rekor for GAL ≥ 4.
- [ ] Honor `maxQueryDepth` server-side for GraphQL.
- [ ] Emit OpenTelemetry traces with the negotiated `queryId` as span attribute.
