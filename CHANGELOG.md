# Changelog

All notable changes to SkillPack will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-beta.3] - 2026-09-19

### Security
- Shared skill-path guard (`skillpack_application::skill_path_guard`) applied at
  gRPC (`server.rs` incl. previously unsanitized `report`, `assess_stream`,
  `assess_batch_stream`), HTTP (`http_server.rs`), MCP (`mcp.rs`), and canonical
  joins (`canonical_service.rs`: package/import/promote). Rejects `..`, `..\\`,
  absolute escapes, symlink escapes, NUL, and separator-carrying components;
  13 unit tests.
- Fail-closed auth: the server now refuses to start when `SKILLPACK_BIND_ALL=1`
  is set without `SKILLPACK_API_TOKEN` (previously logged CRITICAL and started
  unauthenticated anyway).
- `cargo audit` cleared: h2 (RUSTSEC-2026-0258), quinn-proto
  (RUSTSEC-2026-0185), rustls (RUSTSEC-2026-0285), rkyv (RUSTSEC-2026-0235),
  webbrowser (RUSTSEC-2026-0257), crossbeam-epoch (RUSTSEC-2026-0204), anyhow
  (RUSTSEC-2026-0190) via dependency refresh. `rsa` RUSTSEC-2023-0071
  (Marvin, via sigstore→openidconnect) has no upstream fix; documented
  acceptance in `.cargo/audit.toml` — usage is signature verification, not
  the vulnerable decryption path.

### Added
- Automated release-tag workflow `.github/workflows/release-tag.yml`: runs the
  `xtask` stamp (version-triple) check and the full CI-parity gate, then creates
  annotated tag `v<version>` on main pushes when the tag is new.
- SVG grade badge output: `skillpack report --format badge` (and MCP
  `generate_report` format `"badge"`) renders a shields.io-style flat grade
  badge for README embedding — deterministic bytes, XML-escaped skill names,
  0–150 scale, 4 unit tests.
- `SHIP-PLAN.md`: audit-driven upgrade-and-ship plan with evidence labels.

### Changed
- Toolchain 1.95.0 → 1.98.1; MSRV `rust-version` 1.95 → 1.98. CI and
  release-tag workflows pinned to `dtolnay/rust-toolchain@1.98.1`.
- Dependency refresh: ~236 semver-compatible bumps plus majors —
  `jsonschema` 0.46 → 0.56 (`validator_for` constructor), `brotli` 8 → 9,
  `serial_test` 3 → 4.
- Dockerfile builder `rust:1.85-bookworm` → `rust:1.98-bookworm` (was below
  MSRV; container build would have failed).
- Pinned floating inputs: `zot` image `:latest` → `v2.1.21`,
  docs-site `rspress` `latest` → `^1.47.1`, CI `json-schema-to-typescript`
  → `@15`.

### Fixed
- Repository hygiene: ~2,528 build/tool artifacts untracked and purged from
  history (`dashboard/.next`, `.next-stale-bak`, `.playwright-mcp` logs, 39
  stray root screenshots); real `.gitignore` written. Tracked file count
  3,194 → 728.
- `macos/skills-ecosystem` embedded repo (unregistered gitlink, broken for
  clones) absorbed into the parent tree.
- `.dagger` crate version re-synced to the workspace line.
- New clippy `question_mark` finding on 1.98.1 resolved in
  `cli/improve.rs`.

## [1.0.0-beta.2] - 2026-09-08

### Fixed
- CI clippy parity: `AssessEvent` oneof variants boxed via
  `tonic_prost_build::Config::boxed(".skillpack.v1.AssessEvent.event")`;
  resolves the `large_enum_variant` error that made
  `cargo clippy --workspace -- -D warnings` fail on generated code.
- `skillpack-api` is clippy-clean on a fresh `cargo clean -p` rebuild
  (validated in-session).
- `cargo fmt --all` applied; the fmt gate is green workspace-wide.
- Flaky `http_server` tests fixed at the root:
  `TokenValidator::with_token` / `TokenValidator::none` replace
  `std::env::set_var` mutation of `SKILLPACK_API_TOKEN` in tests
  (test-order-dependent 401-instead-of-201 flakes eliminated).
- `KNOWN-ISSUES.md` refreshed: removed stale CORS/rate-limit and OTLP
  entries (both are wired); `.dagger/pipeline.rs` header no longer
  describes the sign/attest stages as stubs.

### Changed
- MSRV raised 1.85 -> 1.95: `fanring 0.3.2` declares
  `rust-version = "1.93"`, and the CI-parity gate pins the toolchain.
- New `crates/xtask` (unpublished): CI-parity release gate with stage
  fanout over a `fanring` ring channel, plus `stamp` and `provenance`
  machine checks for the version/provenance consistency chain.
- `publish/release-manifest.json` version and the provenance
  references re-stamped to 1.0.0-beta.2.

## [1.0.0-beta.1] - 2026-06-05

### Security
- Bearer token enforcement on all mutating HTTP routes and gRPC methods when `SKILLPACK_API_TOKEN` is unset.
- DuckDB multi-tenant isolation with `tenant_id` column filtering on all repository queries.

### Changed
- Systemic elimination of server-side `.unwrap()` and `.expect()` panics in HTTP/gRPC entrypoints and DuckDB persistence, replaced with safe `?` error propagation.
- Dagger pipeline `sign` and `attest` stages now execute real `cosign sign-blob`, SBOM generation, and SLSA provenance instead of stubs.
- MCP `generate_report` tool fully implemented with JSON, Markdown, and SARIF output formats.

## [1.0.0-alpha.1] - 2026-06-04

### Added
- Multi-dimension skill quality assessment across 9 dimensions (Identity, Security, Provenance, Documentation, Testing, Compatibility, Lifecycle, Governance, Evals & HITL).
- CLI (`skillpack`) with commands: check, grade, report, init, package, publish, install, eval, discover, lock, migrate, store sync/migrate/status.
- gRPC server (`skillpack-server`) with streaming assessment, index, ratings, query, and canonical-store services.
- HTTP server with SSE events, REST proxies, and health checks.
- MCP server over stdio JSON-RPC exposing `assess_skill`, `grade_skill`, `list_dimensions` tools.
- VS Code extension for inline diagnostics and skill authoring.
- Next.js dashboard for skill quality metrics.
- macOS Swift `skills-ecosystem` with SkillsCLI, SkillsDaemon, SkillsCore, and SkillsFinder.
- OCI distribution support for push/pull via ORAS-compatible registries.
- Supply-chain dependencies: Sigstore, CycloneDX SBOM, SLSA provenance, in-toto attestations.
- Secret scanner with AWS/GitHub regexes and Shannon-entropy detection.
- Canonical store with DuckDB persistence and filesystem watcher.
- Agentic skill template with catalog, orchestrator agent, and test gates.

### Known Limitations
- See `KNOWN-ISSUES.md` for the full list of alpha limitations.

[1.0.0-beta.3]: https://github.com/ckodex-labs/skillpack/releases/tag/v1.0.0-beta.3
[1.0.0-beta.2]: https://github.com/ckodex-labs/skillpack/releases/tag/v1.0.0-beta.2
[1.0.0-beta.1]: https://github.com/ckodex-labs/skillpack/releases/tag/v1.0.0-beta.1
[1.0.0-alpha.1]: https://github.com/ckodex-labs/skillpack/releases/tag/v1.0.0-alpha.1
