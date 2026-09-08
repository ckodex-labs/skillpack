# Provenance Dimension

**Weight:** 14%  
**Dimension ID:** `Provenance`

## Purpose

Assesses whether a skill provides a machine-readable software bill of materials (SBOM), supply-chain provenance, cryptographic signatures, and a self-assessment envelope from a prior SkillPack run.

## Scoring Criteria

| Checkpoint | Points | Required |
|------------|--------|----------|
| Parseable CycloneDX SBOM (`evidence/sbom.json`, `sbom.json`, or `evidence/sbom.cdx.json`) | 35 | No |
| SLSA provenance present (`evidence/provenance.json` with `predicateType` containing `slsa.dev`) | 25 | No |
| Sigstore signature present (`evidence/sbom.json.sig` or `evidence/cosign.bundle`) and not marked unsigned | 20 | No |
| Prior SkillPack assessment envelope (`evidence/skillpack-assessment.json`) | 20 | No |

### Notes

- SBOM must have `bomFormat: "CycloneDX"` and a recognised `specVersion`
- Unsigned marker (`unsigned: true` in SBOM) downgrades signature score to 0
- Missing SBOM generates a Warning issue but does not block other checkpoints

## File Checks

- `evidence/sbom.json`
- `sbom.json`
- `evidence/sbom.cdx.json`
- `evidence/provenance.json`
- `evidence/sbom.json.sig`
- `evidence/cosign.bundle`
- `evidence/skillpack-assessment.json`

## Tooling

```bash
skillpack grade <path> --output=json | jq '.dimensions[] | select(.id=="Provenance")'
```
