# Security

## Supply Chain Security

SkillPack implements comprehensive supply chain security.

### SLSA Level 3

All releases include cryptographic provenance:

```bash
# Verify provenance
slsa-verifier verify-artifact \
  --provenance-path provenance.json \
  --source-uri github.com/ckodex-labs/skillpack \
  skillpack-linux-amd64
```

### Sigstore Signing

Artifacts are signed keyless via Sigstore:

```bash
# Verify signature
cosign verify \
  --certificate-identity-regexp=".*github.com/ckodex-labs/skillpack.*" \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/ckodex/skillpack:1.0.0
```

### SBOM

CycloneDX 1.5 software bill of materials:

```bash
# Generate SBOM
syft ghcr.io/ckodex/skillpack:1.0.0 -o cyclonedx-json > sbom.cdx.json

# Verify included SBOM
cosign verify-attestation \
  --type cyclonedx \
  ghcr.io/ckodex/skillpack:1.0.0
```

## Threat Model

### STRIDE Analysis

| Category            | Threat                        | Mitigation            |
| ------------------- | ----------------------------- | --------------------- |
| **Spoofing**        | Malicious skill impersonation | Sigstore verification |
| **Tampering**       | Modified templates            | Immutable OCI layers  |
| **Repudiation**     | Deny publishing               | SLSA provenance chain |
| **Info Disclosure** | Credential leakage            | OIDC keyless signing  |
| **DoS**             | Large skill packs             | File size limits      |
| **Elevation**       | Path traversal                | Canonicalization      |

## Security Controls

### Assessment Security

- Input validation on all paths
- Sandbox execution for template evaluation
- Rate limiting on gRPC API
- mTLS for service-to-service

### Data Protection

- No secrets in templates (checked by Security dimension)
- Credentials via environment/secret manager
- Cache encrypted at rest (optional)

## Reporting Vulnerabilities

Report security issues to: **security@ckodex.dev**

Response SLA:

| Phase              | Time               |
| ------------------ | ------------------ |
| Acknowledgment     | 24 hours           |
| Initial assessment | 72 hours           |
| Fix timeline       | Severity-dependent |
