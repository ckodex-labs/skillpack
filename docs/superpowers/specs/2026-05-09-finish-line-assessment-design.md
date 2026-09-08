# SkillPack Finish-Line Assessment Design

**Date:** 2026-05-09
**Status:** Draft (pending user review)
**Scope:** Phased v1 (Internal MVP + Public OSS 1.0) and v2 (frontend ambition + authoring augmentation) for the SkillPack "AI Agent Skill Quality Assessment Framework".
**Anchors:** CKODEX-CODE v16.3 · CATM-SPEC v0.1 · `/schemas` (CNSB v1, CNAAB v1, OpenEvidence Envelope v1, Prove PolicyBundle v1, Skill-Lock v1, agentskills, skillpack-config)
**Brainstorm transcript:** session 463bb1c4-cfcf-4f87-9269-4ed438df9d61
**Build approach:** β — spine-first, replace stubs in place

---

## 1. Scope Locks (v1 / v2 split)

### v1 — Internal MVP + Public OSS 1.0

The schemas in `/schemas` are the binding contract. v1 grades against what they already mandate; "Ckodex compliance" is not a new requirement but the existing schema contract honored in code.

| Locked | Value |
|---|---|
| Inputs accepted | `agentskills.schema.json` (SKILL.md frontmatter) AND `cnsb/v1/cnsb.schema.json` (CNSB SkillBundle) |
| Output per assessment | One signed `evidence/v1/envelope.schema.json` with `statement.type = "SkillAssessment"` and `signatures[]` non-empty |
| Lock command output | `cnsb/v1/skill-lock.schema.json` with SRI integrity hashes |
| Reporters | json, sarif, markdown (sarif must validate against SARIF 2.1) |
| Dimensions | The 9 canonical (Identity & Manifest, Security, Provenance, Documentation, Testing, Compatibility, Lifecycle, Governance, Evals & HITL) |
| Checker contract | Every checker reads content; no `file_exists` shortcut |
| Grade → policy mapping | A/B → `allow`, C → `review`, D/F → `deny` (per `prove/v1/policy.schema.json` Enforcement.effect) |
| Vocabulary | GAL 0–5 as schemas declare; DAL ↔ GAL reconciliation deferred to v2 |
| CLI dry-run | `--dry-run`/`--preview` on every artifact-producing command (lock, init, migrate, future publish/push); read-only commands accept-but-ignore |
| VS Code ext | Read-only diagnostics + grade view only |
| Dashboard | Not required for v1 |

**Public OSS 1.0 bar (additional to MVP):**

- Stable CLI surface — no breaking changes after release within v1.x
- All 9 dimensions documented (what's checked, why it matters, how to fix low scores)
- Tests + CI green on a public CI provider
- Publishable to crates.io and GitHub Releases
- LICENSE (Apache-2.0), CONTRIBUTING, CODE_OF_CONDUCT, SECURITY (already present)

### v1 explicitly OUT of scope (deferred to v2)

VWP §26 input enforcement on graded artifacts · Rekor anchoring of assessment outputs · BPL on promotion-critical scores (envelope `chain.previous[]` is enough for v1) · CKODEX-DS-1 design system mirror in dashboard · GAL ↔ DAL vocabulary unification · AI-assisted authoring · create-skill 1000× · ActionEnvelope wrapping of SkillPack's own tool calls · DAAS routing.

### v2 — Frontend Ambition + Authoring Augmentation

| Track | Content |
|---|---|
| **a4** | Full dashboard: history, comparison, trends, registry browser, drill-down per dimension, full UX |
| **b4** | VS Code extension full feature set: real-time diagnostics + AI-assisted authoring + skill creation wizard + registry browser + autofix suggestions |
| **c4** | Documentation site: docs.skillpack.io with full API ref, dimension explainers, recipe book, migration guides |
| **Ckodex full integration** | ActionEnvelope wrapping, BPL on promotion, Rekor anchoring, CKODEX-DS-1 in dashboard, VWP §26 input enforcement |
| **Create-skill 1000×** | SkillPack-driven skill scaffolding that consumes a description and emits a skill that already grades A or B on first commit (CNSB-conformant, lifecycle hooks pre-wired, ASC declared, evidence stubs in place) |

**v2 entry conditions:** v1 shipped, in use internally on ≥3 real skills, ≥30 days of stable CLI surface (no breaking changes), VS Code extension v1 published to marketplace.

---

## 2. Nine Canonical Dimensions with Schema Bindings

Weights sum to 100. Every checker reads content.

| # | Dimension | W | What it checks (content-based) | Schema binding (v1) |
|---|---|---|---|---|
| 1 | Identity & Manifest | 11 | SKILL.md present + frontmatter validates; CNSB metadata block (apiVersion, kind, name, urn, version) present + validates | `agentskills.schema.json` AND/OR `cnsb/v1/cnsb.schema.json` metadata |
| 2 | Security | 18 | secrets scan (regex + entropy); threat model parseable; license declared; dep CVE check; **`asc[]` non-empty on each CNSB skill** | CNSB `Skill.asc` (ASC pattern); license field |
| 3 | Provenance | 14 | SBOM (CycloneDX) parseable; SLSA provenance present; sigstore signature; **assessment emits valid evidence envelope** | `evidence/v1/envelope.schema.json` (`signatures[] minItems: 1`); `cnsb/v1/skill-lock.schema.json` SRI integrity |
| 4 | Documentation | 11 | README ≥ threshold; examples present; API docs; code samples runnable; **`agentskills.description` ≥ 50 chars** (well below 1024 ceiling) | `agentskills.schema.json` description; CNSB `Skill.description` |
| 5 | Testing | 10 | test files exist; runner config; coverage report; non-trivial assertions; **CNSB `Skill.examples[]` non-empty** | CNSB `Skill.examples[]` presence |
| 6 | Compatibility | 8 | runtime matrix declared; SDK pins; platforms; `agentskills.compatibility` populated; CNSB `galMin/galMax` declared | `agentskills.schema.json` compatibility; CNSB galMin/galMax |
| 7 | Lifecycle | 10 | hooks declared (pack/unpack/install/upgrade/uninstall/verify/preInstall/postInstall) **AND have non-empty command/script bodies** | `skill.type.ts:97-106` Lifecycle interface |
| 8 | Governance | 9 | LICENSE; CODE_OF_CONDUCT; SECURITY policy; CODEOWNERS; **`galMin ≤ galMax`** consistency; **referenced PolicyBundle resolves** | `common/types.schema.json` GAL (0..5); `prove/v1/policy.schema.json` if linked |
| 9 | Evals & HITL | 9 | eval harness; runner config; HITL gates; promotion thresholds; eval results emitted as evidence envelopes; red-team cases | `prove/v1/policy.schema.json` `requireHumanApproval` + `approverRoles`; envelope `chain.previous[]` |

**Sum check:** 11+18+14+11+10+8+10+9+9 = **100** ✓

**Dropped from former code set:**

- `Templates` → absorbed into Documentation (templates are documentation artifacts)
- `Mcp` → becomes a `BonusPoints` modifier (modality, not a dimension; field already exists in `assess_skill.rs`)
- `Accessibility` → sub-check of Documentation

**Out of scope for v1** (would be fake without runtime harness):

- `Performance` — needs benchmark runner; static analysis would be P-VW-006 violation
- `Observability` — needs runtime trace inspection

**Single largest quality lever**: replacing the 7 stub checkers in `crates/skillpack-adapters/src/checkers/mod.rs` with content-reading implementations.

---

## 3. CLI Surface + Dry-Run Contract

**Stability commitment**: every flag and command in this section is frozen at v1.0.0. No breaking changes for v1.x. Removals require deprecation cycle (warn for one minor, drop in next).

### Read-only commands (no `--dry-run` required)

| Command | Purpose | Output | Exit codes |
|---|---|---|---|
| `skillpack check <path>` | Fast structural validation; no envelope | stdout summary | 0 ok, 1 schema invalid, 2 IO error |
| `skillpack grade <path>` | Full 9-dim assessment; **emits signed envelope** | report (format-flagged) + envelope path | 0 ≥ min-grade, 3 < min-grade, 4 stub-detected |
| `skillpack report <envelope>` | Re-render existing envelope | report (format-flagged) | 0 ok, 5 envelope unparseable |
| `skillpack validate <path>` | Schema-only validation (no scoring) | stdout per-file results | 0 ok, 1 schema invalid |
| `skillpack verify <envelope>` | Verify cosign signature + chain integrity | verdict | 0 valid, 6 signature invalid, 7 chain broken |

### Artifact-producing commands (`--dry-run`/`--preview` REQUIRED to be supported)

| Command | Purpose | Without flag | With flag |
|---|---|---|---|
| `skillpack lock <path>` | Emit `skill-lock.json` (CNSB v1, SRI hashes) | writes file, signs | prints planned content + would-write paths to stdout, exits 0 without filesystem mutation |
| `skillpack init` | Scaffold `skillpack-config.json` | writes config | prints planned config |
| `skillpack migrate <path>` | Upgrade config schema version | rewrites in place | prints diff of planned changes |

**v2 commands (not in v1 surface)**: `publish` (push to registry), `push` (push envelope to evidence store), `wizard` (create-skill 1000×).

### Universal flags

| Flag | Default | Notes |
|---|---|---|
| `--config <path>` | `./skillpack-config.json` then `~/.skillpack/config.json` | per `skillpack-config.schema.json` |
| `--format json\|sarif\|markdown` | `json` (machine), `markdown` (TTY) | sarif must validate against SARIF 2.1 |
| `--out <path>` | stdout | report destination |
| `--minimum-grade A\|B\|C\|D\|F` | from config (default `C`) | overrides config |
| `--quiet` / `-q` | off | only exit code + errors to stderr |
| `--verbose` / `-v` | off | per-checker timing + decision trace |
| `--dry-run` / `--preview` | off | accepted by all commands; only meaningful on artifact-producing |

### Configuration precedence

1. CLI flags
2. Env vars (`SKILLPACK_CONFIG`, `SKILLPACK_MIN_GRADE`, `SKILLPACK_FORMAT`)
3. `--config` file
4. `./skillpack-config.json`
5. `~/.skillpack/config.json`
6. Built-in defaults

### Exit code contract

```
0  success (or grade ≥ minimum)
1  schema invalid
2  IO / filesystem error
3  grade below minimum
4  stub detected (P-VW-004 surface — unresolved checker)
5  envelope unparseable
6  signature invalid
7  chain integrity broken
8  budget exceeded (timeout, memory)
64 usage error (CLI parse)
```

### Library boundaries (informs API stability)

- `skillpack-domain` — public types stable at v1: `Dimension`, `Score`, `Grade`, `Envelope`, `BonusPoints`
- `skillpack-application` — use cases as public API: `assess_skill`, `grade_skill`, `lock_skill`, `generate_report`
- `skillpack-adapters::cli` — argv parsing, frozen
- `skillpack-api` (gRPC) — proto v1 frozen; new fields are additive only

---

## 4. Architecture & Four Spaces Mapping

SkillPack is a hexagonal Rust workspace; this section maps existing layers to CKODEX-CODE v16.3 Four Spaces and pins where evidence emission, validation, and presentation actually live.

### Four Spaces ↔ SkillPack crates

| Four Space | SkillPack home | What lives here | Strict rule |
|---|---|---|---|
| **Kernel** | `crates/skillpack-domain` | `Dimension`, `Score`, `Grade`, `BonusPoints`, `CheckResult`, `Envelope` (domain shape), `AssessmentPlan`, ports (traits) | NO `tonic`/`prost`/`hyper`; NO file IO; NO signing |
| **Validation** | `crates/skillpack-application` (use cases) + `crates/skillpack-domain/policy` | use cases (`assess_skill`, `grade_skill`, `lock_skill`, `generate_report`); 9-dimension orchestrator; threshold gates; PolicyBundle effect mapping (A/B→allow, C→review, D/F→deny) | All scoring decisions go through here; no scoring math in adapters |
| **Presentation** | `crates/skillpack-adapters` (CLI, gRPC, MCP, reporters) + `crates/skillpack-api` | argv parsing, gRPC mappers, MCP handlers, json/sarif/markdown reporters, dashboard (v2), VS Code ext (v1 read-only) | Mapping + rendering only; NO scoring logic, NO checker behaviour here |
| **Proof** | `crates/skillpack-adapters/src/evidence/` (NEW for v1) + `crates/skillpack-adapters/src/checkers/` (content readers, not scorers) | OpenEvidence Envelope assembly, cosign signing, Skill-Lock SRI hashing, checker content-readers (parse Cargo.toml, parse SKILL.md, parse SBOM, read coverage report) | Immutable output: signs, never mutates state |

### Hexagonal port/adapter map

**Ports (in `skillpack-domain`)** — traits the kernel declares:

- `SkillSource` — read SKILL.md / CNSB / source files
- `Checker` — one per dimension; `fn check(&self, ctx: &CheckContext) -> CheckResult`
- `EvidenceSink` — emit signed envelope
- `LockEmitter` — write skill-lock with SRI
- `Reporter` — render report (json/sarif/markdown)
- `Signer` — cosign abstraction (replaceable for tests)

**Adapters (in `skillpack-adapters`)** — concrete implementations:

- `SkillSource`: `FilesystemSource`, `OciSource`, `GitSource`, `S3Source`, `SftpSource` (all already in tree)
- `Checker`: 9 implementations (TODAY: 2 real + 7 stubs; v1 release gate: 9 real)
- `EvidenceSink`: `FileEvidenceSink`, `DuckDbEvidenceSink` (DuckDB persistence already in tree at 447 LOC)
- `Reporter`: `JsonReporter`, `SarifReporter`, `MarkdownReporter` (already in tree)
- `Signer`: `CosignSigner` (real), `NoopSigner` (test only)

### Evidence emission flow (the spine)

```
   skillpack grade <path>
        │
        ▼
   application::assess_skill
     ├─ load via SkillSource
     ├─ run 9 Checkers (parallel where independent)
     ├─ aggregate Score + Grade in domain
     ├─ apply BonusPoints (Mcp modality, etc.)
     └─ build domain::Envelope
        │
        ▼
   proof::EnvelopeBuilder
     ├─ wrap statement.type = "SkillAssessment"
     ├─ attach 9-dim payload + grade + stub_count
     ├─ chain.previous[] ← prior envelope (if any)
     └─ Signer.sign() → signatures[] non-empty
        │
        ▼
   EvidenceSink.write(envelope) ───▶ envelope.json (validates against evidence/v1/envelope)
   Reporter.render(envelope)    ───▶ stdout (format-flagged)
```

### Lock emission flow

```
   skillpack lock <path>
        │
        ▼
   application::lock_skill
     ├─ resolve dep tree
     ├─ compute SRI hashes (sha256/384/512)
     ├─ build skill-lock JSON (cnsb/v1/skill-lock)
     └─ LockEmitter.write() OR (--dry-run) print to stdout

   (v1: locks rely on SRI integrity hashes; cosign signing of locks is v2.
    Envelopes ARE cosign-signed in v1 — see flow above.)
```

### Cross-cutting — what does NOT change in v1

- **No ActionEnvelope wrapping** of SkillPack's own tool calls (v2). v1's CLI runs as a normal binary; ActionEnvelope wrapping is for when SkillPack is invoked as a Ckodex skill.
- **No DAAS routing** (v2). v1 is invoked directly.
- **No BPL §13 semantics** (v2). v1 only emits forward `chain.previous[]` linking eval/red-team supersession.
- **No Rekor anchoring** (v2). v1 cosign-signs locally; Rekor public-good integration is a v2 publishing concern.

### Empty-directory disposition

| Path | Today | v1 plan |
|---|---|---|
| `evidence/` | empty | populated with sample signed envelopes (fixture corpus) |
| `crates/skillpack-adapters/src/filesystem/` | scaffold | implement `FilesystemEvidenceSink` + envelope writer |
| `crates/skillpack-adapters/src/grpc/` | scaffold | thin `skillpack-api` mapper; non-blocking for v1 CLI ship |
| `crates/skillpack-adapters/src/checkers/` | 2 real + 7 stubs | 9 real (the work) |

---

## 5. Build Sequence (Approach β, concrete phases)

**Strategy**: Spine before depth. Phase 0 makes the binary emit signed evidence and surface stub-count honestly. Phases 1–9 replace dimension stubs in weight-descending order so every checkpoint moves the achievable grade upward. Phase 10 is the cut/release gate.

**Stop-the-line rule**: a phase doesn't exit until its success criteria pass on the fixture corpus AND envelope output validates against `evidence/v1/envelope.schema.json`. No advancing with red.

### Implementation parallelism — sub-agent dispatch

The orchestrating Opus session authors specs/plans, verifies integration, and resolves cross-task conflicts. **Implementation work is delegated to Sonnet sub-agents** running in parallel for independent tasks. Per-phase fan-out:

| Phase | Sub-agent fan-out (parallel Sonnet) |
|---|---|
| **Phase 0** | 6 sub-agents: (1) `EnvelopeBuilder`, (2) `CosignSigner`, (3) `LockEmitter` + SRI, (4) stub_count surface + exit code 4, (5) `--dry-run` wiring on lock/init/migrate, (6) fixture corpus seeding (known-a/b/c/d/f + edge cases) |
| **Phases 1-9** | within each phase: (a) checker implementation, (b) fixture(s) for that dimension, (c) integration test scaffolding, (d) dimension-doc page. Phase ordering remains strictly sequential. |
| **Phase 10** | 4 sub-agents: (1) cut Templates/Accessibility, fold Mcp into BonusPoints, (2) release-gate scripts (`no_stubs.sh`, dimension-parity check), (3) Dagger CI pipeline, (4) v1.0.0 release prep (CHANGELOG, crates.io publish dry-run, GitHub Release artifacts) |

Opus retains: integration verification, conflict resolution between concurrent edits, release-gate enforcement, sign-off on each phase exit.

### Phase 0 — Spine plumbing (no dimension work)

| Task | Where |
|---|---|
| `EnvelopeBuilder` (statement.type="SkillAssessment") | `skillpack-adapters/src/evidence/builder.rs` (NEW) |
| `CosignSigner` adapter | `skillpack-adapters/src/evidence/signer.rs` (NEW) |
| `LockEmitter` with SRI sha256/384/512 | `skillpack-adapters/src/lock/emitter.rs` (NEW) |
| Surface `stub_count` in `envelope.statement.payload` AND CLI exit 4 | `skillpack-application/src/assess_skill.rs` |
| `--dry-run`/`--preview` on `lock`, `init`, `migrate` | `skillpack-adapters/src/cli/` |
| Fixture corpus: known-a/b/c/d/f + edge cases (malformed-yaml, empty-skill, evil-skill, cnsb-only, agentskills-only, dual-format) | `tests/fixtures/skills/` (NEW) |

**Exit criteria**: `skillpack grade tests/fixtures/skills/known-a` produces a signed envelope validating against `evidence/v1/envelope.schema.json`, AND exits code 4 (7 stubs remain), AND `skillpack lock --dry-run` prints planned content without writing.

### Phases 1–9 — dimension checkers, weight-descending

Each phase: real content reader → schema binding → fixture-corpus test → envelope payload populated → exit 0/3 (not 4) for that dimension.

| Phase | Dim | W | Today | Major work |
|---|---|---|---|---|
| **1** | Security | 18 | `security.rs` partial | extend: license declared check, `asc[]` non-empty per CNSB skill, secrets regex+entropy, dep CVE lookup |
| **2** | Provenance | 14 | stub | SBOM CycloneDX parse, SLSA provenance read, sigstore sig verify, **self-test: own envelope validates** |
| **3** | Identity & Manifest | 11 | `structure.rs` (was "Structure") | upgrade: parse SKILL.md frontmatter via `agentskills.schema.json`, parse CNSB metadata block |
| **4** | Documentation | 11 | stub (Documentation+Templates+Accessibility merge) | README ≥ threshold, examples present, API docs, code samples runnable, `agentskills.description ≥ 50` |
| **5** | Lifecycle | 10 | stub | parse CNSB lifecycle hooks AND **validate non-empty command/script bodies** (the integrity differentiator) |
| **6** | Testing | 10 | NEW (no checker today) | test file discovery, runner config detect, coverage report parse, CNSB `Skill.examples[]` non-empty |
| **7** | Governance | 9 | stub | LICENSE/COC/SECURITY/CODEOWNERS, `galMin ≤ galMax`, PolicyBundle URN resolves |
| **8** | Evals & HITL | 9 | NEW (no checker today) | eval harness detect, HITL gate parse, promotion thresholds, eval results as `chain.previous[]` envelope chain |
| **9** | Compatibility | 8 | NEW (no checker today) | runtime matrix declared, SDK pins, `agentskills.compatibility` populated, CNSB `galMin/galMax` declared |

**Per-phase exit criteria** (uniform): fixture corpus produces deterministic scores across 3 runs · checker reads content (no `file_exists` shortcut) · payload field populated in envelope · `cargo nextest` green · `cargo llvm-cov` ≥ 80% on the new module.

### Phase 10 — Cut, fold, release gate

| Task | Action |
|---|---|
| Delete `Templates`, `Accessibility` stubs | absorbed into Documentation |
| Move `Mcp` stub | becomes `BonusPoints::mcp_modality` in `assess_skill.rs` |
| Verify `all_checkers()` returns exactly 9 | `crates/skillpack-adapters/src/checkers/mod.rs` |
| Verify exit code 4 cannot fire | grep across crates: zero `unimplemented!`, `todo!`, `panic!("stub")`; zero `// TODO(ckodex): stub` markers (P-VW-004) |
| README ↔ code dimension parity | regenerate dimension table in README from `domain::Dimension::all()` |
| CI green on public provider | GitHub Actions hosting Dagger (per `¬business_logic_in_GHA`) |
| Tag `v1.0.0` | `cargo publish` workspace, GitHub Release with signed envelope of itself |

**Release gate (mechanical, not aspirational):**

1. `skillpack grade .` on SkillPack's own repo returns ≥ B
2. `skillpack grade` on each of 3 fixture skills returns the expected grade (A, C, F) bit-stable across 10 runs
3. envelope JSON validates against `evidence/v1/envelope.schema.json`
4. SARIF output validates against SARIF 2.1
5. cosign verify passes on the released binary
6. **No phase-0 escape hatches in tree** — exit code 4 is unreachable

### Parallelization & ordering rules

- Within a phase, checker code and fixture-skill prep can fan out as parallel Sonnet sub-agents; **integration test must run sequentially** (avoids grade-drift confusion when two checkers race a shared parse cache)
- Phases 1–9 are **strictly sequential by weight**: highest weight first means each landed phase moves achievable grade upward by the largest possible delta — visible signal of progress
- Phase 0 is the **only phase that can land before any checker work**; it ships the binary's honest stub-count surface (P-VW-004 compliance) before any score is reported

### What does NOT happen during build

- No refactor of `skillpack-api` (gRPC) until phases 1–9 are complete; v1 ships CLI-first
- No dashboard work (v2)
- No VS Code authoring features (v2; only diagnostics view in v1)
- No Rekor (v2)
- No ActionEnvelope wrapping of SkillPack's own tool calls (v2)

---

## 6. Test / CI Strategy + Fixture Corpus

**Premise**: SkillPack is a tool whose output is grades. The only way to know it works is to grade things whose grade we already know. The fixture corpus is the test plan.

### Fixture corpus (lives at `tests/fixtures/skills/`)

| Fixture | Expected grade | Purpose |
|---|---|---|
| `known-a` | A | Clean: full frontmatter, all 8 lifecycle hooks with non-empty bodies, ASC declared, cosign-signed CNSB, examples[], test runner, LICENSE+COC+SECURITY, CycloneDX SBOM, SLSA provenance, eval harness, HITL gates |
| `known-b` | B | One borderline issue: SBOM present but unsigned (Provenance partial) |
| `known-c` | C | Missing `Skill.examples[]`, no test runner config, COC absent, `verify` lifecycle hook body empty (declared but inert) |
| `known-d` | D | Multiple gaps: no examples, no tests, license missing, asc[] empty |
| `known-f` | F | No SKILL.md, no license, hardcoded secret in source, no SBOM, no signature |
| `malformed-yaml` | — (error) | Bad frontmatter — exercises parse error path; expected exit code 1 |
| `empty-skill` | F | Skeletal but valid: tests "structurally valid is not enough" |
| `evil-skill` | — (Security blocks) | Secrets in source, path traversal in lifecycle command body, allowed-tools escalation — checker self-test |
| `cnsb-only` | A | No `agentskills` SKILL.md; pure CNSB SkillBundle path |
| `agentskills-only` | A | No CNSB; pure Anthropic Agent Skills path |
| `dual-format` | A | Both formats present + consistent |

**Determinism gate**: every fixture is graded 10 times; **bit-identical envelope payload** (excluding timestamps & signatures) across all runs is a release gate.

### Test classification

| Type | Location | Coverage target | Mutation? |
|---|---|---|---|
| Unit | inline `#[cfg(test)]` per crate | overall ≥ 80% | — |
| Integration | `crates/<crate>/tests/` | per-crate ≥ 80% | — |
| Kernel-specific | `crates/skillpack-domain/tests/` | ≥ 90% | **required** (`cargo-mutants`) |
| Validation-specific | use cases in `skillpack-application` | **100%** | **required** |
| Conformance | `test/conformance/` (workspace) | bidirectional spec↔runtime | — |
| Schema | `test/conformance/schemas/` | every emitted artifact validates | — |
| Evidence | `test/conformance/evidence/` | envelope round-trip + signature verify | — |
| E2E | `test/e2e/` | fixture corpus end-to-end | — |

### Three-Pass QA

**1. Coherence** — type consistency, dimension table parity (`README ↔ domain::Dimension::all()`), invariant coverage (every forbidden tuple from CLAUDE.md §10 vector state has a test proving it's unreachable)

**2. Security** — no shadow stubs (P-VW-004 grep), no unsigned envelopes in tree, no unenforced threshold (every `minimumGrade` value has a fixture proving it gates correctly), `cargo audit` clean, `cargo deny check`

**3. Operability** — fixture corpus end-to-end on each target platform, OCI bundle round-trip (build → sign → push to local registry → pull → verify), regression suite (every prior false-positive/false-negative becomes a fixture)

### Schema-validation tests

| Test | Asserts |
|---|---|
| `envelope_validates` | every `skillpack grade` output validates against `schemas/evidence/v1/envelope.schema.json` |
| `sarif_validates` | every `--format sarif` output validates against SARIF 2.1.0 |
| `skill_lock_validates` | every `skillpack lock` output validates against `schemas/cnsb/v1/skill-lock.schema.json` |
| `config_validates` | every `skillpack init` output validates against `schemas/skillpack-config.schema.json` |
| `policy_resolves` | every fixture with `Governance.policyRef` URN resolves to a parseable PolicyBundle |
| `cnsb_round_trip` | parse known-a CNSB → re-emit → parse re-emission → equal |

### CI pipeline (Dagger, hosted by GitHub Actions)

```
stage_0_lint           cargo fmt --check · cargo clippy --workspace -- -D warnings
stage_1_compile        cargo check --workspace --all-targets       (P-VW-002 phantom symbols)
stage_2_dep            cargo fetch --locked                         (P-VW-008 dep resolution)
stage_3_unit           cargo nextest run --workspace
stage_4_cov            cargo llvm-cov --workspace --fail-under-lines 80 → coverage.lcov  (P-VW-005)
stage_5_kernel_cov     cargo llvm-cov -p skillpack-domain --fail-under-lines 90
stage_6_mutation       cargo mutants -p skillpack-domain -p skillpack-application
stage_7_schema         cargo run -p skillpack-tests --bin schema_round_trip
stage_8_fixtures       cargo run -p skillpack-tests --bin fixture_grade_corpus  (10x determinism)
stage_9_e2e            cargo run -p skillpack-e2e (fixture corpus end-to-end)
stage_10_audit         cargo audit · cargo deny check · trivy fs .
stage_11_release_gate  scripts/no_stubs.sh
stage_12_self_grade    cargo run --bin skillpack -- grade . --minimum-grade B
stage_13_artifact      cargo build --release · cosign sign · cyclonedx sbom · cargo run --bin skillpack -- grade target/release | sign envelope
```

**Public CI provider**: GitHub Actions hosting Dagger (Dagger contains the actual logic per CLAUDE.md §3 stack rule). Linux + macOS matrix in v1; Windows deferred.

### VWP §26 self-application

| Rule | CI mechanism |
|---|---|
| P-VW-001 NoUnverifiedCompletion | PR template requires `evidence_refs[]`; lint blocks empty |
| P-VW-002 NoPhantomSymbols | `cargo check` (stage 1) |
| P-VW-004 NoSilentStubs | `scripts/no_stubs.sh` (stage 11) |
| P-VW-005 NoUnmeasuredCoverage | coverage artifact required in PR |
| P-VW-007 NoFakeConformance | conformance vector IDs in PR description must exist in `test/conformance/` |
| P-VW-008 NoHallucinatedDependencies | `cargo fetch --locked` (stage 2) |
| P-SC-002 NoMarketingVocabulary | regex lint on README + docs |
| P-SC-008 NoFakeEvidenceRefs | citation resolver runs on PR description |

### Test data hygiene

- Fixtures are checked-in under Apache-2.0 (workspace license)
- `evil-skill` secrets are clearly synthetic (`AKIA0000000000000000` pattern) — won't trigger external scanners
- No PII in any fixture
- `evil-skill` execution paths sandboxed: lifecycle hook bodies are *parsed and inspected*, never run

### Regression suite policy

Every reported issue (false positive, false negative, panic, schema violation) becomes:

1. A new fixture in `tests/fixtures/regressions/<issue-N>/`
2. A test asserting the fix
3. A line in `CHANGELOG.md` linking issue → fixture

---

## 7. Open Items (resolve during implementation, not blockers for design sign-off)

1. **Cosign keying strategy for v1**: keyless OIDC vs. fixed key file. Likely keyless for OSS contributors, fixed key for SkillPack's own release artifacts. Decide in Phase 0.
2. **CVE database choice**: OSV.dev (free, broad) vs. NVD (slower but canonical). Likely OSV.dev with a 24-hour cache. Decide in Phase 1.
3. **Coverage report parser strategy**: support `lcov`, `cobertura`, and `cargo-llvm-cov` JSON. Phase 6.
4. **Eval harness detection heuristics**: which directory layouts and runner configs constitute "an eval harness exists". Phase 8.
5. **`agentskills.compatibility` schema is loose**: spec doesn't enforce structure beyond "object". Phase 9 may need a sub-schema or document-by-convention.

These are bounded uncertainties, not architectural unknowns. Each has a clear phase to land in.

---

## 8. Success Definition

**v1 is shipped when:**

- All 13 CI stages green on `main`
- 9 dimensions all have content-based checkers (zero stubs)
- Fixture corpus grades A/B/C/D/F bit-stable across 10 runs
- SkillPack grades itself ≥ B
- Cosign-signed `v1.0.0` artifact published to GitHub Releases
- `skillpack` published to crates.io
- README dimensions table matches `domain::Dimension::all()`
- VS Code extension v1 (read-only diagnostics) published to marketplace

**v1 is in use when:**

- Internally graded ≥3 real skills
- ≥30 days of stable CLI surface (no breaking changes)

These two conditions together unlock v2 entry per Section 1.
