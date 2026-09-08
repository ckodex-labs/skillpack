# agentic-skill-template

A minimal, working Claude Code skill that exercises every pattern from
`docs/agentic-authoring-patterns.md`. Clone it, rename it, replace
`hello.sh` with your domain's surface, and you have a deterministic,
envelope-returning, signature-verified, dry-run-defaulted agentic skill that
passes three CI gates.

This README is not the skill; it is the **adaptation guide**. The skill
itself is `skills/agentic-skill-template/SKILL.md`. The orchestrator agent
that drives it is `agents/agentic-engineer.md`.

---

## What the template demonstrates

1. **Anchor-term description.** SKILL.md frontmatter description is dense in
   the words a future agent will search on — never marketing prose.
2. **Progressive disclosure.** SKILL.md body ≤500 lines; protocol depth
   lives in `references/`; behaviour lives in `scripts/`.
3. **Deterministic JSON envelope.** Every script returns the same shape:
   `{ok, data | error, metadata:{command, version}}`.
4. **Dual-channel exit-code contract.** stdout/exit 0 = success or
   retriable; stderr/exit 1 = fatal config — agents read stdout first.
5. **Single orchestrator + script library.** One agent file routes; many
   small scripts do the work. No business logic in the agent.
6. **Routing table that maps user signals → scripts.** Anchor phrases the
   user is likely to say, each resolving to a real script that exists.
7. **Safety defaults.** `--dry-run` on write paths, `mask_token` everywhere,
   `require_https` on every URL, OIDC for CI.
8. **Two-manifest plugin layout.** `.claude-plugin/plugin.json` for install,
   `catalog.json` for marketplace — never interchangeable.
9. **Three test gates.** Structure, scripts (envelope contract), agent.
10. **2026 Claude Code surfaces.** `${CLAUDE_SKILL_DIR}` for path resolution;
    `allowed-tools` ready to enforce in production frontmatter.

---

## Directory layout

```
agentic-skill-template/
├── .claude-plugin/
│   └── plugin.json                       # install manifest
├── catalog.json                          # marketplace index
├── README.md                             # this file
├── agents/
│   └── agentic-engineer.md               # orchestrator template
└── skills/
    └── agentic-skill-template/
        ├── SKILL.md                      # the skill, progressive disclosure root
        ├── scripts/
        │   ├── hello.sh                  # envelope-returning demo
        │   └── lib/
        │       └── common.sh             # emit_ok / emit_error / die / mask_token / require_*
        ├── references/
        │   └── EXAMPLE.md                # anatomy of a reference doc
        └── tests/
            ├── structure.test.sh         # layout + frontmatter caps
            ├── scripts.test.sh           # shellcheck + envelope regression
            └── agent.test.sh             # routing coverage + safety anchors
```

---

## Running the three gates

```bash
cd skills/agentic-skill-template
bash tests/structure.test.sh   # required files, manifests, frontmatter caps
bash tests/scripts.test.sh     # shellcheck, --help, envelope regression
bash tests/agent.test.sh       # agent frontmatter + routing + safety
```

All three must exit 0. `scripts.test.sh` requires `jq` on PATH and (warns,
does not fail, if missing) `shellcheck`.

---

## Adapting in seven steps

Adapt by replacing template content with your domain's content; do not
restructure the layout. The three test gates enforce the shape.

### 1. Rename the skill

```bash
SKILL_OLD=agentic-skill-template
SKILL_NEW=<your-skill-name>           # ^[a-z0-9]+(-[a-z0-9]+)*$, ≤64 chars

mv skills/${SKILL_OLD} skills/${SKILL_NEW}
```

Update the name in all three manifests:

- `.claude-plugin/plugin.json` → `name`
- `catalog.json` → `skills[0].name`, `skills[0].path`
- `skills/${SKILL_NEW}/SKILL.md` → frontmatter `name`

The structure gate's test #6 (`name in SKILL.md matches parent directory`)
fails if any of these drift.

### 2. Rewrite the SKILL.md description

Lead with the anchor terms a future agent will search on. Keep under 1024
chars (schema cap) and ideally under 700 chars (leaves headroom for the 1536
combined name + description runtime budget). Zero marketing vocab.

Example anchor density:
- "Adapter for X protocol over HTTPS with bearer or OIDC token exchange."
- "Returns the dual-channel JSON envelope; success and retriable on stdout exit 0; fatal config on stderr exit 1."

### 3. Replace `hello.sh` with your domain's scripts

Keep the shape:

```bash
#!/usr/bin/env bash
set -euo pipefail
. "${BASH_SOURCE%/*}/lib/common.sh"
CKODEX_COMMAND=<command-name>

# parse flags; --help support is mandatory

require_cmd jq
require_env <ADAPTER>_URL
require_https "${<ADAPTER>_URL}"

# do work, then exactly one of:
emit_ok   '{...data...}'                                # stdout exit 0
emit_error CODE "message" true                          # retriable stdout exit 0
die       CODE "message"                                # fatal stderr exit 1
```

Every script must call `emit_ok`, `emit_error`, or `die` exactly once on
every code path. `scripts.test.sh` enforces the envelope shape.

### 4. Update the routing table

Open `agents/agentic-engineer.md`, locate `## Routing table`, replace rows.
Every row must resolve to a script or reference that exists; `agent.test.sh`
section 3 fails CI otherwise.

### 5. Add references on demand

For each protocol surface (API endpoint catalog, RBAC matrix, CLI flag
table, fail-mode taxonomy), add one file under `references/`. Use
`references/EXAMPLE.md` as the shape. Cross-reference from SKILL.md and the
agent's routing table.

### 6. Tighten safety constraints to your threat surface

The template ships read-only `hello.sh`; safety anchors are stubs. When you
add an authenticated boundary:

- Set `DRY_RUN=1` as the default in every write-path script.
- Require `--no-dry-run` to flip.
- Pass tokens via `<ADAPTER>_TOKEN` env; never on CLI.
- Mask in every diagnostic: `mask_token "${<ADAPTER>_TOKEN}"`.
- Reject `http://` with `require_https`.
- Verify signatures before import / apply / promote; never `--skip-verify`.

### 7. Run the gates locally, then in CI

```bash
bash tests/structure.test.sh && \
bash tests/scripts.test.sh   && \
bash tests/agent.test.sh
```

Wire the same three commands into your CI pipeline. Treat any non-zero exit
as a release blocker.

---

## Cross-references

- **`docs/agentic-authoring-patterns.md`** — the 10-pattern distillation
  this template instantiates.
- **`docs/authoring-guide.md`** — the CNSB-bundle distribution flavor.
  A skill can satisfy both contracts; the bundle is distribution and the
  template is runtime contract.
- **`schemas/agentskills.schema.json`** — the schema the template's
  frontmatter is checked against.
- **`docs/oci-distribution.md`** — how to publish the skill as an OCI
  artifact for cross-tenant distribution.

---

## What this template is not

- Not a substitute for a real domain skill. The routing is a hello-world
  smoke test, not a useful workflow.
- Not a production-ready adapter. There is no network call, no auth, no
  rate-limit handling — those are the work you do when you adapt.
- Not opinionated about your language. Scripts can be bash, Python, Node,
  Go, or anything that emits the JSON envelope. The bash variant is
  shipped because it has the smallest runtime surface.

The value is the **shape**: the contract, the layout, the gates. Adapt
content; preserve shape.
