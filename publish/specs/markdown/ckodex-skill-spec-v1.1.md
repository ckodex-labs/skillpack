# CKODEX Skill Spec v1.1 — Narrative Specification

| Field          | Value                                                    |
|----------------|----------------------------------------------------------|
| Status         | Draft                                                    |
| Version        | 1.1.0                                                    |
| Supersedes     | v1 (extends; backward-compatible)                        |
| Schema $id     | `https://schemas.ckodex.org/skill.v1.1.schema.json`      |
| JSON-LD ctx    | `https://ckodex.org/skill-context/v1`                    |
| Authors        | Ckodex Labs                                              |
| Framework      | CKODEX v16.0                                             |

---

## 1. Scope and Goals

This document specifies the CKODEX companion manifest `skill.json` at
version 1.1. The companion is a sibling file to the upstream Agent
Skills v1 `SKILL.md` and carries semantic, governance, and runtime
extensions that the upstream format does not define.

**Goals:**

1. Add a precomputed synopsis tier between metadata and full body for progressive disclosure (RFC-001 §3.3).
2. Expose structural semantics for ontology-aware skill routing (RFC-001 §4.1, S_ont signal).
3. Carry runtime hints for harness lifecycle management (RFC-001 §4.4 policy pre-filter).
4. Preserve full backward compatibility — v1 manifests validate under v1.1.

**Non-goals:**

- Modifying the upstream `SKILL.md` format or frontmatter.
- Mandating a specific embedding model, vector store, or runtime.
- Requiring any change to the upstream Agent Skills v1 specification.

## 2. Directory Layout

```
skill-root/
  SKILL.md                # Agent Skills v1 — upstream-compatible (unchanged)
  skill.json              # CKODEX companion (this spec)
  references/             # per upstream spec
  scripts/                # per upstream spec
  assets/                 # per upstream spec
```

`SKILL.md` remains the source of truth for the human-readable skill
body. `skill.json` carries machine-consumable metadata, ontology, and
runtime extensions. The two files MUST be consistent — see §6 for the
synopsis-consistency rule.

## 3. Manifest Structure

```jsonc
{
  "apiVersion": "ckodex.org/skill/v1.1",
  "kind": "Skill",
  "metadata": {
    "name": "...",          // required, lowercase-alphanumeric + hyphens, 1–64 chars
    "description": "...",   // required, 1–1024 chars
    "synopsis": "...",      // OPTIONAL (SHOULD for CKODEX-shipped), 1–2048 chars
    "version": "...",       // SemVer recommended
    "license": "...",
    "authors": [...],
    "keywords": [...],
    "labels":      { ... },
    "annotations": { ... }
  },
  "entrypoints": { "skillMd": "SKILL.md" },  // required

  // Optional blocks below — all skill.v1 properties remain available.
  "ontology": {
    "@context": "https://ckodex.org/skill-context/v1",
    "subjects":    [...],
    "produces":    [...],
    "consumes":    [...],
    "related":     [...],
    "exclusiveOf": [...],
    "triggers":    [...],
    "galMinimum":  0..5,
    "proofTypes":  [PCA|UCA|TIP|ZKP|DCA|TKP|WCAG|CLP|UFP, ...]
  },
  "runtime": {
    "implicitInvocation": true|false,
    "minTier": "L0"|"L1"|"L2"|"L3"|"L4R"|"L4X",
    "maxTier": "L1"|"L2"|"L3"|"L4R"|"L4X",
    "l4xAllowed": true|false
  }
}
```

## 4. Field Reference

### 4.1 `metadata.synopsis`

A bounded string (1–2048 chars) precomputed at bundle build time. It
represents the L2 synopsis tier in the RFC-001 progressive-disclosure
FSM and answers four questions about the skill:

1. What does it do?
2. When should the harness trigger it?
3. What does it produce?
4. What does it explicitly NOT cover?

The synopsis SHOULD be a faithful compression of the `SKILL.md` body. A
synopsis–body semantic-consistency check (cosine ≥ 0.55) is RECOMMENDED
at build time. CKODEX-shipped skills MUST include a synopsis;
third-party skills MAY omit it (harnesses will synthesize L2 content
from the description plus the first bounded section of `SKILL.md`).

### 4.2 `ontology`

Optional. A JSON-LD block with a fixed `@context` pointing to
`https://ckodex.org/skill-context/v1`. All array fields are sets
(unique items). Field semantics:

| Field         | Type          | Semantics                                                                                          |
|---------------|---------------|-----------------------------------------------------------------------------------------------------|
| `subjects`    | array<string> | Topic IDs this skill covers. Drives Jaccard overlap for `S_ont` scoring (RFC-001 §4.1).            |
| `produces`    | array<string> | Outputs/proof types/artifact classes this skill emits. Enables pre-warming of downstream skills.   |
| `consumes`    | array<string> | Inputs/proof types this skill ingests. Mirror side of `produces`.                                  |
| `related`     | array<string> | Sibling skill names that often appear together. Weighted at 0.5× in `S_ont`.                       |
| `exclusiveOf` | array<string> | Mutex set: when this skill is promoted, the harness MUST evict every skill named here atomically. |
| `triggers`    | array<string> | Lexical trigger phrases for BM25 / `S_lex` scoring.                                                |
| `galMinimum`  | integer 0..5  | Minimum Governance Autonomy Level admissible for this skill. Hard policy gate.                     |
| `proofTypes`  | array<enum>   | Proof types this skill participates in. Allowed: `PCA UCA TIP ZKP DCA TKP WCAG CLP UFP`.            |

### 4.3 `runtime`

Optional. Runtime hints consumed by the harness lifecycle manager.

| Field                | Type     | Default | Semantics                                                       |
|----------------------|----------|---------|-----------------------------------------------------------------|
| `implicitInvocation` | boolean  | `true`  | If `false`, the harness MUST NOT auto-promote. Explicit only.   |
| `minTier`            | enum     | —       | Lower bound on the FSM tier where this skill may sit.            |
| `maxTier`            | enum     | —       | Upper bound on the FSM tier (e.g. `L4R` blocks all script exec). |
| `l4xAllowed`         | boolean  | `false` | If `false`, the harness MUST refuse `L3 → L4X` for this skill.   |

## 5. Validation Rules (the eight checks)

A v1.1 manifest passes validation when **all** of the following hold:

| §    | Check                                                                                                                          |
|------|--------------------------------------------------------------------------------------------------------------------------------|
| §5.1 | Validates against `skill.v1.1.schema.json` with `additionalProperties: false` at every level (closed schema).                  |
| §5.2 | If `apiVersion` = `ckodex.org/skill/v1` the v1 schema applies; the v1.1-only fields (synopsis, ontology, runtime) are forbidden. |
| §5.3 | If present: `metadata.synopsis` length ∈ `[1, 2048]`.                                                                          |
| §5.4 | If present: `ontology.@context` = `https://ckodex.org/skill-context/v1`.                                                       |
| §5.5 | If present: all `ontology` array fields contain unique items.                                                                  |
| §5.6 | If present: `ontology.galMinimum` ∈ `[0, 5]` and `ontology.proofTypes` ⊆ enum.                                                 |
| §5.7 | If present: `runtime.maxTier` ≥ `runtime.minTier` (lexicographic-by-tier-order, with `L4R = L4X` as a peer pair).               |
| §5.8 | RECOMMENDED build-time check: cosine(embed(synopsis), embed(SKILL.md)) ≥ 0.55.                                                 |

Target for CKODEX-shipped skills: **0 failures / 0 warnings** under all
eight checks.

## 6. Backward Compatibility

A v1 manifest (no `synopsis`, `ontology`, or `runtime`, `apiVersion` =
`ckodex.org/skill/v1`) validates under `skill.v1.schema.json`. A v1.1
manifest (`apiVersion` = `ckodex.org/skill/v1.1`) validates under
`skill.v1.1.schema.json`. The validator selects by `apiVersion`. The
two schemas are independent — v1.1 is a structural superset of v1 with
three optional extensions, but it is NOT a JSON Schema `allOf`
composition over v1. This deliberately avoids the well-known
JSON Schema 2020-12 composition pitfall where `allOf` with
`additionalProperties: false` on the base schema rejects every
extension.

A v1.1 manifest read by a v1-only harness or by a non-CKODEX harness:
the harness reads the standard fields and ignores the unknown
`synopsis`, `ontology`, and `runtime` keys. Behavior degrades to v1
semantics — no progressive disclosure beyond metadata, no
ontology-aware routing.

## 7. Migration v1 → v1.1

| Step | Action                                                                                                                              |
|------|--------------------------------------------------------------------------------------------------------------------------------------|
| 1    | Bump `apiVersion` to `ckodex.org/skill/v1.1` in `skill.json`. (`SKILL.md` unchanged.)                                              |
| 2    | Run `scripts/gen_synopsis.py path/to/skill-root/` to populate `metadata.synopsis`. CKODEX-shipped skills MUST do this.            |
| 3    | OPTIONAL: add the `ontology` block. Recommended for any skill that participates in proof pipelines or has clear topical subjects. |
| 4    | OPTIONAL: add the `runtime` block. Required if you need to forbid implicit invocation or block L4X.                                |
| 5    | Re-validate with `scripts/validate.py`. Target 0F/0W.                                                                              |
| 6    | Re-pack and ship; supersedes the v1 artifact via standard semver promotion.                                                        |

## 8. JSON-LD Context

The fixed `@context` at `https://ckodex.org/skill-context/v1` maps the
ontology field names to canonical URIs:

```jsonc
{
  "@context": {
    "@version": 1.1,
    "ckodex":   "https://ckodex.org/vocab/",
    "oscal":    "https://docs.oasis-open.org/oscal/",
    "nist":     "https://csrc.nist.gov/projects/",
    "schema":   "https://schema.org/",
    "dcterms":  "http://purl.org/dc/terms/",
    "skos":     "http://www.w3.org/2004/02/skos/core#",
    "subjects":    { "@id": "ckodex:subjects",    "@type": "@id", "@container": "@set" },
    "produces":    { "@id": "ckodex:produces",    "@type": "@id", "@container": "@set" },
    "consumes":    { "@id": "ckodex:consumes",    "@type": "@id", "@container": "@set" },
    "related":     { "@id": "ckodex:related",     "@type": "@id", "@container": "@set" },
    "exclusiveOf": { "@id": "ckodex:exclusiveOf", "@type": "@id", "@container": "@set" },
    "triggers":    { "@id": "ckodex:triggers",                    "@container": "@set" },
    "galMinimum":  { "@id": "ckodex:galMinimum",  "@type": "xsd:integer" },
    "proofTypes":  { "@id": "ckodex:proofTypes",                  "@container": "@set" },
    "PCA": "ckodex:proof/PCA", "UCA": "ckodex:proof/UCA",
    "TIP": "ckodex:proof/TIP", "ZKP": "ckodex:proof/ZKP",
    "DCA": "ckodex:proof/DCA"
  }
}
```

The full context document is bundled at `schemas/skill-context-v1.jsonld`.

## 9. Examples

Three illustrative manifests ship with this pack:

- **`examples/ckodex-oscal-v1.1.skill.json`** — full v1.1 with all blocks populated.
- **`examples/minimal-v1.1.skill.json`** — minimum viable v1.1 (name + description + synopsis + entrypoints).
- **`examples/third-party-v1-compat.skill.json`** — a v1 manifest with no extras, demonstrating backward compatibility.

## 10. Security and Threat Model

See RFC-001 §14 for the full threat model. Spec-level mitigations
relevant here:

- Canonical skill ID is `sha256(name ⊕ namespace ⊕ issuer)`, not the bare `name` — defends against skill squatting.
- The `@context` URI is a constant (`const` in the schema) — defends against ontology poisoning via injected contexts.
- `proofTypes` is a closed enum — defends against unknown-predicate attacks.
- `runtime.implicitInvocation: false` is the supported way to forbid auto-promotion — defends against implicit-invocation hijack.

## 11. References

- Agent Skills v1 (upstream): `https://github.com/agentskills/agentskills`
- RFC-001 Progressive Skill Lifecycle Runtime Profile (companion pack)
- JSON Schema 2020-12: `https://json-schema.org/draft/2020-12/schema`
- JSON-LD 1.1: `https://www.w3.org/TR/json-ld11/`

## 12. License

Apache-2.0.

---

*End of CKODEX Skill Spec v1.1.*
