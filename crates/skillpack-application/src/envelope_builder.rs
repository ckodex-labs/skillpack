//! EnvelopeBuilder — fluent builder for unsigned EvidenceEnvelopes.
//!
//! Produces a well-formed [`EvidenceEnvelope`] ready for signing (C0.2).
//! Signing is intentionally out of scope here; `build_unsigned()` sets
//! `signatures` to an empty `Vec` as the pre-signing state.

use skillpack_domain::envelope::{Actor, Chain, EvidenceEnvelope, EvidenceRef, Statement, Subject};
use uuid::Uuid;

/// Builder for [`EvidenceEnvelope`].
///
/// # Example
/// ```no_run
/// use skillpack_application::envelope_builder::EnvelopeBuilder;
///
/// let env = EnvelopeBuilder::new("urn:ckodex:skill:acme:prod:core:my-skill")
///     .with_assessment(serde_json::json!({"overall": 88, "grade": "B+"}))
///     .build_unsigned();
/// ```
pub struct EnvelopeBuilder {
    subject_urn: String,
    payload: Option<serde_json::Value>,
    chain_previous: Vec<String>,
}

impl EnvelopeBuilder {
    /// Create a new builder for the given subject URN.
    pub fn new(subject_urn: impl Into<String>) -> Self {
        Self {
            subject_urn: subject_urn.into(),
            payload: None,
            chain_previous: vec![],
        }
    }

    /// Set the assessment payload (type = "SkillAssessment").
    pub fn with_assessment(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Attach chain-of-custody references to predecessor envelope URNs.
    pub fn with_chain(mut self, previous: Vec<String>) -> Self {
        self.chain_previous = previous;
        self
    }

    /// Build the envelope without signatures.
    ///
    /// The `signatures` field is an empty `Vec`; use the CosignSigner (C0.2)
    /// to produce a signed envelope.
    pub fn build_unsigned(self) -> EvidenceEnvelope {
        let chain = if self.chain_previous.is_empty() {
            None
        } else {
            Some(Chain {
                previous: self
                    .chain_previous
                    .into_iter()
                    .map(|s| EvidenceRef { id: s, kind: None })
                    .collect(),
            })
        };

        EvidenceEnvelope {
            id: format!("urn:ckodex:evidence:{}", Uuid::new_v4()),
            subject: Subject {
                urn: self.subject_urn,
                kind: "SkillBundle".into(),
                labels: None,
            },
            statement: Statement {
                type_: "SkillAssessment".into(),
                schema: None,
                payload: self.payload.unwrap_or(serde_json::Value::Null),
            },
            issued_at: chrono::Utc::now(),
            actor: Actor {
                kind: "agent".into(),
                id: "skillpack".into(),
                display_name: None,
                tenant_urn: None,
                labels: None,
            },
            gal: None,
            asc: vec![],
            chain,
            signatures: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_unsigned_sets_empty_signatures() {
        let env = EnvelopeBuilder::new("urn:ckodex:skill:test:unit")
            .with_assessment(serde_json::json!({"score": 75}))
            .build_unsigned();
        assert!(env.signatures.is_empty());
    }

    #[test]
    fn build_unsigned_id_is_evidence_urn() {
        let env = EnvelopeBuilder::new("urn:ckodex:skill:test:unit")
            .with_assessment(serde_json::json!({}))
            .build_unsigned();
        assert!(env.id.starts_with("urn:ckodex:evidence:"));
    }

    #[test]
    fn chain_none_when_no_previous() {
        let env = EnvelopeBuilder::new("urn:ckodex:skill:test:unit")
            .with_assessment(serde_json::json!({}))
            .build_unsigned();
        assert!(env.chain.is_none());
    }

    #[test]
    fn chain_some_when_previous_provided() {
        let env = EnvelopeBuilder::new("urn:ckodex:skill:test:unit")
            .with_assessment(serde_json::json!({}))
            .with_chain(vec!["urn:ckodex:evidence:prior".into()])
            .build_unsigned();
        assert!(env.chain.is_some());
        assert_eq!(env.chain.unwrap().previous.len(), 1);
    }
}
