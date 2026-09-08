# Identity & Manifest Dimension

**Weight:** 11%  
**Dimension ID:** `IdentityManifest`

## Purpose

Ensures a skill has a discoverable, versioned identity and a valid machine-readable manifest. This is the foundation for all other dimensions — without identity, a skill cannot be referenced, indexed, or trusted.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| SKILL.md with valid YAML frontmatter containing `name` and `version` | 50 | Yes |
| CNSB `.cnsb.json` with valid `apiVersion`, `kind`, and `metadata.{name,urn,version}` | 50 | Yes |

### Partial Credit

- Missing `description` in frontmatter → Warning, -0 pts but flagged
- Missing `urn` in CNSB metadata → Error, blocks full 50 pts
- Malformed JSON in CNSB → Error, blocks full 50 pts

## File Checks

- `SKILL.md` — Anthropic-style agentskills frontmatter
- `*.cnsb.json` — CKODEX Native Skill Bundle manifest

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="IdentityManifest")'
```
