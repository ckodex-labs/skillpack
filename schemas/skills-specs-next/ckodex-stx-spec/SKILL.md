---
name: ckodex-stx-spec
description: Skills Transparency Exchange v1 — symmetric publisher/consumer protocol for exposing and querying CKODEX skill-lifecycle runtime state. Use when designing, implementing, or auditing how a harness publishes skill pool state (tier, score, evidence) or how a client queries it. Covers JSON-LD domain model, JSON Schema 2020-12 contracts, OpenAPI 3.1 REST, gRPC proto3, and GraphQL SDL — all three transports normative.
license: Apache-2.0
metadata:
  version: "0.1.0"
  author: ckodex-labs
---

# ckodex-stx-spec — Skills Transparency Exchange v1

> A symmetric protocol for publishing and querying the runtime state
> of a CKODEX skill pool: which skills are loaded, at what tier, with
> what evidence chain, under what policy.

STX is the wire format that connects two CKODEX specs:

- **`ckodex-skill-spec-v1.1`** — what a skill statically declares (manifest, ontology, runtime hints)
- **`ckodex-skill-lifecycle-rfc001`** — how a harness moves skills through the six-tier FSM at runtime

STX itself defines:

1. A typed graph domain model (`stx-domain.v1.schema.json`)
2. An event stream contract for publish-side state changes (`stx-event.v1.schema.json`)
3. A query/response contract for read access (`stx-query-response.v1.schema.json`)
4. Three normative transports: REST (OpenAPI 3.1), gRPC (proto3), GraphQL (SDL)
5. A capability handshake for transport, privacy mode, and rate-limit negotiation

## When this skill triggers

- "design a transparency API for skill state"
- "expose skill lifecycle state to a client"
- "audit what skills a harness has loaded right now"
- "publish skill tier transitions as events"
- "STX", "Skills Transparency Exchange", "skill graph API"
- "GraphQL / gRPC / OpenAPI for skill state"

## When this skill does NOT trigger

- Authoring a single skill manifest → use `ckodex-skill-spec-v1.1`
- Implementing the FSM kernel inside a harness → use `ckodex-skill-lifecycle-rfc001`
- Generic AI observability / OpenTelemetry → STX is skill-specific, not a replacement for OTel

## What it ships

```
ckodex-stx-spec/
├── SKILL.md, README.md, SPEC.md, CHANGELOG.md, MANIFEST.json
├── schemas/
│   ├── stx-domain.v1.schema.json            graph types (Harness, Session, SkillState, TierTransition, EvidenceBundle, AttestationChain, PolicyGate, CapabilityDescriptor)
│   ├── stx-event.v1.schema.json             publisher event stream
│   ├── stx-query-response.v1.schema.json    consumer query envelope
│   └── stx-context-v1.jsonld                JSON-LD @context for the domain
├── protocols/
│   ├── openapi.yaml                         OpenAPI 3.1 (REST)
│   ├── stx.proto                            gRPC proto3
│   └── schema.graphql                       GraphQL SDL
├── reference/
│   ├── __init__.py
│   ├── publisher.py                         harness-side stub
│   └── consumer.py                          client-side stub
├── conformance/
│   ├── vectors.json                         10 canonical vectors (P1..P5, C1..C5)
│   └── run.py                               kernel-runnable runner
├── examples/
│   ├── handshake-request.json
│   ├── handshake-response.json
│   ├── event-tier-promotion.json
│   ├── event-evidence-emit.json
│   ├── query-skill-pool.json
│   └── query-skill-pool-response.json
├── diagrams/
│   ├── domain-graph.{html, mmd, render-manifest.json}    T1 ckodex-native
│   └── publish-subscribe.{html, mmd, render-manifest.json} T4 swimlane
└── scripts/
    └── validate.sh                          bundle self-test (8 sections)
```

## Symmetric protocol (key idea)

STX has two roles that share one domain model:

- **Publisher** (harness-side) exposes `GET /stx/v1/harness`, `GET /stx/v1/sessions/{id}`, and emits an event stream `STREAM /stx/v1/events`. It MUST publish capability descriptor, current skill pool state, every tier transition, and every evidence bundle.
- **Consumer** (client-side) queries the same domain through any of three transports and receives identical typed responses across all of them.

Both roles use the same `stx-domain.v1.schema.json` types. A conformant implementation passes the publisher vectors (P1..P5) AND the consumer vectors (C1..C5) when acting in the respective role.

## Default privacy mode

STX defaults to `audit-private`. Other modes (`debug-local`, `public-anchor`, `regulated-export`) are negotiated per session via the capability handshake. The default is enforced by the `defaultPrivacyMode` constant in `stx-domain.v1.schema.json`.

## Verification

```bash
bash scripts/validate.sh
# expect: STATUS: 0F / 0W — PASS
```

The bundle self-test covers: file presence, JSON parse, JSON Schema validity, OpenAPI/proto/GraphQL syntactic validity, conformance vectors P1..P5 + C1..C5, MANIFEST.json sha256 ledger.

## References

- `references/SPEC.md` — full narrative specification
- `references/publisher.md` — publisher chapter (what a harness MUST expose)
- `references/consumer.md` — consumer chapter (query patterns + GraphQL)
- `references/transports.md` — how the three transports relate to one shared domain
