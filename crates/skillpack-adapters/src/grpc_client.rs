//! gRPC Client for CanonicalStoreService
//!
//! Thin wrapper around the generated tonic client that connects to the
//! local skillpack-server and provides typed methods for store operations.
//!
//! Transport contract per CLIENT-SPEC.md §3:
//! - Discovery: checks SKILLPACK_API_URL env var; defaults to http://localhost:50051
//! - Health probe: GetStatus with 2s timeout
//! - Auth: Bearer token from SKILLPACK_TOKEN

use skillpack_proto::proto::{
    CheckBoundaryRequest, GetStatusRequest, MigrateAgentsRequest, MigrateAllRequest,
    MigrateClaudeAgentsRequest, MigrateHarnessesRequest, MigrateSkillsRequest, SyncAgentsRequest,
    canonical_store_service_client::CanonicalStoreServiceClient,
};
use tonic::{Request, metadata::MetadataValue, transport::Channel};

/// Error type for gRPC client operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreClientError {
    #[error("Connection failed: {0}")]
    Connection(#[from] tonic::transport::Error),
    #[error("RPC failed: {0}")]
    Rpc(#[from] tonic::Status),
    #[error("Server not running at {0}")]
    ServerUnavailable(String),
}

/// Client for the CanonicalStoreService gRPC endpoint.
pub struct CanonicalStoreClient {
    addr: String,
    token: Option<String>,
}

impl CanonicalStoreClient {
    /// Create a new client pointing at the given address.
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            token: None,
        }
    }

    /// Create a client from environment variables (SKILLPACK_API_URL / SKILLPACK_TOKEN).
    /// Per CLIENT-SPEC.md §3.1: defaults to the local server on port 50051.
    /// Uses the 127.0.0.1 literal rather than `localhost` — resolver stalls on
    /// `localhost` are not covered by connect_timeout and hang the CLI.
    pub fn from_env() -> Self {
        let addr = std::env::var("SKILLPACK_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
        let token = std::env::var("SKILLPACK_TOKEN").ok();
        Self { addr, token }
    }

    async fn connect(&self) -> Result<CanonicalStoreServiceClient<Channel>, StoreClientError> {
        let endpoint = Channel::from_shared(self.addr.clone())
            .map_err(|e| StoreClientError::ServerUnavailable(format!("{}: {}", self.addr, e)))?
            .connect_timeout(std::time::Duration::from_secs(2))
            // Per CLIENT-SPEC.md §2.2: unary RPCs default to a 30s deadline.
            .timeout(std::time::Duration::from_secs(30));
        // connect_timeout does not cover DNS resolution — bound the whole
        // connect phase so a resolver stall cannot hang the CLI.
        let channel = tokio::time::timeout(std::time::Duration::from_secs(5), endpoint.connect())
            .await
            .map_err(|_| {
                StoreClientError::ServerUnavailable(format!(
                    "{} (connect timed out after 5s — is skillpack-server running?)",
                    self.addr
                ))
            })??;
        Ok(CanonicalStoreServiceClient::new(channel))
    }

    fn add_auth<T>(&self, req: Request<T>) -> Request<T> {
        let mut req = req;
        if let Some(token) = &self.token
            && let Ok(val) = MetadataValue::try_from(format!("Bearer {}", token))
        {
            req.metadata_mut().insert("authorization", val);
        }
        req
    }

    /// Get canonical store status.
    pub async fn status(
        &self,
    ) -> Result<skillpack_proto::proto::GetStatusResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(GetStatusRequest {}));
        let resp = client.get_status(req).await?;
        Ok(resp.into_inner())
    }

    /// Check if a path/skill violates IP boundaries.
    pub async fn check_boundary(
        &self,
        target_path: String,
        skill_name: Option<String>,
    ) -> Result<skillpack_proto::proto::CheckBoundaryResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(CheckBoundaryRequest {
            target_path,
            skill_name,
        }));
        let resp = client.check_boundary(req).await?;
        Ok(resp.into_inner())
    }

    /// Migrate all physical skills into the canonical store.
    pub async fn migrate_all(
        &self,
        canonical_root: String,
    ) -> Result<skillpack_proto::proto::MigrateAllResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(MigrateAllRequest { canonical_root }));
        let resp = client.migrate_all(req).await?;
        Ok(resp.into_inner())
    }

    /// Sync agents with the canonical store.
    pub async fn sync_agents(
        &self,
        dry_run: bool,
        no_index: bool,
        only_agent: Option<String>,
    ) -> Result<skillpack_proto::proto::SyncAgentsResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(SyncAgentsRequest {
            dry_run,
            no_index,
            only_agent,
            namespace: None,
        }));
        let resp = client.sync_agents(req).await?;
        Ok(resp.into_inner())
    }

    /// Migrate skills in the canonical store.
    pub async fn migrate_skills(
        &self,
        canonical_root: Option<String>,
    ) -> Result<skillpack_proto::proto::MigrateSkillsResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(MigrateSkillsRequest { canonical_root }));
        let resp = client.migrate_skills(req).await?;
        Ok(resp.into_inner())
    }

    /// Migrate agent skills.
    pub async fn migrate_agents(
        &self,
    ) -> Result<skillpack_proto::proto::MigrateAgentsResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(MigrateAgentsRequest {}));
        let resp = client.migrate_agents(req).await?;
        Ok(resp.into_inner())
    }

    /// Migrate Claude agent definitions.
    pub async fn migrate_claude_agents(
        &self,
        agents_dir: Option<String>,
    ) -> Result<skillpack_proto::proto::MigrateClaudeAgentsResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(MigrateClaudeAgentsRequest { agents_dir }));
        let resp = client.migrate_claude_agents(req).await?;
        Ok(resp.into_inner())
    }

    /// Migrate agent harness rules.
    pub async fn migrate_harnesses(
        &self,
    ) -> Result<skillpack_proto::proto::MigrateHarnessesResponse, StoreClientError> {
        let mut client = self.connect().await?;
        let req = self.add_auth(Request::new(MigrateHarnessesRequest {}));
        let resp = client.migrate_harnesses(req).await?;
        Ok(resp.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_addr_and_no_token() {
        let client = CanonicalStoreClient::new("https://example.com:50051");
        assert_eq!(client.addr, "https://example.com:50051");
        assert!(client.token.is_none());
    }

    #[test]
    #[serial_test::serial]
    fn from_env_uses_skillpack_api_url() {
        unsafe {
            std::env::set_var("SKILLPACK_API_URL", "https://api.skillpack.io");
            std::env::remove_var("SKILLPACK_TOKEN");
        }
        let client = CanonicalStoreClient::from_env();
        assert_eq!(client.addr, "https://api.skillpack.io");
        assert!(client.token.is_none());
        unsafe {
            std::env::remove_var("SKILLPACK_API_URL");
        }
    }

    #[test]
    #[serial_test::serial]
    fn from_env_defaults_to_loopback_literal_when_env_missing() {
        unsafe {
            std::env::remove_var("SKILLPACK_API_URL");
            std::env::remove_var("SKILLPACK_TOKEN");
        }
        let client = CanonicalStoreClient::from_env();
        // Literal loopback, not `localhost`: resolver stalls on the hostname
        // escape connect_timeout and hang the CLI indefinitely.
        assert_eq!(client.addr, "http://127.0.0.1:50051");
        assert!(client.token.is_none());
    }

    #[test]
    #[serial_test::serial]
    fn from_env_reads_skillpack_token() {
        unsafe {
            std::env::set_var("SKILLPACK_API_URL", "https://api.skillpack.io");
            std::env::set_var("SKILLPACK_TOKEN", "my-secret-token");
        }
        let client = CanonicalStoreClient::from_env();
        assert_eq!(client.addr, "https://api.skillpack.io");
        assert_eq!(client.token, Some("my-secret-token".to_string()));
        unsafe {
            std::env::remove_var("SKILLPACK_API_URL");
            std::env::remove_var("SKILLPACK_TOKEN");
        }
    }

    #[test]
    fn add_auth_injects_bearer_header_when_token_present() {
        let client = CanonicalStoreClient {
            addr: "http://localhost:50051".to_string(),
            token: Some("test-token".to_string()),
        };
        let req = Request::new(GetStatusRequest {});
        let req = client.add_auth(req);
        let val = req.metadata().get("authorization");
        assert!(val.is_some());
        assert_eq!(val.unwrap(), "Bearer test-token");
    }

    #[test]
    fn add_auth_skips_header_when_token_absent() {
        let client = CanonicalStoreClient {
            addr: "http://localhost:50051".to_string(),
            token: None,
        };
        let req = Request::new(GetStatusRequest {});
        let req = client.add_auth(req);
        let val = req.metadata().get("authorization");
        assert!(val.is_none());
    }
}
