# Governance Dimension

**Weight:** 9%  
**Dimension ID:** `Governance`

## Purpose

Validates governance, security process, and community health documentation.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| `security/threat-model.yaml` present | 25 | No |
| `security/SECURITY.md` present | 20 | No |
| `security/GOVERNANCE.md` present | 20 | No |
| `CODEOWNERS` or `.github/CODEOWNERS` present | 15 | No |
| `CODE_OF_CONDUCT.md` present | 10 | No |
| `LICENSE` present | 10 | No |

### Quality Gates

- Missing threat model → Warning-level issue
- Missing SECURITY.md → Warning-level issue
- Missing CODEOWNERS → no penalty, just 0 pts

## File Checks

- `security/threat-model.yaml`
- `security/SECURITY.md`
- `security/GOVERNANCE.md`
- `CODEOWNERS`
- `.github/CODEOWNERS`
- `CODE_OF_CONDUCT.md`
- `LICENSE`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Governance")'
```
