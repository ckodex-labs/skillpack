// ============================================================
// SkillPack Registry · mock RegistryEntry corpus
// Grounded in SKILLS-DOSSIER §1.3 skill names + client-model.schema.json
// shape (RegistryEntry). Standing in for IndexService.Search results.
// ============================================================
const DIMS = ['identity','security','provenance','documentation','testing','compatibility','lifecycle','governance','evals_hitl'];

const REGISTRIES = [
  { id: 'ckodex-canonical', label: 'ckodex.canonical', host: 'oci://reg.ckodex.org', trust: 'trusted',   note: 'first-party · cosign root' },
  { id: 'skills-sh',        label: 'skills.sh',         host: 'oci://registry.skills.sh', trust: 'trusted',   note: 'community · attested mirror' },
  { id: 'partner-atlas',    label: 'atlas.partner',     host: 'oci://oci.atlas-labs.io', trust: 'review',    note: 'partner · review required' },
  { id: 'local-cache',      label: 'local.cache',       host: 'file://~/Skills/shared',  trust: 'trusted',   note: 'on-disk canonical store' },
];

const mkDims = (vals) => Object.fromEntries(DIMS.map((d, i) => [d, vals[i]]));

const SKILLS = [
  {
    id: 'writing-skills', name: 'writing-skills', version: '1.4.0', publisher: 'superpowers',
    primitive: 'create', registry: 'ckodex-canonical', grade: 'S+', score: 124, tier: 'L4', gal: 6,
    signed: 'signed', provenance: true, downloads: 48210, updated: '2026-05-21', sizeKB: 142, deps: 2,
    privacy: 'public-anchor', installed: false,
    synopsis: 'Author new skills with a TDD baseline, progressive disclosure, and loophole-closing refactor. Use when creating any skill that an agent will load.',
    dims: mkDims([100, 96, 98, 100, 92, 88, 94, 96, 90]),
    caps: { read: true, write: true, network: false, filesystem: true, execution: false },
    ociRef: 'oci://reg.ckodex.org/skills/writing-skills@sha256:a42f…c01e',
  },
  {
    id: 'conformance', name: 'conformance', version: '0.9.2', publisher: 'ckodex-labs',
    primitive: 'assess', registry: 'ckodex-canonical', grade: 'A', score: 112, tier: 'L3', gal: 5,
    signed: 'signed', provenance: true, downloads: 21884, updated: '2026-05-19', sizeKB: 88, deps: 1,
    privacy: 'audit-private', installed: true,
    synopsis: 'Validate bidirectional conformance-vector references (CV-SAFE, CV-ECON, CV-EVID) against the active policy bundle. Use before promoting a skill past L2.',
    dims: mkDims([96, 100, 90, 84, 86, 80, 92, 98, 78]),
    caps: { read: true, write: false, network: true, filesystem: true, execution: false },
    ociRef: 'oci://reg.ckodex.org/skills/conformance@sha256:c91d…4b2a',
  },
  {
    id: 'proof-audit', name: 'proof-audit', version: '0.7.0', publisher: 'ckodex-labs',
    primitive: 'assess', registry: 'ckodex-canonical', grade: 'A', score: 104, tier: 'L3', gal: 5,
    signed: 'signed', provenance: true, downloads: 17502, updated: '2026-05-14', sizeKB: 74, deps: 1,
    privacy: 'audit-private', installed: false,
    synopsis: 'Audit a skill\'s proofTypes enum and evidence chain for closure and freshness. Emits an EvidenceBundle linking the verdict to the content hash.',
    dims: mkDims([90, 94, 100, 80, 82, 78, 88, 92, 84]),
    caps: { read: true, write: false, network: false, filesystem: true, execution: false },
    ociRef: 'oci://reg.ckodex.org/skills/proof-audit@sha256:77e0…9f3c',
  },
  {
    id: 'freshness', name: 'freshness', version: '0.5.1', publisher: 'ckodex-labs',
    primitive: 'assess', registry: 'skills-sh', grade: 'B', score: 94, tier: 'L2', gal: 4,
    signed: 'signed', provenance: false, downloads: 9920, updated: '2026-04-30', sizeKB: 51, deps: 0,
    privacy: 'audit-private', installed: false,
    synopsis: 'Check that docs are current, versions pinned, and signature age is within the rotation interval. Flags stale evidence past its decay half-life.',
    dims: mkDims([84, 78, 62, 92, 74, 82, 90, 80, 66]),
    caps: { read: true, write: false, network: true, filesystem: true, execution: false },
    ociRef: 'oci://registry.skills.sh/freshness@sha256:12ab…8e5d',
  },
  {
    id: 'ckodex-audit', name: 'ckodex-audit', version: '2.1.0', publisher: 'ckodex-labs',
    primitive: 'assess', registry: 'ckodex-canonical', grade: 'S+', score: 121, tier: 'L4X', gal: 6,
    signed: 'signed', provenance: true, downloads: 33140, updated: '2026-05-24', sizeKB: 196, deps: 3,
    privacy: 'public-anchor', installed: false,
    synopsis: 'Run the full 9-dimension quality checker pipeline and assign a grade S+→F. Use as the canonical quality gate before any registry publish.',
    dims: mkDims([98, 100, 96, 94, 96, 90, 98, 100, 92]),
    caps: { read: true, write: false, network: true, filesystem: true, execution: true },
    ociRef: 'oci://reg.ckodex.org/skills/ckodex-audit@sha256:3f09…d72b',
  },
  {
    id: 'capability-lock', name: 'capability-lock', version: '0.3.0', publisher: 'ckodex-labs',
    primitive: 'bundle', registry: 'ckodex-canonical', grade: 'B', score: 88, tier: 'L2', gal: 4,
    signed: 'signed', provenance: true, downloads: 6610, updated: '2026-05-08', sizeKB: 63, deps: 1,
    privacy: 'audit-private', installed: false,
    synopsis: 'Compile a CapabilityLock (JSON + sha256 + cosign) per skill and assemble a signed OCI SkillBundle. Use when packaging skills for distribution.',
    dims: mkDims([86, 92, 90, 72, 70, 76, 84, 88, 60]),
    caps: { read: true, write: true, network: true, filesystem: true, execution: true },
    ociRef: 'oci://reg.ckodex.org/skills/capability-lock@sha256:5e71…b0c4',
  },
  {
    id: 'find-skills', name: 'find-skills', version: '1.0.4', publisher: 'skills-sh',
    primitive: 'exchange', registry: 'skills-sh', grade: 'B', score: 90, tier: 'L3', gal: 4,
    signed: 'signed', provenance: false, downloads: 58730, updated: '2026-05-17', sizeKB: 39, deps: 0,
    privacy: 'public-anchor', installed: false,
    synopsis: 'Search the skills.sh ecosystem and install by owner/repo@skill. Use when discovering community skills across the open registry.',
    dims: mkDims([88, 70, 58, 90, 80, 86, 82, 74, 68]),
    caps: { read: true, write: true, network: true, filesystem: true, execution: false },
    ociRef: 'oci://registry.skills.sh/find-skills@sha256:9b1c…2a7f',
  },
  {
    id: 'scaffold', name: 'scaffold', version: '1.1.0', publisher: 'ckodex-tools',
    primitive: 'create', registry: 'ckodex-canonical', grade: 'A', score: 100, tier: 'L3', gal: 5,
    signed: 'signed', provenance: true, downloads: 27440, updated: '2026-05-12', sizeKB: 57, deps: 0,
    privacy: 'audit-private', installed: false,
    synopsis: 'Emit a v1.1 skill skeleton tree (SKILL.md, skill.json, references, scripts). Idempotent; enforces the name regex and length bounds.',
    dims: mkDims([100, 84, 80, 96, 88, 90, 86, 82, 72]),
    caps: { read: true, write: true, network: false, filesystem: true, execution: true },
    ociRef: 'oci://reg.ckodex.org/skills/scaffold@sha256:6d2e…11a0',
  },
  {
    id: 'simplify', name: 'simplify', version: '0.4.2', publisher: 'superpowers',
    primitive: 'evolve', registry: 'partner-atlas', grade: 'C', score: 72, tier: 'L1', gal: 3,
    signed: 'unsigned', provenance: false, downloads: 4120, updated: '2026-03-28', sizeKB: 44, deps: 1,
    privacy: 'debug-local', installed: false,
    synopsis: 'Reduce skill surface area by collapsing redundant references and pruning dead scripts. Pattern-level evolve; supersession path not yet wired.',
    dims: mkDims([74, 60, 40, 78, 62, 70, 58, 64, 48]),
    caps: { read: true, write: true, network: false, filesystem: true, execution: false },
    ociRef: 'oci://oci.atlas-labs.io/simplify@sha256:8f4a…73be',
  },
  {
    id: 'token-migration', name: 'token-migration', version: '0.2.0', publisher: 'partner-atlas',
    primitive: 'evolve', registry: 'partner-atlas', grade: 'D', score: 54, tier: 'L1', gal: 2,
    signed: 'untrusted', provenance: false, downloads: 880, updated: '2026-02-11', sizeKB: 31, deps: 2,
    privacy: 'debug-local', installed: false,
    synopsis: 'Migrate design tokens across schema versions. Sourced from a partner registry that is not in the trust list — review before install.',
    dims: mkDims([60, 44, 28, 66, 50, 58, 46, 52, 36]),
    caps: { read: true, write: true, network: true, filesystem: true, execution: true },
    ociRef: 'oci://oci.atlas-labs.io/token-migration@sha256:2c80…aa19',
  },
  {
    id: 'state-vector', name: 'state-vector', version: '0.8.0', publisher: 'ckodex-labs',
    primitive: 'distribute', registry: 'ckodex-canonical', grade: 'A', score: 108, tier: 'L3', gal: 5,
    signed: 'signed', provenance: true, downloads: 14260, updated: '2026-05-20', sizeKB: 102, deps: 2,
    privacy: 'regulated-export', installed: false,
    synopsis: 'Propagate a signed skill set to the 12-agent fanout matrix with per-environment promotion gates. Use for dev → staging → prod distribution.',
    dims: mkDims([94, 96, 92, 86, 84, 88, 96, 94, 80]),
    caps: { read: true, write: true, network: true, filesystem: true, execution: true },
    ociRef: 'oci://reg.ckodex.org/skills/state-vector@sha256:a019…ce42',
  },
  {
    id: 'design-lint', name: 'design-lint', version: '1.2.1', publisher: 'superpowers',
    primitive: 'assess', registry: 'skills-sh', grade: 'B', score: 92, tier: 'L2', gal: 4,
    signed: 'signed', provenance: true, downloads: 19030, updated: '2026-05-05', sizeKB: 68, deps: 1,
    privacy: 'audit-private', installed: false,
    synopsis: 'Lint a design artifact against the active design system: tokens, contrast (APCA), spacing grid, and component vocabulary. Use before design review.',
    dims: mkDims([90, 80, 74, 94, 86, 88, 82, 84, 76]),
    caps: { read: true, write: false, network: false, filesystem: true, execution: false },
    ociRef: 'oci://registry.skills.sh/design-lint@sha256:7e3d…4f90',
  },
];

const PRIMITIVES = [
  { id: 'create',     glyph: '✦', label: 'Create' },
  { id: 'assess',     glyph: '⊢', label: 'Assess' },
  { id: 'evolve',     glyph: '⇗', label: 'Evolve' },
  { id: 'exchange',   glyph: '⇄', label: 'Exchange' },
  { id: 'bundle',     glyph: '⊞', label: 'Bundle' },
  { id: 'distribute', glyph: '≋', label: 'Distribute' },
];

// ---- Discovery: registry endpoints (extends REGISTRIES with probe data) ----
const ENDPOINTS = [
  { id: 'ckodex-canonical', label: 'ckodex.canonical', url: 'https://reg.ckodex.org', wellKnown: 'ok', protocol: 1, transports: ['gRPC', 'REST'], features: ['assessment','grading','registry','lifecycle','migration'], trust: 'trusted', latencyMs: 18, health: 'healthy' },
  { id: 'skills-sh',        label: 'skills.sh',         url: 'https://registry.skills.sh', wellKnown: 'ok', protocol: 1, transports: ['REST'], features: ['registry','grading'], trust: 'trusted', latencyMs: 64, health: 'healthy' },
  { id: 'partner-atlas',    label: 'atlas.partner',     url: 'https://oci.atlas-labs.io', wellKnown: 'stale', protocol: 1, transports: ['gRPC'], features: ['registry'], trust: 'review', latencyMs: 142, health: 'degraded' },
  { id: 'local-cache',      label: 'local.cache',       url: 'file://~/Skills/shared', wellKnown: 'n/a', protocol: 1, transports: ['in-proc'], features: ['registry','assessment','migration'], trust: 'trusted', latencyMs: 1, health: 'healthy' },
];

// ---- The .well-known/skillpack.json manifest (inlined; mirrors the real file) ----
const WELL_KNOWN = {
  schemaVersion: 1,
  service: 'skillpack-registry',
  registry: 'ckodex.canonical',
  host: 'oci://reg.ckodex.org',
  protocolVersions: [1],
  transports: [
    { protocol: 'gRPC', endpoint: 'grpc://reg.ckodex.org:50051', primary: true },
    { protocol: 'REST', endpoint: 'https://reg.ckodex.org:50052/api/v1', primary: false },
  ],
  features: ['assessment', 'grading', 'registry', 'lifecycle', 'streaming', 'migration'],
  discovery: { search: '/api/v1/skills/search', profile: '/api/v1/skills/{id}', health: '/health' },
  trust: { signing: 'cosign', publicKey: '/.well-known/cosign.pub', transparencyLog: 'https://rekor.ckodex.org', trustList: ['ckodex.canonical', 'skills.sh'] },
  migration: { supported: true, targetApiVersion: 'ckodex.skill/v1.1', adapters: ['skill.json@v1', 'cursor.mdc', 'openai.function', 'skill.md'] },
  privacyDefault: 'audit-private',
};

// ---- Connection sequence (spec §3.1) ----
const CONNECT_SEQ = [
  { id: 'discover', glyph: '⌖', title: 'Discovery', detail: 'GET /.well-known/skillpack.json · resolve SKILLPACK_API_URL' },
  { id: 'health',   glyph: '♥', title: 'Health probe', detail: 'GET /health · timeout 2s' },
  { id: 'cap',      glyph: '⇄', title: 'Capability negotiation', detail: 'select highest mutual protocol · v1' },
  { id: 'auth',     glyph: '⊢', title: 'Authenticate', detail: 'Bearer · SKILLPACK_TOKEN' },
  { id: 'keep',     glyph: '∿', title: 'Keepalive', detail: 'gRPC ping every 30s · ack 10s' },
];

// ---- Migration adapters (foreign format → SkillPack v1.1) ----
const MIGRATION_SOURCES = [
  {
    id: 'skilljson-v1', label: 'skill.json @ v1.0', kind: 'legacy', adapter: 'schema-bump',
    note: 'CKODEX v1.0 manifest · in-place apiVersion bump',
    before: `{
  "name": "legacy-formatter",
  "version": "0.3.0",
  "description": "Formats code blocks",
  "entry": "format.py"
}`,
    diff: [
      { t: 'ctx', s: '{' },
      { t: 'add', s: '  "apiVersion": "ckodex.skill/v1.1",' },
      { t: 'add', s: '  "schemaVersion": 1,' },
      { t: 'ctx', s: '  "name": "legacy-formatter",' },
      { t: 'ctx', s: '  "version": "0.3.0",' },
      { t: 'add', s: '  "synopsis": "MIGRATION: synthesized — Use when formatting code blocks.",' },
      { t: 'ctx', s: '  "description": "Formats code blocks",' },
      { t: 'add', s: '  "ontology": { "primitive": "evolve", "MIGRATION": "review" },' },
      { t: 'add', s: '  "runtime": { "minTier": "L0", "maxTier": "L2" },' },
      { t: 'add', s: '  "capabilities": { "read": true, "write": true,' },
      { t: 'add', s: '    "network": false, "filesystem": true, "execution": false },' },
      { t: 'add', s: '  "contentHash": "sha256:6d2e…11a0",' },
      { t: 'ctx', s: '  "entry": "format.py"' },
      { t: 'ctx', s: '}' },
    ],
    result: { grade: 'B', score: 84, tier: 'L1', issues: 1 },
  },
  {
    id: 'cursor-mdc', label: 'cursor .mdc rule', kind: 'foreign', adapter: 'cursor→skill',
    note: 'Cursor rules file · frontmatter + body extraction',
    before: `---
description: React conventions
globs: ["**/*.tsx"]
alwaysApply: false
---
Use function components and hooks.`,
    diff: [
      { t: 'add', s: '# SKILL.md  (generated from cursor.mdc)' },
      { t: 'add', s: '---' },
      { t: 'add', s: 'apiVersion: ckodex.skill/v1.1' },
      { t: 'add', s: 'name: react-conventions' },
      { t: 'add', s: 'synopsis: "MIGRATION: Use when authoring React .tsx components."' },
      { t: 'ctx', s: 'description: React conventions' },
      { t: 'add', s: 'ontology: { primitive: assess, glob: "**/*.tsx" }' },
      { t: 'del', s: 'globs: ["**/*.tsx"]        # → ontology.glob' },
      { t: 'del', s: 'alwaysApply: false         # → runtime.autoload' },
      { t: 'add', s: 'runtime: { autoload: false, minTier: L0 }' },
      { t: 'ctx', s: '---' },
      { t: 'ctx', s: 'Use function components and hooks.' },
    ],
    result: { grade: 'C', score: 72, tier: 'L0', issues: 3 },
  },
  {
    id: 'openai-fn', label: 'openai function spec', kind: 'foreign', adapter: 'fn→skill',
    note: 'OpenAI tool/function JSON · parameter schema → capabilities',
    before: `{
  "name": "get_weather",
  "description": "Get weather for a city",
  "parameters": { "type": "object",
    "properties": { "city": { "type": "string" } } }
}`,
    diff: [
      { t: 'ctx', s: '{' },
      { t: 'add', s: '  "apiVersion": "ckodex.skill/v1.1",' },
      { t: 'ctx', s: '  "name": "get_weather",' },
      { t: 'add', s: '  "synopsis": "MIGRATION: Use when fetching weather for a city.",' },
      { t: 'ctx', s: '  "description": "Get weather for a city",' },
      { t: 'add', s: '  "capabilities": { "network": true, "read": true },' },
      { t: 'del', s: '  "parameters": { … }       # → references/schema.json' },
      { t: 'add', s: '  "ontology": { "primitive": "exchange" },' },
      { t: 'add', s: '  "contentHash": "sha256:9b1c…2a7f"' },
      { t: 'ctx', s: '}' },
    ],
    result: { grade: 'C', score: 66, tier: 'L0', issues: 2 },
  },
];

// ---- Sandbox specification ----
const SANDBOX = [
  { k: 'boundary class', v: 'ephemeral', tone: 'witness' },
  { k: 'network egress', v: 'denied', tone: 'deny' },
  { k: 'filesystem', v: '/tmp/sbx-{id} (scoped)', tone: 'teal' },
  { k: 'lease', v: '120s · auto-revoke', tone: 'mute' },
  { k: 'IPGuard', v: 'active', tone: 'attested' },
];

// ---- 12-agent fanout matrix (dossier §3.6) ----
const AGENTS = [
  { name: 'Claude Code', method: 'symlink', state: 'synced' },
  { name: 'Cursor Memory', method: 'symlink', state: 'synced' },
  { name: 'Codex', method: 'symlink', state: 'synced' },
  { name: 'Gemini CLI', method: 'symlink', state: 'synced' },
  { name: 'Windsurf', method: 'symlink', state: 'synced' },
  { name: 'Cline', method: 'symlink', state: 'synced' },
  { name: 'Aider', method: 'symlink', state: 'synced' },
  { name: 'Continue', method: 'symlink', state: 'pending' },
  { name: 'Zed', method: 'symlink', state: 'synced' },
  { name: 'Copilot', method: 'symlink', state: 'synced' },
  { name: 'Amp', method: 'symlink', state: 'synced' },
  { name: 'Cursor Rules', method: 'mdc-index', state: 'synced' },
];

// ---- Recent lifecycle activity ----
const ACTIVITY = [
  { ev: 'promoted', glyph: '⇗', skill: 'ckodex-audit', detail: 'L4 → L4X · GAL-6 attested', time: '14:12', tone: 'attested' },
  { ev: 'migrated', glyph: '⇄', skill: 'react-conventions', detail: 'cursor.mdc → v1.1 · sandbox dry-run', time: '13:48', tone: 'witness' },
  { ev: 'installed', glyph: '↧', skill: 'conformance', detail: 'oci pull · 12-agent fanout ok', time: '13:30', tone: 'teal' },
  { ev: 'denied', glyph: '✕', skill: 'token-migration', detail: 'SKILLPACK_SEC_UNTRUSTED_REGISTRY', time: '12:55', tone: 'deny' },
  { ev: 'superseded', glyph: '⇗', skill: 'scaffold', detail: 'v1.0.4 → v1.1.0 · BPL link preserved', time: '11:20', tone: 'witness' },
];

Object.assign(window, {
  SKILLS, REGISTRIES, DIMS, PRIMITIVES, mkDims,
  ENDPOINTS, WELL_KNOWN, CONNECT_SEQ, MIGRATION_SOURCES, SANDBOX, AGENTS, ACTIVITY,
});
