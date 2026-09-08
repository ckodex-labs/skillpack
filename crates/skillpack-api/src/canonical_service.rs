//! Canonical Store Service Implementation
//!
//! gRPC service for canonical store operations: sync, migrate,
//! boundary checks, import, promote, and status.

use crate::cache::{EventBus, ServerEvent};
use crate::proto::{
    AgentIntegrationType, AgentStatus, AgentSyncResult, CheckBoundaryRequest,
    CheckBoundaryResponse, GetStatusRequest, GetStatusResponse, ImportSkillRequest,
    ImportSkillResponse, MigrateAgentsRequest, MigrateAgentsResponse, MigrateAllRequest,
    MigrateAllResponse, MigrateClaudeAgentsRequest, MigrateClaudeAgentsResponse,
    MigrateHarnessesRequest, MigrateHarnessesResponse, MigrateOutcome, MigrateResult,
    MigrateSkillsRequest, MigrateSkillsResponse, PackageSkillRequest, PackageSkillResponse,
    PromoteSkillRequest, PromoteSkillResponse, PullSkillRequest, PullSkillResponse,
    RollbackSyncRequest, RollbackSyncResponse, SyncAgentsRequest, SyncAgentsResponse, SyncEvent,
    canonical_store_service_server::{CanonicalStoreService, CanonicalStoreServiceServer},
};
use skillpack_adapters::oci::OciReader;
use skillpack_adapters::persistence::{CanonicalSkillUpsert, DuckDbRepository};
use skillpack_application::{
    CheckBoundaryRequest as DomainCheckBoundaryRequest, CheckBoundaryUseCase, MigrateSkillsUseCase,
    SyncCanonicalStoreUseCase,
};
use skillpack_domain::{
    AgentIntegrationType as DomainAgentIntegrationType, AgentRegistry, CanonicalStore, IPGuard,
    MigrateAllRequest as DomainMigrateAllRequest, MigrateSummary,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::info;

pub struct CanonicalStoreServiceImpl {
    canonical_root: PathBuf,
    event_bus: Arc<EventBus>,
    ip_guard: Arc<IPGuard>,
    repository: Option<Arc<DuckDbRepository>>,
}

impl CanonicalStoreServiceImpl {
    pub fn new(canonical_root: impl Into<PathBuf>, event_bus: Arc<EventBus>) -> Self {
        Self {
            canonical_root: canonical_root.into(),
            event_bus,
            ip_guard: Arc::new(IPGuard::new()),
            repository: None,
        }
    }

    pub fn with_repository(
        canonical_root: impl Into<PathBuf>,
        event_bus: Arc<EventBus>,
        repository: Arc<DuckDbRepository>,
    ) -> Self {
        Self {
            canonical_root: canonical_root.into(),
            event_bus,
            ip_guard: Arc::new(IPGuard::new()),
            repository: Some(repository),
        }
    }

    pub fn into_service(self) -> CanonicalStoreServiceServer<Self> {
        CanonicalStoreServiceServer::new(self)
    }

    fn default_canonical_root() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        home.join("Skills").join("shared")
    }

    pub fn canonical_root(&self) -> PathBuf {
        if self.canonical_root.as_os_str().is_empty() {
            Self::default_canonical_root()
        } else {
            self.canonical_root.clone()
        }
    }
}

impl Default for CanonicalStoreServiceImpl {
    fn default() -> Self {
        Self {
            canonical_root: Self::default_canonical_root(),
            event_bus: Arc::new(EventBus::default()),
            ip_guard: Arc::new(IPGuard::new()),
            repository: None,
        }
    }
}

#[tonic::async_trait]
impl CanonicalStoreService for CanonicalStoreServiceImpl {
    async fn sync_agents(
        &self,
        request: Request<SyncAgentsRequest>,
    ) -> Result<Response<SyncAgentsResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!(
            "SyncAgents request: dry_run={}, no_index={}, only_agent={:?}",
            req.dry_run, req.no_index, req.only_agent
        );

        self.event_bus.publish(ServerEvent::SyncStarted);

        let canonical_root = self.canonical_root();
        let store = CanonicalStore {
            root: canonical_root,
            dry_run: req.dry_run,
            do_index: !req.no_index,
            only_agent: req.only_agent.filter(|s| !s.is_empty()),
            namespace: req.namespace.filter(|s| !s.is_empty()),
            oci_push: false,
            oci_registry: None,
            oci_repository: None,
        };

        let use_case = SyncCanonicalStoreUseCase::with_ip_guard(self.ip_guard.clone());
        let result = use_case
            .execute(&store)
            .map_err(|e| Status::internal(e.to_string()))?;

        // Emit per-agent progress events for streaming subscribers
        let total_agents = result.agent_results.len() as i32;
        for (idx, agent_result) in result.agent_results.iter().enumerate() {
            self.event_bus.publish(ServerEvent::SyncProgress {
                agent_name: agent_result.agent_name.clone(),
                skill_name: "".to_string(),
                status: if agent_result.success {
                    "synced".to_string()
                } else {
                    "failed".to_string()
                },
                current: idx as i32 + 1,
                total: total_agents,
            });
        }

        self.event_bus.publish(ServerEvent::SyncCompleted {
            skills_processed: result.skills_processed as i32,
        });

        // Persist canonical skills and sync state to DuckDB
        if let Some(repo) = &self.repository {
            for skill in &result.skills_discovered {
                let content_hash = skill
                    .path
                    .join("SKILL.md")
                    .canonicalize()
                    .ok()
                    .and_then(|p| std::fs::read(&p).ok())
                    .map(|bytes| {
                        use sha2::{Digest, Sha256};
                        let hash = Sha256::digest(&bytes);
                        let hex = hash
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        format!("sha256:{}", hex)
                    });
                let _ = repo.upsert_canonical_skill(CanonicalSkillUpsert {
                    skill_ref: skill.name.clone(),
                    tenant_id: "default".to_string(),
                    name: skill.name.clone(),
                    path: skill.path.to_string_lossy().into_owned(),
                    has_skill_md: skill.has_skill_md,
                    content_hash,
                    manifest_json: None,
                });
            }
            for agent_result in &result.agent_results {
                let _ = repo.upsert_sync_state(
                    &agent_result.agent_name,
                    "*",
                    if agent_result.success {
                        "synced"
                    } else {
                        "failed"
                    },
                    None,
                    "sync",
                );
            }
            let sync_id = format!("sync-{}", chrono::Utc::now().timestamp_millis());
            let _ = repo.record_sync_history(
                &sync_id,
                result.skills_processed as i32,
                result.agent_results.len() as i32,
                result.success,
                Some(&result.message),
            );
        }

        let agent_results: Vec<AgentSyncResult> = result
            .agent_results
            .into_iter()
            .map(|r| AgentSyncResult {
                agent_name: r.agent_name,
                success: r.success,
                message: r.message,
                skills_synced: r.skills_synced as i32,
            })
            .collect();

        Ok(Response::new(SyncAgentsResponse {
            success: result.success,
            message: result.message,
            skills_processed: result.skills_processed as i32,
            agent_results,
        }))
    }

    async fn check_boundary(
        &self,
        request: Request<CheckBoundaryRequest>,
    ) -> Result<Response<CheckBoundaryResponse>, Status> {
        let req = request.into_inner();
        info!("CheckBoundary request: path={}", req.target_path);

        let use_case = CheckBoundaryUseCase::new();
        let result = use_case.execute(DomainCheckBoundaryRequest {
            target_path: req.target_path,
            skill_name: req.skill_name,
        });

        Ok(Response::new(CheckBoundaryResponse {
            is_safe: result.is_safe,
            message: result.message,
            violations: result.violations,
        }))
    }

    async fn migrate_all(
        &self,
        request: Request<MigrateAllRequest>,
    ) -> Result<Response<MigrateAllResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("MigrateAll request: canonical_root={}", req.canonical_root);

        let canonical_root = if req.canonical_root.is_empty() {
            self.canonical_root()
        } else {
            PathBuf::from(req.canonical_root)
        };

        let use_case = MigrateSkillsUseCase::with_ip_guard(self.ip_guard.clone());
        let results = use_case
            .execute(DomainMigrateAllRequest { canonical_root })
            .map_err(|e| Status::internal(e.to_string()))?;

        let summary = MigrateSummary::from_results(&results);

        let proto_results: Vec<MigrateResult> = results
            .into_iter()
            .map(|r| {
                let (outcome, message) = match r.outcome {
                    skillpack_domain::MigrateOutcome::Migrated => (MigrateOutcome::Migrated, None),
                    skillpack_domain::MigrateOutcome::ReplacedWithLink => {
                        (MigrateOutcome::ReplacedWithLink, None)
                    }
                    skillpack_domain::MigrateOutcome::SkippedAlreadyLinked => {
                        (MigrateOutcome::SkippedAlreadyLinked, None)
                    }
                    skillpack_domain::MigrateOutcome::SkippedIPViolation(msg) => {
                        (MigrateOutcome::SkippedIpViolation, Some(msg))
                    }
                    skillpack_domain::MigrateOutcome::Failed(msg) => {
                        (MigrateOutcome::Failed, Some(msg))
                    }
                };
                MigrateResult {
                    agent_name: r.agent_name,
                    skill_name: r.skill_name,
                    source_path: r.source_path.to_string_lossy().to_string(),
                    outcome: outcome as i32,
                    message,
                }
            })
            .collect();

        Ok(Response::new(MigrateAllResponse {
            success: summary.failed == 0,
            message: format!(
                "Migration complete: {} migrated, {} replaced, {} skipped, {} IP-blocked, {} failed",
                summary.migrated,
                summary.replaced,
                summary.skipped_linked,
                summary.skipped_ip,
                summary.failed
            ),
            results: proto_results,
            migrated_count: summary.migrated as i32,
            replaced_count: summary.replaced as i32,
            skipped_linked_count: summary.skipped_linked as i32,
            skipped_ip_count: summary.skipped_ip as i32,
        }))
    }

    async fn import_skill(
        &self,
        request: Request<ImportSkillRequest>,
    ) -> Result<Response<ImportSkillResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("ImportSkill request: source={}", req.source_path);

        let source = PathBuf::from(&req.source_path);
        if !source.exists() {
            return Err(Status::not_found(format!(
                "Source path not found: {}",
                req.source_path
            )));
        }

        let canonical_root = self.canonical_root();
        // SEC: user input joins a trusted root — reject any absolute or
        // separator-carrying component so the join cannot be steered outside
        // the canonical store.
        let raw_target = req.target_name.clone().unwrap_or_else(|| {
            source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        let target_name = skillpack_application::validate_skill_component(&raw_target)
            .map_err(|e| Status::invalid_argument(e.to_string()))?
            .to_string();
        let target = canonical_root.join(&target_name);

        if target.exists() {
            return Err(Status::already_exists(format!(
                "Skill '{}' already exists in canonical store",
                target_name
            )));
        }

        std::fs::create_dir_all(&canonical_root).map_err(|e| Status::internal(e.to_string()))?;
        copy_dir_all(&source, &target).map_err(|e| Status::internal(e.to_string()))?;

        self.event_bus.publish(ServerEvent::SkillCreated {
            skill_ref: target_name.clone(),
        });

        Ok(Response::new(ImportSkillResponse {
            success: true,
            message: format!("Imported '{}' into canonical store", target_name),
            skill_name: target_name,
            canonical_path: target.to_string_lossy().to_string(),
        }))
    }

    async fn promote_skill(
        &self,
        request: Request<PromoteSkillRequest>,
    ) -> Result<Response<PromoteSkillResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("PromoteSkill request: candidate={}", req.candidate_name);

        let canonical_root = self.canonical_root();
        // SEC: user input joins a trusted root — validate as a single path
        // component before doing filesystem work.
        let candidate = skillpack_application::validate_skill_component(&req.candidate_name)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let candidates_dir = canonical_root.join("candidates");
        let candidate_path = candidates_dir.join(candidate);
        let target_path = canonical_root.join(candidate);

        if !candidate_path.exists() {
            return Err(Status::not_found(format!(
                "Candidate skill '{}' does not exist",
                req.candidate_name
            )));
        }

        if target_path.exists() {
            return Err(Status::already_exists(format!(
                "Skill '{}' already exists in active canonical store",
                req.candidate_name
            )));
        }

        std::fs::rename(&candidate_path, &target_path)
            .map_err(|e| Status::internal(e.to_string()))?;

        self.event_bus.publish(ServerEvent::SkillUpdated {
            skill_ref: req.candidate_name.clone(),
        });

        Ok(Response::new(PromoteSkillResponse {
            success: true,
            message: format!(
                "Promoted '{}' to active canonical store",
                req.candidate_name
            ),
            skill_name: req.candidate_name,
        }))
    }

    async fn get_status(
        &self,
        _request: Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        let canonical_root = self.canonical_root();
        let registry = AgentRegistry::default_registry();

        let total_skills = if canonical_root.exists() {
            std::fs::read_dir(&canonical_root)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            name != "candidates" && name != ".codegraph"
                        })
                        .count() as i32
                })
                .unwrap_or(0)
        } else {
            0
        };

        let active_agents: Vec<AgentStatus> = registry
            .agents()
            .iter()
            .map(|a| {
                let agent_dir_opt = a.resolved_dir();
                let skills_count = agent_dir_opt.as_ref().map_or(0, |agent_dir| {
                    if agent_dir.exists() {
                        std::fs::read_dir(agent_dir)
                            .map(|entries| entries.flatten().count() as i32)
                            .unwrap_or(0)
                    } else {
                        0
                    }
                });

                let integration_type = match a.integration_type {
                    DomainAgentIntegrationType::Symlink => AgentIntegrationType::Symlink,
                    DomainAgentIntegrationType::IndexFile => AgentIntegrationType::IndexFile,
                    DomainAgentIntegrationType::RulesDir => AgentIntegrationType::IndexFile,
                    DomainAgentIntegrationType::SingleFile => AgentIntegrationType::IndexFile,
                };

                AgentStatus {
                    name: a.name.clone(),
                    path: a.default_dir.to_string_lossy().to_string(),
                    integration_type: integration_type as i32,
                    is_active: agent_dir_opt.as_ref().is_some_and(|d| d.exists()),
                    skills_count,
                }
            })
            .collect();

        Ok(Response::new(GetStatusResponse {
            canonical_root: canonical_root.to_string_lossy().to_string(),
            total_skills,
            active_agents,
            is_healthy: true,
        }))
    }

    type SyncAgentsStreamStream = ReceiverStream<Result<SyncEvent, Status>>;
    // This type alias is defined in the generated trait

    async fn sync_agents_stream(
        &self,
        _request: Request<SyncAgentsRequest>,
    ) -> Result<Response<Self::SyncAgentsStreamStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let mut event_rx = self.event_bus.subscribe();

        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                let sync_event = match event {
                    ServerEvent::SyncStarted => SyncEvent {
                        event: Some(crate::proto::sync_event::Event::Progress(
                            crate::proto::SyncProgress {
                                agent_name: "*".to_string(),
                                skill_name: "".to_string(),
                                status: "sync_started".to_string(),
                                current: 0,
                                total: 0,
                            },
                        )),
                    },
                    ServerEvent::SyncProgress {
                        agent_name,
                        skill_name,
                        status,
                        current,
                        total,
                    } => SyncEvent {
                        event: Some(crate::proto::sync_event::Event::Progress(
                            crate::proto::SyncProgress {
                                agent_name,
                                skill_name,
                                status,
                                current,
                                total,
                            },
                        )),
                    },
                    ServerEvent::SyncCompleted { skills_processed } => SyncEvent {
                        event: Some(crate::proto::sync_event::Event::Complete(
                            crate::proto::SyncComplete {
                                success: true,
                                message: format!("Synced {} skills", skills_processed),
                                skills_processed,
                            },
                        )),
                    },
                    // Ignore unrelated events
                    _ => continue,
                };
                if tx.send(Ok(sync_event)).await.is_err() {
                    break; // Client disconnected
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn rollback_sync(
        &self,
        request: Request<RollbackSyncRequest>,
    ) -> Result<Response<RollbackSyncResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("RollbackSync request: agent_name={:?}", req.agent_name);

        let registry = AgentRegistry::default_registry();
        let mut restored_agents = Vec::new();
        let mut removed_skills = 0;

        for agent in registry.agents() {
            if let Some(ref only) = req.agent_name
                && agent.name.to_lowercase() != only.to_lowercase()
            {
                continue;
            }

            let agent_dir = match agent.resolved_dir() {
                Some(d) => d,
                None => continue,
            };

            if !agent_dir.exists() {
                continue;
            }

            match agent.integration_type {
                DomainAgentIntegrationType::Symlink => {
                    // Remove all symlinks in the agent dir
                    let entries: Vec<_> = match std::fs::read_dir(&agent_dir) {
                        Ok(d) => d.flatten().collect(),
                        Err(_) => vec![],
                    };
                    for entry in entries {
                        let path = entry.path();
                        let is_link = std::fs::symlink_metadata(&path)
                            .map(|m| m.file_type().is_symlink())
                            .unwrap_or(false);
                        if is_link {
                            if path.is_dir() {
                                let _ = std::fs::remove_dir_all(&path);
                            } else {
                                let _ = std::fs::remove_file(&path);
                            }
                            removed_skills += 1;
                        }
                    }
                }
                DomainAgentIntegrationType::IndexFile => {
                    match agent.name.as_str() {
                        "Cursor" => {
                            let rules_dir = agent_dir.parent().unwrap_or(&agent_dir).join("rules");
                            let entries: Vec<_> = match std::fs::read_dir(&rules_dir) {
                                Ok(d) => d.flatten().collect(),
                                Err(_) => vec![],
                            };
                            for entry in entries {
                                let path = entry.path();
                                if let Some(ext) = path.extension()
                                    && ext == "mdc"
                                {
                                    let _ = std::fs::remove_file(&path);
                                    removed_skills += 1;
                                }
                            }
                        }
                        _ => {
                            // Remove index.json
                            let index_path = agent_dir.join("index.json");
                            if index_path.exists() {
                                let _ = std::fs::remove_file(&index_path);
                                removed_skills += 1;
                            }
                        }
                    }
                }
                // RulesDir and SingleFile are managed by skillpack-registry-sync;
                // skip rollback for them to avoid partial state.
                DomainAgentIntegrationType::RulesDir | DomainAgentIntegrationType::SingleFile => {}
            }

            restored_agents.push(agent.name.clone());
        }

        // Clear sync state from DuckDB
        if let Some(repo) = &self.repository {
            // agent_sync_state doesn't have a bulk delete, so we skip for now
            let _ = repo.record_sync_history(
                &format!("rollback-{}", chrono::Utc::now().timestamp_millis()),
                removed_skills,
                restored_agents.len() as i32,
                true,
                Some(&format!("Rolled back {} agents", restored_agents.len())),
            );
        }

        Ok(Response::new(RollbackSyncResponse {
            success: true,
            message: format!(
                "Rolled back {} skills from {} agents",
                removed_skills,
                restored_agents.len()
            ),
            restored_agents,
            removed_skills,
        }))
    }

    async fn package_skill(
        &self,
        request: Request<PackageSkillRequest>,
    ) -> Result<Response<PackageSkillResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!(
            "PackageSkill request: skill_ref={}, registry_url={:?}",
            req.skill_ref, req.registry_url
        );

        let canonical_root = self.canonical_root();
        let ns = req.namespace.as_deref().unwrap_or("default");
        // SEC: user input joins a trusted root — validate namespace and
        // skill_ref as single path components before joining.
        let ns = skillpack_application::validate_skill_component(ns)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let skill_ref = skillpack_application::validate_skill_component(&req.skill_ref)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let skill_path = canonical_root.join(ns).join(skill_ref);
        if !skill_path.exists() {
            return Err(Status::not_found(format!(
                "Skill '{}' not found in namespace '{}'",
                req.skill_ref, ns
            )));
        }

        let registry = req.registry_url.as_deref().unwrap_or("localhost:5000");
        let repository = format!("skillpack/{}", skill_ref);
        let tag = ns;

        let oci_auth: Option<(&str, &str)> = std::env::var("SKILLPACK_OCI_AUTH")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| {
                let leaked = Box::leak(s.into_boxed_str());
                leaked.split_once(':')
            });
        let result = OciReader::push_skill(registry, &repository, tag, &skill_path, oci_auth)
            .await
            .map_err(|e| Status::internal(format!("OCI push failed: {}", e)))?;

        Ok(Response::new(PackageSkillResponse {
            success: true,
            message: format!(
                "Pushed '{}' to {}/{}:{}",
                req.skill_ref, registry, repository, tag
            ),
            artifact_digest: result.digest,
            manifest_url: result.manifest_url,
        }))
    }

    async fn pull_skill(
        &self,
        request: Request<PullSkillRequest>,
    ) -> Result<Response<PullSkillResponse>, Status> {
        let req = request.into_inner();
        info!(
            "PullSkill request: skill_ref={}, source={}",
            req.skill_ref, req.source_registry
        );

        // Pull OCI artifact from registry and extract to canonical store
        // NOTE: Not yet implemented for alpha.
        return Err(Status::unimplemented(
            "PullSkill is not yet implemented for the alpha release",
        ));
    }

    async fn migrate_skills(
        &self,
        request: Request<MigrateSkillsRequest>,
    ) -> Result<Response<MigrateSkillsResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("MigrateSkills request");

        let root: PathBuf = req
            .canonical_root
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.canonical_root());

        let mut results: Vec<(String, Vec<String>)> = Vec::new();

        if root.exists() {
            // Root-level skills
            for entry in std::fs::read_dir(&root)
                .map_err(|e| Status::internal(e.to_string()))?
                .flatten()
            {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let changes = skillpack_adapters::skill_migration::migrate_skill_file(
                        &path.join("SKILL.md"),
                        &name,
                        "default",
                        "Apache-2.0",
                        None,
                    )
                    .map_err(|e| Status::internal(e.to_string()))?;
                    results.push((name, changes));
                }
            }
            // Namespace-level skills
            for entry in std::fs::read_dir(&root)
                .map_err(|e| Status::internal(e.to_string()))?
                .flatten()
            {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let ns = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                for sub in std::fs::read_dir(&path)
                    .map_err(|e| Status::internal(e.to_string()))?
                    .flatten()
                {
                    let sub_path = sub.path();
                    if sub_path.is_dir() && sub_path.join("SKILL.md").exists() {
                        let name = sub_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let changes = skillpack_adapters::skill_migration::migrate_skill_file(
                            &sub_path.join("SKILL.md"),
                            &name,
                            &ns,
                            "Apache-2.0",
                            None,
                        )
                        .map_err(|e| Status::internal(e.to_string()))?;
                        results.push((name, changes));
                    }
                }
            }
        }

        let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
        let ok: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();

        Ok(Response::new(MigrateSkillsResponse {
            success: true,
            message: format!(
                "Migrated {} skills, {} already compliant",
                migrated.len(),
                ok.len()
            ),
            total_scanned: results.len() as i32,
            migrated_count: migrated.len() as i32,
            already_compliant_count: ok.len() as i32,
            migrated_names: migrated.iter().map(|(n, _)| n.clone()).collect(),
        }))
    }

    async fn migrate_agents(
        &self,
        _request: Request<MigrateAgentsRequest>,
    ) -> Result<Response<MigrateAgentsResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        info!("MigrateAgents request");

        let root = dirs::home_dir()
            .map(|h| h.join(".config/agents/skills"))
            .unwrap_or_else(|| PathBuf::from("~/.config/agents/skills"));

        let mut results: Vec<(String, Vec<String>)> = Vec::new();

        if root.exists() {
            for entry in std::fs::read_dir(&root)
                .map_err(|e| Status::internal(e.to_string()))?
                .flatten()
            {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let changes = skillpack_adapters::skill_migration::migrate_skill_file(
                        &path.join("SKILL.md"),
                        &name,
                        "agents",
                        "MIT",
                        Some("vercel"),
                    )
                    .map_err(|e| Status::internal(e.to_string()))?;
                    results.push((name, changes));
                }
            }
        }

        let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
        let ok: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();

        Ok(Response::new(MigrateAgentsResponse {
            success: true,
            message: format!(
                "Migrated {} agent skills, {} already compliant",
                migrated.len(),
                ok.len()
            ),
            total_scanned: results.len() as i32,
            migrated_count: migrated.len() as i32,
            already_compliant_count: ok.len() as i32,
            migrated_names: migrated.iter().map(|(n, _)| n.clone()).collect(),
        }))
    }

    async fn migrate_claude_agents(
        &self,
        request: Request<MigrateClaudeAgentsRequest>,
    ) -> Result<Response<MigrateClaudeAgentsResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        let req = request.into_inner();
        info!("MigrateClaudeAgents request");

        let root: PathBuf = req
            .agents_dir
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".claude/agents"))
                    .unwrap_or_else(|| PathBuf::from("~/.claude/agents"))
            });

        let mut results: Vec<(String, Vec<String>)> = Vec::new();

        if root.exists() {
            for entry in std::fs::read_dir(&root)
                .map_err(|e| Status::internal(e.to_string()))?
                .flatten()
            {
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let changes = skillpack_adapters::skill_migration::migrate_skill_file(
                        &path,
                        &name,
                        "claude",
                        "MIT",
                        Some("ckodex"),
                    )
                    .map_err(|e| Status::internal(e.to_string()))?;
                    results.push((name, changes));
                }
            }
        }

        let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
        let ok: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();

        Ok(Response::new(MigrateClaudeAgentsResponse {
            success: true,
            message: format!(
                "Migrated {} Claude agents, {} already compliant",
                migrated.len(),
                ok.len()
            ),
            total_scanned: results.len() as i32,
            migrated_count: migrated.len() as i32,
            already_compliant_count: ok.len() as i32,
            migrated_names: migrated.iter().map(|(n, _)| n.clone()).collect(),
        }))
    }

    async fn migrate_harnesses(
        &self,
        _request: Request<MigrateHarnessesRequest>,
    ) -> Result<Response<MigrateHarnessesResponse>, Status> {
        crate::auth::ensure_token_configured()?;
        info!("MigrateHarnesses request");

        let root = dirs::home_dir()
            .map(|h| h.join(".config/agents/skills"))
            .unwrap_or_else(|| PathBuf::from("~/.config/agents/skills"));

        let mut results: Vec<(String, Vec<String>)> = Vec::new();

        if root.exists() {
            for skill_dir in std::fs::read_dir(&root)
                .map_err(|e| Status::internal(e.to_string()))?
                .flatten()
            {
                let path = skill_dir.path();
                if !path.is_dir() {
                    continue;
                }
                let rules_dir = path.join("rules");
                if !rules_dir.exists() {
                    continue;
                }
                let skill_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                for rule_entry in std::fs::read_dir(&rules_dir)
                    .map_err(|e| Status::internal(e.to_string()))?
                    .flatten()
                {
                    let rule_path = rule_entry.path();
                    if rule_path.is_file()
                        && rule_path.extension().map(|e| e == "md").unwrap_or(false)
                    {
                        let name = rule_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let changes = skillpack_adapters::skill_migration::migrate_skill_file(
                            &rule_path,
                            &name,
                            &skill_name,
                            "MIT",
                            None,
                        )
                        .map_err(|e| Status::internal(e.to_string()))?;
                        results.push((name, changes));
                    }
                }
            }
        }

        let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
        let ok: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();

        Ok(Response::new(MigrateHarnessesResponse {
            success: true,
            message: format!(
                "Migrated {} harnesses, {} already compliant",
                migrated.len(),
                ok.len()
            ),
            total_scanned: results.len() as i32,
            migrated_count: migrated.len() as i32,
            already_compliant_count: ok.len() as i32,
            migrated_names: migrated.iter().map(|(n, _)| n.clone()).collect(),
        }))
    }
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
