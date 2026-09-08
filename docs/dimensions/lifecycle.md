# Lifecycle Dimension

**Weight:** 10%  
**Dimension ID:** `Lifecycle`

## Purpose

Assesses whether a skill defines standard lifecycle hooks (install, uninstall, upgrade, verify, pack) either through a CNSB bundle, Makefile targets, package.json scripts, or Cargo.toml.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| CNSB lifecycle `install` hook | 20 | No |
| CNSB lifecycle `uninstall` hook | 20 | No |
| CNSB lifecycle `upgrade` hook | 15 | No |
| CNSB lifecycle `verify` hook | 15 | No |
| CNSB lifecycle `pack` hook | 10 | No |
| CNSB lifecycle `unpack` hook | 10 | No |
| CNSB lifecycle `preInstall` / `postInstall` | 5 each | No |
| Lock file (`skill.lock`, `package-lock.json`, `Cargo.lock`) | 10 / 5 | No |
| `.well-known/skills.json` discovery file | 5 | No |

### Fallback Scoring (no CNSB)

- `Makefile` targets: `install:` (15), `uninstall:` (15), `upgrade:` (10), `verify:` (10), `pack:`/`build:` (10)
- `package.json` scripts: `install`/`postinstall` (15), `build` (10), `test` (10)
- `Cargo.toml` → 20 pts (built-in lifecycle)

### Quality Gates

- No lifecycle defined at all → Warning-level issue
- Missing lock file → Note-level issue

## File Checks

- `*.cnsb.json` (`lifecycle` section)
- `Makefile`
- `package.json`
- `Cargo.toml`
- `skill.lock`
- `.well-known/skills.json`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Lifecycle")'
```
