//! Assess Bundle Use Case

use anyhow::Result;
use skillpack_domain::{
    BundleAssessment, BundleReader, DimensionChecker, SkillAssessmentSummary, SkillReader,
};
use std::path::Path;

/// Request for bundle assessment
pub struct AssessBundleRequest {
    pub bundle_path: String,
    pub min_score: Option<f64>,
}

/// Response from bundle assessment
pub struct AssessBundleResponse {
    pub assessment: BundleAssessment,
    pub meets_minimum: bool,
}

/// Assess bundle use case
pub fn assess_bundle<R, B>(
    _reader: &R,
    bundle_reader: &B,
    _checkers: &[Box<dyn DimensionChecker>],
    request: AssessBundleRequest,
) -> Result<AssessBundleResponse>
where
    R: SkillReader,
    B: BundleReader,
{
    let bundle_path = Path::new(&request.bundle_path);

    // Find and read bundle
    let bundles = bundle_reader.list_bundles(bundle_path);
    if bundles.is_empty() {
        return Err(anyhow::anyhow!("No CNSB bundle found in path"));
    }

    let bundle = bundle_reader.read_bundle(&bundle_path.join(&bundles[0]))?;

    // Assess each skill in bundle
    let mut skill_assessments = Vec::new();
    let mut total_score = 0.0;

    for skill in &bundle.skills {
        // For now, assess the bundle path itself for each skill
        // In a full implementation, skills would have their own paths
        let skill_score = 100.0; // Placeholder
        let skill_grade = "A".to_string();

        skill_assessments.push(SkillAssessmentSummary {
            skill_name: skill.name.clone(),
            skill_version: skill.version.clone().unwrap_or_default(),
            score: skill_score,
            grade: skill_grade,
            issue_count: 0,
        });

        total_score += skill_score;
    }

    let skill_count = bundle.skill_count();
    let aggregate_score = if skill_count > 0 {
        total_score / skill_count as f64
    } else {
        0.0
    };
    let aggregate_grade = if aggregate_score >= 90.0 {
        "A"
    } else if aggregate_score >= 80.0 {
        "B"
    } else {
        "C"
    };

    let assessment = BundleAssessment {
        bundle: bundle.metadata.clone(),
        skill_assessments,
        aggregate_score,
        aggregate_grade: aggregate_grade.to_string(),
        governance_compliant: bundle.meets_governance(3),
    };

    let meets_minimum = request.min_score.is_none_or(|min| aggregate_score >= min);

    Ok(AssessBundleResponse {
        assessment,
        meets_minimum,
    })
}
