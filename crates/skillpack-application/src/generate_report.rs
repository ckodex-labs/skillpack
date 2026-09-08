//! Generate Report Use Case

use anyhow::Result;
use skillpack_domain::{Assessment, ReportGenerator};

/// Metadata about the assessment run surfaced alongside the report.
/// Used by callers to communicate coverage gaps (e.g. stub dimensions).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AssessmentMeta {
    /// Names of dimensions whose checkers are stubs (no content analysis yet).
    pub stub_dimensions: Vec<String>,
}

/// Input for report generation, including stub dimension names surfaced from assessment.
pub struct GenerateReportRequest {
    pub assessment: Assessment,
    pub format: ReportFormat,
    /// Stub dimension names collected from `AssessSkillResponse::stub_dimensions`.
    pub stub_dimensions: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Json,
    Sarif,
    Markdown,
}

/// Response with generated report
pub struct GenerateReportResponse {
    pub content: String,
    pub format: ReportFormat,
    /// Assessment metadata including stub dimension coverage.
    pub meta: AssessmentMeta,
}

/// Generate report use case
pub fn generate_report(
    generators: &[Box<dyn ReportGenerator>],
    request: GenerateReportRequest,
) -> Result<GenerateReportResponse> {
    let format_str = match request.format {
        ReportFormat::Json => "json",
        ReportFormat::Sarif => "sarif",
        ReportFormat::Markdown => "markdown",
    };

    let generator = generators
        .iter()
        .find(|g| g.format() == format_str)
        .ok_or_else(|| anyhow::anyhow!("No generator for format: {}", format_str))?;

    let content = generator.generate(&request.assessment)?;

    let meta = AssessmentMeta {
        stub_dimensions: request.stub_dimensions,
    };

    Ok(GenerateReportResponse {
        content,
        format: request.format,
        meta,
    })
}
