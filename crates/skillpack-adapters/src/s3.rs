//! S3 Storage Adapter
//!
//! Implements SkillReader for S3-compatible storage (AWS, MinIO, SeaweedFS).

use skillpack_domain::{
    BundleReader, BundleReaderError, S3Source, SkillBundle, SkillIdentity, SkillReader,
    SkillReaderError,
};
use std::path::Path;

/// S3 skill reader
pub struct S3Reader {
    source: S3Source,
    cache_dir: std::path::PathBuf,
}

impl S3Reader {
    pub fn new(source: S3Source) -> Self {
        let cache_dir = std::env::temp_dir().join("skillpack-s3-cache");
        Self { source, cache_dir }
    }

    pub fn with_cache_dir(source: S3Source, cache_dir: std::path::PathBuf) -> Self {
        Self { source, cache_dir }
    }

    /// Download skill from S3 to local cache
    pub async fn download(&self) -> anyhow::Result<std::path::PathBuf> {
        // In production, use aws-sdk-s3 or object_store crate
        // For now, use aws CLI subprocess
        let local_path = self
            .cache_dir
            .join(&self.source.bucket)
            .join(&self.source.key);
        std::fs::create_dir_all(&local_path)?;

        let s3_uri = format!("s3://{}/{}", self.source.bucket, self.source.key);

        let mut cmd = std::process::Command::new("aws");
        cmd.args(["s3", "sync", &s3_uri, &local_path.to_string_lossy()]);

        if let Some(endpoint) = &self.source.endpoint {
            cmd.args(["--endpoint-url", endpoint]);
        }

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "S3 sync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(local_path)
    }

    fn cached_path(&self) -> std::path::PathBuf {
        self.cache_dir
            .join(&self.source.bucket)
            .join(&self.source.key)
    }
}

impl SkillReader for S3Reader {
    fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
        // Try to read from cached SKILL.md
        let cached = self.cached_path();
        if let Ok(content) = std::fs::read_to_string(cached.join("SKILL.md")) {
            let name =
                extract_yaml_field(&content, "name").unwrap_or_else(|| self.source.key.clone());
            let version =
                extract_yaml_field(&content, "version").unwrap_or_else(|| "0.0.0".to_string());
            return Ok(SkillIdentity {
                name,
                version,
                path: format!("s3://{}/{}", self.source.bucket, self.source.key),
            });
        }

        Ok(SkillIdentity {
            name: self.source.key.clone(),
            version: "0.0.0".to_string(),
            path: format!("s3://{}/{}", self.source.bucket, self.source.key),
        })
    }

    fn file_exists(&self, _path: &Path, relative: &str) -> bool {
        self.cached_path().join(relative).exists()
    }

    fn read_file(&self, _path: &Path, relative: &str) -> Result<String, SkillReaderError> {
        std::fs::read_to_string(self.cached_path().join(relative))
            .map_err(|e| SkillReaderError::ReadError(e.to_string()))
    }

    fn list_files(&self, _path: &Path, pattern: &str) -> Vec<String> {
        let cached = self.cached_path();
        let regex = regex::Regex::new(pattern).ok();
        walkdir::WalkDir::new(&cached)
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
                    .strip_prefix(&cached)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }
}

impl BundleReader for S3Reader {
    fn read_bundle(&self, _path: &Path) -> Result<SkillBundle, BundleReaderError> {
        let bundles = self.list_bundles(Path::new(""));
        if bundles.is_empty() {
            return Err(BundleReaderError::NotFound(
                "No CNSB bundle in S3 path".to_string(),
            ));
        }
        let content = std::fs::read_to_string(self.cached_path().join(&bundles[0]))
            .map_err(|e| BundleReaderError::ReadError(e.to_string()))?;
        SkillBundle::from_json(&content)
            .map_err(|e| BundleReaderError::InvalidFormat(e.to_string()))
    }

    fn list_bundles(&self, _path: &Path) -> Vec<String> {
        self.list_files(Path::new(""), r"\.cnsb\.json$")
    }
}

fn extract_yaml_field(content: &str, field: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.trim().starts_with(&format!("{}:", field)))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
}
