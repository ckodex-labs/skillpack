//! SARIF Reporter

use serde_json::json;
use skillpack_domain::{Assessment, ReportError, ReportGenerator};

pub struct SarifReporter;

impl ReportGenerator for SarifReporter {
    fn generate(&self, assessment: &Assessment) -> Result<String, ReportError> {
        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "SkillPack",
                        "version": "1.0.0",
                        "informationUri": "https://github.com/ckodex/skillpack"
                    }
                },
                "results": assessment.issues.iter().map(|issue| {
                    json!({
                        "ruleId": format!("{:?}", issue.dimension),
                        "level": match issue.severity {
                            skillpack_domain::Severity::Error => "error",
                            skillpack_domain::Severity::Warning => "warning",
                            skillpack_domain::Severity::Note => "note",
                        },
                        "message": { "text": &issue.message },
                        "locations": issue.file.as_ref().map(|f| vec![json!({
                            "physicalLocation": {
                                "artifactLocation": { "uri": f },
                                "region": { "startLine": issue.line.unwrap_or(1) }
                            }
                        })]).unwrap_or_default()
                    })
                }).collect::<Vec<_>>()
            }]
        });

        serde_json::to_string_pretty(&sarif)
            .map_err(|e| ReportError::GenerationError(e.to_string()))
    }

    fn format(&self) -> &'static str {
        "sarif"
    }
}
