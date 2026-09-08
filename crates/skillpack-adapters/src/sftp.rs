//! SFTP/SSH Adapter
//!
//! Implements SkillReader for SFTP/SSH remote sources.

use skillpack_domain::{
    BundleReader, BundleReaderError, SftpSource, SkillBundle, SkillIdentity, SkillReader,
    SkillReaderError,
};
use std::path::Path;

/// SFTP skill reader
pub struct SftpReader {
    source: SftpSource,
    cache_dir: std::path::PathBuf,
}

impl SftpReader {
    pub fn new(source: SftpSource) -> Self {
        let cache_dir = std::env::temp_dir().join("skillpack-sftp-cache");
        Self { source, cache_dir }
    }

    pub fn with_cache_dir(source: SftpSource, cache_dir: std::path::PathBuf) -> Self {
        Self { source, cache_dir }
    }

    /// Download from SFTP to local cache using rsync over SSH
    pub fn download(&self) -> anyhow::Result<std::path::PathBuf> {
        let local_path = self
            .cache_dir
            .join(&self.source.host)
            .join(self.source.path.trim_start_matches('/'));
        std::fs::create_dir_all(&local_path)?;

        let remote = format!(
            "{}{}:{}",
            self.source
                .user
                .as_ref()
                .map(|u| format!("{}@", u))
                .unwrap_or_default(),
            self.source.host,
            self.source.path
        );

        let mut cmd = std::process::Command::new("rsync");
        cmd.args([
            "-avz",
            "-e",
            &format!("ssh -p {}", self.source.port),
            &format!("{}/", remote),
            local_path.to_string_lossy().as_ref(),
        ]);

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "rsync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(local_path)
    }

    fn cached_path(&self) -> std::path::PathBuf {
        self.cache_dir
            .join(&self.source.host)
            .join(self.source.path.trim_start_matches('/'))
    }
}

impl SkillReader for SftpReader {
    fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
        let cached = self.cached_path();
        if let Ok(content) = std::fs::read_to_string(cached.join("SKILL.md")) {
            let name =
                extract_yaml_field(&content, "name").unwrap_or_else(|| "unknown".to_string());
            let version =
                extract_yaml_field(&content, "version").unwrap_or_else(|| "0.0.0".to_string());
            return Ok(SkillIdentity {
                name,
                version,
                path: format!(
                    "sftp://{}:{}{}",
                    self.source.host, self.source.port, self.source.path
                ),
            });
        }

        Ok(SkillIdentity {
            name: self
                .source
                .path
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string(),
            version: "0.0.0".to_string(),
            path: format!(
                "sftp://{}:{}{}",
                self.source.host, self.source.port, self.source.path
            ),
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

impl BundleReader for SftpReader {
    fn read_bundle(&self, _path: &Path) -> Result<SkillBundle, BundleReaderError> {
        let bundles = self.list_bundles(Path::new(""));
        if bundles.is_empty() {
            return Err(BundleReaderError::NotFound(
                "No CNSB bundle in SFTP path".to_string(),
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
