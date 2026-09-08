# Security Policy

## Supported Versions

| Version        | Supported             |
| -------------- | --------------------- |
| 1.0.0-beta.x   | Preview (best-effort) |
| < 1.0.0-beta.1 | No                    |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by emailing:

```text
security@ckodex.org
```

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include:

- Type of issue (e.g., buffer overflow, privilege escalation, injection)
- Full paths of source file(s) related to the issue
- Location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce
- Step-by-step instructions to reproduce
- Proof-of-concept or exploit code (if possible)
- Impact of the issue

## Security Measures

SkillPack implements the following security measures:

### Supply Chain Security

> **Alpha status:** The following controls are planned and partially implemented. The automated release pipeline (Dagger) is currently a stub; manual `make sign` / `make sbom` / `make provenance` targets exist but are not yet enforced in CI.

- **(Planned)** All releases signed with Sigstore/cosign
- **(Planned)** SBOM (Software Bill of Materials) generated for each release
- **(Planned)** SLSA Level 3 provenance attestations
- Dependency vulnerability scanning via `cargo audit` in CI (active)

### Schema Validation

- Strict JSON Schema validation for all inputs
- No arbitrary code execution from skill bundles
- Sandboxed evaluation of skill metadata

### Code Quality

- All changes require signed commits
- Automated security scanning in CI
- Regular dependency updates

## Disclosure Policy

- We aim to acknowledge receipt within 48 hours
- We aim to confirm the vulnerability within 7 days
- We aim to provide a fix within 30 days for critical issues
- We will coordinate disclosure timing with you

## Recognition

We appreciate responsible disclosure and will acknowledge security researchers who report valid vulnerabilities (with your permission).
