//! Index Use Cases
//!
//! Application layer use cases for Skills Index operations.

use anyhow::Result;
use skillpack_domain::{
    Assessment, DimensionChecker, IndexHistory, IndexValue, SkillReader, SkillsIndex,
};
use std::collections::HashMap;

/// Create a new index
pub struct CreateIndexRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

pub struct CreateIndexResponse {
    pub index: SkillsIndex,
}

pub fn create_index(req: CreateIndexRequest) -> Result<CreateIndexResponse> {
    let mut index = SkillsIndex::new(&req.id, &req.name);
    if let Some(desc) = req.description {
        index.description = desc;
    }
    Ok(CreateIndexResponse { index })
}

/// Add skill to index
pub struct AddToIndexRequest {
    pub index: SkillsIndex,
    pub skill_path: String,
    pub weight: Option<f64>,
}

pub struct AddToIndexResponse {
    pub index: SkillsIndex,
}

pub fn add_to_index(req: AddToIndexRequest) -> Result<AddToIndexResponse> {
    let mut index = req.index;
    let weight = req
        .weight
        .unwrap_or(1.0 / (index.constituents.len() + 1) as f64);
    index.add_constituent(&req.skill_path, weight);
    index.normalize_weights();
    Ok(AddToIndexResponse { index })
}

/// Calculate index value
pub struct CalculateIndexResponse {
    pub value: IndexValue,
    pub assessments: HashMap<String, Assessment>,
}

pub fn calculate_index<R>(
    index: &SkillsIndex,
    reader: R,
    make_checkers: &dyn Fn() -> Vec<Box<dyn DimensionChecker>>,
) -> Result<CalculateIndexResponse>
where
    R: SkillReader + Clone,
{
    use crate::assess_skill::{AssessSkillRequest, AssessSkillUseCase};

    let mut scores = HashMap::new();
    let mut assessments = HashMap::new();

    // Assess each constituent
    for constituent in &index.constituents {
        let use_case = AssessSkillUseCase::new(reader.clone(), make_checkers());
        let request = AssessSkillRequest {
            skill_path: constituent.skill_ref.clone(),
            min_score: None,
        };

        match use_case.execute(request) {
            Ok(response) => {
                let score = response.assessment.total_score().value();
                scores.insert(constituent.skill_ref.clone(), score);
                assessments.insert(constituent.skill_ref.clone(), response.assessment);
            }
            Err(e) => {
                tracing::warn!("Assessment failed for {}: {}", constituent.skill_ref, e);
                scores.insert(constituent.skill_ref.clone(), 0.0);
            }
        }
    }

    let value = index.calculate_value(&scores);

    Ok(CalculateIndexResponse { value, assessments })
}

/// Get index trend
pub struct GetTrendRequest {
    pub history: IndexHistory,
    pub periods: usize,
}

pub struct GetTrendResponse {
    pub current_value: f64,
    pub trend_direction: String,
    pub percent_change: f64,
}

pub fn get_trend(req: GetTrendRequest) -> Result<GetTrendResponse> {
    let trend = req.history.trend(req.periods);

    match trend {
        Some(t) => Ok(GetTrendResponse {
            current_value: req.history.latest().map(|v| v.value).unwrap_or(0.0),
            trend_direction: format!("{:?}", t.direction),
            percent_change: t.percent_change,
        }),
        None => Ok(GetTrendResponse {
            current_value: req.history.latest().map(|v| v.value).unwrap_or(0.0),
            trend_direction: "Flat".to_string(),
            percent_change: 0.0,
        }),
    }
}

/// Compare two indices
pub struct CompareIndicesRequest {
    pub index_a: IndexValue,
    pub index_b: IndexValue,
}

pub struct CompareIndicesResponse {
    pub difference: f64,
    pub stronger_index: String,
}

pub fn compare_indices(req: CompareIndicesRequest) -> Result<CompareIndicesResponse> {
    let diff = req.index_a.value - req.index_b.value;
    let stronger = if diff > 0.0 {
        req.index_a.index_id.as_str().to_string()
    } else if diff < 0.0 {
        req.index_b.index_id.as_str().to_string()
    } else {
        "equal".to_string()
    };

    Ok(CompareIndicesResponse {
        difference: diff.abs(),
        stronger_index: stronger,
    })
}
