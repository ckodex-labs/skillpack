//! Assessment Exemption Policy
//!
//! Some skills legitimately cannot satisfy every dimension — a pure-reference
//! data skill has no tests to run, a meta/routing skill has no runtime to
//! provenance. Rather than grade them unfairly or relax the bar globally, a
//! skill may *request* a dimension exemption in its frontmatter:
//!
//! ```yaml
//! x-skillpack-exempt: [testing, evals_hitl]
//! ```
//!
//! The anti-gaming property is that the **grant lives outside the graded
//! artifact**. A skill cannot exempt itself: the [`ExemptionPolicy`] is owned
//! by the grader (code default or grader-side config), and a request that the
//! policy does not grant is a *loud failure*, not a silent pass — the
//! exemption is ignored and a hard issue is recorded.

use crate::DimensionId;
use std::collections::{HashMap, HashSet};

/// Grader-owned map of skill name → dimensions that skill may skip.
#[derive(Debug, Clone, Default)]
pub struct ExemptionPolicy {
    grants: HashMap<String, HashSet<DimensionId>>,
}

impl ExemptionPolicy {
    /// Empty policy — nothing is exempt. The safe default: every exemption
    /// request fails loud until the grader explicitly grants it.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build a policy from (skill_name, dimensions) grants.
    pub fn from_grants<I>(grants: I) -> Self
    where
        I: IntoIterator<Item = (String, Vec<DimensionId>)>,
    {
        Self {
            grants: grants
                .into_iter()
                .map(|(name, dims)| (name.to_lowercase(), dims.into_iter().collect()))
                .collect(),
        }
    }

    /// Does the grader grant `skill` an exemption from `dim`?
    pub fn grants(&self, skill: &str, dim: DimensionId) -> bool {
        self.grants
            .get(&skill.to_lowercase())
            .map(|dims| dims.contains(&dim))
            .unwrap_or(false)
    }

    /// True if any grant exists (used to decide whether to consult the policy).
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }
}

/// Parse a dimension name as it appears in `x-skillpack-exempt`. Accepts the
/// canonical names and common lowercase/snake variants.
pub fn parse_exempt_dimension(s: &str) -> Option<DimensionId> {
    match s.trim().to_lowercase().replace(['-', ' '], "_").as_str() {
        "identityandmanifest" | "identity" | "identity_and_manifest" | "manifest" => {
            Some(DimensionId::IdentityAndManifest)
        }
        "security" => Some(DimensionId::Security),
        "provenance" => Some(DimensionId::Provenance),
        "documentation" | "docs" => Some(DimensionId::Documentation),
        "testing" | "tests" => Some(DimensionId::Testing),
        "compatibility" | "compat" => Some(DimensionId::Compatibility),
        "lifecycle" => Some(DimensionId::Lifecycle),
        "governance" => Some(DimensionId::Governance),
        "evalshitl" | "evals_hitl" | "evals" | "hitl" => Some(DimensionId::EvalsHitl),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_policy_grants_nothing() {
        let p = ExemptionPolicy::empty();
        assert!(!p.grants("anything", DimensionId::Testing));
        assert!(p.is_empty());
    }

    #[test]
    fn grants_are_case_insensitive_and_scoped() {
        let p = ExemptionPolicy::from_grants([(
            "Skill-Steward".to_string(),
            vec![DimensionId::Testing, DimensionId::EvalsHitl],
        )]);
        assert!(p.grants("skill-steward", DimensionId::Testing));
        assert!(p.grants("SKILL-STEWARD", DimensionId::EvalsHitl));
        // Not granted for a dimension it wasn't given.
        assert!(!p.grants("skill-steward", DimensionId::Security));
        // Not granted for a different skill.
        assert!(!p.grants("other", DimensionId::Testing));
    }

    #[test]
    fn parse_dimension_accepts_variants() {
        assert_eq!(
            parse_exempt_dimension("testing"),
            Some(DimensionId::Testing)
        );
        assert_eq!(parse_exempt_dimension("Tests"), Some(DimensionId::Testing));
        assert_eq!(
            parse_exempt_dimension("evals_hitl"),
            Some(DimensionId::EvalsHitl)
        );
        assert_eq!(
            parse_exempt_dimension("identity"),
            Some(DimensionId::IdentityAndManifest)
        );
        assert_eq!(parse_exempt_dimension("nonsense"), None);
    }
}
