# SkillPack

> **Private Beta Release** — `1.0.0-beta.1`  
> This is an early release for trusted, local/private evaluation. See [KNOWN-ISSUES.md](KNOWN-ISSUES.md) and [evidence/ALPHA-READINESS.md](evidence/ALPHA-READINESS.md) for current limitations and risks.

**AI Agent Skill Quality Assessment Framework**

SkillPack is a production-grade tool for assessing, validating, and publishing AI agent skill packs. It provides multi-dimensional quality scoring equivalent to SonarQube for traditional codebases.

## Features

- **Multi-Dimension Assessment** - Identity & Manifest, Security, Provenance, Documentation, Testing, Compatibility, Lifecycle, Governance, Evals & HITL
- **Grade System** - S+/S/A/B/C/D/F with configurable thresholds
- **CLI Tool** - `skillpack check`, `init`, `package`, `publish`, `install`, `eval`, `discover`, `lock`, `migrate`, `store sync/migrate/status`
- **VS Code Extension** - Inline diagnostics, AI assistant, skill creation wizard
- **gRPC API** - High-performance server for registry integration
- **OCI Distribution** - Push/pull skills via ORAS-compatible registries

## Quick Start

```bash
# Install CLI
cargo install --path . --bin skillpack

# Create new skill
skillpack init my-skill --template mcp

# Assess skill quality
cd my-skill
skillpack check

# Generate lock file
skillpack lock

# Package skill (supports tar.gz, tar.bz2, tar.br, tar.zst, zip)
skillpack package --format zip

# Publish to OCI registry
skillpack publish --registry ghcr.io/ckodex/my-skill:0.1.0

# Install from registry
skillpack install ghcr.io/ckodex/my-skill:0.1.0 --output ./skills

# Run evaluation suite
skillpack eval --suite smoke

# Discover skills in a directory
skillpack discover --limit 10
```

## CLI Commands

| Command                                 | Description                                  |
| --------------------------------------- | -------------------------------------------- |
| `skillpack check [path]`                | Run full quality assessment                  |
| `skillpack grade [path]`                | Show letter grade only                       |
| `skillpack report [path]`               | Generate JSON/SARIF/Markdown report          |
| `skillpack validate <file>`             | Validate schema (CNSB/CNAAB/Evidence/Policy) |
| `skillpack init <name>`                 | Scaffold new skill project                   |
| `skillpack package [path]`              | Create archive (tar.gz/bz2/br/zst/zip)       |
| `skillpack publish [path]`              | Push skill to OCI registry                   |
| `skillpack install <ref>`               | Pull skill from OCI registry                 |
| `skillpack eval [path]`                 | Run evaluation suites (smoke/compliance)     |
| `skillpack discover [path]`             | Discover skills in directory                 |
| `skillpack lock [path]`                 | Generate skill.lock with integrity hashes    |
| `skillpack migrate [path]`              | Migrate manifest to latest schema version    |
| `skillpack store sync`                  | Sync agent directories with canonical store  |
| `skillpack store migrate`               | Migrate physical skills to canonical store   |
| `skillpack store status`                | Show canonical store health & agent status   |
| `skillpack store check-boundary <path>` | Check IP boundary violations                 |

## VS Code Extension

Install from `vscode-extension/` for:
- Real-time quality diagnostics
- AI-assisted skill improvement
- Skill creation wizard
- Registry browser and installation

## Environment Variables

| Variable                         | Default       | Description                                                |
| -------------------------------- | ------------- | ---------------------------------------------------------- |
| `SKILLPACK_PORT`                 | `50051`       | gRPC server port                                           |
| `SKILLPACK_HTTP_PORT`            | `50052`       | HTTP server port                                           |
| `SKILLPACK_BIND_ALL`             | `false`       | Bind to `0.0.0.0` instead of `127.0.0.1`                   |
| `SKILLPACK_API_TOKEN`            | *(none)*      | Bearer token for mutation endpoints; unset = unprotected   |
| `SKILLPACK_CORS_ORIGINS`         | *(none)*      | Comma-separated allowed origins; unset = deny cross-origin |
| `SKILLPACK_REQUEST_TIMEOUT_SECS` | `30`          | HTTP request timeout (DoS protection)                      |
| `SKILLPACK_MAX_BODY_SIZE_MB`     | `10`          | Max HTTP request body size in MiB (DoS protection)         |
| `SKILLPACK_DB_PATH`              | *(in-memory)* | File path for DuckDB persistence                           |
| `SKILLPACK_OCI_AUTH`             | *(none)*      | OCI registry auth in `user:pass` format                    |

**Security note:** Always set `SKILLPACK_API_TOKEN` before exposing the server to any network. `SKILLPACK_BIND_ALL=1` without a token is a critical security risk.

## Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Start gRPC server
cargo run --bin skillpack-server

# Compile VS Code extension
cd vscode-extension && npm run compile
```

## License

Apache-2.0
