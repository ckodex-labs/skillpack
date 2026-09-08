# known-a

Reference A-grade fixture covering full SkillPack canonical 9-dimension surface.

## Features

- CKODEX-compliant skill structure
- Full lifecycle hooks (install, uninstall, upgrade, verify, pack, unpack)
- Security threat model included
- Automated testing scaffolding
- Continuous integration ready
- SLSA Level 3 provenance
- CycloneDX SBOM

## Quick Start

```bash
make install
make verify
skillpack check --min-score 80
```

## Project Structure

```
known-a/
├── SKILL.md              # Skill manifest
├── known-a.cnsb.json     # CNSB bundle manifest
├── README.md             # This file
├── LICENSE               # Apache-2.0
├── CODE_OF_CONDUCT.md    # Community standards
├── SECURITY.md           # Security policy
├── GOVERNANCE.md         # Decision-making process
├── CODEOWNERS            # Repository ownership
├── CHANGELOG.md          # Version history
├── Makefile              # Build & lifecycle automation
├── tests/                # Test suite
├── examples/             # Usage examples
├── security/             # Security artifacts
├── evidence/             # Provenance & SBOM
├── evals/                # Evaluation harness
└── .github/workflows/    # CI/CD
```

## Contributing

See [GOVERNANCE.md](security/GOVERNANCE.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Security

Please report security issues following the process in [SECURITY.md](security/SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE) for full text.
