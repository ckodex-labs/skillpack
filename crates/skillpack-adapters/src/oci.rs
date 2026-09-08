//! OCI Registry Adapter
//!
//! Implements SkillReader for OCI registries using oci-client.
//! Supports push, pull, and Sigstore signature verification.

use anyhow::Result;
use sha2::{Digest, Sha256};
use skillpack_domain::{
    BundleReader, BundleReaderError, OciSource, SkillBundle, SkillIdentity, SkillReader,
    SkillReaderError,
};
use std::path::Path;

/// OCI artifact media types for SkillPack
pub mod media_types {
    /// CNSB skill bundle
    pub const CNSB_BUNDLE: &str = "application/vnd.ckodex.cnsb.v1+json";
    /// CNAAB agent app bundle  
    pub const CNAAB_BUNDLE: &str = "application/vnd.ckodex.cnaab.v1+json";
    /// Evidence envelope
    pub const EVIDENCE_ENVELOPE: &str = "application/vnd.ckodex.evidence.v1+json";
    /// Policy bundle
    pub const POLICY_BUNDLE: &str = "application/vnd.ckodex.policy.v1+json";
    /// Config blob
    pub const CONFIG: &str = "application/vnd.ckodex.skillpack.config.v1+json";

    // AIPACK §4.1 / §4.2 — canonical media types for A4 Skill artifacts
    /// AIPACK A4 Skill manifest (OCI image manifest mediaType)
    pub const AIPACK_SKILL_MANIFEST: &str = "application/vnd.ai.skill.v1+json";
    /// AIPACK A4 Skill bundle content layer (tar+zstd)
    pub const AIPACK_SKILL_BUNDLE: &str = "application/vnd.ai.skill.bundle.v1.tar+zstd";
    /// AIPACK in-toto attestation referrer
    pub const AIPACK_ATTESTATION: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
}

/// Build an OCI client config, choosing plain HTTP for a loopback registry
/// (localhost / 127.0.0.1 / ::1) or when `SKILLPACK_OCI_PLAIN_HTTP=1`. All other
/// hosts — including `.local` mDNS names — keep the default HTTPS + TLS
/// verification. This is what lets a locally-run registry (e.g.
/// ckx-oci-distribution's adapter on localhost:8080) be used without TLS.
/// True only for genuine loopback literals. `.local` mDNS names are network-
/// reachable and trivially spoofable, so they are deliberately NOT treated as
/// local: auto-downgrading them to cleartext HTTP would be a silent MITM
/// primitive. To use plaintext against a `.local` (or any remote) registry, the
/// operator must opt in explicitly with `SKILLPACK_OCI_PLAIN_HTTP=1`.
fn is_loopback_host(host_only: &str) -> bool {
    matches!(host_only, "localhost" | "127.0.0.1" | "::1")
}

fn oci_client_config(reference: &str) -> oci_client::client::ClientConfig {
    use oci_client::client::{ClientConfig, ClientProtocol};

    let host = reference.split('/').next().unwrap_or(reference);
    let host_only = host.split(':').next().unwrap_or(host);
    let plain = is_loopback_host(host_only)
        || std::env::var("SKILLPACK_OCI_PLAIN_HTTP").as_deref() == Ok("1");

    let mut config = ClientConfig::default();
    if plain {
        config.protocol = ClientProtocol::Http;
    }
    config
}

/// Build the OCI base-endpoint probe URL for a registry reference: `/v2/`,
/// with http for a loopback registry (localhost / 127.0.0.1 / ::1) and https
/// otherwise, unless the reference already carries a scheme. Used by
/// `skillpack doctor` to health-check the configured registry.
pub fn registry_probe_url(registry: &str) -> String {
    if registry.contains("://") {
        return format!("{}/v2/", registry.trim_end_matches('/'));
    }
    let host_only = registry.split(':').next().unwrap_or(registry);
    let scheme = if is_loopback_host(host_only) {
        "http"
    } else {
        "https"
    };
    format!("{}://{}/v2/", scheme, registry.trim_end_matches('/'))
}

/// OCI Registry skill reader
pub struct OciReader {
    source: OciSource,
    cache_dir: std::path::PathBuf,
}

impl OciReader {
    pub fn new(source: OciSource) -> Self {
        let cache_dir = std::env::temp_dir().join("skillpack-oci-cache");
        Self { source, cache_dir }
    }

    pub fn with_cache_dir(source: OciSource, cache_dir: std::path::PathBuf) -> Self {
        Self { source, cache_dir }
    }

    /// Pull artifact from OCI registry to local cache
    pub async fn pull(&self) -> Result<std::path::PathBuf> {
        use oci_client::Client;

        let reference = self.source.reference();
        let local_path = self
            .cache_dir
            .join(&self.source.repository)
            .join(&self.source.tag);

        // Create cache directory
        std::fs::create_dir_all(&local_path)?;

        // Pull using oci-client - simplified for 0.16 API
        let config = oci_client_config(&reference);
        let client = Client::new(config);

        let image_ref: oci_client::Reference = reference
            .parse()
            .map_err(|e: oci_client::ParseError| anyhow::anyhow!("Invalid reference: {}", e))?;

        // Pull manifest and config
        let (manifest, _) = client
            .pull_manifest(&image_ref, &oci_client::secrets::RegistryAuth::Anonymous)
            .await?;

        // Store manifest as metadata
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(local_path.join("manifest.json"), manifest_json)?;

        Ok(local_path)
    }

    /// Push skill bundle to OCI registry
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

    /// Package a skill directory into a gzipped tar layer (bytes).
    pub fn package_skill_directory(skill_path: &std::path::Path) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let enc = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.append_dir_all(".", skill_path)?;
            tar.finish()?;
        }
        Ok(buf)
    }

    /// Push a skill directory as an OCI artifact to a registry.
    pub async fn push_skill(
        registry: &str,
        repository: &str,
        tag: &str,
        skill_path: &std::path::Path,
        auth: Option<(&str, &str)>,
    ) -> Result<OciPushResult> {
        let content = Self::package_skill_directory(skill_path)?;
        Self::push(
            registry,
            repository,
            tag,
            &content,
            media_types::AIPACK_SKILL_BUNDLE,
            auth,
        )
        .await
    }

    /// Compute SHA256 digest
    pub fn compute_digest(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("sha256:{}", hex::encode(hasher.finalize()))
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

impl SkillReader for OciReader {
    fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
        // For OCI, identity comes from manifest annotations
        Ok(SkillIdentity {
            name: self.source.repository.clone(),
            version: self.source.tag.clone(),
            path: self.source.reference(),
        })
    }

    fn file_exists(&self, _path: &Path, relative: &str) -> bool {
        let cached_path = self
            .cache_dir
            .join(&self.source.repository)
            .join(&self.source.tag);
        cached_path.join(relative).exists()
    }

    fn read_file(&self, _path: &Path, relative: &str) -> Result<String, SkillReaderError> {
        let cached_path = self
            .cache_dir
            .join(&self.source.repository)
            .join(&self.source.tag);
        std::fs::read_to_string(cached_path.join(relative))
            .map_err(|e| SkillReaderError::ReadError(e.to_string()))
    }

    fn list_files(&self, _path: &Path, pattern: &str) -> Vec<String> {
        let cached_path = self
            .cache_dir
            .join(&self.source.repository)
            .join(&self.source.tag);
        let regex = regex::Regex::new(pattern).ok();
        walkdir::WalkDir::new(&cached_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                regex
                    .as_ref()
                    .is_none_or(|r| r.is_match(&e.path().to_string_lossy()))
            })
            .map(|e| {
                e.path()
                    .strip_prefix(&cached_path)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }
}

impl BundleReader for OciReader {
    fn read_bundle(&self, _path: &Path) -> Result<SkillBundle, BundleReaderError> {
        let bundles = self.list_bundles(Path::new(""));
        if bundles.is_empty() {
            return Err(BundleReaderError::NotFound(
                "No CNSB bundle in OCI artifact".to_string(),
            ));
        }
        let cached_path = self
            .cache_dir
            .join(&self.source.repository)
            .join(&self.source.tag);
        let content = std::fs::read_to_string(cached_path.join(&bundles[0]))
            .map_err(|e| BundleReaderError::ReadError(e.to_string()))?;
        SkillBundle::from_json(&content)
            .map_err(|e| BundleReaderError::InvalidFormat(e.to_string()))
    }

    fn list_bundles(&self, _path: &Path) -> Vec<String> {
        self.list_files(Path::new(""), r"\.cnsb\.json$")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oci_client::client::ClientProtocol;

    fn protocol_of(reference: &str) -> ClientProtocol {
        oci_client_config(reference).protocol
    }

    #[test]
    fn loopback_registries_use_plain_http() {
        for r in [
            "localhost:8080/skills/x:1",
            "127.0.0.1:5000/x:1",
            "localhost/x:1",
        ] {
            assert!(
                matches!(protocol_of(r), ClientProtocol::Http),
                "{} should use HTTP",
                r
            );
        }
    }

    #[test]
    fn dot_local_is_not_auto_downgraded_to_http() {
        // SEC: `.local` is mDNS-spoofable and network-reachable, so it must NOT
        // be silently downgraded to cleartext (that would be a MITM primitive).
        assert!(
            !matches!(protocol_of("registry.local/x:1"), ClientProtocol::Http),
            "registry.local must stay HTTPS unless explicitly opted in"
        );
    }

    #[test]
    fn remote_registries_keep_https() {
        for r in ["ghcr.io/ckodex/x:1", "registry.skillpack.dev/x:1"] {
            assert!(
                !matches!(protocol_of(r), ClientProtocol::Http),
                "{} should stay HTTPS",
                r
            );
        }
    }

    #[test]
    fn probe_url_local_is_http_v2() {
        assert_eq!(
            registry_probe_url("localhost:8080"),
            "http://localhost:8080/v2/"
        );
        assert_eq!(
            registry_probe_url("127.0.0.1:5000"),
            "http://127.0.0.1:5000/v2/"
        );
    }

    #[test]
    fn probe_url_remote_is_https_v2() {
        assert_eq!(
            registry_probe_url("registry.skillpack.dev"),
            "https://registry.skillpack.dev/v2/"
        );
    }

    #[test]
    fn probe_url_respects_explicit_scheme() {
        assert_eq!(
            registry_probe_url("https://reg.example.com/"),
            "https://reg.example.com/v2/"
        );
    }
}
