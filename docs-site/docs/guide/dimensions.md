# Dimensions

SkillPack assesses skills across 8 quality dimensions.

## Scoring

**Total Score:** 150 points max

| Component    | Points                    |
| ------------ | ------------------------- |
| Base Score   | 100 (weighted dimensions) |
| Bonus Points | 50 (excellence criteria)  |

## Dimension Weights

| Dimension     | Weight | Max Points |
| ------------- | ------ | ---------- |
| Security      | 25%    | 25         |
| Provenance    | 20%    | 20         |
| Structure     | 15%    | 15         |
| Governance    | 15%    | 15         |
| Templates     | 10%    | 10         |
| MCP           | 5%     | 5          |
| Documentation | 5%     | 5          |
| Accessibility | 5%     | 5          |

## Dimension Details

### Structure (15%)

Required skill structure:

- `SKILL.md` present and valid
- CNSB JSON bundle (optional but scored)
- `templates/` directory
- `security/` directory
- `evidence/` directory

### Security (25%)

Supply chain and code security:

- `security/SECURITY.md` present
- `security/threat-model.yaml` present
- No secrets in templates
- Security scanning config

### Provenance (20%)

Artifact traceability:

- `evidence/provenance.json` (SLSA)
- `evidence/sbom.json` (CycloneDX)
- CI pipeline attestations
- Signed releases

### Governance (15%)

Organizational standards:

- Threat model documented
- Security policy defined
- Governance procedures

### Templates (10%)

Handlebars templates quality:

- Templates present
- Valid syntax
- Documented usage

### MCP (5%)

Model Context Protocol support:

- MCP server in `capacities/mcp/`
- Valid server implementation

### Documentation (5%)

User documentation:

- `SKILL.md` complete
- `docs/USAGE.md` present
- `README.md` present

### Accessibility (5%)

Inclusive design:

- Accessibility guide present
- Templates include a11y patterns

## Bonus Points

| Criterion           | Points |
| ------------------- | ------ |
| SLSA Level 3+       | +10    |
| Sigstore Signing    | +10    |
| Dagger Pipeline     | +10    |
| STRIDE Threat Model | +10    |
| MCP Server          | +10    |

## Grades

| Grade | Score   | Description |
| ----- | ------- | ----------- |
| S+    | 145-150 | Exceptional |
| S     | 135-144 | Excellent   |
| A     | 120-134 | Very Good   |
| B     | 100-119 | Good        |
| C     | 80-99   | Acceptable  |
| D     | 60-79   | Needs Work  |
| F     | 0-59    | Failing     |
