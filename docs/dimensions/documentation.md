# Documentation Dimension

**Weight:** 11%  
**Dimension ID:** `Documentation`

## Purpose

Ensures a skill is accompanied by sufficient human-readable documentation: a primary descriptor file, usage guides, a changelog, and a code of conduct.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| `SKILL.md` present | 40 | Yes |
| `docs/USAGE.md` present | 20 | No |
| `README.md` present | 20 | No |
| `CODE_OF_CONDUCT.md` present | 10 | No |
| `CHANGELOG.md` present | 10 | No |

### Quality Gates

- SKILL.md shorter than 200 bytes → Note-level issue flagged
- Missing SKILL.md → Error-level issue, 0 pts for primary checkpoint

## File Checks

- `SKILL.md`
- `docs/USAGE.md`
- `README.md`
- `CODE_OF_CONDUCT.md`
- `CHANGELOG.md`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Documentation")'
```
