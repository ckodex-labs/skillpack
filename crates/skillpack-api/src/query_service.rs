//! Query Service Handler
//!
//! gRPC service implementation for QueryService.

use crate::cache::{EventBus, ServerEvent, TtlCache};
use crate::proto;
use crate::proto::query_service_server::QueryService;
use skillpack_domain::SupportLevel;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

/// CLIENT-SPEC.md §4.1 TTL values (seconds)
#[allow(dead_code)]
const TTL_SKILL_LIST: Duration = Duration::from_secs(300); // 5 min
#[allow(dead_code)]
const TTL_SKILL_DETAIL: Duration = Duration::from_secs(120); // 2 min
#[allow(dead_code)]
const TTL_ASSESSMENT: Duration = Duration::from_secs(60); // 1 min
#[allow(dead_code)]
const TTL_SYNC_STATUS: Duration = Duration::from_secs(30); // 30 s
#[allow(dead_code)]
const TTL_REGISTRY_SEARCH: Duration = Duration::from_secs(600); // 10 min

/// Query service implementation
pub struct QueryServiceImpl {
    // In-memory cache with TTL per CLIENT-SPEC.md §4.1
    skills: Arc<RwLock<TtlCache<String, SkillProfile>>>,
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
}

/// Internal skill profile
#[derive(Clone, serde::Serialize)]
pub struct SkillProfile {
    pub skill_ref: String,
    pub name: String,
    pub description: Option<String>,
    pub score: f64,
    pub grade: String,
    pub moodys_rating: String,
    pub reputation_tier: String,
    pub compatibility_score: u32,
    pub carbon_rating: String,
    pub monthly_cost_usd: f64,
    pub tags: Vec<String>,
    pub capabilities: HashMap<String, SupportLevel>,
}

impl QueryServiceImpl {
    pub fn new(
        event_bus: Arc<EventBus>,
        skills: Arc<RwLock<TtlCache<String, SkillProfile>>>,
    ) -> Self {
        // Spawn background task to invalidate cache on skill mutations
        let skills_clone = skills.clone();
        let mut rx = event_bus.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    ServerEvent::SkillCreated { skill_ref }
                    | ServerEvent::SkillUpdated { skill_ref }
                    | ServerEvent::SkillDeleted { skill_ref } => {
                        let mut cache = skills_clone.write().await;
                        cache.invalidate(&skill_ref);
                    }
                    ServerEvent::SyncCompleted { .. } => {
                        // Full cache flush after sync
                        let mut cache = skills_clone.write().await;
                        cache.clear();
                    }
                    _ => {}
                }
            }
        });

        Self { skills, event_bus }
    }

    /// Seed with sample data
    pub async fn seed_sample_data(&self) {
        let mut skills = self.skills.write().await;

        let samples = vec![
            SkillProfile {
                skill_ref: "security-scanner".into(),
                name: "Security Scanner".into(),
                description: Some("Enterprise security scanning skill".into()),
                score: 142.0,
                grade: "S".into(),
                moodys_rating: "Aa1".into(),
                reputation_tier: "Platinum".into(),
                compatibility_score: 95,
                carbon_rating: "A".into(),
                monthly_cost_usd: 45.0,
                tags: vec!["security".into(), "enterprise".into(), "audit".into()],
                capabilities: [
                    ("skill-md".into(), SupportLevel::Full),
                    ("security-md".into(), SupportLevel::Full),
                    ("slsa-provenance".into(), SupportLevel::Full),
                    ("mcp-server".into(), SupportLevel::Partial),
                ]
                .into_iter()
                .collect(),
            },
            SkillProfile {
                skill_ref: "code-analyzer".into(),
                name: "Code Analyzer".into(),
                description: Some("Static code analysis with AI".into()),
                score: 138.0,
                grade: "S".into(),
                moodys_rating: "Aa2".into(),
                reputation_tier: "Gold".into(),
                compatibility_score: 88,
                carbon_rating: "B".into(),
                monthly_cost_usd: 120.0,
                tags: vec!["analysis".into(), "ai".into(), "quality".into()],
                capabilities: [
                    ("skill-md".into(), SupportLevel::Full),
                    ("security-md".into(), SupportLevel::Partial),
                    ("mcp-server".into(), SupportLevel::Full),
                ]
                .into_iter()
                .collect(),
            },
            SkillProfile {
                skill_ref: "terraform-skill".into(),
                name: "Terraform Skill".into(),
                description: Some("Infrastructure as Code automation".into()),
                score: 125.0,
                grade: "A+".into(),
                moodys_rating: "A1".into(),
                reputation_tier: "Gold".into(),
                compatibility_score: 92,
                carbon_rating: "A".into(),
                monthly_cost_usd: 30.0,
                tags: vec!["iac".into(), "terraform".into(), "devops".into()],
                capabilities: [
                    ("skill-md".into(), SupportLevel::Full),
                    ("slsa-provenance".into(), SupportLevel::Full),
                ]
                .into_iter()
                .collect(),
            },
        ];

        for skill in samples {
            skills.insert(skill.skill_ref.clone(), skill);
        }
    }
}

impl Default for QueryServiceImpl {
    fn default() -> Self {
        let event_bus = Arc::new(EventBus::default());
        let skills = Arc::new(RwLock::new(TtlCache::new(TTL_SKILL_LIST)));
        Self::new(event_bus, skills)
    }
}

#[tonic::async_trait]
impl QueryService for QueryServiceImpl {
    async fn search(
        &self,
        request: Request<proto::SearchRequest>,
    ) -> Result<Response<proto::SearchResponse>, Status> {
        let req = request.into_inner();
        let skills = self.skills.read().await;

        let mut results: Vec<&SkillProfile> = skills.values();

        // Apply filters
        if let Some(ref text) = req.text {
            let text_lower = text.to_lowercase();
            results.retain(|s| {
                s.name.to_lowercase().contains(&text_lower)
                    || s.description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&text_lower))
            });
        }

        if let Some(min_score) = req.min_score {
            results.retain(|s| s.score >= min_score);
        }

        if let Some(ref min_grade) = req.min_grade {
            // Simple grade comparison (S > A+ > A > B > C > D > F)
            results.retain(|s| grade_value(&s.grade) >= grade_value(min_grade));
        }

        // Filter by required capabilities
        if !req.requires_capabilities.is_empty() {
            results.retain(|s| {
                req.requires_capabilities.iter().all(|cap| {
                    matches!(
                        s.capabilities.get(cap),
                        Some(SupportLevel::Full) | Some(SupportLevel::Partial)
                    )
                })
            });
        }

        // Sort
        match proto::SortField::try_from(req.sort_by).unwrap_or(proto::SortField::Unspecified) {
            proto::SortField::Score => results.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            proto::SortField::Name => results.sort_by(|a, b| a.name.cmp(&b.name)),
            proto::SortField::Cost => results.sort_by(|a, b| {
                a.monthly_cost_usd
                    .partial_cmp(&b.monthly_cost_usd)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => {}
        }

        // Reverse for ascending
        if req.sort_direction == proto::SortDirection::Asc as i32 {
            results.reverse();
        }

        let total = results.len() as u32;
        let offset = req.offset.unwrap_or(0);
        let limit = req.limit.unwrap_or(20);

        let page: Vec<proto::SkillSummary> = results
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(skill_profile_to_summary)
            .collect();

        Ok(Response::new(proto::SearchResponse {
            skills: page,
            total_count: total,
            page_size: limit,
            offset,
        }))
    }

    async fn get_skill_profile(
        &self,
        request: Request<proto::GetSkillProfileRequest>,
    ) -> Result<Response<proto::GetSkillProfileResponse>, Status> {
        let req = request.into_inner();
        let skills = self.skills.read().await;

        let skill = skills
            .get(&req.skill_ref)
            .ok_or_else(|| Status::not_found(format!("Skill not found: {}", req.skill_ref)))?;

        Ok(Response::new(proto::GetSkillProfileResponse {
            profile: Some(skill_profile_to_proto(skill)),
        }))
    }

    async fn get_compatibility(
        &self,
        request: Request<proto::GetCompatibilityRequest>,
    ) -> Result<Response<proto::GetCompatibilityResponse>, Status> {
        let req = request.into_inner();
        let skills = self.skills.read().await;

        let skill = skills
            .get(&req.skill_ref)
            .ok_or_else(|| Status::not_found(format!("Skill not found: {}", req.skill_ref)))?;

        let capabilities: Vec<proto::CapabilitySupport> = skill
            .capabilities
            .iter()
            .map(|(id, level)| proto::CapabilitySupport {
                capability_id: id.clone(),
                capability_name: id.replace('-', " ").to_uppercase(),
                level: match level {
                    SupportLevel::Full => proto::SupportLevel::Full as i32,
                    SupportLevel::Partial => proto::SupportLevel::Partial as i32,
                    SupportLevel::None => proto::SupportLevel::None as i32,
                    SupportLevel::Unknown => proto::SupportLevel::Unknown as i32,
                    SupportLevel::Deprecated => proto::SupportLevel::Deprecated as i32,
                },
                notes: None,
            })
            .collect();

        Ok(Response::new(proto::GetCompatibilityResponse {
            compatibility: Some(proto::SkillCompatibility {
                skill_ref: skill.skill_ref.clone(),
                capabilities,
                compatibility_score: skill.compatibility_score,
                updated_at: chrono::Utc::now().to_rfc3339(),
            }),
        }))
    }

    async fn get_cost_estimate(
        &self,
        request: Request<proto::GetCostEstimateRequest>,
    ) -> Result<Response<proto::GetCostEstimateResponse>, Status> {
        let req = request.into_inner();
        let skills = self.skills.read().await;

        let skill = skills
            .get(&req.skill_ref)
            .ok_or_else(|| Status::not_found(format!("Skill not found: {}", req.skill_ref)))?;

        let invocations = req.monthly_invocations.unwrap_or(1000);
        let cost_per_inv = skill.monthly_cost_usd / 1000.0;
        let estimated_cost = cost_per_inv * invocations as f64;

        Ok(Response::new(proto::GetCostEstimateResponse {
            cost: Some(proto::CostEstimate {
                skill_ref: skill.skill_ref.clone(),
                monthly_cost_usd: estimated_cost,
                cost_tier: if estimated_cost < 10.0 {
                    "Free"
                } else if estimated_cost < 50.0 {
                    "Low"
                } else if estimated_cost < 200.0 {
                    "Medium"
                } else {
                    "High"
                }
                .to_string(),
                carbon: Some(proto::CarbonFootprint {
                    co2_grams_per_invocation: 0.5,
                    monthly_co2_kg: 0.5 * invocations as f64 / 1000.0,
                    carbon_rating: skill.carbon_rating.clone(),
                    region: req.region,
                }),
                tokens: Some(proto::TokenCost {
                    input_tokens: 500,
                    output_tokens: 1500,
                    cost_per_invocation: cost_per_inv,
                }),
            }),
        }))
    }
}

// Helper functions
fn grade_value(grade: &str) -> u32 {
    match grade {
        "S+" => 150,
        "S" => 140,
        "A+" => 130,
        "A" => 120,
        "B" => 100,
        "C" => 80,
        "D" => 60,
        "F" => 40,
        _ => 0,
    }
}

fn skill_profile_to_summary(skill: &SkillProfile) -> proto::SkillSummary {
    proto::SkillSummary {
        skill_ref: skill.skill_ref.clone(),
        name: skill.name.clone(),
        description: skill.description.clone(),
        score: skill.score,
        grade: skill.grade.clone(),
        moodys_rating: skill.moodys_rating.clone(),
        reputation_tier: skill.reputation_tier.clone(),
        compatibility_score: skill.compatibility_score,
        carbon_rating: skill.carbon_rating.clone(),
        monthly_cost_usd: skill.monthly_cost_usd,
        tags: skill.tags.clone(),
    }
}

fn skill_profile_to_proto(skill: &SkillProfile) -> proto::SkillProfile {
    proto::SkillProfile {
        summary: Some(skill_profile_to_summary(skill)),
        compatibility: Some(proto::SkillCompatibility {
            skill_ref: skill.skill_ref.clone(),
            capabilities: skill
                .capabilities
                .iter()
                .map(|(id, level)| proto::CapabilitySupport {
                    capability_id: id.clone(),
                    capability_name: id.clone(),
                    level: match level {
                        SupportLevel::Full => proto::SupportLevel::Full as i32,
                        SupportLevel::Partial => proto::SupportLevel::Partial as i32,
                        _ => proto::SupportLevel::None as i32,
                    },
                    notes: None,
                })
                .collect(),
            compatibility_score: skill.compatibility_score,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }),
        reputation: None,
        cost: None,
        rating: None,
        indices: vec![],
    }
}

fn _carbon_order(rating: &str) -> u8 {
    match rating.to_uppercase().as_str() {
        "A+" => 6,
        "A" => 5,
        "B" => 4,
        "C" => 3,
        "D" => 2,
        "F" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{EventBus, ServerEvent};

    #[tokio::test]
    async fn cache_invalidated_on_skill_updated_event() {
        let event_bus = Arc::new(EventBus::new(16));
        let skills = Arc::new(RwLock::new(TtlCache::new(TTL_SKILL_LIST)));
        let qs = QueryServiceImpl::new(event_bus.clone(), skills.clone());

        // Seed cache
        qs.seed_sample_data().await;

        // Verify entry exists
        {
            let cache = skills.read().await;
            assert!(cache.get(&"security-scanner".to_string()).is_some());
        }

        // Publish update event
        event_bus.publish(ServerEvent::SkillUpdated {
            skill_ref: "security-scanner".to_string(),
        });

        // Allow background task to process
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify entry was invalidated
        {
            let cache = skills.read().await;
            assert!(cache.get(&"security-scanner".to_string()).is_none());
        }
    }

    #[tokio::test]
    async fn cache_cleared_on_sync_completed_event() {
        let event_bus = Arc::new(EventBus::new(16));
        let skills = Arc::new(RwLock::new(TtlCache::new(TTL_SKILL_LIST)));
        let qs = QueryServiceImpl::new(event_bus.clone(), skills.clone());

        qs.seed_sample_data().await;

        {
            let cache = skills.read().await;
            assert!(!cache.is_empty());
        }

        event_bus.publish(ServerEvent::SyncCompleted {
            skills_processed: 3,
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        {
            let cache = skills.read().await;
            assert!(cache.is_empty());
        }
    }
}
