//! Unified Source Factory
//!
//! Creates appropriate reader based on source URI.

use crate::filesystem::FilesystemReader;
use crate::git::GitReader;
use crate::oci::OciReader;
use crate::s3::S3Reader;
use crate::sftp::SftpReader;
use skillpack_domain::{BundleReader, SkillReader, SkillSource};

/// Skills reader that boxes different source types
pub enum SkillsReader {
    Local(FilesystemReader),
    Oci(OciReader),
    Git(GitReader),
    S3(S3Reader),
    Sftp(SftpReader),
}

impl SkillsReader {
    /// Create reader from URI
    pub fn from_uri(uri: &str) -> anyhow::Result<Self> {
        let source = SkillSource::from_uri(uri)?;
        Self::from_source(source)
    }

    /// Create reader from parsed source
    pub fn from_source(source: SkillSource) -> anyhow::Result<Self> {
        Ok(match source {
            SkillSource::Local(_path) => Self::Local(FilesystemReader::new()),
            SkillSource::Oci(src) => Self::Oci(OciReader::new(src)),
            SkillSource::Git(src) => Self::Git(GitReader::new(src)),
            SkillSource::S3(src) => Self::S3(S3Reader::new(src)),
            SkillSource::Sftp(src) => Self::Sftp(SftpReader::new(src)),
            SkillSource::AzureBlob(_) => {
                return Err(anyhow::anyhow!("Azure Blob not yet implemented"));
            }
            SkillSource::Gcs(_) => {
                return Err(anyhow::anyhow!("GCS not yet implemented"));
            }
        })
    }

    /// Get as SkillReader trait object
    pub fn as_skill_reader(&self) -> &dyn SkillReader {
        match self {
            Self::Local(r) => r,
            Self::Oci(r) => r,
            Self::Git(r) => r,
            Self::S3(r) => r,
            Self::Sftp(r) => r,
        }
    }

    /// Get as BundleReader trait object
    pub fn as_bundle_reader(&self) -> &dyn BundleReader {
        match self {
            Self::Local(r) => r,
            Self::Oci(r) => r,
            Self::Git(r) => r,
            Self::S3(r) => r,
            Self::Sftp(r) => r,
        }
    }
}

/// Builder for remote skill sources with configuration
pub struct SourceBuilder {
    cache_dir: Option<std::path::PathBuf>,
}

impl SourceBuilder {
    pub fn new() -> Self {
        Self { cache_dir: None }
    }

    pub fn with_cache_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.cache_dir = Some(dir);
        self
    }

    pub fn build(&self, uri: &str) -> anyhow::Result<SkillsReader> {
        let source = SkillSource::from_uri(uri)?;
        let cache_dir = self
            .cache_dir
            .clone()
            .unwrap_or_else(|| std::env::temp_dir().join("skillpack-cache"));

        Ok(match source {
            SkillSource::Local(_) => SkillsReader::Local(FilesystemReader::new()),
            SkillSource::Oci(src) => SkillsReader::Oci(OciReader::with_cache_dir(src, cache_dir)),
            SkillSource::Git(src) => SkillsReader::Git(GitReader::with_cache_dir(src, cache_dir)),
            SkillSource::S3(src) => SkillsReader::S3(S3Reader::with_cache_dir(src, cache_dir)),
            SkillSource::Sftp(src) => {
                SkillsReader::Sftp(SftpReader::with_cache_dir(src, cache_dir))
            }
            _ => return Err(anyhow::anyhow!("Source type not yet implemented")),
        })
    }
}

impl Default for SourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
