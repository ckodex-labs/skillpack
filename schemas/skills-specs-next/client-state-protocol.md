# Client State Management & Reactivity Protocol

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.1.0                                                                |
| Scope         | Cache semantics, event delivery, optimistic updates, offline behavior |

---

## 1. Design Goals

1. **Eventual consistency**: All clients converge to the same server state
2. **Responsive UI**: Local cache serves reads; sync happens in background
3. **Graceful degradation**: Offline clients continue with cached data
4. **Predictable invalidation**: TTL + event-driven cache invalidation
5. **No lost mutations**: Optimistic updates with rollback on failure

---

## 2. Cache Architecture

### 2.1 Cache Tiers

```
┌─────────────────────────────────────────────────────────────┐
│                     CLIENT CACHE                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Memory      │  │  Persistent  │  │  Server      │    │
│  │  (hot)       │  │  (warm)      │  │  (source)    │    │
│  │  < 1 min     │  │  < 1 hour    │  │  infinite    │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Per-Data TTL & Invalidation

| Data | Memory TTL | Persistent | Invalidate On |
|------|-----------|------------|---------------|
| Skill list | 5 min | 1 hour | `skill_created`, `skill_deleted`, `skill_updated` |
| Skill detail | 2 min | 30 min | `skill_updated` for that skill |
| Assessment result | 1 min | 15 min | `assessment_completed` for that skill |
| Sync status | 30s | 5 min | `sync_started`, `sync_completed`, `sync_failed` |
| Registry search | 10 min | 24 hour | Manual refresh or registry change |
| User preferences | Session | 1 year | `preference_changed` |
| Lifecycle events | 1 min | 24 hour | New event arrival |

### 2.3 Cache Key Format

```
skills:list:{query_hash}     // Skill list with filters
skills:detail:{skill_id}     // Full skill detail
assessments:{skill_id}:latest // Latest assessment
sync:status                  // Daemon sync status
registry:search:{query_hash} // Registry search results
prefs:{user_id}              // User preferences
lifecycle:recent             // Recent lifecycle events
```

---

## 3. Event Delivery

### 3.1 Event Types

```protobuf
enum EventType {
  SKILL_CREATED = 0;
  SKILL_UPDATED = 1;
  SKILL_DELETED = 2;
  ASSESSMENT_STARTED = 3;
  ASSESSMENT_COMPLETED = 4;
  SYNC_STARTED = 5;
  SYNC_PROGRESS = 6;
  SYNC_COMPLETED = 7;
  SYNC_FAILED = 8;
  LIFECYCLE_EVENT = 9;
  PREFERENCE_CHANGED = 10;
}

message ServerEvent {
  string event_id = 1;      // UUID
  EventType type = 2;
  string timestamp = 3;     // RFC 3339
  string skill_id = 4;      // Optional: affected skill
  google.protobuf.Any payload = 5;
}
```

### 3.2 Delivery Mechanisms (Preference Order)

| Priority | Mechanism | Clients | Fallback |
|----------|-----------|---------|----------|
| 1 | gRPC streaming (`WatchEvents`) | VS Code, Dashboard, macOS Daemon | Polling |
| 2 | Server-Sent Events (`GET /events`) | Web Dashboard, CNI | Polling |
| 3 | Polling (`GET /status` every 30s) | All | — |

### 3.3 SSE Endpoint

```
GET /api/v1/events
  Accept: text/event-stream
  Authorization: Bearer <token>

Response:
  event: assessment_completed
  id: {event_id}
  data: { "skillId": "...", "grade": "A", "totalScore": 95 }

  event: skill_updated
  id: {event_id}
  data: { "skillId": "...", "fields": ["manifest", "assessment"] }
```

### 3.4 Reconnection

- **gRPC**: Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- **SSE**: Browser auto-reconnects with `Last-Event-ID`; client replays from last ID
- **Polling**: Continues at fixed interval regardless of connection state

---

## 4. Optimistic Updates

### 4.1 Mutation Flow

```
1. USER ACTION (e.g., "promote skill")
   │
2. OPTIMISTIC APPLY
   │  → Update local cache immediately
   │  → Render updated UI
   │
3. REQUEST TO SERVER
   │  → gRPC/REST mutation call
   │
4a. SUCCESS
   │  → Server response reconciles with cache
   │  → Usually no-op (already optimistically applied)
   │  → Emit success event
   │
4b. FAILURE
   │  → Rollback local cache to pre-mutation state
   │  → Show error with retry option
   │  → Re-render UI with original state
```

### 4.2 Rollback Rules

| Mutation Type | Rollback Strategy | Conflict Resolution |
|---------------|-------------------|-------------------|
| Promote skill | Revert tier to previous value | Server-wins (re-fetch from server) |
| Demote skill | Same as promote | Server-wins |
| Update preferences | Revert to last known value | Last-write-wins (merge if possible) |
| Install skill | Remove from installed list | Server-wins |
| Delete skill | Restore to list | Server-wins |
| Assessment trigger | No optimistic update (async) | N/A |

### 4.3 Pending State UX

While mutation is in flight:
- **CLI**: Spinner + "promoting..." message
- **VS Code**: Status bar item + tree item loading spinner
- **Web**: Button disabled + inline loading indicator
- **macOS**: Menu item grayed + activity indicator
- **CNI**: No UX; block until complete or timeout

---

## 5. Offline Behavior

### 5.1 Offline Detection

```
Connection state:
  online  → health check succeeds within 2s
  degraded → health check succeeds but > 2s (show warning)
  offline → health check fails for 2 consecutive attempts
```

### 5.2 Per-Client Offline Strategy

| Client | Reads | Mutations | Reconnection |
|--------|-------|-----------|--------------|
| **CLI** | Cache not applicable (always queries) | Queue in memory; fail if critical | Retry with backoff; exit on persistent failure |
| **VS Code** | Serve from memory cache; show "stale" badge | Queue; show pending indicator | Auto-reconnect; sync on reconnect |
| **Web** | Serve from Service Worker cache; show offline banner | Queue in IndexedDB; disable buttons | Auto-reconnect; toast on restore |
| **macOS** | Serve from last daemon state; show offline icon | Queue via daemon; daemon retries | Daemon handles reconnection; UI reflects state |
| **CNI** | N/A (headless) | Fail-fast with exit code 69 (unavailable) | No retry; CI pipeline handles failure |
| **MCP** | Serve from cache | Fail-fast | MCP client handles reconnection |

### 5.3 Mutation Queue

Queued mutations are persisted (where applicable):
- **VS Code**: Extension global state (limited durability)
- **Web**: IndexedDB (durable across sessions)
- **macOS**: Daemon memory (ephemeral; daemon restart clears)
- **CLI**: Not queued; immediate fail or retry

Queue ordering: FIFO. On reconnect, flush queue sequentially.

---

## 6. Synchronization Guarantees

### 6.1 Monotonic Reads

If a client observes state S at time T, it will never observe state S' < S at time T' > T.

Enforced by: server timestamps on all responses; client rejects stale responses.

### 6.2 Read-Your-Writes

After a successful mutation, a subsequent read by the same client MUST reflect the mutation.

Enforced by: optimistic update + server confirmation reconciliation.

### 6.3 Eventual Consistency

All clients converge to the same server state within the TTL window of the data type.

Maximum inconsistency window: 30 seconds (sync status TTL).

---

## 7. State Diagram: Connection Lifecycle

```
          ┌──────────┐
          │  CLOSED  │
          └────┬─────┘
               │ connect()
               ▼
          ┌──────────┐
          │ CONNECTING │◄── retry on failure
          └────┬─────┘
               │ health check OK
               ▼
          ┌──────────┐
          │  ONLINE  │◄────────────────┐
          └────┬─────┘                  │
               │ health check degraded   │
               ▼                        │ health check recovers
          ┌──────────┐                  │
          │ DEGRADED │──────────────────┘
          └────┬─────┘
               │ health check fails x2
               ▼
          ┌──────────┐
          │  OFFLINE │◄── retry with backoff
          └──────────┘
```
