//! Domain to Proto Conversions

use crate::proto;
use skillpack_domain::{Assessment, Severity};
use std::collections::HashMap;

pub fn assessment_to_proto(assessment: &Assessment) -> proto::Assessment {
    proto::Assessment {
        id: assessment.id.0.clone(),
        skill: Some(proto::SkillIdentity {
            name: assessment.skill.name.clone(),
            version: assessment.skill.version.clone(),
            path: assessment.skill.path.clone(),
        }),
        dimension_scores: assessment
            .dimension_scores
            .iter()
            .map(|(k, v)| (k.name().to_string(), v.value()))
            .collect::<HashMap<_, _>>(),
        bonus_points: Some(proto::BonusPoints {
            slsa_level_3: assessment.bonus_points.slsa_level_3,
            sigstore_signing: assessment.bonus_points.sigstore_signing,
            dagger_pipeline: assessment.bonus_points.dagger_pipeline,
            stride_threat_model: assessment.bonus_points.stride_threat_model,
            mcp_server: assessment.bonus_points.mcp_server,
            total: assessment.bonus_points.total(),
        }),
        issues: assessment
            .issues
            .iter()
            .map(|i| proto::Issue {
                dimension: i.dimension.name().to_string(),
                severity: match i.severity {
                    Severity::Error => proto::Severity::Error as i32,
                    Severity::Warning => proto::Severity::Warning as i32,
                    Severity::Note => proto::Severity::Note as i32,
                },
                message: i.message.clone(),
                file: i.file.clone(),
                line: i.line,
            })
            .collect(),
        assessed_at: assessment.assessed_at.to_rfc3339(),
        base_score: assessment.base_score().value(),
        total_score: assessment.total_score().value(),
        grade: assessment.grade().as_str().to_string(),
        profile: assessment.profile.name().to_string(),
    }
}
