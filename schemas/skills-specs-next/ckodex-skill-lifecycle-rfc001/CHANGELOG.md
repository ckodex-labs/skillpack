# Changelog — CKODEX Skill Lifecycle RFC-001

## [0.2.0] — 2026-05-23

External-review integration.

### Reframed

- Status reframed from "upstream replacement candidate" to "runtime profile". RFC now explicitly does NOT require any change to upstream Agent Skills v1.
- Title: "Progressive Skill Lifecycle Runtime Profile for Agent Harnesses".

### Added

- **L4 split into L4R (resource read) and L4X (script execute).** Reading reference docs and executing scripts have different risk profiles and require different gates.
- **Threat model** (§14) covering skill squatting, malicious synopsis, ontology poisoning, prompt-induced overpromotion, skill persistence attacks, cross-tenant leakage, script abuse, evidence leakage, implicit-invocation hijack, framework downgrade.
- **Capability negotiation handshake** (§15 + Appendix D + new schema `skill-runtime-capabilities.v1.schema.json`).
- **Acceptance metrics** with numeric SLO targets (§16): token reduction ≥60%, false-positive promotions ≤5%, p95 router latency ≤50 ms, etc.
- **Implementation architecture sketch** (§17): `ckx-skill-kernel`, `ckx-skill-registry`, `ckx-skill-validation`, `ckx-skill-router`, `ckx-skill-evidence`, `ckx-skill-adapters`.
- **Eight additional conformance vectors (E–L)**: explicit-invocation bypass, implicit-invocation disabled, malicious ontology, privacy mode, script execution gate, version skew, duplicate names, synopsis mismatch.
- **PCA privacy modes** (§9 + Appendix C + new schema `pca-lifecycle.v1.schema.json`): `debug-local`, `audit-private`, `public-anchor`, `regulated-export`. Default is `audit-private`. `public-anchor` excludes prompt-derived signal and human-readable skill names (unless on a public-allowlist).
- **Reference Python kernel** (`reference/kernel.py`) and **scoring module** (`reference/scoring.py`).
- **Conformance runner** (`conformance/run.py`) — kernel-runnable for vectors A/B/D/F, structural for the remaining eight.
- **Two diagrams**: T4 swimlane event-loop and T1 ckodex-native FSM lifecycle.
- **Bundle self-test** (`scripts/validate.sh`) with sha256 ledger verification.

### Changed

- **Scoring normalization**: all signal components clamped to `[0,1]`. `S_ont` raw value divided by 1.5 and clamped (prevents the previous out-of-range issue where Jaccard sums could exceed 1.0).
- **`S_usr` graded**: now `{1.0 explicit, 0.8 pinned, 0.6 memory, 0.0 none}` instead of a single Boolean.
- **`metadata.synopsis` strength**: MUST (v0.1.0) → SHOULD overall, MUST for CKODEX-shipped skills only. Third-party v1/v1.1 skills MAY omit it; harnesses synthesize L2 from the description.
- **Schema composition**: v1.1 schema uses `allOf` plus `unevaluatedProperties: false` per JSON Schema 2020-12 guidance (previously used `additionalProperties: false`, which is unsafe with `allOf`).
- **Baseline claim corrected**: from "two-tier upstream model" to "metadata → instructions → resources, no runtime FSM defined".
- **Manifest placement clarified**: ontology lives in the CKODEX companion `skill.json` (already CKODEX-namespaced), NOT in upstream `SKILL.md` frontmatter.

### Pushed back

- **OpenAI Codex 2% / 8000-char cap citation**: removed pending primary-source verification; structural claim about "startup bloat bounded, post-selection accumulation is the real problem" retained.
- **`skill.ckodex.json` rename**: rejected. Existing CKODEX bundles ship `skill.json`; renaming breaks ten shipped skills. Scope clarified inline (§7.1) instead.

## [0.1.0] — 2026-05-22 (internal draft, not shipped)

- Initial FSM proposal: five tiers (L0..L4), multi-signal scoring, hysteresis, drift eviction, JSON-LD ontology, single PCA mode.
- Four conformance vectors (A: cold-start, B: drift, C: budget, D: mutex).
