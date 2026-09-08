# Six-Primitive Lifecycle: Cross-Project Mapping

**Date:** 2026-05-28
**Status:** Standing learning (session output, ready for review)
**Scope:** How the six skill-lifecycle primitives — CREATE / ASSESS / EVOLVE / EXCHANGE / BUNDLE / DISTRIBUTE — partition responsibility between `ckodex-skill-pack` (Rust assessor, this repo) and `skills-ecosystem` (Swift runtime, sibling repo at `/Users/mchorfa/Developer/skills-ecosystem`).
**Anchors:** CKODEX-CODE v16.3 §26 (VWP) · CATM-SPEC v0.1 · CNSB v1 (`schemas/cnsb/v1/cnsb.schema.json`) · evidence v1 (`schemas/evidence/v1/envelope.schema.json`) · skill-lock v1 (`schemas/cnsb/v1/skill-lock.schema.json`) · sibling dossier `/Users/mchorfa/Developer/skills-ecosystem/SKILLS-DOSSIER.md`
**Vocabulary reconciliation:** GAL 0–5 (this repo, schema-bound) vs. DAL 0–4 (CKODEX v16.3) — **deferred per `2026-05-09-finish-line-assessment-design.md` §1 "v1 explicitly OUT of scope"**.

---

## 0. Why this document exists

A `/goal` session deep-analyzed three disjoint layers and produced `SKILLS-DOSSIER.md` (synced to this repo root, 28509 bytes). The dossier identifies six primitives covering the whole lifecycle. Of those six, **this repo (`ckodex-skill-pack`) implements exactly one — ASSESS — and that is its entire scope.** The other five primitives are either:

- **upstream** of ASSESS (CREATE feeds skills in)
- **downstream** of ASSESS (EVOLVE consumes scores; BUNDLE/DISTRIBUTE/EXCHANGE consume signed evidence envelopes)
- **adjacent** (run on the Swift runtime or external registries)

This learning spec gives operators a single page that says **where ckodex-skill-pack stops**, **what each adjacent primitive expects from it**, and **which schema fields are already load-bearing for each of the six primitives**.

> Per VWP P-VW-001 / P-SC-003: every claim below is classified **[C]** concrete / **[S]** specified / **[A]** aspirational, with evidence_refs anchored to file:line.

---

## 1. Six-primitive frame at a glance

| # | Primitive | Owner (today) | Schema artifact (today) | Status |
|---|-----------|---------------|--------------------------|--------|
| 1 | CREATE     | `skills-ecosystem/SkillsCore/SkillCrafter.swift` + `skills-specs-next/ckodex-skill-tools/scaffold.py` | `agentskills.schema.json` (SKILL.md frontmatter) | **[C]** |
| 2 | ASSESS     | **`ckodex-skill-pack` (this repo)** | `evidence/v1/envelope.schema.json` with `statement.type = "SkillAssessment"` | **[C]** (v1 spec locked, see `docs/superpowers/specs/2026-05-09-finish-line-assessment-design.md`) |
| 3 | EVOLVE     | — (no implementation) | `cnsb/v1/cnsb.schema.json` (carries version) but no FSM | **[A]** (RFC-001 v0.2.0 in `skills-ecosystem/skills-specs-next/`) |
| 4 | EXCHANGE   | `skills-ecosystem/skills-specs-next/ckodex-stx-spec/` (REST/gRPC/GraphQL drafted) | STX protocol envelopes | **[S]** |
| 5 | BUNDLE     | `cnsb/v1/cnsb.schema.json` + `cnsb/v1/skill-lock.schema.json` (this repo) | CNSB SkillBundle + skill-lock SRI | **[C]** |
| 6 | DISTRIBUTE | `docs/oci-distribution.md` (this repo) + sibling Swift runtime fanout | OCI artifact path | **[S]** |

**Read this as:** ASSESS and BUNDLE are concrete in this repo. EVOLVE is the only primitive with no implementation in either repo — it lives entirely in the spec corpus and is the highest-leverage missing piece.

---

## 2. ASSESS — what this repo owns end-to-end

`ckodex-skill-pack`'s 9 canonical dimensions ARE the ASSESS primitive. Mapping is 1:1 — no leakage into the other five primitives.

| Dimension (W) | What's graded | Six-primitive role |
|---|---|---|
| Identity & Manifest (11) | `SKILL.md` frontmatter + CNSB metadata | Validates CREATE output is parseable |
| Security (18)            | secrets, threats, license, CVE, **`asc[]` non-empty** | Gates EXCHANGE + DISTRIBUTE eligibility |
| Provenance (14)          | SBOM, SLSA, sigstore, **signed evidence envelope** | Inputs to BUNDLE (skill-lock SRI) |
| Documentation (11)       | README, examples, API docs, `agentskills.description ≥ 50 chars` | Surfaces CREATE quality |
| Testing (10)             | test files, runner, coverage, **CNSB `Skill.examples[]`** | Inputs to EVOLVE (regression baselines) |
| Compatibility (8)        | runtime matrix, SDK pins, `galMin/galMax` | Constrains EVOLVE supersession path |
| Lifecycle (10)           | CNSB hooks `pack/unpack/install/upgrade/uninstall/verify/preInstall/postInstall` | Inputs to BUNDLE + DISTRIBUTE |
| Governance (9)           | LICENSE, CODE_OF_CONDUCT, SECURITY, CODEOWNERS, `galMin ≤ galMax`, PolicyBundle resolves | Inputs to EXCHANGE (cross-tenant gates) |
| Evals & HITL (9)         | eval harness, runner, HITL gates, promotion thresholds, red-team | Inputs to EVOLVE (promote vs. supersede decision) |

**Sum check:** 11+18+14+11+10+8+10+9+9 = **100** ✓ (evidence_ref: `docs/superpowers/specs/2026-05-09-finish-line-assessment-design.md` §2)

**Output contract (release-blocking):** One signed `evidence/v1/envelope.schema.json` per assessment, with `signatures[] minItems: 1` and `statement.type = "SkillAssessment"`. Grade A/B → `allow`, C → `review`, D/F → `deny` per `prove/v1/policy.schema.json` `Enforcement.effect`. (evidence_ref: same doc, Locked table row "Output per assessment")

---

## 3. What the other five primitives expect from ASSESS

Each downstream/upstream primitive consumes or feeds something specific. Naming these contracts now prevents schema sprawl later.

### 3.1 CREATE → ASSESS (upstream contract)

- **Input emitted by CREATE that ASSESS reads:** `agentskills.schema.json` (SKILL.md frontmatter) OR `cnsb/v1/cnsb.schema.json` (CNSB SkillBundle metadata block — `apiVersion`, `kind`, `name`, `urn`, `version`).
- **ASSESS guarantee back to CREATE:** machine-readable diagnostics in JSON + SARIF reporter format, citing dimension ID + field path.
- **Gap [S]:** SkillCrafter (`skills-ecosystem/SkillsCore/SkillCrafter.swift:23-158`) emits SKILL.md but does NOT yet emit CNSB metadata. CREATE-time CNSB stub generation is a v2 ask (a.k.a. "create-skill 1000×" in the v1/v2 split doc).

### 3.2 ASSESS → EVOLVE (downstream contract, partial)

- **Output consumed:** the signed evidence envelope's `statement.score` + dimension breakdown + `chain.previous[]` linkage.
- **Gap [A]:** there is no EVOLVE implementation anywhere. RFC-001 v0.2.0 (in `skills-ecosystem/skills-specs-next/RFC-001-skill-lifecycle-v0.2.0.md`) defines a 6-tier FSM (L0/L1/L2/L3/L4R/L4X) and demotion triggers, but **no code reads scores and decides promote-vs-supersede.** This is decision **D-1** in the sibling dossier §7.
- **Recommended v1 stance for this repo:** emit `chain.previous[]` correctly so EVOLVE can be retrofitted later without re-grading history. This is already in the v1 lock.

### 3.3 ASSESS → BUNDLE (downstream contract, concrete)

- **Output consumed:** `cnsb/v1/skill-lock.schema.json` with SRI integrity hashes (already in v1 scope, evidence_ref: `docs/superpowers/specs/2026-05-09-finish-line-assessment-design.md` "Lock command output").
- **Status [C]:** end-to-end works today through the `lock` command. No gap.

### 3.4 BUNDLE → DISTRIBUTE (adjacent contract)

- **Output consumed:** CNSB tarball + signed lock + evidence envelope.
- **Status [S]:** `docs/oci-distribution.md` specifies the OCI artifact layout. No registry binding shipped yet. This is decision **D-2** in the sibling dossier (tar+sha vs. OCI vs. phased).

### 3.5 ASSESS → EXCHANGE (downstream contract, gated)

- **Output consumed:** the evidence envelope acts as the trust handshake artifact in any STX `metadata_only` or `content_hash` exchange mode.
- **Gap [S]:** STX protocol is drafted in `skills-ecosystem/skills-specs-next/ckodex-stx-spec/` but has no code path in this repo. Privacy default is decision **D-3** in the sibling dossier (`metadata_only` recommended).

### 3.6 DISTRIBUTE → CREATE (loop closure, aspirational)

- The full lifecycle closes when a DISTRIBUTEd skill is re-pulled by a peer's CREATE step (or import). **[A]** — neither repo wires the loop today.

---

## 4. Vocabulary mismatch (will bite if ignored)

| Concept | This repo (`ckodex-skill-pack`) | CKODEX v16.3 (`~/.claude/CLAUDE.md`) | Sibling (`skills-ecosystem`) |
|---|---|---|---|
| Autonomy axis | **GAL 0–5** (schema-bound, see `schemas/common/types.schema.json`) | **DAL 0–4** | Uses DAL via CKODEX header |
| Lifecycle states | 9-dimension grade (A/B/C/D/F) | Vector state (empty/positive/negative/anti) + harness lens (exploratory/verified/promotable/retired) | RFC-001 6-tier (L0..L4X) |
| Evidence model | `evidence/v1/envelope.schema.json` (this repo) | `EvidenceBundle` + BPL (CKODEX §13) | Reuses CKODEX |
| Bundle format | CNSB v1 (this repo) | `SkillBundle` (OCI) per CKODEX §7 | RFC-001 references CNSB |

**Decision deferred to v2** per `2026-05-09-finish-line-assessment-design.md`: "GAL ↔ DAL vocabulary unification" is explicitly out of v1 scope. **This means:** any cross-project tooling that bridges this repo and `skills-ecosystem` must translate GAL ↔ DAL at the boundary. The boundary lives at the evidence envelope handoff (ASSESS → EVOLVE).

> Per CKODEX P-SC-005 (NoUndefinedReuse): treat "verified" in evidence envelope as **GAL-defined** in this repo's output; the consumer (Swift runtime / EVOLVE) is responsible for mapping to DAL semantics.

---

## 5. Open decisions blocking cross-project v0.2

These mirror sibling dossier §7. Each has implications for this repo's roadmap.

| ID | Question | Affects this repo? |
|---|---|---|
| **D-1** | EVOLVE semantics: mutation (rewrite-in-place) vs. supersession (new immutable version + lineage edge) vs. hybrid | Yes — determines whether `chain.previous[]` in evidence envelope is a hash-chain (supersession) or a mutation-log (mutation) |
| **D-2** | BUNDLE substrate: tar+sha256 minimum vs. OCI artifact (cosign-signable) vs. phased | Yes — `cnsb/v1/skill-lock.schema.json` SRI strategy already implicitly assumes hash-tree; OCI adds a registry hop |
| **D-3** | STX privacy default: `metadata_only` (safest) vs. `content_hash` vs. `content_partial` vs. `content_full` | No (deferred to v2), but the evidence envelope shape needs to be safe to publish at `metadata_only` minimum |

---

## 6. Reading order (for an operator picking this up cold)

1. **This file** — six-primitive frame + cross-project boundaries (5 min)
2. `SKILLS-DOSSIER.md` (repo root) — full standing dossier with all 11 TODOs (T-01..T-11) and full primitive specs (20 min)
3. `docs/superpowers/specs/2026-05-09-finish-line-assessment-design.md` — v1/v2 scope locks and 9-dimension weights (15 min)
4. `docs/dimensions/*.md` — per-dimension scoring rubric (browse as needed)
5. `schemas/cnsb/v1/cnsb.schema.json` + `schemas/evidence/v1/envelope.schema.json` — the binding contracts
6. **Sibling repo:** `/Users/mchorfa/Developer/skills-ecosystem/skills-specs-next/RFC-001-skill-lifecycle-v0.2.0.md` — EVOLVE FSM (read-only, no implementation in either repo)

---

## 7. Self-attestation (VWP §26.E)

```yaml
capability_id: "six-primitive-cross-project-mapping"
claim: "Maps six skill-lifecycle primitives onto this repo's 9-dimension ASSESS scope, identifies cross-project contracts, and flags vocabulary mismatch"
classification: S
# S because: §3.1 CNSB-stub gap, §3.2 EVOLVE missing, §3.4 OCI binding pending, §3.5 STX no code path
# Concrete parts (ASSESS scope, dimension table, schema cross-refs) are [C], but the mapping as a whole spans aspirational primitives.
evidence_refs:
  - kind: spec_reference
    path: "docs/superpowers/specs/2026-05-09-finish-line-assessment-design.md"
    note: "v1 scope locks and 9-dimension weight table"
  - kind: schema
    path: "schemas/cnsb/v1/cnsb.schema.json"
    note: "BUNDLE primitive binding"
  - kind: schema
    path: "schemas/evidence/v1/envelope.schema.json"
    note: "ASSESS output contract"
  - kind: schema
    path: "schemas/cnsb/v1/skill-lock.schema.json"
    note: "ASSESS → BUNDLE handoff"
  - kind: dossier
    path: "/Users/mchorfa/Developer/skills-ecosystem/SKILLS-DOSSIER.md"
    note: "Sibling dossier; mirrored to this repo root as SKILLS-DOSSIER.md (28509 bytes)"
  - kind: dossier
    path: "SKILLS-DOSSIER.md"
    note: "Local mirror"
stubs_remaining: 0
deferred:
  - "GAL ↔ DAL vocabulary unification (v2)"
  - "CNSB stub emission at CREATE time (v2, 'create-skill 1000×')"
  - "EVOLVE implementation (D-1 decision required first)"
  - "OCI registry binding (D-2 decision required first)"
  - "STX code path (D-3 decision required first; v2 anyway per scope lock)"
baseline_refs: []
```

---

## 8. What this document does NOT do

Per CKODEX P-SC-002 (NoMarketingVocabulary) and P-SC-007 (NoUngroundedGeneralization):

- It does **not** introduce a new primitive vocabulary — the six primitives are taken verbatim from the user's `/goal` directive in the session that produced `SKILLS-DOSSIER.md`.
- It does **not** claim ckodex-skill-pack will implement EVOLVE / EXCHANGE / DISTRIBUTE — those remain owned elsewhere or unowned.
- It does **not** require schema changes — every cited field already exists in `schemas/` today.
- It does **not** require a v1 scope change — every gap is parked at v2 per the existing scope lock.

---

*"Decisions are routed. Skills are locked. Evidence is shipped. Context is leased."*
