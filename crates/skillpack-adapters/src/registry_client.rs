//! Remote Registry Client
//!
//! HTTP client for federated skill discovery.
//! Fetches skills from remote registries via .well-known/skills.json.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Remote registry client
pub struct RegistryClient {
    client: reqwest::Client,
    _timeout: Duration,
}

/// Remote registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRegistry {
    pub name: String,
    pub base_url: String,
    pub trust_level: TrustLevel,
    #[serde(default)]
    pub verify_signatures: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TrustLevel {
    /// Fully trusted (official registries)
    Full,
    /// Partially trusted (verified publisher)
    Verified,
    /// Untrusted (community, requires review)
    #[default]
    Community,
}

/// Discovery document from .well-known/skills.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDocument {
    pub version: String,
    pub issuer: String,
    #[serde(default)]
    pub registry_endpoint: String,
    pub skills: Vec<RemoteSkill>,
    #[serde(default)]
    pub federation: Option<FederationInfo>,
}

/// Remote skill entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub oci_ref: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub grade: Option<String>,
    #[serde(default)]
    pub gal_min: u8,
    #[serde(default)]
    pub gal_max: u8,
}

/// Federation peer info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationInfo {
    pub peers: Vec<String>,
    #[serde(default)]
    pub sync_interval_secs: u64,
}

impl RegistryClient {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(30))
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("skillpack-registry/1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            _timeout: timeout,
        }
    }

    /// Discover skills from a remote registry
    pub async fn discover(&self, registry: &RemoteRegistry) -> Result<DiscoveryDocument> {
        let url = format!(
            "{}/.well-known/skills.json",
            registry.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch discovery document: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Registry returned error: {}", response.status()));
        }

        let doc: DiscoveryDocument = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse discovery document: {}", e))?;

        Ok(doc)
    }

    /// Fetch skill metadata from registry
    pub async fn get_skill(
        &self,
        registry: &RemoteRegistry,
        skill_name: &str,
    ) -> Result<RemoteSkill> {
        let url = format!(
            "{}/api/v1/skills/{}",
            registry.base_url.trim_end_matches('/'),
            skill_name
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch skill: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Skill not found: {}", response.status()));
        }

        let skill: RemoteSkill = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse skill: {}", e))?;

        Ok(skill)
    }

    /// Search skills across a registry
    pub async fn search(&self, registry: &RemoteRegistry, query: &str) -> Result<Vec<RemoteSkill>> {
        let url = format!(
            "{}/api/v1/search?q={}",
            registry.base_url.trim_end_matches('/'),
            query
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to search: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Search failed: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct SearchResponse {
            skills: Vec<RemoteSkill>,
        }

        let result: SearchResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search results: {}", e))?;

        Ok(result.skills)
    }

    /// Fetch federation peers from a registry
    pub async fn get_federation_peers(&self, registry: &RemoteRegistry) -> Result<Vec<String>> {
        let doc = self.discover(registry).await?;
        Ok(doc.federation.map(|f| f.peers).unwrap_or_default())
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Federation coordinator for multi-registry sync
pub struct FederationCoordinator {
    client: RegistryClient,
    registries: Vec<RemoteRegistry>,
}

impl FederationCoordinator {
    pub fn new(registries: Vec<RemoteRegistry>) -> Self {
        Self {
            client: RegistryClient::new(),
            registries,
        }
    }

    /// Discover skills from all federated registries
    pub async fn discover_all(&self) -> Result<Vec<(String, Vec<RemoteSkill>)>> {
        let mut results = Vec::new();

        for registry in &self.registries {
            match self.client.discover(registry).await {
                Ok(doc) => {
                    results.push((registry.name.clone(), doc.skills));
                }
                Err(e) => {
                    tracing::warn!("Failed to discover from {}: {}", registry.name, e);
                }
            }
        }

        Ok(results)
    }

    /// Search across all registries
    pub async fn federated_search(&self, query: &str) -> Result<Vec<(String, RemoteSkill)>> {
        let mut results = Vec::new();

        for registry in &self.registries {
            match self.client.search(registry, query).await {
                Ok(skills) => {
                    for skill in skills {
                        results.push((registry.name.clone(), skill));
                    }
                }
                Err(e) => {
                    tracing::debug!("Search failed on {}: {}", registry.name, e);
                }
            }
        }

        Ok(results)
    }

    /// Build peer network by crawling federation links
    pub async fn build_peer_network(&self) -> Result<Vec<String>> {
        let mut all_peers = std::collections::HashSet::new();

        for registry in &self.registries {
            if let Ok(peers) = self.client.get_federation_peers(registry).await {
                for peer in peers {
                    all_peers.insert(peer);
                }
            }
        }

        Ok(all_peers.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_document_parsing() {
        let json = r#"{
            "version": "1.0",
            "issuer": "https://skills.example.com",
            "registry_endpoint": "https://registry.example.com",
            "skills": [
                {
                    "name": "security-scanner",
                    "description": "Security scanning skill",
                    "oci_ref": "ghcr.io/example/security-scanner:1.0.0",
                    "version": "1.0.0",
                    "tags": ["security", "audit"],
                    "grade": "A"
                }
            ],
            "federation": {
                "peers": ["https://peer1.example.com", "https://peer2.example.com"],
                "sync_interval_secs": 3600
            }
        }"#;

        let doc: DiscoveryDocument = serde_json::from_str(json).unwrap();
        assert_eq!(doc.version, "1.0");
        assert_eq!(doc.skills.len(), 1);
        assert_eq!(doc.skills[0].name, "security-scanner");
        assert!(doc.federation.is_some());
        assert_eq!(doc.federation.unwrap().peers.len(), 2);
    }

    #[test]
    fn test_remote_registry_config() {
        let registry = RemoteRegistry {
            name: "official".into(),
            base_url: "https://registry.skillpack.io".into(),
            trust_level: TrustLevel::Full,
            verify_signatures: true,
        };

        assert!(matches!(registry.trust_level, TrustLevel::Full));
    }
}
