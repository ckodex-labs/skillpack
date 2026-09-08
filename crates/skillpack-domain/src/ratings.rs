//! Moody's-Style Credit Ratings for Skills
//!
//! Like Moody's bond ratings (Aaa, Aa1, Baa2) but for AI agent skills.
//! Provides investment-grade classification and risk outlook.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Moody's-style skill rating (Aaa to C)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SkillRating {
    // Investment Grade (High Quality)
    Aaa, // Prime - Highest quality, minimal risk
    Aa1, // High grade
    Aa2,
    Aa3,
    A1, // Upper medium grade
    A2,
    A3,
    Baa1, // Lower medium grade
    Baa2,
    Baa3, // Lowest investment grade

    // Speculative Grade (Non-Investment)
    Ba1, // Speculative
    Ba2,
    Ba3,
    B1, // Highly speculative
    B2,
    B3,
    Caa1, // Substantial risk
    Caa2,
    Caa3,
    Ca, // Extremely speculative
    C,  // Lowest rating
}

impl SkillRating {
    /// Check if investment grade
    pub fn is_investment_grade(&self) -> bool {
        matches!(
            self,
            Self::Aaa
                | Self::Aa1
                | Self::Aa2
                | Self::Aa3
                | Self::A1
                | Self::A2
                | Self::A3
                | Self::Baa1
                | Self::Baa2
                | Self::Baa3
        )
    }

    /// Get rating category
    pub fn category(&self) -> RatingCategory {
        match self {
            Self::Aaa | Self::Aa1 | Self::Aa2 | Self::Aa3 => RatingCategory::Prime,
            Self::A1 | Self::A2 | Self::A3 => RatingCategory::HighGrade,
            Self::Baa1 | Self::Baa2 | Self::Baa3 => RatingCategory::MediumGrade,
            Self::Ba1 | Self::Ba2 | Self::Ba3 => RatingCategory::Speculative,
            Self::B1 | Self::B2 | Self::B3 => RatingCategory::HighlySpeculative,
            Self::Caa1 | Self::Caa2 | Self::Caa3 | Self::Ca | Self::C => {
                RatingCategory::SubstantialRisk
            }
        }
    }

    /// Convert from quality score (0-150)
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            145..=150 => Self::Aaa,
            140..=144 => Self::Aa1,
            135..=139 => Self::Aa2,
            130..=134 => Self::Aa3,
            125..=129 => Self::A1,
            120..=124 => Self::A2,
            115..=119 => Self::A3,
            110..=114 => Self::Baa1,
            105..=109 => Self::Baa2,
            100..=104 => Self::Baa3,
            90..=99 => Self::Ba1,
            80..=89 => Self::Ba2,
            70..=79 => Self::Ba3,
            60..=69 => Self::B1,
            50..=59 => Self::B2,
            40..=49 => Self::B3,
            30..=39 => Self::Caa1,
            20..=29 => Self::Caa2,
            10..=19 => Self::Caa3,
            5..=9 => Self::Ca,
            _ => Self::C,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aaa => "Aaa",
            Self::Aa1 => "Aa1",
            Self::Aa2 => "Aa2",
            Self::Aa3 => "Aa3",
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::Baa1 => "Baa1",
            Self::Baa2 => "Baa2",
            Self::Baa3 => "Baa3",
            Self::Ba1 => "Ba1",
            Self::Ba2 => "Ba2",
            Self::Ba3 => "Ba3",
            Self::B1 => "B1",
            Self::B2 => "B2",
            Self::B3 => "B3",
            Self::Caa1 => "Caa1",
            Self::Caa2 => "Caa2",
            Self::Caa3 => "Caa3",
            Self::Ca => "Ca",
            Self::C => "C",
        }
    }

    /// Long-form description
    pub fn description(&self) -> &'static str {
        match self.category() {
            RatingCategory::Prime => "Prime quality, minimal risk",
            RatingCategory::HighGrade => "High quality, very low risk",
            RatingCategory::MediumGrade => "Medium quality, moderate risk",
            RatingCategory::Speculative => "Speculative, notable risk",
            RatingCategory::HighlySpeculative => "Highly speculative, substantial risk",
            RatingCategory::SubstantialRisk => "High risk, potential for significant issues",
        }
    }
}

/// Rating category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingCategory {
    Prime,
    HighGrade,
    MediumGrade,
    Speculative,
    HighlySpeculative,
    SubstantialRisk,
}

impl RatingCategory {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Prime => "🏆",
            Self::HighGrade => "⭐",
            Self::MediumGrade => "✓",
            Self::Speculative => "⚠️",
            Self::HighlySpeculative => "🔶",
            Self::SubstantialRisk => "🔴",
        }
    }
}

/// Rating outlook (future direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingOutlook {
    /// Rating likely to improve
    Positive,
    /// Rating expected to remain stable
    Stable,
    /// Rating may decline
    Negative,
    /// Under review for potential change
    UnderReview,
}

impl RatingOutlook {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Positive => "📈",
            Self::Stable => "➡️",
            Self::Negative => "📉",
            Self::UnderReview => "🔍",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "Positive",
            Self::Stable => "Stable",
            Self::Negative => "Negative",
            Self::UnderReview => "Under Review",
        }
    }
}

/// Complete rating assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingAssessment {
    pub skill_ref: String,
    pub rating: SkillRating,
    pub outlook: RatingOutlook,
    pub assessed_at: DateTime<Utc>,
    pub next_review: Option<DateTime<Utc>>,
    /// Rating factors
    pub factors: RatingFactors,
    /// Rating history
    pub history: Vec<RatingChange>,
    /// Analyst notes
    pub notes: Option<String>,
}

impl RatingAssessment {
    pub fn new(skill_ref: impl Into<String>, rating: SkillRating) -> Self {
        Self {
            skill_ref: skill_ref.into(),
            rating,
            outlook: RatingOutlook::Stable,
            assessed_at: Utc::now(),
            next_review: None,
            factors: RatingFactors::default(),
            history: Vec::new(),
            notes: None,
        }
    }

    /// Upgrade the rating
    pub fn upgrade(&mut self, new_rating: SkillRating, reason: impl Into<String>) {
        let old = self.rating;
        self.history.push(RatingChange {
            from: old,
            to: new_rating,
            action: RatingAction::Upgrade,
            reason: reason.into(),
            timestamp: Utc::now(),
        });
        self.rating = new_rating;
        self.assessed_at = Utc::now();
    }

    /// Downgrade the rating
    pub fn downgrade(&mut self, new_rating: SkillRating, reason: impl Into<String>) {
        let old = self.rating;
        self.history.push(RatingChange {
            from: old,
            to: new_rating,
            action: RatingAction::Downgrade,
            reason: reason.into(),
            timestamp: Utc::now(),
        });
        self.rating = new_rating;
        self.assessed_at = Utc::now();
    }

    /// Affirm current rating
    pub fn affirm(&mut self) {
        self.history.push(RatingChange {
            from: self.rating,
            to: self.rating,
            action: RatingAction::Affirmed,
            reason: "Rating affirmed on review".to_string(),
            timestamp: Utc::now(),
        });
        self.assessed_at = Utc::now();
    }
}

/// Rating factors breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RatingFactors {
    /// Quality/technical score (40%)
    pub quality_score: f64,
    /// Security posture (20%)
    pub security_score: f64,
    /// Governance maturity (15%)
    pub governance_score: f64,
    /// Track record/history (15%)
    pub track_record_score: f64,
    /// Maintainer reliability (10%)
    pub maintainer_score: f64,
}

impl RatingFactors {
    /// Calculate weighted composite
    pub fn composite(&self) -> f64 {
        (self.quality_score * 0.40)
            + (self.security_score * 0.20)
            + (self.governance_score * 0.15)
            + (self.track_record_score * 0.15)
            + (self.maintainer_score * 0.10)
    }
}

/// Rating change history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingChange {
    pub from: SkillRating,
    pub to: SkillRating,
    pub action: RatingAction,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingAction {
    Upgrade,
    Downgrade,
    Affirmed,
    InitialRating,
    Withdrawn,
}

/// Rating agency (who assigned the rating)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingAgency {
    pub name: String,
    pub id: String,
    pub verified: bool,
}

/// Standard agencies
pub mod agencies {
    use super::*;

    pub fn skillpack_agency() -> RatingAgency {
        RatingAgency {
            name: "SkillPack".to_string(),
            id: "skillpack".to_string(),
            verified: true,
        }
    }

    pub fn community_agency() -> RatingAgency {
        RatingAgency {
            name: "Community".to_string(),
            id: "community".to_string(),
            verified: false,
        }
    }
}

/// Rating watch alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingWatch {
    pub skill_ref: String,
    pub current_rating: SkillRating,
    pub watch_direction: WatchDirection,
    pub reason: String,
    pub placed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchDirection {
    UpgradePossible,
    DowngradePossible,
    UncertainDirection,
}
