# Consumer chapter — query patterns + validation contract

This chapter is the consumer-side contract. A client conforming to STX
v1 consumer role MUST satisfy everything below.

## 1. Handshake first

Before any other operation, a consumer MUST:

1. Call `GetCapability` (gRPC) / `GET /capability` (REST) / `query { capability { ... } }` (GraphQL).
2. Verify `stxVersion == "v1"`. If not, refuse to proceed.
3. Verify the publisher advertises at least one transport the consumer supports.
4. Verify `defaultPrivacyMode == "audit-private"`. If not, the publisher is non-conformant and the consumer MUST refuse to use it.
5. Verify `audit-private ∈ privacyModesSupported`. If not, refuse.

A consumer MAY then negotiate a stricter mode (`public-anchor`,
`regulated-export`) by passing it in subsequent requests. A consumer
MUST NOT negotiate `debug-local` unless it has direct authorization
from the publisher operator.

## 2. Envelope validation (every response)

Every consumer SHOULD pipe responses through a validator equivalent
to `reference/consumer.py::validate_query_response`. Required checks:

| Check                                                  | Vector |
|--------------------------------------------------------|--------|
| `apiVersion == "ckodex.org/stx/v1"`                    | C1     |
| `kind == "StxQueryResponse"`                           | C1     |
| `queryId` is present and non-empty                     | C1     |
| `respondedAt` is a parseable ISO 8601 timestamp        | C1     |
| `privacyMode` is in the four-mode enum                 | C1     |
| Every `result.nodes[].@type` is in the closed type set | C2     |
| Privacy-mode invariants enforced per `@type`           | C3     |
| `skillId` matches `^sha256:[0-9a-f]{64}$` when present | C4     |
| `score` is in `[0,1]` when present                     | C5     |

A failing validator means the publisher is non-conformant. The
consumer SHOULD NOT use the data and SHOULD log the validator's
error list.

## 3. Privacy-mode invariants the consumer enforces

The consumer's validator MUST flag the following even though the
publisher *should* have prevented them:

- `privacyMode == "public-anchor"` AND `SkillState.skillName` is present
  → `error_code: privacy_violation`. Reject the response.
- `privacyMode != "regulated-export"` AND the response lacks a
  signature → consumer MAY warn but MUST NOT reject for this alone
  (signing is publisher-side).

The consumer's defense-in-depth is the entire point of the validator.

## 4. Sequence-ordering responsibility

When consuming the event stream, the consumer MUST:

- Track the highest `sequence` seen per `sessionId`.
- Reject any incoming event whose `sequence ≤ last_seen[sessionId]`.
- On replay request (REST `?since=`, gRPC `since_sequence`, GraphQL `sinceSequence`), reset `last_seen[sessionId]` to `since - 1`.

A consumer that fails to enforce this is vulnerable to forged event
injection. The publisher's authentication layer is the first line of
defense; the consumer's sequence check is the second.

## 5. Query patterns

### 5.1 "What skills are loaded right now"

REST:
```http
GET /stx/v1/sessions/{sid}/skill-states
Stx-Privacy-Mode: audit-private
```

GraphQL:
```graphql
query LoadedSkills($sid: Urn!) {
  session(id: $sid) {
    skillStates {
      id
      tier
      skillName
      score
    }
  }
}
```

### 5.2 "Show me only L3 skills"

REST: `GET /sessions/{sid}/skill-states?tier=L3`

GraphQL:
```graphql
query L3Only($sid: Urn!) {
  session(id: $sid) { skillStates(tier: L3) { id tier score } }
}
```

### 5.3 "Audit one skill's history"

REST:
```http
GET /sessions/{sid}/transitions?since=2026-05-25T00:00:00Z
```

GraphQL (graph traversal in one query):
```graphql
query History($sid: Urn!) {
  session(id: $sid) {
    skillStates {
      id
      transitions {
        timestamp
        fromTier
        toTier
        trigger
        evidenceBundle {
          digest
          rekorAnchor
        }
      }
    }
  }
}
```

The GraphQL traversal returns transitions and their evidence bundles
in a single round trip; REST requires N+1.

### 5.4 "Verify an evidence bundle"

```http
GET /stx/v1/evidence/{evidenceId}
```

Returns a single `EvidenceBundle` node. The consumer MAY then verify
`digest`, follow `attestationChainId` for the signatures, and check
`rekorAnchor` against a Rekor transparency log.

## 6. Consumer-side redaction

If the consumer's policy is stricter than the publisher's, the
consumer MAY further strip fields locally using
`reference/consumer.py::redact_for_mode`. This does NOT replace the
publisher's redaction — it adds defense in depth on the consumer side.

## 7. Implementation checklist

- [ ] Capability handshake on every connection (cache for ≤ 5 min, then re-verify).
- [ ] Pipe every response through `validate_query_response`.
- [ ] Track per-session `last_seen[sequence]` for event streams.
- [ ] Refuse `publishers` that advertise a non-`audit-private` default.
- [ ] Surface validator errors to the user, not silently swallow them.
- [ ] If using GraphQL, respect `maxQueryDepth` from the handshake — don't author queries deeper than the publisher will accept.
- [ ] Tag outgoing requests with a `queryId` (UUIDv4) for end-to-end tracing.
