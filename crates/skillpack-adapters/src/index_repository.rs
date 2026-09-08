//! JSON File-based Index Repository
//!
//! Simple file-based persistence for skills indices.

use skillpack_domain::{
    IndexHistory, IndexId, IndexRepository, IndexRepositoryError, IndexValue, SkillsIndex,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

/// JSON file-based index repository
pub struct JsonIndexRepository {
    base_path: PathBuf,
    cache: RwLock<HashMap<String, SkillsIndex>>,
}

impl JsonIndexRepository {
    pub fn new(base_path: PathBuf) -> Self {
        fs::create_dir_all(&base_path).ok();
        Self {
            base_path,
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn index_path(&self, id: &IndexId) -> PathBuf {
        self.base_path.join(format!("{}.json", id.as_str()))
    }

    fn history_path(&self, id: &IndexId) -> PathBuf {
        self.base_path.join(format!("{}_history.json", id.as_str()))
    }
}

impl IndexRepository for JsonIndexRepository {
    fn create(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        let path = self.index_path(&index.id);
        if path.exists() {
            return Err(IndexRepositoryError::AlreadyExists(
                index.id.as_str().to_string(),
            ));
        }

        let json = serde_json::to_string_pretty(index)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        fs::write(&path, json).map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(index.id.as_str().to_string(), index.clone());
        }

        Ok(())
    }

    fn get(&self, id: &IndexId) -> Result<SkillsIndex, IndexRepositoryError> {
        // Check cache first
        if let Ok(cache) = self.cache.read()
            && let Some(index) = cache.get(id.as_str())
        {
            return Ok(index.clone());
        }

        let path = self.index_path(id);
        if !path.exists() {
            return Err(IndexRepositoryError::NotFound(id.as_str().to_string()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let index: SkillsIndex = serde_json::from_str(&content)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(id.as_str().to_string(), index.clone());
        }

        Ok(index)
    }

    fn list(&self) -> Result<Vec<SkillsIndex>, IndexRepositoryError> {
        let mut indices = Vec::new();

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && !path.to_string_lossy().contains("_history")
                && let Ok(content) = fs::read_to_string(&path)
                && let Ok(index) = serde_json::from_str::<SkillsIndex>(&content)
            {
                indices.push(index);
            }
        }

        Ok(indices)
    }

    fn update(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        let path = self.index_path(&index.id);

        let json = serde_json::to_string_pretty(index)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        fs::write(&path, json).map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(index.id.as_str().to_string(), index.clone());
        }

        Ok(())
    }

    fn delete(&self, id: &IndexId) -> Result<(), IndexRepositoryError> {
        let path = self.index_path(id);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        }

        // Remove from cache
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(id.as_str());
        }

        // Also remove history
        let history_path = self.history_path(id);
        if history_path.exists() {
            fs::remove_file(&history_path).ok();
        }

        Ok(())
    }

    fn store_value(&self, value: &IndexValue) -> Result<(), IndexRepositoryError> {
        let path = self.history_path(&value.index_id);

        let mut history = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
            serde_json::from_str::<IndexHistory>(&content)
                .unwrap_or_else(|_| IndexHistory::new(value.index_id.clone()))
        } else {
            IndexHistory::new(value.index_id.clone())
        };

        history.add(value.clone());

        let json = serde_json::to_string_pretty(&history)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        fs::write(&path, json).map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn get_history(&self, id: &IndexId) -> Result<IndexHistory, IndexRepositoryError> {
        let path = self.history_path(id);

        if !path.exists() {
            return Ok(IndexHistory::new(id.clone()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let history: IndexHistory = serde_json::from_str(&content)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::index::predefined;

    #[test]
    fn test_create_and_get_index() {
        let temp_dir = std::env::temp_dir().join("skillpack-test-indices");
        let repo = JsonIndexRepository::new(temp_dir.clone());

        let index = predefined::skillpack_100();
        repo.create(&index).unwrap();

        let retrieved = repo.get(&index.id).unwrap();
        assert_eq!(retrieved.name, "SkillPack-100");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
