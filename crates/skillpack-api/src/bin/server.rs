//! SkillPack gRPC Server Binary
//!
//! Production server with all gRPC services:
//! - SkillPackService (assessment, grading, reporting)
//! - IndexService (skills index management)
//! - RatingsService (credit ratings)
//! - QueryService (search and compatibility)

use skillpack_adapters::persistence::DuckDbRepository;
use skillpack_api::{
    CanonicalStoreServiceImpl, IndexServiceImpl, QueryServiceImpl, RatingsServiceImpl,
    SkillPackServer,
    auth::{TokenValidator, grpc_interceptor},
    cache::{EventBus, TtlCache},
    http_server,
    proto::{
        canonical_store_service_server::CanonicalStoreServiceServer,
        index_service_server::IndexServiceServer, query_service_server::QueryServiceServer,
        ratings_service_server::RatingsServiceServer,
        skill_pack_service_server::SkillPackServiceServer,
    },
    telemetry::{init_telemetry, shutdown_telemetry},
    watcher,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing (OTLP enabled via OTEL_EXPORTER_OTLP_ENDPOINT env var)
    init_telemetry("skillpack-server", None)?;

    let port = std::env::var("SKILLPACK_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(50051);
    let bind_all = std::env::var("SKILLPACK_BIND_ALL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let host = if bind_all { "0.0.0.0" } else { "127.0.0.1" };
    let addr = format!("{}:{}", host, port).parse()?;

    info!("Initializing SkillPack gRPC Server...");

    // Initialize persistence: file-backed via env var, in-memory fallback
    let repository = Arc::new(match std::env::var("SKILLPACK_DB_PATH").ok() {
        Some(path) if !path.is_empty() => {
            info!("Opening file-backed DuckDB at {}", path);
            DuckDbRepository::open(&path)?
        }
        _ => {
            info!("Using in-memory DuckDB");
            DuckDbRepository::in_memory()?
        }
    });

    // CLIENT-SPEC.md §4.3: shared event bus for cache invalidation + streaming
    let event_bus = Arc::new(EventBus::new(256));

    // Shared in-memory cache (used by QueryService + HTTP mutation endpoints)
    let skills_cache = Arc::new(RwLock::new(TtlCache::new(Duration::from_secs(300))));

    // Auth: read SKILLPACK_API_TOKEN env var (disabled if unset)
    let validator = Arc::new(TokenValidator::from_env());
    if validator.is_enabled() {
        info!("Bearer token auth enabled");
    } else if bind_all {
        eprintln!("\n┌──────────────────────────────────────────────────────────────────────┐");
        eprintln!("│  FATAL: SKILLPACK_BIND_ALL=1 but SKILLPACK_API_TOKEN is unset.       │");
        eprintln!("│  Refusing to bind 0.0.0.0 without bearer-token auth.               │");
        eprintln!("└──────────────────────────────────────────────────────────────────────┘\n");
        tracing::error!("refusing to start: SKILLPACK_BIND_ALL=1 requires SKILLPACK_API_TOKEN");
        return Err("SKILLPACK_BIND_ALL=1 requires SKILLPACK_API_TOKEN".into());
    } else {
        eprintln!("\n┌──────────────────────────────────────────────────────────────────────┐");
        eprintln!("│  WARNING: Bearer token auth is DISABLED                              │");
        eprintln!("│  Mutation/migration endpoints are unprotected.                     │");
        eprintln!("│  Set SKILLPACK_API_TOKEN before exposing this server to any network. │");
        eprintln!("└──────────────────────────────────────────────────────────────────────┘\n");
        tracing::warn!(
            "Bearer token auth is DISABLED. Mutation/migration endpoints are unprotected."
        );
        tracing::warn!("Set SKILLPACK_API_TOKEN before exposing this server to any network.");
    }

    // Create services
    let skillpack_service = SkillPackServer::new();
    let index_service = IndexServiceImpl::new(repository.clone());
    let ratings_service = RatingsServiceImpl::new();
    let query_service = QueryServiceImpl::new(event_bus.clone(), skills_cache.clone());
    let canonical_service =
        CanonicalStoreServiceImpl::with_repository("", event_bus.clone(), repository.clone());

    // Spawn filesystem watcher for auto-sync
    let canonical_root = canonical_service.canonical_root();

    // Background: assess the canonical store once so the registry catalog
    // endpoint serves real grades without the client fanning out per-skill.
    let catalog = Arc::new(std::sync::RwLock::new(http_server::CatalogState::empty()));
    {
        let catalog = catalog.clone();
        let root = canonical_root.clone();
        info!("Hydrating registry catalog from {}", root.display());
        tokio::task::spawn_blocking(move || {
            http_server::hydrate_catalog(catalog, root);
            info!("Registry catalog hydration complete");
        });
    }

    let event_bus_clone = event_bus.clone();
    let watcher_task = tokio::spawn(async move {
        match watcher::start_canonical_watcher(canonical_root.clone()) {
            Ok(rx) => {
                watcher::run_auto_sync_loop(rx, canonical_root, event_bus_clone).await;
            }
            Err(e) => {
                tracing::error!("Failed to start canonical store watcher: {}", e);
            }
        }
    });

    // Seed sample data
    query_service.seed_sample_data().await;

    // Spawn HTTP server (SSE + REST mutations)
    let http_port = std::env::var("SKILLPACK_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(50052);
    let http_host = if bind_all { "0.0.0.0" } else { "127.0.0.1" };
    let http_addr = format!("{}:{}", http_host, http_port);
    let http_app = http_server::build_router(
        skills_cache.clone(),
        event_bus.clone(),
        validator.clone(),
        Some(repository.clone()),
        catalog.clone(),
    );
    let http_addr_clone = http_addr.clone();
    let http_server = tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&http_addr_clone).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind HTTP listener on {}: {}", http_addr_clone, e);
                return;
            }
        };
        info!("HTTP server listening on {}", http_addr_clone);
        if let Err(e) = axum::serve(listener, http_app).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    info!("Starting gRPC server on {}", addr);
    info!("HTTP server on {}", http_addr);
    info!("Services:");
    info!("  - skillpack.v1.SkillPackService");
    info!("  - skillpack.v1.IndexService");
    info!("  - skillpack.v1.RatingsService");
    info!("  - skillpack.v1.QueryService");
    info!("  - skillpack.v1.CanonicalStoreService");
    info!("  - HTTP /health + /events (SSE) + /skills (REST)");

    let grpc_interceptor = grpc_interceptor(validator.clone());

    let grpc_server = Server::builder()
        .add_service(SkillPackServiceServer::with_interceptor(
            skillpack_service,
            grpc_interceptor.clone(),
        ))
        .add_service(IndexServiceServer::with_interceptor(
            index_service,
            grpc_interceptor.clone(),
        ))
        .add_service(RatingsServiceServer::with_interceptor(
            ratings_service,
            grpc_interceptor.clone(),
        ))
        .add_service(QueryServiceServer::with_interceptor(
            query_service,
            grpc_interceptor.clone(),
        ))
        .add_service(CanonicalStoreServiceServer::with_interceptor(
            canonical_service,
            grpc_interceptor,
        ))
        .serve(addr);

    // Graceful shutdown: wait for SIGINT / SIGTERM
    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        result = http_server => {
            if let Err(e) = result {
                tracing::error!("HTTP server error: {}", e);
            }
        }
        result = watcher_task => {
            if let Err(e) = result {
                tracing::error!("Watcher task error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal, stopping servers gracefully...");
        }
    }

    shutdown_telemetry();
    info!("SkillPack server shut down complete.");
    Ok(())
}
