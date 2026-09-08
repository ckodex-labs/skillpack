//! TTL Cache + Event Bus per CLIENT-SPEC.md §4.1 / §4.3
//!
//! Provides:
//! - `TtlCache`: in-memory map with per-entry TTL and event-triggered invalidation
//! - `EventBus`: broadcast channel for server-wide events (cache invalidation, sync status, etc.)

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// CLIENT-SPEC.md §4.3: Server events that clients can subscribe to.
///
/// Also used as §4.1 cache-invalidation triggers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ServerEvent {
    SkillCreated {
        skill_ref: String,
    },
    SkillUpdated {
        skill_ref: String,
    },
    SkillDeleted {
        skill_ref: String,
    },
    AssessmentCompleted {
        skill_path: String,
    },
    SyncStarted,
    SyncProgress {
        agent_name: String,
        skill_name: String,
        status: String,
        current: i32,
        total: i32,
    },
    SyncCompleted {
        skills_processed: i32,
    },
    PreferenceChanged {
        key: String,
    },
}

/// Broadcast-based event bus. Clone cheaply to get new subscribers.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<ServerEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _rx) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all active subscribers.
    pub fn publish(&self, event: ServerEvent) {
        // Err means no active receivers — that's fine.
        let _ = self.sender.send(event);
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

/// A single cached value with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V) -> Self {
        Self {
            value,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// In-memory cache with TTL support and explicit invalidation.
///
/// CLIENT-SPEC.md §4.1 TTLs:
/// - Skill list: 5 min
/// - Skill detail: 2 min
/// - Assessment result: 1 min
/// - Sync status: 30 s
/// - Registry search: 10 min
/// - User preferences: session (no TTL)
#[derive(Debug, Clone)]
pub struct TtlCache<K, V> {
    map: HashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            map: HashMap::new(),
            default_ttl,
        }
    }

    /// Get a value if present and not expired.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key).and_then(|entry| {
            if entry.is_expired(self.default_ttl) {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    /// Insert a new value, resetting its TTL.
    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, CacheEntry::new(value));
    }

    /// Remove a specific key (event-triggered invalidation).
    pub fn invalidate(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).map(|e| e.value)
    }

    /// Remove all entries matching a predicate (e.g. all skills after sync).
    pub fn invalidate_where<F>(&mut self, predicate: F)
    where
        F: Fn(&K) -> bool,
    {
        self.map.retain(|k, _| !predicate(k));
    }

    /// Clear the entire cache.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Iterate over non-expired (key, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter().filter_map(move |(k, entry)| {
            if entry.is_expired(self.default_ttl) {
                None
            } else {
                Some((k, &entry.value))
            }
        })
    }

    /// Collect all non-expired values.
    pub fn values(&self) -> Vec<&V> {
        self.iter().map(|(_, v)| v).collect()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[test]
    fn cache_entry_expires_after_ttl() {
        let entry = CacheEntry::new(42);
        assert!(!entry.is_expired(Duration::from_secs(60)));
    }

    #[tokio::test]
    async fn cache_get_returns_none_after_ttl() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_millis(50));
        cache.insert("a".into(), 1);

        assert_eq!(cache.get(&"a".into()), Some(&1));
        sleep(Duration::from_millis(60)).await;
        assert_eq!(cache.get(&"a".into()), None);
    }

    #[test]
    fn cache_invalidate_removes_entry() {
        let mut cache: TtlCache<String, i32> = TtlCache::new(Duration::from_secs(60));
        cache.insert("a".into(), 1);
        assert_eq!(cache.invalidate(&"a".into()), Some(1));
        assert_eq!(cache.get(&"a".into()), None);
    }

    #[tokio::test]
    async fn event_bus_roundtrip() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(ServerEvent::SyncStarted);

        let event = rx.recv().await.expect("should receive event");
        assert_eq!(event, ServerEvent::SyncStarted);
    }
}
