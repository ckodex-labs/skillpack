# Changelog — CKODEX Skill Spec

All notable changes to this specification are documented here.
Format follows Keep a Changelog principles.

## [1.1.0] — 2026-05-23

### Added

- `metadata.synopsis` field (string, 1–2048 chars) for the L2 progressive-disclosure tier introduced by RFC-001.
- `ontology` block with JSON-LD `@context` for structural skill semantics:
  - `subjects`, `produces`, `consumes`, `related`, `exclusiveOf`, `triggers`, `galMinimum`, `proofTypes`.
- `runtime` block for harness lifecycle hints:
  - `implicitInvocation`, `minTier`, `maxTier`, `l4xAllowed`.
- `apiVersion` enum extended to accept both `ckodex.org/skill/v1` and `ckodex.org/skill/v1.1`.
- New fixed JSON-LD context document at `https://ckodex.org/skill-context/v1`.
- Bundled examples: `ckodex-oscal-v1.1`, `minimal-v1.1`, `third-party-v1-compat`.
- Validator script (`scripts/validate.py`) with schema selection by `apiVersion`.
- Build-time synopsis helper (`scripts/gen_synopsis.py`).
- Bundle self-test (`scripts/validate.sh`) with sha256 ledger verification.

### Changed

- Schema architecture: v1.1 is a standalone superset schema (not a JSON Schema `allOf` composition over v1). Two independent schemas; validator selects by `apiVersion`. This avoids the well-known 2020-12 composition pitfall where `additionalProperties: false` on the base rejects every extension via `allOf`.

### Compatibility

- A v1 manifest validates unchanged under the v1.1 schema (with `apiVersion` left at v1).
- A v1.1 manifest read by a v1-only or non-CKODEX harness is accepted; the v1.1-only blocks are silently ignored.
- No upstream Agent Skills change required. `SKILL.md` format is untouched.

### Security

- Canonical skill ID defined as `sha256(name ⊕ namespace ⊕ issuer)` to defend against squatting/collision.
- `@context` URI pinned via `const` in schema to defend against ontology-poisoning by context injection.
- `proofTypes` is a closed enum to defend against unknown-predicate attacks.

## [1.0.0] — prior

- Baseline `skill.v1.schema.json`, `skillsbundle.v1.schema.json`.
- Agent Skills v1 upstream compatibility (frontmatter + body).
- No synopsis tier; no ontology; no runtime hints.

---

## Migration Notes

### Authoring a new v1.1 skill

1. Start from `examples/minimal-v1.1.skill.json`.
2. Fill `metadata.name`, `metadata.description`.
3. Run `scripts/gen_synopsis.py` to populate `metadata.synopsis` from your `SKILL.md` body, then hand-edit for clarity.
4. Add `ontology` if your skill participates in proof pipelines or has clear topical subjects.
5. Add `runtime.l4xAllowed: false` if your skill does not need script execution.
6. Validate with `scripts/validate.py path/to/skill.json`.

### Upgrading an existing v1 skill

```bash
# Step 1: bump apiVersion in skill.json
sed -i.bak 's|ckodex.org/skill/v1|ckodex.org/skill/v1.1|' skill.json

# Step 2: generate synopsis
python3 scripts/gen_synopsis.py path/to/skill-root/

# Step 3: validate
python3 scripts/validate.py path/to/skill-root/skill.json

# Steps 4–5: optionally add ontology / runtime blocks by hand
```

If you cannot or do not wish to migrate, leave `apiVersion` at
`ckodex.org/skill/v1`. The skill continues to work on both v1.0 and
v1.1 harnesses; v1.1 harnesses will simply route with `S_ont = 0` and
synthesize L2 content from the `description`.
