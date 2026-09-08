---
name: example-legacy-skill
description: A pre-existing Agent Skills v1 skill that ships only SKILL.md. Used by the ckodex-skill-tools self-test to exercise the migration path.
license: MIT
---

# example-legacy-skill

> A small legacy skill bundled with `ckodex-skill-tools` to demonstrate
> the v1 → v1.1 migration. This skill does nothing real — it is a fixture.

## When to use this skill

Use this when the migrator self-test wants to exercise the
v1-to-v1.1 upgrade path on a realistic input shape: a single
SKILL.md with frontmatter, no companion skill.json, no references
directory, no scripts directory.

## How it works

The skill has no behavior. The body exists so the migrator has prose
to compose a synopsis from. The migrator should produce a synopsis
between roughly 500 and 1500 characters that summarizes this body and
mentions the absence of inputs and outputs.

## Inputs

None.

## Outputs

None.

## Edge cases

The skill is a fixture and refuses every real request. The migrator
should preserve this behavior; it only changes the manifest, not the
SKILL.md body.

## References

None.
