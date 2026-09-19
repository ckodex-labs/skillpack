# Known Issues — SkillPack Beta

> This document tracks acknowledged limitations and gaps closed out in the
> `1.0.0-beta.2` release. Items remaining open are marked **[POST-ALPHA]**. Closed items live in
> the **Resolved** sections with the evidence that closed them (claimed -> validated discipline).

## 1. Security

- **Default-open auth:** The HTTP/gRPC server disables bearer-token authentication when `SKILLPACK_API_TOKEN` is unset. This is convenient for local development but means mutation/migration endpoints are unprotected if the server is exposed to a network.
  - **Mitigation:** Fail-closed in 1.0.0-beta.3 — the server refuses to start when `SKILLPACK_BIND_ALL=1` is set without `SKILLPACK_API_TOKEN`. Loopback binding without a token still warns but starts (dev convenience).
  - **Status:** Resolved in 1.0.0-beta.3.

- **docs-site transitive advisories:** `rspress@1.47.2` (latest stable) pins `react-router-dom@6.30.6`, inside the `react-router 6.0.0–7.17.0` advisory range (GHSA-wrjc-x8rr-h8h6, GHSA-337j-9hxr-rhxg — open redirect, SSR hydration constructor injection). No fixed 6.x exists; rspress 2.x is beta-only. `prismjs` was cleared via an npm `overrides` pin to `^1.30.0`.
  - **Mitigation:** The docs site is statically generated at build time; these are build-chain/library advisories, not reachable runtime attack surface in the emitted HTML. Revisit when rspress 2.x ships stable.
  - **Status:** Accepted in 1.0.0-beta.3; tracked for post-beta.

- **Path-traversal hardening:** Enforced at every entry point (gRPC, HTTP, MCP, canonical joins).
  - **Evidence:** Shared `skill_path_guard` in `skillpack-application` (`validate_skill_path`, `validate_skill_path_cwd`, `validate_skill_component`); wired at `server.rs` (assess/grade/report/assess_stream/assess_batch_stream), `http_server.rs` (`assess_handler`), `mcp.rs` (3 tool call-sites), and `canonical_service.rs` (package_skill ns+skill_ref, import_skill target_name, promote_skill candidate_name). Rejects `..`, `..\\`, absolute escapes, symlink escapes (resolution-before-containment), NUL, and empty or separator-carrying components. 13 unit tests green plus workspace gate green.
  - **Status:** Resolved in 1.0.0-beta.2+

## 2. CI / CD / Release

- **Dagger pipeline sign/attest stages:** `.dagger/pipeline.rs` executes real `cosign sign-blob`, SBOM generation, and SLSA provenance generation; stale alpha-stub header removed. Provenance placeholder digests are replaced by the `xtask` gate (see Resolved section).
  - **Status:** Resolved in 1.0.0-beta.2.

- **No changelog or release tagging workflow:** Prior to this release, no `CHANGELOG.md` or automated release tagging existed.
  - **Evidence:** `.github/workflows/release-tag.yml` added — on push to `main` (or manual dispatch) it runs `cargo run -p xtask -- stamp` (version-triple consistency gate), the full `cargo run -p xtask -- ci` gate, then creates annotated tag `v<version>` iff absent; version extracted by the same grep pipeline validated locally against `1.0.0-beta.2`.
  - **Status:** Resolved in 1.0.0-beta.2+

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

## 7. Resolved in 1.0.0-beta.2+

Closed items, retained for the audit trail (each entry cites the evidence that closed it).

- **CORS and request limits:** HTTP layer applies deny-by-default CORS (`SKILLPACK_CORS_ORIGINS`), `TimeoutLayer` (`SKILLPACK_REQUEST_TIMEOUT_SECS`), and `RequestBodyLimitLayer` (`SKILLPACK_MAX_BODY_SIZE_MB`). Rate limiting remains a reverse-proxy concern for non-local deployments.
- **OTLP export:** `init_telemetry` wires `opentelemetry-otlp` when `SKILLPACK_OTLP_ENDPOINT` or `--otlp-endpoint` is set; stdout-only fallback if the exporter cannot be built.
- **Dagger sign/attest stages:** real `cosign sign-blob`, CycloneDX SBOM, and SLSA provenance generation in `.dagger/pipeline.rs`; alpha-stub header comment deleted.
- **Stale-binary drift:** `skillpack` smoke reproducers now run inside the `xtask ci` gate off a fresh workspace build, so a cached binary can no longer make a passing claim against stale code.
