# Assessment calibration notes — AgentSkills profile

Context for the profile-aware assessment introduced in
`skillpack-domain/src/profile.rs` and the AgentSkills rubric branches in
`skillpack-adapters/src/checkers/`. Cross-checked against the validation
harness in Addy Osmani's `agent-skills` catalog (surveyed 2026-07-12 from
`mc-gen-agent-skills`), whose conventions large skill catalogs follow.

## Why two profiles

The original 9-dimension rubric measured CNSB supply-chain artifacts (SBOM,
SLSA provenance, threat models, CI eval workflows). Real-world agent skills
carry none of those, so 400 of 404 fleet skills graded F while an empty
CNSB scaffold graded A — the assessor measured packaging, not quality.
Detection is structural: a `*.cnsb.json` manifest opts into the strict CNSB
profile; everything else is scored as an AgentSkills-convention skill.

## Conventions the AgentSkills rubric encodes (validated externally)

| Rule | Where enforced here |
|---|---|
| `name` kebab-case `^[a-z0-9]+(-[a-z0-9]+)*$`, ≤64 | identity checker |
| `description` ≤1024 chars, must carry a trigger clause ("use when/before/after/during") | `common::description_quality` |
| SKILL.md body <500 lines; depth moves to `references/` | compatibility checker |
| Internal resource links must resolve | compatibility checker |
| Discipline sections (When to Use / Red Flags / Rationalizations / Verification) mark battle-tested skills | documentation checker |
| Scripts preferred over inline code; dangerous patterns scanned | security checker |

## Implemented since

- **Description collision check** — `skillpack audit collisions` (also part of
  the default `skillpack audit` run). Pure TF-IDF cosine in
  `skillpack-domain/src/collision.rs`: warn ≥0.5, error ≥0.75, name tokens
  weighted 2x, IDF = ln(1 + n/(1+df)), suffix stemmer. First fleet run found
  28 collisions / 7 severe (e.g. `state-vector ↔ vector-state` at 1.00).

## Implemented since (cont.)

- **Trigger-routing eval (Tier 2)** — `skillpack route "<prompt>"` ranks the
  fleet against a task prompt via the collision TF-IDF engine
  (`rank_by_query`). `--expect <skill> --top-k <n>` makes it a CI gate:
  non-zero exit when the intended skill doesn't place. First live run
  surfaced a real defect — the `terraform` skill under-routes its own core
  use case ("provision cloud infrastructure") to rank 3.

## Implemented since (cont. 2)

- **Workflow-summary penalty** — `description_quality` now docks a description
  that enumerates steps ("1. … 2. …", "first … then … finally"). A
  step-listing description makes the agent follow the summary instead of
  reading the skill; a what+when router scores higher.
- **Fixture-driven routing eval** — `skillpack route --check <dir>` reads
  `<dir>/evals/routing.json` (`{ "positive": [...], "top_k": 3 }`) and asserts
  every positive prompt places the skill within top_k, else non-zero exit.
  This is the repeatable, CI-gateable form of the Tier-2 routing check,
  matching the surveyed agent-skills `evals/cases/<skill>.json` pattern.

## Implemented since (cont. 3)

- **Anti-gaming exemption model** — a skill may *request* dimension exemptions
  via frontmatter `x-skillpack-exempt: [testing, evals_hitl]`, but the grant
  lives **outside the artifact**: a grader-owned policy
  (`~/.config/skillpack/exemptions.toml`, or `SKILLPACK_EXEMPTIONS`; format
  `[grants]\n"skill-name" = ["testing", …]`). A *granted* exemption drops the
  dimension and re-normalizes the remaining weights (fair grading on what
  applies). An *ungranted* request fails loud — a hard Error issue is recorded,
  surfaced in `check` output, and the exemption is ignored, so a skill can
  never relax its own bar. Domain: `ExemptionPolicy`, `Assessment.exempt_dimensions`,
  re-normalized `base_score`. `check` now prints an error-severity summary so
  governance failures are visible, not buried.

## Implemented since (cont. 4)

- **`skillpack improve`** — assessment-driven remediation, closing the last
  under-served verb from the lifecycle set. It applies only the
  *mechanically-fixable* findings — add missing `version`, create a
  `CHANGELOG.md`, stub `references/…`/`scripts/…` paths the body links to but
  that don't exist — then re-assesses and reports the grade delta. Stub files
  carry a `TODO(skillpack):` marker so gaps stay honest; content-level findings
  (thin description, missing example) are reported for the author, never faked.
  `--dry-run` previews without writing. Verified: a fixable skill went F(55) →
  C(76), idempotent on re-run. Fixers live in
  `crates/skillpack-adapters/src/cli/improve.rs` (7 unit tests).

## Implemented since (cont. 5) — backlog now empty

- **Tier-3 behavioral evals** — `skillpack behavioral <dir>` runs a skill
  through a real agent in an isolated throwaway workspace and grades the
  tool-call trace **mechanically** (no LLM in the loop). Fixture:
  `evals/behavioral.json` (cases with `expect_tools` / `forbid_tools` /
  `expect_output_contains`). Opt-in and gated (`SKILLPACK_BEHAVIORAL=1` +
  `SKILLPACK_BEHAVIORAL_CMD`), advisory (never feeds the promote gate), with
  per-case timeout + process-group kill, symlink-skipping workspace staging,
  and untrusted-trace fencing. Threat model + encoded mitigations:
  `docs/tier3-behavioral-risk-analysis.md`. Pure core in
  `skillpack-domain/src/behavioral.rs`, IO/isolation in
  `skillpack-adapters/src/behavioral.rs` (13 deterministic tests via a mock
  runner; only the agent spawn is non-deterministic).

_All surveyed backlog items are now implemented._

## (historical) Backlog — ideas surveyed but not yet implemented
3. **Behavioral evals (Tier 3, opt-in)**: run the skill via a headless agent,
   grade the tool-call trace (what the agent did), not the prose. Trace must
   be fenced as untrusted data (injection defense).
4. **Anti-gaming exemption model**: any relaxation of the rubric for atypical
   skills (meta/routing) must be grader-owned (allowlist in the checker), and
   a skill self-declaring an exemption it wasn't granted should fail loudly.
5. **"Description is not a workflow summary" check**: a description that
   summarizes steps makes agents follow the summary instead of reading the
   skill. Heuristic candidate: penalize enumerations ("1.", "then", "finally")
   in frontmatter descriptions.
