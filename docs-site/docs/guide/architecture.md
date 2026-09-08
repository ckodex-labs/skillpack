# Architecture

SkillPack follows CKODEX architectural principles.

## Three Spaces + Proof

```
┌─────────────────────────────────────────────────────────┐
│                   Presentation Space                     │
│  CLI │ gRPC API │ Next.js Dashboard │ GitHub Actions    │
├─────────────────────────────────────────────────────────┤
│                   Validation Space                       │
│  OPA Policies │ Quality Gates │ Promotion Rules         │
├─────────────────────────────────────────────────────────┤
│                     Kernel Space                         │
│  Domain Model │ Dimensions │ Scoring │ Assessment       │
├─────────────────────────────────────────────────────────┤
│                     Proof Space                          │
│  SLSA Provenance │ SBOM │ Sigstore │ in-toto            │
└─────────────────────────────────────────────────────────┘
```

## Hexagonal Architecture

```
         ┌───────────────────────────┐
         │      Application          │
         │   (Use Cases)             │
         └─────────┬─────────────────┘
                   │
    ┌──────────────┴──────────────┐
    │                             │
┌───▼───┐                     ┌───▼───┐
│ Ports │◄────── Domain ──────►│ Ports │
└───┬───┘    (Assessment)     └───┬───┘
    │                             │
┌───▼───────────┐     ┌───────────▼───┐
│ CLI Adapter   │     │ OCI Adapter   │
│ gRPC Adapter  │     │ Git Adapter   │
│ FS Adapter    │     │ S3 Adapter    │
└───────────────┘     └───────────────┘
```

## Crate Structure

| Crate                 | Layer          | Purpose           |
| --------------------- | -------------- | ----------------- |
| `skillpack-domain`      | Kernel         | Pure domain model |
| `skillpack-application` | Application    | Use cases         |
| `skillpack-adapters`    | Infrastructure | CLI, readers      |
| `skillpack-api`         | Presentation   | gRPC server       |

## Domain Model

### Assessment Aggregate

```rust
pub struct Assessment {
    pub id: AssessmentId,
    pub skill: SkillIdentity,
    pub dimension_scores: HashMap<DimensionId, Score>,
    pub bonus_points: BonusPoints,
    pub issues: Vec<Issue>,
    pub assessed_at: DateTime<Utc>,
}
```

### Dimension Value Object

8 dimensions with weighted scoring:

| Dimension     | Weight |
| ------------- | ------ |
| Structure     | 15%    |
| Security      | 25%    |
| Provenance    | 20%    |
| Governance    | 15%    |
| Templates     | 10%    |
| MCP           | 5%     |
| Documentation | 5%     |
| Accessibility | 5%     |
