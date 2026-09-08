# scaffolder reference

`scripts/scaffold.py` produces a complete CKODEX v1.1 skill tree at a
target directory. This document covers the output contract, template
variables, validation rules, and refusals.

## Output tree

```
<target>/
├── SKILL.md                YAML frontmatter + Markdown body skeleton
├── skill.json              v1.1 manifest with synopsis + ontology + runtime stubs
├── references/
│   └── usage.md            starter reference
├── scripts/
│   └── validate.sh         7-section bundle self-test
└── MANIFEST.json           sha256 ledger of every file (generated last)
```

## Template variables

The four template files in `assets/templates/` use Python `string.Template`
syntax (`$name`). The scaffolder substitutes:

| Variable          | Source                            | Required |
|-------------------|-----------------------------------|----------|
| `name`            | `--name` CLI flag                 | yes      |
| `description`     | `--description` CLI flag          | yes      |
| `license`         | `--license` (default `Apache-2.0`)| no       |
| `author`          | `--author` (default `unknown`)    | no       |
| `version`         | `--version` (default `0.1.0`)     | no       |
| `synopsis`        | `--synopsis` (auto-generated if absent) | no  |
| `tools_version`   | hardcoded `TOOLS_VERSION`         | no       |

## Auto-generated synopsis

If `--synopsis` is not supplied, the scaffolder composes a minimal
synopsis from `name` and `description`. The auto-generated synopsis is
intentionally generic — author SHOULD hand-edit before shipping.

Pattern:
> When the user mentions `{name}` or related topics: this skill `{description}`.
> Triggers: explicit mention of `{name}`; topical keywords from the description.
> Inputs and outputs: TODO — fill in after authoring SKILL.md body.
> NOT a substitute for general-purpose tools; refuses out-of-scope requests.

The auto-generated synopsis is always ≤ 2048 chars by construction.

## Validation rules

Before any file is written, the scaffolder validates:

| Rule                                           | Failure mode  |
|------------------------------------------------|----------------|
| `name` matches `^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$` | refuse |
| `len(name) <= 64`                              | refuse |
| `1 <= len(description) <= 1024`                | refuse |
| `1 <= len(synopsis) <= 2048` (if provided)     | refuse |

These match `skill.v1.1.schema.json` exactly. A scaffolded skill that
passes the scaffolder's pre-check is guaranteed to pass schema validation.

## Refusal cases

| Condition                                              | Resolution                |
|--------------------------------------------------------|---------------------------|
| Target directory does not exist                        | Pass `--init` to create   |
| Target exists but is not a directory                   | Choose a different path   |
| `SKILL.md` already exists at target                    | Pass `--force` or use `migrate.py` |
| Invalid name / oversized description / oversized synopsis | Fix the input          |

## Dry-run by default

The scaffolder never writes files unless `--apply` (or `--write`) is
passed. The dry-run output lists every file that would be created and
its expected size. This is the supported workflow:

```bash
# Step 1: see what would happen
python3 scripts/scaffold.py /tmp/my-skill --init \
    --name my-skill --description "..."

# Step 2: write
python3 scripts/scaffold.py /tmp/my-skill --init \
    --name my-skill --description "..." --apply

# Step 3: verify
cd /tmp/my-skill && bash scripts/validate.sh
# expect: STATUS: 0F / 0W — PASS
```

## Exit codes

- `0` — success (dry-run summary printed, or files written)
- `1` — validation failure or conflict (e.g. existing SKILL.md without --force)
- `2` — unexpected error (caller should treat as bug)

## What you do next

The scaffolded SKILL.md body and `synopsis` are skeletons. Before the
skill is useful you MUST:

1. Hand-edit `SKILL.md` to describe the actual procedure, inputs, outputs, edge cases, and refusals.
2. Re-write `metadata.synopsis` in `skill.json` to faithfully compress the new body.
3. Populate `ontology.subjects`, `ontology.produces`, `ontology.consumes`, and `ontology.triggers` with topical and lexical anchors a harness can match against.
4. Tighten `runtime.maxTier` and `runtime.l4xAllowed` to match the actual risk profile.
5. Re-run `bash scripts/validate.sh` and confirm still `0F / 0W`.
