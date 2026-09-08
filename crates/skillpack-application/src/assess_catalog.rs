//! Assess Catalog Use Case
//!
//! Orchestrates assessing many skills (a canonical-store catalog) by reusing the
//! single-skill assessment. This is domain orchestration — it belongs in the
//! application layer, not in the transport/presentation layer that serves it.

use skillpack_domain::{DimensionChecker, SkillReader};

use crate::assess_skill::{AssessSkillRequest, AssessSkillUseCase};

/// One assessed skill in a catalog pass. Plain data the transport layer maps to
/// its wire format.
pub struct CatalogSkillResult {
    /// Directory name — the key the registry catalog is indexed by.
    pub name: String,
    /// Full skill path.
    pub path: String,
    pub grade: String,
    pub score: f64,
    pub profile: String,
    pub issue_count: usize,
    /// `(dimension_id, score)` pairs, in checker order.
    pub dimensions: Vec<(String, f64)>,
}

/// Assess an entire catalog of skills.
pub struct AssessCatalogUseCase<R: SkillReader> {
    inner: AssessSkillUseCase<R>,
}

impl<R: SkillReader> AssessCatalogUseCase<R> {
    pub fn new(reader: R, checkers: Vec<Box<dyn DimensionChecker>>) -> Self {
        Self {
            inner: AssessSkillUseCase::new(reader, checkers),
        }
    }

    /// Assess each skill directory in turn, invoking `on_result(done, total,
    /// result)` after each. `result` is `None` when a skill fails to assess, so
    /// the caller can still advance progress. Streaming (rather than returning a
    /// `Vec`) lets a caller fill a progressive cache without holding everything.
    pub fn assess_paths<F>(&self, skill_dirs: &[String], mut on_result: F)
    where
        F: FnMut(usize, usize, Option<CatalogSkillResult>),
    {
        let total = skill_dirs.len();
        for (idx, dir) in skill_dirs.iter().enumerate() {
            on_result(idx + 1, total, self.assess_one(dir));
        }
    }

    fn assess_one(&self, dir: &str) -> Option<CatalogSkillResult> {
        let response = self
            .inner
            .execute(AssessSkillRequest {
                skill_path: dir.to_string(),
                min_score: None,
            })
            .ok()?;
        let a = &response.assessment;
        let name = std::path::Path::new(dir)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| dir.to_string());
        Some(CatalogSkillResult {
            name,
            path: dir.to_string(),
            grade: a.grade().as_str().to_string(),
            score: a.total_score().value(),
            profile: a.profile.name().to_string(),
            issue_count: a.issues.len(),
            dimensions: a
                .dimension_scores
                .iter()
                .map(|(d, s)| (d.name().to_string(), s.value()))
                .collect(),
        })
    }
}
