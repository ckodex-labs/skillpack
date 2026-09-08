# SkillPack — Deep Codebase Architecture Map

> For AI agents and human developers navigating the SkillPack monorepo.
> Last updated: 2026-05-12

## 1. Project Overview

SkillPack is a **quality assessment & packaging toolchain for AI agent skills**, built around a domain-driven, hexagonal (ports-and-adapters) Rust core. It provides:

- **CLI** (`skillpack`) — assess, grade, package, publish, install skills
- **VSCode Extension** — IDE-integrated authoring, validation, and publishing
- **gRPC API** (`skillpack-server`) — programmatic assessment, ratings, indices, search
- **Dashboard** (Next.js) — web UI for skill quality metrics
- **MCP Server** — exposes skills as Model Context Protocol resources

The repo is a **Cargo workspace** with 5 crates plus TypeScript extensions and docs.

---

## 2. Architecture Pattern: Hexagonal (Ports & Adapters)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                           │
│   CLI  │  VSCode Ext  │  gRPC API  │  Dashboard  │  MCP Server      │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                       APPLICATION LAYER                             │
│   AssessSkillUseCase  │  GradeSkillUseCase  │  IndexOperations ...  │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                         DOMAIN LAYER (Kernel)                       │
│   Assessment  │  Dimension  │  Grade  │  Score  │  Envelope ...     │
│   <--- Ports: SkillReader, DimensionChecker, Signer, ... --->       │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                        ADAPTERS LAYER                               │
│   FilesystemReader  │  OciReader  │  GitReader  │  S3Reader ...     │
│   Checkers (9 dims) │  Reporters  │  Evidence Builders              │
│   OCI Publish  │  Sigstore Sign  │  DuckDB Persistence              │
└─────────────────────────────────────────────────────────────────────┘
```

**Dependency rule**: Domain has zero external deps. Application depends only on Domain. Adapters implement Domain ports and are wired by the presentation layer.

---

## 3. Crate Map

| Crate                   | Path                           | Responsibility                                               | Key Modules                                                                                        |
| ----------------------- | ------------------------------ | ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- |
| `skillpack-domain`      | `crates/skillpack-domain`      | **Pure domain model**. No infrastructure deps.               | `assessment`, `dimension`, `grade`, `score`, `envelope`, `ports`, `index`, `ratings`               |
| `skillpack-application` | `crates/skillpack-application` | **Use cases / orchestration**. Bridges domain + adapters.    | `assess_skill`, `grade_skill`, `generate_report`, `envelope_builder`, `index_operations`           |
| `skillpack-adapters`    | `crates/skillpack-adapters`    | **Infrastructure**. Implements all ports. Houses CLI binary. | `cli`, `checkers`, `filesystem`, `oci`, `git`, `evidence`, `persistence`, `registry_client`, `mcp` |
| `skillpack-api`         | `crates/skillpack-api`         | **gRPC presentation layer**. Protobuf-generated types.       | `server`, `conversions`, `index_service`, `query_service`, `ratings_service`                       |
| `skillpack-tests`       | `crates/skillpack-tests`       | **Integration test binaries**.                               | `schema_round_trip`, `fixture_grade_corpus`                                                        |

### 3.1 Binary Names

| Binary                 | Source                                            | Purpose                                     |
| ---------------------- | ------------------------------------------------- | ------------------------------------------- |
| `skillpack`            | `skillpack-adapters/src/cli/main.rs`              | CLI entry point — delegates to `cli::run()` |
| `skillpack-server`     | `skillpack-api/src/bin/server.rs`                 | gRPC server on `[::1]:50051`                |
| `schema_round_trip`    | `skillpack-tests/src/bin/schema_round_trip.rs`    | Test binary for schema validation           |
| `fixture_grade_corpus` | `skillpack-tests/src/bin/fixture_grade_corpus.rs` | Test binary for fixture grading             |

---

## 4. Domain Model (Kernel Space)

### 4.1 Core Types

Defined in `crates/skillpack-domain/src/`:

```rust
// assessment.rs
pub struct Assessment {
    pub id: AssessmentId,           // UUID
    pub skill: SkillIdentity,     // name, version, path
    pub dimension_scores: HashMap<DimensionId, Score>,
    pub bonus_points: BonusPoints,  // SLSA, Sigstore, Dagger, STRIDE, MCP
    pub issues: Vec<Issue>,
    pub assessed_at: DateTime<Utc>,
    pub stub_count: u32,          // how many checkers are stubs
}

pub struct SkillIdentity {
    pub name: String,
    pub version: String,
    pub path: String,
}

pub struct Issue {
    pub dimension: DimensionId,
    pub severity: Severity,       // Error | Warning | Note
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}
```

### 4.2 Quality Dimensions (9)

| Enum Variant          | Checker Module  | File                        |
| --------------------- | --------------- | --------------------------- |
| `IdentityAndManifest` | `identity`      | `checkers/identity.rs`      |
| `Security`            | `security`      | `checkers/security.rs`      |
| `Provenance`          | `provenance`    | `checkers/provenance.rs`    |
| `Documentation`       | `documentation` | `checkers/documentation.rs` |
| `Testing`             | `testing`       | `checkers/testing.rs`       |
| `Compatibility`       | `compatibility` | `checkers/compatibility.rs` |
| `Lifecycle`           | `lifecycle`     | `checkers/lifecycle.rs`     |
| `Governance`          | `governance`    | `checkers/governance.rs`    |
| `EvalsHitl`           | `evals_hitl`    | `checkers/evals_hitl.rs`    |

All checkers implement the `DimensionChecker` port:

```rust
pub trait DimensionChecker: Send + Sync {
    fn dimension(&self) -> DimensionId;
    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>);
    fn is_stub(&self) -> bool;  // false = real content analysis
}
```

### 4.3 Grade System

| Grade | Score Range | Description                     |
| ----- | ----------- | ------------------------------- |
| S     | 120+        | Exceptional (with bonus points) |
| A     | 100-119     | Excellent                       |
| B     | 80-99       | Good                            |
| C     | 60-79       | Acceptable                      |
| D     | 40-59       | Poor                            |
| F     | <40         | Unacceptable                    |

### 4.4 Ports (Interfaces for Adapters)

In `crates/skillpack-domain/src/ports.rs`:

- `SkillReader` — read skill identity, files, existence checks
- `DimensionChecker` — run dimension-specific quality checks
- `AssessmentRepository` — persist/load assessments
- `ReportGenerator` — generate reports (JSON, SARIF, Markdown)
- `BundleReader` — read CNSB manifest bundles
- `Signer` — sign evidence envelopes (Sigstore keyless)
- `EvidenceSink` — write evidence to disk
- `LockEmitter` — emit `skill.lock` files

---

## 5. CLI Commands

All commands live in `crates/skillpack-adapters/src/cli/mod.rs`. Entry point is `skillpack_adapters::cli::run()`.

| Command           | Purpose                                                      | Key Function   |
| ----------------- | ------------------------------------------------------------ | -------------- |
| `check [PATH]`    | Full assessment with optional `--min-score`                  | `run_check`    |
| `grade [PATH]`    | Show letter grade with `--minimum` threshold                 | `run_grade`    |
| `report [PATH]`   | Generate report (JSON/SARIF/Markdown)                        | `run_report`   |
| `validate FILE`   | Validate JSON against schema (CNSB, CNAAB, Evidence, Policy) | `run_validate` |
| `init NAME`       | Scaffold basic/full/mcp-server skill                         | `run_init`     |
| `wizard NAME`     | Interactive A/B-grade skill scaffolding                      | `run_wizard`   |
| `lock [PATH]`     | Generate `skill.lock` with content hashes                    | `run_lock`     |
| `migrate [PATH]`  | Migrate manifest to latest schema version                    | `run_migrate`  |
| `package [PATH]`  | Create distributable `.tar.gz` archive                       | `run_package`  |
| `publish [PATH]`  | Push to OCI registry with optional `--sign`                  | `run_publish`  |
| `install REF`     | Install skill from registry or GitHub                        | `run_install`  |
| `discover [PATH]` | Find skill manifests in directory                            | `run_discover` |
| `eval [PATH]`     | Run evaluation suites (smoke, compliance)                    | `run_eval`     |

### 5.1 CLI Code Structure

```
crates/skillpack-adapters/src/cli/
├── mod.rs          # All command handlers + Cli/Commands structs + tests
├── dry_run.rs      # DryRun helper for --dry-run / --preview
└── (commands inline in mod.rs)
```

---

## 6. VSCode Extension

TypeScript extension in `vscode-extension/`. Activated by workspace containing `SKILL.md`, `skill.cnsb.json`, or `eval.yml`.

### 6.1 Commands (25+)

All registered in `src/extension.ts`, implemented in `src/commands/*.ts`:

| Category       | Commands                                                                                       |
| -------------- | ---------------------------------------------------------------------------------------------- |
| **Assessment** | `assess`, `validate`, `showReport`, `eval`                                                     |
| **Authoring**  | `newSkill`, `init`, `askAI`, `improveSkill`, `generateCapacity`, `addExample`, `generateTests` |
| **Registry**   | `discoverSkills`, `searchRegistry`, `installSkill`, `investigateSkill`, `refreshRegistry`      |
| **OCI**        | `ociLogin`, `ociLogout`, `ociListRegistries`                                                   |
| **Lifecycle**  | `package`, `publish`, `applyFix`                                                               |
| **Docs**       | `openDocs`                                                                                     |

### 6.2 Providers

| Provider                      | File                       | Purpose                           |
| ----------------------------- | -------------------------- | --------------------------------- |
| `SkillPackTreeProvider`       | `providers/treeView.ts`    | Tree view of skill structure      |
| `RegistryTreeProvider`        | `providers/treeView.ts`    | Registry browser                  |
| `OciRegistryTreeProvider`     | `providers/treeView.ts`    | OCI registry view                 |
| `SkillPackCodeActionProvider` | `providers/codeActions.ts` | Quick fixes in Markdown/JSON/YAML |
| `SkillCompletionProvider`     | `providers/completion.ts`  | Auto-completion for skill files   |
| `SkillHoverProvider`          | `providers/hover.ts`       | Hover info                        |

### 6.3 Utilities

| Module                 | Key Exports                                            |
| ---------------------- | ------------------------------------------------------ |
| `utils/state.ts`       | Global extension state (diagnostics, status bar)       |
| `utils/diagnostics.ts` | `assessSkill()`, `updateWorkspaceContext()`            |
| `utils/ociAuth.ts`     | OCI registry authentication with VSCode secret storage |
| `utils/api.ts`         | gRPC client wrapper                                    |
| `utils/html.ts`        | Report HTML generation                                 |

---

## 7. gRPC API

All proto definitions are consolidated in `proto/skillpack/v1/`:

| Proto File        | Services                                                             |
| ----------------- | -------------------------------------------------------------------- |
| `skillpack.proto` | `SkillPackService`, `IndexService`, `RatingsService`, `QueryService` |
| `canonical.proto` | `CanonicalStoreService`                                              |

Generated Rust code included via `tonic::include_proto!("skillpack.v1")` in `crates/skillpack-proto/src/lib.rs`.

Server binary wires all services together with a shared `DuckDbRepository`.

---

## 8. Dashboard (Next.js)

Location: `dashboard/`. Minimal Next.js 15 + React 19 app with Recharts.

| File                 | Purpose                             |
| -------------------- | ----------------------------------- |
| `src/app/layout.tsx` | Root layout with metadata           |
| `src/app/page.tsx`   | Main dashboard page                 |
| `src/lib/client.ts`  | Connect-RPC web client for gRPC API |
| `package.json`       | `skillpack-dashboard`               |

---

## 9. Key Entry Points for AI Agents

### 9.1 "I need to add/modify a CLI command"

1. Edit `crates/skillpack-adapters/src/cli/mod.rs`
2. Add variant to `Commands` enum (~line 28)
3. Add handler function (e.g., `async fn run_mycommand(...)`)
4. Wire in `cli::run()` match statement
5. Add test in the `#[cfg(test)]` module at bottom of file

### 9.2 "I need to add a quality dimension"

1. Add `DimensionId` variant in `crates/skillpack-domain/src/dimension.rs`
2. Add weight in `standard_dimensions()`
3. Create checker in `crates/skillpack-adapters/src/checkers/my_dimension.rs`
4. Implement `DimensionChecker` trait
5. Register in `crates/skillpack-adapters/src/checkers/mod.rs::all_checkers()`
6. Update `dimension_parity.sh` validation

### 9.3 "I need to add a new report format"

1. Add variant to `ReportOutputFormat` in CLI
2. Implement `ReportGenerator` trait in `crates/skillpack-adapters/src/reporters/`
3. Register reporter in CLI `run_report()` match

### 9.4 "I need to change the skill manifest schema"

1. Edit `schemas/cnsb.schema.json` (single source of truth)
2. Mirror to `vscode-extension/schemas/cnsb.schema.json`
3. Update `crates/skillpack-domain/src/schema_validation.rs`
4. Update any wizard-generated manifest content in `cli/mod.rs`
5. Run `cargo test` — schema validation tests will catch drift

### 9.5 "I need to add a VSCode command"

1. Implement handler in `vscode-extension/src/commands/myCommand.ts`
2. Export from `vscode-extension/src/commands/index.ts`
3. Register in `vscode-extension/src/extension.ts` `activate()`
4. Add to `package.json` `contributes.commands` array

### 9.6 "I need to add a gRPC method"

1. Edit `proto/skillpack/v1/skillpack.proto` (or appropriate proto file)
2. Run `cargo build -p skillpack-api` to regenerate tonic code
3. Implement method in `crates/skillpack-api/src/server.rs`
4. Update `docs-site/docs/guide/grpc-api.md`

---

## 10. File Navigator by Task

### Assessment Pipeline
```
cli/mod.rs::run_check()
  └─> AssessSkillUseCase::new(reader, all_checkers())
       └─> skillpack-application/src/assess_skill.rs
            └─> for each checker: DimensionChecker::check(reader, path)
                 └─> checkers/{identity,security,provenance,...}.rs
```

### Packaging Pipeline
```
cli/mod.rs::run_package()
  └─> run_check() [if not --no-check]
       └─> tar.gz creation with SKILL.md + manifest + evidence + lock
```

### Publishing Pipeline
```
cli/mod.rs::run_publish()
  └─> run_check() [if not --no-check]
       └─> OCI push via oci-client
            └─> optionally: sign_with_sigstore() [SIGSTORE_ID_TOKEN]
```

### Evidence Signing
```
evidence/builder.rs::EvidenceBuilder
  └─> signer.rs::CosignSigner::sign_keyless()
       └─> sigstore crate (Fulcio + Rekor)
```

### Persistence
```
persistence/mod.rs::DuckDbRepository
  └─> DuckDB embedded database for assessments, indices, ratings
```

---

## 11. Testing Strategy

| Layer       | Test Type          | Location                                                                                          |
| ----------- | ------------------ | ------------------------------------------------------------------------------------------------- |
| Domain      | Unit tests         | `crates/skillpack-domain/tests/` + inline `#[cfg(test)]`                                          |
| Adapters    | Unit + integration | `crates/skillpack-adapters/src/cli/mod.rs` (tests at bottom) + `crates/skillpack-adapters/tests/` |
| Application | Doc tests          | `crates/skillpack-application/src/`                                                               |
| Integration | Test binaries      | `crates/skillpack-tests/src/bin/`                                                                 |
| Fixtures    | Corpus tests       | `tests/fixtures/skills/`                                                                          |
| Validation  | Shell scripts      | `validation/dimension_parity.sh`, `validation/no_stubs.sh`                                        |
| End-to-end  | Manual             | `skillpack wizard` → `skillpack eval --suite smoke`                                               |

### Key Test Fixtures

- `tests/fixtures/skills/known-a/` — A-grade reference skill
- `tests/fixtures/skills/known-c/` — C-grade skill
- `tests/fixtures/skills/invalid/` — Invalid skill (schema errors)
- `tests/fixtures/skills/unsigned/` — Missing provenance

---

## 12. Extension Points

### Adding a New Source Reader

Implement `SkillReader` trait from `domain::ports`, then register in `source_factory.rs`.

### Adding a New Registry Backend

Implement `RemoteRegistry` trait, register in `registry_client.rs`.

### Adding a New Evidence Format

Implement `EvidenceSink` trait, use in `evidence/builder.rs`.

### Adding a New OCI Media Type

Add to `crates/skillpack-adapters/src/oci.rs`:

```rust
pub const CONFIG: &str = "application/vnd.ckodex.skillpack.config.v1+json";
```

---

## 13. Environment & Config

| Variable              | Purpose                                       |
| --------------------- | --------------------------------------------- |
| `SIGSTORE_ID_TOKEN`   | OIDC token for keyless signing via Fulcio     |
| `SKILLPACK_MIN_GRADE` | Default minimum grade threshold               |
| `SKILLPACK_FORMAT`    | Default output format (json, sarif, markdown) |
| `SKILLPACK_CONFIG`    | Path to `.skillpack.json` config file         |

### VSCode Secret Keys

Stored in VSCode `SecretStorage`:
- `skillpack-oci:{registry}` — registry credentials

---

## 14. Quick Commands for Agents

```bash
# Check everything compiles
cargo check --workspace

# Run all tests
cargo test --workspace

# Run clippy (zero tolerance)
cargo clippy --workspace -- -D warnings

# Build CLI binary
cargo build --release -p skillpack-adapters

# Build gRPC server
cargo build --release -p skillpack-api

# End-to-end smoke test
cargo run --bin skillpack -- wizard test-skill --no-check
cd test-skill && cargo run --bin skillpack -- eval --suite smoke

# Schema validation test binary
cargo run --bin schema_round_trip

# Fixture grading corpus
cargo run --bin fixture_grade_corpus

# Check for remaining old brand names
grep -ri "skilliq" --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git .
grep -ri "skillqa" --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git .
```

---

## 15. Directory Tree (Simplified)

```
ckodex-skilliq/
├── Cargo.toml                    # Workspace manifest
├── README.md
├── ARCHITECTURE.md               # This file
│
├── crates/
│   ├── skillpack-domain/         # Pure domain (no deps)
│   │   ├── src/
│   │   │   ├── assessment.rs     # Assessment aggregate root
│   │   │   ├── dimension.rs      # 9 quality dimensions
│   │   │   ├── grade.rs          # Grade enum + scoring
│   │   │   ├── ports.rs          # Hexagonal ports (traits)
│   │   │   ├── envelope.rs       # Evidence envelope
│   │   │   ├── index.rs          # SkillsIndex (S&P 500 style)
│   │   │   └── ...
│   │   └── tests/
│   │
│   ├── skillpack-application/    # Use cases
│   │   └── src/
│   │       ├── assess_skill.rs   # Main assessment orchestrator
│   │       ├── grade_skill.rs    # Grade calculation
│   │       ├── envelope_builder.rs # Evidence envelope builder
│   │       └── index_operations.rs
│   │
│   ├── skillpack-adapters/       # Infrastructure + CLI binary
│   │   ├── Cargo.toml            # [[bin]] name = "skillpack"
│   │   ├── src/
│   │   │   ├── cli/
│   │   │   │   ├── main.rs       # Binary entry point
│   │   │   │   ├── mod.rs        # All commands + tests
│   │   │   │   └── dry_run.rs
│   │   │   ├── checkers/         # 9 dimension checkers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── identity.rs
│   │   │   │   ├── security.rs
│   │   │   │   └── ...
│   │   │   ├── evidence/         # Signing, building, sinking
│   │   │   │   ├── builder.rs
│   │   │   │   ├── signer.rs     # Sigstore keyless signing
│   │   │   │   └── sink.rs
│   │   │   ├── filesystem.rs     # FilesystemReader (SkillReader impl)
│   │   │   ├── oci.rs            # OCI registry push/pull
│   │   │   ├── registry_client.rs # Remote registry client
│   │   │   ├── persistence/      # DuckDB repository
│   │   │   ├── mcp.rs            # MCP server implementation
│   │   │   └── ...
│   │   └── tests/                # Integration tests
│   │
│   ├── skillpack-api/            # gRPC API + server binary
│   │   ├── Cargo.toml            # [[bin]] name = "skillpack-server"
│   │   ├── build.rs              # tonic-prost-build proto compilation
│   │   └── src/
│   │       ├── bin/server.rs     # Server binary
│   │       ├── server.rs         # SkillPackServer (gRPC impl)
│   │       ├── conversions.rs    # Domain <-> Proto conversions
│   │       └── ...
│   │
│   └── skillpack-tests/          # Integration test binaries
│       └── src/bin/
│           ├── schema_round_trip.rs
│           └── fixture_grade_corpus.rs
│
├── proto/                        # Single source of truth for all protobuf
│   ├── skillpack/v1/
│   │   ├── skillpack.proto       # All services (SkillPack, Index, Ratings, Query)
│   │   └── canonical.proto     # CanonicalStore service
│   ├── macos/v1/
│   │   └── skills.proto          # macOS-specific SkillsService
│   └── ckodex/stx/v1/
│       └── stx.proto             # Skills Transparency Exchange
│
├── vscode-extension/             # VSCode extension (TypeScript)
│   ├── package.json              # Extension manifest
│   └── src/
│       ├── extension.ts          # Activation + command registration
│       ├── commands/             # 20+ command handlers
│       ├── providers/            # Tree views, code actions, completion
│       └── utils/                # State, diagnostics, OCI auth, API client
│
├── dashboard/                    # Next.js web dashboard
│   └── src/
│       ├── app/
│       │   ├── layout.tsx
│       │   └── page.tsx
│       └── lib/client.ts         # Connect-RPC web client
│
├── docs-site/                    # Rspress documentation site
├── schemas/                      # JSON Schema definitions
│   ├── cnsb.schema.json          # Skill manifest schema
│   ├── skill-lock.schema.json
│   └── ...
│
├── tests/
│   └── fixtures/
│       └── skills/               # Reference skills for testing
│           ├── known-a/          # A-grade reference
│           ├── known-c/          # C-grade reference
│           └── ...
│
├── validation/                   # CI validation scripts
│   ├── dimension_parity.sh
│   └── no_stubs.sh
│
├── security/                     # Security docs & threat model
├── docs/                         # Architecture & design docs
└── examples/                     # Example skill files
```

---

## 16. Important Rules for AI Agents

1. **Never add infrastructure deps to `skillpack-domain`**. It must remain pure.
2. **Always update tests when changing CLI output strings**. Tests assert on literal output.
3. **Always mirror schema changes** to `vscode-extension/schemas/`.
4. **Run `cargo clippy --workspace -- -D warnings`** before finishing. Zero warnings tolerance.
5. **Wizard-generated files** are defined as string literals in `cli/mod.rs`. Search for `generate_wizard_*` functions.
6. **The `all_checkers()` function** is the single source of truth for which dimensions are assessed. It must cover all `DimensionId::all()` variants.
7. **Proto changes require rebuilding** `skillpack-api`. Run `cargo build -p skillpack-api` to regenerate tonic code.
8. **End-to-end validation**: after CLI changes, run `skillpack wizard` → `skillpack eval --suite smoke` to verify.
