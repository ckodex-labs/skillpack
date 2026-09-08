# Skills Authoring Guide

> How to create production-grade AI agent skills for the CKODEX ecosystem

## Quick Start

```bash
# Create a new skill
mkdir my-skill && cd my-skill

# Initialize required files
touch SKILL.md skill.cnsb.json skill.lock Makefile
```

---

## Required Files

| File              | Purpose                        |
| ----------------- | ------------------------------ |
| `SKILL.md`        | Skill documentation (required) |
| `skill.cnsb.json` | Skill bundle manifest          |
| `skill.lock`      | Pinned dependency versions     |
| `Makefile`        | Lifecycle operations           |

---

## 1. SKILL.md Template

```markdown
---
name: my-skill
version: 1.0.0
description: Brief description of what this skill does
---

# My Skill

## Description
Detailed explanation of the skill's purpose and capabilities.

## Usage
\`\`\`bash
# Example invocation
my-skill --input "example"
\`\`\`

## Inputs
| Name  | Type   | Required | Description          |
| ----- | ------ | -------- | -------------------- |
| input | string | Yes      | The input to process |

## Outputs
| Name   | Type   | Description       |
| ------ | ------ | ----------------- |
| result | object | Processing result |

## Examples
- See `examples/` directory for usage patterns
```

---

## 2. Skill Bundle (skill.cnsb.json)

```json
{
  "apiVersion": "cnsb.ckodex.org/v1",
  "kind": "SkillBundle",
  "metadata": {
    "name": "my-skill",
    "version": "1.0.0",
    "description": "What this skill does",
    "labels": {
      "category": "development",
      "language": "typescript"
    }
  },
  "skills": [
    {
      "id": "main",
      "entry": "dist/index.js",
      "description": "Main skill entry point",
      "galMin": 0,
      "galMax": 3,
      "inputs": [
        { "name": "input", "type": "string", "required": true }
      ],
      "outputs": [
        { "name": "result", "type": "object" }
      ]
    }
  ],
  "dependencies": [
    "urn:ckodex:skill:common-utils:^1.0.0"
  ],
  "lifecycle": {
    "install": {
      "command": "npm install",
      "timeout": "5m"
    },
    "verify": {
      "command": "npm test",
      "retries": 2
    },
    "uninstall": {
      "command": "rm -rf node_modules dist"
    }
  }
}
```

---

## 3. Lock File (skill.lock)

Generated automatically by `skillpack lock`:

```json
{
  "lockVersion": 1,
  "metadata": {
    "generated": "2026-02-01T22:37:55Z",
    "generator": "skillpack@1.0.0"
  },
  "dependencies": {
    "urn:ckodex:skill:common-utils:1.2.3": {
      "version": "1.2.3",
      "resolved": "ghcr.io/ckodex/skills/common-utils@sha256:abc123",
      "integrity": "sha256-abc123def456..."
    }
  }
}
```

---

## 4. Makefile Targets

```makefile
.PHONY: build install verify uninstall publish

# Build the skill
build:
 npm run build

# Install dependencies
install:
 npm ci

# Run tests and verification
verify:
 npm test
 skillpack check .

# Remove installed skill
uninstall:
 rm -rf node_modules dist

# Publish to OCI registry
publish: verify
 skillpack oci push --registry ghcr.io/myorg/skills --tag $(VERSION)
 cosign sign ghcr.io/myorg/skills/my-skill:$(VERSION)
```

---

## 5. Directory Structure

```
my-skill/
├── SKILL.md                 # Documentation (required)
├── skill.cnsb.json          # Bundle manifest
├── skill.lock               # Dependency lock
├── Makefile                 # Lifecycle targets
├── package.json             # Node.js dependencies (if applicable)
├── src/
│   └── index.ts             # Skill implementation
├── dist/                    # Built artifacts
├── examples/
│   └── basic.json           # Usage examples
├── security/
│   ├── threat-model.yaml    # STRIDE analysis
│   └── SECURITY.md          # Security policy
├── evidence/
│   ├── provenance.json      # Build provenance
│   └── sbom.json            # Software bill of materials
└── .github/
    └── workflows/
        └── publish-skill.yml # CI/CD automation
```

---

## 6. Governance Autonomy Levels (GAL)

| Level | Description               | Use Case              |
| ----- | ------------------------- | --------------------- |
| 0     | Full human control        | Dangerous operations  |
| 1     | Human approval required   | Sensitive data access |
| 2     | Human notification        | Normal operations     |
| 3     | Autonomous with logging   | Routine tasks         |
| 4     | Autonomous, limited scope | Background processing |
| 5     | Fully autonomous          | Read-only operations  |

Set `galMin` and `galMax` based on your skill's risk profile.

---

## 7. Quality Checklist

Before publishing, ensure:

- [ ] `SKILL.md` is complete with usage examples
- [ ] `skill.cnsb.json` validates against schema
- [ ] `skill.lock` is generated and committed
- [ ] Makefile has `install`, `verify`, `uninstall` targets
- [ ] Security documentation exists
- [ ] Tests pass (`skillpack check .` returns grade ≥ B)

Run assessment:

```bash
skillpack check .
skillpack grade .
skillpack report . --format markdown
```

---

## 8. Publishing

```bash
# 1. Build and verify
make verify

# 2. Generate lock file
skillpack lock .

# 3. Push to OCI registry
make publish

# 4. Sign with Sigstore
cosign sign ghcr.io/myorg/skills/my-skill:1.0.0
```

---

## Resources

- [CNSB Schema Reference](./schemas/cnsb/v1/cnsb-standalone.schema.json)
- [Skill Lock Schema](./schemas/cnsb/v1/skill-lock.schema.json)
- [OCI Distribution Guide](./docs/oci-distribution.md)
- [SkillPack CLI Reference](./docs/cli-reference.md)
