# Evidence Bundle — SkillPack Alpha

This directory contains evidence artifacts produced for the `1.0.0-alpha.1` readiness review and remediation.

| File | Description | Date |
|------|-------------|------|
| `ALPHA-READINESS.md` | Full readiness report (10 areas, scorecard, go/no-go) | 2026-06-04 |
| `risk-acceptance.md` | Signed risk acceptance for default-open auth in alpha | 2026-06-04 |
| `sbom.cyclonedx.json` | CycloneDX SBOM generated via `cargo cyclonedx` | 2026-06-04 |
| `cargo-audit.json` | `cargo audit --json` output (1 advisory: RUSTSEC-2023-0071 / rsa) | 2026-06-04 |
| `test-summary.txt` | `cargo test --workspace` results (165 passed, 5 ignored, 0 failed) | 2026-06-04 |

## Test Summary

```
165 passed, 0 failed, 5 ignored across all crates
```

The 5 ignored tests are integration tests in `crates/skillpack-api/tests/canonical_service_test.rs` that require a running `skillpack-server` on `127.0.0.1:50052`.

## Security Advisory

`cargo-audit.json` contains one active advisory:
- **RUSTSEC-2023-0071** (`rsa` crate): Marvin Attack — potential key recovery through timing sidechannels.  
  Impact: Low for SkillPack alpha (local-only use, no network RSA operations).  
  Status: No patch available upstream; acceptable alpha risk.
