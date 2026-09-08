# CKODEX Skill Lifecycle — RFC-001 v0.2.0

A drop-in runtime profile that extends Agent Skills v1 with a six-tier
progressive-disclosure FSM, multi-signal scoring, hysteresis, eviction,
privacy-aware lifecycle evidence, and a capability-negotiation handshake.

The profile is upstream-compatible: it requires no change to the Agent
Skills v1 specification or to the `SKILL.md` format. All extensions
live in the companion `skill.json` manifest defined by the sibling
`ckodex-skill-spec-v1.1` pack.

## What's in this pack

```
ckodex-skill-lifecycle-rfc001/
├── README.md                                       (this file)
├── RFC-001-skill-lifecycle.md                      full v0.2.0 spec
├── CHANGELOG.md                                    v0.1.0 → v0.2.0 delta
├── MANIFEST.json                                   sha256 ledger of every file
├── schemas/
│   ├── pca-lifecycle.v1.schema.json                PCA evidence bundle schema
│   └── skill-runtime-capabilities.v1.schema.json   capability handshake schema
├── reference/
│   ├── __init__.py                                 package interface
│   ├── kernel.py                                   FSM + thresholds + per-turn step()
│   └── scoring.py                                  S_lex / S_sem / S_ont / S_co / S_rec / S_usr + composite
├── conformance/
│   ├── README.md                                   how to run the vectors
│   ├── vectors.json                                12 canonical conformance vectors (A..L)
│   └── run.py                                      kernel runner for A, B, D, F + structural for C/E/G/H/I/J/K/L
├── diagrams/
│   ├── event-loop.{mmd, html, render-manifest.json}    T4 swimlane — controller flow
│   └── fsm-lifecycle.{mmd, html, render-manifest.json} T1 ckodex-native — state space
└── scripts/
    └── validate.sh                                 bundle self-test
```

## Quick start

```bash
# Verify the pack itself (integrity + schemas + reference self-tests + conformance)
./scripts/validate.sh

# Run only the conformance vectors against the reference kernel
python3 conformance/run.py

# Use the reference kernel from your own code
python3 -c "
from reference.kernel import SkillState, Tier, TurnInputs, step
pool = {'sha256:aa': SkillState(skill_id='sha256:aa', skill_name='example')}
inputs = {'sha256:aa': TurnInputs(score=0.72, drift=0.10, s_pol=1, intent_to_invoke=True,
                                   score_components={}, weights={})}
result = step(pool, inputs)
for d in result.decisions:
    print(f'{d.from_tier} -> {d.to_tier}  trigger={d.trigger}')
"
```

The reference kernel is stdlib-only — no embedding model, no vector
store, no network. Plug your own via the `Embedder` and `PCAEmitter`
protocols at the bottom of `reference/kernel.py`.

## The six tiers in one sentence

```
  L0 registered → L1 metadata → L2 synopsis → L3 full SKILL.md → L4R resource read
                                                              ↘ L4X script execute (high-risk)
```

Demotion paths run in the reverse direction. Drift, idle, score decay,
budget pressure, mutex, framework break, and emergency protocols
trigger demotion or eviction.

## Verification

Every file in this pack is hashed in `MANIFEST.json`. The bundle
self-test (`scripts/validate.sh`) re-computes those hashes and runs all
reference self-tests plus the conformance suite. To verify an
extracted archive matches the shipped artifact, run
`./scripts/validate.sh` inside the unpacked tree. Target: 0F / 0W.

## Related

- **`ckodex-skill-spec-v1.1`** — the companion pack defining the `skill.json` manifest format that this lifecycle consumes.
- **Agent Skills v1** (upstream): `https://github.com/agentskills/agentskills` — no changes required.
- **CKODEX v16.0** — the framework this RFC is authored under.

## License

Apache-2.0.
