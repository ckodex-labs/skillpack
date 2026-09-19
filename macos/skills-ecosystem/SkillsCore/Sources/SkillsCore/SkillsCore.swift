// SkillsCore — Thin Client Layer (Deprecated Business Logic)
//
// MIGRATION NOTICE (2026-05):
// All canonical store business logic (sync, migrate, IP guard, assessment)
// has been migrated to the Rust skillpack-server workspace at the monorepo root.
//
// This module now retains ONLY:
//   - AgentConfig / AgentRegistry (agent directory definitions)
//   - SkillDiscovery (local filesystem enumeration)
//   - DirectoryWatcher (FSEventStream macOS-specific)
//   - XPC protocols (local IPC)
//
// Deprecated types kept for backwards compatibility during migration:
//   - SyncManager, IPGuard, MigrateAllStrategy, AgentSyncer, ManifestGenerator,
//     SkillCrafter, SkillImporter, Local/Distributed storage providers,
//     FoundationDBClient, RustFSClient
//
// See: macos/skills-ecosystem/MIGRATION.md
