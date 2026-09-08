//! Skills Index Domain Model
//!
//! Like stock indices (S&P 500, NASDAQ), but for AI agent skills.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an index
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexId(String);

impl IndexId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Skills Index - aggregates multiple skills into a composite score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsIndex {
    /// Unique index identifier
    pub id: IndexId,
    /// Human-readable name
    pub name: String,
    /// Index description
    pub description: String,
    /// Index methodology/calculation method
    pub methodology: IndexMethodology,
    /// Index constituents (skills)
    pub constituents: Vec<IndexConstituent>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last rebalance timestamp
    pub last_rebalanced: DateTime<Utc>,
    /// Index metadata
    pub metadata: IndexMetadata,
}

impl SkillsIndex {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: IndexId::new(id),
            name: name.into(),
            description: String::new(),
            methodology: IndexMethodology::default(),
            constituents: Vec::new(),
            created_at: now,
            last_rebalanced: now,
            metadata: IndexMetadata::default(),
        }
    }

    /// Calculate current index value from constituents
    pub fn calculate_value(&self, scores: &HashMap<String, f64>) -> IndexValue {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut constituent_values = Vec::new();

        for constituent in &self.constituents {
            if let Some(&score) = scores.get(&constituent.skill_ref) {
                let weighted = score * constituent.weight;
                weighted_sum += weighted;
                total_weight += constituent.weight;
                constituent_values.push(ConstituentValue {
                    skill_ref: constituent.skill_ref.clone(),
                    score,
                    weighted_score: weighted,
                });
            }
        }

        let value = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        IndexValue {
            index_id: self.id.clone(),
            value,
            timestamp: Utc::now(),
            constituent_count: self.constituents.len(),
            constituent_values,
        }
    }

    /// Add a skill to the index
    pub fn add_constituent(&mut self, skill_ref: impl Into<String>, weight: f64) {
        self.constituents
            .push(IndexConstituent::new(skill_ref, weight));
        self.last_rebalanced = Utc::now();
    }

    /// Add a skill with full profile data
    pub fn add_constituent_full(&mut self, constituent: IndexConstituent) {
        self.constituents.push(constituent);
        self.last_rebalanced = Utc::now();
    }

    /// Remove a skill from the index
    pub fn remove_constituent(&mut self, skill_ref: &str) -> bool {
        let before = self.constituents.len();
        self.constituents.retain(|c| c.skill_ref != skill_ref);
        if self.constituents.len() != before {
            self.last_rebalanced = Utc::now();
            true
        } else {
            false
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize_weights(&mut self) {
        let total: f64 = self.constituents.iter().map(|c| c.weight).sum();
        if total > 0.0 {
            for constituent in &mut self.constituents {
                constituent.weight /= total;
            }
        }
        self.last_rebalanced = Utc::now();
    }
}

/// Index constituent - a skill in the index with full quality profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConstituent {
    /// Reference to skill (path, URI, or identifier)
    pub skill_ref: String,
    /// Weight in the index (0.0 - 1.0)
    pub weight: f64,
    /// When added to index
    pub added_at: DateTime<Utc>,
    /// Quality score (0-150)
    pub quality_score: Option<f64>,
    /// Letter grade
    pub grade: Option<String>,
    /// Reputation tier
    pub reputation_tier: Option<String>,
    /// Compatibility score (0-100)
    pub compatibility_score: Option<u32>,
    /// Carbon rating (A-F)
    pub carbon_rating: Option<String>,
    /// Monthly cost estimate (USD)
    pub monthly_cost_usd: Option<f64>,
    /// Moody's-style credit rating
    pub moodys_rating: Option<String>,
}

impl IndexConstituent {
    pub fn new(skill_ref: impl Into<String>, weight: f64) -> Self {
        Self {
            skill_ref: skill_ref.into(),
            weight,
            added_at: Utc::now(),
            quality_score: None,
            grade: None,
            reputation_tier: None,
            compatibility_score: None,
            carbon_rating: None,
            monthly_cost_usd: None,
            moodys_rating: None,
        }
    }

    /// Composite quality score combining all factors
    pub fn composite_score(&self) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Quality score (40% weight)
        if let Some(q) = self.quality_score {
            score += (q / 150.0) * 40.0;
            factors += 1;
        }

        // Compatibility (30% weight)
        if let Some(c) = self.compatibility_score {
            score += (c as f64 / 100.0) * 30.0;
            factors += 1;
        }

        // Reputation bonus (20% weight)
        if let Some(ref tier) = self.reputation_tier {
            let tier_score = match tier.as_str() {
                "Diamond" => 100.0,
                "Platinum" => 85.0,
                "Gold" => 70.0,
                "Silver" => 55.0,
                "Bronze" => 40.0,
                "Verified" => 25.0,
                _ => 10.0,
            };
            score += (tier_score / 100.0) * 20.0;
            factors += 1;
        }

        // Carbon efficiency (10% weight)
        if let Some(ref rating) = self.carbon_rating {
            let carbon_score = match rating.as_str() {
                "A" => 100.0,
                "B" => 80.0,
                "C" => 60.0,
                "D" => 40.0,
                _ => 20.0,
            };
            score += (carbon_score / 100.0) * 10.0;
            factors += 1;
        }

        if factors > 0 { score } else { 0.0 }
    }
}

/// Index calculation methodology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMethodology {
    /// Weighting scheme
    pub weighting: WeightingScheme,
    /// Minimum score threshold for inclusion
    pub min_score_threshold: Option<f64>,
    /// Maximum constituents
    pub max_constituents: Option<usize>,
    /// Rebalance frequency
    pub rebalance_frequency: RebalanceFrequency,
}

impl Default for IndexMethodology {
    fn default() -> Self {
        Self {
            weighting: WeightingScheme::EqualWeight,
            min_score_threshold: None,
            max_constituents: None,
            rebalance_frequency: RebalanceFrequency::Manual,
        }
    }
}

/// How constituents are weighted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightingScheme {
    /// All skills weighted equally
    EqualWeight,
    /// Weighted by score (higher scores = more weight)
    ScoreWeighted,
    /// Weighted by dimension (e.g., security-heavy)
    DimensionWeighted { dimension: String, bias: f64 },
    /// Custom user-defined weights
    Custom,
}

/// How often the index is rebalanced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceFrequency {
    Manual,
    Daily,
    Weekly,
    Monthly,
}

/// Index metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Index category
    pub category: Option<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Point-in-time index value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexValue {
    /// Index this value belongs to
    pub index_id: IndexId,
    /// Composite index value (0-150 scale)
    pub value: f64,
    /// When calculated
    pub timestamp: DateTime<Utc>,
    /// Number of constituents
    pub constituent_count: usize,
    /// Individual constituent scores
    pub constituent_values: Vec<ConstituentValue>,
}

impl IndexValue {
    /// Get letter grade for index value
    pub fn grade(&self) -> &'static str {
        match self.value as u32 {
            145..=150 => "S+",
            135..=144 => "S",
            120..=134 => "A",
            100..=119 => "B",
            80..=99 => "C",
            60..=79 => "D",
            _ => "F",
        }
    }

    /// Get trend compared to previous value
    pub fn trend(&self, previous: &IndexValue) -> IndexTrend {
        let change = self.value - previous.value;
        let pct_change = if previous.value > 0.0 {
            (change / previous.value) * 100.0
        } else {
            0.0
        };

        IndexTrend {
            direction: if change > 0.5 {
                TrendDirection::Up
            } else if change < -0.5 {
                TrendDirection::Down
            } else {
                TrendDirection::Flat
            },
            absolute_change: change,
            percent_change: pct_change,
        }
    }
}

/// Individual constituent score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituentValue {
    pub skill_ref: String,
    pub score: f64,
    pub weighted_score: f64,
}

/// Index history - time series of values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexHistory {
    pub index_id: IndexId,
    pub values: Vec<IndexValue>,
}

impl IndexHistory {
    pub fn new(index_id: IndexId) -> Self {
        Self {
            index_id,
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, value: IndexValue) {
        self.values.push(value);
    }

    pub fn latest(&self) -> Option<&IndexValue> {
        self.values.last()
    }

    /// Get values in a time range
    pub fn range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&IndexValue> {
        self.values
            .iter()
            .filter(|v| v.timestamp >= start && v.timestamp <= end)
            .collect()
    }

    /// Calculate trend over last N values
    pub fn trend(&self, n: usize) -> Option<IndexTrend> {
        if self.values.len() < 2 {
            return None;
        }

        let recent: Vec<_> = self.values.iter().rev().take(n).collect();
        if recent.len() < 2 {
            return None;
        }

        let latest = recent[0];
        let oldest = recent.last()?;
        Some(latest.trend(oldest))
    }
}

/// Trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexTrend {
    pub direction: TrendDirection,
    pub absolute_change: f64,
    pub percent_change: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Flat,
}

/// Predefined index types
pub mod predefined {
    use super::*;

    /// SkillPack-100: Top 100 skills by score
    pub fn skillpack_100() -> SkillsIndex {
        let mut index = SkillsIndex::new("skillpack-100", "SkillPack-100");
        index.description = "Top 100 AI agent skills by quality score".to_string();
        index.methodology = IndexMethodology {
            weighting: WeightingScheme::EqualWeight,
            min_score_threshold: Some(100.0), // B grade minimum
            max_constituents: Some(100),
            rebalance_frequency: RebalanceFrequency::Weekly,
        };
        index.metadata.category = Some("flagship".to_string());
        index
    }

    /// SkillPack-Security: Security-focused skills
    pub fn skillpack_security() -> SkillsIndex {
        let mut index = SkillsIndex::new("skillpack-security", "SkillPack-Security");
        index.description = "Skills with highest security scores".to_string();
        index.methodology = IndexMethodology {
            weighting: WeightingScheme::DimensionWeighted {
                dimension: "security".to_string(),
                bias: 2.0,
            },
            min_score_threshold: Some(80.0),
            max_constituents: Some(50),
            rebalance_frequency: RebalanceFrequency::Daily,
        };
        index.metadata.category = Some("sector".to_string());
        index.metadata.tags = vec!["security".to_string(), "compliance".to_string()];
        index
    }

    /// SkillPack-Enterprise: Enterprise-grade skills
    pub fn skillpack_enterprise() -> SkillsIndex {
        let mut index = SkillsIndex::new("skillpack-enterprise", "SkillPack-Enterprise");
        index.description = "Enterprise-grade skills with full governance".to_string();
        index.methodology = IndexMethodology {
            weighting: WeightingScheme::ScoreWeighted,
            min_score_threshold: Some(120.0), // A grade minimum
            max_constituents: Some(200),
            rebalance_frequency: RebalanceFrequency::Weekly,
        };
        index.metadata.category = Some("enterprise".to_string());
        index.metadata.tags = vec!["enterprise".to_string(), "governance".to_string()];
        index
    }

    /// SkillPack-Growth: Emerging skills
    pub fn skillpack_growth() -> SkillsIndex {
        let mut index = SkillsIndex::new("skillpack-growth", "SkillPack-Growth");
        index.description = "Emerging skills with high improvement trajectory".to_string();
        index.methodology = IndexMethodology {
            weighting: WeightingScheme::EqualWeight,
            min_score_threshold: Some(60.0),
            max_constituents: Some(50),
            rebalance_frequency: RebalanceFrequency::Daily,
        };
        index.metadata.category = Some("growth".to_string());
        index.metadata.tags = vec!["emerging".to_string(), "growth".to_string()];
        index
    }
}
