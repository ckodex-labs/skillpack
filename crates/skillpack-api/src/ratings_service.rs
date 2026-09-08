//! Ratings Service Handler
//!
//! gRPC service implementation for RatingsService.

use crate::proto;
use crate::proto::ratings_service_server::RatingsService;
use skillpack_domain::{
    RatingAssessment, RatingCategory, RatingFactors, RatingOutlook, SkillRating,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

/// In-memory ratings storage (replace with DuckDbRepository in production)
pub struct RatingsServiceImpl {
    ratings: Arc<RwLock<HashMap<String, RatingAssessment>>>,
}

impl RatingsServiceImpl {
    pub fn new() -> Self {
        Self {
            ratings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Seed with sample data
    pub async fn seed_sample_data(&self) {
        let mut ratings = self.ratings.write().await;

        // Sample investment-grade skills
        let samples = vec![
            ("security-scanner", SkillRating::Aa1, 142.0),
            ("code-analyzer", SkillRating::Aa2, 138.0),
            ("terraform-skill", SkillRating::A1, 125.0),
            ("docs-generator", SkillRating::Baa1, 112.0),
            ("legacy-skill", SkillRating::Ba2, 85.0),
        ];

        for (skill_ref, rating, score) in samples {
            let mut assessment = RatingAssessment::new(skill_ref, rating);
            assessment.factors = RatingFactors {
                quality_score: score,
                security_score: score * 0.9,
                governance_score: score * 0.8,
                track_record_score: score * 0.85,
                maintainer_score: score * 0.95,
            };
            ratings.insert(skill_ref.to_string(), assessment);
        }
    }
}

impl Default for RatingsServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl RatingsService for RatingsServiceImpl {
    async fn get_rating(
        &self,
        request: Request<proto::GetRatingRequest>,
    ) -> Result<Response<proto::GetRatingResponse>, Status> {
        let req = request.into_inner();

        let ratings = self.ratings.read().await;
        let assessment = ratings
            .get(&req.skill_ref)
            .ok_or_else(|| Status::not_found(format!("Rating not found: {}", req.skill_ref)))?;

        Ok(Response::new(proto::GetRatingResponse {
            rating: Some(rating_assessment_to_proto(assessment)),
        }))
    }

    async fn get_rating_history(
        &self,
        request: Request<proto::GetRatingHistoryRequest>,
    ) -> Result<Response<proto::GetRatingHistoryResponse>, Status> {
        let req = request.into_inner();

        let ratings = self.ratings.read().await;
        let assessment = ratings
            .get(&req.skill_ref)
            .ok_or_else(|| Status::not_found(format!("Rating not found: {}", req.skill_ref)))?;

        let history: Vec<proto::RatingChange> = assessment
            .history
            .iter()
            .map(|h| proto::RatingChange {
                from_rating: h.from.as_str().to_string(),
                to_rating: h.to.as_str().to_string(),
                action: match h.action {
                    skillpack_domain::RatingAction::Upgrade => proto::RatingAction::Upgrade as i32,
                    skillpack_domain::RatingAction::Downgrade => {
                        proto::RatingAction::Downgrade as i32
                    }
                    skillpack_domain::RatingAction::Affirmed => {
                        proto::RatingAction::Affirmed as i32
                    }
                    skillpack_domain::RatingAction::InitialRating => {
                        proto::RatingAction::Initial as i32
                    }
                    skillpack_domain::RatingAction::Withdrawn => {
                        proto::RatingAction::Withdrawn as i32
                    }
                },
                reason: h.reason.clone(),
                timestamp: h.timestamp.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(proto::GetRatingHistoryResponse { history }))
    }

    async fn compare_ratings(
        &self,
        request: Request<proto::CompareRatingsRequest>,
    ) -> Result<Response<proto::CompareRatingsResponse>, Status> {
        let req = request.into_inner();

        let ratings = self.ratings.read().await;
        let mut comparisons = Vec::new();
        let mut best_rating: Option<(&str, SkillRating)> = None;

        for skill_ref in &req.skill_refs {
            if let Some(assessment) = ratings.get(skill_ref) {
                comparisons.push(proto::RatingComparison {
                    skill_ref: skill_ref.clone(),
                    rating: assessment.rating.as_str().to_string(),
                    is_investment_grade: assessment.rating.is_investment_grade(),
                    composite_score: assessment.factors.composite(),
                });

                if let Some((_, current)) = best_rating {
                    if assessment.rating < current {
                        best_rating = Some((skill_ref, assessment.rating));
                    }
                } else {
                    best_rating = Some((skill_ref, assessment.rating));
                }
            }
        }

        Ok(Response::new(proto::CompareRatingsResponse {
            comparisons,
            best_rated: best_rating.map(|(s, _)| s.to_string()).unwrap_or_default(),
        }))
    }
}

// Proto conversion helper
fn rating_assessment_to_proto(assessment: &RatingAssessment) -> proto::RatingAssessment {
    proto::RatingAssessment {
        skill_ref: assessment.skill_ref.clone(),
        rating: assessment.rating.as_str().to_string(),
        category: match assessment.rating.category() {
            RatingCategory::Prime => proto::RatingCategory::Prime as i32,
            RatingCategory::HighGrade => proto::RatingCategory::HighGrade as i32,
            RatingCategory::MediumGrade => proto::RatingCategory::MediumGrade as i32,
            RatingCategory::Speculative => proto::RatingCategory::Speculative as i32,
            RatingCategory::HighlySpeculative => proto::RatingCategory::HighlySpeculative as i32,
            RatingCategory::SubstantialRisk => proto::RatingCategory::SubstantialRisk as i32,
        },
        is_investment_grade: assessment.rating.is_investment_grade(),
        outlook: match assessment.outlook {
            RatingOutlook::Positive => proto::RatingOutlook::Positive as i32,
            RatingOutlook::Stable => proto::RatingOutlook::Stable as i32,
            RatingOutlook::Negative => proto::RatingOutlook::Negative as i32,
            RatingOutlook::UnderReview => proto::RatingOutlook::UnderReview as i32,
        },
        assessed_at: assessment.assessed_at.to_rfc3339(),
        factors: Some(proto::RatingFactors {
            quality_score: assessment.factors.quality_score,
            security_score: assessment.factors.security_score,
            governance_score: assessment.factors.governance_score,
            track_record_score: assessment.factors.track_record_score,
            maintainer_score: assessment.factors.maintainer_score,
            composite: assessment.factors.composite(),
        }),
        notes: assessment.notes.clone(),
    }
}
