//! Dimension Checkers
//!
//! Implements DimensionChecker port for each quality dimension.
//! Each checker lives in its own module for independent evolution.

pub mod common;
pub mod compatibility;
pub mod documentation;
pub mod evals_hitl;
pub mod governance;
pub mod identity;
pub mod lifecycle;
pub mod provenance;
pub mod secrets;
pub mod security;
pub mod testing;

use skillpack_domain::DimensionChecker;

pub fn all_checkers() -> Vec<Box<dyn DimensionChecker>> {
    vec![
        Box::new(identity::IdentityChecker),
        Box::new(security::SecurityChecker),
        Box::new(provenance::ProvenanceChecker),
        Box::new(documentation::DocumentationChecker),
        Box::new(testing::TestingChecker),
        Box::new(compatibility::CompatibilityChecker),
        Box::new(lifecycle::LifecycleChecker),
        Box::new(governance::GovernanceChecker),
        Box::new(evals_hitl::EvalsHitlChecker),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::DimensionId;
    use std::collections::HashSet;

    #[test]
    fn all_checkers_covers_all_dimensions() {
        let checkers = all_checkers();
        assert_eq!(
            checkers.len(),
            9,
            "expected 9 checkers, got {}",
            checkers.len()
        );
        let covered: HashSet<DimensionId> = checkers.iter().map(|c| c.dimension()).collect();
        for dim in DimensionId::all() {
            assert!(
                covered.contains(&dim),
                "dimension {:?} not covered by any checker",
                dim
            );
        }
    }

    /// Calibration contract: the assessor must rank real-world skills by
    /// content quality, not by presence of CNSB packaging boilerplate.
    /// Regression guard for the 400-of-404-skills-grade-F failure mode.
    #[test]
    fn agentskills_profile_discriminates_quality() {
        use crate::filesystem::FilesystemReader;
        use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
        use skillpack_domain::AssessmentProfile;
        use std::path::PathBuf;

        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/skills");

        let assess = |dir: &str| {
            let use_case = AssessSkillUseCase::new(FilesystemReader::new(), all_checkers());
            use_case
                .execute(AssessSkillRequest {
                    skill_path: fixtures.join(dir).to_str().unwrap().to_string(),
                    min_score: None,
                })
                .unwrap()
                .assessment
        };

        let rich = assess("agentskills-rich");
        let minimal = assess("agentskills-only");
        let cnsb = assess("known-a");

        // Profile detection is structural
        assert_eq!(rich.profile, AssessmentProfile::AgentSkills);
        assert_eq!(minimal.profile, AssessmentProfile::AgentSkills);
        assert_eq!(cnsb.profile, AssessmentProfile::Cnsb);

        // A complete agent skill must reach at least the B band (80+)
        let rich_total = rich.total_score().value();
        assert!(
            rich_total >= 80.0,
            "rich agentskills fixture should reach B band, got {}",
            rich_total
        );

        // Quality must discriminate: rich clearly above a bare-minimum skill
        let minimal_total = minimal.total_score().value();
        assert!(
            rich_total >= minimal_total + 15.0,
            "rich ({}) must outscore minimal ({}) by a full band",
            rich_total,
            minimal_total
        );

        // CNSB skills keep the strict bar (fixture is a known-A)
        assert!(
            cnsb.total_score().value() >= 90.0,
            "known-a CNSB fixture regressed below A: {}",
            cnsb.total_score().value()
        );
    }

    /// Anti-gaming contract: a skill cannot exempt itself. An ungranted
    /// `x-skillpack-exempt` request fails loud and is ignored; a grader-granted
    /// one is honored and re-normalizes the score.
    #[test]
    fn exemption_requires_grader_grant() {
        use crate::filesystem::FilesystemReader;
        use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
        use skillpack_domain::{DimensionId, ExemptionPolicy};
        use std::path::PathBuf;

        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/skills/exempt-request");
        let path = fixture.to_str().unwrap().to_string();

        let assess = |policy: ExemptionPolicy| {
            AssessSkillUseCase::new(FilesystemReader::new(), all_checkers())
                .with_exemption_policy(policy)
                .execute(AssessSkillRequest {
                    skill_path: path.clone(),
                    min_score: None,
                })
                .unwrap()
                .assessment
        };

        // Ungranted: request is refused loudly, dimensions stay in scope.
        let denied = assess(ExemptionPolicy::empty());
        assert!(denied.exempt_dimensions.is_empty());
        let unauthorized = denied
            .issues
            .iter()
            .filter(|i| i.message.contains("unauthorized exemption"))
            .count();
        assert_eq!(
            unauthorized, 2,
            "expected 2 unauthorized-exemption issues, got {}",
            unauthorized
        );

        // Granted: the grader (outside the artifact) allows the exemption.
        let granted = assess(ExemptionPolicy::from_grants([(
            "exempt-request".to_string(),
            vec![DimensionId::Testing, DimensionId::EvalsHitl],
        )]));
        assert!(granted.exempt_dimensions.contains(&DimensionId::Testing));
        assert!(granted.exempt_dimensions.contains(&DimensionId::EvalsHitl));
        assert_eq!(
            granted
                .issues
                .iter()
                .filter(|i| i.message.contains("unauthorized exemption"))
                .count(),
            0,
            "granted exemption must not raise an unauthorized issue"
        );

        // The granted score is computed only over the non-exempt dimensions
        // (re-normalized), so it differs from the full-rubric denied score.
        // Direction depends on whether the exempted dimensions scored above or
        // below the rest — the domain test covers the fair-raise case.
        assert_ne!(
            granted.base_score().value(),
            denied.base_score().value(),
            "exemption must change the weighting"
        );
    }

    #[test]
    fn report_meta_has_no_stub_dimensions() {
        use crate::filesystem::FilesystemReader;
        use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
        use std::path::PathBuf;

        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests/fixtures/skills/known-a");

        let checkers = all_checkers();
        let reader = FilesystemReader::new();
        let use_case = AssessSkillUseCase::new(reader, checkers);

        let response = use_case
            .execute(AssessSkillRequest {
                skill_path: fixture.to_str().unwrap().to_string(),
                min_score: None,
            })
            .unwrap();

        assert_eq!(
            response.stub_dimensions.len(),
            0,
            "expected 0 stub dimensions after finish-line, got {:?}",
            response.stub_dimensions,
        );
    }
}
