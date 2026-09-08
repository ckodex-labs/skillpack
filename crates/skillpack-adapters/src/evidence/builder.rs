//! EnvelopeBuilder — builds domain `EvidenceEnvelope` with statement.type = "SkillAssessment".

use chrono::Utc;
use serde_json::json;
use skillpack_domain::Assessment;
use skillpack_domain::envelope::{Actor, Chain, EvidenceEnvelope, EvidenceRef, Statement, Subject};
use uuid::Uuid;

pub struct EnvelopeBuilder {
    previous: Vec<String>,
}

pub struct EnvelopePending {
    statement: Statement,
    previous: Vec<String>,
}

impl Default for EnvelopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeBuilder {
    pub fn new() -> Self {
        Self {
            previous: Vec::new(),
        }
    }

    pub fn with_previous(mut self, prev: Vec<String>) -> Self {
        self.previous = prev;
        self
    }

    pub fn from_assessment(self, assessment: &Assessment) -> EnvelopePending {
        let payload = json!({
            "grade": assessment.grade().as_str(),
            "stub_count": assessment.stub_count,
            "dimensions": assessment.dimensions_payload(),
            "bonus_points": assessment.bonus_points.total(),
            "skill": {
                "name": assessment.skill.name,
                "version": assessment.skill.version,
                "path": assessment.skill.path,
            }
        });
        EnvelopePending {
            statement: Statement {
                type_: "SkillAssessment".into(),
                schema: None,
                payload,
            },
            previous: self.previous,
        }
    }
}

impl EnvelopePending {
    pub fn build(self) -> EvidenceEnvelope {
        let chain = if self.previous.is_empty() {
            None
        } else {
            Some(Chain {
                previous: self
                    .previous
                    .into_iter()
                    .map(|s| EvidenceRef { id: s, kind: None })
                    .collect(),
            })
        };

        EvidenceEnvelope {
            id: format!("urn:ckodex:evidence:{}", Uuid::new_v4()),
            subject: Subject {
                urn: "urn:ckodex:skill:unknown".into(),
                kind: "SkillBundle".into(),
                labels: None,
            },
            statement: self.statement,
            issued_at: Utc::now(),
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
