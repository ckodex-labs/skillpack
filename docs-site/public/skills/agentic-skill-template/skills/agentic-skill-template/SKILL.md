---
name: agentic-skill-template
description: Template for Claude Code Agent Skills — demonstrates the deterministic JSON envelope contract, the dual-channel exit-code split (stdout exit 0 for success or retriable error; stderr exit 1 for fatal config error), anchor-term routing, token masking, HTTPS-only enforcement, --dry-run default on write paths, and the three-test-gate CI surface (structure + scripts + agent). Adapt by renaming the skill and replacing the hello script with domain-specific scripts that follow the same envelope contract.
license: MIT
compatibility: Requires bash, jq, and shellcheck. Designed for Claude Code; reusable in any agentic runtime that can invoke shell scripts and parse JSON.
metadata:
  author: ckodex-skill-pack
  version: 0.1.0
  status: template
---

# Agentic Skill Template

A minimum-viable agentic skill that demonstrates the patterns documented in
[`docs/agentic-authoring-patterns.md`](../../../docs/agentic-authoring-patterns.md).

Every script returns the same JSON envelope so an agent can chain calls
without parsing prose:

```
success: {"ok":true,  "data": <object>, "metadata":{"command":"<name>","version":"0.1.0"}}
failure: {"ok":false, "error":{"code":"<CODE>","message":"<...>","retriable":<bool>}, "metadata":{"command":"<name>","version":"0.1.0"}}
```

## Channel + exit-code contract

| Channel | Exit | When                                                                  | Agent action                              |
| ------- | ---- | --------------------------------------------------------------------- | ----------------------------------------- |
| stdout  | 0    | success, or operational error with `retriable` flag (HTTP, timeout)   | parse `.data` / `.error`; chain or retry  |
| stderr  | 1    | fatal config error (bad `--flag`, missing env, file/signature absent) | surface envelope to user; **never retry** |

Read stdout first. If empty and exit ≠ 0, read stderr — that envelope is a
programming error in the call, not an operational failure.

## When to invoke

| User signal                                                  | Script                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------- |
| "say hello" / "smoke test" / "verify the envelope contract"  | `${CLAUDE_SKILL_DIR}/scripts/hello.sh --name <name>`    |
| "what does this template show?"                              | Read this `SKILL.md` and the routing table              |
| "show me the envelope shape"                                 | `${CLAUDE_SKILL_DIR}/scripts/hello.sh --name demo`      |
| Anything outside the template's domain                       | Do not invoke. Route to the appropriate domain skill.   |

## When NOT to invoke

- Real domain work — this is a template, not a production skill.
- Anything requiring network calls, secrets, or external services.
- As a substitute for a real domain skill.

## Safety defaults (release-blocking)

- **HTTPS-only at the URL boundary.** `require_https` in `scripts/lib/common.sh`
  rejects `http://`. The template ships no URL flags, but the helper is
  exported so adapters wire it in immediately.
- **Token masking in every error path.** `mask_token` produces `****<last4>`;
  raw tokens never appear in stdout, stderr, or logs.
- **`--dry-run` default on write-path scripts.** `hello.sh` is read-only, so
  it has no dry-run flag. When adapting, every write script must default
  `DRY_RUN=1` and require `--no-dry-run` to flip it.
- **Signature verification mandatory before any "import" step.** Not exercised
  by the template, but the pattern is: never `--skip-verify` in prod.

## Adapting the template

1. Rename the parent directory: `mv agentic-skill-template <your-skill-name>`.
2. Update `name:` in `SKILL.md`, `name` in `.claude-plugin/plugin.json`, and
   `skills[0].name` + `skills[0].path` in `catalog.json` to match.
3. Replace `scripts/hello.sh` with your domain scripts. Each must source
   `scripts/lib/common.sh` and use `emit_ok` / `emit_error` / `die`.
4. Update the routing table above and in `agents/agentic-engineer.md`.
5. Run the three test gates: `bash tests/structure.test.sh`,
   `bash tests/scripts.test.sh`, `bash tests/agent.test.sh`.
6. Adjust the agent's `description` and `tools:` for the domain.
7. Document every chain in `references/` — never compose by parsing prose.

## References

- [`references/EXAMPLE.md`](references/EXAMPLE.md) — anatomy of a reference doc.
- [`../../../docs/agentic-authoring-patterns.md`](../../../docs/agentic-authoring-patterns.md)
  — the ten patterns this template exercises.
- [`../../../schemas/agentskills.schema.json`](../../../schemas/agentskills.schema.json)
  — frontmatter schema (`description` max 1024 chars).
- [`../../../schemas/agentskills-directory.schema.json`](../../../schemas/agentskills-directory.schema.json)
  — directory layout schema (`SKILL.md` recommended <500 lines).
