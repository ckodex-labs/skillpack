# Testing Dimension

**Weight:** 10%  
**Dimension ID:** `Testing`

## Purpose

Validates that a skill includes automated tests, evaluation harnesses, and continuous-integration workflows.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| `tests/` directory present | 40 | Yes |
| `evals/` directory or `evals/runner.yaml` present | 30 | No |
| CI workflow (`.github/workflows/test.yml`, `ci.yml`, or `eval.yml`) | 30 | No |

### Quality Gates

- Missing `tests/` → Warning-level issue
- Missing `evals/runner.yaml` → Note-level issue (covered under Evals & HITL)

## File Checks

- `tests/`
- `evals/`
- `evals/runner.yaml`
- `.github/workflows/test.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/eval.yml`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Testing")'
```
