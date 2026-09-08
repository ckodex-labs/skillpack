# The canonical registry — one store, consumed by reference

SkillPack keeps every skill **once**, in a canonical store, and every agent
harness **references** it via symlink. No harness copies skills into its own
tree. This is the single-source-of-truth registry.

- **Store root:** `~/Skills/shared` (override with `SKILLPACK_CANONICAL_ROOT`).
- **Discovery manifest:** `~/Skills/shared/.skillpack-registry.json` — a
  `.well-known`-style contract any harness can read to find the registry and
  learn how to consume it. Refresh with `skillpack store manifest`.
- **Private/restricted skills:** `~/Skills/private` (kept out of `shared` so the
  IP boundary guard never blocks a public sync).

## The consumption model

```
~/Skills/shared/<skill>/          <- the one real copy
~/.claude/skills/<skill>   ->  ~/Skills/shared/<skill>   (symlink)
~/.codex/skills/<skill>    ->  ~/Skills/shared/<skill>   (symlink)
<any harness>/skills/<skill> -> ~/Skills/shared/<skill>  (symlink)
```

Edit a skill once in the store; every harness sees it immediately. No drift,
no fan-out copies to keep in sync.

## Onboarding a built-in harness

19 harnesses are recognized out of the box (Claude Code, Codex, Aider, Gemini
CLI, Cursor, Windsurf, Zed, Continue, Copilot, …). Just sync:

```bash
skillpack store sync --dry-run   # preview the reference fanout
skillpack store sync             # establish the symlinks
```

Each built-in harness's skills directory is relocatable via its env override
(e.g. `CLAUDE_SKILLS_DIR`, `CODEX_SKILLS_DIR`).

## Onboarding a NEW harness — no recompile

Declare it in `SKILLPACK_CUSTOM_AGENTS` (comma-separated `name=dir` pairs).
The variable is read by both the CLI and the server, so set it wherever the
server runs:

```bash
export SKILLPACK_CUSTOM_AGENTS="MyHarness=/path/to/myharness/skills,Other=/opt/other/skills"
skillpack store sync --only-agent MyHarness   # or a full sync
```

The harness now receives a symlink per skill, pointing at the canonical store —
verified end-to-end: a fresh harness gained all store skills as references with
zero copies and zero code change.

## Consuming the registry from a foreign (non-SkillPack) harness

Any tool can consume the registry by reading the manifest and symlinking:

```bash
root=$(jq -r .canonicalRoot ~/Skills/shared/.skillpack-registry.json)
for name in $(jq -r '.skills[].name' ~/Skills/shared/.skillpack-registry.json); do
  ln -sfn "$root/$name" "$MY_SKILLS_DIR/$name"
done
```

The manifest also carries each skill's `description`, lifecycle `status`, and
`hasSkillMd`, so a harness can filter (e.g. skip `retired` skills) before
linking.

## Reconciling the current state

- **Duplicate physical copies** (a skill that exists as a real dir in an agent
  path *and* in the store) should become symlinks: `skillpack store sync`
  replaces them with references.
- **Unique physical skills** (present in an agent path but not the store) are
  pulled into the store first: `skillpack store migrate`, then `sync` fans them
  back out as references.
- **Collisions / duplicates** across the fleet: `skillpack audit` (duplicate
  names) and `skillpack audit collisions` (descriptions that compete for the
  same triggers).
