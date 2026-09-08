# OCI Artifact Distribution

This document describes how to publish and consume SkillPack bundles via OCI registries.

## Media Types

| Type              | Media Type                                |
| ----------------- | ----------------------------------------- |
| CNSB Bundle       | `application/vnd.ckodex.cnsb.v1+json`     |
| CNAAB Bundle      | `application/vnd.ckodex.cnaab.v1+json`    |
| Evidence Envelope | `application/vnd.ckodex.evidence.v1+json` |
| Policy Bundle     | `application/vnd.ckodex.policy.v1+json`   |

## Publishing

### Using Make

```bash
# Publish to default registry (ghcr.io/ckodex)
make publish

# Publish to custom registry
REGISTRY=my-registry.io NAMESPACE=myorg make publish

# Sign with Sigstore
make sign

# Generate SBOM
make sbom
```

### Using CLI

```bash
# Push skill bundle
skillpack oci push \
  --registry ghcr.io/ckodex/my-skill \
  --tag v1.0.0 \
  --file skill-bundle.cnsb.json

# Pull and validate
skillpack oci pull ghcr.io/ckodex/my-skill:v1.0.0
skillpack validate downloaded-bundle.json --schema cnsb
```

## Verification

### Signature Verification

```bash
# Verify Sigstore signature
cosign verify ghcr.io/ckodex/my-skill:v1.0.0

# Full supply chain verification
make verify-supply-chain
```

### SBOM Consumption

```bash
# Generate SBOM for your build
cargo sbom > sbom.cyclonedx.json

# Verify SBOM
cosign verify-attestation \
  --type cyclonedx \
  ghcr.io/ckodex/my-skill:v1.0.0
```

## GitHub Actions Integration

```yaml
- name: Publish to GHCR
  env:
    REGISTRY: ghcr.io
    NAMESPACE: ${{ github.repository_owner }}
  run: |
    echo ${{ secrets.GITHUB_TOKEN }} | docker login ghcr.io -u ${{ github.actor }} --password-stdin
    make publish
    make sign
```

## Artifact Structure

```
skill-bundle.cnsb.json      # CNSB v1 manifest
├── metadata
│   ├── name
│   ├── urn
│   └── version
├── skills[]
│   ├── id
│   ├── entry
│   ├── galMin
│   └── galMax
└── evidence                # Optional attestations
```
