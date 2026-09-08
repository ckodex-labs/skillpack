# Client Error Taxonomy

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.1.0                                                                |
| Scope         | Structured error codes, severity levels, per-client rendering hints |

---

## 1. Error Structure

Every error returned by any SkillPack surface conforms to this structure:

```json
{
  "code": "SKILLPACK_IP_VIOLATION",
  "category": "security",
  "severity": "error",
  "message": "Skill name contains restricted pattern: 'thales'",
  "details": {
    "violations": ["thales"],
    "suggestion": "Rename skill or use --force to override"
  },
  "clientHint": {
    "cli": "Print red text; suggest --force flag",
    "vscode": "Show error notification with 'View Details' button; offer --force in message actions",
    "web": "Display inline red banner with dismiss button and 'Override' link",
    "macos": "Show NSAlert with informative text; add 'Override' button if forceable",
    "cni": "Emit JSON to stderr; exit 78 (configuration error); include --force in hint"
  },
  "traceId": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "docsUrl": "https://docs.ckodex.org/errors/SKILLPACK_IP_VIOLATION"
}
```

### 1.1 Field Semantics

| Field | Required | Description |
|-------|----------|-------------|
| `code` | Yes | Upper snake case: `SKILLPACK_{CATEGORY}_{DESCRIPTION}` |
| `category` | Yes | `validation`, `security`, `network`, `storage`, `auth`, `internal` |
| `severity` | Yes | `fatal`, `error`, `warning`, `info` |
| `message` | Yes | Human-readable, no jargon, actionable where possible |
| `details` | No | Arbitrary object; schema depends on code |
| `clientHint` | No | Per-client rendering suggestions; clients MAY ignore |
| `traceId` | Yes | UUID for correlation across logs |
| `docsUrl` | No | Link to documentation for this error |

---

## 2. Severity Levels

### 2.1 Definition

| Severity | Meaning | Action |
|----------|---------|--------|
| `fatal` | Unrecoverable; system or client cannot continue | Abort operation; show crash UI; log for investigation |
| `error` | Operation failed; no state changed | Show failure; offer retry or alternative |
| `warning` | Operation completed but degraded | Show notice; continue; log for review |
| `info` | Informational; no action required | Log or transient toast; no user interruption |

### 2.2 Per-Client Rendering

| Severity | CLI | VS Code | Web | macOS | CNI |
|----------|-----|---------|-----|-------|-----|
| `fatal` | Exit 1, red stderr, stack trace if `--verbose` | Error notification, disable extension until restart | Full-screen error overlay | Alert modal (critical) | Exit 1, full JSON to stderr |
| `error` | Red text, exit 78 | Error notification with action buttons | Inline banner, red border | Alert sheet | Exit 78, JSON to stderr |
| `warning` | Yellow text, exit 0 | Warning notification | Inline warning, yellow border | Log entry + subtle badge | Exit 0, JSON to stdout with `warnings` array |
| `info` | Gray text, silent if `--quiet` | Status bar message | Toast (auto-dismiss 3s) | Console log | Stdout JSON, no exit code impact |

---

## 3. Error Code Registry

### 3.1 Validation (`SKILLPACK_VAL_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_VAL_INVALID_MANIFEST` | validation | error | `Manifest validation failed: {reason}` | `{ schemaErrors: string[] }` |
| `SKILLPACK_VAL_MISSING_REQUIRED` | validation | error | `Missing required field: {field}` | `{ field: string, location: string }` |
| `SKILLPACK_VAL_SCHEMA_VERSION` | validation | error | `Unsupported schema version: {version}` | `{ version: int, supported: int[] }` |
| `SKILLPACK_VAL_INVALID_VERSION` | validation | error | `Invalid semantic version: {version}` | `{ version: string }` |
| `SKILLPACK_VAL_DUPLICATE_SKILL` | validation | warning | `Skill '{name}' already exists in canonical store` | `{ name: string, existingPath: string }` |

### 3.2 Security (`SKILLPACK_SEC_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_SEC_IP_VIOLATION` | security | error | `Skill name contains restricted pattern: {violations}` | `{ violations: string[], suggestion: string }` |
| `SKILLPACK_SEC_POLICY_DENIAL` | security | error | `Operation denied by policy: {policy}` | `{ policy: string, reason: string }` |
| `SKILLPACK_SEC_UNTRUSTED_REGISTRY` | security | warning | `Registry '{registry}' is not in trust list` | `{ registry: string, trustList: string[] }` |
| `SKILLPACK_SEC_INVALID_SIGNATURE` | security | error | `Evidence signature verification failed` | `{ signer: string, error: string }` |
| `SKILLPACK_SEC_NO_PROVENANCE` | security | warning | `Skill has no provenance attestation` | `{ skillName: string }` |

### 3.3 Network (`SKILLPACK_NET_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_NET_CONNECTION_REFUSED` | network | error | `Cannot connect to server at {url}` | `{ url: string, attempts: int }` |
| `SKILLPACK_NET_TIMEOUT` | network | error | `Request timed out after {timeout}s` | `{ timeout: number, operation: string }` |
| `SKILLPACK_NET_TLS_ERROR` | network | error | `TLS handshake failed: {reason}` | `{ reason: string }` |
| `SKILLPACK_NET_RATE_LIMITED` | network | warning | `Rate limit exceeded; retry after {retryAfter}s` | `{ retryAfter: int, limit: int }` |
| `SKILLPACK_NET_DNS_FAILURE` | network | error | `DNS resolution failed for {host}` | `{ host: string }` |

### 3.4 Storage (`SKILLPACK_STO_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_STO_PATH_NOT_FOUND` | storage | error | `Path not found: {path}` | `{ path: string }` |
| `SKILLPACK_STO_PERMISSION_DENIED` | storage | error | `Permission denied: {path}` | `{ path: string, required: string }` |
| `SKILLPACK_STO_DISK_FULL` | storage | error | `Insufficient disk space: {needed} needed, {available} available` | `{ needed: string, available: string }` |
| `SKILLPACK_STO_OCI_PUSH_FAILED` | storage | error | `OCI push failed: {reason}` | `{ registry: string, ref: string, reason: string }` |
| `SKILLPACK_STO_OCI_PULL_FAILED` | storage | error | `OCI pull failed: {reason}` | `{ registry: string, ref: string, reason: string }` |
| `SKILLPACK_STO_FDB_UNAVAILABLE` | storage | warning | `FoundationDB metadata index is offline` | `{ endpoint: string }` |
| `SKILLPACK_STO_RUSTFS_UNAVAILABLE` | storage | warning | `RustFS payload store is offline` | `{ endpoint: string }` |
| `SKILLPACK_STO_FALLBACK_ACTIVE` | storage | info | `Operating in local fallback mode` | `{ missingBackend: string }` |

### 3.5 Auth (`SKILLPACK_AUTH_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_AUTH_UNAUTHORIZED` | auth | error | `Authentication required` | `{ resource: string }` |
| `SKILLPACK_AUTH_FORBIDDEN` | auth | error | `Permission denied for {action}` | `{ action: string, resource: string }` |
| `SKILLPACK_AUTH_TOKEN_EXPIRED` | auth | error | `Token expired at {expiry}` | `{ expiry: string }` |
| `SKILLPACK_AUTH_INVALID_TOKEN` | auth | error | `Invalid or malformed token` | `{ hint: string }` |

### 3.6 Internal (`SKILLPACK_INT_*`)

| Code | Category | Severity | Message Template | Details Schema |
|------|----------|----------|------------------|----------------|
| `SKILLPACK_INT_UNEXPECTED` | internal | fatal | `Unexpected internal error: {message}` | `{ message: string, location: string }` |
| `SKILLPACK_INT_GRPC_ERROR` | internal | error | `gRPC error: {status} — {message}` | `{ status: string, message: string }` |
| `SKILLPACK_INT_NOT_IMPLEMENTED` | internal | error | `Operation not yet implemented: {operation}` | `{ operation: string }` |
| `SKILLPACK_INT_DEPENDENCY_MISSING` | internal | error | `Required dependency not found: {dependency}` | `{ dependency: string, installHint: string }` |

---

## 4. Exit Codes (CLI & CNI)

| Exit Code | Name | Meaning |
|-----------|------|---------|
| 0 | `SUCCESS` | Operation completed successfully |
| 1 | `GENERAL_ERROR` | Unspecified failure |
| 64 | `USAGE_ERROR` | Command-line usage error |
| 65 | `DATA_ERROR` | Input data error (invalid manifest, etc.) |
| 66 | `NO_INPUT` | Cannot open input file |
| 69 | `UNAVAILABLE` | Service unavailable |
| 70 | `INTERNAL_ERROR` | Internal software error |
| 73 | `CREATION_ERROR` | Cannot create output file |
| 74 | `IO_ERROR` | Input/output error |
| 77 | `PERMISSION_DENIED` | Insufficient permission |
| 78 | `CONFIG_ERROR` | Configuration error (includes validation failures) |
| 79 | `NOT_FOUND` | Entity not found |

Mapping:
- `fatal` → exit 1 (or 70 for internal)
- `error` + category=validation → exit 78
- `error` + category=security → exit 77
- `error` + category=network → exit 69
- `error` + category=storage → exit 74
- `error` + category=auth → exit 77
- `warning` → exit 0 (with warnings in output)
- `info` → exit 0
