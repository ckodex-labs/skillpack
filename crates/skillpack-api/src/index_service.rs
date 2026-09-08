//! Index Service Handler
//!
//! gRPC service implementation for IndexService.

use crate::proto;
use crate::proto::index_service_server::IndexService;
use skillpack_domain::{IndexId, IndexRepository, IndexValue, SkillsIndex};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

/// Index service implementation
pub struct IndexServiceImpl<R: IndexRepository + Send + Sync + 'static> {
    repository: Arc<RwLock<R>>,
}

impl<R: IndexRepository + Send + Sync + 'static> IndexServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(RwLock::new(repository)),
        }
    }
}

#[tonic::async_trait]
impl<R: IndexRepository + Send + Sync + 'static> IndexService for IndexServiceImpl<R> {
    async fn create_index(
        &self,
        request: Request<proto::CreateIndexRequest>,
    ) -> Result<Response<proto::CreateIndexResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();

        let mut index = SkillsIndex::new(&req.id, &req.name);
        index.description = req.description;

        let repo = self.repository.write().await;
        repo.create(&index)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::CreateIndexResponse {
            index: Some(skills_index_to_proto(&index)),
        }))
    }

    async fn get_index(
        &self,
        request: Request<proto::GetIndexRequest>,
    ) -> Result<Response<proto::GetIndexResponse>, Status> {
        let req = request.into_inner();
        let id = IndexId::new(&req.index_id);

        let repo = self.repository.read().await;
        let index = repo
            .get(&id)
            .map_err(|e| Status::not_found(e.to_string()))?;

        // Calculate current value
        let scores: HashMap<String, f64> = index
            .constituents
            .iter()
            .filter_map(|c| c.quality_score.map(|s| (c.skill_ref.clone(), s)))
            .collect();
        let value = index.calculate_value(&scores);

        Ok(Response::new(proto::GetIndexResponse {
            index: Some(skills_index_to_proto(&index)),
            current_value: Some(index_value_to_proto(&value)),
        }))
    }

    async fn list_indices(
        &self,
        _request: Request<proto::ListIndicesRequest>,
    ) -> Result<Response<proto::ListIndicesResponse>, Status> {
        let repo = self.repository.read().await;
        let indices = repo.list().map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::ListIndicesResponse {
            indices: indices.iter().map(skills_index_to_proto).collect(),
        }))
    }

    async fn add_constituent(
        &self,
        request: Request<proto::AddConstituentRequest>,
    ) -> Result<Response<proto::AddConstituentResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        let id = IndexId::new(&req.index_id);

        let repo = self.repository.write().await;
        let mut index = repo
            .get(&id)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let weight = req.weight.unwrap_or(1.0);
        index.add_constituent(&req.skill_ref, weight);

        repo.update(&index)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::AddConstituentResponse {
            index: Some(skills_index_to_proto(&index)),
        }))
    }

    async fn calculate_value(
        &self,
        request: Request<proto::CalculateValueRequest>,
    ) -> Result<Response<proto::CalculateValueResponse>, Status> {
        let req = request.into_inner();
        let id = IndexId::new(&req.index_id);

        let repo = self.repository.read().await;
        let index = repo
            .get(&id)
            .map_err(|e| Status::not_found(e.to_string()))?;

        // Calculate from constituent scores
        let scores: HashMap<String, f64> = index
            .constituents
            .iter()
            .filter_map(|c| c.quality_score.map(|s| (c.skill_ref.clone(), s)))
            .collect();
        let value = index.calculate_value(&scores);

        // Get previous value for trend
        let history = repo.get_history(&id).ok();
        let trend = history.and_then(|h| {
            h.values.last().map(|prev| {
                let change = value.value - prev.value;
                let pct = if prev.value > 0.0 {
                    change / prev.value * 100.0
                } else {
                    0.0
                };
                proto::IndexTrend {
                    direction: if change > 0.0 {
                        1
                    } else if change < 0.0 {
                        2
                    } else {
                        3
                    },
                    absolute_change: change,
                    percent_change: pct,
                }
            })
        });

        Ok(Response::new(proto::CalculateValueResponse {
            value: Some(index_value_to_proto(&value)),
            trend,
        }))
    }

    async fn get_history(
        &self,
        request: Request<proto::GetHistoryRequest>,
    ) -> Result<Response<proto::GetHistoryResponse>, Status> {
        let req = request.into_inner();
        let id = IndexId::new(&req.index_id);

        let repo = self.repository.read().await;
        let history = repo
            .get_history(&id)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(proto::GetHistoryResponse {
            values: history.values.iter().map(index_value_to_proto).collect(),
        }))
    }
}

// Proto conversion helpers
fn skills_index_to_proto(index: &SkillsIndex) -> proto::SkillsIndex {
    proto::SkillsIndex {
        id: index.id.as_str().to_string(),
        name: index.name.clone(),
        description: index.description.clone(),
        methodology: format!("{:?}", index.methodology.weighting),
        constituents: index
            .constituents
            .iter()
            .map(|c| proto::IndexConstituent {
                skill_ref: c.skill_ref.clone(),
                weight: c.weight,
                added_at: c.added_at.to_rfc3339(),
                quality_score: c.quality_score,
                grade: c.grade.clone(),
                reputation_tier: c.reputation_tier.clone(),
                compatibility_score: c.compatibility_score,
                carbon_rating: c.carbon_rating.clone(),
                monthly_cost_usd: c.monthly_cost_usd,
                moodys_rating: c.moodys_rating.clone(),
            })
            .collect(),
        created_at: index.created_at.to_rfc3339(),
        last_rebalanced: index.last_rebalanced.to_rfc3339(),
    }
}

fn index_value_to_proto(value: &IndexValue) -> proto::IndexValue {
    proto::IndexValue {
        index_id: value.index_id.as_str().to_string(),
        value: value.value,
        timestamp: value.timestamp.to_rfc3339(),
        constituent_count: value.constituent_count as u32,
        grade: value.grade().to_string(),
    }
}
