//! CNSB skill-lock emitter.
//!
//! Emits a SkillLock JSON object conforming to the `schemas/cnsb/v1/skill-lock.schema.json`
//! schema (required top-level fields: `lockVersion`, `dependencies`).
//! Signing logic is handled separately (C0.2).

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Emits CNSB skill-lock JSON objects and computes SRI integrity strings.
pub struct LockEmitter;

impl LockEmitter {
    /// Compute SRI integrity string (`sha256-{base64std}`) for a byte slice.
    ///
    /// Uses standard base64 with padding, matching the SRI specification
    /// (W3C Subresource Integrity, §3.5).
    #[must_use]
    pub fn compute_integrity(bytes: &[u8]) -> String {
        let hash = Sha256::digest(bytes);
        let encoded = data_encoding::BASE64.encode(&hash);
        format!("sha256-{encoded}")
    }

    /// Emit a CNSB skill-lock JSON value conforming to `skill-lock.schema.json` v1.
    ///
    /// Produces:
    /// ```json
    /// {
    ///   "lockVersion": 1,
    ///   "dependencies": {
    ///     "<path>": { "version": "1.0.0", "resolved": "<path>", "integrity": "<sri>" }
    ///   },
    ///   "metadata": { "generated": "<rfc3339>", "generator": "skillpack@<version>" }
    /// }
    /// ```
    ///
    /// # Arguments
    /// * `_skill_urn` — The URN identifying the skill (reserved for future use in signing).
    /// * `file_digests` — Ordered list of `(path, integrity)` pairs. Each entry becomes a
    ///   dependency keyed by path. The `resolved` field is a placeholder until OCI resolver
    ///   is wired (post-C0.2).
    #[must_use]
    pub fn emit(_skill_urn: &str, file_digests: &[(PathBuf, String)]) -> serde_json::Value {
        // TODO(ckodex): wire _skill_urn into signing envelope when C0.2 signing is integrated
        let dependencies: serde_json::Map<String, serde_json::Value> = file_digests
            .iter()
            .map(|(path, integrity)| {
                let path_str = path.to_string_lossy().into_owned();
                let value = serde_json::json!({
                    // TODO(ckodex): version should come from the resolved dependency manifest, not a hardcoded placeholder
                    "version": "1.0.0",
                    // TODO(ckodex): replace with OCI resolved reference when registry integration is complete (post-C0.2)
                    "resolved": path_str,
                    "integrity": integrity
                });
                (path_str, value)
            })
            .collect();

        let generated = Utc::now().to_rfc3339();
        let generator = concat!("skillpack@", env!("CARGO_PKG_VERSION"));

        serde_json::json!({
            "lockVersion": 1,
            "dependencies": dependencies,
            "metadata": {
                "generated": generated,
                "generator": generator,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn compute_integrity_produces_sri_string() {
        let input = b"hello world";
        let result = LockEmitter::compute_integrity(input);
        assert!(result.starts_with("sha256-"), "must be SRI sha256 prefix");
        // Known SHA-256 of "hello world" in SRI base64
        assert_eq!(
            result,
            "sha256-uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek="
        );
    }

    #[test]
    fn emit_produces_schema_compliant_lock() {
        let digests = vec![(PathBuf::from("SKILL.md"), "sha256-abc123=".to_string())];
        let lock = LockEmitter::emit("urn:ckodex:skill:tenant:demo", &digests);

        assert_eq!(lock["lockVersion"], 1);

        let deps = lock["dependencies"].as_object().unwrap();
        assert!(deps.contains_key("SKILL.md"));
        assert_eq!(deps["SKILL.md"]["integrity"], "sha256-abc123=");
        assert_eq!(deps["SKILL.md"]["version"], "1.0.0");
        assert_eq!(deps["SKILL.md"]["resolved"], "SKILL.md");
    }

    #[test]
    fn emit_rejects_old_shape() {
        let digests = vec![(PathBuf::from("SKILL.md"), "sha256-abc123=".to_string())];
        let lock = LockEmitter::emit("urn:ckodex:skill:tenant:demo", &digests);

        assert!(
            lock.get("apiVersion").is_none(),
            "apiVersion must not be present"
        );
        assert!(lock.get("kind").is_none(), "kind must not be present");
        assert!(lock.get("spec").is_none(), "spec must not be present");
    }

    #[test]
    fn emit_empty_digests_produces_empty_dependencies() {
        let lock = LockEmitter::emit("urn:ckodex:skill:tenant:demo", &[]);
        assert_eq!(lock["lockVersion"], 1);
        let deps = lock["dependencies"].as_object().unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn emit_multiple_digests_all_present_in_dependencies() {
        let digests = vec![
            (PathBuf::from("SKILL.md"), "sha256-aaa=".to_string()),
            (PathBuf::from("src/main.py"), "sha256-bbb=".to_string()),
        ];
        let lock = LockEmitter::emit("urn:ckodex:skill:tenant:demo", &digests);
        let deps = lock["dependencies"].as_object().unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains_key("SKILL.md"));
        assert!(deps.contains_key("src/main.py"));
    }

    #[test]
    fn emit_metadata_contains_generator_and_generated() {
        let lock = LockEmitter::emit("urn:test", &[]);
        let meta = &lock["metadata"];
        assert!(
            meta["generator"]
                .as_str()
                .unwrap()
                .starts_with("skillpack@")
        );
        // generated must be a non-empty string (RFC 3339)
        assert!(!meta["generated"].as_str().unwrap().is_empty());
    }
}
