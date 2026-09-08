# Skills Specs Next — Lifecycle Cross-Reference Index

> T-16 (§9) — Authoritative cross-reference linking every spec artifact in this
> directory to its lifecycle FSM state, RFC anchor, schema file, and TODO closure
> status. Update this file whenever a spec file is added, retired, or superseded.

---

## 1. Spec Artifacts

| Artifact                                | Kind                 | FSM State  | RFC Anchor    | Schema / Implementation             | TODO         |
| --------------------------------------- | -------------------- | ---------- | ------------- | ----------------------------------- | ------------ |
| `RFC-001-skill-lifecycle-v0.2.0.md`     | RFC                  | Active     | RFC-001 §1–§6 | `ckodex-skill-lifecycle-rfc001/`    | T-11         |
| `RFC-001-skill-lifecycle.md`            | RFC                  | Deprecated | RFC-001 v0.1  | superseded by v0.2.0                | —            |
| `ckodex-skill-spec-v1.1/`               | Spec                 | Active     | RFC-001 §3.1  | `schemas/skill.v1.1.schema.json`    | T-02, T-15 ✓ |
| `ckodex-stx-spec/`                      | Spec                 | Active     | RFC-001 §4.6  | `schemas/stx-domain.v1.schema.json` | T-07, T-08   |
| `ckodex-skill-lifecycle-rfc001/`        | Schema bundle        | Active     | RFC-001 §2    | `pca-lifecycle.v1.schema.json`      | —            |
| `ckodex-skill-tools/`                   | Tool refs            | Active     | RFC-001 §5    | n/a (tooling docs)                  | —            |
| `STX-IMPLEMENTATION-PROMPT.md`          | Implementation guide | Active     | RFC-001 §4.6  | —                                   | T-07         |
| `t1-ckodex-labs-rfc001-fsm-lifecycle.*` | Diagram              | Active     | RFC-001 §2.1  | Mermaid FSM render                  | —            |
| `t4-ckodex-labs-rfc001-event-loop.*`    | Diagram              | Active     | RFC-001 §3.4  | Mermaid event-loop render           | —            |

---

## 2. Lifecycle FSM States (RFC-001 §2.1)

```
experimental → beta → stable → deprecated → retired
                              ↘ superseded (concurrent with deprecated)
```

| State          | Allowed transitions         | Promotion gate                        | Demotion guard                                             |
| -------------- | --------------------------- | ------------------------------------- | ---------------------------------------------------------- |
| `experimental` | → `beta`                    | manual review                         | none                                                       |
| `beta`         | → `stable`, → `deprecated`  | EvidenceBundle freshness + tier check | none                                                       |
| `stable`       | → `deprecated`              | n/a                                   | must not regress to `beta`                                 |
| `deprecated`   | → `retired`, → `superseded` | T-24 4-phase plan required            | none                                                       |
| `retired`      | (terminal)                  | n/a                                   | T-04 forbidden: `retired ∧ execute`, `retired ∧ promotion` |
| `superseded`   | → `retired`                 | `superseded_by` URN required          | none                                                       |

---

## 3. Schema Cross-Reference

| Schema file                                                          | `$id`                                                 | Used by                      | Added fields (sprint)                                                             |
| -------------------------------------------------------------------- | ----------------------------------------------------- | ---------------------------- | --------------------------------------------------------------------------------- |
| `schemas/common/types.schema.json`                                   | `https://schemas.ckodex.org/common/types.schema.json` | CNSB, CNAAB, evidence, prove | `CkodexUrn` T-13 ✓                                                                |
| `schemas/cnsb.schema.json`                                           | `https://ckodex.org/schema/cnsb.json`                 | legacy consumers             | **DEPRECATED** T-12 ✓ — redirect to `cnsb/v1/`                                    |
| `schemas/cnsb/v1/cnsb.schema.json`                                   | `https://schemas.ckodex.org/cnsb/v1/cnsb.schema.json` | `skillpack publish`, bundler | `schemaVersion` T-22 ✓, `bundleLayer` T-17 ✓, `SkillLifecycle` T-14 ✓             |
| `ckodex-skill-spec-v1.1/schemas/skill.v1.1.schema.json`              | `https://schemas.ckodex.org/skill/v1.1`               | skill manifest validator     | `schemaVersion` T-22 ✓, `aipackCapabilities` T-23 ✓, `synopsisHash`+`tier` T-15 ✓ |
| `ckodex-skill-lifecycle-rfc001/schemas/pca-lifecycle.v1.schema.json` | `https://schemas.ckodex.org/pca-lifecycle/v1`         | lifecycle checker            | unchanged                                                                         |
| `ckodex-stx-spec/schemas/stx-domain.v1.schema.json`                  | `https://schemas.ckodex.org/stx-domain/v1`            | STX publisher                | unchanged                                                                         |

---

## 4. TODO Closure Status

| TODO | Description                                                          | Status         |
| ---- | -------------------------------------------------------------------- | -------------- |
| T-01 | `SkillEvolver` (schema-bump / content-patch / supersession)          | ✓ Closed       |
| T-02 | `Skill` struct: `version`, `contentHash`, `tier`, `ontologyRef`      | ✓ Closed       |
| T-03 | `SkillStorageProvider`: `getManifest`, `getEvidence`, `listVersions` | ✓ Closed       |
| T-04 | Forbidden-tuple checks in `SyncManager`                              | ✓ Closed       |
| T-05 | `bundle.json` emission from `ManifestGenerator`                      | ✓ Closed       |
| T-06 | `SkillBundleAssembler` + cosign + Rekor                              | `[A]` deferred |
| T-07 | STX REST + gRPC publisher projections                                | ✓ Closed       |
| T-08 | `LifecycleDecisionBundle` from `SyncManager`                         | ✓ Closed       |
| T-09 | Per-environment promotion gates                                      | `[A]` deferred |
| T-10 | Storage fallback behavior + integration tests                        | ✓ Closed       |
| T-11 | `intent_to_invoke` per RFC-001 §3.2                                  | ✓ Closed       |
| T-12 | Retire root `schemas/cnsb.schema.json`                               | ✓ Closed       |
| T-13 | `CkodexUrn` strict type                                              | ✓ Closed       |
| T-14 | `SkillLifecycle` hook object in CNSB schema                          | ✓ Closed       |
| T-15 | `synopsisHash` + `tier` in skill v1.1 schema                         | ✓ Closed       |
| T-16 | This index file                                                      | ✓ Closed       |
| T-17 | AIPACK media type constants + `bundleLayer`                          | ✓ Closed       |
| T-18 | `urn:skill:static-analysis:v1` OCI referrer                          | ✓ Closed       |
| T-19 | `urn:skill:capability-declaration:v1` OCI referrer                   | ✓ Closed       |
| T-20 | `cyclonedx.org/bom` SBOM OCI referrer                                | ✓ Closed       |
| T-21 | `urn:skill:safety-review:v1` attestation stub                        | `[A]` deferred |
| T-22 | `schemaVersion: 1` in CNSB + skill v1.1 schemas                      | ✓ Closed       |
| T-23 | `aipackCapabilities` shim (AIPACK §6.4)                              | ✓ Closed       |
| T-24 | 4-phase deprecation lifecycle + `urn:aipack:deprecation:v1`          | `[A]` deferred |
| T-25 | RV(C) → skillpack-grade bridge                                       | `[A]` deferred |
| T-26 | Blast-radius containment spike                                       | `[A]` deferred |

---

## 5. Client Specification Documents

| Document                   | Purpose                                                   | Status       |
| -------------------------- | --------------------------------------------------------- | ------------ |
| `CLIENT-SPEC.md`           | Master unified client design brief & spec                 | Draft v0.1.0 |
| `client-model.schema.json` | Shared canonical data model (JSON Schema)                 | Draft v0.1.0 |
| `client-api-contract.md`   | Transport layer contract (gRPC + REST + XPC + MCP)        | Draft v0.1.0 |
| `client-state-protocol.md` | Cache, events, optimistic updates, offline behavior       | Draft v0.1.0 |
| `client-error-codes.md`    | Structured error taxonomy with per-client rendering hints | Draft v0.1.0 |
| `client-feature-matrix.md` | Per-client operation support (MUST/SHOULD/MAY/MUST NOT)   | Draft v0.1.0 |

*Auto-maintained. Last updated by sprint tooling — do not edit the table headers manually.*
