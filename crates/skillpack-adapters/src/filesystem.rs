//! Filesystem Adapter
//!
//! Implements SkillReader and BundleReader ports for local filesystem.

use skillpack_domain::{
    BundleReader, BundleReaderError, SkillBundle, SkillIdentity, SkillReader, SkillReaderError,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct FilesystemReader {
    exists_cache: Arc<Mutex<HashMap<String, bool>>>,
    bundles_cache: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Default for FilesystemReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemReader {
    pub fn new() -> Self {
        Self {
            exists_cache: Arc::new(Mutex::new(HashMap::new())),
            bundles_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Clear all caches (useful before a fresh assessment)
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.exists_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.bundles_cache.lock() {
            cache.clear();
        }
    }
}

impl SkillReader for FilesystemReader {
    fn read_identity(&self, path: &Path) -> Result<SkillIdentity, SkillReaderError> {
        // First try CNSB bundle
        let cnsb_files = self.list_bundles(path);
        if !cnsb_files.is_empty() {
            if let Ok(bundle) = self.read_bundle(&path.join(&cnsb_files[0])) {
                return Ok(SkillIdentity {
                    name: bundle.metadata.name,
                    version: bundle.metadata.version,
                    path: path.to_string_lossy().to_string(),
                });
            }
        }

        // Fallback to SKILL.md
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            return Err(SkillReaderError::NotFound(skill_md.display().to_string()));
        }

        let content = std::fs::read_to_string(&skill_md)
            .map_err(|e| SkillReaderError::ReadError(e.to_string()))?;

        let name = extract_yaml_field(&content, "name").unwrap_or_else(|| "unknown".to_string());
        let version =
            extract_yaml_field(&content, "version").unwrap_or_else(|| "0.0.0".to_string());

        Ok(SkillIdentity {
            name,
            version,
            path: path.to_string_lossy().to_string(),
        })
    }

    fn file_exists(&self, path: &Path, relative: &str) -> bool {
        let key = format!("{}|{}", path.display(), relative);
        if let Ok(cache) = self.exists_cache.lock() {
            if let Some(&result) = cache.get(&key) {
                return result;
            }
        }
        let result = path.join(relative).exists();
        if let Ok(mut cache) = self.exists_cache.lock() {
            cache.insert(key, result);
        }
        result
    }

    fn read_file(&self, path: &Path, relative: &str) -> Result<String, SkillReaderError> {
        std::fs::read_to_string(path.join(relative))
            .map_err(|e| SkillReaderError::ReadError(e.to_string()))
    }

    fn list_files(&self, path: &Path, pattern: &str) -> Vec<String> {
        let regex = regex::Regex::new(pattern).ok();
        WalkDir::new(path)
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
                    .strip_prefix(path)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }
}

impl BundleReader for FilesystemReader {
    fn read_bundle(&self, path: &Path) -> Result<SkillBundle, BundleReaderError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BundleReaderError::ReadError(e.to_string()))?;

        SkillBundle::from_json(&content)
            .map_err(|e| BundleReaderError::InvalidFormat(e.to_string()))
    }

    fn list_bundles(&self, path: &Path) -> Vec<String> {
        let key = path.display().to_string();
        if let Ok(cache) = self.bundles_cache.lock() {
            if let Some(result) = cache.get(&key) {
                return result.clone();
            }
        }
        let result: Vec<String> = WalkDir::new(path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().to_string_lossy().ends_with(".cnsb.json"))
            .map(|e| {
                e.path()
                    .strip_prefix(path)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        if let Ok(mut cache) = self.bundles_cache.lock() {
            cache.insert(key, result.clone());
        }
        result
    }
}

fn extract_yaml_field(content: &str, field: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.trim().starts_with(&format!("{}:", field)))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
}
