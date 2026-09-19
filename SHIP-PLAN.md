# SkillPack — Upgrade & Ship Plan

> Generated 2026-09-19 from a live audit of the working tree. Every finding below
> was observed in-repo or via tooling output (`cargo update --dry-run`,
> `cargo audit`, `xtask stamp`, `git ls-files`, `rustup check`, `gh`).
> Evidence labels: OBSERVED = seen in source/output, VERIFIED = gate/tool ran
> green, INFERRED = strongly suggested, DECISION = needs owner call.

## Audit summary

| Area | State | Evidence |
|---|---|---|
| Version triple | consistent at `1.0.0-beta.2` | VERIFIED — `xtask stamp` green |
| `xtask ci` gate | running at plan-write time | gate output, this session |
| Cargo deps | 236 semver-compatible updates pending | `cargo update --dry-run` |
| Vulnerabilities | 7 RUSTSEC advisories, 8 unmaintained warnings | `cargo audit` |
| Toolchain | pinned 1.95.0; stable 1.98.1 available | `rustup check` |
| Tracked junk | ~2,528 files (~79% of tree): `.next` builds, `.playwright-mcp` logs, 39 root PNGs | `git ls-files` |
| `.gitignore` | 5 lines — bin re-includes only | file contents |
| Remote / tags | none configured, zero tags | `git remote -v`, `git tag` |
| `macos/skills-ecosystem` | embedded repo, no `.gitmodules`, dirty on `feature/hardening-polish-ipc` | `git -C` status |
| Dockerfile | builder `rust:1.85-bookworm` < MSRV 1.95 — container build expected to fail | Dockerfile line 8 |
| `.dagger` crate | version `1.0.0-beta.1`, stale vs workspace | `.dagger/Cargo.toml` |
| docs-site | `rspress: "latest"` — floating dep | `docs-site/package.json` |
| docker-compose | `zot:latest` — floating image tag | `docker-compose.yml` |
| release manifest | `generated_at` June 2026, hashes unverified against current tree | manifest field |
| README | references `.cargo/config.toml` — file does not exist | `ls .cargo` empty |

## Phase 0 — Baseline and decisions (do first)

1. Confirm `xtask ci` baseline green before touching anything. If it fails
   pre-change, that failure is the first fix item — do not stack upgrades on a
   red baseline.
2. DECISION — `macos/skills-ecosystem` embedded repo:
   - (a) Absorb it into this repo (delete inner `.git`, commit files directly).
     Simplest; one repo, one history.
   - (b) Formalize as a real submodule (`.gitmodules` + its own remote). Only if
     it ships/versioned independently.
   - (c) Remove the gitlink and keep it a separate project entirely.
   It currently has uncommitted work on `feature/hardening-polish-ipc` — commit
   or stash inside it first regardless of choice.
   Recommendation: (a) absorb — the CI already treats it as in-repo
   (`swift-schema-sync` job cds into it), and a gitlink with no `.gitmodules`
   is broken for every future clone.
3. DECISION — public-history cleanliness: ~2,528 junk files are in committed
   history. Options:
   - (a) `git rm --cached` going forward — junk stays in history forever;
     clone size stays inflated.
   - (b) `git filter-repo` purge before first push — clean history, rewrites
     all SHAs. Safe precisely because nothing is published yet.
   Recommendation: (b) — this is the only moment history rewrite is free.
4. DECISION — remote destination. `Cargo.toml` and CHANGELOG links already
   claim `github.com/ckodex/skillpack` (repo does not resolve today — create
   the org repo or pick a different canonical URL and update both files).
5. DECISION — version line. Recommend cutting `1.0.0-beta.3` after this round
   rather than re-stamping beta.2 — the vuln fixes and hygiene changes are
   exactly what a new beta stamp exists to mark.

## Phase 1 — Repository hygiene

Ordered so the public history is clean before any push.

- [ ] Write a real `.gitignore` (keep the two `!crates/*/src/bin/` re-includes):
      `target/`, `**/target/` exceptions for tracked fixtures, `node_modules/`,
      `.next/`, `.next-stale-bak/`, `out/`, `dist/`, `.DS_Store`,
      `.playwright-mcp/`, `.pi/`, `*.log`, `.env*`.
- [ ] `git rm -r --cached dashboard/.next dashboard/.next-stale-bak
      .playwright-mcp` (and any other build/cache paths surfaced).
- [ ] Root PNGs: 39 stray dev screenshots at repo root; README references only
      `docs/assets/screenshots/*`. DECISION: delete, or sweep into
      `docs/assets/screenshots/` if any are worth keeping.
- [ ] `.pi/` — agent session data; add to `.gitignore`, untrack if tracked.
- [ ] `crates/skillpack-adapters/target/` holds 2 tracked test fixtures —
      relocate under `tests/fixtures/` or the crate's test dir so `target/`
      can be ignored cleanly. INFERRED intentional; verify before moving.
- [ ] If Phase 0 decision is filter-repo: purge the above paths from history
      in the same pass, then commit the new `.gitignore`.
- [ ] Fix README's `.cargo/config.toml` reference — the shared target dir is
      set by user env/global config, not a repo file. Either add the file or
      correct the doc.

## Phase 2 — Rust upgrades

- [ ] `cargo update` (all 236 semver-compatible bumps). Expected to clear most
      advisories: dry-run already shows `crossbeam-epoch → 0.9.21`,
      `anyhow → 1.0.104`, `h2`, `quinn-proto`, `rkyv`, `webbrowser` paths.
- [ ] Re-run `cargo audit`; target zero unaddressed advisories.
      - `rsa` Marvin (RUSTSEC-2023-0071, via `sigstore`→`openidconnect`):
        DECISION — no fix exists upstream; either accept with documented
        rationale in `cargo-deny` config / audit ignore list, or isolate the
        sigstore feature. Document whichever way it lands.
      - Unmaintained warnings (`instant`, `proc-macro-error`,
        `proc-macro-error2`, `smallstr`, `scc`, `spin`, `aes`): identify which
        top-level dep pulls each; upgrade or allowlist with reason. `anyhow`
        unsoundness (RUSTSEC-2026-0190) clears with the update.
- [ ] Major bumps to evaluate one at a time (each gets its own commit + gate
      run): `jsonschema 0.46 → 0.56` (large API surface — biggest item),
      `brotli 8 → 9`, `serial_test 3 → 4` (dev-dep, cheap).
- [ ] Toolchain: bump `rust-toolchain.toml` channel `1.95.0 → 1.98.1`;
      decide whether `rust-version` (MSRV floor) stays 1.95 or moves to 1.98.
      Keep `Cargo.toml`, workflows, and Dockerfile coherent on the result.
- [ ] `cargo run -p xtask -- ci` after every bump group — not once at the end.

## Phase 3 — JavaScript / TypeScript upgrades

- [ ] `dashboard/`: `npm update` within caret ranges (next ^16, react ^19,
      eslint 10, typescript 6, vitest 3); `npm audit fix` where safe;
      `npm run build` + `npm run test` + `npm run lint` green.
- [ ] `vscode-extension/`: same loop; additionally reconcile
      `engines.vscode ^1.96.0` vs `@types/vscode ^1.118.0` — either raise the
      engine floor or cap the types package so the compile target matches the
      declared runtime floor.
- [ ] `docs-site/`: pin `rspress` to a real version (replace `"latest"`);
      regenerate lockfile; `rspress build` green.
- [ ] Delete `dashboard/.next`, `dashboard/.next-stale-bak` from disk after
      untracking.

## Phase 4 — Container / CI / pipeline

- [ ] Dockerfile: builder `rust:1.85-bookworm` → pinned current stable
      (`rust:1.98-bookworm` or per Phase 2 toolchain decision). Container
      build is currently expected to fail — verify with a real `docker build`.
- [ ] DECISION — runtime base: keep `debian:bookworm-slim` for beta.3, or jump
      to `gcr.io/distroless/cc-debian13` now and close the KNOWN-ISSUES item.
- [ ] `docker-compose.yml`: pin `zot` image to a digest or fixed tag (drop
      `:latest`); healthcheck uses `wget` — verify it exists in the zot image
      or switch to a scratch-friendly check.
- [ ] `.dagger/Cargo.toml`: version → match workspace, pin `tokio` to the
      workspace line, check `dagger-sdk 0.19` for updates.
- [ ] `skillpack-ci.yml` `schema-sync` job: pin the global
      `json-schema-to-typescript` install (currently floating `npm i -g`
      latest).
- [ ] `audit` job: cache the `cargo-audit` install (or use a maintained
      action); add `--deny warnings` policy decision.
- [ ] `artifacts` job uploads `target/release/*` — on GitHub-hosted runners
      the user's `~/.cache/cargo-target` CARGO_TARGET_DIR does not apply, so
      paths are correct, but verify once CI runs for real.

## Phase 5 — Known-issues burn-down (post-beta.3 candidates)

Scoped small fixes first; the rest stay honestly open in KNOWN-ISSUES.

- [ ] Fail-closed auth: when the server binds a non-loopback address and
      `SKILLPACK_API_TOKEN` is unset, refuse to start (or warn loudly).
      Closes the "default-open" item properly instead of "user vigilance".
- [ ] Coverage gaps: HTTP handlers, OCI publish/install, DuckDB persistence,
      MCP JSON-RPC — add in that order; each is a standalone chunk.
- [ ] Enumerate remaining stub checkers with `file:line` list into
      KNOWN-ISSUES (per repo evidence discipline).

## Phase 6 — Evidence & manifest integrity

- [ ] Re-verify `publish/release-manifest.json` hashes against current files;
      the manifest predates beta.2 changes. If `xtask` lacks a verify
      subcommand, script the sha256 comparison — a stale manifest is a false
      attestation.
- [ ] Regenerate/re-stamp the manifest to `1.0.0-beta.3`.
- [ ] CHANGELOG: new `1.0.0-beta.3` section — dependency refresh, advisory
      fixes, repo hygiene, Dockerfile fix.
- [ ] KNOWN-ISSUES: move closed items to Resolved with evidence refs.

## Phase 7 — Ship

- [ ] Create the GitHub repo (destination per Phase 0 decision), add remote,
      push `main`.
- [ ] Watch first CI run; fix whatever only-ever-fails-on-Linux.
- [ ] `release-tag.yml` cuts `v1.0.0-beta.3` automatically once green.
- [ ] Secondary surfaces, in priority order: OCI push of
      `agentic-skill-template` to the zot/registry flow, docs-site deploy,
      `cargo publish` decision (workspace crates are unpublished today),
      VS Code marketplace publish.
- [ ] Announcement artifact (ckodex-announcements) once the tag exists.

## Ordering rationale

Hygiene (Phase 1) precedes the first push because anything pushed becomes
permanent public history. Rust deps (Phase 2) precede tagging because shipping
a tag carrying known advisories contradicts the project's own grading thesis.
Phases 3–6 can interleave; Phase 5 items are deliberately scoped so beta.3 is
not held hostage by the coverage backlog.

## Exit criteria for "out to the world"

1. Public repo with clean tree, clean history (per decision), green CI.
2. `v1.0.0-beta.3` tag created by the release workflow, not by hand.
3. `cargo audit` output attached to the release notes or evidence dir —
      advisories either fixed or explicitly accepted with rationale.
4. One external-facing proof: a real third-party `SKILL.md` graded end-to-end,
      published where the README links point.
