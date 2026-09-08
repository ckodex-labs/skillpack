//! Filesystem watcher for the canonical store
//!
//! Watches `~/Skills/shared` for changes and triggers debounced auto-sync
//! to keep agent directories in sync with the canonical store.
//!
//! Uses the `notify` crate for cross-platform filesystem watching.

use notify::Watcher;
use skillpack_application::SyncCanonicalStoreUseCase;
use skillpack_domain::CanonicalStore;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Debounce window: rapid filesystem changes are coalesced into a single sync.
const DEBOUNCE_MS: u64 = 2000;

/// Start watching the canonical store directory and trigger auto-sync on changes.
///
/// Returns a channel receiver that emits `()` whenever a sync should be triggered.
/// The caller (usually `server.rs`) should spawn a task that awaits these events
/// and runs `SyncCanonicalStoreUseCase`.
pub fn start_canonical_watcher(canonical_root: PathBuf) -> anyhow::Result<mpsc::Receiver<()>> {
    if !canonical_root.exists() {
        warn!(
            "Canonical store does not exist yet, skipping watcher: {}",
            canonical_root.display()
        );
        // Return a receiver that will never fire — the caller can still await it
        let (_, rx) = mpsc::channel(1);
        return Ok(rx);
    }

    let (tx, rx) = mpsc::channel(1);

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Filter out access events and metadata-only changes
                    if event.kind.is_access() {
                        return;
                    }
                    // Only care about create, modify, remove
                    let is_relevant =
                        event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove();
                    if !is_relevant {
                        return;
                    }
                    if tx.try_send(()).is_err() {
                        // Channel full — debounce is already pending
                    }
                }
                Err(e) => {
                    error!("Filesystem watcher error: {}", e);
                }
            }
        })?;

    watcher.watch(&canonical_root, notify::RecursiveMode::Recursive)?;

    // Keep the watcher alive by moving it into a static
    Box::leak(Box::new(watcher));

    info!(
        "Watching canonical store for changes: {}",
        canonical_root.display()
    );
    Ok(rx)
}

/// Run the auto-sync loop: debounce watcher events and trigger sync.
///
/// This should be spawned as a tokio background task.
pub async fn run_auto_sync_loop(
    mut event_rx: mpsc::Receiver<()>,
    canonical_root: PathBuf,
    event_bus: std::sync::Arc<crate::cache::EventBus>,
) {
    let use_case = SyncCanonicalStoreUseCase::new();

    while event_rx.recv().await.is_some() {
        // Debounce: wait for a quiet period before syncing
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(DEBOUNCE_MS)) => break,
                _ = event_rx.recv() => continue, // reset debounce timer
            }
        }

        let store = CanonicalStore {
            root: canonical_root.clone(),
            dry_run: false,
            do_index: true,
            only_agent: None,
            namespace: None,
            oci_push: false,
            oci_registry: None,
            oci_repository: None,
        };

        event_bus.publish(crate::cache::ServerEvent::SyncStarted);
        info!("Auto-sync triggered by filesystem change");

        match use_case.execute(&store) {
            Ok(result) => {
                let total_agents = result.agent_results.len() as i32;
                for (idx, agent_result) in result.agent_results.iter().enumerate() {
                    event_bus.publish(crate::cache::ServerEvent::SyncProgress {
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
                info!(
                    "Auto-sync complete: {} skills across {} agents",
                    result.skills_processed,
                    result.agent_results.len()
                );
                event_bus.publish(crate::cache::ServerEvent::SyncCompleted {
                    skills_processed: result.skills_processed as i32,
                });
            }
            Err(e) => {
                error!("Auto-sync failed: {}", e);
            }
        }
    }

    info!("Auto-sync loop exiting");
}
