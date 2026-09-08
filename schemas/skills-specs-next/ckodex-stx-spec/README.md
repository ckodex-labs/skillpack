# CKODEX STX Spec v0.1.0 — Skills Transparency Exchange

A symmetric publisher/consumer protocol for exposing and querying the
runtime state of a CKODEX skill pool. One typed graph domain; three
normative transports (REST, gRPC, GraphQL).

## What's in this pack

```
ckodex-stx-spec/
├── SKILL.md, README.md, CHANGELOG.md, MANIFEST.json
├── skill.json                                       v1.1 manifest (dogfooded)
├── schemas/
│   ├── stx-domain.v1.schema.json                    typed graph (source of truth)
│   ├── stx-event.v1.schema.json                     publisher event stream
│   ├── stx-query-response.v1.schema.json            consumer query envelope
│   └── stx-context-v1.jsonld                        JSON-LD @context
├── protocols/
│   ├── openapi.yaml                                 OpenAPI 3.1 (REST)
│   ├── stx.proto                                    gRPC proto3
│   └── schema.graphql                               GraphQL SDL
├── reference/
│   ├── __init__.py
│   ├── publisher.py                                 publisher state container + emitter
│   └── consumer.py                                  query-response validator
├── conformance/
│   ├── vectors.json                                 10 vectors (P1..P5, C1..C5)
│   └── run.py                                       kernel-runnable runner
├── references/
│   ├── SPEC.md                                      narrative specification
│   ├── publisher.md                                 publisher chapter
│   ├── consumer.md                                  consumer chapter
│   └── transports.md                                transport equivalence
├── examples/                                        6 worked examples
└── scripts/
    └── validate.sh                                  bundle self-test
```

## Quick start

```bash
# Verify the pack
bash scripts/validate.sh

# Run conformance vectors
python3 conformance/run.py

# Use the reference publisher
python3 -c "
from reference.publisher import Publisher, sha256_hex
p = Publisher(harness_id='urn:ckodex:harness:demo', harness_version='1.0.0', framework_version='v16.0')
s = p.open_session('urn:ckodex:session:demo', gal=3, privacy_mode='audit-private')
rec = p.register_skill(s.id, sha256_hex('ckodex-oscal'), 'ckodex-oscal')
p.transition(s.id, rec.id, to_tier='L2', trigger='score_threshold', score=0.55)
p.transition(s.id, rec.id, to_tier='L3', trigger='intent_to_invoke', score=0.72)
resp = p.query_session(s.id, privacy_mode='audit-private')
import json; print(json.dumps(resp, indent=2))
"
```

## Conformance

Pass criteria: `bash scripts/validate.sh` returns `STATUS: 0F / 0W — PASS`.

The bundle self-test exercises:

1. Required files on disk
2. JSON Schema 2020-12 validity for all three schema files
3. YAML / proto / GraphQL syntactic parse for the three protocol files
4. Reference publisher self-test
5. Reference consumer self-test
6. All 10 conformance vectors (P1..P5 + C1..C5)
7. JSON parse of all 6 examples
8. MANIFEST.json sha256 ledger

## Default privacy mode

`audit-private`. The default is enforced by the `defaultPrivacyMode`
`const` in `stx-domain.v1.schema.json`. Publishers that violate this
fail vector P1; consumers that accept a non-`audit-private` default
fail vector C1.

## Related

- `ckodex-skill-spec-v1.1` — companion pack defining the static skill manifest format STX exposes.
- `ckodex-skill-lifecycle-rfc001` — companion pack defining the runtime FSM whose transitions STX publishes.
- `ckodex-skill-tools` — companion pack for scaffolding/migrating skills that this protocol exposes.

## License

Apache-2.0.
