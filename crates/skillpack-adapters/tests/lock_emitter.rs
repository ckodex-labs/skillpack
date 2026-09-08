use skillpack_adapters::lock::LockEmitter;
use std::path::PathBuf;

#[test]
fn integrity_is_sha256_base64() {
    let h = LockEmitter::compute_integrity(b"hello");
    assert!(h.starts_with("sha256-"), "SRI prefix required");
    // sha256("hello") raw bytes base64 = "LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ="
    assert_eq!(h, "sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=");
}

#[test]
fn emit_produces_schema_compliant_lock() {
    let lock = LockEmitter::emit(
        "urn:ckodex:skill:tenant:demo",
        &[(PathBuf::from("SKILL.md"), "sha256-abc=".into())],
    );
    // Must match cnsb/v1/skill-lock.schema.json
    assert_eq!(lock["lockVersion"], 1);
    let deps = lock["dependencies"].as_object().unwrap();
    assert!(deps.contains_key("SKILL.md"));
    assert_eq!(deps["SKILL.md"]["integrity"], "sha256-abc=");
    assert_eq!(deps["SKILL.md"]["version"], "1.0.0");
    assert_eq!(deps["SKILL.md"]["resolved"], "SKILL.md");
    // Old fields must be absent (additionalProperties: false in schema)
    assert!(lock.get("apiVersion").is_none());
    assert!(lock.get("kind").is_none());
    assert!(lock.get("spec").is_none());
}

#[test]
fn emit_handles_multiple_files() {
    let files = vec![
        (PathBuf::from("SKILL.md"), "sha256-aaa=".into()),
        (PathBuf::from("README.md"), "sha256-bbb=".into()),
        (PathBuf::from("LICENSE"), "sha256-ccc=".into()),
    ];
    let lock = LockEmitter::emit("urn:ckodex:skill:tenant:multi", &files);
    let deps = lock["dependencies"].as_object().unwrap();
    assert_eq!(deps.len(), 3);
    assert!(deps.contains_key("SKILL.md"));
    assert!(deps.contains_key("README.md"));
    assert!(deps.contains_key("LICENSE"));
}

#[test]
fn emit_integrity_matches_schema_pattern() {
    let digests = vec![(
        std::path::PathBuf::from("SKILL.md"),
        LockEmitter::compute_integrity(b"hello world"),
    )];
    let lock = LockEmitter::emit("urn:ckodex:skill:test", &digests);
    let deps = lock["dependencies"].as_object().unwrap();
    let integrity = deps["SKILL.md"]["integrity"].as_str().unwrap();
    let re = regex::Regex::new(r"^sha(256|384|512)-[A-Za-z0-9+/=]+$").unwrap();
    assert!(
        re.is_match(integrity),
        "integrity '{}' does not match schema pattern",
        integrity
    );
}
