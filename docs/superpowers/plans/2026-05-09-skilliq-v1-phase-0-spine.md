# SkillPack v1 — Phase 0 Spine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended for this plan — components are designed for parallel Sonnet dispatch) or `superpowers:executing-plans` for inline execution. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Build the seven foundational pieces (C0.0–C0.6) that every dimension checker in v1 depends on: canonical `DimensionId` enum, signed-evidence envelope output, cosign signing, CNSB skill-lock emission, stub-checker self-reporting, `--dry-run` safety wiring, and a fixture corpus.

**Architecture:** Phase 0 reconciles the in-tree drift (current 9 enum variants ≠ canonical 9-dim set) and lays the schema-bound output substrate. After Phase 0, each dimension checker (Phases 1–9) is a focused content-analysis task that consumes the spine. No checker rewrite happens in Phase 0.

**Tech Stack:** Rust 1.85 / 2024, `jsonschema = 0.40.2`, `sigstore = 0.13.0`, `sha2 = 0.10.9`, `clap = 4.5.56`, `serde_json = 1.0.149`, existing in-tree schemas at `schemas/evidence/v1/envelope.schema.json` and `schemas/cnsb/v1/skill-lock.schema.json`.

---

## Decisions Required Before Execution

Each item has a safe default; user must accept or override before fan-out.

| ID | Decision | Default |
|----|----------|---------|
| **D1** | Grade-letter boundaries (overall_score → A/B/C/D/F) | A ≥ 90, B ≥ 80, C ≥ 70, D ≥ 60, F < 60 |
| **D2** | README "real content" minimum (non-stub) | ≥ 200 words **AND** ≥ 2 H2 sections |
| **D3** | Phase ordering after Phase 0 | Descending weight: Sec(18)→Prov(14)→Identity(11)→Doc(11)→Test(10)→Lifecycle(10)→Gov(9)→Evals(9)→Compat(8) |
| **D4** | Stub-count surface format | JSON `meta.stub_dimensions: ["Provenance", ...]` + CLI banner if `len > 0` |
| **D5** | `skillpack grade --fail-below=B` exit-code policy | Exit 1 iff any input grade < threshold |
| **D6** | Fixture corpus minimum (count + identities) | 4 fixtures: `perfect-a/`, `missing-security/`, `malformed-frontmatter/`, `cnsb-no-lifecycle/` |
| **D7** | GAL vocab during v1 (schemas use 0–5; CLAUDE.md uses DAL 0–4) | Use schema GAL 0–5 verbatim in v1; defer reconciliation to v2 |

---

## File Map

**Create:**

- `crates/skillpack-domain/src/envelope.rs` — `EvidenceEnvelope` domain type
- `crates/skillpack-domain/src/stub_marker.rs` — `IsStub` trait + helper
- `crates/skillpack-application/src/envelope_builder.rs` — `EnvelopeBuilder` use case
- `crates/skillpack-adapters/src/signing/mod.rs` — `Signer` port + module root
- `crates/skillpack-adapters/src/signing/cosign.rs` — `CosignSigner` adapter
- `crates/skillpack-adapters/src/lock/mod.rs`, `lock/emitter.rs` — `LockEmitter`
- `crates/skillpack-adapters/src/cli/dry_run.rs` — `DryRun` guard + macro
- `crates/skillpack-adapters/tests/spine_integration.rs` — end-to-end Phase 0 test
- `tests/fixtures/skillpack/perfect-a/` (and 3 sibling fixture dirs) — full skill packages
- `crates/skillpack-domain/tests/envelope_schema.rs` — round-trip validation against `schemas/evidence/v1/envelope.schema.json`

**Modify:**

- `crates/skillpack-domain/src/dimension.rs` — replace `DimensionId` enum + `standard_dimensions()` with canonical 9-dim set
- `crates/skillpack-domain/src/lib.rs` — re-export `envelope`, `stub_marker`
- `crates/skillpack-domain/src/ports.rs` — add `is_stub(&self) -> bool { false }` default to `DimensionChecker`
- `crates/skillpack-application/src/lib.rs` — re-export `envelope_builder`
- `crates/skillpack-application/src/assess_skill.rs` — collect `stub_dimensions`; emit envelope on completion
- `crates/skillpack-application/src/generate_report.rs` — surface `meta.stub_dimensions`
- `crates/skillpack-adapters/src/checkers/mod.rs` — annotate the 7 placeholder checkers as `is_stub() = true` (until Phases 1–9 replace them)
- `crates/skillpack-adapters/src/cli/mod.rs` — add `--dry-run` global flag; wire on lock + future commands
- `crates/skillpack-adapters/Cargo.toml` — add `sigstore = "0.13.0"`, `jsonschema = "0.40.2"`

**Reference (read-only, do not modify):**

- `schemas/evidence/v1/envelope.schema.json`
- `schemas/cnsb/v1/skill-lock.schema.json`
- `schemas/cnsb/v1/cnsb.schema.json`

---

## Component C0.0 — Canonical DimensionId Reconciliation

**Blocks:** all other components. Must land first.

**Files:** `crates/skillpack-domain/src/dimension.rs`; cascading touches in `crates/skillpack-adapters/src/checkers/mod.rs`, `crates/skillpack-application/src/assess_skill.rs`.

**Interface change (full replacement):**

```rust
pub enum DimensionId {
    IdentityManifest,
    Security,
    Provenance,
    Documentation,
    Testing,
    Compatibility,
    Lifecycle,
    Governance,
    EvalsHitl,
}

pub fn standard_dimensions() -> Vec<Dimension> {
    vec![
        Dimension { id: DimensionId::IdentityManifest, weight: Weight(11) },
        Dimension { id: DimensionId::Security,         weight: Weight(18) },
        Dimension { id: DimensionId::Provenance,       weight: Weight(14) },
        Dimension { id: DimensionId::Documentation,    weight: Weight(11) },
        Dimension { id: DimensionId::Testing,          weight: Weight(10) },
        Dimension { id: DimensionId::Compatibility,    weight: Weight(8)  },
        Dimension { id: DimensionId::Lifecycle,        weight: Weight(10) },
        Dimension { id: DimensionId::Governance,       weight: Weight(9)  },
        Dimension { id: DimensionId::EvalsHitl,        weight: Weight(9)  },
    ]
}
```

**Tasks:**

- [ ] **C0.0.1** Write failing test `crates/skillpack-domain/src/dimension.rs::tests::canonical_weights_sum_to_100`:

  ```rust
  #[test]
  fn canonical_weights_sum_to_100() {
      let total: u8 = standard_dimensions().iter().map(|d| d.weight.value()).sum();
      assert_eq!(total, 100);
  }
  ```

  Run: `cargo test -p skillpack-domain canonical_weights_sum_to_100`. Expected: FAIL (compile error — old enum variants).
- [ ] **C0.0.2** Replace `DimensionId` enum + `name()` + `all()` per interface above. Update `Default for Dimension` to `IdentityManifest, Weight(11)`.
- [ ] **C0.0.3** In `crates/skillpack-adapters/src/checkers/mod.rs`: rename `StructureChecker` → drop (will be replaced by `IdentityManifestChecker` in Phase 3); rename `TemplatesChecker`, `McpChecker`, `AccessibilityChecker` → drop from `all_checkers()`. Add three new placeholder structs: `TestingChecker`, `CompatibilityChecker`, `EvalsHitlChecker` with stub `check()` returning `Score::dimension(50.0)`. Replace `all_checkers()` body to return the canonical 9.
- [ ] **C0.0.4** Run: `cargo build --workspace`. Expected: PASS. Run: `cargo test -p skillpack-domain canonical_weights_sum_to_100`. Expected: PASS.
- [ ] **C0.0.5** Commit:

  ```bash
  git add crates/skillpack-domain/src/dimension.rs crates/skillpack-adapters/src/checkers/mod.rs
  git commit -m "feat(domain): reconcile DimensionId to canonical 9-dim set (Phase 0.0)"
  ```

---

## Component C0.1 — EnvelopeBuilder

**Depends on:** C0.0. **Blocks:** C0.2.

**Files:** `crates/skillpack-domain/src/envelope.rs` (NEW), `crates/skillpack-application/src/envelope_builder.rs` (NEW), `crates/skillpack-domain/tests/envelope_schema.rs` (NEW).

**Interface:**

```rust
// crates/skillpack-domain/src/envelope.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEnvelope {
    pub id: String,                       // urn:ckodex:evidence:...
    pub subject: Subject,                 // { urn, kind: "SkillBundle", labels? }
    pub statement: Statement,             // { type: "SkillAssessment", payload }
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub actor: Actor,                     // { kind: "agent", id: "skillpack" }
    #[serde(skip_serializing_if = "Option::is_none")] pub gal: Option<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)] pub asc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub chain: Option<Chain>,
    pub signatures: Vec<Signature>,       // minItems: 1
}
```

```rust
// crates/skillpack-application/src/envelope_builder.rs
pub struct EnvelopeBuilder { /* fields */ }

impl EnvelopeBuilder {
    pub fn new(subject_urn: impl Into<String>) -> Self;
    pub fn with_assessment(mut self, payload: serde_json::Value) -> Self;
    pub fn with_chain(mut self, previous: Vec<String>) -> Self;
    pub fn build_unsigned(self) -> EvidenceEnvelope;  // signatures: vec![]
}
```

**Tasks:**

- [ ] **C0.1.1** Add `chrono = { version = "0.4.43", features = ["serde"] }` and `uuid = { version = "1.20", features = ["v4"] }` to `crates/skillpack-domain/Cargo.toml` (already in workspace deps; just inherit).
- [ ] **C0.1.2** Write failing schema-conformance test `crates/skillpack-domain/tests/envelope_schema.rs`:

  ```rust
  #[test]
  fn unsigned_envelope_validates_minus_signatures() {
      let env = EnvelopeBuilder::new("urn:ckodex:skill:test:demo")
          .with_assessment(serde_json::json!({"overall": 92, "grade": "A"}))
          .build_unsigned();
      let json = serde_json::to_value(&env).unwrap();
      // signatures field must exist; minItems: 1 is enforced post-signing
      assert!(json["signatures"].is_array());
      assert_eq!(json["statement"]["type"], "SkillAssessment");
      assert_eq!(json["subject"]["kind"], "SkillBundle");
  }
  ```

  Run: `cargo test -p skillpack-domain envelope_schema`. Expected: FAIL (no module).
- [ ] **C0.1.3** Implement `envelope.rs` with `Subject`, `Statement`, `Actor`, `Chain`, `Signature` (Signature := `{ keyid: String, sig: String, algo: String }`), `EvidenceEnvelope` per interface. Add `pub mod envelope;` + `pub use envelope::*;` to `crates/skillpack-domain/src/lib.rs`.
- [ ] **C0.1.4** Implement `envelope_builder.rs` per interface. `id` := `format!("urn:ckodex:evidence:{}", uuid::Uuid::new_v4())`; `issued_at` := `chrono::Utc::now()`; `actor` := `Actor { kind: "agent".into(), id: "skillpack".into() }`. Add `pub mod envelope_builder;` to `crates/skillpack-application/src/lib.rs`.
- [ ] **C0.1.5** Run: `cargo test -p skillpack-domain envelope_schema`. Expected: PASS.
- [ ] **C0.1.6** Commit:

  ```bash
  git add crates/skillpack-domain/src/envelope.rs crates/skillpack-domain/tests/envelope_schema.rs \
          crates/skillpack-application/src/envelope_builder.rs crates/skillpack-domain/src/lib.rs \
          crates/skillpack-application/src/lib.rs
  git commit -m "feat(domain,application): add EvidenceEnvelope + EnvelopeBuilder (Phase 0.1)"
  ```

---

## Component C0.2 — CosignSigner

**Depends on:** C0.1. **Blocks:** integration step.

**Files:** `crates/skillpack-adapters/src/signing/mod.rs`, `cosign.rs` (NEW); update `crates/skillpack-adapters/Cargo.toml`.

**Interface:**

```rust
// crates/skillpack-adapters/src/signing/mod.rs
#[async_trait::async_trait]
pub trait Signer: Send + Sync {
    async fn sign_envelope(&self, env: &mut EvidenceEnvelope) -> Result<(), SignError>;
}

// crates/skillpack-adapters/src/signing/cosign.rs
pub struct CosignSigner { /* keyref, mode: Keyless | KeyFile { path } */ }
impl CosignSigner {
    pub fn keyless() -> Self;
    pub fn from_key_file(path: impl AsRef<Path>) -> Result<Self, SignError>;
}
#[async_trait::async_trait]
impl Signer for CosignSigner { /* impl */ }
```

**Tasks:**

- [ ] **C0.2.1** Add `sigstore = "0.13.0"` to `crates/skillpack-adapters/Cargo.toml` (`[dependencies]` section, inherit from workspace).
- [ ] **C0.2.2** Write failing test `crates/skillpack-adapters/tests/cosign_signer.rs`:

  ```rust
  #[tokio::test]
  async fn signer_appends_signature_to_envelope() {
      use tempfile::NamedTempFile;
      let mut env = build_test_envelope();      // helper: returns unsigned envelope
      let key = generate_test_keyfile().await;  // helper: writes ephemeral cosign key
      let signer = CosignSigner::from_key_file(key.path()).unwrap();
      signer.sign_envelope(&mut env).await.unwrap();
      assert_eq!(env.signatures.len(), 1);
      assert!(!env.signatures[0].sig.is_empty());
      assert_eq!(env.signatures[0].algo, "ecdsa-p256-sha256");
  }
  ```

  Run: `cargo test -p skillpack-adapters cosign_signer`. Expected: FAIL.
- [ ] **C0.2.3** Implement `signing/mod.rs` with `Signer` trait + `SignError` enum (`thiserror`). Re-export `CosignSigner`.
- [ ] **C0.2.4** Implement `cosign.rs`:
  - `KeyFile` mode: deserialize cosign key file via `sigstore::cosign::CosignCapabilities`, sign canonical-JSON serialization of envelope (without `signatures`), populate `env.signatures` with `Signature { keyid: pubkey_fingerprint, sig: base64(signature), algo: "ecdsa-p256-sha256" }`.
  - `Keyless` mode: stub returning `SignError::NotYetImplemented` for v1 (keyless requires Fulcio; deferred).
- [ ] **C0.2.5** Run: `cargo test -p skillpack-adapters cosign_signer`. Expected: PASS.
- [ ] **C0.2.6** Commit:

  ```bash
  git add crates/skillpack-adapters/src/signing crates/skillpack-adapters/tests/cosign_signer.rs \
          crates/skillpack-adapters/Cargo.toml
  git commit -m "feat(adapters): add CosignSigner (key-file mode) (Phase 0.2)"
  ```

---

## Component C0.3 — LockEmitter

**Depends on:** C0.0. **Independent** of C0.1/C0.2.

**Files:** `crates/skillpack-adapters/src/lock/mod.rs`, `emitter.rs` (NEW).

**Interface:**

```rust
pub struct LockEmitter;
impl LockEmitter {
    /// Compute SRI integrity (`sha256-{base64}`) for a byte slice.
    pub fn compute_integrity(bytes: &[u8]) -> String;
    /// Emit a CNSB skill-lock JSON for the given skill manifest + file digests.
    pub fn emit(skill_urn: &str, file_digests: &[(PathBuf, String)]) -> serde_json::Value;
}
```

**Tasks:**

- [ ] **C0.3.1** Write failing test `crates/skillpack-adapters/tests/lock_emitter.rs`:

  ```rust
  #[test]
  fn integrity_is_sha256_base64() {
      let h = LockEmitter::compute_integrity(b"hello");
      assert!(h.starts_with("sha256-"));
      // sha256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
      // base64 of those raw bytes:
      assert_eq!(h, "sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=");
  }

  #[test]
  fn emit_produces_valid_lock_shape() {
      let lock = LockEmitter::emit("urn:ckodex:skill:t:demo", &[
          (PathBuf::from("SKILL.md"), "sha256-abc=".into()),
      ]);
      assert_eq!(lock["apiVersion"], "cnsb.ckodex.org/v1");
      assert_eq!(lock["kind"], "SkillLock");
      assert_eq!(lock["spec"]["files"][0]["path"], "SKILL.md");
  }
  ```

  Run: `cargo test -p skillpack-adapters lock_emitter`. Expected: FAIL.
- [ ] **C0.3.2** Implement `lock/emitter.rs`:
  - `compute_integrity` := `format!("sha256-{}", base64::encode(sha2::Sha256::digest(bytes)))` (use `base64 = "0.22"` if not in workspace; otherwise `data_encoding`).
  - `emit` returns `json!({ "apiVersion": "cnsb.ckodex.org/v1", "kind": "SkillLock", "metadata": { "urn": skill_urn }, "spec": { "files": [...] } })` matching `schemas/cnsb/v1/skill-lock.schema.json`.
- [ ] **C0.3.3** Run: `cargo test -p skillpack-adapters lock_emitter`. Expected: PASS.
- [ ] **C0.3.4** Commit:

  ```bash
  git add crates/skillpack-adapters/src/lock crates/skillpack-adapters/tests/lock_emitter.rs
  git commit -m "feat(adapters): add LockEmitter for CNSB skill-lock (Phase 0.3)"
  ```

---

## Component C0.4 — Stub-Count Surface

**Depends on:** C0.0. Independent of others.

**Files:** `crates/skillpack-domain/src/ports.rs` (modify); `crates/skillpack-application/src/assess_skill.rs` (modify); `crates/skillpack-application/src/generate_report.rs` (modify); `crates/skillpack-adapters/src/checkers/mod.rs` (modify).

**Interface change to existing trait:**

```rust
// crates/skillpack-domain/src/ports.rs — append default method
pub trait DimensionChecker: Send + Sync {
    fn dimension(&self) -> DimensionId;
    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>);
    /// Returns true iff this checker is a placeholder (file_exists-only / no content analysis).
    /// Phase 0 marks 7 checkers true; Phases 1–9 flip to false as content checks ship.
    fn is_stub(&self) -> bool { false }
}
```

**Tasks:**

- [ ] **C0.4.1** Write failing test `crates/skillpack-application/src/assess_skill.rs::tests::report_lists_stub_dimensions` (after C0.0.3 the stub checkers exist):

  ```rust
  #[test]
  fn report_meta_includes_stub_dimensions() {
      let report = run_assessment_against_fixture("perfect-a");
      let stubs: Vec<&str> = report.meta.stub_dimensions.iter().map(|s| s.as_str()).collect();
      assert!(stubs.contains(&"Provenance"));    // Phase 0: still stub
      assert!(stubs.contains(&"Testing"));       // Phase 0: still stub
      assert!(!stubs.contains(&"Security"));     // already content-based
      assert!(!stubs.contains(&"IdentityManifest")); // already content-based (was Structure)
  }
  ```

  Run: `cargo test -p skillpack-application stub_dimensions`. Expected: FAIL.
- [ ] **C0.4.2** Add `is_stub` default to `DimensionChecker` per interface above.
- [ ] **C0.4.3** Override `is_stub() -> bool { true }` on the 7 placeholder checkers in `crates/skillpack-adapters/src/checkers/mod.rs`: `ProvenanceChecker`, `GovernanceChecker`, `LifecycleChecker`, `DocumentationChecker`, plus the three new placeholders from C0.0.3 (`TestingChecker`, `CompatibilityChecker`, `EvalsHitlChecker`). Leave `SecurityChecker` and the renamed `IdentityManifestChecker` (formerly `StructureChecker`) at default `false`.
- [ ] **C0.4.4** Modify `crates/skillpack-application/src/assess_skill.rs` and the report struct in `generate_report.rs` to add `meta: AssessmentMeta { stub_dimensions: Vec<String> }`. Populate by iterating registered checkers, calling `is_stub()`, and pushing `dimension().name().to_string()` when true.
- [ ] **C0.4.5** Run: `cargo test -p skillpack-application stub_dimensions`. Expected: PASS.
- [ ] **C0.4.6** Commit:

  ```bash
  git add crates/skillpack-domain/src/ports.rs crates/skillpack-application/src/assess_skill.rs \
          crates/skillpack-application/src/generate_report.rs \
          crates/skillpack-adapters/src/checkers/mod.rs
  git commit -m "feat: surface stub-checker count in assessment meta (Phase 0.4)"
  ```

---

## Component C0.5 — `--dry-run` Wiring

**Depends on:** none (independent — but commits AFTER C0.0 to avoid enum churn).

**Files:** `crates/skillpack-adapters/src/cli/dry_run.rs` (NEW), `crates/skillpack-adapters/src/cli/mod.rs` (modify).

**Interface:**

```rust
// dry_run.rs
pub struct DryRun(pub bool);
impl DryRun {
    /// Execute `f` only when not in dry-run; otherwise log "[DRY-RUN] would: {description}".
    pub fn perform<F: FnOnce() -> R, R: Default>(&self, description: &str, f: F) -> R {
        if self.0 { tracing::info!(target: "dry_run", "would: {}", description); R::default() }
        else { f() }
    }
}
```

**Tasks:**

- [ ] **C0.5.1** Write failing test `crates/skillpack-adapters/tests/dry_run.rs`:

  ```rust
  #[test]
  fn dry_run_skips_side_effect() {
      let mut counter = 0;
      let dr = DryRun(true);
      dr.perform("increment counter", || counter += 1);
      assert_eq!(counter, 0);

      let dr = DryRun(false);
      dr.perform("increment counter", || counter += 1);
      assert_eq!(counter, 1);
  }
  ```

  Run: `cargo test -p skillpack-adapters dry_run`. Expected: FAIL.
- [ ] **C0.5.2** Implement `dry_run.rs` per interface; export from `cli/mod.rs`.
- [ ] **C0.5.3** In `crates/skillpack-adapters/src/cli/main.rs`, add a global flag `#[arg(long, global = true)] pub dry_run: bool` to the top-level `Cli` struct (or wherever `clap::Parser` is derived). Wire into `lock` subcommand only for v1 (other artifact-producing commands inherit when added).
- [ ] **C0.5.4** Run: `cargo test -p skillpack-adapters dry_run`. Expected: PASS. Manual smoke: `cargo run -p skillpack-adapters -- lock --dry-run examples/skill.txt` should print `[DRY-RUN] would: write skill.lock` and exit 0 without creating a file.
- [ ] **C0.5.5** Commit:

  ```bash
  git add crates/skillpack-adapters/src/cli
  git commit -m "feat(cli): add --dry-run global flag + DryRun guard (Phase 0.5)"
  ```

---

## Component C0.6 — Fixture Corpus

**Depends on:** none (independent).

**Files:** create `tests/fixtures/skillpack/<name>/...` for 4 fixtures.

| Fixture | Purpose | Required files |
|---------|---------|----------------|
| `perfect-a/` | should grade A across content-based checkers | `SKILL.md` (≥ 200 words, valid frontmatter), `README.md` (≥ 200 words, ≥ 2 H2), `LICENSE`, `SECURITY.md`, `evidence/sbom.json` (valid CycloneDX), `cnsb/skill.cnsb.json` with full lifecycle hooks |
| `missing-security/` | should drop Security score | `SKILL.md` valid; **no** `LICENSE`, **no** `SECURITY.md`, secret string `AWS_SECRET_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE` in `examples/leak.sh` |
| `malformed-frontmatter/` | should fail Identity & Manifest | `SKILL.md` with broken YAML (unterminated quote) |
| `cnsb-no-lifecycle/` | should fail Lifecycle | `cnsb/skill.cnsb.json` missing `lifecycle` block |

**Tasks:**

- [ ] **C0.6.1** Create the four fixture directories with content-realistic files (no Lorem ipsum — write actual short README/SKILL bodies that read as a real skill so checkers can't game them).
- [ ] **C0.6.2** Write integration test `crates/skillpack-adapters/tests/spine_integration.rs` that asserts each fixture produces a parseable assessment + envelope (full grading happens in later phases):

  ```rust
  #[tokio::test]
  async fn perfect_a_fixture_emits_signed_envelope() {
      let report = assess_fixture("perfect-a").await.unwrap();
      assert!(report.envelope.is_some());
      let env = report.envelope.unwrap();
      assert_eq!(env.statement.r#type, "SkillAssessment");
      assert!(!env.signatures.is_empty());
  }
  ```

- [ ] **C0.6.3** Run: `cargo test -p skillpack-adapters spine_integration`. Expected: FAIL initially (no envelope wiring yet — that's the integration step).
- [ ] **C0.6.4** Commit fixtures only (test stays red until integration):

  ```bash
  git add tests/fixtures/skillpack crates/skillpack-adapters/tests/spine_integration.rs
  git commit -m "test(fixtures): add 4 SkillPack fixture corpora (Phase 0.6)"
  ```

---

## Integration Step — Wire Spine Into `assess_skill`

**Depends on:** C0.0–C0.6 all merged.

**Files:** `crates/skillpack-application/src/assess_skill.rs` (modify); `crates/skillpack-adapters/tests/spine_integration.rs` (test should now pass).

**Tasks:**

- [ ] **I.1** In `assess_skill.rs::assess(...)` final step: build envelope via `EnvelopeBuilder::new(skill_urn).with_assessment(json!({ "overall": report.overall_score, "grade": report.grade, "stub_dimensions": report.meta.stub_dimensions }))`. Inject a `Signer` via constructor (DI); call `signer.sign_envelope(&mut env).await?`. Attach to `report.envelope`.
- [ ] **I.2** Run: `cargo test -p skillpack-adapters spine_integration --all`. Expected: PASS.
- [ ] **I.3** Run: `cargo build --workspace --all-targets`. Expected: PASS, zero warnings (clippy pedantic acceptable on legacy code).
- [ ] **I.4** Manual smoke: `cargo run -p skillpack-adapters -- assess tests/fixtures/skillpack/perfect-a --emit-envelope > /tmp/env.json && jq . /tmp/env.json` shows valid signed envelope.
- [ ] **I.5** Commit:

  ```bash
  git add crates/skillpack-application/src/assess_skill.rs
  git commit -m "feat: wire EnvelopeBuilder + Signer into assess_skill (Phase 0 integration)"
  ```

---

## Dispatch Sequence (parallel Sonnet sub-agents)

```
Round 1 (sequential):  C0.0 — DimensionId reconciliation [BLOCKING]
                       │
                       ▼
Round 2 (parallel):    C0.1  C0.3  C0.4  C0.5  C0.6     [5 agents fan out]
                       │     │     │     │     │
                       ▼     ▼     ▼     ▼     ▼
Round 3:               C0.2 (depends on C0.1)            [1 agent]
                       │
                       ▼
Round 4 (sequential):  Integration + smoke verification  [Opus or 1 agent]
```

Wall-clock target: 4 sequential rounds. Round 2 collapses 5 components into one round.

Per memory feedback: dispatch each component as a fresh Sonnet sub-agent with `model: "sonnet"`, `run_in_background: true` for Round 2 fan-out. Reserve Opus for orchestration + Round 4 integration verification.

---

## Self-Review Checklist (executed by Opus before dispatch)

- [x] **Spec coverage:** Every locked dimension obligation in `project_skillpack_finish_line_scope.md` v1-A and v1-B has a touchpoint in this plan or is explicitly deferred to Phases 1–9 (dimension content checks).
- [x] **No placeholders:** No "TBD", "implement appropriately", "etc." — every code block is concrete.
- [x] **Type consistency:** `DimensionId` variants used in C0.0 match those referenced in C0.4 and the integration step. `EvidenceEnvelope` shape matches schema. `Signer` trait signature consistent across C0.1/C0.2/I.1.
- [x] **File paths exact:** Every file path is fully qualified from repo root.
- [x] **Commits per task:** Each component ends with a single commit; integration adds one more. Total Phase 0 commits: 8.

---

## Out of Scope for Phase 0 (handled in later phases)

- Content-based dimension checkers (Phases 1–9 per D3 ordering)
- Sigstore keyless / Fulcio integration (deferred — key-file mode only in v1)
- Rekor anchoring (v2)
- BPL chain on promotion (v2)
- VS Code extension authoring features (v2)
- VWP §26 enforcement on graded artifacts (v2 — grader still self-applies VWP on its own work)
- DAL ↔ GAL vocab unification (v2 — D7 keeps schema GAL 0–5)
