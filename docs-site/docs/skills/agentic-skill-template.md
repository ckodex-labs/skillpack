# Agentic Skill Template

> A minimal, working Claude Code skill that exercises every pattern from the agentic authoring guide.

## Overview

This template demonstrates:

1. **Anchor-term description** — SKILL.md frontmatter description is dense in the words a future agent will search on.
2. **Progressive disclosure** — SKILL.md body ≤500 lines; protocol depth lives in `references/`; behaviour lives in `scripts/`.
3. **Deterministic JSON envelope** — Every script returns the same shape: `{ok, data | error, metadata:{command, version}}`.
4. **Dual-channel exit-code contract** — stdout/exit 0 = success or retriable; stderr/exit 1 = fatal config.
5. **Single orchestrator + script library** — One agent file routes; many small scripts do the work.
6. **Routing table** — Anchor phrases the user is likely to say, each resolving to a real script.
7. **Safety defaults** — `--dry-run` on write paths, `mask_token` everywhere, `require_https` on every URL.
8. **Two-manifest plugin layout** — `.claude-plugin/plugin.json` for install, `catalog.json` for marketplace.
9. **Three test gates** — Structure, scripts (envelope contract), agent.
10. **2026 Claude Code surfaces** — `${CLAUDE_SKILL_DIR}` for path resolution; `allowed-tools` ready to enforce.

## Directory Layout

```text
agentic-skill-template/
├── .claude-plugin/
│   └── plugin.json                       # install manifest
├── catalog.json                          # marketplace index
├── README.md                             # adaptation guide
├── agents/
│   └── agentic-engineer.md               # orchestrator template
└── skills/
    └── agentic-skill-template/
        ├── SKILL.md                      # the skill
        ├── scripts/
        │   ├── hello.sh                  # envelope-returning demo
        │   └── lib/
        │       └── common.sh             # emit_ok / emit_error / die
        ├── references/
        │   └── EXAMPLE.md                # anatomy of a reference doc
        └── tests/
            ├── structure.test.sh         # layout + frontmatter caps
            ├── scripts.test.sh           # shellcheck + envelope regression
            └── agent.test.sh             # routing coverage + safety
```

## Downloads

- **Source tree**: [agentic-skill-template/](/skills/agentic-skill-template/)
- **Tarball**: [agentic-skill-template.tar.gz](/skills/agentic-skill-template.tar.gz)

## Cross-references

- [Authoring Guide](/specs/skills-dossier)
- [Skill Spec v1.1](/specs/skill-spec-v1.1)
- [Schema Index](/specs/schema-index)
