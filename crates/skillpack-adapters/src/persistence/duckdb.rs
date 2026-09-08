//! DuckDB Persistence Adapter
//!
//! SQL-based analytics storage for complex queries and aggregations.

use anyhow::Result;
use duckdb::{Connection, params};
use skillpack_domain::{
    IndexHistory, IndexId, IndexRepository, IndexRepositoryError, IndexValue, RatingAssessment,
    SkillsIndex,
};
use std::path::Path;
use std::sync::Mutex;

/// DuckDB-backed repository for indices and ratings
///
/// Note: DuckDB Connection is not Sync, so we use Mutex for thread safety.
/// For high-concurrency scenarios, consider a connection pool.
pub struct DuckDbRepository {
    connection: Mutex<Connection>,
}

// Implement Send + Sync manually since we protect access with Mutex
unsafe impl Send for DuckDbRepository {}
unsafe impl Sync for DuckDbRepository {}

impl DuckDbRepository {
    /// Create new in-memory DuckDB repository
    pub fn in_memory() -> Result<Self> {
        let connection = Connection::open_in_memory()?;
        let repo = Self {
            connection: Mutex::new(connection),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Create new file-backed DuckDB repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Connection::open(path)?;
        let repo = Self {
            connection: Mutex::new(connection),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Safely lock the DuckDB connection, mapping a poisoned mutex to an error.
    fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|e| anyhow::anyhow!("DuckDB mutex poisoned: {}", e))
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        // Skills Index table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills_indices (
                id VARCHAR,
                tenant_id VARCHAR DEFAULT 'default',
                name VARCHAR NOT NULL,
                description TEXT,
                methodology VARCHAR,
                data JSON NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (id, tenant_id)
            )",
            [],
        )?;

        // Index values (time series)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS index_values (
                id INTEGER PRIMARY KEY,
                index_id VARCHAR NOT NULL,
                tenant_id VARCHAR DEFAULT 'default',
                value DOUBLE NOT NULL,
                grade VARCHAR,
                constituent_count INTEGER,
                data JSON NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Ratings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skill_ratings (
                skill_ref VARCHAR,
                tenant_id VARCHAR DEFAULT 'default',
                rating VARCHAR NOT NULL,
                category VARCHAR,
                is_investment_grade BOOLEAN,
                outlook VARCHAR,
                quality_score DOUBLE,
                security_score DOUBLE,
                governance_score DOUBLE,
                track_record_score DOUBLE,
                maintainer_score DOUBLE,
                composite_score DOUBLE,
                data JSON NOT NULL,
                assessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (skill_ref, tenant_id)
            )",
            [],
        )?;

        // Rating history
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rating_history (
                id INTEGER PRIMARY KEY,
                skill_ref VARCHAR NOT NULL,
                from_rating VARCHAR,
                to_rating VARCHAR NOT NULL,
                action VARCHAR NOT NULL,
                reason TEXT,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Skills compatibility (CanIUse)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skill_compatibility (
                skill_ref VARCHAR NOT NULL,
                capability_id VARCHAR NOT NULL,
                support_level VARCHAR NOT NULL,
                notes TEXT,
                verified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (skill_ref, capability_id)
            )",
            [],
        )?;

        // Cost estimates
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cost_estimates (
                skill_ref VARCHAR PRIMARY KEY,
                monthly_cost_usd DOUBLE,
                cost_tier VARCHAR,
                co2_grams DOUBLE,
                carbon_rating VARCHAR,
                input_tokens BIGINT,
                output_tokens BIGINT,
                data JSON NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Canonical skills registry
        conn.execute(
            "CREATE TABLE IF NOT EXISTS canonical_skills (
                skill_ref VARCHAR,
                tenant_id VARCHAR DEFAULT 'default',
                name VARCHAR NOT NULL,
                path VARCHAR NOT NULL,
                has_skill_md BOOLEAN DEFAULT false,
                content_hash VARCHAR,
                manifest_json TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (skill_ref, tenant_id)
            )",
            [],
        )?;

        // Agent sync state
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_sync_state (
                agent_name VARCHAR NOT NULL,
                skill_ref VARCHAR NOT NULL,
                sync_status VARCHAR NOT NULL DEFAULT 'pending',
                last_sync_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                checksum VARCHAR,
                action VARCHAR NOT NULL DEFAULT 'add',
                PRIMARY KEY (agent_name, skill_ref)
            )",
            [],
        )?;

        // Sync history log
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_history (
                id INTEGER PRIMARY KEY,
                sync_id VARCHAR NOT NULL,
                started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                completed_at TIMESTAMP,
                skills_processed INTEGER DEFAULT 0,
                agents_count INTEGER DEFAULT 0,
                success BOOLEAN DEFAULT true,
                message TEXT
            )",
            [],
        )?;

        // Create indices for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_index_values_ts ON index_values(index_id, timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_canonical_skills_hash ON canonical_skills(content_hash)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_sync_state ON agent_sync_state(agent_name, sync_status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_history_id ON sync_history(sync_id, started_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ratings_rating ON skill_ratings(rating)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ratings_grade ON skill_ratings(is_investment_grade)",
            [],
        )?;

        Ok(())
    }

    // ========================================
    // Ratings Methods
    // ========================================

    /// Store a rating assessment
    pub fn store_rating(&self, assessment: &RatingAssessment) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let json = serde_json::to_string(assessment)?;

        conn.execute(
            "INSERT OR REPLACE INTO skill_ratings 
            (skill_ref, rating, category, is_investment_grade, outlook,
             quality_score, security_score, governance_score, 
             track_record_score, maintainer_score, composite_score,
             data, assessed_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            params![
                assessment.skill_ref,
                assessment.rating.as_str(),
                format!("{:?}", assessment.rating.category()),
                assessment.rating.is_investment_grade(),
                assessment.outlook.as_str(),
                assessment.factors.quality_score,
                assessment.factors.security_score,
                assessment.factors.governance_score,
                assessment.factors.track_record_score,
                assessment.factors.maintainer_score,
                assessment.factors.composite(),
                json,
            ],
        )?;

        Ok(())
    }

    /// Get rating for a skill
    pub fn get_rating(&self, skill_ref: &str) -> Result<Option<RatingAssessment>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT data FROM skill_ratings WHERE skill_ref = ?")?;

        let result = stmt.query_row([skill_ref], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let assessment: RatingAssessment = serde_json::from_str(&json)?;
                Ok(Some(assessment))
            }
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all investment-grade skills
    pub fn list_investment_grade(&self) -> Result<Vec<RatingAssessment>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT data FROM skill_ratings WHERE is_investment_grade = true ORDER BY composite_score DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut ratings = Vec::new();
        for json in rows.flatten() {
            if let Ok(assessment) = serde_json::from_str::<RatingAssessment>(&json) {
                ratings.push(assessment);
            }
        }

        Ok(ratings)
    }

    // ========================================
    // Analytics Methods
    // ========================================

    /// Get index performance statistics
    pub fn get_index_stats(&self, index_id: &str) -> Result<IndexStats> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as count,
                AVG(value) as avg_value,
                MIN(value) as min_value,
                MAX(value) as max_value,
                STDDEV(value) as std_dev
            FROM index_values 
            WHERE index_id = ?",
        )?;

        let stats = stmt.query_row([index_id], |row| {
            Ok(IndexStats {
                count: row.get(0)?,
                avg_value: row.get::<_, Option<f64>>(1)?.unwrap_or(0.0),
                min_value: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                max_value: row.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                std_dev: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            })
        })?;

        Ok(stats)
    }

    /// Get rating distribution
    pub fn get_rating_distribution(&self) -> Result<Vec<(String, i64)>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT rating, COUNT(*) as count 
            FROM skill_ratings 
            GROUP BY rating 
            ORDER BY rating",
        )?;

        let rows = stmt.query_map([], |row| {
            let rating: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((rating, count))
        })?;

        let mut distribution = Vec::new();
        for entry in rows.flatten() {
            distribution.push(entry);
        }

        Ok(distribution)
    }
}

/// Index performance statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub count: i64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub std_dev: f64,
}

// ========================================
// IndexRepository Implementation
// ========================================

impl IndexRepository for DuckDbRepository {
    fn create(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let json = serde_json::to_string(index)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        conn.execute(
            "INSERT INTO skills_indices (id, name, description, methodology, data) 
             VALUES (?, ?, ?, ?, ?)",
            params![
                index.id.as_str(),
                index.name,
                index.description,
                format!("{:?}", index.methodology.weighting),
                json,
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                IndexRepositoryError::AlreadyExists(index.id.as_str().to_string())
            } else {
                IndexRepositoryError::StorageError(e.to_string())
            }
        })?;

        Ok(())
    }

    fn get(&self, id: &IndexId) -> Result<SkillsIndex, IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT data FROM skills_indices WHERE id = ?")
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let result = stmt.query_row([id.as_str()], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let index: SkillsIndex = serde_json::from_str(&json)
                    .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
                Ok(index)
            }
            Err(duckdb::Error::QueryReturnedNoRows) => {
                Err(IndexRepositoryError::NotFound(id.as_str().to_string()))
            }
            Err(e) => Err(IndexRepositoryError::StorageError(e.to_string())),
        }
    }

    fn list(&self) -> Result<Vec<SkillsIndex>, IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT data FROM skills_indices ORDER BY name")
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let json: String = row.get(0)?;
                Ok(json)
            })
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut indices = Vec::new();
        for json in rows.flatten() {
            if let Ok(index) = serde_json::from_str::<SkillsIndex>(&json) {
                indices.push(index);
            }
        }

        Ok(indices)
    }

    fn update(&self, index: &SkillsIndex) -> Result<(), IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let json = serde_json::to_string(index)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        conn.execute(
            "UPDATE skills_indices SET name = ?, description = ?, data = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![index.name, index.description, json, index.id.as_str()],
        ).map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &IndexId) -> Result<(), IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        conn.execute("DELETE FROM skills_indices WHERE id = ?", [id.as_str()])
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn store_value(&self, value: &IndexValue) -> Result<(), IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let json = serde_json::to_string(value)
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        conn.execute(
            "INSERT INTO index_values (index_id, value, grade, constituent_count, data) 
             VALUES (?, ?, ?, ?, ?)",
            params![
                value.index_id.as_str(),
                value.value,
                value.grade(),
                value.constituent_count as i32,
                json,
            ],
        )
        .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn get_history(&self, id: &IndexId) -> Result<IndexHistory, IndexRepositoryError> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT data FROM index_values WHERE index_id = ? ORDER BY timestamp")
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map([id.as_str()], |row| {
                let json: String = row.get(0)?;
                Ok(json)
            })
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;

        let mut history = IndexHistory::new(id.clone());
        for json in rows.flatten() {
            if let Ok(value) = serde_json::from_str::<IndexValue>(&json) {
                history.add(value);
            }
        }

        Ok(history)
    }
}

/// Parameter bundle for `upsert_canonical_skill` (keeps the public arg count at 1).
pub struct CanonicalSkillUpsert {
    pub skill_ref: String,
    pub tenant_id: String,
    pub name: String,
    pub path: String,
    pub has_skill_md: bool,
    pub content_hash: Option<String>,
    pub manifest_json: Option<String>,
}

// ========================================
// Canonical Store Persistence
// ========================================

impl DuckDbRepository {
    /// Upsert a canonical skill record
    pub fn upsert_canonical_skill(&self, upsert: CanonicalSkillUpsert) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let CanonicalSkillUpsert {
            skill_ref,
            tenant_id,
            name,
            path,
            has_skill_md,
            content_hash,
            manifest_json,
        } = upsert;
        conn.execute(
            "INSERT OR REPLACE INTO canonical_skills 
             (skill_ref, tenant_id, name, path, has_skill_md, content_hash, manifest_json, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            params![
                skill_ref,
                tenant_id,
                name,
                path,
                has_skill_md,
                content_hash.as_deref().unwrap_or(""),
                manifest_json.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(())
    }

    /// Get a canonical skill by reference
    pub fn get_canonical_skill(
        &self,
        skill_ref: &str,
        tenant_id: &str,
    ) -> Result<Option<CanonicalSkillRecord>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT skill_ref, name, path, has_skill_md, content_hash, manifest_json, 
             created_at::TEXT, updated_at::TEXT 
             FROM canonical_skills WHERE skill_ref = ? AND tenant_id = ?",
        )?;
        let row = stmt.query_row([skill_ref, tenant_id], |row| {
            Ok(CanonicalSkillRecord {
                skill_ref: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                has_skill_md: row.get(3)?,
                content_hash: row.get(4)?,
                manifest_json: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });
        match row {
            Ok(record) => Ok(Some(record)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all canonical skills for a tenant
    pub fn list_canonical_skills(&self, tenant_id: &str) -> Result<Vec<CanonicalSkillRecord>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT skill_ref, name, path, has_skill_md, content_hash, manifest_json, 
             created_at::TEXT, updated_at::TEXT 
             FROM canonical_skills WHERE tenant_id = ? ORDER BY name",
        )?;
        let rows = stmt.query_map([tenant_id], |row| {
            Ok(CanonicalSkillRecord {
                skill_ref: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                has_skill_md: row.get(3)?,
                content_hash: row.get(4)?,
                manifest_json: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    /// Delete a canonical skill
    pub fn delete_canonical_skill(&self, skill_ref: &str, tenant_id: &str) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        conn.execute(
            "DELETE FROM canonical_skills WHERE skill_ref = ? AND tenant_id = ?",
            [skill_ref, tenant_id],
        )?;
        Ok(())
    }

    /// Upsert agent sync state
    pub fn upsert_sync_state(
        &self,
        agent_name: &str,
        skill_ref: &str,
        sync_status: &str,
        checksum: Option<&str>,
        action: &str,
    ) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO agent_sync_state 
             (agent_name, skill_ref, sync_status, last_sync_at, checksum, action)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP, ?, ?)",
            params![
                agent_name,
                skill_ref,
                sync_status,
                checksum.unwrap_or(""),
                action,
            ],
        )?;
        Ok(())
    }

    /// Get sync state for an agent + skill
    pub fn get_sync_state(
        &self,
        agent_name: &str,
        skill_ref: &str,
    ) -> Result<Option<AgentSyncState>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT agent_name, skill_ref, sync_status, last_sync_at, checksum, action 
             FROM agent_sync_state WHERE agent_name = ? AND skill_ref = ?",
        )?;
        let row = stmt.query_row([agent_name, skill_ref], |row| {
            Ok(AgentSyncState {
                agent_name: row.get(0)?,
                skill_ref: row.get(1)?,
                sync_status: row.get(2)?,
                last_sync_at: row.get(3)?,
                checksum: row.get(4)?,
                action: row.get(5)?,
            })
        });
        match row {
            Ok(state) => Ok(Some(state)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all sync states for an agent
    pub fn list_sync_states_for_agent(&self, agent_name: &str) -> Result<Vec<AgentSyncState>> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT agent_name, skill_ref, sync_status, last_sync_at, checksum, action 
             FROM agent_sync_state WHERE agent_name = ? ORDER BY skill_ref",
        )?;
        let rows = stmt.query_map([agent_name], |row| {
            Ok(AgentSyncState {
                agent_name: row.get(0)?,
                skill_ref: row.get(1)?,
                sync_status: row.get(2)?,
                last_sync_at: row.get(3)?,
                checksum: row.get(4)?,
                action: row.get(5)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    /// Record a sync run in history
    pub fn record_sync_history(
        &self,
        sync_id: &str,
        skills_processed: i32,
        agents_count: i32,
        success: bool,
        message: Option<&str>,
    ) -> Result<()> {
        let conn = self
            .conn()
            .map_err(|e| IndexRepositoryError::StorageError(e.to_string()))?;
        conn.execute(
            "INSERT INTO sync_history 
             (sync_id, started_at, completed_at, skills_processed, agents_count, success, message)
             VALUES (?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ?, ?, ?, ?)",
            params![
                sync_id,
                skills_processed,
                agents_count,
                success,
                message.unwrap_or(""),
            ],
        )?;
        Ok(())
    }
}

/// Record of a canonical skill in the registry
#[derive(Debug, Clone)]
pub struct CanonicalSkillRecord {
    pub skill_ref: String,
    pub name: String,
    pub path: String,
    pub has_skill_md: bool,
    pub content_hash: String,
    pub manifest_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Sync state for a single agent+skill pair
#[derive(Debug, Clone)]
pub struct AgentSyncState {
    pub agent_name: String,
    pub skill_ref: String,
    pub sync_status: String,
    pub last_sync_at: String,
    pub checksum: String,
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::{CanonicalSkillUpsert, DuckDbRepository};

    #[test]
    fn tenant_isolation_filters_by_tenant_id() {
        // Use in_memory(): a private, per-repo in-memory DB. open(":memory:")
        // is the file-backed API and creates a real file named ":memory:" in
        // the cwd, so parallel test runs race on DuckDB's file lock.
        let repo = DuckDbRepository::in_memory().unwrap();

        repo.upsert_canonical_skill(CanonicalSkillUpsert {
            skill_ref: "skill-a".to_string(),
            tenant_id: "tenant-a".to_string(),
            name: "Skill A".to_string(),
            path: "/a".to_string(),
            has_skill_md: true,
            content_hash: Some("hash-a".to_string()),
            manifest_json: Some("{}".to_string()),
        })
        .unwrap();
        repo.upsert_canonical_skill(CanonicalSkillUpsert {
            skill_ref: "skill-b".to_string(),
            tenant_id: "tenant-b".to_string(),
            name: "Skill B".to_string(),
            path: "/b".to_string(),
            has_skill_md: true,
            content_hash: Some("hash-b".to_string()),
            manifest_json: Some("{}".to_string()),
        })
        .unwrap();

        let a_skills = repo.list_canonical_skills("tenant-a").unwrap();
        assert_eq!(a_skills.len(), 1);
        assert_eq!(a_skills[0].skill_ref, "skill-a");

        let b_skills = repo.list_canonical_skills("tenant-b").unwrap();
        assert_eq!(b_skills.len(), 1);
        assert_eq!(b_skills[0].skill_ref, "skill-b");
    }

    #[test]
    fn default_tenant_is_used_when_column_missing() {
        // When querying a tenant_id that has no rows, return empty
        // Use in_memory(): a private, per-repo in-memory DB. open(":memory:")
        // is the file-backed API and creates a real file named ":memory:" in
        // the cwd, so parallel test runs race on DuckDB's file lock.
        let repo = DuckDbRepository::in_memory().unwrap();
        let skills = repo.list_canonical_skills("nonexistent").unwrap();
        assert!(skills.is_empty());
    }
}
