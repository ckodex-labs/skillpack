# STRIDE Threat Model - SkillPack

version: "1.1"
last_updated: "2026-06-04"

## System Overview

SkillPack is a skill quality assessment tool operating in trust boundary between:
- User's local filesystem (skill packs)
- OCI registries (artifact verification)
- CI/CD pipelines (quality gates)

## Assets

| Asset              | Sensitivity | Description                     |
| ------------------ | ----------- | ------------------------------- |
| Assessment Reports | Medium      | Skill quality scores and issues |
| Skill Packs        | Low         | User's skill implementations    |
| OCI Credentials    | High        | Registry authentication         |
| Signing Keys       | Critical    | Sigstore OIDC tokens            |

## Threats and Mitigations

### Spoofing

| ID  | Threat                        | Mitigation            |
| --- | ----------------------------- | --------------------- |
| S1  | Malicious skill impersonation | Sigstore verification |
| S2  | Fake assessment results       | SLSA provenance       |

### Tampering

| ID  | Threat             | Mitigation                 |
| --- | ------------------ | -------------------------- |
| T1  | Modified templates | Immutable OCI layers       |
| T2  | Altered scores     | Cryptographic attestations |

### Repudiation

| ID  | Threat                   | Mitigation            |
| --- | ------------------------ | --------------------- |
| R1  | Deny publishing          | SLSA provenance chain |
| R2  | Alter assessment history | in-toto attestations  |

### Information Disclosure

| ID  | Threat               | Mitigation            |
| --- | -------------------- | --------------------- |
| I1  | Credential leakage   | OIDC keyless signing  |
| I2  | Source code exposure | Private registry auth |

### Denial of Service

| ID  | Threat               | Mitigation       |
| --- | -------------------- | ---------------- |
| D1  | Large skill packs    | File size limits |
| D2  | Recursive structures | Depth limits     |

### Elevation of Privilege

| ID  | Threat             | Mitigation           |
| --- | ------------------ | -------------------- |
| E1  | Path traversal     | Canonicalization     |
| E2  | Template injection | Sandboxed evaluation |

## Controls Summary

> **Alpha note:** The following controls are on the roadmap. The Dagger CI pipeline stages for signing, SBOM, provenance, and OPA are currently stubbed; `Makefile` targets exist for manual execution.

- **(Planned)** Sigstore keyless signing
- **(Planned)** SLSA Level 3 provenance
- **(Planned)** CycloneDX SBOM generation
- **(Planned)** in-toto attestations
- **(Planned)** OPA policy gates

## Alpha Risks

- Default-open auth when `SKILLPACK_API_TOKEN` is unset (mitigated by loopback binding).
- Path traversal: canonicalization is claimed but not visibly enforced at gRPC/HTTP/MCP entry points.
- No CORS or rate limiting on the HTTP layer.
