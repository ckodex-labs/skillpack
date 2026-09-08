# Agentic Skill Authoring Patterns

> Companion to [`authoring-guide.md`](./authoring-guide.md). That guide covers
> the **CNSB bundle** packaging path (`skill.cnsb.json`, `skill.lock`, OCI
> distribution). This guide covers the **Claude Code Agent Skill** runtime path
> (`SKILL.md` + `scripts/` + `references/` + an orchestrator agent), distilled
> from a production agentic skill that ships 13 deterministic shell scripts
> behind one orchestrator.
>
> The two paths are complementary, not alternatives. CNSB is *distribution*.
> Agent Skills is *runtime contract*. A skill can satisfy both.

Working template that exercises every pattern below:
[`examples/agentic-skill-template/`](../examples/agentic-skill-template/).

---

## 0. Schema source-of-truth

- Frontmatter: [`schemas/agentskills.schema.json`](../schemas/agentskills.schema.json) (`$id: https://agentskills.io/schema/skill.json`).
  - `name`: required, max 64, `^[a-z0-9]+(-[a-z0-9]+)*$`, must equal parent dir.
  - `description`: required, max **1024 chars** (schema-enforced). Runtime adds
    a stricter combined cap on `description + when_to_use ≤ 1536 chars`.
  - `license`, `compatibility ≤500`, `metadata` (string-valued map),
    `allowed-tools` (space-delimited).
- Directory layout: [`schemas/agentskills-directory.schema.json`](../schemas/agentskills-directory.schema.json).
  - `SKILL.md` required at root, body recommended <500 lines.
  - `scripts/`, `references/`, `assets/` optional.

---

## The ten patterns

Every pattern below has a one-line invariant and a pointer to where the template
exercises it. Read invariants as `□` (always-holds) — violating them produces
the listed failure mode.

### Pattern 1 — Anchor-term-dense description (≤1024, no marketing)

**Invariant:** `□(description ⟹ anchor_terms ≠ ∅ ∧ ¬marketing_vocab ∧ len ≤ 1024)`

The description is the only field the model sees when deciding whether to load
the skill. It must be a dense bag of anchor terms — concrete nouns the user is
likely to type — not a tagline. Marketing vocabulary (`robust`, `seamless`,
`comprehensive`, `enterprise-grade`) is a VWP P-SC-002 violation: those terms
substitute for evidence.

**Target shape:** 400–900 chars, 15–30 anchor terms, zero hedging
(`should work`, `probably`), zero marketing.

**Anti-pattern:**

```yaml
description: A robust, enterprise-grade toolkit for seamless integration.
```

**Pattern:**

```yaml
description: Use when the user works with the JFrog Platform — Artifactory
  repository CRUD across local/remote/virtual/federated rclass; build
  promotion; Release Bundle v2 create/promote/distribute; Xray scans, policies,
  watches; SBOM, SLSA provenance, VSA, evidence attachment (CycloneDX, SPDX,
  in-toto, DSSE, cosign, Rekor); OIDC for CI; GitLab MR gates; project quotas;
  jf CLI v2; air-gap distribution.
```

**Why it matters:** under context pressure, the skill listing budget defaults
to ~1% of the context window. Skills whose descriptions don't match user
vocabulary get evicted before they're loaded.

**Template ref:** `examples/agentic-skill-template/skills/agentic-skill-template/SKILL.md` lines 1-7.

---

### Pattern 2 — Progressive disclosure (SKILL.md ≤500 + references/ + scripts/)

**Invariant:** `□(SKILL.md ≤ 500_lines ∧ deep_detail ∈ references/)`

A skill stays loaded once invoked. Every line of `SKILL.md` competes for the
25k post-compaction budget. Keep the body to routing + invariants + safety;
push protocol details, API endpoint catalogs, and per-package config into
`references/*.md` that the agent loads on demand.

**Layered surface:**

| Layer            | Loaded         | Holds                                          |
| ---------------- | -------------- | ---------------------------------------------- |
| `SKILL.md`       | on invocation  | when-to-use, routing table, safety, envelope   |
| `references/*.md`| on demand      | API catalog, package-type matrix, CLI flags    |
| `scripts/*.sh`   | on `bash` call | executable logic                               |
| `assets/`        | on demand      | templates, fixtures, prompts                   |

**Template ref:** template ships `SKILL.md` (≈100 lines) + `references/EXAMPLE.md` + `scripts/`.

---

### Pattern 3 — Deterministic JSON envelope (chainable, parseable)

**Invariant:** `□(script_output ⟹ valid_json_envelope)`

Every script returns the same shape, so an agent can chain calls without ever
parsing prose. This is the single most load-bearing pattern in the
artifactory skill — it's what makes 13 scripts behave like one composable
toolkit.

**Success envelope:**

```json
{
  "ok": true,
  "data": { "...": "command-specific" },
  "metadata": { "command": "hello", "jfrog_url": "https://...", "version": "0.1.0" }
}
```

**Failure envelope:**

```json
{
  "ok": false,
  "error": { "code": "UPPER_SNAKE", "message": "human-readable", "retriable": true },
  "metadata": { "command": "hello", "jfrog_url": "https://...", "version": "0.1.0" }
}
```

Both shapes carry identical `metadata`. The `retriable` boolean is the agent's
contract for whether to chain or back off.

**Template ref:** `scripts/lib/common.sh` defines `emit_ok` and `emit_error`.

---

### Pattern 4 — Dual-channel exit-code contract

**Invariant:** `□(exit ∈ {0, 1}) ∧ □(stdout = success ∨ retriable_error) ∧ □(stderr = fatal_config_error)`

The deterministic envelope alone isn't enough — the agent also needs to know
*which channel to read first*. The contract:

| Channel | Exit | When                                                              | Agent action                              |
| ------- | ---- | ----------------------------------------------------------------- | ----------------------------------------- |
| stdout  | 0    | success, or operational error with `retriable` flag (HTTP, 5xx)   | parse `.data` / `.error`; chain or retry  |
| stderr  | 1    | fatal config error (bad flag, missing env, file/signature absent) | surface envelope to user; **never retry** |

Operational failures (timeouts, 503s, transient network errors) land on
**stdout exit 0** with `ok:false, retriable:true`. Programming errors (a missing
`--required-flag`) land on **stderr exit 1** with `retriable:false`.

This split lets the agent chain calls with `|| break` semantics instead of
parsing two different envelope shapes from two different streams.

**Template ref:** `scripts/lib/common.sh` `emit_error` (stdout/exit 0) vs `die` (stderr/exit 1).

---

### Pattern 5 — Single orchestrator agent + script library

**Invariant:** `□(agent_count = 1 ∧ scripts_count ≥ 1)`

Resist the urge to ship one agent per capability. Ship one orchestrator whose
job is to *route* user signals to scripts, parse envelopes, and chain calls.
The orchestrator's frontmatter `description` is the routing surface; the
scripts are the executable surface.

**Why:** N agents create N contexts, N safety surfaces, N descriptions
competing for the listing budget. One orchestrator with N scripts produces
1 context, 1 audit trail, 1 safety contract.

**Frontmatter pattern (Claude Code agents):**

```yaml
---
name: skill-engineer
description: Orchestrator that routes <domain> work into deterministic scripts
  under skills/<skill>/scripts/. Parses the JSON envelope from every script;
  respects the dual-channel contract. Defaults write-path calls to --dry-run
  and masks tokens in all error paths.
model: sonnet
tools: Read, Write, Edit, Bash, Glob, Grep
---
```

**Template ref:** `examples/agentic-skill-template/agents/agentic-engineer.md`.

---

### Pattern 6 — Routing table (user prompt → script + reference)

**Invariant:** `□(every_user_signal ⟹ ∃!(script, reference))`

The orchestrator's `SKILL.md` and agent file both carry a routing table that
maps anchor phrases the user is likely to say to the canonical script + the
reference doc that explains the protocol. This is the runtime equivalent of
DAAS decomposition (CLAUDE.md §6).

**Shape:**

| User signal                                   | Script + reference                                  |
| --------------------------------------------- | --------------------------------------------------- |
| "create / list / delete a repo"               | `scripts/repo-crud.sh` + `references/REPO_MGMT.md`  |
| "promote build … to …"                        | `scripts/build-promote.sh` + `references/LIFE.md`   |
| "Xray scan / policy / watch"                  | `scripts/xray-scan.sh` + `references/XRAY.md`       |

The orchestrator never invents a script name — every row resolves to a file
that exists. Conformance test #3 (see Pattern 9) enforces this.

**Template ref:** `examples/agentic-skill-template/agents/agentic-engineer.md` "Routing table" section.

---

### Pattern 7 — Safety defaults (write-path dry-run, token masking, HTTPS-only)

**Invariant:** `□(write_path ⟹ default(--dry-run)) ∧ □(token_in_error ⟹ masked) ∧ □(url ⟹ scheme = https)`

Three release-blocking defaults the orchestrator must encode:

1. **`--dry-run` default on every write-path script.** The user must explicitly
   pass `--no-dry-run`. This survives accidental autonomous escalation.
2. **Token masking in *every* error path.** `mask_token` produces
   `****<last4>`; raw tokens never appear in stdout, stderr, logs, or comments
   posted to external systems.
3. **HTTPS-only at the URL boundary.** `https_required` refuses `http://` for
   the platform URL. No `--insecure`, no `-k`. (Air-gap signature verification
   has the same status — never `--skip-verify` in prod.)

**Why these three:** they're the smallest set that defeats the most common
agentic-failure modes — accidental writes, token leakage in posted comments,
and credential interception over plaintext.

**Template ref:** `scripts/lib/common.sh` exports `mask_token`, `https_required`, and `DRY_RUN` default.

---

### Pattern 8 — Two-manifest plugin pattern (`.claude-plugin/plugin.json` + `catalog.json`)

**Invariant:** `□(installable_plugin ⟹ ∃.claude-plugin/plugin.json) ∧ □(marketplace_entry ⟹ ∃catalog.json)`

These are not interchangeable. They serve different consumers:

| File                          | Consumer            | Contains                                          |
| ----------------------------- | ------------------- | ------------------------------------------------- |
| `.claude-plugin/plugin.json`  | Claude Code install | name, version, license, keywords, author          |
| `catalog.json`                | Marketplace index   | `skills[]` + `agents[]` arrays with paths         |

Skip `.claude-plugin/plugin.json` and the plugin won't install. Skip
`catalog.json` and it won't be discoverable. Both are required for full
runtime + marketplace coverage.

**Template ref:** template ships both at repository root.

---

### Pattern 9 — Three test gates (structure + scripts + agent)

**Invariant:** `□(release ⟹ structure.test ∧ scripts.test ∧ agent.test ⊨ pass)`

The minimum CI surface for an agentic skill:

| Gate                | Checks                                                                              |
| ------------------- | ----------------------------------------------------------------------------------- |
| `structure.test.sh` | files exist; `SKILL.md` <500 lines; `description` ≤1024 chars; manifests valid JSON |
| `scripts.test.sh`   | shellcheck clean; `--help` works on every script; transport-error envelope regression |
| `agent.test.sh`     | agent frontmatter has `name`/`description`/`model`/`tools`; routes every anchor; safety clauses present |

The third test (agent) is the one most teams skip and most needs. It catches
routing drift — when a script gets renamed but the agent still references the
old name — at PR time, not at user-invocation time.

**Template ref:** `examples/agentic-skill-template/skills/agentic-skill-template/tests/{structure,scripts,agent}.test.sh`.

---

### Pattern 10 — 2026 Claude Code surfaces (`$ARGUMENTS`, `${CLAUDE_SKILL_DIR}`, `context: fork`, `!\`cmd\``)

**Invariant:** `□(skill_uses_new_surface ⟹ skill_documents_minimum_runtime_version)`

The 2026 Claude Code Skills spec adds runtime surfaces beyond what older
skills exercise. Use them when they fit; document the dependency.

| Surface                       | Use                                                              |
| ----------------------------- | ---------------------------------------------------------------- |
| `$ARGUMENTS`, `$N`, `$name`   | Argument substitution in SKILL.md body                           |
| `${CLAUDE_SKILL_DIR}`         | Resolve paths relative to the skill root (portable, cwd-independent) |
| `${CLAUDE_SESSION_ID}`        | Session-scoped temp files                                        |
| `${CLAUDE_EFFORT}`            | Adjust verbosity / depth based on the runtime's effort hint      |
| `` !`<cmd>` `` (inline)       | Inject command output into the skill body at load time           |
| ```` ```! ```` (fenced)       | Multi-line dynamic context block                                 |
| `context: fork`               | Run the skill in a forked subagent context (doesn't pollute caller) |
| `disable-model-invocation: true` | User-invocable only (slash command); model cannot auto-load   |
| `allowed-tools: Bash(git:*)`  | Pre-approved tool surface                                        |

**Listing budget:** `skillListingBudgetFraction` defaults to 1% of the context
window. `maxSkillDescriptionChars` caps each entry. Skills whose descriptions
exceed the cap are listed name-only or evicted entirely.

**Post-compaction:** the first 5000 tokens of each loaded skill survive
compaction; total skill budget after compaction is 25k tokens. Plan the body
of `SKILL.md` to fit useful routing into the first 5k.

**Template ref:** template's `SKILL.md` uses `${CLAUDE_SKILL_DIR}` in script invocations.

---

## Anti-patterns (release-blocking)

These are the failure modes the patterns above defend against. Every one is
either a VWP violation or a runtime-behavior bug.

| Anti-pattern                                                | Why it's blocked                                                  |
| ----------------------------------------------------------- | ----------------------------------------------------------------- |
| Marketing vocab in `description` (`robust`, `seamless`)     | VWP P-SC-002 — substitutes for evidence                           |
| Hedging vocab in agent output (`should work`, `probably`)   | VWP P-SC-001 — no signal of verification                          |
| Claim without `evidence_refs`                               | VWP P-VW-001 — unverified completion                              |
| Fake / unresolvable citation (CVE, RFC, hash, URN)          | VWP P-SC-008 — citation must resolve                              |
| Token echoed to logs/comments                               | Credential leak surface                                           |
| `http://` accepted for platform URL                         | Plaintext credential interception                                 |
| `--skip-verify` on air-gap import                           | Bypass of the only mandatory boundary check                       |
| `--no-dry-run` as default                                   | Accidental destructive writes under autonomy                      |
| Two agents where one orchestrator suffices                  | Double listing budget cost, split safety surface                  |
| Scripts that parse each other's prose output                | Brittle; breaks under wording drift                               |
| `SKILL.md` > 500 lines                                      | Burns post-compaction budget on every invocation                  |
| `description` > 1024 chars                                  | Schema violation; entry silently truncated or evicted             |
| Stub without surfacing in completion report                 | VWP P-VW-004 — silent stub                                        |
| Mock in production code                                     | Mocks live in tests only                                          |

---

## Composition rule

A capability that can't be reduced to a single envelope-returning script
shouldn't be a script — it's a *chain*. Document chains in `references/`
as named sequences, not as new scripts. The orchestrator composes them at
runtime by piping `.data.*` from one envelope into the next:

```bash
# Chain: scan → promote → attest
bash scripts/xray-scan.sh --target build --name "$N" --number "$M" \
  | jq -e '.ok and (.data.violations|length == 0)' >/dev/null || break
bash scripts/build-promote.sh --build-name "$N" --build-number "$M" --no-dry-run
bash scripts/transparency-attach.sh --target build --build-name "$N" --build-number "$M" --no-dry-run
```

The agent never composes by parsing prose. Always parse the envelope.

---

## Classification (per CKODEX VWP §26)

| Pattern | Class | Status                                                                |
| ------- | ----- | --------------------------------------------------------------------- |
| 1–9     | **C** | Concrete, mechanically enforced by the three test gates in template   |
| 10      | **C** | Concrete; runtime presence is testable via env-var lookup at load time |

All patterns are mechanically verifiable. No `[S]` (specified-but-unimplemented)
or `[A]` (aspirational) entries in this guide.

---

## Cross-references

- [`authoring-guide.md`](./authoring-guide.md) — CNSB bundle packaging path
- [`oci-distribution.md`](./oci-distribution.md) — OCI registry publishing
- [`dimensions/`](./dimensions/) — SkillPack quality assessment dimensions
- [`../schemas/agentskills.schema.json`](../schemas/agentskills.schema.json) — frontmatter schema
- [`../schemas/agentskills-directory.schema.json`](../schemas/agentskills-directory.schema.json) — directory layout schema
- [`../examples/agentic-skill-template/`](../examples/agentic-skill-template/) — working template
