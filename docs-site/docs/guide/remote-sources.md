# Remote Sources

SkillPack supports pulling skills from remote sources.

## Supported Sources

| Source    | URI Format                      | Implementation |
| --------- | ------------------------------- | -------------- |
| **OCI**   | `oci://registry/repo:tag`       | oci-client     |
| **Git**   | `git://github.com/org/repo@ref` | git clone      |
| **S3**    | `s3://bucket/path`              | aws s3 sync    |
| **SFTP**  | `sftp://host/path`              | rsync over SSH |
| **Local** | `/path/to/skill`                | Filesystem     |

## OCI Registry

Pull skills from any OCI-compliant registry:

```bash
# Docker Hub
skillpack check oci://docker.io/org/skill:v1

# GitHub Container Registry
skillpack check oci://ghcr.io/org/skill:v1.0.0

# Private registry
skillpack check oci://registry.example.com/skill:latest
```

## Git Repository

Clone directly from Git:

```bash
# GitHub
skillpack check git://github.com/org/skill@main

# Specific tag
skillpack check git://github.com/org/skill@v1.0.0

# Subdirectory
skillpack check git://github.com/org/monorepo@main --path skills/my-skill
```

## S3 Storage

Support for AWS S3 and S3-compatible storage:

```bash
# AWS S3
skillpack check s3://my-bucket/skills/my-skill

# MinIO / SeaweedFS
skillpack check s3://bucket/path --endpoint http://minio:9000
```

## SFTP/SSH

Pull from secure remote servers:

```bash
skillpack check sftp://user@host.com/path/to/skill
skillpack check sftp://host.com:2222/path/to/skill
```

## Unified API

```rust
use skillpack_adapters::{SkillsReader, SourceBuilder};

// Auto-detect source from URI
let reader = SkillsReader::from_uri("oci://ghcr.io/org/skill:v1")?;

// With custom cache
let reader = SourceBuilder::new()
    .with_cache_dir("/tmp/skillpack-cache".into())
    .build("git://github.com/org/skill@main")?;

// Use as trait object
let skill = reader.as_skill_reader().read_identity(path)?;
let bundle = reader.as_bundle_reader().read_bundle(path)?;
```

## Caching

Remote sources are cached locally:

```
~/.skillpack-cache/
├── oci/
│   └── ghcr.io/
│       └── org/
│           └── skill/
│               └── v1/
├── git/
│   └── a1b2c3d4/  # URL hash
│       └── main/
└── s3/
    └── my-bucket/
        └── skills/
```

## Authentication

### OCI

```bash
# Docker Hub
docker login

# GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u $GITHUB_USER --password-stdin
```

### S3

```bash
# AWS credentials
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

### SFTP

Uses SSH agent or `~/.ssh/config`:

```
Host myserver
  HostName host.com
  User deploy
  IdentityFile ~/.ssh/deploy_key
```
