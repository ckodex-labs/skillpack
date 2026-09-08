# Changelog

All notable changes to SkillPack will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[1.0.0-beta.2]: https://github.com/ckodex/skillpack/releases/tag/v1.0.0-beta.2
[1.0.0-beta.1]: https://github.com/ckodex/skillpack/releases/tag/v1.0.0-beta.1
[1.0.0-alpha.1]: https://github.com/ckodex/skillpack/releases/tag/v1.0.0-alpha.1
