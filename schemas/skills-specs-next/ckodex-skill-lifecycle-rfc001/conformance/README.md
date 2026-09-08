# Conformance Vectors

Twelve canonical test vectors (A–L) covering the behaviors RFC-001
v0.2.0 introduces. A v1.1-conformant harness MUST pass all twelve.

## Running

```bash
python3 conformance/run.py
```

## What gets exercised against the reference kernel

| ID | Name                             | Kernel-runnable | Notes                                    |
|----|----------------------------------|-----------------|------------------------------------------|
| A  | cold-start OSCAL query           | ✅ full         | L1→L2→L3 promotion on combined signal   |
| B  | topic drift after 6 turns        | ✅ full         | Drift demotion overrides recency         |
| C  | budget pressure                  | structural      | Requires multi-skill pool fixture        |
| D  | mutex                            | ✅ full         | `exclusiveOf` atomic eviction            |
| E  | explicit invocation bypass       | structural      | Requires real explicit-mention parser    |
| F  | implicit invocation disabled     | ✅ full         | `S_pol=0` blocks promotion               |
| G  | malicious ontology               | structural      | JSON Schema validation, not runtime      |
| H  | privacy mode                     | structural      | PCA emitter contract                     |
| I  | script execution gate            | structural      | Requires capability-policy mock          |
| J  | version skew                     | structural      | EP-013 framework integrity hook          |
| K  | duplicate skill names            | structural      | Registry-level canonical-ID computation  |
| L  | synopsis mismatch                | structural      | Build-time validator with embedding      |

The four kernel-runnable vectors fully exercise the FSM transitions
defined in RFC §3.2: promotion, drift demotion, mutex eviction, and
policy-gate denial. The remaining eight vectors are well-formed
fixture specifications that any conformant harness can run against its
own adapter stack (registry, capability policy, PCA emitter,
embedding-based validators).

## Adapting structural vectors to your harness

Each structural vector includes:

- `setup` — initial state to install
- `input` — the turn or action to take
- `expected` — observable assertions

Translate these to your test framework of choice. The reference
implementation deliberately does NOT mock embedders or registries —
those are adapter concerns and would prejudice the test surface.
