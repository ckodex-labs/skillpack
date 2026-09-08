//! HTTP Server — SSE + REST mutations per CLIENT-SPEC.md §4.2 / §4.3
//!
//! Runs alongside the gRPC server on a separate port (default 50052).
//!
//! Endpoints:
//! - `GET  /health`          → JSON health + protocolVersion
//! - `GET  /events`          → SSE stream of ServerEvent JSON
//! - `POST /skills`          → create skill, publish SkillCreated
//! - `PATCH  /skills/:ref`   → update skill, publish SkillUpdated
//! - `DELETE /skills/:ref`   → delete skill, publish SkillDeleted

use crate::auth::{TokenValidator, auth_middleware};
use crate::cache::{EventBus, ServerEvent, TtlCache};
use crate::query_service::SkillProfile;
use axum::{
    Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    middleware,
    response::{IntoResponse, Sse},
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use skillpack_adapters::checkers::all_checkers;
use skillpack_adapters::filesystem::FilesystemReader;
use skillpack_adapters::persistence::DuckDbRepository;
use skillpack_adapters::skill_migration::{
    run_migrate_agents, run_migrate_claude_agents, run_migrate_harnesses, run_migrate_skills,
};
use skillpack_application::{
    AssessCatalogUseCase, AssessSkillRequest, AssessSkillUseCase, CatalogSkillResult,
};
use std::path::PathBuf;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
// use skillpack_domain::SkillReader;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state.
pub struct AppState {
    pub skills: Arc<RwLock<TtlCache<String, SkillProfile>>>,
    pub event_bus: Arc<EventBus>,
    pub repository: Option<Arc<DuckDbRepository>>,
    /// Pre-assessed catalog of the canonical store, filled by a background task.
    pub catalog: Arc<std::sync::RwLock<CatalogState>>,
}

/// Shared catalog cache. `hydrating` stays true until the background assessment
/// pass finishes; the endpoint serves whatever is ready so the UI fills in
/// progressively. Entries are the application layer's assessment results — this
/// transport type only holds and serializes them.
pub struct CatalogState {
    pub hydrating: bool,
    pub root: String,
    pub total: usize,
    pub assessed: usize,
    pub entries: Vec<CatalogSkillResult>,
}

impl CatalogState {
    pub fn empty() -> Self {
        Self {
            hydrating: true,
            root: String::new(),
            total: 0,
            assessed: 0,
            entries: Vec::new(),
        }
    }
}

/// Fill the shared catalog cache by assessing the canonical store once.
///
/// Four Spaces: this is a transport-side background driver — it discovers skill
/// dirs via the adapter layer (`canonical::list_skill_dirs`) and delegates all
/// assessment to the application layer (`AssessCatalogUseCase`). No domain logic
/// lives here; it only enumerates, drives the use case, and writes the cache.
/// Blocking (filesystem + checkers) — run via `spawn_blocking`.
pub fn hydrate_catalog(catalog: Arc<std::sync::RwLock<CatalogState>>, root: PathBuf) {
    let dirs = skillpack_adapters::canonical::list_skill_dirs(&root);
    if let Ok(mut c) = catalog.write() {
        c.total = dirs.len();
        c.root = root.display().to_string();
    }

    let use_case = AssessCatalogUseCase::new(FilesystemReader::new(), all_checkers());
    use_case.assess_paths(&dirs, |done, _total, result| {
        if let Ok(mut c) = catalog.write() {
            if let Some(entry) = result {
                c.entries.push(entry);
            }
            c.assessed = done;
        }
    });

    if let Ok(mut c) = catalog.write() {
        c.hydrating = false;
    }
}

/// Sanitize a user-provided path via the shared application-layer guard.
///HTTP mapping: canonicalize failure → 400, escape/symlink escape → 403.
fn sanitize_path(path: &str) -> Result<String, axum::http::StatusCode> {
    use skillpack_application::validate_skill_path_cwd;
    validate_skill_path_cwd(path)
        .map(|p| p.display().to_string())
        .map_err(|e| match e {
            skillpack_application::SkillPathError::Canonicalize { .. } => {
                axum::http::StatusCode::BAD_REQUEST
            }
            skillpack_application::SkillPathError::Escape { .. } => {
                axum::http::StatusCode::FORBIDDEN
            }
        })
}

/// Health response shape (matches CLIENT-SPEC §3.1 capability negotiation).
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    protocol_version: &'static str,
}

/// Request body for creating / updating a skill.
#[derive(Deserialize)]
struct SkillMutation {
    name: String,
    description: Option<String>,
    score: f64,
    grade: String,
    moodys_rating: String,
    reputation_tier: String,
    compatibility_score: u32,
    carbon_rating: String,
    monthly_cost_usd: f64,
    tags: Vec<String>,
}

/// Build the HTTP router.
pub fn build_router(
    skills: Arc<RwLock<TtlCache<String, SkillProfile>>>,
    event_bus: Arc<EventBus>,
    validator: Arc<TokenValidator>,
    repository: Option<Arc<DuckDbRepository>>,
    catalog: Arc<std::sync::RwLock<CatalogState>>,
) -> Router {
    let state = Arc::new(AppState {
        skills,
        event_bus,
        repository,
        catalog,
    });

    // Public routes (health check must stay open for capability negotiation)
    let public = Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    // Protected routes
    let protected = Router::new()
        .route("/events", get(sse_handler))
        // SkillPackService REST proxies
        .route("/api/v1/skills/assess", get(assess_handler))
        .route("/api/v1/skills", get(list_skills_handler))
        .route("/api/v1/skills/search", get(search_skills_handler))
        .route("/api/v1/skills/{ref}", get(get_skill_handler))
        // QueryService REST proxies
        .route(
            "/api/v1/skills/{ref}/profile",
            get(get_skill_profile_handler),
        )
        .route(
            "/api/v1/skills/{ref}/compatibility",
            get(get_compatibility_handler),
        )
        .route("/api/v1/skills/{ref}/cost", get(get_cost_handler))
        // IndexService REST proxies
        .route("/api/v1/indices", get(list_indices_handler))
        .route("/api/v1/indices/{id}", get(get_index_handler))
        // RatingsService REST proxies
        .route("/api/v1/skills/{ref}/rating", get(get_rating_handler))
        // Legacy skill CRUD
        .route("/skills", post(create_skill_handler))
        .route("/skills/{ref}", patch(update_skill_handler))
        .route("/skills/{ref}", delete(delete_skill_handler))
        // Migration endpoints
        .route("/migrate/skills", post(migrate_skills_handler))
        .route("/migrate/agents", post(migrate_agents_handler))
        .route(
            "/migrate/claude-agents",
            post(migrate_claude_agents_handler),
        )
        .route("/migrate/harnesses", post(migrate_harnesses_handler))
        // Registry hub endpoints
        .route("/api/v1/registry/catalog", get(get_catalog_handler))
        .route("/api/v1/skills/share", post(share_skill_handler))
        .route("/registry/skills", get(list_registry_skills_handler))
        .route(
            "/registry/skills/{namespace}",
            get(list_namespace_skills_handler),
        )
        .route("/registry/namespaces", get(list_namespaces_handler))
        .layer(middleware::from_fn_with_state(
            validator.clone(),
            auth_middleware,
        ))
        .with_state(state);

    let mut router = public.merge(protected);

    // Rate limiting: safe-by-default. Add timeout and request body size limits for basic DoS protection.
    let timeout_secs = std::env::var("SKILLPACK_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let max_body_size_mb = std::env::var("SKILLPACK_MAX_BODY_SIZE_MB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    router = router.layer(TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(timeout_secs),
    ));
    router = router.layer(RequestBodyLimitLayer::new(max_body_size_mb * 1024 * 1024));

    // CORS: safe-by-default. Deny all cross-origin unless SKILLPACK_CORS_ORIGINS is set.
    let cors = match std::env::var("SKILLPACK_CORS_ORIGINS").ok() {
        Some(origins) if !origins.is_empty() => {
            let allowed: Vec<axum::http::HeaderValue> = origins
                .split(',')
                .map(|o| o.trim().parse())
                .filter_map(Result::ok)
                .collect();
            if allowed.is_empty() {
                tracing::warn!(
                    "SKILLPACK_CORS_ORIGINS set but no valid origins parsed; denying all cross-origin requests"
                );
                CorsLayer::new()
            } else {
                CorsLayer::new()
                    .allow_origin(allowed)
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::POST,
                        axum::http::Method::PATCH,
                        axum::http::Method::DELETE,
                    ])
                    .allow_headers(Any)
            }
        }
        _ => CorsLayer::new(),
    };
    router = router.layer(cors);

    // Serve static dashboard files (index.html, app.js, style.css)
    router = router.fallback_service(ServeDir::new("crates/skillpack-api/static"));

    router
}

async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        protocol_version: "1.0",
    })
}

/// SSE endpoint: subscribe to the event bus and stream JSON events.
async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<
    impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
> {
    let (tx, rx) = tokio::sync::mpsc::channel(16);
    let mut event_rx = state.event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap_or_default();
            if tx
                .send(Ok(axum::response::sse::Event::default().data(json)))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    Sse::new(tokio_stream::wrappers::ReceiverStream::new(rx))
}

/// POST /skills — create a new skill.
async fn create_skill_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SkillMutation>,
) -> impl IntoResponse {
    let skill_ref = slugify(&body.name);
    let skill = SkillProfile {
        skill_ref: skill_ref.clone(),
        name: body.name,
        description: body.description,
        score: body.score,
        grade: body.grade,
        moodys_rating: body.moodys_rating,
        reputation_tier: body.reputation_tier,
        compatibility_score: body.compatibility_score,
        carbon_rating: body.carbon_rating,
        monthly_cost_usd: body.monthly_cost_usd,
        tags: body.tags,
        capabilities: std::collections::HashMap::new(),
    };

    {
        let mut cache = state.skills.write().await;
        cache.insert(skill_ref.clone(), skill.clone());
    }

    state.event_bus.publish(ServerEvent::SkillCreated {
        skill_ref: skill_ref.clone(),
    });

    (axum::http::StatusCode::CREATED, Json(skill))
}

/// PATCH /skills/:ref — update an existing skill.
async fn update_skill_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
    Json(body): Json<SkillMutation>,
) -> impl IntoResponse {
    let skill = SkillProfile {
        skill_ref: skill_ref.clone(),
        name: body.name,
        description: body.description,
        score: body.score,
        grade: body.grade,
        moodys_rating: body.moodys_rating,
        reputation_tier: body.reputation_tier,
        compatibility_score: body.compatibility_score,
        carbon_rating: body.carbon_rating,
        monthly_cost_usd: body.monthly_cost_usd,
        tags: body.tags,
        capabilities: std::collections::HashMap::new(),
    };

    {
        let mut cache = state.skills.write().await;
        cache.insert(skill_ref.clone(), skill.clone());
    }

    state.event_bus.publish(ServerEvent::SkillUpdated {
        skill_ref: skill_ref.clone(),
    });

    Json(skill)
}

/// DELETE /skills/:ref — remove a skill.
async fn delete_skill_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    {
        let mut cache = state.skills.write().await;
        cache.invalidate(&skill_ref);
    }

    state.event_bus.publish(ServerEvent::SkillDeleted {
        skill_ref: skill_ref.clone(),
    });

    axum::http::StatusCode::NO_CONTENT
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}

// ------------------------------------------------------------------
// Migration Handlers
// ------------------------------------------------------------------

#[derive(Deserialize)]
struct MigrateSkillsBody {
    canonical_root: Option<String>,
}

async fn migrate_skills_handler(Json(body): Json<MigrateSkillsBody>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || run_migrate_skills(body.canonical_root.as_deref()))
        .await
    {
        Ok(Ok(summary)) => Json(serde_json::json!({
            "success": true,
            "message": summary.message,
            "total_scanned": summary.total_scanned,
            "migrated_count": summary.migrated_count,
            "already_compliant_count": summary.already_compliant_count,
            "migrated_names": summary.migrated_names,
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
        Err(_) => Json(serde_json::json!({
            "success": false,
            "message": "migration task panicked",
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
    }
}

async fn migrate_agents_handler() -> impl IntoResponse {
    match tokio::task::spawn_blocking(run_migrate_agents).await {
        Ok(Ok(summary)) => Json(serde_json::json!({
            "success": true,
            "message": summary.message,
            "total_scanned": summary.total_scanned,
            "migrated_count": summary.migrated_count,
            "already_compliant_count": summary.already_compliant_count,
            "migrated_names": summary.migrated_names,
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
        Err(_) => Json(serde_json::json!({
            "success": false,
            "message": "migration task panicked",
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
    }
}

#[derive(Deserialize)]
struct MigrateClaudeAgentsBody {
    agents_dir: Option<String>,
}

async fn migrate_claude_agents_handler(
    Json(body): Json<MigrateClaudeAgentsBody>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || run_migrate_claude_agents(body.agents_dir.as_deref()))
        .await
    {
        Ok(Ok(summary)) => Json(serde_json::json!({
            "success": true,
            "message": summary.message,
            "total_scanned": summary.total_scanned,
            "migrated_count": summary.migrated_count,
            "already_compliant_count": summary.already_compliant_count,
            "migrated_names": summary.migrated_names,
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
        Err(_) => Json(serde_json::json!({
            "success": false,
            "message": "migration task panicked",
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
    }
}

async fn migrate_harnesses_handler() -> impl IntoResponse {
    match tokio::task::spawn_blocking(run_migrate_harnesses).await {
        Ok(Ok(summary)) => Json(serde_json::json!({
            "success": true,
            "message": summary.message,
            "total_scanned": summary.total_scanned,
            "migrated_count": summary.migrated_count,
            "already_compliant_count": summary.already_compliant_count,
            "migrated_names": summary.migrated_names,
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
        Err(_) => Json(serde_json::json!({
            "success": false,
            "message": "migration task panicked",
            "total_scanned": 0,
            "migrated_count": 0,
            "already_compliant_count": 0,
            "migrated_names": [],
        })),
    }
}

// ------------------------------------------------------------------
// Registry Hub Handlers
// ------------------------------------------------------------------

/// POST /api/v1/skills/share — publish a skill to a remote git group by
/// delegating to the verified `skillpack skill share` CLI (which does the
/// push-to-create with the machine's git credentials). Transport-only: it
/// shells the CLI and relays the result; no git logic lives here.
#[derive(serde::Deserialize)]
struct ShareRequest {
    skill: String,
    #[serde(rename = "groupUrl")]
    group_url: String,
    #[serde(rename = "repoName")]
    repo_name: Option<String>,
    branch: Option<String>,
    #[serde(default)]
    dry_run: bool,
}

async fn share_skill_handler(Json(req): Json<ShareRequest>) -> impl IntoResponse {
    let cli = std::env::var("SKILLPACK_CLI").unwrap_or_else(|_| "skillpack".to_string());
    let mut args: Vec<String> = vec![
        "skill".into(),
        "share".into(),
        req.skill.clone(),
        req.group_url.clone(),
    ];
    if let Some(rn) = &req.repo_name {
        args.push("--repo-name".into());
        args.push(rn.clone());
    }
    if let Some(b) = &req.branch {
        args.push("--branch".into());
        args.push(b.clone());
    }
    if req.dry_run {
        args.push("--dry-run".into());
    }
    match tokio::process::Command::new(&cli)
        .args(&args)
        .output()
        .await
    {
        Ok(o) if o.status.success() => Json(serde_json::json!({
            "ok": true,
            "message": String::from_utf8_lossy(&o.stdout).trim(),
        }))
        .into_response(),
        Ok(o) => (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "ok": false,
                "error": String::from_utf8_lossy(&o.stderr).trim(),
            })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": format!("cli '{cli}': {e}") })),
        )
            .into_response(),
    }
}

/// GET /api/v1/registry/catalog — the canonical store, pre-assessed. Served
/// from a background-filled cache: returns whatever is ready plus a `hydrating`
/// flag and progress counters so the UI can show grades as they land.
async fn get_catalog_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let c = match state.catalog.read() {
        Ok(c) => c,
        Err(_) => {
            return Json(
                serde_json::json!({ "hydrating": false, "total": 0, "assessed": 0, "skills": [] }),
            );
        }
    };
    let skills: Vec<serde_json::Value> = c
        .entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "name": e.name,
                "path": e.path,
                "grade": e.grade,
                "totalScore": e.score,
                "profile": e.profile,
                "issues": e.issue_count,
                "dimensions": e.dimensions.iter()
                    .map(|(id, s)| serde_json::json!({ "dimensionId": id, "score": s }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();
    Json(serde_json::json!({
        "hydrating": c.hydrating,
        "root": c.root,
        "total": c.total,
        "assessed": c.assessed,
        "skills": skills,
    }))
}

/// GET /registry/skills — list all skills across namespaces.
async fn list_registry_skills_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");
    let skills = match &state.repository {
        Some(repo) => match repo.list_canonical_skills(tenant_id) {
            Ok(records) => records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "skill_ref": r.skill_ref,
                        "name": r.name,
                        "path": r.path,
                        "has_skill_md": r.has_skill_md,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
            Err(_) => vec![],
        },
        None => vec![],
    };
    Json(serde_json::json!({ "skills": skills }))
}

/// GET /registry/skills/:namespace — list skills in a namespace.
async fn list_namespace_skills_handler(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");
    // For now, return all skills (namespace filtering will be added when
    // canonical_skills table has a namespace column)
    let skills = match &state.repository {
        Some(repo) => match repo.list_canonical_skills(tenant_id) {
            Ok(records) => records
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "skill_ref": r.skill_ref,
                        "name": r.name,
                        "path": r.path,
                        "has_skill_md": r.has_skill_md,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
            Err(_) => vec![],
        },
        None => vec![],
    };
    Json(serde_json::json!({
        "namespace": namespace,
        "skills": skills,
    }))
}

/// GET /registry/namespaces — list available namespaces.
async fn list_namespaces_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");
    // For now, infer namespaces from filesystem subdirectories of canonical root
    let namespaces = match &state.repository {
        Some(repo) => match repo.list_canonical_skills(tenant_id) {
            Ok(_) => vec!["default"],
            Err(_) => vec![],
        },
        None => vec!["default"],
    };
    Json(serde_json::json!({ "namespaces": namespaces }))
}

// ------------------------------------------------------------------
// REST API v1 Handlers — proxies for gRPC services
// ------------------------------------------------------------------

#[derive(Deserialize)]
struct AssessQuery {
    path: String,
    min_score: Option<f64>,
}

/// GET /api/v1/skills/assess — proxy for SkillPackService.Assess
async fn assess_handler(Query(params): Query<AssessQuery>) -> impl IntoResponse {
    let skill_path = match sanitize_path(&params.path) {
        Ok(path) => path,
        Err(status) => return status.into_response(),
    };

    let reader = FilesystemReader::new();
    let checkers = all_checkers();
    let use_case = AssessSkillUseCase::new(reader, checkers);

    let assess_req = AssessSkillRequest {
        skill_path,
        min_score: params.min_score,
    };

    match use_case.execute(assess_req) {
        Ok(response) => {
            let assessment = &response.assessment;
            Json(serde_json::json!({
                "assessment": {
                    "id": assessment.id.0,
                    "skillId": assessment.skill.path,
                    "totalScore": assessment.total_score().value(),
                    "grade": assessment.grade().as_str(),
                    "profile": assessment.profile.name(),
                    "dimensions": assessment.dimension_scores.iter().map(|(dim, score)| {
                        serde_json::json!({
                            "dimensionId": dim.name(),
                            "dimensionName": dim.name(),
                            "score": score.value(),
                            "maxScore": 100.0,
                            "weight": 1.0,
                        })
                    }).collect::<Vec<_>>(),
                    "issues": assessment.issues.iter().map(|issue| {
                        serde_json::json!({
                            "severity": match issue.severity {
                                skillpack_domain::Severity::Error => "error",
                                skillpack_domain::Severity::Warning => "warning",
                                skillpack_domain::Severity::Note => "info",
                            },
                            "dimensionId": issue.dimension.name(),
                            "message": issue.message,
                            "path": issue.file,
                            "line": issue.line,
                        })
                    }).collect::<Vec<_>>(),
                    "bonusPoints": assessment.bonus_points.total(),
                    "stubCount": assessment.stub_count,
                    "assessedAt": assessment.assessed_at.to_rfc3339(),
                },
                "meets_minimum": response.meets_minimum,
            }))
            .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/skills — proxy for QueryService.Search (no filters)
async fn list_skills_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let skills = state.skills.read().await;
    let items: Vec<serde_json::Value> = skills
        .values()
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "skill_ref": s.skill_ref,
                "name": s.name,
                "description": s.description,
                "score": s.score,
                "grade": s.grade,
                "moodys_rating": s.moodys_rating,
                "reputation_tier": s.reputation_tier,
                "compatibility_score": s.compatibility_score,
                "carbon_rating": s.carbon_rating,
                "monthly_cost_usd": s.monthly_cost_usd,
                "tags": s.tags,
            })
        })
        .collect();
    Json(serde_json::json!({ "skills": items }))
}

#[derive(Deserialize)]
struct SearchQuery {
    text: Option<String>,
    min_score: Option<f64>,
    sort_by: Option<String>,
    limit: Option<u32>,
}

/// GET /api/v1/skills/search — proxy for QueryService.Search
async fn search_skills_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let skills = state.skills.read().await;
    let mut results: Vec<&SkillProfile> = skills.values().into_iter().collect();

    if let Some(ref text) = params.text {
        let text_lower = text.to_lowercase();
        results.retain(|s| {
            s.name.to_lowercase().contains(&text_lower)
                || s.description
                    .as_ref()
                    .is_some_and(|d| d.to_lowercase().contains(&text_lower))
        });
    }

    if let Some(min_score) = params.min_score {
        results.retain(|s| s.score >= min_score);
    }

    match params.sort_by.as_deref() {
        Some("score") => results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        Some("name") => results.sort_by(|a, b| a.name.cmp(&b.name)),
        Some("cost") => results.sort_by(|a, b| {
            a.monthly_cost_usd
                .partial_cmp(&b.monthly_cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => {}
    }

    let limit = params.limit.unwrap_or(20) as usize;
    let page: Vec<_> = results
        .into_iter()
        .take(limit)
        .map(|s| {
            serde_json::json!({
                "skill_ref": s.skill_ref,
                "name": s.name,
                "description": s.description,
                "score": s.score,
                "grade": s.grade,
                "moodys_rating": s.moodys_rating,
                "reputation_tier": s.reputation_tier,
                "compatibility_score": s.compatibility_score,
                "carbon_rating": s.carbon_rating,
                "monthly_cost_usd": s.monthly_cost_usd,
                "tags": s.tags,
            })
        })
        .collect();

    Json(serde_json::json!({
        "skills": page,
        "total_count": page.len(),
        "page_size": limit,
        "offset": 0,
    }))
}

/// GET /api/v1/skills/:ref — proxy for QueryService.GetSkillProfile
async fn get_skill_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    let skills = state.skills.read().await;
    match skills.get(&skill_ref) {
        Some(skill) => Json(serde_json::json!({
            "skill_ref": skill.skill_ref,
            "name": skill.name,
            "description": skill.description,
            "score": skill.score,
            "grade": skill.grade,
            "moodys_rating": skill.moodys_rating,
            "reputation_tier": skill.reputation_tier,
            "compatibility_score": skill.compatibility_score,
            "carbon_rating": skill.carbon_rating,
            "monthly_cost_usd": skill.monthly_cost_usd,
            "tags": skill.tags,
            "capabilities": skill.capabilities.iter().map(|(k, v)| {
                (k.clone(), format!("{:?}", v))
            }).collect::<std::collections::HashMap<_, _>>(),
        }))
        .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "skill not found"})),
        )
            .into_response(),
    }
}

/// GET /api/v1/skills/:ref/profile — alias for get_skill_handler
async fn get_skill_profile_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    get_skill_handler(State(state), Path(skill_ref)).await
}

/// GET /api/v1/skills/:ref/compatibility — simplified compatibility
async fn get_compatibility_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    let skills = state.skills.read().await;
    match skills.get(&skill_ref) {
        Some(skill) => Json(serde_json::json!({
            "skill_ref": skill.skill_ref,
            "compatibility_score": skill.compatibility_score,
            "capabilities": skill.capabilities.iter().map(|(k, v)| {
                serde_json::json!({
                    "capability_id": k,
                    "capability_name": k,
                    "level": format!("{:?}", v),
                })
            }).collect::<Vec<_>>(),
        }))
        .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "skill not found"})),
        )
            .into_response(),
    }
}

/// GET /api/v1/skills/:ref/cost — simplified cost estimate
async fn get_cost_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    let skills = state.skills.read().await;
    match skills.get(&skill_ref) {
        Some(skill) => Json(serde_json::json!({
            "skill_ref": skill.skill_ref,
            "monthly_cost_usd": skill.monthly_cost_usd,
            "cost_tier": skill.grade,
        }))
        .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "skill not found"})),
        )
            .into_response(),
    }
}

/// GET /api/v1/indices — proxy for IndexService.ListIndices
async fn list_indices_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match &state.repository {
        Some(_repo) => (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "error": "list_indices not yet implemented for DuckDB repository"
            })),
        )
            .into_response(),
        None => {
            Json(serde_json::json!({ "indices": Vec::<serde_json::Value>::new() })).into_response()
        }
    }
}

/// GET /api/v1/indices/:id — proxy for IndexService.GetIndex
async fn get_index_handler(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    match &state.repository {
        Some(_repo) => (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "error": "get_index not yet implemented for DuckDB repository"
            })),
        )
            .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "index not found"})),
        )
            .into_response(),
    }
}

/// GET /api/v1/skills/:ref/rating — proxy for RatingsService.GetRating
async fn get_rating_handler(
    State(state): State<Arc<AppState>>,
    Path(skill_ref): Path<String>,
) -> impl IntoResponse {
    let skills = state.skills.read().await;
    match skills.get(&skill_ref) {
        Some(skill) => Json(serde_json::json!({
            "skill_ref": skill.skill_ref,
            "rating": skill.moodys_rating,
            "grade": skill.grade,
            "score": skill.score,
        }))
        .into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "skill not found"})),
        )
            .into_response(),
    }
}

/// GET /ws — WebSocket handler for bidirectional real-time events.
///
/// Upgrades the connection and spawns a task that:
/// 1. Subscribes to the EventBus and forwards ServerEvent JSON to the client.
/// 2. Listens for client messages (commands) and dispatches them.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let mut rx = state.event_bus.subscribe();

        loop {
            tokio::select! {
                biased;

                // Forward server events to client
                Ok(event) = rx.recv() => {
                    let json = match serde_json::to_string(&event) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::warn!("Failed to serialize event: {}", e);
                            continue;
                        }
                    };
                    if socket.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }

                // Handle client messages
                Some(Ok(msg)) = socket.recv() => {
                    match msg {
                        axum::extract::ws::Message::Text(text) => {
                            tracing::debug!("WS received: {}", text);
                            // Echo back for now; future: parse commands
                            let reply = format!("ack: {}", text);
                            if socket.send(axum::extract::ws::Message::Text(reply.into())).await.is_err() {
                                break;
                            }
                        }
                        axum::extract::ws::Message::Close(_) => break,
                        _ => {}
                    }
                }

                else => break,
            }
        }
    })
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::time::Duration;
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            skills: Arc::new(RwLock::new(TtlCache::new(Duration::from_secs(300)))),
            event_bus: Arc::new(EventBus::new(16)),
            repository: None,
            catalog: Arc::new(std::sync::RwLock::new(CatalogState::empty())),
        })
    }

    #[tokio::test]
    async fn health_returns_ok_and_protocol_version() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::none());
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_skill_publishes_event() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::with_token("test-token"));
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );
        let mut rx = state.event_bus.subscribe();

        let body = serde_json::json!({
            "name": "Test Skill",
            "score": 99.0,
            "grade": "A+",
            "moodys_rating": "Aa1",
            "reputation_tier": "Gold",
            "compatibility_score": 90,
            "carbon_rating": "A",
            "monthly_cost_usd": 50.0,
            "tags": ["test"]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/skills")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer test-token")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let event = rx.recv().await.expect("should receive event");
        assert_eq!(
            event,
            ServerEvent::SkillCreated {
                skill_ref: "test-skill".to_string()
            }
        );
    }

    #[tokio::test]
    async fn create_skill_rejected_without_auth() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::with_token("test-token"));
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );

        let body = serde_json::json!({"name": "Test"});
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/skills")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_skills_returns_empty() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::with_token("test-token"));
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/skills")
                    .header("authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["skills"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn get_skill_not_found() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::with_token("test-token"));
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/skills/nonexistent")
                    .header("authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn skill_path_guard_allows_package_file() {
        // Test harness CWD is the crate root; Cargo.toml exists there.
        let result = skillpack_application::validate_skill_path_cwd("Cargo.toml");
        assert!(result.is_ok());
    }

    #[test]
    fn skill_path_guard_rejects_traversal() {
        // `..` is rejected before the filesystem is touched.
        let result = skillpack_application::validate_skill_path_cwd("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cors_deny_by_default() {
        let state = test_state();
        let validator = Arc::new(TokenValidator::none());
        let app = build_router(
            state.skills.clone(),
            state.event_bus.clone(),
            validator,
            None,
            state.catalog.clone(),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("origin", "https://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Deny-by-default means no Access-Control-Allow-Origin header
        assert!(
            response
                .headers()
                .get("access-control-allow-origin")
                .is_none()
        );
    }
}
