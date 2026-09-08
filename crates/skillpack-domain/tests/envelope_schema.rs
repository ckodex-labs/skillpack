use skillpack_application::envelope_builder::EnvelopeBuilder;

#[test]
fn unsigned_envelope_validates_shape() {
    let env = EnvelopeBuilder::new("urn:ckodex:skill:test:demo")
        .with_assessment(serde_json::json!({"overall": 92, "grade": "A"}))
        .build_unsigned();
    let json = serde_json::to_value(&env).unwrap();

    // Required fields present
    assert!(json["id"].is_string(), "id must be a string");
    assert!(json["subject"]["urn"].is_string(), "subject.urn required");
    assert_eq!(json["subject"]["kind"], "SkillBundle");
    assert_eq!(json["statement"]["type"], "SkillAssessment");
    assert!(json["issuedAt"].is_string(), "issuedAt required");
    assert_eq!(json["actor"]["type"], "agent");
    assert_eq!(json["actor"]["urn"], "skillpack");
    assert!(json["signatures"].is_array(), "signatures must be array");
    // unsigned = empty signatures array (signing happens in C0.2)
    assert_eq!(json["signatures"].as_array().unwrap().len(), 0);
}

#[test]
fn envelope_id_is_urn_prefixed() {
    let env = EnvelopeBuilder::new("urn:ckodex:skill:test:demo2")
        .with_assessment(serde_json::json!({}))
        .build_unsigned();
    assert!(
        env.id.starts_with("urn:ckodex:evidence:"),
        "id must be a ckodex evidence URN"
    );
}
