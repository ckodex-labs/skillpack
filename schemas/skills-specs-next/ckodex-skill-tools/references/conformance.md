# conformance reference

Every skill produced by `scaffold.py` or migrated by `migrate.py` ships
with the same `scripts/validate.sh` self-test. This document documents
the seven sections, what they enforce, how to read the output, and how
to extend it.

## The seven sections

| §  | Check                                | Failure mode                                |
|----|--------------------------------------|---------------------------------------------|
| §1 | Required files present               | `SKILL.md`, `skill.json`, `MANIFEST.json`, `references/usage.md`, `scripts/validate.sh` all must exist |
| §2 | JSON parse                           | `skill.json` and `MANIFEST.json` must parse with stdlib `json` |
| §3 | Manifest required fields             | `apiVersion`, `kind`, `metadata.name`, `metadata.description`, `entrypoints.skillMd` must be non-empty |
| §4 | SKILL.md frontmatter consistency     | `name` and `description` in the frontmatter must equal the values in `skill.json` |
| §5 | Name regex + length limits           | `name` matches `^(?!-)(?!.*--)[a-z0-9]+(?:-[a-z0-9]+)*$`, ≤64 chars; `description` ∈ [1,1024]; `synopsis` ∈ [1,2048] |
| §6 | Declared resources exist on disk     | Every path listed in `resources.scripts`, `resources.references`, `resources.assets` must exist |
| §7 | MANIFEST.json sha256 ledger          | Every file listed in `MANIFEST.json` must have a matching sha256 on disk (excluding `__pycache__`, `*.pyc`, `*.pyo`) |

## Status line

The validator emits one of three status lines:

- `STATUS: 0F / 0W — PASS` — clean pass; ship it
- `STATUS: 0F / NW — PASS with warnings` — passes but flagged; review the warnings
- `STATUS: NF / NW — FAIL` — at least one failure; do not ship

A bundle should achieve `0F / 0W` before being published.

## Excluded files

The validator deliberately excludes:

- `__pycache__/` directories (Python bytecode caches; CI machines vary)
- `*.pyc`, `*.pyo` files (same reason)
- The `MANIFEST.json` file itself (it can't ledger itself)

These exclusions live in both the MANIFEST.json scan path and the
validate.sh §7 ledger check. Keep them in sync if you extend either.

## Extending the validator

Add a section to your skill's `validate.sh`:

```bash
section "My extra check"
if some-condition; then
  ok "passes"
else
  fail "explains the failure"
fi
```

The `section`, `ok`, `fail`, `warn` shell helpers are defined near the
top of the template. They increment the `$SECTIONS`, `$FAIL`, `$WARN`
counters that drive the final status line. Do NOT bypass these helpers
— the final status arithmetic depends on them.

## Common failure patterns

| Failure                                       | Likely cause                                          |
|-----------------------------------------------|--------------------------------------------------------|
| `§2 skill.json parse error`                   | Trailing comma, comment, or stray character in JSON   |
| `§3 metadata.name: missing or empty`          | Frontmatter parse failed; check YAML indentation      |
| `§4 frontmatter name X != skill.json Y`       | Author renamed in one file but not the other          |
| `§5 name violates v1 regex`                   | Uppercase, leading/trailing hyphen, or consecutive hyphens |
| `§6 references/foo.md declared but not on disk` | Stub referenced but file not created                |
| `§7 path: digest mismatch`                    | File edited without regenerating MANIFEST.json        |

For §7 mismatches, regenerate the ledger:

```python
# regen.py — drop into your skill root
import hashlib, json, os, pathlib
root = pathlib.Path('.')
EXCLUDE_DIRS = {"__pycache__", ".pytest_cache", ".mypy_cache"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo"}
files = []
for p in sorted(root.rglob('*')):
    if not p.is_file(): continue
    if p.name == 'MANIFEST.json': continue
    if p.suffix in EXCLUDE_SUFFIXES: continue
    if any(part in EXCLUDE_DIRS for part in p.parts): continue
    files.append({
        'path': str(p.relative_to(root)).replace(os.sep, '/'),
        'sha256': 'sha256:' + hashlib.sha256(p.read_bytes()).hexdigest(),
        'bytes': p.stat().st_size,
    })
m = {'schemaVersion': 'ckodex.org/pack-manifest/v1',
     'pack': root.resolve().name, 'framework': 'CKODEX v16.0',
     'files': files, 'fileCount': len(files)}
open('MANIFEST.json', 'w').write(json.dumps(m, indent=2) + '\n')
```

Or just re-run `migrate.py --apply` on the skill — it regenerates the
ledger as part of the migration pipeline.

## Verify-by-extract

For shipping `.tgz` / `.zip` archives, the canonical post-package check is:

```bash
mkdir verify && cd verify
tar -xzf ../my-skill.tgz
cd my-skill
bash scripts/validate.sh
# expect: STATUS: 0F / 0W — PASS
```

If the archive passes inside a fresh extracted tempdir, it ships. If
not, it does not.
