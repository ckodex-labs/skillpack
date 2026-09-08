# Security Dimension

**Weight:** 18%  
**Dimension ID:** `Security`

## Purpose

Validates that a skill declares its security posture, carries no leaked secrets, uses approved ASC classifications, and does not contain dangerous lifecycle commands.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| License declared in SKILL.md frontmatter or `LICENSE` file present | 25 | Yes |
| Valid OIS-ASC pattern in CNSB metadata annotations | 25 | Yes |
| No secrets detected in source files (AWS keys, GitHub tokens, high-entropy strings) | 25 | Yes |
| No dangerous lifecycle commands (`curl \| sh`, `eval`, `rm -rf /`) | 25 | Yes |

### Penalties

- Secret detected → Error per occurrence, -25 pts total if any found
- Dangerous command in CNSB lifecycle → Error per command, -25 pts total if any found
- Invalid ASC pattern → Error, 0 pts for ASC checkpoint

## File Checks

- `SKILL.md` frontmatter (`license:`)
- `LICENSE`
- `*.cnsb.json` (`metadata.annotations.asc`)
- All `*.rs`, `*.py`, `*.js`, `*.ts`, `*.yaml`, `*.yml`, `*.json`, `*.md`, `*.tf`, `*.sh`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Security")'
```
