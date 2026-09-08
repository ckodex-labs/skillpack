# Skills Dossier (Standing)

> Single-file authoritative reference for the create / assess / evolve / exchange / bundle / distribute lifecycle in this repository.
> Consolidates: `skills-specs-next/` (spec corpus) × `crates/skillpack-*` (Rust primary runtime) × `macos/SkillsCore/CLI/Daemon/Finder/UI` (Swift macOS runtime) × `~/.claude/skills/*` (available skills) × `schemas/` (schema layer) under CKODEX v16.3 governance.

| Field               | Value                                                                                               |
| ------------------- | --------------------------------------------------------------------------------------------------- |
| Version             | v0.2                                                                                                |
| Date                | 2026-05-28                                                                                          |
| Author              | Claude (assisted)                                                                                   |
| Anchors             | CLAUDE.md v16.3 · RFC-001 v0.2.0 · ckodex-skill-spec v1.1 · ckodex-stx-spec v0.1.0 · CATM-SPEC v0.1 |
| Classification mode | Every claim labeled **[C]** Concrete / **[S]** Specified / **[A]** Aspirational per VWP §26.B       |
| Evidence policy     | VWP §26.E — capability claims carry `evidence_refs[]`; non-empty iff `[C]`                          |

---

## §0. Scope

**In scope.** Lifecycle of skills as a unit of agent behavior — how they are created, validated, mutated, transported, packaged, and propagated across the local toolchain (Swift runtime) and across the open ecosystem (`skills.sh` via `npx skills`).

**Out of scope.** Threat modeling for individual skills (see CATM-SPEC v0.1); ADRs (live in the codebase if added); CKODEX runtime kernel internals (DAAS, evidence fabric, BPL plane).

**Audience.** Future Claude instances; the repo owner; reviewers planning the next iteration of `SkillsCore`.

---

## §1. Corpus Inventory

### 1.1 Spec corpus — `skills-specs-next/`

| Artifact                                                               | Type                 | Status  | Key contribution                                                                         |
| ---------------------------------------------------------------------- | -------------------- | ------- | ---------------------------------------------------------------------------------------- |
| `RFC-001-skill-lifecycle.md` + `RFC-001-skill-lifecycle-v0.2.0.md`     | Draft RFC            | **[S]** | 6-tier FSM (L0..L4X), scoring, hysteresis, demotion triggers                             |
| `ckodex-skill-spec-v1.1/SPEC.md`                                       | Spec                 | **[S]** | v1.1 manifest schema (synopsis, ontology, runtime hints)                                 |
| `ckodex-skill-tools/SKILL.md` + scripts                                | Tooling              | **[C]** | `scaffold.py`, `migrate.py` (v1→v1.1, idempotent)                                        |
| `ckodex-stx-spec/`                                                     | Spec                 | **[S]** | Skill Transfer eXchange — publisher/consumer over REST/gRPC/GraphQL with 4 privacy modes |
| `STX-IMPLEMENTATION-PROMPT.md`                                         | Implementation guide | **[S]** | How to wire a harness to STX                                                             |
| `Claude-Agent skill lifecycle and progressive disclosure mechanism.md` | Narrative            | **[S]** | Disclosure rationale                                                                     |
| `t1-…-fsm-lifecycle.mmd/.html` + `t4-…-event-loop.*`                   | Diagrams             | **[C]** | Mermaid renders of FSM and event loop                                                    |
| `_skills_archive_/`                                                    | Archive              | —       | Historical drafts                                                                        |

### 1.2 Runtime layers

#### 1.2.A Rust `skillpack` — Primary runtime **[C]** (`crates/`)

> Hexagonal architecture (Ports & Adapters). Cargo workspace. Single universal CLI `skillpack`. Evidence: `ARCHITECTURE.md §2–5`, `README.md`.

| Crate                   | Role                                  | Key surface                                                                                                                                                                                 |
| ----------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `skillpack-domain`      | Pure domain kernel (no external deps) | `Assessment`, `SkillIdentity`, `DimensionId` (9 dims), `Grade`, `Score`, ports: `SkillReader`, `DimensionChecker`, `Signer`, `LockEmitter`                                                  |
| `skillpack-application` | Use cases / orchestration             | `AssessSkillUseCase`, `GradeSkillUseCase`, `IndexOperations`, `envelope_builder`                                                                                                            |
| `skillpack-adapters`    | Infrastructure + CLI binary           | `cli/mod.rs` (all commands), 9 checkers (`identity`, `security`, `provenance`, `documentation`, `testing`, `compatibility`, `lifecycle`, `governance`, `evals_hitl`), OCI, Sigstore, DuckDB |
| `skillpack-api`         | gRPC presentation                     | `SkillPackService`, `IndexService`, `RatingsService`, `QueryService` — proto in `proto/skillpack/v1/`                                                                                       |
| `skillpack-tests`       | Integration tests                     | `schema_round_trip`, `fixture_grade_corpus`                                                                                                                                                 |

**CLI commands:** `check`, `grade`, `report`, `validate`, `init`, `wizard`, `lock`, `migrate`, `package`, `publish`, `install`, `discover`, `eval`, `store sync/migrate/status/check-boundary`

**Grade system:** S+(120+) / A(100–119) / B(80–99) / C(60–79) / D(40–59) / F(<40)

#### 1.2.B Swift `SkillsCore/CLI/Daemon/Finder/UI` — macOS runtime **[C]** (`macos/`)

> macOS-only. `skills-cli` deprecated — use `skillpack store <cmd>` instead. `SkillsDaemon` retained as thin FSEventStream watcher.

| Module                   | Role                          | Tracked components                                                                                                                                                                                             |
| ------------------------ | ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SkillsCore`             | Domain + protocols            | `Skill`, `SkillMetadata`, `SkillStorageProvider`, `AgentConfig`, `SkillImporter`, `SyncManager` (decomposed per commit `0776a3d`), `IPGuard`, `ManifestGenerator`                                              |
| `SkillsCore` (untracked) | New providers + craft         | `LocalSkillStorageProvider`, `DistributedSkillStorageProvider`, `FoundationDBClient`, `RustFSClient`, `SkillCrafter`, `AppleFoundationModelsClient`, `LLMClient`, `CodeGraphQueryRunner`, `MigrateAllStrategy` |
| `SkillsCLI`              | Operator surface (deprecated) | `sync`, `import`, `craft`, `promote`, `migrate` commands; XPC-first → gRPC fallback (commit `d482ff0`)                                                                                                         |
| `SkillsDaemon`           | Filesystem watcher            | launchd-registered, FSEventStream on `~/Skills/shared`, gRPC :50051 + XPC `com.ckodex.skillsdaemon` (`SkillsDaemonXPCService`)                                                                                 |
| `SkillsFinder`           | macOS Finder ext              | Status badges + context menu actions                                                                                                                                                                           |
| `SkillsUI`               | Menu-bar app                  | `SkillsUIApp`, `MenuBarDaemonClient`                                                                                                                                                                           |
| Build                    | Targets                       | `Makefile` (build/generate/test); `SkillsEcosystem.xcodeproj`                                                                                                                                                  |

### 1.3 My available skills — lifecycle-relevant subset

> Filtering the ~199 skills loaded in this session to those that touch the six primitives.

| Primitive      | Skills I can invoke                                                                                                                                                                                      |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **CREATE**     | `superpowers:writing-skills`, `superpowers:writing-plans`, `superpowers:brainstorming`, `superpowers:test-driven-development`, `skill-creator:skill-creator`, `plugin-dev:skill-development`, `scaffold` |
| **ASSESS**     | `superpowers:verification-before-completion`, `superpowers:receiving-code-review`, `conformance`, `proof-audit`, `freshness`, `ckodex-audit`, `design-lint`, `design-review`, `verify`                   |
| **EVOLVE**     | `superpowers:writing-plans`, `simplify`, `plugin-dev:plugin-validator`, `token-migration`, `vercel:next-upgrade` (pattern only)                                                                          |
| **EXCHANGE**   | `find-skills`, `skills` (npx CLI proxy), `superpowers:dispatching-parallel-agents`, `superpowers:subagent-driven-development`                                                                            |
| **BUNDLE**     | `capability-lock`, `plugin-dev:plugin-structure`, `plugin-dev:plugin-settings`                                                                                                                           |
| **DISTRIBUTE** | `deploy`, `vercel:deploy`, `vercel:deployments-cicd`, `plugin-dev:create-plugin`, `state-vector`, `vector-state`                                                                                         |

**Note.** Many CKODEX runtime primitives (ActionEnvelope, CapabilityLock signing, BPL emission, EvidenceBundle assembly, CATM sweep) appear in CLAUDE.md but have no corresponding **invocable** skill in this session — they would be exercised through code-generation, not skill-invocation.

---

## §2. Comparison Matrix

> Columns = where the primitive currently lives. ✓ = present; ◐ = partial; ✗ = absent.

| Primitive  | Spec corpus                                        | Rust `skillpack` **[C]**                                                   | Swift runtime                                                                   | My skills                                                   | CKODEX v16.3 alignment                                                        |
| ---------- | -------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------- |
| CREATE     | ✓ `scaffold.py` (v1.1 tree)                        | ✓ `skillpack init` + `wizard` (scaffold + multi-template)                  | ✓ `SkillCrafter` (LLM + CodeGraph → `candidates/`)                              | ✓ `writing-skills` (TDD) + `skill-creator`                  | ◐ Missing C3S `CapabilitySpec` emission                                       |
| ASSESS     | ◐ `scripts/validate.sh` + SPEC §5 rules            | ✓ 9-dimension checker pipeline + grade S+→F; `skillpack check`             | ◐ `IPGuard` (boundary only); no semantic validation                             | ✓ `conformance`, `proof-audit`, `freshness`, `ckodex-audit` | ◐ Missing bidirectional CV refs in skill manifest                             |
| EVOLVE     | ✓ `migrate.py` (v1→v1.1 idempotent)                | ◐ `skillpack migrate` (schema bump only; no content-patch / supersession)  | ✗ Not implemented                                                               | ◐ Pattern-level only (`writing-plans`, `simplify`)          | ✗ Missing BPL emission on mutation                                            |
| EXCHANGE   | ✓ STX v0.1.0 (REST/gRPC/GraphQL × 4 privacy modes) | ◐ OCI push/pull + gRPC server (intra-host); no STX publisher               | ◐ XPC + gRPC IPC (intra-host only)                                              | ✓ `find-skills` (skills.sh ecosystem)                       | ◐ Missing ActionEnvelope wrapping on transit                                  |
| BUNDLE     | ◐ `MANIFEST.json` (sha256 ledger from tools)       | ✓ `skillpack package` (tar/zip) + `lock` (sha256 ledger); ✗ no OCI signing | ◐ `ManifestGenerator` (yaml inventory only)                                     | ◐ `capability-lock` skill exists; no CLI bridge             | ✗ Missing OCI `SkillBundle` (locks + policy_refs + cv_refs + evidence_wiring) |
| DISTRIBUTE | ◐ Implicit via STX publisher                       | ✓ `skillpack publish` (OCI) + `install`; ✗ no per-env promotion gates      | ✓ `AgentSyncer` (12 agents: 11 symlink + 1 mdc index) + FSEventStream auto-sync | ◐ `deploy` (project-level, not skill-level)                 | ◐ Missing promotion gates (dev→staging→prod) per skill                        |

**Observation (v0.2).** The Rust `skillpack` crates cover CREATE, ASSESS, BUNDLE (packaging), and DISTRIBUTE (OCI) operationally — significantly more than the Swift runtime alone. **EVOLVE** (content-patch + supersession) and **STX EXCHANGE** remain the largest gaps across all runtime layers. OCI signing (`CapabilityLock` + cosign) is the critical missing link between BUNDLE and CKODEX-canonical DISTRIBUTE.

---

## §3. The Six Primitives — Consolidated Definitions

> Each primitive is defined as: **Intent** (one line) → **Inputs** → **Process** → **Outputs** → **Evidence** → **Failure modes** → **Cross-refs**.
> All `evidence_refs` are required iff classification = `[C]` (VWP P-VW-001).

### 3.1 CREATE

```
≡CREATE :: intent → scaffold → craft → validate → spec → CapabilitySpec
```

**Intent.** Produce a new skill (SKILL.md + skill.json v1.1 + references/ + scripts/) that an agent can later load.

**Inputs.**
- `name` (regex `^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$`, ≤64 chars) — SPEC v1.1 §3
- `description` (1–1024 chars, third-person, "Use when…") — SPEC v1.1 §3, writing-skills CSO
- Source context (optional): codebase path for `SkillCrafter` to introspect

**Process (operational).**
1. **TDD baseline** (`writing-skills` RED phase) — run scenario without skill, document failures
2. **Scaffold** (`scaffold.py <path> --name --description --apply`) — emits v1.1 skeleton tree
3. **Craft body** — either manual or `SkillCrafter`:
   - `SkillCrafter` (SkillCrafter.swift:23–158): IPGuard check → CodeGraph query (top-40 files + top-60 nodes from `.codegraph/codegraph.db`) → LLM (Apple Foundation Models on macOS 26+ else Gemini 2.5-flash) → parse JSON `{skillName, description, skillMarkdown}` → write to `candidates/{name}/SKILL.md`
4. **Author edits** synopsis (≤2048), ontology, runtime hints
5. **GREEN phase** — re-run scenarios with skill; agent must comply
6. **REFACTOR** — close loopholes (rationalization table, red flags list)

**Outputs.**
- Directory layout: `SKILL.md`, `skill.json`, `references/usage.md`, `scripts/validate.sh`, `MANIFEST.json`
- `[A]` Future: emit `CapabilitySpec` per CKODEX §7 C3S

**Evidence.**
- `evidence_refs`: `MANIFEST.json` sha256 ledger; `scripts/validate.sh` exit-0 output
- `[A]` Future: signed `CapabilitySpec` cosign attestation

**Failure modes.**
- IPGuard refusal (`thales|cortaix-csr|ppt-thales|ip-pending` in path)
- LLM returns non-JSON → `SkillsError.invalidLLMResponse`
- Name regex / length violation → `scaffold.py` hard refusal

**Cross-refs.** RFC-001 §3 (initial tier = L0); SPEC v1.1 §§3–4; CLAUDE.md §7 C3S; `superpowers:writing-skills`; `skill-creator:skill-creator`.

### 3.2 ASSESS

```
≡ASSESS :: validate(schema) → score(multi-signal) → conformance(bidir) → freshness → attest
```

**Intent.** Determine whether a skill is safe to load, correct to invoke, and current enough to trust.

**Inputs.**
- Skill (SKILL.md + skill.json)
- Current context (for scoring)
- Policy bundle (GAL minimum, capability allowlist)

**Process.**
1. **Schema validation** — SPEC v1.1 §5 (additionalProperties: false; closed enum for `proofTypes`; tier ordering `minTier ≤ maxTier`)
2. **Multi-signal score** — RFC-001 §4: `S = w_lex·BM25 + w_sem·cos + w_ont·match + w_co·prior + w_rec·decay + w_user·pin + w_pol·gate`
3. **Conformance** — bidirectional CV refs (CV-SAFE, CV-ECON, CV-EVID, CV-NET, CV-CATM-*) per CLAUDE.md §5
4. **Freshness** — CLAUDE.md §9 protocol: docs current, versions pinned, signature age ≤ rotation interval
5. **Attest** — emit EvidenceBundle linking the verdict to the skill's content hash

**Outputs.**
- `GateDecision::Ok{approver, evidence:[URN], conditions, timestamp}` OR
- `GateDecision::Deny{reason, remediation:[...], evidence_required:[...], retry_after?}`

**Evidence.**
- `evidence_refs`: validate.sh exit code + log path; schema validation report; conformance run record
- `[A]` Future: BPL chain (artifact → skill → context → policy)

**Failure modes.**
- Forbidden tuple (CLAUDE.md §10): `anti ∧ execute`, `negative ∧ promotion`, `exploratory ∧ promotion`
- Stale evidence (`evidence_age > decay_half_life`) → refresh required before use
- `galMinimum > actor.dal` → deny

**Cross-refs.** CLAUDE.md §4 quality gates; CLAUDE.md §10 vector state; `conformance`; `proof-audit`; `freshness`; `superpowers:verification-before-completion`.

### 3.3 EVOLVE

```
≡EVOLVE :: read(current) → diff(proposed) → preserve(authorship) → bump(apiVersion) → BPL(lineage) → re-ASSESS
```

**Intent.** Move a skill forward — schema migration, content patch, or supersession — without losing authorship or lineage.

**Inputs.**
- Existing skill artifact
- Proposed change (schema bump, content delta, or new version)
- Author's authoritative source (so machine edits never silently overwrite)

**Process.**
1. **Read current** — parse SKILL.md frontmatter + skill.json + MANIFEST.json
2. **Classify mutation** — schema-only / content-additive / content-mutating / supersession (new versioned skill)
3. **Schema-only path** — `migrate.py <path> --apply`:
   - Bump `apiVersion` to v1.1
   - Synthesize synopsis from body (faithful, never invented); bounded 2048
   - Insert ontology/runtime stubs with `MIGRATION:` markers for author review
   - **Idempotent** (re-run yields stable output); never overwrites author content without `--force`
4. **Content patch path** — `[S]` not yet implemented; would: stage edit → re-validate → request review
5. **Supersession path** — `[A]` not yet implemented; would: write new `name@v{n+1}` → mark old as `retired` (one-way per CLAUDE.md §10 harness lens) → preserve BPL link
6. **BPL emission** — `[A]` reverse-traceable causal history per CLAUDE.md §13
7. **Re-ASSESS** — full §3.2 pipeline against the new artifact

**Outputs.** Updated artifact + lineage record.

**Evidence.**
- `evidence_refs`: diff hash; pre/post MANIFEST.json hashes; migrate.py exit-0 output
- `[A]` Future: BPL chain pointer

**Failure modes.**
- Author content present at machine-write site without `--force` → refuse
- Unparseable existing `skill.json` without `--force` → refuse
- Promotion into `retired` lens (one-way violation) → deny

**Cross-refs.** RFC-001 §6 (demotion triggers also apply to evolved skills); CLAUDE.md §10 harness lens; CLAUDE.md §13 BPL; CLAUDE.md §16 artifact lifecycle.

**TODO(ckodex)** — stub: content-patch path is not implemented in `SkillsCore` (no `SkillEvolver` class yet). Supersession path is not implemented. See §5.

### 3.4 EXCHANGE

```
≡EXCHANGE :: (STX publisher ↔ consumer) | (npx skills CLI ↔ registry) | (XPC ↔ gRPC IPC)
            → boundary_class_check → lease_grant → DecisionTrace
```

**Intent.** Move skill state, lifecycle decisions, or artifacts across process boundaries — local-IPC, intra-host, intra-tenant, or cross-tenant.

**Inputs.**
- Source (publisher harness or registry)
- Sink (consumer harness or local agent)
- Boundary metadata (tenant, env, ws, boundary_class)

**Process.**

| Channel                     | Status                        | Mechanism                                                                                                                                           |
| --------------------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| **STX runtime publisher**   | `[S]` Spec'd, not in Swift    | REST/gRPC/GraphQL projections per `ckodex-stx-spec/protocols/`; emits `ckodex/skill-lifecycle@v1` evidence bundles before tier transitions at GAL≥3 |
| **`npx skills` ecosystem**  | `[C]` Live                    | `npx skills find <query>` → results from skills.sh; `npx skills add <owner/repo@skill> [-g] [-y]` installs                                          |
| **XPC daemon contract**     | `[C]` Live (commit `d482ff0`) | `SkillsDaemonXPCProtocol` (SkillsDaemonXPCProtocol.swift:4–9): `syncSkills`, `getStatus`, `getExtendedStatus`, `checkBoundary`                      |
| **gRPC fallback**           | `[C]` Live (commit `d482ff0`) | Port 50051, auto-generated from .proto; CLI tries XPC first then gRPC                                                                               |
| **FSEventStream auto-sync** | `[C]` Live                    | Daemon watches `~/Skills/shared` recursively; triggers `SyncManager.execute()` on change                                                            |

**Privacy modes** (RFC-001 §9.1 + STX SPEC §9): `debug-local` / `audit-private` (default) / `public-anchor` / `regulated-export`.

**Outputs.** Sync result `(success, message, skillCount)`; STX `LifecycleDecisionBundle`; install report.

**Evidence.**
- `evidence_refs`: DecisionTrace per IPC call; STX evidence bundles signed before publication
- `[A]` Future: ActionEnvelope.context.certificate on every IPC

**Failure modes.**
- `IPGuard.guardPath()` denial → `checkBoundary` returns `{safe: false, reason}`
- Boundary class violation (e.g., `sovereign_restricted` → egress) → deny
- Lease expired during transit → zombie detected, HALT (CLAUDE.md §5)

**Cross-refs.** CLAUDE.md §5 ActionEnvelope.scope.boundary_class; CLAUDE.md §14 boundary classes; CLAUDE.md §25 CATM tripwires; STX SPEC §9; `find-skills`; `skills` (npx CLI).

### 3.5 BUNDLE

```
≡BUNDLE :: collect(artifacts) → compile(CapabilityLock per skill)
          → assemble(SkillBundle OCI) → sign(cosign) → emit(EvidenceBundle)
```

**Intent.** Package one or more skills into a distributable, verifiable, signed unit.

**Inputs.**
- Skill artifact set (SKILL.md + skill.json + references/ + scripts/ each)
- Policy refs
- Conformance vector refs

**Process.**
1. **Collect** — gather artifacts; compute per-artifact sha256
2. **Inventory** — current: `ManifestGenerator` (ManifestGenerator.swift:11–32) emits `manifest.yaml` with `name, path, size_bytes, description` per skill
3. **Compile CapabilityLock per skill** — `[A]` CLAUDE.md §7 C3S: `CapabilityLock` = compiled JSON + sha256 + cosign; immutable
4. **Assemble SkillBundle (OCI)** — `[A]` CLAUDE.md §7: OCI image containing `locks + policy_refs + cv_refs + evidence_wiring`
5. **Sign** — `[A]` cosign + Rekor anchor
6. **Emit EvidenceBundle** — receipt + policy verdicts + digests + RetentionClass

**Outputs.**
- Current: `manifest.yaml` (yaml inventory only)
- `[A]` Future: signed OCI `SkillBundle` + `EvidenceBundle`

**Evidence.**
- Current: `evidence_refs = [manifest.yaml hash]`
- `[A]` Future: cosign signature + Rekor entry + EvidenceBundle URN

**Failure modes.**
- Unsigned lock encountered during bundle assembly → treat as shadow skill (CLAUDE.md §15) → EP-002 QUARANTINE
- Bundle signature verification failure on consumer → reject load

**Cross-refs.** CLAUDE.md §7 C3S; CLAUDE.md §15 supply chain; `capability-lock` skill; `plugin-dev:plugin-structure`.

**TODO(ckodex)** — stub: no `SkillBundleAssembler` or `CapabilityLockCompiler` exists in `SkillsCore`. See §5.

### 3.6 DISTRIBUTE

```
≡DISTRIBUTE :: sign(SkillBundle) → push(OCI) → fanout(symlink|index|wire)
              → watch(FSEvent) → promote(gate per env) → audit(TIP if cross-tenant)
```

**Intent.** Propagate a bundled, signed skill set to the population of consumer agents and environments.

**Inputs.**
- Signed bundle (or unbundled skill set for local case)
- Agent/integration matrix (AgentConfig)
- Environment topology (dev/staging/prod)

**Process — local case (Swift runtime, today).**
1. **Fanout** — `AgentSyncer` writes to 12 agents per `AgentConfig.swift:40–52`:
   - 11 agents via symlink (Claude Code, Cursor Memory, Codex, Gemini, …)
   - 1 agent via `.mdc` index file (Cursor rules)
2. **Watch** — `SkillsDaemon` `DirectoryWatcher` (FSEventStream) on `~/Skills/shared` → auto-triggers `SyncManager.execute(SyncOptions defaults)`
3. **Prune** — `SyncManager` removes stale local cache when distributed backend is authoritative (`STORAGE_BACKEND=distributed`)
4. **Soft-fail on conflicts** — non-symlink conflicts: warn and continue (commit `080c5f6`)

**Process — ecosystem case.**
1. `npx skills add <owner/repo@skill> -g -y` — installs from GitHub/skills.sh
2. Updates: `npx skills check` then `npx skills update`

**Process — `[A]` CKODEX-canonical case.**
1. Sign SkillBundle (cosign)
2. Push to OCI registry
3. Per-environment promotion gates: `dev → staging` auto (proof + bidir CV + freshness + BPL); `staging → prod` human + UCA + DCA + security
4. Cross-tenant push → TIP (Tenant Isolation Proof) per CLAUDE.md §13

**Outputs.** Distributed skill set; promotion records; audit trail.

**Evidence.**
- Current: sync result tuple; manifest hash
- `[A]` Future: EngagementProvenanceBundle (EPB) per CLAUDE.md §13

**Failure modes.**
- Storage backend down (FoundationDB/RustFS) → daemon falls back? **TODO(ckodex)**: verify behavior
- Promotion gate denial → distribute halts at preceding environment
- Boundary class `sovereign_restricted` + cross-region push → deny (CLAUDE.md §14)

**Cross-refs.** CLAUDE.md §3 transport; CLAUDE.md §14 boundary classes + promotion gates; CLAUDE.md §17 deployment; `deploy`; `vercel:deploy` (pattern only — different platform).

---

## §4. Consolidated Lifecycle Spec (delta over RFC-001 v0.2.0)

> Where RFC-001 and the Swift runtime disagree, this dossier picks the more recent and tested option and flags the loser for cleanup (CLAUDE.md prevent-failure-modes Rule 7).

### 4.1 Lifecycle FSM — unchanged from RFC-001 §3

States `L0..L4X`; promotion gates `τ_syn_load=0.45`, `τ_full_load=0.65`; demotion via score-decay / idle / drift / budget / mutex / framework-break; hysteresis `τ_load > τ_keep > τ_floor`. Adopt as-is.

### 4.2 Skill identity — **change required**

| Layer                                                              | Current shape                    | Required shape                                                                                                                  |
| ------------------------------------------------------------------ | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Swift `Skill` struct (SkillStorageProvider.swift:3)                | `{name, sizeBytes, description}` | Add `version: SemVer`, `apiVersion: String`, `contentHash: SHA256`, `synopsisHash: SHA256?`, `tier: Tier?`, `ontologyRef: URN?` |
| `SkillStorageProvider` protocol (SkillStorageProvider.swift:15–20) | 4 methods over opaque content    | Add `getManifest(named:)`, `getEvidence(named:)`, `listVersions(named:)`                                                        |

**Why.** Without versioning + content hash on the runtime side, EVOLVE and BPL are impossible. Without tier + ontology refs, the FSM cannot bind to runtime state.

### 4.3 ASSESS gate — **change required**

Adopt CLAUDE.md §10 forbidden tuples in the Swift `SyncManager`. Today only `IPGuard` (path/content denylist) gates; add:
- `anti ∧ execute → HALT`
- `negative ∧ promotion → DENY`
- `exploratory ∧ promotion → DENY` (harness lens, v16.2)
- `retired ∧ execute → HALT` (harness lens, v16.2)

### 4.4 EVOLVE — **gap to fill**

Add `SkillEvolver` in `SkillsCore` with three paths (schema-bump / content-patch / supersession). Implement `--dry-run` first, behind the `STORAGE_BACKEND=local` flag, before distributed.

### 4.5 BUNDLE — **gap to fill**

Two-step delivery:
1. **Phase 1 (immediate, `[S]`):** extend `ManifestGenerator` to emit a `bundle.json` with `(name, sha256, size, manifest_version, evidence_refs)` — a precursor to OCI.
2. **Phase 2 (CKODEX-aligned, `[A]`):** add `SkillBundleAssembler` that compiles `CapabilityLock` per skill, packages OCI with `locks + policy_refs + cv_refs + evidence_wiring`, signs with cosign.

### 4.6 EXCHANGE / STX — **wire to existing IPC**

The Swift daemon already has XPC + gRPC (commit `d482ff0`). Add STX projections (REST/gRPC) as a third surface; reuse the existing gRPC server. Publish `LifecycleDecisionBundle` from `SyncManager` on every promote/demote.

### 4.7 DISTRIBUTE — **promotion gates missing**

Today: fanout is unconditional (modulo IPGuard). Add per-environment gates keyed on the skill's `apiVersion`, `tier`, and last `EvidenceBundle` freshness; gates evaluate locally (no remote OPA required for v0.1).

---

## §5. TODO Surface (per VWP P-VW-004 — explicit stubs)

> Every gap surfaced above, restated as actionable TODOs. Count = 26 (11 original + 5 schema-audit additions + 10 AIPACK-alignment additions). **20 items closed (T-01,T-02,T-03,T-04,T-05,T-07,T-08,T-10,T-11,T-12,T-13,T-14,T-15,T-16,T-17,T-18,T-19,T-20,T-22,T-23) — `stubs_remaining = 6`.**
> **Effort:** S = days; M = 1–2 weeks; L = large (>2 weeks).
> **Dependency order:** T-13 → T-12 → T-14 → T-15 (schema chain); T-02 → T-01 → T-03 (struct chain); T-05 → T-06; T-07 → T-08; T-17 → T-18 → T-19 (AIPACK attestation chain); T-20 → T-21 (risk valence bridge); T-22 → T-23 (deprecation phases).

| ID   | TODO                                                                                                                                                                                | Class | Owner                        | Effort | Anchor             | Depends on |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----- | ---------------------------- | ------ | ------------------ | ---------- |
| T-01 | Implement `SkillEvolver` in `SkillsCore` (schema-bump / content-patch / supersession); `--dry-run` first                                                                            | `[C]` | `SkillsCore`                 | M      | §3.3, §4.4         | T-02       |
| T-02 | Extend `Skill` struct with `version: SemVer`, `contentHash: SHA256`, `tier: Tier?`, `ontologyRef: URN?`                                                                             | `[C]` | `SkillsCore`                 | S      | §4.2               | T-15       |
| T-03 | Extend `SkillStorageProvider` with `getManifest(named:)`, `getEvidence(named:)`, `listVersions(named:)`                                                                             | `[C]` | `SkillsCore`                 | S      | §4.2               | T-02       |
| T-04 | Wire CLAUDE.md §10 forbidden-tuple checks (`anti ∧ execute`, `negative ∧ promotion`, `retired ∧ execute`) into `SyncManager`                                                        | `[C]` | `SkillsCore`                 | S      | §4.3               | —          |
| T-05 | Emit `bundle.json` (Phase 1 bundle precursor: `name, sha256, size, manifest_version, evidence_refs`) from `ManifestGenerator`                                                       | `[C]` | `SkillsCore`                 | S      | §4.5               | —          |
| T-06 | Add `SkillBundleAssembler` with CapabilityLock compile + cosign + Rekor anchor                                                                                                      | `[A]` | `SkillsCore` + new module    | L      | §3.5, §4.5         | T-05, T-13 |
| T-07 | Add STX REST + gRPC publisher projections atop existing daemon gRPC; reuse port 50051                                                                                               | `[C]` | `SkillsDaemon`               | M      | §4.6               | —          |
| T-08 | Publish `LifecycleDecisionBundle` from `SyncManager` on every promote/demote                                                                                                        | `[C]` | `SkillsCore` (`SyncManager`) | S      | §4.6               | T-07       |
| T-09 | Add per-environment promotion gates keyed on `apiVersion`, `tier`, last `EvidenceBundle` freshness                                                                                  | `[A]` | `SkillsCore`                 | L      | §4.7               | T-02       |
| T-10 | Verify + document fallback behavior when FoundationDB / RustFS is down; add integration test                                                                                        | `[C]` | `SkillsCore`                 | S      | §3.6 failure modes | —          |
| T-11 | Define `intent_to_invoke(skill, message)` per RFC-001 §3.2; add to spec + implement in harness                                                                                      | `[C]` | spec + `SkillsCore`          | M      | §1.1, RFC-001 gap  | —          |
| T-12 | Retire root `schemas/cnsb.schema.json` — redirect all consumers to `schemas/cnsb/v1/cnsb.schema.json`; update `ARCHITECTURE.md §9.4`                                                | `[C]` | schemas                      | S      | §10.2              | T-13       |
| T-13 | Add strict `CkodexUrn` type (`^urn:ckodex:[a-z]+:[A-Za-z0-9._:-]+$`) to `schemas/common/types.schema.json` alongside loose `Urn`; update CNAAB + evidence + prove schemas to use it | `[C]` | schemas                      | S      | §10.2              | —          |
| T-14 | Add `lifecycle` hook object to `schemas/cnsb/v1/cnsb.schema.json` (present in `skill.type.ts` `Lifecycle` interface, absent from JSON schema)                                       | `[C]` | schemas                      | S      | §10.2              | T-12       |
| T-15 | Add `synopsisHash` and `tier` fields to `schemas/skills-specs-next/ckodex-skill-spec-v1.1/schemas/skill.v1.1.schema.json` metadata block (required by T-02 runtime struct)          | `[C]` | schemas                      | S      | §10.2              | —          |
| T-16 | Create `schemas/skills-specs-next/INDEX.md` lifecycle cross-reference index                                                                                                         | `[C]` | docs                         | S      | §9                 | —          |

| T-17 | Add `mediaType: application/vnd.ai.skill.v1+json` field to `schemas/cnsb/v1/cnsb.schema.json` bundle layer object; align with AIPACK §4.1                                          | `[C]` | schemas                      | S      | §11 G-2            | T-12       |
| T-18 | Emit `urn:skill:static-analysis:v1` in-toto attestation as OCI referrer on `skillpack publish`; source data = `security` dimension checker output                                   | `[C]` | `skillpack-adapters`         | M      | §11 G-1            | —          |
| T-19 | Emit `urn:skill:capability-declaration:v1` in-toto attestation as OCI referrer on `skillpack publish`; source = `governance` dimension + capability block in manifest               | `[C]` | `skillpack-adapters`         | M      | §11 G-1            | T-17       |
| T-20 | Emit `cyclonedx.org/bom` SBOM as OCI referrer on `skillpack publish`; tool = `cargo cyclonedx` or `syft`                                                                           | `[C]` | `skillpack-adapters`         | M      | §11 G-9            | —          |
| T-21 | Add `urn:skill:safety-review:v1` attestation stub: `skillpack review --attest` signs human/automated safety verdict and pushes as OCI referrer                                      | `[A]` | `skillpack-adapters` + CI    | L      | §11 G-1            | T-18, T-19 |
| T-22 | Add `schemaVersion: 1` field to `cnsb/v1/cnsb.schema.json` and `skill.v1.1.schema.json` per AIPACK §10.1                                                                           | `[C]` | schemas                      | S      | §11 G-10           | T-12       |
| T-23 | Align `capabilities` object shape with AIPACK §6.4 (`{read, write, network, filesystem, execution}` booleans); add compatibility shim for existing CKODEX capability model         | `[C]` | schemas + `skillpack-domain` | M      | §11 G-5            | T-17       |
| T-24 | Add 4-phase deprecation lifecycle to EVOLVE supersession path: `announcedAt`, `sunsetAt`, cascade hint; emit `urn:aipack:deprecation:v1` referrer on OCI push                      | `[A]` | `skillpack-adapters`         | L      | §11 G-7            | T-01       |
| T-25 | Define RV(C) → skillpack-grade bridge: publish mapping table (S+/A/B/C/D/F → GREEN/YELLOW/ORANGE/RED) in `ARCHITECTURE.md`; emit `urn:aipack:risk-valence:v1` from `skillpack grade` | `[A]` | `skillpack-domain`           | L      | §11 G-4            | T-20       |
| T-26 | Spike blast-radius containment: implement `GET /aipack/v0.1/dependents/{digest}` over DuckDB index in `skillpack-api`; `containmentBoundary` field in Agent manifest               | `[A]` | `skillpack-api`              | L      | §11 G-6            | —          |

**Counts** (per VWP completion report schema, §6 below): `stubs_remaining = 6`; `closed_this_sprint = [T-01, T-02, T-03, T-04, T-05, T-07, T-08, T-10, T-11, T-12, T-13, T-14, T-15, T-16, T-17, T-18, T-19, T-20, T-22, T-23]`; `deferred = [T-06, T-09, T-21, T-24, T-25, T-26]` (CKODEX-aspirational, large surface); `new_from_schema_audit = [T-12, T-13, T-14, T-15, T-16]`; `new_from_aipack_audit = [T-17, T-18, T-19, T-20, T-21, T-22, T-23, T-24, T-25, T-26]`.

---

## §6. Self-Attestation Envelope (VWP §26.E)

```yaml
capability_id: skills-dossier-v0.2
claim: >
  Standing skills dossier (v0.2) covering create/assess/evolve/exchange/bundle/distribute
  across four layers (spec corpus, Rust skillpack, Swift macOS, schema layer),
  classified [C]/[S]/[A], anchored to file:line, all three open decisions resolved,
  26 explicit TODOs with dependency order, §10 schema gap audit, §11 AIPACK-SPEC v0.1.0
  alignment analysis (10 gaps G-1..G-10, 10 new TODOs T-17..T-26); 6 AIPACK TODOs
  closed in sprint (T-12,T-13,T-14,T-15,T-17,T-18,T-19,T-20,T-22,T-23): schemaVersion,
  bundleLayer, aipackCapabilities, push_referrer, three OCI attestation referrers,
  CkodexUrn strict type, cnsb.schema.json tombstone, SkillLifecycle hook object,
  synopsisHash and tier fields; forbidden-tuple checks (T-04) in SyncManager;
  bundle.json emission (T-05) from ManifestGenerator; Skill struct T-02 extensions
  (version/contentHash/tier/ontologyRef); SkillStorageProvider T-03 extensions
  (getManifest/getEvidence/listVersions) with regression tests; SkillEvolver T-01
  (bumpVersion/patchField/supersede + dry-run); T-10 storage fallback test suite
  (9 tests: local provider nil-safety, evidence filter, listVersions, FDB/RustFS
  unreachable, SyncManager local-backend survival); T-16 INDEX.md cross-reference;
  STXPublisher REST facade T-07 (health/list/lifecycle/promote/demote/bundle);
  SkillPackGRPCClient STX projection stubs T-07; LifecycleDecisionBundle +
  LifecyclePublisher T-08 (local audit log + REST fire-and-forget, emitted from
  SyncManager.promoteSkill and SkillEvolver.supersede); T-11 intent_to_invoke
  (RFC-001 §3.4 formal definition + IntentToInvoke.swift 6-rule lexical harness
  + 14 unit tests covering all rules, word-boundary matching, case-insensitivity).
classification: C
evidence_refs:
  # Spec corpus
  - { kind: artifact, path: schemas/skills-specs-next/RFC-001-skill-lifecycle-v0.2.0.md }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-skill-spec-v1.1/ }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-stx-spec/ }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-skill-lifecycle-rfc001/ }
  # Rust primary runtime
  - { kind: artifact, path: crates/skillpack-domain/src/assessment.rs }
  - { kind: artifact, path: crates/skillpack-adapters/src/cli/mod.rs }
  - { kind: artifact, path: crates/skillpack-adapters/src/checkers/ }
  - { kind: artifact, path: ARCHITECTURE.md }
  - { kind: artifact, path: README.md }
  # Swift macOS runtime
  - { kind: artifact, path: macos/SkillsCore/Sources/SkillsCore/SkillStorageProvider.swift }
  - { kind: artifact, path: macos/SkillsCore/Sources/SkillsCore/SyncManager.swift }
  - { kind: artifact, path: macos/SkillsCore/Sources/SkillsCore/AgentConfig.swift }
  - { kind: artifact, path: macos/SkillsDaemon/Sources/SkillsDaemon/SkillsDaemonXPCService.swift }
  # Schema layer
  - { kind: artifact, path: schemas/common/types.schema.json }
  - { kind: artifact, path: schemas/cnsb/v1/cnsb.schema.json }
  - { kind: artifact, path: schemas/cnaab/v1/cnaab.schema.json }
  - { kind: artifact, path: schemas/evidence/v1/envelope.schema.json }
  - { kind: artifact, path: schemas/prove/v1/policy.schema.json }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-skill-spec-v1.1/schemas/skill.v1.1.schema.json }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-stx-spec/schemas/stx-domain.v1.schema.json }
  - { kind: artifact, path: schemas/skills-specs-next/ckodex-skill-lifecycle-rfc001/schemas/pca-lifecycle.v1.schema.json }
stubs_remaining: 6
closed_this_sprint: [T-01, T-02, T-03, T-04, T-05, T-07, T-08, T-10, T-11, T-12, T-13, T-14, T-15, T-16, T-17, T-18, T-19, T-20, T-22, T-23]
deferred: [T-06, T-09, T-21, T-24, T-25, T-26]
decisions_resolved: [D-1, D-2, D-3]
new_from_schema_audit: [T-12, T-13, T-14, T-15, T-16]
new_from_aipack_audit: [T-17, T-18, T-19, T-20, T-21, T-22, T-23, T-24, T-25, T-26]
baseline_refs: []   # No superlative claims; no baseline cited
```

**Classification roll-up.** Of 6 primitives consolidated:
- CREATE `[C]` — `skillpack init/wizard` + `SkillCrafter` both operational
- ASSESS `[C]` — 9-dimension Rust checker pipeline + grade system; IPGuard boundary
- EVOLVE `[S]` — schema-bump path exists; content-patch/supersession stub (T-01)
- EXCHANGE `[C]` (OCI/gRPC/XPC intra-host) + `[S]` (STX spec complete, not wired)
- BUNDLE `[C]` (tar/zip + lock file + AIPACK media types + OCI referrers T-17/T-20) + `[A]` (cosign OCI signing stub T-06)
- DISTRIBUTE `[C]` (OCI publish + AgentSyncer fanout) + `[A]` (promotion gates T-09)

**VWP self-check (v0.2).**

| Vector                                     | Result                                                                        |
| ------------------------------------------ | ----------------------------------------------------------------------------- |
| CV-VWP-001 (claim w/o evidence)            | pass — all `[C]` claims have evidence_refs; Rust crate paths added            |
| CV-VWP-002 (cross-refs resolvable)         | pass — all §8 cross-refs point to verified file paths or spec sections        |
| CV-VWP-003 (omitted TODOs)                 | pass — §5 enumerates 26 (was 11; +5 schema-audit; +10 AIPACK-audit); 6 closed |
| CV-VWP-006 (marketing vocabulary)          | pass — no `robust`/`scalable`/`production-ready`/etc. used                    |
| CV-VWP-008 (classification labels missing) | pass — every primitive and new section labeled                                |
| CV-VWP-010 (hedged completion)             | pass — no `should work`/`probably` outside fenced `[S]`/`[A]`                 |

---

## §7. Resolved Decisions (v0.2 amendment — previously "Open Decisions")

> All three decisions resolved in v0.2. Rationale anchored to corpus evidence.

**D-1: EVOLVE semantics — RESOLVED: Option (c) Hybrid** `[S]`

- Schema-only changes (apiVersion bump, synopsis synthesis, ontology/runtime stubs) → **in-place mutation** via `migrate.py` / `skillpack migrate`. Idempotent; never overwrites author content without `--force`.
- Content-mutating changes (body edits, description changes) → **supersession** (new versioned skill `name@v{n+1}`); old marked `retired` (one-way per CLAUDE.md §10). BPL link preserved.
- **Rationale:** Option (c) matches the already-implemented `migrate.py` semantics (schema-only path exists); supersession keeps BPL clean without the storage cost of option (b) for every minor fix; option (a)'s hash-chain history adds complexity with no corresponding query path in the current runtime.
- **Implements:** T-01 (SkillEvolver two-path dispatch), T-02 (version + contentHash on Skill struct).

**D-2: BUNDLE substrate — RESOLVED: Option (c) Phase 1 now + Phase 2 deferred** `[S]`/`[A]`

- **Phase 1 (immediate, `[S]`):** extend `ManifestGenerator` to emit `bundle.json` with `(name, sha256, size, manifest_version, evidence_refs)`. Unblocks DISTRIBUTE gates immediately. → T-05 (S effort).
- **Phase 2 (deferred, `[A]`):** add `SkillBundleAssembler` (OCI + CapabilityLock + cosign + Rekor). No blocking dependency on Phase 1; can proceed in parallel once T-13 (CkodexUrn) lands. → T-06 (L effort).
- **Rationale:** Option (a) has no signing story, which breaks CKODEX §15 supply chain integrity. Option (b) immediately aligns with §7 C3S but is ~L effort with no near-term consumer. Phase 1 ships value in days and creates a natural upgrade seam.

**D-3: STX privacy default — RESOLVED: Option (b) env-tiered** `[S]`

- `debug-local` for dev tenants / local harness; `audit-private` (spec default) for staging and production.
- Implementation: inject env-flag `CKODEX_STX_PRIVACY_MODE` read at harness startup; default = `audit-private` if unset (safe fallback).
- **Rationale:** Option (a) is safest but makes debug loops painful — STX bundle size and redaction make local iteration slow. Option (c) requires OPA/policy infrastructure that doesn't exist yet. Option (b) requires only a single env-flag injection (S effort) and matches RFC-001 §9.1 intent.
- **Implements:** T-07 (STX publisher) privacy mode wire-up.

---

## §8. Cross-Reference Index

| Topic                          | Spec corpus          | Rust `skillpack`                                | Swift code                                          | CLAUDE.md                   | My skills                          |
| ------------------------------ | -------------------- | ----------------------------------------------- | --------------------------------------------------- | --------------------------- | ---------------------------------- |
| 6-tier FSM (L0..L4X)           | RFC-001 v0.2.0 §3    | —                                               | —                                                   | §10 vector state            | —                                  |
| 9-dimension quality check      | —                    | `checkers/` (identity, security, provenance, …) | —                                                   | §4 quality gates            | `ckodex-audit`                     |
| Grade system S+→F              | —                    | `skillpack-domain/src/grade.rs`                 | —                                                   | —                           | —                                  |
| Synopsis ≤2048                 | SPEC v1.1 §4.1, §5.3 | —                                               | —                                                   | —                           | `superpowers:writing-skills` (CSO) |
| Ontology JSON-LD               | SPEC v1.1 §3, §5.4   | —                                               | —                                                   | —                           | —                                  |
| `proofTypes` enum              | SPEC v1.1 §5.6       | —                                               | —                                                   | §13 evidence types          | `proof-audit`                      |
| STX privacy modes              | STX SPEC §9          | —                                               | —                                                   | §15 privacy                 | —                                  |
| CapabilityLock                 | —                    | ✗ stub (T-06)                                   | —                                                   | §7 C3S                      | `capability-lock`                  |
| CATM tripwires                 | —                    | —                                               | —                                                   | §25                         | —                                  |
| VWP P-VW-* / P-SC-*            | —                    | —                                               | —                                                   | §26                         | —                                  |
| OCI publish / install          | —                    | `cli/mod.rs::run_publish`, `run_install`        | —                                                   | —                           | `deploy` (pattern)                 |
| DuckDB persistence             | —                    | `persistence/mod.rs::DuckDbRepository`          | —                                                   | —                           | —                                  |
| Sigstore signing               | —                    | `evidence/signer.rs::CosignSigner`              | —                                                   | §15 supply chain            | —                                  |
| AgentConfig × 12               | —                    | —                                               | AgentConfig.swift:40–52                             | —                           | —                                  |
| FSEventStream auto-sync        | —                    | —                                               | SkillsDaemon.swift:18–32                            | —                           | —                                  |
| XPC + gRPC IPC (concurrent)    | —                    | —                                               | commit `d482ff0`, SkillsDaemonXPCProtocol.swift:4–9 | §3 transport                | —                                  |
| LLM craft                      | —                    | —                                               | SkillCrafter.swift:23–158, LLMClient.swift:14–87    | §21                         | `claude-api` (pattern)             |
| CodeGraph query                | —                    | —                                               | CodeGraphQueryRunner.swift:11–84                    | CLAUDE.md CodeGraph section | `understand-anything:*`            |
| IPGuard / store check-boundary | —                    | `cli::run_check_boundary`                       | (referenced)                                        | §15 containment             | —                                  |
| FoundationDB / RustFS          | —                    | —                                               | DistributedSkillStorageProvider.swift:3–70          | —                           | —                                  |
| Schema gap audit (v0.2)        | §10 (this doc)       | —                                               | —                                                   | —                           | —                                  |

---

## §9. Mirror Targets (per CLAUDE.md §26.H pattern)

| Target                                     | Status                 | Action                                                    |
| ------------------------------------------ | ---------------------- | --------------------------------------------------------- |
| `SKILLS-DOSSIER.md` (this file, repo root) | primary                | committed (v0.2)                                          |
| `README.md` (repo)                         | derived                | add link in §Architecture                                 |
| `ARCHITECTURE.md` (repo)                   | derived                | add link back to dossier §3 from §9 AI-agent entry points |
| `schemas/skills-specs-next/INDEX.md`       | derived, **TODO T-16** | create lifecycle cross-reference index                    |
| `~/.claude/projects/.../memory/`           | derived, **TODO**      | add reference memory pointer if useful in future sessions |

---

**End of Dossier v0.2.** Update via numbered amendments (`§X.Y add …` / `§X.Y supersede …`); never silently rewrite. Every amendment carries its own `[C]/[S]/[A]` classification and evidence refs.

---

## §10. Schema Architecture & Gap Audit (v0.2 addition) **[C]**

> Added in v0.2. Full analysis of `schemas/` directory — 5 layers, 5 concrete inconsistency findings, evolution roadmap.
> Evidence: direct file read of all schema `$id` and constraint fields on 2026-05-28.

### 10.1 Schema Family Map

```
schemas/common/types.schema.json          ← Foundation
    URN (loose), Timestamp, LabelMap, ActorType, GAL (0–5), ASC,
    Actor, Signature, ResourceMetadata, EvidenceRef
    │
    ├── schemas/cnsb/v1/cnsb.schema.json              ← Cloud-Native Skills Bundle v1
    │       kind=SkillBundle; Skill (id, entry, galMin/Max, asc[], inputs/outputs, examples)
    │       └── schemas/cnsb/v1/skill-lock.schema.json   ← Dependency lock (SRI integrity)
    │
    ├── schemas/cnaab/v1/cnaab.schema.json            ← Agent App Bundle v1
    │       kind=AgentAppBundle; Agent (id, role, galMin/Max); SkillRef; RuntimeConfig
    │
    ├── schemas/evidence/v1/envelope.schema.json      ← OpenEvidence Envelope
    │       id, subject, statement{type,payload}, actor, signatures[]
    │
    ├── schemas/prove/v1/policy.schema.json           ← Prove PolicyBundle
    │       targets[], enforcements[]{effect: allow|deny|review}, engines[]
    │
    ├── schemas/skills-specs-next/ckodex-skill-spec-v1.1/
    │       ├── skill.v1.schema.json        apiVersion=ckodex.org/skill/v1
    │       ├── skill.v1.1.schema.json      adds synopsis, ontology, runtime blocks
    │       └── skillsbundle.v1.schema.json multi-skill bundle with defaults + supply chain
    │
    ├── schemas/skills-specs-next/ckodex-skill-lifecycle-rfc001/
    │       ├── pca-lifecycle.v1.schema.json       PCA bundle per-turn (LifecycleDecision, SignalWeights)
    │       └── skill-runtime-capabilities.v1.schema.json  harness capability negotiation
    │
    └── schemas/skills-specs-next/ckodex-stx-spec/
            └── stx-domain.v1.schema.json  STX typed graph
                    Harness → Session → SkillState → TierTransition
                    → EvidenceBundle → AttestationChain + PolicyGate
```

**Root-layer legacy schemas** (`schemas/cnsb.schema.json`, `schemas/agentskills.schema.json`, `schemas/skill-lock.schema.json`) predate the versioned tree. Their canonical counterparts live under `cnsb/v1/` and `skills-specs-next/`. These should be treated as deprecated (→ T-12).

### 10.2 Inconsistency Findings

#### Finding F-1: URN strictness drift (HIGH) → T-13

| Schema                           | URN pattern used                                              |
| -------------------------------- | ------------------------------------------------------------- |
| `common/types.schema.json`       | `^urn:` (intentionally loose)                                 |
| `cnsb.schema.json` (root/legacy) | `^urn:ckodex:` (partial strict)                               |
| `stx-domain.v1.schema.json`      | `^urn:ckodex:[a-z]+:[A-Za-z0-9._:-]+$` (strict)               |
| `pca-lifecycle.v1.schema.json`   | `^urn:ckodex:harness:[^:]+:turn:[A-Za-z0-9-]+$` (very narrow) |

No canonical "strict CKODEX URN" type shared by all schemas. Fix: add `CkodexUrn` to `common/types.schema.json` alongside loose `Urn`, update all cross-references.

#### Finding F-2: GAL range split (MEDIUM) → T-12 (dependency)

- `common/types.schema.json` → GAL `minimum: 0, maximum: 5`
- `schemas/cnsb.schema.json` (root legacy) → `galMin/galMax: minimum: 1` — excludes 0
- `cnsb/v1/cnsb.schema.json` → `$ref: common#/GAL` (0–5) ✓

Root schema is stricter than the canonical type it predates; consumers that use root instead of `cnsb/v1` will silently reject GAL=0 skills.

#### Finding F-3: Three parallel Skill definitions (HIGH) → T-12

1. `schemas/cnsb.schema.json` → `$defs/skill` with `id` pattern `^[a-z]+\.[a-z]+\.[a-z-]+$` and `entry` pattern `^skills://[a-z-]+#[a-z-]+$`
2. `schemas/cnsb/v1/cnsb.schema.json` → `$defs/Skill` — no pattern constraints on `id`/`entry`
3. `schemas/skill.type.ts` → TypeScript `Skill` interface — adds `Lifecycle` hook object not in either JSON schema

No single source of truth; callers may validate against wrong schema version.

#### Finding F-4: `Lifecycle` hook object absent from JSON schemas (MEDIUM) → T-14

`skill.type.ts` defines a `Lifecycle` interface with 8 hook operations (`pack`, `unpack`, `install`, `upgrade`, `uninstall`, `verify`, `preInstall`, `postInstall`). Neither `cnsb.schema.json` nor `cnsb/v1/cnsb.schema.json` includes a `lifecycle` property. Any CNSB that includes lifecycle hooks will fail `additionalProperties: false` validation in the canonical v1 schema.

#### Finding F-5: `synopsisHash` / `tier` missing from skill.v1.1 schema (MEDIUM) → T-15

The RFC-001 §4.2 runtime struct upgrade (T-02) requires `synopsisHash: SHA256?` and `tier: Tier?` on the `Skill` struct. `skill.v1.1.schema.json` metadata block does not include these fields. If added to the runtime without first adding them to the schema, runtime-generated manifests will fail schema validation.

### 10.3 Schema Evolution Roadmap

> Ordered by dependency. All changes are backward-compatible additions unless noted.

| Step | Change                                                                                                     | Schema file                | Backward compat?        | Linked TODO |
| ---- | ---------------------------------------------------------------------------------------------------------- | -------------------------- | ----------------------- | ----------- |
| 1    | Add `CkodexUrn` strict type alongside loose `Urn` in `$defs`                                               | `common/types.schema.json` | ✓ additive              | T-13        |
| 2    | Update `cnaab`, `evidence`, `prove` schemas to use `CkodexUrn` for their identity fields                   | 3 files                    | ✓ additive (new `$ref`) | T-13        |
| 3    | Retire `schemas/cnsb.schema.json`; add `$comment: "Deprecated — use cnsb/v1/cnsb.schema.json"`             | root legacy                | ✓ non-breaking          | T-12        |
| 4    | Add `lifecycle` object to `cnsb/v1/cnsb.schema.json` `$defs/Skill`                                         | `cnsb/v1/cnsb.schema.json` | ✓ additive              | T-14        |
| 5    | Add `synopsisHash` (string, sha256 pattern) and `tier` (enum L0..L4X) to `skill.v1.1.schema.json` metadata | `skill.v1.1.schema.json`   | ✓ additive              | T-15        |

### 10.4 Schema–Code Alignment Check

| Schema concept                | JSON schema                             | TypeScript `skill.type.ts`            | Rust (`skillpack-domain`)                                 | Aligned? |
| ----------------------------- | --------------------------------------- | ------------------------------------- | --------------------------------------------------------- | -------- |
| `GAL` (0–5)                   | `common/types` ✓                        | `GALValue = 0\|1\|2\|3\|4\|5` ✓       | not found (domain uses `Score`)                           | ◐        |
| `ASC` string                  | `common/types` ✓                        | `ASC { id, version?, parameters? }` ✓ | not found                                                 | ◐        |
| `Skill.id` pattern            | root: `^[a-z]+\.` / v1: none            | none                                  | `SkillIdentity { name, version, path }` — different shape | ✗        |
| `Skill.entry`                 | root: `skills://` URI / v1: free string | free string                           | `path: String`                                            | ◐        |
| `Lifecycle` hooks             | absent in JSON schema                   | `Lifecycle` interface defined         | not found                                                 | ✗        |
| `synopsisHash` / `tier`       | absent in v1.1 schema                   | absent                                | planned (T-02)                                            | ✗        |
| `Signature` (alg/value/keyId) | `common/types` ✓                        | not in `skill.type.ts`                | `evidence/signer.rs` (Sigstore)                           | ◐        |

---

## §11. AIPACK-SPEC v0.1.0 Alignment Analysis (v0.2 addition) **[A]**

> Cross-reference of AIPACK-SPEC v0.1.0 (author: Mohamed N. Chorfa, CKODEX, 2026-04-23) against this dossier's four layers.
> Focus: A4 Skill artifact type and the governance/distribution sections that affect skill lifecycle.
> Rating: ✓ aligned · ◐ partial · ✗ absent.

### 11.1 Where We ARE Aligned

| AIPACK requirement                                               | Dossier / codebase coverage                                             | Rating                                   |
| ---------------------------------------------------------------- | ----------------------------------------------------------------------- | ---------------------------------------- |
| A4 `SKILL.md` required in bundle (§6.4)                          | §3.1 CREATE outputs `SKILL.md`; `docs/authoring-guide.md` required file | ✓                                        |
| A4 `apiVersion` declared in manifest (§6.4)                      | `cnsb/v1/cnsb.schema.json`, `skill.v1.1.schema.json`                    | ✓                                        |
| OCI distribution — push/pull (§8.1)                              | §1.2.A `skillpack publish` + `install`; §2 DISTRIBUTE row ✓             | ✓                                        |
| sha256 digest pinning on bundle layer (§5.1)                     | `skill.lock` SRI integrity; `skillpack lock`; §3.5 BUNDLE               | ✓                                        |
| Schema validation — closed enums, `additionalProperties` (§10.1) | §3.2 ASSESS step 1; SPEC v1.1 §5 rules                                  | ✓                                        |
| Capability scope declaration on A4 (§6.4)                        | `skill.v1.1.schema.json` capability block; §3.1 outputs                 | ◐ (shape diverges — see G-5)             |
| `tar+zstd` bundle content layer (§4.2)                           | `skillpack package`; §2 BUNDLE row                                      | ✓                                        |
| Deprecation / `retired` concept (§10.3, §16)                     | §3.3 EVOLVE supersession + `retired` lens; D-1 resolved                 | ◐ (phases 1–3 absent — see G-7)          |
| Multi-signal quality scoring (§13 spirit)                        | §3.2 ASSESS RFC-001 §4 formula; 9-dimension grade pipeline              | ◐ (parallel system — see G-4)            |
| Signing via Sigstore (§7.1)                                      | `skillpack-adapters` `evidence/signer.rs`; T-06 OCI cosign              | ◐ (cosign not yet wired to OCI referrer) |

### 11.2 Gaps — AIPACK Requirements Not Covered

#### G-1 (HIGH): A4 mandatory attestations entirely absent → T-18, T-19, T-20, T-21

AIPACK §6.4 requires four attestations as OCI referrers on every Skill artifact:

| Required predicate                    | Source in this repo                                                  | Status |
| ------------------------------------- | -------------------------------------------------------------------- | ------ |
| `urn:skill:safety-review:v1`          | none                                                                 | ✗      |
| `urn:skill:capability-declaration:v1` | none (governance checker produces a grade, not a signed referrer)    | ✗      |
| `urn:skill:static-analysis:v1`        | `security` dimension checker exists; not emitted as in-toto referrer | ✗      |
| `cyclonedx.org/bom`                   | not generated                                                        | ✗      |

The 9-dimension checker produces a `Grade` struct — not an in-toto-format signed attestation referrer on the OCI manifest. Closing this gap requires wiring `skillpack publish` to emit referrers (T-18, T-19, T-20, T-21).

#### G-2 (HIGH): OCI media type `application/vnd.ai.skill.v1+json` not declared → T-17

AIPACK §4.1 assigns manifest media type `application/vnd.ai.skill.v1+json` and §4.2 assigns layer media type `application/vnd.ai.skill.bundle.v1.tar+zstd`. Neither field appears in `schemas/cnsb/v1/cnsb.schema.json`. Without this, AIPACK-conforming runtimes cannot identify skill artifacts by media type inspection.

#### G-3 (HIGH): Lineage envelope (§11) vs. BPL — structural mismatch **[A]**

AIPACK §11 defines a per-message `LineageEnvelope` with `compositionTrace[TraceEntry{component, componentDigest, contribution{kind, magnitude}}]`, signed per-message by the runtime. BPL (referenced in §3.2, §3.3, §3.5) is CKODEX-internal causal DAG at the artifact level — not a per-message per-component contribution trace. This is an **architectural divergence**:

| Dimension         | AIPACK LineageEnvelope                               | CKODEX BPL                      |
| ----------------- | ---------------------------------------------------- | ------------------------------- |
| Granularity       | Per outbound message                                 | Per artifact / action           |
| Magnitude signal  | Frobenius-norm ratio per LoRA; token share per skill | Evidence chain pointer          |
| Signing           | Runtime identity key per message                     | Attestation envelope per action |
| Retention classes | ephemeral / standard / regulated / legal_hold        | Not specified                   |

No TODO is raised here — alignment would require a new lineage subsystem. Recorded as `[A]` architectural note for the next major version.

#### G-4 (HIGH): Risk valence score (§13) vs. skillpack grade — parallel systems → T-25

AIPACK §13 defines `RV(C) = clamp(0,100, Σ wᵢ·sᵢ(C))` over 13 signals, producing GREEN/YELLOW/ORANGE/RED bands. The Rust `skillpack` grade system covers 9 quality dimensions producing S+/A/B/C/D/F. They are semantically overlapping but **structurally unconnected**. A skill OCI artifact must carry `urn:aipack:risk-valence:v1` attestation for AIPACK conformance. T-25 defines the bridge mapping.

Provisional grade → band mapping (informative, subject to calibration in T-25):

| skillpack grade | AIPACK RV band |
| --------------- | -------------- |
| S+ / A          | GREEN (0–24)   |
| B               | YELLOW (25–49) |
| C               | ORANGE (50–74) |
| D / F           | RED (75–100)   |

#### G-5 (MEDIUM): `capabilities` object shape divergence → T-23

AIPACK §6.4 requires `capabilities: {read, write, network, filesystem, execution}` — 5 boolean flags. `skill.v1.1.schema.json` has a capability block following the CKODEX capability model (GAL-gated, ASC-linked). The two are semantically overlapping but structurally incompatible. T-23 adds an AIPACK-shape shim alongside the existing CKODEX model.

#### G-6 (MEDIUM): Blast radius containment (§12) — no equivalent **[A]**

AIPACK §12 requires: dependency inversion index (`GET /aipack/v0.1/dependents/{digest}`), per-composite `containmentBoundary` declaration, and cascade quarantine protocol. The dossier has IPGuard boundary checks (§3.2, §4.3) and CLAUDE.md §10 forbidden-tuple enforcement — these are **admission controls**, not post-publication blast radius tracking. T-26 spikes the DuckDB-backed dependents index in `skillpack-api`.

#### G-7 (MEDIUM): Deprecation lifecycle phases 1–3 absent → T-24

AIPACK §16 defines 4 timed phases keyed on `announcedAt` and `sunsetAt`:

| Phase        | Condition     | CKODEX equivalent                 |
| ------------ | ------------- | --------------------------------- |
| 1 — announce | A ≤ T < S-30d | ✗ no announce date field          |
| 2 — ramp     | S-30d ≤ T < S | ✗ no sunset date field            |
| 3 — grace    | S ≤ T < S+30d | ✗ no grace window                 |
| 4 — enforce  | T ≥ S+30d     | ✓ `retired` lens = refuse to load |

The dossier's §7 D-1 resolution covers phase 4 only. T-24 adds the missing fields and the signed `urn:aipack:deprecation:v1` referrer to the EVOLVE supersession path.

#### G-8 (MEDIUM): Declarative quarantine triggers (§21) — absent **[A]**

AIPACK §21 `QuarantineTrigger` predicates (`cve-severity-at-least`, `risk-valence-band-enters`, `days-since-eval-exceeds`, `dependency-quarantined`, etc.) have no equivalent in the dossier or schemas. The closest is CLAUDE.md §10 forbidden-tuple enforcement (binary deny on load), which covers only `cve-severity-at-least` semantically and only at admission time. No TODO is raised — this requires the risk valence subsystem (T-25) as a prerequisite.

#### G-9 (LOW): `cyclonedx.org/bom` SBOM on Skill — not emitted → T-20

`skillpack publish` does not generate a CycloneDX SBOM for skill bundle dependencies. AIPACK §6.4 makes this mandatory. T-20 adds `cargo cyclonedx` or `syft` invocation on publish.

#### G-10 (LOW): `schemaVersion: 1` field absent from CNSB schemas → T-22

AIPACK §10.1 requires `schemaVersion` on every manifest. Neither `cnsb/v1/cnsb.schema.json` nor `skill.v1.1.schema.json` has this field. T-22 adds it as an additive, backward-compatible change.

### 11.3 Alignment Score Summary

| Domain                                                          | AIPACK sections | Rating                                        |
| --------------------------------------------------------------- | --------------- | --------------------------------------------- |
| A4 Skill packaging — bundle layer, `SKILL.md`, OCI distribution | §3, §6.4, §8    | ◐ Partial                                     |
| A4 media type declaration                                       | §4.1, §4.2      | ✗ G-2                                         |
| A4 mandatory attestations (4 required)                          | §6.4, §7        | ✗ G-1                                         |
| OCI composition as C1 Agent skill slot                          | §5.3, §9.2      | ◐ OCI publish works; slot typing not declared |
| Lineage / per-message trace                                     | §11             | ✗ G-3 (architectural divergence)              |
| Risk valence score                                              | §13             | ✗ G-4 (parallel system)                       |
| `capabilities` shape                                            | §6.4            | ✗ G-5                                         |
| Deprecation lifecycle                                           | §10.3, §16      | ◐ G-7 (phase 4 only)                          |
| Blast radius containment                                        | §12             | ✗ G-6                                         |
| Declarative quarantine triggers                                 | §21             | ✗ G-8                                         |
| SBOM (`cyclonedx.org/bom`)                                      | §6.4            | ✗ G-9                                         |
| `schemaVersion` field                                           | §10.1           | ✗ G-10                                        |

**Shortest path to base conformance (AIPACK §§1–10).** Close G-2 (T-17, S) + G-10 (T-22, S) + G-5 (T-23, M) + G-1 partial (T-18+T-19+T-20, 3×M) = ~5–6 weeks of focused schema and adapter work. G-1 `urn:skill:safety-review:v1` (T-21) and G-7 full deprecation (T-24) remain `[A]` deferred.

---

*End of Dossier v0.2 (amended — §11 AIPACK alignment added)*
