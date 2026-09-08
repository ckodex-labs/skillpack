---
name: ckodex-skill-tools
description: Scaffold new CKODEX v1.1 skills from a blank slate, and migrate existing Agent Skills v1 skills to v1.1 in place. Use whenever the user asks to "create a new skill", "start a skill", "bootstrap a skill", "upgrade a skill to v1.1", "add synopsis/ontology/runtime to a skill", "migrate v1 → v1.1", or asks how to write a CKODEX manifest. Refuses to operate on directories that already contain conflicting files unless --force is set; emits explicit dry-run summaries before any write.
license: Apache-2.0
metadata:
  version: "0.1.0"
  author: ckodex-labs
---

# ckodex-skill-tools — Scaffolder + Migrator

Two stdlib-only Python CLIs that produce verifier-passing skill bundles
on a target machine.

| Tool             | Purpose                                                                              |
|------------------|--------------------------------------------------------------------------------------|
| `scaffold.py`    | Create a fresh CKODEX v1.1 skill tree at a target directory                          |
| `migrate.py`     | Upgrade an existing Agent Skills v1 skill (just `SKILL.md`, or `skill.json` already) to v1.1 |

Both tools default to **dry-run**. They emit a summary of every file
they would create or modify, then exit. Pass `--apply` (or `--write`)
to actually change disk. The `--force` flag overrides conflict refusal.

## When to use which

- **Empty target directory** or **just a name in mind** → `scaffold.py`
- **Existing `SKILL.md` (and possibly `skill.json`)** → `migrate.py`

## Quick start

```bash
# Scaffold a new skill
python3 scripts/scaffold.py path/to/my-new-skill \
  --name "data-extraction" \
  --description "Extracts structured data from PDFs, spreadsheets, and HTML." \
  --apply

# Migrate an existing v1 skill in place
python3 scripts/migrate.py path/to/existing-skill --apply

# Both have --help with full option lists
python3 scripts/scaffold.py --help
python3 scripts/migrate.py --help
```

## What `scaffold.py` produces

A complete CKODEX v1.1 skill tree, ready to ship:

```
<target>/
├── SKILL.md                 frontmatter + body skeleton
├── skill.json               v1.1 manifest with synopsis + ontology + runtime stubs
├── references/
│   └── usage.md             starter reference
├── scripts/
│   └── validate.sh          7-section bundle self-test
└── MANIFEST.json             sha256 ledger of every file
```

Running `bash scripts/validate.sh` inside the produced tree returns
`STATUS: 0F / 0W — PASS`. That is the acceptance criterion.

## What `migrate.py` does

For an existing skill rooted at `<target>/`:

1. Parse `SKILL.md` frontmatter (name, description, license, optional metadata).
2. Read existing `skill.json` if present; treat unknown blocks as opaque pass-through.
3. Compose a synopsis from the body (faithful compression; never invented detail). The synopsis is bounded to 2048 chars.
4. Bump `apiVersion` to `ckodex.org/skill/v1.1`.
5. Add empty stub blocks for `ontology` and `runtime` that the author SHOULD fill in (annotated with `MIGRATION:` markers in the file so they're easy to find).
6. Add a `MANIFEST.json` sha256 ledger if absent.
7. Add `scripts/validate.sh` if absent.
8. NEVER overwrite a user-authored `synopsis`, `ontology`, or `runtime` already in `skill.json` unless `--force` is set.

## Hard refusals (both tools)

- Target directory does not exist (without `--init`).
- Target is not a directory.
- Target contains a `SKILL.md` already and `scaffold.py` was used without `--force`.
- Skill `name` violates the Agent Skills v1 regex (`^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$`, ≤64 chars).
- `description` exceeds 1024 chars.
- `synopsis` exceeds 2048 chars.
- Existing `skill.json` is unparseable JSON (without `--force` to overwrite).

## References

- `references/scaffolder.md` — full scaffold output contract and template variables.
- `references/migrator.md` — v1 → v1.1 migration ruleset, idempotency guarantees, conflict handling.
- `references/conformance.md` — what `validate.sh` checks; how it returns 0F/0W; how to extend it.
