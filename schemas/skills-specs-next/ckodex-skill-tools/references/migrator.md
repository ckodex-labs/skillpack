# migrator reference

`scripts/migrate.py` upgrades an existing Agent Skills v1 skill to
CKODEX v1.1 in place. This document covers the migration ruleset,
idempotency guarantees, and conflict-handling policy.

## Migration ruleset

Given a target directory containing at minimum a `SKILL.md` (and
optionally a `skill.json`), the migrator:

1. **Parses `SKILL.md` frontmatter.** Flat key:value pairs. The `name`, `description`, and `license` fields are read. Nested mappings and block scalars are NOT supported (Agent Skills v1 documents flat frontmatter).
2. **Loads existing `skill.json` if present.** Treated as the source of truth for fields it already carries. Unparseable JSON triggers a refusal unless `--force` is set.
3. **Composes a bounded synopsis** from the body (everything after the second `---` line):
   - Strips code fences and headers; collapses whitespace.
   - Linearizes to ~1500 chars (target) with a 2048-char hard ceiling.
   - If the body is empty, falls back to a `name`+`description`-derived stub.
4. **Bumps `apiVersion`** to `ckodex.org/skill/v1.1`.
5. **Sets `kind`** to `"Skill"` if absent.
6. **Sets `metadata.name` and `metadata.description`** from the frontmatter (these are the authoritative source — `skill.json` values are overridden to keep the two in sync).
7. **Sets `metadata.license`** from the frontmatter if `skill.json` does not already carry one.
8. **Adds `metadata.synopsis`** — unless one already exists and `--force` was NOT passed, in which case the existing value is preserved.
9. **Adds `entrypoints.skillMd = "SKILL.md"`** if absent.
10. **Adds stub `ontology` block** with empty arrays and `@context` pinned to `https://ckodex.org/skill-context/v1`. Author MUST hand-edit to populate. Preserved if author already wrote one (unless `--force`).
11. **Adds stub `runtime` block** with safe defaults (`implicitInvocation: true`, `minTier: "L1"`, `maxTier: "L3"`, `l4xAllowed: false`). Preserved if author already wrote one (unless `--force`).
12. **Writes `scripts/validate.sh`** if absent. Same 7-section template as the scaffolder.
13. **Generates `MANIFEST.json`** — sha256 ledger of every file in the tree (excluding `__pycache__/*` and `*.pyc/*.pyo`).

## Idempotency

Running `migrate.py --apply` on an already-migrated tree:

- `metadata.synopsis` — re-composed from the same `SKILL.md` body. If the body has not changed, the synopsis bytes are identical. If the author hand-edited the synopsis and the body has not changed, the author's edits are preserved (without `--force`).
- `ontology` / `runtime` — preserved as-is. The stubs the migrator added on the first run are kept; any author additions to the stubs are kept.
- `MANIFEST.json` — regenerated. Digests update if anything else changed; otherwise byte-identical.
- `scripts/validate.sh` — kept as-is if present.

The end state is stable: a second `--apply` run on the same tree never breaks the validator. The bundle self-test verifies this.

## Conflict handling

| Condition                                                  | Without `--force`               | With `--force`           |
|------------------------------------------------------------|----------------------------------|--------------------------|
| Existing `skill.json` is unparseable JSON                  | refuse                           | overwrite                |
| `metadata.synopsis` already populated                      | preserve                         | overwrite with composed  |
| `ontology` block already present                           | preserve                         | overwrite with stub      |
| `runtime` block already present                            | preserve                         | overwrite with stub      |

The `--force` flag is destructive. The recommended workflow is:

```bash
# Step 1: dry-run, read every "PRESERVED" note carefully
python3 scripts/migrate.py path/to/skill

# Step 2: apply without --force, accepting the preserves
python3 scripts/migrate.py path/to/skill --apply

# Step 3: verify
cd path/to/skill && bash scripts/validate.sh
```

If after the migration you want to regenerate a stub block, hand-delete
that block from `skill.json` and re-run `--apply`. The migrator will
re-add the stub. This is safer than `--force`.

## What does NOT get migrated

The migrator deliberately does NOT:

- Edit `SKILL.md` content. The body is the author's source of truth.
- Invent ontology subjects/produces/consumes/triggers. These require domain knowledge the migrator does not have.
- Set `runtime.l4xAllowed: true`. Script execution is high-risk; the safer default is `false`. Author flips it consciously.
- Add SLSA / signatures / `supplyChain` fields. Those belong to the build pipeline, not the migrator.

## Output: what to hand-edit after migration

Look for `MIGRATION:` annotations in the migration summary. Specifically:

1. **`metadata.synopsis`** — composed from the body; almost always benefits from a hand-trim that emphasizes triggers and scope boundaries.
2. **`ontology.subjects`** — start with topical nouns the SKILL.md body mentions.
3. **`ontology.produces`** — proof types or artifact classes the skill emits.
4. **`ontology.consumes`** — proof types or artifact classes the skill ingests.
5. **`ontology.triggers`** — lexical phrases a harness should match against. Include 3–8 phrases.
6. **`runtime.maxTier`** — set to `L4R` if your skill reads bundled resources, `L4X` if it executes scripts.
7. **`runtime.l4xAllowed`** — set to `true` only if `maxTier` is `L4X`.

## Exit codes

- `0` — success (dry-run summary printed, or migration applied)
- `1` — refusal (unparseable JSON, invalid frontmatter, validation failure)
- `2` — unexpected error
