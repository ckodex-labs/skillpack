//! JSON Reporter

use skillpack_domain::{Assessment, ReportError, ReportGenerator};

pub struct JsonReporter;

impl ReportGenerator for JsonReporter {
    fn generate(&self, assessment: &Assessment) -> Result<String, ReportError> {
        serde_json::to_string_pretty(assessment)
            .map_err(|e| ReportError::GenerationError(e.to_string()))
    }

    fn format(&self) -> &'static str {
        "json"
    }
}
