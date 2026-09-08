//! Grade Skill Use Case

use crate::{AssessSkillRequest, AssessSkillUseCase};
use anyhow::Result;
use skillpack_domain::Grade;
use skillpack_domain::SkillReader;

/// Request for grade check
pub struct GradeSkillRequest {
    pub skill_path: String,
    pub minimum_grade: Grade,
}

/// Response from grade check
pub struct GradeSkillResponse {
    pub grade: Grade,
    pub total_score: f64,
    pub meets_minimum: bool,
}

/// Grade skill use case (thin wrapper around assess)
pub fn grade_skill<R: SkillReader>(
    use_case: &AssessSkillUseCase<R>,
    request: GradeSkillRequest,
) -> Result<GradeSkillResponse> {
    let assess_request = AssessSkillRequest {
        skill_path: request.skill_path,
        min_score: None,
    };

    let response = use_case.execute(assess_request)?;
    let grade = response.assessment.grade();
    let total_score = response.assessment.total_score().value();
    let meets_minimum = grade.meets_minimum(&request.minimum_grade);

    Ok(GradeSkillResponse {
        grade,
        total_score,
        meets_minimum,
    })
}
