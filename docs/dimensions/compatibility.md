# Compatibility Dimension

**Weight:** 8%  
**Dimension ID:** `Compatibility`

## Purpose

Ensures a skill declares its runtime requirements, platform constraints, and integration modalities (e.g., MCP server).

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| `SKILL.md` contains `compatibility:`, `runtime:`, or `platforms:` front-matter | 50 | No |
| MCP server modality (`capacities/mcp/server` or `mcp.json`) | 30 | No |
| Dependency manifest (`requirements.txt`, `package.json`, or `Cargo.toml`) | 20 | No |

### Quality Gates

- SKILL.md present but missing compatibility front-matter → Note-level issue
- Missing dependency manifest → no penalty, just 0 pts for that checkpoint

## File Checks

- `SKILL.md` (front-matter scan)
- `capacities/mcp/server`
- `mcp.json`
- `requirements.txt`
- `package.json`
- `Cargo.toml`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Compatibility")'
```
