# Transport equivalence chapter

STX defines one typed graph domain. Three transports project that
domain into wire formats. Each transport has affordances the others
lack, but every operation is reachable through every transport.

This chapter is the cross-walk between the three.

## 1. Side-by-side operation map

| Operation               | REST                                  | gRPC                  | GraphQL                                     |
|-------------------------|---------------------------------------|-----------------------|---------------------------------------------|
| Capability handshake    | `GET /capability` (no auth)           | `GetCapability`       | `query { capability { ... } }`              |
| Harness root            | `GET /harness`                        | `GetHarness`          | `query { harness { ... } }`                 |
| List sessions           | `GET /sessions?cursor=...&limit=...`  | `ListSessions`        | `query { sessions(cursor, limit) { ... } }` |
| One session             | `GET /sessions/{id}`                  | `GetSession`          | `query { session(id) { ... } }`             |
| Skill pool by tier      | `GET /sessions/{id}/skill-states?tier=L3` | `ListSkillStates` | `Session.skillStates(tier: L3)`             |
| Tier transitions        | `GET /sessions/{id}/transitions`      | `ListTransitions`     | `Session.transitions`                       |
| Evidence bundle by ID   | `GET /evidence/{id}`                  | `GetEvidence`         | `query { evidence(id) { ... } }`            |
| Event stream            | `GET /events` (SSE)                   | `StreamEvents`        | `subscription { events { ... } }`           |

## 2. Privacy-mode propagation

| Transport | How the negotiated mode is conveyed                                    |
|-----------|-------------------------------------------------------------------------|
| REST      | `Stx-Privacy-Mode` request header                                       |
| gRPC      | request message field `privacy_mode` + duplicate as `stx-privacy-mode` gRPC metadata |
| GraphQL   | the field on each request type OR `extensions.stx.privacyMode`         |

In all three, the response carries `privacyMode` as a top-level field
of `StxQueryResponse`, and `redacted` lists every field path that
was withheld.

## 3. Pagination

| Transport | Mechanism                                                              |
|-----------|-------------------------------------------------------------------------|
| REST      | `?cursor=...&limit=...` query params; response `pageInfo.cursor`        |
| gRPC      | `cursor` + `limit` request fields; response `page_info.cursor`          |
| GraphQL   | `cursor` + `limit` arguments on the connection type; `pageInfo.cursor`  |

## 4. Affordances unique to each transport

- **REST**: cacheable via `ETag`/`If-None-Match`; observable through any HTTP-aware proxy without protocol-specific tooling.
- **gRPC**: efficient binary serialization; native server-streaming for events; works well behind a service mesh.
- **GraphQL**: client picks the projection (no over-fetching, no N+1); single round-trip graph traversal for complex audits; native subscription transport for events.

## 5. When to pick which

| Scenario                                                  | Best fit              |
|-----------------------------------------------------------|------------------------|
| Quick `curl` audit from a debug shell                      | REST                  |
| High-throughput agent fleet talking to one harness         | gRPC                  |
| Audit dashboard that traverses the graph in one query      | GraphQL               |
| Standards-conscious public exposure (cache, ETag, ACME)    | REST + `public-anchor`|
| Behind-mesh service-to-service                             | gRPC + mTLS           |
| Web UI with reactive subscriptions                         | GraphQL               |

## 6. Conformance is per-transport AND per-role

A publisher that advertises `["rest", "grpc"]` (no GraphQL) is a
conformant *publisher* and the consumer's GraphQL client cannot use
it. Symmetric-conformant implementations advertise all three.

The conformance vectors in `conformance/vectors.json` are
transport-neutral. They exercise the typed domain through the
reference Python implementation. A REST-only adapter passes the same
vectors when wrapped around the same publisher core.

## 7. Mapping pitfalls

| Pitfall                                                                          | Avoidance                                                            |
|----------------------------------------------------------------------------------|----------------------------------------------------------------------|
| GraphQL `Int` is 32-bit; STX `Int` for `sequence` might exceed 2³¹               | Use `Long`/`Int64` or string-typed counters in your SDL extensions   |
| REST verbs for non-CRUD reads ("audit", "verify") tempt POST                     | Don't. STX reads are GETs. If you need a side effect, define an explicit RPC. |
| gRPC enum default is 0 (UNSPECIFIED)                                             | Code generators must check for UNSPECIFIED on every enum read        |
| Privacy-mode header lost across a proxy                                          | Always echo `privacyMode` in the response envelope; check on consumer side |
| OpenAPI `$ref` to an external file may not resolve in older codegen tools        | Inline the schema if your toolchain lacks remote $ref support        |
