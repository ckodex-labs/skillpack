# CKODEX Skill Spec v1.1

A composition-safe, backward-compatible extension to the Agent Skills v1
format adding:

- **`metadata.synopsis`** — a precomputed bounded synopsis (~400 tokens) for an L2 progressive-disclosure tier
- **`ontology`** — a JSON-LD ontology block declaring `subjects`, `produces`, `consumes`, `related`, `exclusiveOf`, `triggers`, `galMinimum`, and `proofTypes`
- **`runtime`** — runtime hints (`implicitInvocation`, `minTier`, `maxTier`, `l4xAllowed`) consumed by the harness lifecycle manager

The upstream `SKILL.md` format is unchanged. CKODEX-aware harnesses
additionally read the sibling `skill.json` manifest. Harnesses without
CKODEX support ignore the companion and rely on `SKILL.md` exclusively.

## What's in this pack

```
ckodex-skill-spec-v1.1/
├── README.md                                       (this file)
├── SPEC.md                                         narrative specification
├── CHANGELOG.md                                    v1 → v1.1 delta
├── MANIFEST.json                                   sha256 ledger of every file
├── schemas/
│   ├── skill.v1.schema.json                        baseline v1 (unchanged)
│   ├── skill.v1.1.schema.json                      v1.1 delta (allOf + unevaluatedProperties)
│   ├── skillsbundle.v1.schema.json                 bundle manifest
│   └── skill-context-v1.jsonld                     JSON-LD context for the ontology
├── examples/
│   ├── ckodex-oscal-v1.1.skill.json                full-featured v1.1 example
│   ├── minimal-v1.1.skill.json                     minimum-viable v1.1 example
│   └── third-party-v1-compat.skill.json            v1 manifest (backward-compat)
└── scripts/
    ├── validate.py                                 manifest validator (jsonschema-based)
    ├── gen_synopsis.py                             build-time synopsis helper
    └── validate.sh                                 bundle self-test (this pack)
```

## Quick start

```bash
# Verify the pack itself (integrity + schemas + examples)
./scripts/validate.sh

# Validate your own skill manifest
python3 scripts/validate.py path/to/your/skill.json

# Generate a synopsis at build time from SKILL.md
python3 scripts/gen_synopsis.py path/to/skill-root/
```

Dependencies for `validate.py`:

```bash
python3 -m pip install --user jsonschema referencing
```

The bundle self-test (`validate.sh`) gracefully skips schema validation
if those packages are absent and still verifies file presence, JSON
parse, and `MANIFEST.json` sha256 integrity.

## Verification

Every file in this pack is hashed in `MANIFEST.json`:

```json
{
  "schemaVersion": "ckodex.org/pack-manifest/v1",
  "pack": "ckodex-skill-spec-v1.1",
  "files": [
    { "path": "schemas/skill.v1.1.schema.json", "sha256": "sha256:..." },
    ...
  ]
}
```

`validate.sh` re-computes those hashes and refuses to claim 0F/0W if any
file has drifted from its recorded digest. To verify an extracted
archive matches the shipped artifact, run `./scripts/validate.sh` inside
the unpacked tree.

## Compatibility matrix

| Manifest         | Harness v1.0       | Harness v1.1 (CKODEX)            |
|------------------|--------------------|----------------------------------|
| v1 (no extras)   | full               | full + degraded routing (S_ont=0) |
| v1.1 (extras)    | reads SKILL.md only<br>(ignores ontology/runtime) | full ontology-aware routing       |

v1 and v1.1 are independent standalone schemas. The validator selects
by `apiVersion`. v1.1 is a structural superset of v1 with three
optional extensions (`metadata.synopsis`, `ontology`, `runtime`).

## Related

- **RFC-001: Progressive Skill Lifecycle Runtime Profile** — the harness-side runtime profile that consumes the v1.1 ontology and runtime blocks. Ships as the companion `ckodex-skill-lifecycle-rfc001` pack.
- **CKODEX v16.0** — the framework this spec is authored under.

## License

Apache-2.0.
