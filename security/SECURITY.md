# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x     | :white_check_mark: |

## Reporting a Vulnerability

Report vulnerabilities to: security@ckodex.dev

**Response SLA:**
- Acknowledgment: 24 hours
- Initial assessment: 72 hours
- Fix timeline: Severity-dependent

## Security Measures

- **SLSA Level 3**: Cryptographic build provenance
- **Sigstore**: Keyless artifact signing
- **SBOM**: CycloneDX 1.5 software bill of materials
- **STRIDE**: Threat model documented
- **OPA**: Policy-based quality gates

## Supply Chain

All releases signed with Sigstore keyless:
```bash
cosign verify ghcr.io/ckodex/skillpack:1.0.0
```
