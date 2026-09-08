//! Assessment Aggregate Root
//!
//! Core domain aggregate for skill quality assessment.

use crate::{AssessmentProfile, BonusPoints, DimensionId, Grade, Score, dimensions_for};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Assessment aggregate root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub id: AssessmentId,
    pub skill: SkillIdentity,
    pub dimension_scores: HashMap<DimensionId, Score>,
    pub bonus_points: BonusPoints,
    pub issues: Vec<Issue>,
    pub assessed_at: DateTime<Utc>,
    /// Count of dimensions whose checkers are stubs (no content analysis).
    pub stub_count: u32,
    /// Rubric this skill was assessed under (drives dimension weights).
    #[serde(default)]
    pub profile: AssessmentProfile,
    /// Dimensions the grader has GRANTED this skill an exemption from. Granted
    /// dimensions are excluded from the weighted score, which is re-normalized
    /// over the remaining dimensions so the skill is graded fairly on what
    /// applies. Ungranted requests never reach here — they fail loud upstream.
    #[serde(default)]
    pub exempt_dimensions: HashSet<DimensionId>,
}

/// Assessment identifier (UUID)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssessmentId(pub String);

/// Skill identity value object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIdentity {
    pub name: String,
    pub version: String,
    pub path: String,
}

impl SkillIdentity {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            path: path.into(),
        }
    }
}

impl std::fmt::Display for SkillIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Quality issue found during assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub dimension: DimensionId,
    pub severity: Severity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Assessment {
    /// Create new assessment
    pub fn new(skill: SkillIdentity) -> Self {
        Self {
            id: AssessmentId(uuid::Uuid::new_v4().to_string()),
            skill,
            dimension_scores: HashMap::new(),
            bonus_points: BonusPoints::default(),
            issues: Vec::new(),
            assessed_at: Utc::now(),
            stub_count: 0,
            profile: AssessmentProfile::default(),
            exempt_dimensions: HashSet::new(),
        }
    }

    /// Produce a JSON payload mapping dimension names to scores.
    pub fn dimensions_payload(&self) -> serde_json::Value {
        let map: serde_json::Map<String, serde_json::Value> = self
            .dimension_scores
            .iter()
            .map(|(d, s)| {
                (
                    d.name().to_string(),
                    serde_json::json!({"score": s.value()}),
                )
            })
            .collect();
        serde_json::Value::Object(map)
    }

    /// Record dimension score
    pub fn record_dimension(&mut self, dimension: DimensionId, score: Score) {
        self.dimension_scores.insert(dimension, score);
    }

    /// Add issue
    pub fn add_issue(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    /// Calculate base score (weighted per assessment profile).
    ///
    /// Granted exemptions are dropped and the remaining dimension weights are
    /// re-normalized to sum to 1.0, so an exempt skill is scored only on the
    /// dimensions that apply to it — never penalized for a dimension the
    /// grader agreed it cannot satisfy.
    pub fn base_score(&self) -> Score {
        let dimensions = dimensions_for(self.profile);
        let active: Vec<_> = dimensions
            .iter()
            .filter(|d| !self.exempt_dimensions.contains(&d.id))
            .collect();
        let total_weight: f64 = active.iter().map(|d| d.weight.as_fraction()).sum();
        if total_weight == 0.0 {
            return Score::new(0.0);
        }
        let weighted_sum: f64 = active
            .iter()
            .map(|d| {
                let score = self
                    .dimension_scores
                    .get(&d.id)
                    .map(|s| s.value())
                    .unwrap_or(0.0);
                score * (d.weight.as_fraction() / total_weight)
            })
            .sum();
        Score::new(weighted_sum)
    }

    /// Calculate total score (base + bonus)
    pub fn total_score(&self) -> Score {
        Score::new(self.base_score().value() + self.bonus_points.total() as f64)
    }

    /// Derive grade from total score
    pub fn grade(&self) -> Grade {
        Grade::from_score(self.total_score().value())
    }

    /// Check if assessment meets minimum grade
    pub fn meets_minimum(&self, minimum: Grade) -> bool {
        self.grade().meets_minimum(&minimum)
    }
}

#[cfg(test)]
mod exemption_tests {
    use super::*;

    fn full_marks(profile: AssessmentProfile) -> Assessment {
        let mut a = Assessment::new(SkillIdentity::new("t", "1", "/t"));
        a.profile = profile;
        for d in DimensionId::all() {
            a.record_dimension(d, Score::dimension(100.0));
        }
        a
    }

    #[test]
    fn exempting_a_zero_dimension_raises_the_score() {
        // Score everything 100 except Testing = 0.
        let mut a = full_marks(AssessmentProfile::AgentSkills);
        a.record_dimension(DimensionId::Testing, Score::dimension(0.0));
        let before = a.base_score().value();

        // Grant exemption from Testing: it drops out and weights re-normalize.
        a.exempt_dimensions.insert(DimensionId::Testing);
        let after = a.base_score().value();

        assert!(
            after > before,
            "exempting a failing dimension should raise the score: {} !> {}",
            after,
            before
        );
        // With every remaining dimension at 100, the re-normalized base is 100.
        assert!((after - 100.0).abs() < 1e-6, "expected ~100, got {}", after);
    }

    #[test]
    fn exempting_all_dimensions_is_zero_not_nan() {
        let mut a = full_marks(AssessmentProfile::AgentSkills);
        for d in DimensionId::all() {
            a.exempt_dimensions.insert(d);
        }
        let s = a.base_score().value();
        assert!(s.is_finite());
        assert_eq!(s, 0.0);
    }
}

// Simple UUID implementation for no-std compatibility
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }
    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            write!(f, "{:032x}", nanos)
        }
    }
}
