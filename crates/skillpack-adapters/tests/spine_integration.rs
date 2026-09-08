use skillpack_adapters::checkers::all_checkers;
use skillpack_adapters::evidence::signer::NoopSigner;
use skillpack_adapters::filesystem::FilesystemReader;
use skillpack_application::AssessSkillUseCase;
use skillpack_application::envelope_builder::EnvelopeBuilder;
use skillpack_domain::ports::Signer;
use std::fs;
use tempfile::TempDir;

/// Run assessment on a skill fixture, build envelope, and sign.
fn assess_fixture(name: &str) -> anyhow::Result<skillpack_domain::EvidenceEnvelope> {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().join(name);
    fs::create_dir_all(&path)?;

    // Minimal skill structure
    fs::write(
        path.join("skill.cnsb.json"),
        serde_json::to_string(&serde_json::json!({
            "metadata": { "name": name, "version": "1.0.0" }
        }))?,
    )?;
    fs::write(path.join("SKILL.md"), "# Test Skill\n")?;
    fs::write(path.join("README.md"), "# README\n")?;

    let reader = FilesystemReader::new();
    let checkers = all_checkers();
    let use_case = AssessSkillUseCase::new(reader, checkers);

    let response = use_case.execute(skillpack_application::AssessSkillRequest {
        skill_path: path.to_string_lossy().to_string(),
        min_score: None,
    })?;

    let envelope = EnvelopeBuilder::new(&format!("urn:ckodex:skill:test:{name}"))
        .with_assessment(serde_json::json!({
            "overall": response.assessment.total_score().value(),
            "grade": response.assessment.grade().as_str(),
            "dimensions": response.assessment.dimension_scores.len(),
        }))
        .build_unsigned();

    let signer = NoopSigner;
    let signed = signer.sign(envelope)?;

    Ok(signed)
}

#[test]
fn fixture_emits_signed_envelope() {
    let env = assess_fixture("perfect-a").unwrap();
    assert_eq!(env.statement.type_, "SkillAssessment");
    assert!(!env.signatures.is_empty());
    assert_eq!(env.signatures[0].algo, "noop");
}

#[test]
fn fixture_envelope_has_evidence_urn_id() {
    let env = assess_fixture("perfect-a").unwrap();
    assert!(env.id.starts_with("urn:ckodex:evidence:"));
}
