# Known Issues — SkillPack Beta

> This document tracks acknowledged limitations and gaps for the `1.0.0-beta.1` release.
> Items are not blockers for a trusted, local/private alpha audience unless explicitly marked **[CRITICAL]**.

## 1. Security

- **Default-open auth:** The HTTP/gRPC server disables bearer-token authentication when `SKILLPACK_API_TOKEN` is unset. This is convenient for local development but means mutation/migration endpoints are unprotected if the server is exposed to a network.
  - **Mitigation:** Always set `SKILLPACK_API_TOKEN` before exposing the server. The server now defaults to binding on `127.0.0.1` only.
  - **Status:** Partially mitigated; requires user vigilance.

 User-provided `skill_path` arguments in gRPC, HTTP, and MCP handlers are passed directly to the filesystem reader. Path-traversal hardening is claimed in the threat model but not visibly enforced at entry points.
  - **Mitigation:** Only run against trusted skill packs on local filesystem.
  - **Status:** Planned for post-alpha.

## 2. CI / CD / Release

- **Dagger pipeline sign/attest stages:** `.dagger/pipeline.rs` executes real `cosign sign-blob`, SBOM generation, and SLSA provenance generation; stale alpha-stub header removed. Provenance placeholder digests are replaced by the `xtask` gate (see Resolved section).
  - **Status:** Resolved in 1.0.0-beta.2.

- **No changelog or release tagging workflow:** Prior to this release, no `CHANGELOG.md` or automated release tagging existed.
  - **Status:** Added in this release; automated release workflow planned for post-alpha.

## 3. Observability

- **OTLP export:** resolved in 1.0.0-beta.2 — see the Resolved section at the end of this file.

## 4. Documentation

- **README crate list mismatch:** The README lists crates (`skillpack-cli`, `skillpack-vscode`, `skillpack-docs`) that are not workspace members.
  - **Status:** Fixed in this release.

- **MCP `generate_report` is stubbed:** Returns a misleading "Report generated successfully" message with no actual content.
  - **Status:** Fixed in this release (returns explicit not-yet-implemented error).

## 5. Testing

- **Coverage gaps:** No tests for HTTP server handlers, OCI publish/install, DuckDB persistence, or MCP JSON-RPC handlers.
  - **Status:** Planned for post-alpha.

- **CI greenness unverified:** The `dtolnay/rust-action@stable` action name was incorrect and the `schema-sync` job invoked macOS/Swift tooling on Linux runners.
  - **Status:** Fixed in this release.

## 6. Container

- **Non-distroless base image:** The runtime stage uses `debian:bookworm-slim` instead of a distroless image.
  - **Impact:** Larger attack surface than necessary.
  - **Status:** Acceptable for alpha; planned for post-alpha.

## 7. Resolved in 1.0.0-beta.2

Closed items, retained for the audit trail.

- **CORS and request limits:** HTTP layer applies deny-by-default CORS (`SKILLPACK_CORS_ORIGINS`), `TimeoutLayer` (`SKILLPACK_REQUEST_TIMEOUT_SECS`), and `RequestBodyLimitLayer` (`SKILLPACK_MAX_BODY_SIZE_MB`). Rate limiting remains a reverse-proxy concern for non-local deployments.
- **OTLP export:** `init_telemetry` wires `opentelemetry-otlp` when `SKILLPACK_OTLP_ENDPOINT` or `--otlp-endpoint` is set; stdout-only fallback if the exporter cannot be built.
- **Dagger sign/attest stages:** real `cosign sign-blob`, CycloneDX SBOM, and SLSA provenance generation in `.dagger/pipeline.rs`; alpha-stub header comment deleted.
- **Stale-binary drift:** `skillpack` smoke reproducers now run inside the `xtask ci` gate off a fresh workspace build, so a cached binary can no longer make a passing claim against stale code.
