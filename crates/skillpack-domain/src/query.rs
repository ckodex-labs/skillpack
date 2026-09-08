//! Skill Query System
//!
//! Search and filter skills by various criteria.

use crate::caniuse::SupportLevel;
use crate::reputation::ReputationTier;
use serde::{Deserialize, Serialize};

/// Query filters for searching skills
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillQuery {
    /// Text search (name, description)
    pub text: Option<String>,
    /// Minimum quality score
    pub min_score: Option<f64>,
    /// Maximum quality score
    pub max_score: Option<f64>,
    /// Minimum grade
    pub min_grade: Option<String>,
    /// Required capabilities (CanIUse)
    pub requires_capabilities: Vec<String>,
    /// Minimum capability support level
    pub min_support_level: Option<SupportLevel>,
    /// Minimum reputation tier
    pub min_reputation: Option<ReputationTier>,
    /// Required badges
    pub requires_badges: Vec<String>,
    /// Filter by index membership
    pub in_index: Option<String>,
    /// Tags to match
    pub tags: Vec<String>,
    /// Maintainer/organization filter
    pub maintainer: Option<String>,
    /// Sort order
    pub sort_by: SortField,
    /// Sort direction
    pub sort_direction: SortDirection,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Score,
    Name,
    Reputation,
    Downloads,
    Updated,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Descending,
    Ascending,
}

impl SkillQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    pub fn min_grade(mut self, grade: impl Into<String>) -> Self {
        self.min_grade = Some(grade.into());
        self
    }

    pub fn requires(mut self, capability: impl Into<String>) -> Self {
        self.requires_capabilities.push(capability.into());
        self
    }

    pub fn min_reputation(mut self, tier: ReputationTier) -> Self {
        self.min_reputation = Some(tier);
        self
    }

    pub fn in_index(mut self, index: impl Into<String>) -> Self {
        self.in_index = Some(index.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn sort_by(mut self, field: SortField) -> Self {
        self.sort_by = field;
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub skills: Vec<SkillSummary>,
    pub total_count: usize,
    pub page_size: usize,
    pub offset: usize,
}

/// Summary info for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub skill_ref: String,
    pub name: String,
    pub description: Option<String>,
    pub score: f64,
    pub grade: String,
    pub reputation_tier: ReputationTier,
    pub compatibility_score: u32,
    pub tags: Vec<String>,
}

/// Predefined query templates
pub mod templates {
    use super::*;

    /// Find enterprise-ready skills
    pub fn enterprise_ready() -> SkillQuery {
        SkillQuery::new()
            .min_grade("A".to_string())
            .min_reputation(ReputationTier::Gold)
            .requires("security-md")
            .requires("threat-model")
            .requires("slsa-provenance")
    }

    /// Find security-focused skills
    pub fn security_focused() -> SkillQuery {
        SkillQuery::new()
            .requires("security-md")
            .requires("threat-model")
            .requires("no-secrets")
            .tag("security")
    }

    /// Find skills with MCP support
    pub fn mcp_enabled() -> SkillQuery {
        SkillQuery::new()
            .requires("mcp-server")
            .requires("mcp-tools")
    }

    /// Find well-documented skills
    pub fn well_documented() -> SkillQuery {
        SkillQuery::new().min_score(80.0).tag("documentation")
    }

    /// Find top rated skills
    pub fn top_rated() -> SkillQuery {
        SkillQuery::new()
            .min_reputation(ReputationTier::Platinum)
            .sort_by(SortField::Score)
            .limit(100)
    }
}
