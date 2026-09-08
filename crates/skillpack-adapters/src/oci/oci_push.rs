//! OCI push operations: bundle and referrer pushes plus result types.
//! Split from oci.rs for the 500-LOC fence; impl OciReader continues here.

use super::{OciReader, media_types, oci_client_config};
use anyhow::Result;
use sha2::{Digest, Sha256};

impl OciReader {
    pub async fn push(
        registry: &str,
        repository: &str,
        tag: &str,
        content: &[u8],
        media_type: &str,
        auth: Option<(&str, &str)>,
    ) -> Result<OciPushResult> {
        use oci_client::Client;
        use oci_client::secrets::RegistryAuth;

        let reference = format!("{}/{}:{}", registry, repository, tag);
        let config = oci_client_config(&reference);
        let client = Client::new(config);

        let image_ref: oci_client::Reference = reference
            .parse()
            .map_err(|e: oci_client::ParseError| anyhow::anyhow!("Invalid reference: {}", e))?;

        let auth = match auth {
            Some((user, pass)) => RegistryAuth::Basic(user.to_string(), pass.to_string()),
            None => RegistryAuth::Anonymous,
        };

        // Compute digest
        let mut hasher = Sha256::new();
        hasher.update(content);
        let digest = format!("sha256:{}", hex::encode(hasher.finalize()));

        // Create config blob
        let config_content = serde_json::json!({
            "mediaType": media_types::CONFIG,
            "skillpack_version": "1.0.0",
            "created": chrono::Utc::now().to_rfc3339()
        });
        let config_bytes = serde_json::to_vec(&config_content)?;

        // Push config and layer
        let layers = vec![oci_client::client::ImageLayer {
            data: content.to_vec().into(),
            media_type: media_type.to_string(),
            annotations: None,
        }];

        let config_layer = oci_client::client::Config {
            data: config_bytes.into(),
            media_type: media_types::CONFIG.to_string(),
            annotations: None,
        };

        // Push to registry
        let push_response = client
            .push(&image_ref, &layers, config_layer, &auth, None)
            .await?;

        Ok(OciPushResult {
            reference: reference.clone(),
            digest,
            manifest_url: push_response.manifest_url,
        })
    }

    /// Verify Sigstore signature for an artifact using cosign
    pub async fn verify_signature(
        registry: &str,
        repository: &str,
        digest: &str,
    ) -> Result<SignatureVerification> {
        use std::process::Command;

        let image_ref = format!("{}/{}@{}", registry, repository, digest);

        let output = Command::new("cosign")
            .args([
                "verify",
                "--insecure-ignore-tlog",
                "--insecure-ignore-sct",
                &image_ref,
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute cosign: {}", e))?;

        if output.status.success() {
            let signer = String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|l| l.contains("Subject"))
                .map(|l| l.to_string());

            Ok(SignatureVerification {
                signed: true,
                signer,
                transparency_log: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no matching signatures") || stderr.contains("manifest not found") {
                Ok(SignatureVerification {
                    signed: false,
                    signer: None,
                    transparency_log: None,
                })
            } else {
                Err(anyhow::anyhow!("Verification failed: {}", stderr))
            }
        }
    }

    /// Push an AIPACK in-toto attestation as an OCI referrer on a subject digest.
    ///
    /// AIPACK §6.4 / §7 — each mandatory attestation predicate is pushed as a separate
    /// referrer manifest pointing to the subject skill artifact via `subjectDigest`.
    /// The payload must be a valid in-toto v1 Statement JSON object.
    ///
    /// # Arguments
    /// * `registry` - registry host (e.g. `ghcr.io`)
    /// * `repository` - repository path without tag (e.g. `ckodex/my-skill`)
    /// * `subject_digest` - `sha256:<hex>` digest of the skill artifact layer
    /// * `predicate_type` - AIPACK predicate URN (e.g. `urn:skill:static-analysis:v1`)
    /// * `payload` - serialised in-toto Statement JSON bytes
    /// * `auth` - optional (username, password) registry credentials
    pub async fn push_referrer(
        registry: &str,
        repository: &str,
        subject_digest: &str,
        predicate_type: &str,
        payload: &[u8],
        auth: Option<(&str, &str)>,
    ) -> Result<OciPushResult> {
        use oci_client::Client;
        use oci_client::secrets::RegistryAuth;

        // Tag the referrer manifest using the subject digest (OCI referrers API convention)
        let safe_digest = subject_digest.replace(':', "-");
        let reference = format!("{}/{}:{}", registry, repository, safe_digest);
        let config = oci_client_config(&reference);
        let client = Client::new(config);

        let image_ref: oci_client::Reference = reference
            .parse()
            .map_err(|e: oci_client::ParseError| anyhow::anyhow!("Invalid reference: {}", e))?;

        let auth = match auth {
            Some((user, pass)) => RegistryAuth::Basic(user.to_string(), pass.to_string()),
            None => RegistryAuth::Anonymous,
        };

        // Digest the payload
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let payload_digest = format!("sha256:{}", hex::encode(hasher.finalize()));

        // Referrer config blob — minimal OCI scratch config
        let config_content = serde_json::json!({
            "mediaType": media_types::AIPACK_ATTESTATION,
            "predicateType": predicate_type,
            "subjectDigest": subject_digest,
            "created": chrono::Utc::now().to_rfc3339()
        });
        let config_bytes = serde_json::to_vec(&config_content)?;

        let layers = vec![oci_client::client::ImageLayer {
            data: payload.to_vec().into(),
            media_type: media_types::AIPACK_ATTESTATION.to_string(),
            annotations: None,
        }];

        let config_layer = oci_client::client::Config {
            data: config_bytes.into(),
            media_type: media_types::AIPACK_ATTESTATION.to_string(),
            annotations: None,
        };

        let push_response = client
            .push(&image_ref, &layers, config_layer, &auth, None)
            .await?;

        Ok(OciPushResult {
            reference: reference.clone(),
            digest: payload_digest,
            manifest_url: push_response.manifest_url,
        })
    }
}
/// Result of pushing to OCI registry
#[derive(Debug, Clone)]
pub struct OciPushResult {
    pub reference: String,
    pub digest: String,
    pub manifest_url: String,
}

/// Signature verification result
#[derive(Debug, Clone)]
pub struct SignatureVerification {
    pub signed: bool,
    pub signer: Option<String>,
    pub transparency_log: Option<String>,
}
