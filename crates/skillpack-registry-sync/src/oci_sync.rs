//! OCI 1.2 registry sync — push and pull skill artifacts to/from Zot.
//!
//! Each skill is stored as an OCI artifact:
//!   - mediaType: application/vnd.ckodex.skill.v1+json
//!   - layer:     SKILL.md content (text/plain)
//!   - annotations: name, version, description, content_hash, source_agent

use anyhow::{Context, Result};
use oci_client::{
    Client, Reference,
    client::{ClientConfig, Config, ImageLayer},
    manifest::OciManifest,
    secrets::RegistryAuth,
};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

use crate::dedup::DeduplicatedSkill;

const SKILL_MEDIA_TYPE: &str = "application/vnd.ckodex.skill.v1+json";
const SKILL_LAYER_TYPE: &str = "text/plain; charset=utf-8";

/// OCI registry configuration.
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Registry host and optional port (e.g. "localhost:5000").
    pub endpoint: String,
    /// Repository prefix within the registry (e.g. "skills").
    pub repository: String,
    /// Whether to use HTTP instead of HTTPS (for local Zot).
    pub insecure: bool,
}

impl RegistryConfig {
    pub fn local_zot() -> Self {
        Self {
            endpoint: "localhost:5000".to_string(),
            repository: "skills".to_string(),
            insecure: true,
        }
    }

    pub fn oci_ref(&self, skill_name: &str, tag: &str) -> String {
        format!(
            "{}/{}/{}:{}",
            self.endpoint, self.repository, skill_name, tag
        )
    }
}

/// Push a single skill to the OCI registry.
///
/// Returns the manifest digest of the pushed artifact.
pub async fn push_skill(skill: &DeduplicatedSkill, config: &RegistryConfig) -> Result<String> {
    let skill_md = skill.canonical_path.join("SKILL.md");
    let content = if skill_md.exists() {
        std::fs::read(&skill_md).context("reading SKILL.md")?
    } else {
        format!("# {}\n\nNo SKILL.md found.\n", skill.canonical_name).into_bytes()
    };

    let client_config = build_client_config(config.insecure);
    let client = Client::new(client_config);
    let auth = RegistryAuth::Anonymous;

    let reference: Reference = config
        .oci_ref(&skill.canonical_name, &skill.content_hash[..12])
        .parse()
        .context("parsing OCI reference")?;

    // Layer: SKILL.md content
    let layer = ImageLayer::new(content, SKILL_LAYER_TYPE.to_string(), None);

    // Config: minimal JSON config blob
    let cfg = Config::new(b"{}".to_vec(), SKILL_MEDIA_TYPE.to_string(), None);

    let response = client
        .push(&reference, &[layer], cfg, &auth, None)
        .await
        .context("pushing skill to OCI registry")?;

    info!(skill = %skill.canonical_name, registry = %config.endpoint, "pushed skill");
    Ok(response.manifest_url)
}

/// Pull a skill from the OCI registry and write it to `dest_dir/<skill_name>/SKILL.md`.
pub async fn pull_skill(
    skill_name: &str,
    tag: &str,
    dest_dir: &Path,
    config: &RegistryConfig,
) -> Result<()> {
    let client_config = build_client_config(config.insecure);
    let client = Client::new(client_config);
    let auth = RegistryAuth::Anonymous;

    let reference: Reference = config
        .oci_ref(skill_name, tag)
        .parse()
        .context("parsing OCI pull reference")?;

    let (manifest, _) = client
        .pull_manifest(&reference, &auth)
        .await
        .context("pulling manifest")?;

    let OciManifest::Image(img) = manifest else {
        warn!(skill = %skill_name, "unexpected manifest type, skipping");
        return Ok(());
    };

    let skill_dir = dest_dir.join(skill_name);
    std::fs::create_dir_all(&skill_dir).context("creating skill directory")?;

    // Pull first layer (the SKILL.md content) and write it directly via AsyncWrite
    if let Some(layer) = img.layers.first() {
        debug!(digest = %layer.digest, "pulling layer");
        let skill_md_path = skill_dir.join("SKILL.md");
        let mut file = tokio::fs::File::create(&skill_md_path)
            .await
            .context("creating SKILL.md")?;
        client
            .pull_blob(&reference, layer, &mut file)
            .await
            .context("pulling layer blob")?;
        file.flush().await.context("flushing SKILL.md")?;
    }

    info!(skill = %skill_name, dest = %skill_dir.display(), "pulled skill");
    Ok(())
}

fn build_client_config(insecure: bool) -> ClientConfig {
    ClientConfig {
        protocol: if insecure {
            oci_client::client::ClientProtocol::Http
        } else {
            oci_client::client::ClientProtocol::Https
        },
        ..ClientConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_ref_format() {
        let cfg = RegistryConfig::local_zot();
        let r = cfg.oci_ref("my-skill", "latest");
        assert_eq!(r, "localhost:5000/skills/my-skill:latest");
    }

    #[test]
    fn test_local_zot_defaults() {
        let cfg = RegistryConfig::local_zot();
        assert!(cfg.insecure);
        assert_eq!(cfg.endpoint, "localhost:5000");
    }
}
