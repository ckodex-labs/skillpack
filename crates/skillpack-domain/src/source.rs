//! Remote Skill Source Domain Model
//!
//! Unified abstraction for remote skill sources.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Remote skill source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillSource {
    /// Local filesystem
    Local(PathBuf),

    /// OCI registry (ghcr.io, docker.io, etc.)
    Oci(OciSource),

    /// Git repository
    Git(GitSource),

    /// S3-compatible storage
    S3(S3Source),

    /// SFTP/SSH
    Sftp(SftpSource),

    /// Azure Blob Storage
    AzureBlob(AzureBlobSource),

    /// Google Cloud Storage
    Gcs(GcsSource),
}

impl SkillSource {
    /// Parse from URI
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        if uri.starts_with("oci://") || uri.starts_with("ghcr.io/") || uri.starts_with("docker.io/")
        {
            Ok(Self::Oci(OciSource::from_uri(uri)?))
        } else if uri.starts_with("git://")
            || uri.starts_with("https://github.com/")
            || uri.starts_with("git@")
        {
            Ok(Self::Git(GitSource::from_uri(uri)?))
        } else if uri.starts_with("s3://") {
            Ok(Self::S3(S3Source::from_uri(uri)?))
        } else if uri.starts_with("sftp://") || uri.starts_with("ssh://") {
            Ok(Self::Sftp(SftpSource::from_uri(uri)?))
        } else if uri.starts_with("az://") || uri.starts_with("azure://") {
            Ok(Self::AzureBlob(AzureBlobSource::from_uri(uri)?))
        } else if uri.starts_with("gs://") {
            Ok(Self::Gcs(GcsSource::from_uri(uri)?))
        } else {
            Ok(Self::Local(PathBuf::from(uri)))
        }
    }
}

/// OCI registry source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciSource {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

impl OciSource {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri.strip_prefix("oci://").unwrap_or(uri);
        let parts: Vec<&str> = uri.split('/').collect();
        if parts.len() < 2 {
            return Err(SourceParseError::InvalidUri(uri.to_string()));
        }

        let registry = parts[0].to_string();
        let repo_tag = parts[1..].join("/");
        let (repository, tag) = if let Some((r, t)) = repo_tag.rsplit_once(':') {
            (r.to_string(), t.to_string())
        } else {
            (repo_tag, "latest".to_string())
        };

        Ok(Self {
            registry,
            repository,
            tag,
            digest: None,
        })
    }

    pub fn reference(&self) -> String {
        format!("{}/{}:{}", self.registry, self.repository, self.tag)
    }
}

/// Git repository source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    pub url: String,
    pub r#ref: String,        // branch, tag, or commit
    pub path: Option<String>, // subdirectory within repo
}

impl GitSource {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri.strip_prefix("git://").unwrap_or(uri);
        let (url, r#ref) = if let Some((u, r)) = uri.rsplit_once('@') {
            (u.to_string(), r.to_string())
        } else {
            (uri.to_string(), "main".to_string())
        };

        Ok(Self {
            url,
            r#ref,
            path: None,
        })
    }
}

/// S3-compatible storage source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Source {
    pub bucket: String,
    pub key: String,
    pub region: Option<String>,
    pub endpoint: Option<String>, // For MinIO, SeaweedFS, etc.
}

impl S3Source {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri.strip_prefix("s3://").unwrap_or(uri);
        let (bucket, key) = uri.split_once('/').unwrap_or((uri, ""));
        Ok(Self {
            bucket: bucket.to_string(),
            key: key.to_string(),
            region: None,
            endpoint: None,
        })
    }
}

/// SFTP/SSH source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpSource {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub path: String,
}

impl SftpSource {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri
            .strip_prefix("sftp://")
            .or_else(|| uri.strip_prefix("ssh://"))
            .unwrap_or(uri);
        let (host_port, path) = uri.split_once('/').unwrap_or((uri, "/"));
        let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
            (h.to_string(), p.parse().unwrap_or(22))
        } else {
            (host_port.to_string(), 22)
        };

        Ok(Self {
            host,
            port,
            user: None,
            path: format!("/{}", path),
        })
    }
}

/// Azure Blob Storage source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureBlobSource {
    pub account: String,
    pub container: String,
    pub blob: String,
}

impl AzureBlobSource {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri
            .strip_prefix("az://")
            .or_else(|| uri.strip_prefix("azure://"))
            .unwrap_or(uri);
        let parts: Vec<&str> = uri.splitn(3, '/').collect();
        if parts.len() < 3 {
            return Err(SourceParseError::InvalidUri(uri.to_string()));
        }
        Ok(Self {
            account: parts[0].to_string(),
            container: parts[1].to_string(),
            blob: parts[2].to_string(),
        })
    }
}

/// Google Cloud Storage source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsSource {
    pub bucket: String,
    pub object: String,
}

impl GcsSource {
    pub fn from_uri(uri: &str) -> Result<Self, SourceParseError> {
        let uri = uri.strip_prefix("gs://").unwrap_or(uri);
        let (bucket, object) = uri.split_once('/').unwrap_or((uri, ""));
        Ok(Self {
            bucket: bucket.to_string(),
            object: object.to_string(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SourceParseError {
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Unsupported source type")]
    UnsupportedType,
}
