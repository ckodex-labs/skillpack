# Evals & HITL Dimension

**Weight:** 9%  
**Dimension ID:** `EvalsHitl`

## Purpose

Assesses whether a skill includes an evaluation harness and human-in-the-loop (HITL) approval gates.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| `evals/runner.yaml` present | 40 | Yes |
| `evals/runner.yaml` contains `hitl:` or `approver_roles:` configuration | 30 | No |
| CI workflow for evals (`.github/workflows/eval.yml` or `hitl.yml`) | 30 | No |

### Quality Gates

- Missing `evals/runner.yaml` → Warning-level issue, 0 pts for primary checkpoint
- Runner present but missing HITL config → Note-level issue

## File Checks

- `evals/runner.yaml`
- `.github/workflows/eval.yml`
- `.github/workflows/hitl.yml`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="EvalsHitl")'
```
