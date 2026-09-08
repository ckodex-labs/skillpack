# Getting Started

## Installation

### From Source

```bash
git clone https://github.com/ckodex/skillpack.git
cd skillpack
cargo build --release
```

### From OCI Registry

```bash
# Pull and run
docker run ghcr.io/ckodex/skillpack:latest check .
```

## Quick Start

### Assess a Skill Pack

```bash
# Local skill
skillpack check ./my-skill

# Remote skill (OCI)
skillpack check oci://ghcr.io/org/skill:v1

# Remote skill (Git)
skillpack check git://github.com/org/skill@main
```

### Check Grade

```bash
skillpack grade ./my-skill --minimum A
```

### Generate Report

```bash
skillpack report ./my-skill --format sarif --output report.sarif
```

## Skill Structure

SkillPack expects skills to follow the Agent Skills specification:

```
my-skill/
├── SKILL.md          # Required - skill metadata
├── templates/        # Handlebars templates
├── security/
│   ├── SECURITY.md
│   └── threat-model.yaml
├── evidence/
│   ├── provenance.json
│   └── sbom.json
└── capacities/
    └── mcp/          # MCP server (optional)
```

## Next Steps

- [Architecture](/guide/architecture) - Understand the CKODEX design
- [Dimensions](/guide/dimensions) - Learn about quality dimensions
- [Remote Sources](/guide/remote-sources) - Pull skills from registries
