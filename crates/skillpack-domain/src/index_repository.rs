//! Index Repository Port
//!
//! Defines storage contract for Skills Indices.

use crate::index::{IndexHistory, IndexId, IndexValue, SkillsIndex};
use std::sync::Arc;
use thiserror::Error;

/// Index storage errors
#[derive(Debug, Error)]
pub enum IndexRepositoryError {
    #[error("Index not found: {0}")]
    NotFound(String),
    #[error("Index already exists: {0}")]
    AlreadyExists(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Port for index persistence
pub trait IndexRepository: Send + Sync {
    /// Create a new index
    fn create(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError>;

    /// Get index by ID
    fn get(&self, id: &IndexId) -> Result<SkillsIndex, IndexRepositoryError>;

    /// List all indices
    fn list(&self) -> Result<Vec<SkillsIndex>, IndexRepositoryError>;

    /// Update existing index
    fn update(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError>;

    /// Delete index
    fn delete(&self, id: &IndexId) -> Result<(), IndexRepositoryError>;

    /// Store index value snapshot
    fn store_value(&self, value: &IndexValue) -> Result<(), IndexRepositoryError>;

    /// Get index history
    fn get_history(&self, id: &IndexId) -> Result<IndexHistory, IndexRepositoryError>;
}

/// Blanket impl for Arc-wrapped repositories
impl<T: IndexRepository> IndexRepository for Arc<T> {
    fn create(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        (**self).create(index)
    }

    fn get(&self, id: &IndexId) -> Result<SkillsIndex, IndexRepositoryError> {
        (**self).get(id)
    }

    fn list(&self) -> Result<Vec<SkillsIndex>, IndexRepositoryError> {
        (**self).list()
    }

    fn update(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        (**self).update(index)
    }

    fn delete(&self, id: &IndexId) -> Result<(), IndexRepositoryError> {
        (**self).delete(id)
    }

    fn store_value(&self, value: &IndexValue) -> Result<(), IndexRepositoryError> {
        (**self).store_value(value)
    }

    fn get_history(&self, id: &IndexId) -> Result<IndexHistory, IndexRepositoryError> {
        (**self).get_history(id)
    }
}
