# Alpha Release Readiness Report — SkillPack

> Static (read-only) review of the full `ckodex-skill-pack` monorepo.
> Reviewer roles: alpha-release readiness, principal engineer, security architect, QA lead, product reliability.
> Method: code/config/docs inspection only — no builds, tests, or audits were executed. Scores reflect inspectable evidence; where a score is limited by lack of execution, it is stated.
> Date: 2026-06-04

---

## Context (reconciled with repository)

The prompt template placeholders are resolved from repo evidence below. Items marked **(assumption)** are not formally documented in-repo and should be corrected by the team.

- **Project name:** SkillPack (`ckodex/skillpack`), workspace version `1.0.0`, Apache-2.0 (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Cargo.toml:14-19`).
- **Release target:** Alpha.
- **Primary users (assumption):** (1) skill authors, (2) AI-agent / platform integrators (gRPC/MCP/VS Code), (3) registry / governance operators.
- **Target environment:** Local dev = YES; Docker = YES (`Dockerfile`, `docker-compose.yml`); Kubernetes = not provided; Cloud = registry-agnostic OCI (GHCR default); Air-gapped = partially (OCI/S3/SFTP source adapters exist, but signing/attestation needs network).
- **Core stack:** Rust 2024 / rustc 1.85 workspace (8 crates); frontends = Next.js dashboard + VS Code extension (TypeScript) + macOS Swift `skills-ecosystem`; backend = gRPC (tonic) + HTTP (axum) + MCP (stdio JSON-RPC); storage = DuckDB (bundled); CI/CD = GitHub Actions + a Rust Dagger module; security tooling = Sigstore, CycloneDX SBOM, SLSA, in-toto, JSON-Schema validation, secret scanner.
- **Known alpha goals (assumption):** (1) demonstrate multi-dimension skill quality assessment + grading; (2) provide CLI + server + MCP + IDE surfaces over one domain core; (3) package/publish/install skills via OCI with supply-chain metadata.
- **Known non-goals (assumption):** (1) production multi-tenant registry hosting; (2) enterprise hardening / full SLSA L3 attested releases; (3) execution of untrusted skill code.

---

## Executive Summary

- **Overall readiness:** **CONDITIONAL GO**
- **Confidence level:** **Medium** (static review; CI/build not executed)
- **Recommended alpha-date readiness:** **Ready after fixes** (blockers are mostly release-plumbing and truth-in-documentation, not core product)
- **Overall Alpha Readiness Score:** **3.0 / 5** (average of the 10 area scores below)

### Top 5 Blockers

1. **CI is likely broken / unverified.** The workflow references `dtolnay/rust-action@stable`, which is not the canonical action (`dtolnay/rust-toolchain@stable`) (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.github/workflows/skillpack-ci.yml:21`). The `schema-sync` job runs `make generate-types`, which invokes Swift generation via a macOS sub-make and `npx --prefix dashboard json2ts` on an `ubuntu-latest` runner (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:147-165`) — these will fail on Linux. No green-CI evidence exists.
2. **Release pipeline is a stub.** `.dagger/pipeline.rs` returns `passed: true` for lint/test/build/sign/attest with `// In real impl:` comments — nothing is actually signed or attested (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.dagger/pipeline.rs:62-123`). Meanwhile `SECURITY.md` and the threat model claim "SLSA Level 3 provenance," "cosign signing," and "OPA policy gates" (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/SECURITY.md:36-42`, `@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/security/threat-model.md:66-72`). The `evidence/`, `proof/sbom/`, and `proof/attestations/` directories are empty.
3. **Server is default-open and exposes powerful filesystem-mutating endpoints.** Auth is disabled when `SKILLPACK_API_TOKEN` is unset (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/auth.rs:34-45`), and the HTTP router exposes `POST /migrate/skills`, `/migrate/agents`, skill CRUD, etc. (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/http_server.rs:103-120`). No CORS policy or rate limiting is configured.
4. **Version + scope are mislabeled for an alpha.** Workspace is `1.0.0` (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Cargo.toml:15`) but this is an alpha; there is no `CHANGELOG`, `KNOWN_ISSUES`, `ROADMAP`, or "alpha limitations" doc at repo root.
5. **Documentation-vs-reality gaps undermine trust.** `README.md` lists crates that aren't workspace members (`skillpack-cli`, `skillpack-vscode`, `skillpack-docs`) and duplicates its Architecture section (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/README.md:50-93`); the MCP `generate_report` tool is stubbed and `list_dimensions` returns a dimension set/weights that don't match the 9 real checkers (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/mcp.rs:191-208` vs `@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/checkers/mod.rs:19-31`).

### Top 5 Risks

1. **Panic surface:** ~307 `.unwrap()` calls across the Rust crates; an unhandled `unwrap` on attacker-influenced input can crash the server/CLI (DoS).
2. **Truthfulness of security posture:** claimed SLSA L3 / cosign / OPA controls are not wired in CI — risk of false assurance to alpha users and auditors.
3. **OTel export not actually wired:** `init_telemetry` accepts an OTLP endpoint but ignores it (`_otlp_endpoint`) (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/telemetry.rs:16-38`); distributed tracing is effectively unavailable despite the dependency set.
4. **Schema/type drift:** canonical types are codegen'd into Rust/TS/Swift; if the (likely-failing) `schema-sync` gate doesn't run, drift can ship silently (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:167-181`).
5. **Unvalidated path inputs:** MCP/gRPC/HTTP assess paths are taken verbatim (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/mcp.rs:142-148`, `@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/server.rs:46-49`); local-only read assessment limits impact, but path-traversal hardening (claimed in threat model E1) is not visibly enforced at these entry points.

### Top 5 Quick Wins

1. Fix the CI action name (`dtolnay/rust-toolchain@stable`) and split `schema-sync` so Linux runners don't invoke Swift/macOS make — get a green pipeline.
2. Bind the server to `127.0.0.1` by default and log a loud warning when `SKILLPACK_API_TOKEN` is unset; document that mutation/migration endpoints require a token.
3. Add `CHANGELOG.md`, `KNOWN-ISSUES.md`, and an "Alpha Limitations" section; re-tag as `1.0.0-alpha.1` (or `0.x`) to set expectations.
4. Reconcile docs: fix README crate list/duplication, correct MCP `list_dimensions` to the real 9 dimensions, and either implement or clearly mark MCP `generate_report` as not-yet-implemented.
5. Either implement the Dagger sign/attest stages or soften `SECURITY.md`/threat-model claims to "planned" so the security narrative matches reality; commit a sample SBOM into `proof/sbom/`.

---

## Readiness Scorecard

| Area | Score 0–5 | Status | Rationale |
|---|---:|---|---|
| Product | 4 | GO | Clear value prop, coherent journeys (init→check→lock→package→publish→install→eval), template + docs for onboarding; minor stubbed surfaces. |
| Engineering | 3 | CONDITIONAL | Clean hexagonal core and broad feature set, but ~307 `.unwrap()`s, version/doc drift, and stubbed code. |
| Security | 3 | CONDITIONAL | Threat model, bearer auth, secret scanning, schema validation present; default-open auth + powerful mutation endpoints + unsubstantiated supply-chain claims pull it down. |
| Agentic / AI | 3 | GO (bounded) | MCP exposes read-only assessment tools over local stdio; no exec/write/network tools. ARL-2. One stubbed tool + inaccurate dimension metadata. |
| Testing | 3 | CONDITIONAL | ~172 test fns / 33 files + fixture corpus + integration crate; gaps in HTTP/OCI/persistence/MCP and no e2e/contract/fuzz; CI greenness unverified. |
| CI/CD Release | 2 | NO-GO (as-is) | CI likely broken; release pipeline stubbed; no real signing/SBOM/provenance, changelog, tagging, or rollback procedure. |
| Observability / Ops | 3 | CONDITIONAL | Structured logging + Prometheus metrics + carbon scoring + healthcheck; OTLP/tracing export not wired; no runbook/dashboards. |
| Documentation | 4 | GO | Strong README/ARCHITECTURE/docs/docs-site/CONTRIBUTING/SECURITY/threat-model; missing changelog/known-issues/alpha-limits and some drift. |
| UX / Accessibility | 3 | CONDITIONAL | Good CLI ergonomics (clap/colored); dashboard + VS Code extension present; app-level a11y unverified. |
| Compliance Evidence | 2 | NO-GO (as-is) | Tooling/deps exist but evidence dirs empty and pipeline stubbed; cannot currently emit an evidence bundle automatically. |

```text
Overall Alpha Readiness Score = (4+3+3+3+3+2+3+4+3+2) / 10 = 3.0 / 5
```

---

## 1. Product Readiness — Score 4/5

**Assessment.** The core problem — multi-dimensional quality assessment of AI-agent skill packs ("SonarQube for skills") — is clearly solved and demonstrable. Nine real dimension checkers run end-to-end (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/checkers/mod.rs:19-31`). Primary journeys are implemented across CLI, gRPC, HTTP, MCP, and IDE. Onboarding is supported via `skillpack init --template` and a worked `examples/agentic-skill-template/` with a catalog and orchestrator agent (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/examples/agentic-skill-template/catalog.json:1-20`). A clear demo path exists (`README` Quick Start + `make assess`).

- **Alpha blockers:** none that block evaluation; the MCP `generate_report` stub may confuse agent users (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/mcp.rs:191-198`).
- **Nice-to-have:** document alpha goals/non-goals explicitly; a one-command demo script.
- **Recommended alpha user profile:** trusted skill authors and integration partners running locally or in a private container; not public/multi-tenant.
- **Suggested release-notes summary:** *"SkillPack alpha: assess, grade, and package AI-agent skills across 9 quality dimensions, with CLI, gRPC/HTTP server, MCP server, and VS Code/dashboard surfaces. OCI publish + supply-chain metadata are preview-quality. Auth and signed-release pipeline are not yet production-hardened."*

## 2. Engineering Readiness — Score 3/5

**Strengths.** Strict hexagonal architecture with a documented dependency rule (domain has zero infra deps) (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/ARCHITECTURE.md:20-48`); pinned `Cargo.lock`; layered Dockerfile with dependency caching and a non-root runtime user (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Dockerfile:53-65`); canonical-type codegen across Rust/TS/Swift with drift checks (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:167-181`).

- **Fragile areas:** ~307 `.unwrap()` calls (panic/DoS risk on bad input); Dockerfile relies on `touch`-to-force-rebuild and `2>/dev/null || true` masking of the cache build (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Dockerfile:32-41`).
- **Architecture risks:** README advertises crates that don't exist as workspace members (`skillpack-cli`, `skillpack-vscode`, `skillpack-docs`) vs the real 8 members (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Cargo.toml:3-12`); CONTRIBUTING shows a different crate layout again (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/CONTRIBUTING.md:64-70`).
- **Required refactors before alpha:** replace user-input-reachable `.unwrap()`s in adapters/server entry paths with `?`/typed errors; fix version label.
- **Deferred (acceptable post-alpha):** broader error-type unification, Dockerfile build simplification, fuller config-via-file support.

## 3. Security Readiness — Score 3/5

**Inspected.** Bearer-token auth with default-open behavior for local dev (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/auth.rs:21-93`); secret scanner with AWS/GitHub regexes + Shannon-entropy heuristic (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/checkers/secrets.rs:28-62`); JSON-Schema validation in the domain; STRIDE threat model present (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/security/threat-model.md:1-72`); non-root container; `cargo audit` job in CI (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.github/workflows/skillpack-ci.yml:41-51`).

**Framework mapping:**
- **OWASP Top 10:** A01 Broken Access Control (default-open auth + mutation endpoints); A05 Security Misconfiguration (no CORS/rate-limit; claims vs. config mismatch); A08 Software & Data Integrity (signing/provenance claimed but not executed); A09 Logging/Monitoring (no warning on auth-disabled; OTLP not wired).
- **OWASP LLM/Agent:** LLM06 (sensitive-info disclosure via unvalidated assess path → file reads) — low impact, local; LLM08-style excessive agency is **not** present (MCP tools are read-only).
- **NIST SP 800-53 families:** AC (access control) partial; AU (audit) partial — request-level audit trail for mutation/migration endpoints is absent; CM (configuration mgmt) partial; SR (supply chain) **claimed but not operational**.
- **CIS themes:** container runs non-root (good); image is `debian:bookworm-slim` not distroless despite Dagger comment (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Dockerfile:46`).

- **Critical risks:** none for a *trusted local* alpha **provided** the server is not exposed to untrusted networks.
- **High risks:** default-open auth on filesystem-mutating `/migrate/*` + skill CRUD endpoints (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/http_server.rs:103-120`); unsubstantiated supply-chain claims.
- **Medium risks:** ~307 panic sites; no CORS/rate limit; secret scanner coverage is narrow.
- **Acceptable alpha risks:** narrow secret-scanner regex set; DuckDB local file store; non-distroless base image.
- **Required mitigations before alpha:** bind to localhost by default; emit a startup WARN when auth is disabled; require token for mutation/migration routes; align `SECURITY.md`/threat-model claims with what CI actually enforces.
- **Do NOT release if:** the server is exposed on a non-loopback interface without `SKILLPACK_API_TOKEN` set; or if releases are presented as "signed / SLSA L3 / SBOM-attested" while the pipeline remains stubbed.

## 4. Agentic / AI Readiness — **ARL-2 (Human-supervised tool use)** — Score 3/5

**Inspected.** MCP server over stdio JSON-RPC 2.0 exposes 4 tools — `assess_skill`, `grade_skill`, `generate_report`, `list_dimensions` (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/mcp.rs:39-105`, `220-318`). All tools are **read-only assessment**; there is no code execution, file write, or network tool. Blast radius is therefore inherently low.

- **Tool permissions / sandbox:** tools only read a filesystem path and compute scores; no write/exec. Path argument is unvalidated but cannot mutate state.
- **Human approval gates:** N/A for read-only tools; the example agent template defaults write-path calls to `--dry-run` and masks tokens (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/examples/agentic-skill-template/catalog.json:14`) — a good pattern to enforce project-wide.
- **Prompt-injection / exfiltration:** limited — outputs are numeric scores/grades; `generate_report` is stubbed so it cannot leak file contents (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/mcp.rs:191-198`).
- **Auditability:** MCP tool calls are not logged/traced; add structured audit logging before broadening tools.
- **Evaluation coverage:** `skillpack eval` suites and `skillpack-tests` exist, but MCP handlers themselves lack tests.
- **Required gates / sandboxing / audit:** validate/canonicalize the `path` argument; log every tool call; keep tools read-only for alpha; if `generate_report` is implemented to include file content, add redaction.
- **Go / no-go for agent capabilities:** **GO** for the current read-only MCP toolset; **NO-GO** for adding any write/exec/network tool without approval gates and audit logging.

## 5. Test & Quality Readiness — Score 3/5

**Inspected.** ~172 test functions across 33 Rust files; a fixtures corpus under `tests/fixtures/{skills,skilliq}` (40 + 19 entries); a dedicated `skillpack-tests` integration crate (`schema_round_trip`, `fixture_grade_corpus`); checker-level tests assert full dimension coverage and no stub dimensions (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-adapters/src/checkers/mod.rs:39-88`).

- **Coverage gaps:** HTTP server handlers, OCI publish/install, DuckDB persistence, MCP JSON-RPC handlers, gRPC services; no e2e, contract, golden, fuzz, or accessibility/perf tests.
- **Flaky/missing:** CI greenness is unverified (see §6); no coverage threshold gate.
- **Minimum alpha test suite:** `cargo test --workspace` + the fixture grade corpus + at least one HTTP-auth test and one MCP tools/list+tools/call test.
- **Recommended commands:** `cargo test --workspace`; `cargo test -p skillpack-domain schema_validation`; `cargo run --bin skillpack -- check .`; `cargo run --bin skillpack -- eval --suite smoke`.
- **Required CI gates before alpha:** working `fmt`/`clippy -D warnings`/`test`/`audit` + schema-sync, all observed green at least once.

## 6. CI/CD & Release Readiness — Score 2/5

**Inspected.** GitHub Actions workflow with check/clippy/test/build + self-assessment, `cargo audit`, schema validation, and type-sync jobs (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.github/workflows/skillpack-ci.yml:13-85`); `Makefile` release/sign/sbom/provenance targets (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:46-99`); a Rust Dagger module (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.dagger/pipeline.rs`).

- **Findings:** (a) `dtolnay/rust-action@stable` is not the canonical toolchain action — likely fails setup (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.github/workflows/skillpack-ci.yml:21`); (b) `schema-sync` runs Swift/macOS make + `npx --prefix dashboard` on Linux (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:147-165`); (c) Dagger sign/attest/build are stubbed (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/.dagger/pipeline.rs:84-123`); (d) no changelog, release tagging workflow, image-publish job, or rollback doc.
- **Required release checklist (alpha):** green CI; pinned toolchain; `1.0.0-alpha.N` tag; built CLI + server artifacts with `checksums.txt` (`make release`); committed SBOM; documented manual approval; documented rollback.
- **Required artifacts:** `skillpack`/`skillpack-server` binaries, `sbom.cyclonedx.json`, container image, `checksums.txt`.
- **Required manual approval points:** publish-to-registry and (when real) signing steps.
- **Required rollback procedure:** keep previous tagged image/binary; document `docker compose` pin-to-previous-tag and `cargo install --version` downgrade; DuckDB file is local so data rollback = restore prior `/data/skillpack.db`.

## 7. Observability & Operations Readiness — Score 3/5

**Inspected.** Structured logging via `tracing`, a Prometheus `/metrics` exporter, custom assessment/OCI/issue metrics, and a carbon-scoring gauge (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/telemetry.rs:16-165`); HTTP `/health` endpoint + container healthcheck (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/docker-compose.yml:20-25`).

- **Missing telemetry:** OTLP/distributed tracing is declared but **not wired** — the endpoint arg is ignored (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/telemetry.rs:16-38`); no readiness (vs. liveness) separation; no request-level audit log for mutations.
- **Required alpha dashboards/logs:** Prometheus scrape of assessment count/score/duration + error counters; capture server stdout logs.
- **Minimum runbook:** how to start/stop (`docker compose up`), where data lives (`/data/skillpack.db`), how to set the API token, how to read `/health` and `/metrics`, and how to roll back.
- **Known operational risks:** panics from `.unwrap()`; unbounded request handling (no rate limit); local-only single-file DB (no HA).

## 8. Documentation Readiness — Score 4/5

**Inspected.** `README.md`, a deep `ARCHITECTURE.md`, `docs/` (authoring, agentic patterns, OCI, per-dimension), an rspress `docs-site/` with guide pages (getting-started, cli, grpc-api, security, dimensions, architecture, remote-sources), `CONTRIBUTING.md`, `SECURITY.md`, and a STRIDE threat model.

- **Missing docs:** `CHANGELOG`, `KNOWN-ISSUES`, "Alpha Limitations," and a `CODE_OF_CONDUCT.md` (referenced by CONTRIBUTING but absent) (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/CONTRIBUTING.md:5-7`).
- **Confusing docs:** README duplicates its Architecture block and lists non-member crates (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/README.md:48-95`); SECURITY claims (SLSA L3/cosign) outrun the implementation.
- **Required alpha documentation:** alpha-limitations + known-issues; accurate crate map; "how releases are (and aren't yet) signed."
- **Suggested README improvements:** de-duplicate, fix crate list, add an "Alpha status" banner and link to this report.

## 9. UX / Accessibility Readiness — Score 3/5

**Inspected.** CLI uses `clap` (derive + env + color) and `colored` output with a broad, well-named command surface (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/README.md:63-83`); a Next.js dashboard and a feature-rich VS Code extension (`package.json` ~14 KB) provide GUI surfaces; gRPC + REST proxy endpoints give a reasonable API DX (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/http_server.rs:82-101`).

- **Accessibility risks:** dashboard/VS Code a11y (keyboard nav, contrast, screen-reader labels) is unverified in this static pass; the product *scores* accessibility for skills but its own UIs aren't audited.
- **Friction points:** no loud signal when running auth-disabled; MCP `generate_report` returns a misleading success string.
- **Required fixes before alpha:** ensure error messages on bad paths/missing manifests are actionable; basic dashboard a11y smoke check.

## 10. Compliance & Evidence Readiness — Score 2/5

**Inspected.** Dependencies and Make targets exist for SBOM (CycloneDX), Sigstore signing, SLSA provenance, and in-toto (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Cargo.toml:77-88`, `@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/Makefile:73-99`), but the `evidence/`, `proof/sbom/`, and `proof/attestations/` directories are empty and the pipeline that would populate them is stubbed.

- **Missing evidence:** SBOM artifact, provenance, signatures, vuln-scan output, test-evidence, release-approval record, risk-acceptance note.
- **AI-BOM:** not produced; given MCP/agent surfaces, an AI-BOM (tools, models referenced, data sources) would strengthen the alpha story.
- **Required evidence bundle for alpha:** at minimum a committed SBOM + `cargo audit` output + this readiness report + a signed risk-acceptance note for the default-open-auth decision.
- **Suggested artifact structure:**
  ```text
  evidence/
    ALPHA-READINESS.md          (this report)
    risk-acceptance.md          (signed, dated)
    sbom.cyclonedx.json
    cargo-audit.json
    test-summary.txt
  proof/
    sbom/  provenance/  attestations/   (populated by a real pipeline)
  ```

---

## Consolidated Go / No-Go

**CONDITIONAL GO** for a *trusted, local/private* alpha audience, contingent on the following before tagging:

1. Green CI (fix toolchain action + Linux-safe schema-sync).
2. Server defaults to loopback; loud warning + token requirement for mutation/migration endpoints.
3. Re-tag as `*-alpha`; add CHANGELOG + alpha-limitations + known-issues.
4. Reconcile security/docs claims with reality (or implement the signing/SBOM pipeline) and commit at least an SBOM + risk-acceptance note.

**NO-GO conditions:** server exposed on a non-loopback interface without a token; or marketing the release as signed/SLSA-attested while the pipeline is stubbed.

*Scores reflect static inspection only; running `cargo build/test/clippy/audit` and a real Dagger/CI run would raise confidence and could move Testing and CI/CD scores once verified green.*
