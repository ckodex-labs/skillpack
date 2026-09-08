# Contributing to SkillPack

Thank you for your interest in contributing to SkillPack! This document provides guidelines for contributing.

## Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Issues

- Check existing issues before creating a new one
- Use the appropriate issue template
- Provide clear reproduction steps
- Include version information (`skillpack --version`)

### Pull Requests

1. **Fork and clone** the repository
2. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/my-feature
   ```
3. **Make changes** following our coding standards
4. **Test thoroughly**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace
   ```
5. **Commit** with clear messages following [Conventional Commits](https://conventionalcommits.org/):
   ```
   feat(checker): add lifecycle dimension scoring
   fix(schema): correct GAL validation range
   docs: update authoring guide examples
   ```
6. **Push** and create a PR

### Development Setup

```bash
# Clone
git clone https://github.com/ckodex/skillpack.git
cd skillpack

# Build
cargo build --workspace

# Test
cargo test --workspace

# Run lints
 cargo clippy --workspace --all-targets -- -D warnings -W clippy::unit_cmp
 cargo fmt --all -- --check

# Run the unified release gate (fmt, clippy, test, smoke)
cargo run -p xtask -- ci

# Run SkillPack on itself
cargo run --bin skillpack -- check .
```

## Architecture

SkillPack follows **Hexagonal Architecture** (Ports & Adapters) with DDD:

```
crates/
├── skillpack-domain/       # Kernel Space: Pure domain logic
├── skillpack-adapters/     # Infrastructure: Checkers, readers, persistence
├── skillpack-cli/          # Presentation: CLI interface
└── skillpack-server/       # Presentation: gRPC server
```

### Key Principles

1. **Domain-first**: Core logic has no infrastructure dependencies
2. **Port interfaces**: All external access via traits in `ports.rs`
3. **Immutable by default**: Use `pub(crate)` for internal mutation
4. **Evidence-native**: Every assessment produces verifiable output

## Adding a New Dimension Checker

1. Add dimension ID to `skillpack-domain/src/dimension.rs`:
   ```rust
   pub enum DimensionId {
       // ... existing
       MyNewDimension,
   }
   ```

2. Update `standard_dimensions()` with weight (ensure total = 100%)

3. Implement checker in `skillpack-adapters/src/checkers/`:
   ```rust
   pub struct MyNewChecker;
   impl DimensionChecker for MyNewChecker {
       fn dimension(&self) -> DimensionId { DimensionId::MyNewDimension }
       fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
           // Implementation
       }
   }
   ```

4. Add to `all_checkers()` in `checkers/mod.rs`

5. Add tests and documentation

## Schema Changes

When modifying JSON schemas:

1. Update the standalone schema in `schemas/*/v1/*-standalone.schema.json`
2. Update TypeScript types in `schemas/skill.type.ts`
3. Run schema validation tests: `cargo test -p skillpack-domain schema_validation`
4. Update the authoring guide if needed

## Commit Signing

All commits must be signed:

```bash
git config commit.gpgsign true
```

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## Questions?

- Open a [Discussion](https://github.com/ckodex/skillpack/discussions)
- Check [Documentation](./docs/)
- Review [Authoring Guide](./docs/authoring-guide.md)
