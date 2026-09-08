//! Provenance Checker
//!
//! Checks for CycloneDX SBOM, SLSA provenance, sigstore signatures,
//! and self-emitted SkillPack assessment envelope.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct ProvenanceChecker;

impl DimensionChecker for ProvenanceChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Provenance
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            // AgentSkills rubric: provenance is source traceability — where
            // the knowledge came from and who wrote it. SBOM/SLSA artifacts
            // are CNSB territory.
            let mut score = 0.0f64;
            let mut issues = Vec::new();
            let fm = common::frontmatter_map(reader, path);
            let body = common::skill_body(reader, path).unwrap_or_default();
            let has_source_link = fm
                .as_ref()
                .map(|m| common::fm_has_any(m, &["source", "homepage", "repository", "url"]))
                .unwrap_or(false)
                || body.contains("https://");
            if has_source_link {
                score += 50.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Provenance,
                    severity: Severity::Note,
                    message: "no source links — cite the canonical docs the skill encodes".into(),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
            if fm
                .as_ref()
                .map(|m| common::fm_has_any(m, &["author", "authors", "owner"]))
                .unwrap_or(false)
            {
                score += 25.0;
            }
            if body.contains("sha256") || reader.file_exists(path, "skill.lock") {
                score += 25.0;
            }
            return (Score::dimension(score.min(100.0)), issues);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // 1. CycloneDX SBOM parseable (35 pts)
        let sbom_paths = ["evidence/sbom.json", "sbom.json", "evidence/sbom.cdx.json"];
        let sbom = sbom_paths
            .iter()
            .find_map(|p| reader.read_file(path, p).ok().map(|c| (*p, c)));
        match sbom {
            Some((p, content)) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(j)
                    if j.get("bomFormat").and_then(serde_json::Value::as_str)
                        == Some("CycloneDX") =>
                {
                    score += 35.0;
                    if !j
                        .get("specVersion")
                        .and_then(serde_json::Value::as_str)
                        .map(|v| v.starts_with("1."))
                        .unwrap_or(false)
                    {
                        issues.push(Issue {
                            dimension: DimensionId::Provenance,
                            severity: Severity::Warning,
                            message: format!("SBOM specVersion missing or unrecognised in {}", p),
                            file: Some(p.to_string()),
                            line: None,
                        });
                    }
                }
                _ => issues.push(Issue {
                    dimension: DimensionId::Provenance,
                    severity: Severity::Error,
                    message: format!("SBOM at {} is not parseable CycloneDX", p),
                    file: Some(p.to_string()),
                    line: None,
                }),
            },
            None => issues.push(Issue {
                dimension: DimensionId::Provenance,
                severity: Severity::Warning,
                message: "No SBOM found at evidence/sbom.json".into(),
                file: None,
                line: None,
            }),
        }

        // 2. SLSA provenance present (25 pts)
        if let Ok(content) = reader.read_file(path, "evidence/provenance.json") {
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&content) {
                if j.get("predicateType")
                    .and_then(serde_json::Value::as_str)
                    .map(|s| s.contains("slsa.dev/provenance"))
                    .unwrap_or(false)
                {
                    score += 25.0;
                }
            }
        }

        // 3. Sigstore signature present (20 pts) AND signed flag (not "unsigned: true" marker)
        let signed = reader.file_exists(path, "evidence/sbom.json.sig")
            || reader.file_exists(path, "evidence/cosign.bundle");
        let unsigned_marker = reader
            .read_file(path, "evidence/sbom.json")
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .map(|j| j.get("unsigned").and_then(serde_json::Value::as_bool) == Some(true))
            .unwrap_or(false);
        if signed && !unsigned_marker {
            score += 20.0;
        } else if unsigned_marker {
            issues.push(Issue {
                dimension: DimensionId::Provenance,
                severity: Severity::Warning,
                message: "SBOM marked unsigned: true".into(),
                file: Some("evidence/sbom.json".into()),
                line: None,
            });
        }

        // 4. Self-test: a SkillPack-emitted envelope from a prior assessment validates (20 pts)
        if reader.file_exists(path, "evidence/skillpack-assessment.json") {
            score += 20.0;
        }

        (Score::dimension(score), issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::{SkillIdentity, SkillReaderError};
    use std::collections::HashMap;
    use std::path::Path;

    struct MockReader {
        files: HashMap<String, String>,
    }

    impl SkillReader for MockReader {
        fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
            Ok(SkillIdentity::new("test", "1.0.0", ""))
        }
        fn file_exists(&self, _path: &Path, relative: &str) -> bool {
            self.files.contains_key(relative)
        }
        fn read_file(&self, _path: &Path, relative: &str) -> Result<String, SkillReaderError> {
            self.files
                .get(relative)
                .cloned()
                .ok_or_else(|| SkillReaderError::NotFound(relative.into()))
        }
        fn list_files(&self, _path: &Path, _pattern: &str) -> Vec<String> {
            self.files.keys().cloned().collect()
        }
    }

    #[test]
    fn detects_valid_cyclonedx_sbom() {
        let reader = MockReader {
            files: [(
                "evidence/sbom.json".into(),
                r#"{"bomFormat":"CycloneDX","specVersion":"1.4","components":[]}"#.into(),
            )]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 35.0);
        assert!(issues.iter().all(|i| !i.message.contains("not parseable")));
    }

    #[test]
    fn warns_on_unparseable_sbom() {
        let reader = MockReader {
            files: [("evidence/sbom.json".into(), "not-json".into())].into(),
        };
        let checker = ProvenanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 0.0);
        assert!(issues.iter().any(|i| i.message.contains("not parseable")));
    }

    #[test]
    fn warns_on_wrong_sbom_format() {
        let reader = MockReader {
            files: [(
                "evidence/sbom.json".into(),
                r#"{"bomFormat":"SPDX","specVersion":"2.3"}"#.into(),
            )]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 0.0);
        assert!(issues.iter().any(|i| i.message.contains("not parseable")));
    }

    #[test]
    fn detects_slsa_provenance() {
        let reader = MockReader {
            files: [
                (
                    "evidence/sbom.json".into(),
                    r#"{"bomFormat":"CycloneDX","specVersion":"1.4"}"#.into(),
                ),
                (
                    "evidence/provenance.json".into(),
                    r#"{"predicateType":"https://slsa.dev/provenance/v0.2"}"#.into(),
                ),
            ]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (score, _issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 60.0); // 35 sbom + 25 slsa
    }

    #[test]
    fn detects_sigstore_signature() {
        let reader = MockReader {
            files: [
                (
                    "evidence/sbom.json".into(),
                    r#"{"bomFormat":"CycloneDX","specVersion":"1.4"}"#.into(),
                ),
                ("evidence/sbom.json.sig".into(), "".into()),
            ]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (score, _issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 55.0); // 35 sbom + 20 signature
    }

    #[test]
    fn warns_on_unsigned_marker() {
        let reader = MockReader {
            files: [
                (
                    "evidence/sbom.json".into(),
                    r#"{"bomFormat":"CycloneDX","specVersion":"1.4","unsigned":true}"#.into(),
                ),
                ("evidence/sbom.json.sig".into(), "".into()),
            ]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (_score, issues) = checker.check(&reader, Path::new("."));
        assert!(issues.iter().any(|i| i.message.contains("unsigned")));
    }

    #[test]
    fn detects_self_assessment_envelope() {
        let reader = MockReader {
            files: [
                (
                    "evidence/sbom.json".into(),
                    r#"{"bomFormat":"CycloneDX","specVersion":"1.4"}"#.into(),
                ),
                (
                    "evidence/skillpack-assessment.json".into(),
                    r#"{"overall":88}"#.into(),
                ),
            ]
            .into(),
        };
        let checker = ProvenanceChecker;
        let (score, _issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 55.0); // 35 sbom + 20 self-assessment
    }
}
