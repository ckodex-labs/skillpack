//! Assess Skill Use Case

use anyhow::Result;
use rayon::prelude::*;
use skillpack_domain::{
    Assessment, AssessmentProfile, BonusPoints, DimensionChecker, DimensionId, ExemptionPolicy,
    Issue, Severity, SkillReader, parse_exempt_dimension,
};
use std::path::Path;

/// Request for assess skill use case
pub struct AssessSkillRequest {
    pub skill_path: String,
    pub min_score: Option<f64>,
}

/// Response from assess skill use case
pub struct AssessSkillResponse {
    pub assessment: Assessment,
    pub meets_minimum: bool,
    /// Names of dimensions whose checkers are stubs (no content analysis yet).
    /// Surface in report meta so callers can communicate coverage gaps.
    pub stub_dimensions: Vec<String>,
}

/// Assess skill use case
pub struct AssessSkillUseCase<R: SkillReader> {
    reader: R,
    checkers: Vec<Box<dyn DimensionChecker>>,
    /// Grader-owned exemption policy. Empty by default: nothing is exempt
    /// unless the grader explicitly grants it (outside the graded artifact).
    exemption_policy: ExemptionPolicy,
}

impl<R: SkillReader> AssessSkillUseCase<R> {
    pub fn new(reader: R, checkers: Vec<Box<dyn DimensionChecker>>) -> Self {
        Self {
            reader,
            checkers,
            exemption_policy: ExemptionPolicy::empty(),
        }
    }

    /// Attach a grader-owned exemption policy. The policy is not read from the
    /// skill under assessment — that's the anti-gaming property.
    pub fn with_exemption_policy(mut self, policy: ExemptionPolicy) -> Self {
        self.exemption_policy = policy;
        self
    }

    pub fn execute(&self, request: AssessSkillRequest) -> Result<AssessSkillResponse> {
        let path = Path::new(&request.skill_path);

        // Read skill identity
        let identity = self.reader.read_identity(path)?;

        // Create assessment under the detected rubric (CNSB vs agentskills)
        let mut assessment = Assessment::new(identity);
        assessment.profile = AssessmentProfile::detect(&self.reader, path);

        // Run dimension checks in parallel
        let results: Vec<_> =
            rayon::iter::ParallelIterator::collect(self.checkers.par_iter().map(|checker| {
                let (score, issues) = checker.check(&self.reader, path);
                (checker.dimension(), score, issues, checker.is_stub())
            }));

        for (dimension, score, issues, is_stub) in results {
            if is_stub {
                assessment.stub_count += 1;
            }
            assessment.record_dimension(dimension, score);
            for issue in issues {
                assessment.add_issue(issue);
            }
        }

        // Resolve exemption requests against the grader-owned policy.
        self.resolve_exemptions(path, &mut assessment);

        // Calculate bonus points
        assessment.bonus_points = self.calculate_bonus(path);

        // Check minimum
        let meets_minimum = match request.min_score {
            Some(min) => assessment.total_score().value() >= min,
            None => true,
        };

        // Collect stub dimension names for report meta
        let stub_dimensions: Vec<String> = self
            .checkers
            .iter()
            .filter(|c| c.is_stub())
            .map(|c| c.dimension().name().to_string())
            .collect();

        Ok(AssessSkillResponse {
            assessment,
            meets_minimum,
            stub_dimensions,
        })
    }

    /// Read `x-skillpack-exempt` from SKILL.md frontmatter and reconcile each
    /// requested dimension against the policy:
    /// - granted  → dimension excluded from scoring (re-normalized)
    /// - ungranted → hard Error issue; the exemption is ignored (fail loud)
    fn resolve_exemptions(&self, path: &Path, assessment: &mut Assessment) {
        let requested = self.read_exempt_requests(path);
        let skill_name = assessment.skill.name.clone();
        for dim in requested {
            if self.exemption_policy.grants(&skill_name, dim) {
                assessment.exempt_dimensions.insert(dim);
            } else {
                assessment.add_issue(Issue {
                    dimension: dim,
                    severity: Severity::Error,
                    message: format!(
                        "unauthorized exemption: '{}' requests exemption from {} but the grader has not granted it — request ignored",
                        skill_name,
                        dim.name()
                    ),
                    file: Some("SKILL.md".into()),
                    line: None,
                });
            }
        }
    }

    /// Parse the `x-skillpack-exempt` frontmatter list (array or comma string).
    fn read_exempt_requests(&self, path: &Path) -> Vec<DimensionId> {
        let Ok(content) = self.reader.read_file(path, "SKILL.md") else {
            return Vec::new();
        };
        let Some(fm) = content
            .strip_prefix("---\n")
            .and_then(|s| s.find("\n---").map(|e| &s[..e]))
        else {
            return Vec::new();
        };
        let Ok(parsed) = serde_yaml::from_str::<serde_yaml::Value>(fm) else {
            return Vec::new();
        };
        let raw = parsed.get("x-skillpack-exempt");
        let mut dims = Vec::new();
        match raw {
            Some(serde_yaml::Value::Sequence(seq)) => {
                for v in seq {
                    if let Some(s) = v.as_str()
                        && let Some(d) = parse_exempt_dimension(s)
                    {
                        dims.push(d);
                    }
                }
            }
            Some(serde_yaml::Value::String(s)) => {
                for part in s.split(',') {
                    if let Some(d) = parse_exempt_dimension(part) {
                        dims.push(d);
                    }
                }
            }
            _ => {}
        }
        dims
    }

    fn calculate_bonus(&self, path: &Path) -> BonusPoints {
        BonusPoints {
            slsa_level_3: self.reader.file_exists(path, "evidence/provenance.json"),
            sigstore_signing: self
                .reader
                .file_exists(path, ".github/workflows/publish-skill.yml"),
            dagger_pipeline: self.reader.file_exists(path, "ci/Cargo.toml"),
            stride_threat_model: self.reader.file_exists(path, "security/threat-model.yaml"),
            mcp_server: self.reader.file_exists(path, "capacities/mcp/server"),
        }
    }
}
