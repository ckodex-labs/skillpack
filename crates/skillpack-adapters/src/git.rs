//! Git Repository Adapter
//!
//! Implements SkillReader for Git repositories.

use skillpack_domain::{
    BundleReader, BundleReaderError, GitSource, SkillBundle, SkillIdentity, SkillReader,
    SkillReaderError,
};
use std::path::Path;

/// Git repository skill reader
pub struct GitReader {
    source: GitSource,
    cache_dir: std::path::PathBuf,
}

impl GitReader {
    pub fn new(source: GitSource) -> Self {
        let cache_dir = std::env::temp_dir().join("skillpack-git-cache");
        Self { source, cache_dir }
    }

    pub fn with_cache_dir(source: GitSource, cache_dir: std::path::PathBuf) -> Self {
        Self { source, cache_dir }
    }

    /// Clone or fetch repository to local cache
    pub fn fetch(&self) -> anyhow::Result<std::path::PathBuf> {
        // Refuse git remote-helper transports (`ext::`, `fd::`, …) that execute
        // arbitrary commands: `git clone 'ext::sh -c ...'` is remote code
        // execution. GIT_ALLOW_PROTOCOL below is the enforced backstop; this
        // guard gives a clear error before we ever spawn git.
        ensure_safe_git_url(&self.source.url)?;

        let repo_hash = sha256_short(&self.source.url);
        let local_path = self.cache_dir.join(&repo_hash).join(&self.source.r#ref);

        if local_path.exists() {
            // Pull latest
            let output = std::process::Command::new("git")
                .env("GIT_ALLOW_PROTOCOL", GIT_ALLOWED_PROTOCOLS)
                .args(["pull", "--rebase"])
                .current_dir(&local_path)
                .output()?;
            if !output.status.success() {
                tracing::warn!("Git pull failed, using existing cache");
            }
        } else {
            // Clone fresh
            std::fs::create_dir_all(&local_path)?;
            let output = std::process::Command::new("git")
                // Restrict transports to network + local file; `ext`/`fd` and
                // any other remote helper are not in the list, so git refuses
                // them instead of executing their command payload.
                .env("GIT_ALLOW_PROTOCOL", GIT_ALLOWED_PROTOCOLS)
                .args([
                    "clone",
                    "--depth",
                    "1",
                    "--branch",
                    &self.source.r#ref,
                    // `--` terminates option parsing so a url beginning with
                    // `-` cannot be treated as a git flag.
                    "--",
                    &self.source.url,
                    &local_path.to_string_lossy(),
                ])
                .output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        // Handle subpath if specified
        if let Some(subpath) = &self.source.path {
            Ok(local_path.join(subpath))
        } else {
            Ok(local_path)
        }
    }

    fn cached_path(&self) -> std::path::PathBuf {
        let repo_hash = sha256_short(&self.source.url);
        let base = self.cache_dir.join(&repo_hash).join(&self.source.r#ref);
        if let Some(subpath) = &self.source.path {
            base.join(subpath)
        } else {
            base
        }
    }
}

impl SkillReader for GitReader {
    fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
        let cached = self.cached_path();
        if let Ok(content) = std::fs::read_to_string(cached.join("SKILL.md")) {
            let name =
                extract_yaml_field(&content, "name").unwrap_or_else(|| "unknown".to_string());
            let version = extract_yaml_field(&content, "version")
                .unwrap_or_else(|| self.source.r#ref.clone());
            return Ok(SkillIdentity {
                name,
                version,
                path: format!("{}@{}", self.source.url, self.source.r#ref),
            });
        }

        Ok(SkillIdentity {
            name: self
                .source
                .url
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string(),
            version: self.source.r#ref.clone(),
            path: format!("{}@{}", self.source.url, self.source.r#ref),
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
            .filter(|e| !e.path().to_string_lossy().contains(".git"))
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

impl BundleReader for GitReader {
    fn read_bundle(&self, _path: &Path) -> Result<SkillBundle, BundleReaderError> {
        let bundles = self.list_bundles(Path::new(""));
        if bundles.is_empty() {
            return Err(BundleReaderError::NotFound(
                "No CNSB bundle in Git repo".to_string(),
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

/// Transports git is permitted to use (via `GIT_ALLOW_PROTOCOL`). Excludes the
/// command-executing remote helpers `ext` and `fd`. `file` is included so local
/// path repositories (`git:///path`) still clone.
const GIT_ALLOWED_PROTOCOLS: &str = "file:git:http:https:ssh";

/// Reject a clone URL whose transport can execute an arbitrary command. This is
/// a belt-and-suspenders check in front of `GIT_ALLOW_PROTOCOL`: it catches the
/// remote-helper syntax `<helper>::<payload>` for any non-network helper.
fn ensure_safe_git_url(url: &str) -> anyhow::Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty git URL");
    }
    // Remote-helper form is `transport::address`. A `::` that is not part of a
    // `scheme://` marks a helper transport (ext, fd, …) — refuse it.
    if let Some(idx) = trimmed.find("::") {
        let helper = &trimmed[..idx];
        let is_scheme_sep = trimmed.get(idx..idx + 3) == Some("://");
        if !is_scheme_sep && !helper.contains('/') {
            anyhow::bail!(
                "refusing git remote-helper transport '{}::' — only file/git/http/https/ssh URLs are allowed",
                helper
            );
        }
    }
    Ok(())
}

fn extract_yaml_field(content: &str, field: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.trim().starts_with(&format!("{}:", field)))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
}

fn sha256_short(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ext_remote_helper() {
        // The RCE vector: `git clone 'ext::sh -c ...'` runs the command.
        assert!(ensure_safe_git_url("ext::sh -c \"id\"").is_err());
        assert!(ensure_safe_git_url("fd::17/foo").is_err());
    }

    #[test]
    fn allows_network_and_local_urls() {
        assert!(ensure_safe_git_url("https://github.com/owner/repo").is_ok());
        assert!(ensure_safe_git_url("git://example.com/repo.git").is_ok());
        assert!(ensure_safe_git_url("ssh://git@host/repo.git").is_ok());
        assert!(ensure_safe_git_url("git@github.com:owner/repo.git").is_ok());
        // Local path repo (git:///tmp/x parses to this) must still clone.
        assert!(ensure_safe_git_url("/tmp/gitskill").is_ok());
    }

    #[test]
    fn rejects_empty_url() {
        assert!(ensure_safe_git_url("   ").is_err());
    }
}
