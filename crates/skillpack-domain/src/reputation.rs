//! Skill Reputation System
//!
//! Trust scores, verification badges, and community ratings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reputation score (0-1000)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ReputationScore(u32);

impl ReputationScore {
    pub fn new(value: u32) -> Self {
        Self(value.min(1000))
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn tier(&self) -> ReputationTier {
        match self.0 {
            900..=1000 => ReputationTier::Diamond,
            750..=899 => ReputationTier::Platinum,
            600..=749 => ReputationTier::Gold,
            450..=599 => ReputationTier::Silver,
            300..=449 => ReputationTier::Bronze,
            150..=299 => ReputationTier::Verified,
            _ => ReputationTier::Unverified,
        }
    }
}

/// Reputation tiers (like gaming ranks or marketplace badges)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReputationTier {
    Unverified,
    Verified,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl ReputationTier {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Diamond => "💎",
            Self::Platinum => "🏆",
            Self::Gold => "🥇",
            Self::Silver => "🥈",
            Self::Bronze => "🥉",
            Self::Verified => "✓",
            Self::Unverified => "○",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Diamond => "Diamond",
            Self::Platinum => "Platinum",
            Self::Gold => "Gold",
            Self::Silver => "Silver",
            Self::Bronze => "Bronze",
            Self::Verified => "Verified",
            Self::Unverified => "Unverified",
        }
    }
}

/// Verification badge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationBadge {
    pub badge_type: BadgeType,
    pub issued_by: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub verification_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeType {
    /// Verified by maintainer identity
    MaintainerVerified,
    /// Verified by organization
    OrganizationVerified,
    /// Security audit passed
    SecurityAudited,
    /// SLSA compliance verified
    SlsaCompliant,
    /// Enterprise certified
    EnterpriseCertified,
    /// Community endorsed
    CommunityEndorsed,
}

impl BadgeType {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::MaintainerVerified => "👤",
            Self::OrganizationVerified => "🏢",
            Self::SecurityAudited => "🔒",
            Self::SlsaCompliant => "📜",
            Self::EnterpriseCertified => "🏛️",
            Self::CommunityEndorsed => "👥",
        }
    }
}

/// Full reputation profile for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReputation {
    pub skill_ref: String,
    pub score: ReputationScore,
    pub badges: Vec<VerificationBadge>,
    /// Quality score history
    pub quality_history: Vec<QualitySnapshot>,
    /// Usage metrics
    pub metrics: UsageMetrics,
    /// Community ratings
    pub ratings: CommunityRatings,
    pub updated_at: DateTime<Utc>,
}

impl SkillReputation {
    pub fn new(skill_ref: impl Into<String>) -> Self {
        Self {
            skill_ref: skill_ref.into(),
            score: ReputationScore::default(),
            badges: Vec::new(),
            quality_history: Vec::new(),
            metrics: UsageMetrics::default(),
            ratings: CommunityRatings::default(),
            updated_at: Utc::now(),
        }
    }

    /// Calculate reputation from all factors
    pub fn calculate_score(&mut self) {
        let mut score = 0u32;

        // Base quality score (0-300)
        if let Some(latest) = self.quality_history.last() {
            score += (latest.score * 2.0) as u32; // Max 300
        }

        // Badge bonuses (0-300)
        for badge in &self.badges {
            score += match badge.badge_type {
                BadgeType::EnterpriseCertified => 100,
                BadgeType::SecurityAudited => 80,
                BadgeType::SlsaCompliant => 60,
                BadgeType::OrganizationVerified => 50,
                BadgeType::MaintainerVerified => 30,
                BadgeType::CommunityEndorsed => 20,
            };
        }

        // Usage metrics (0-200)
        score += (self.metrics.weekly_downloads / 100).min(100) as u32;
        score += (self.metrics.dependent_skills * 10).min(100);

        // Community ratings (0-200)
        score += (self.ratings.average_rating * 40.0) as u32; // Max 200

        self.score = ReputationScore::new(score);
        self.updated_at = Utc::now();
    }

    pub fn tier(&self) -> ReputationTier {
        self.score.tier()
    }

    pub fn add_badge(&mut self, badge: VerificationBadge) {
        self.badges.push(badge);
        self.calculate_score();
    }
}

/// Quality score snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    pub score: f64,
    pub grade: String,
    pub timestamp: DateTime<Utc>,
}

/// Usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub total_downloads: u64,
    pub weekly_downloads: u64,
    pub dependent_skills: u32,
    pub stars: u32,
    pub forks: u32,
}

/// Community ratings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRatings {
    pub total_ratings: u32,
    pub average_rating: f64,           // 0.0 - 5.0
    pub rating_distribution: [u32; 5], // 1-5 stars
}

impl Default for CommunityRatings {
    fn default() -> Self {
        Self {
            total_ratings: 0,
            average_rating: 0.0,
            rating_distribution: [0; 5],
        }
    }
}

impl CommunityRatings {
    pub fn add_rating(&mut self, rating: u8) {
        let rating = rating.clamp(1, 5);
        self.rating_distribution[(rating - 1) as usize] += 1;
        self.total_ratings += 1;

        // Recalculate average
        let sum: u32 = self
            .rating_distribution
            .iter()
            .enumerate()
            .map(|(i, count)| (i as u32 + 1) * count)
            .sum();
        self.average_rating = sum as f64 / self.total_ratings as f64;
    }
}
