# Client Feature Matrix

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.1.0                                                                |
| Scope         | Per-client operation support, viewport constraints, interaction model |

---

## 1. Operation Matrix

**Legend**: M = MUST, S = SHOULD, O = MAY, N = MUST NOT

| # | Operation | CLI (C-01) | VS Code (C-02) | Web (C-03) | macOS (C-04–06) | CNI (C-08) | MCP (C-09) |
|---|-----------|:----------:|:--------------:|:----------:|:---------------:|:----------:|:----------:|
| 1 | Assess skill | M | M | M | S | M | N |
| 2 | Grade skill | M | M | M | S | N | N |
| 3 | Create skill | M | M | O | N | N | N |
| 4 | Edit skill | N | M | O | N | N | N |
| 5 | View skill detail | M | M | M | N | N | S |
| 6 | Publish to registry | M | M | N | S | M | N |
| 7 | Install from registry | M | M | M | M | M | S |
| 8 | Sync to agents | M | S | N | M | M | N |
| 9 | View sync status | M | M | M | M | M | S |
| 10 | Lifecycle promote | M | S | S | S | M | N |
| 11 | Lifecycle demote | M | S | S | S | M | N |
| 12 | Lifecycle supersede | M | S | S | S | M | N |
| 13 | IP boundary check | M | M | M | M | M | N |
| 14 | Auto-assess on save | N | M | N | N | N | N |
| 15 | AI-assisted authoring | N | M | O | N | N | N |
| 16 | Registry search | M | M | M | N | M | N |
| 17 | Registry browse | M | M | M | N | N | N |
| 18 | Skill history / audit | M | S | M | N | N | N |
| 19 | Evaluation suite run | M | S | N | N | M | N |
| 20 | Generate lock file | M | S | N | N | M | N |
| 21 | OCI login / logout | M | M | N | N | M | N |
| 22 | View reports (JSON/SARIF/MD) | M | M | M | N | N | N |
| 23 | Settings / preferences | M | M | M | M | N | N |
| 24 | Export skill | M | S | O | N | N | N |
| 25 | MCP resource read | N | N | N | N | N | M |
| 26 | MCP tool invoke | N | N | N | N | N | M |

---

## 2. Rationale by Operation

### 2.1 Assess Skill (#1)

- **CLI**: Core operation; primary use case
- **VS Code**: Core operation; inline diagnostics depend on assessment
- **Web**: Core operation; dashboard shows assessment results
- **macOS**: SHOULD; menu bar could trigger assessment but not primary UX
- **CNI**: MUST; CI gate requires assessment before promotion
- **MCP**: MUST NOT; assessment is not an MCP tool/resource operation

### 2.2 Create Skill (#3)

- **CLI**: MUST; `skillpack init` and `skillpack wizard`
- **VS Code**: MUST; wizard command with form input
- **Web**: MAY; web-based creation is useful but not essential
- **macOS/CNI/MCP**: MUST NOT; creation requires file system interaction beyond menu bar scope

### 2.3 Edit Skill (#4)

- **CLI**: MUST NOT; CLI is not an editor
- **VS Code**: MUST; inline editing with validation
- **Web**: MAY; Monaco-based editor possible but complex
- **macOS/CNI/MCP**: MUST NOT; no editing surface

### 2.4 Publish to Registry (#6)

- **CLI**: MUST; `skillpack publish`
- **VS Code**: MUST; publish command with registry selection
- **Web**: MUST NOT; web client should not have registry push credentials
- **macOS**: SHOULD; menu bar could offer quick publish
- **CNI**: MUST; CI pipeline publishes after quality gate
- **MCP**: MUST NOT; not an MCP concern

### 2.5 Install from Registry (#7)

- **CLI**: MUST; `skillpack install`
- **VS Code**: MUST; registry browser with install button
- **Web**: MUST; web-based installation for non-developer users
- **macOS**: MUST; menu bar install from registry
- **CNI**: MUST; CI/CD installs dependencies
- **MCP**: SHOULD; MCP could expose install as a tool

### 2.6 Sync to Agents (#8)

- **CLI**: MUST; `skillpack store sync`
- **VS Code**: SHOULD; sync command available but not auto
- **Web**: MUST NOT; web client has no agent file system access
- **macOS**: MUST; core daemon responsibility
- **CNI**: MUST; container deployment syncs skills to agent volumes
- **MCP**: MUST NOT; sync is infrastructure, not AI interaction

### 2.7 Lifecycle Operations (#10–12)

- **CLI**: MUST; `skillpack promote/demote`
- **VS Code**: SHOULD; context menu actions
- **Web**: SHOULD; admin dashboard operations
- **macOS**: SHOULD; menu bar quick actions
- **CNI**: MUST; CI pipeline promotes after validation
- **MCP**: MUST NOT; lifecycle is infrastructure, not AI interaction

### 2.8 IP Boundary Check (#13)

- **All**: MUST; IPGuard is a cross-cutting concern enforced everywhere

### 2.9 Auto-Assess on Save (#14)

- **VS Code**: MUST; core IDE integration value
- **All others**: MUST NOT; no save event hook available

### 2.10 AI-Assisted Authoring (#15)

- **VS Code**: MUST; AI command palette, inline suggestions
- **Web**: MAY; chat-based assistant possible
- **All others**: MUST NOT; no AI integration surface

---

## 3. Viewport & Interaction Constraints

### 3.1 CLI (C-01)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | 80-char terminal minimum; auto-detect actual width |
| Primary input | Keyboard (flags, subcommands, interactive prompts) |
| Output | stdout (structured data), stderr (errors/warnings), colored if TTY |
| Paging | Auto-detect pipe vs TTY; use `less` if TTY and output > terminal height |
| Progress | Spinner for long ops; progress bar for assess stream |
| Tables | Auto-fit to terminal width; truncate with `…` if needed |

### 3.2 VS Code Extension (C-02)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | Sidebar 300px wide; panel bottom; editor full width |
| Primary input | Keyboard (commands) + mouse (tree clicks, buttons) |
| Output | Tree views, webview panels, status bar, notifications |
| Progress | Status bar spinner, tree item loading, notification with progress |
| Tables | Virtual scrolling for large lists; sortable columns |
| Context | `workspaceHasSkill` context key gates command visibility |

### 3.3 Web Dashboard (C-03)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | Responsive: 320px (mobile) to 4K (desktop) |
| Primary input | Mouse + touch (mobile) + keyboard |
| Output | React components, charts (Recharts), cards, tables |
| Progress | Skeleton loaders, progress bars, toast notifications |
| Tables | Pagination + virtual scroll for >100 rows |
| Accessibility | WCAG 2.1 AA minimum; screen reader tested |

### 3.4 macOS Menu Bar (C-04)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | Fixed 320px window; not resizable |
| Primary input | Mouse (menu clicks, toggles) |
| Output | SwiftUI views, system icons, color-coded status |
| Progress | Rotating sync icon, progress text |
| Tables | Scroll view for lists; max height 200px |
| Accessibility | VoiceOver labels on all controls |

### 3.5 macOS Finder (C-06)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | Badge overlays (16×16 px), context menu |
| Primary input | Mouse (right-click) |
| Output | Badge (green/red), context menu items |
| Progress | Badge color change only |
| Tables | N/A |
| Accessibility | Badge must not be color-only (shape/icon distinction) |

### 3.6 CNI (C-08)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | None (headless) |
| Primary input | stdin JSON, env vars, command-line flags |
| Output | stdout (JSON result), stderr (errors), exit code |
| Progress | JSON stream if `--stream` flag; otherwise blocking |
| Tables | N/A |
| Accessibility | N/A |

### 3.7 MCP Server (C-09)

| Attribute | Constraint |
|-----------|-----------|
| Viewport | N/A (protocol layer) |
| Primary input | JSON-RPC over stdio or SSE |
| Output | JSON-RPC responses, MCP resource content |
| Progress | Async notifications via JSON-RPC |
| Tables | N/A |
| Accessibility | N/A |

---

## 4. Notification Surface by Client

| Event | CLI | VS Code | Web | macOS | CNI |
|-------|-----|---------|-----|-------|-----|
| Assessment complete | stdout | Status bar + notification | Toast | Status text | Stdout JSON |
| Sync complete | stdout | Notification | Toast | Status text | Stdout JSON |
| IP violation | stderr + exit 77 | Error notification | Inline banner | Alert sheet | Stderr JSON + exit 77 |
| Registry update | — | Tree refresh | Toast | — | — |
| Lifecycle event | — | — | — | — | Stdout JSON |
| Connection lost | stderr | Status bar warning | Offline banner | Icon change | Stderr |
| Connection restored | stderr | Status bar clear | Online toast | Icon change | Stderr |

---

## 5. Offline Behavior Summary

| Client | Read Strategy | Write Strategy | Reconnection |
|--------|--------------|----------------|--------------|
| CLI | No cache (query every time) | Fail-fast or retry | CLI-level retry with backoff |
| VS Code | Memory cache, stale badge | Queue + optimistic | Auto-reconnect, sync on restore |
| Web | Service Worker + IndexedDB | Queue in IndexedDB | Auto-reconnect, background sync |
| macOS | Daemon state (last known) | Daemon queues + retries | Daemon handles automatically |
| CNI | N/A | Fail-fast | No retry; CI handles |
| MCP | Cache last resources | Fail-fast | MCP client reconnects |
