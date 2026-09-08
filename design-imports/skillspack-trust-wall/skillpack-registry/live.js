// ============================================================
// SkillPack Registry · live data layer
// Wires the browser to the running skillpack-server:
//   - real canonical catalog (canonical-registry.json, 196 skills)
//   - live /health (connection status)
//   - live /api/v1/skills/assess (real 9-dimension grade, on demand)
// Nothing is fabricated: fields the catalog does not carry (grade, dims,
// provenance, downloads, version) are null/unknown until a real assessment
// runs. If the server or catalog is unreachable, App falls back to the
// curated sample corpus — the design never breaks.
// ============================================================

const SKR_API = (window.__SKR_API__ || 'http://localhost:50081').replace(/\/+$/, '');

// server AssessmentResult.dimensions[].dimensionId → UI DIMS key
const SKR_DIM_MAP = {
  IdentityAndManifest: 'identity',
  Security: 'security',
  Provenance: 'provenance',
  Documentation: 'documentation',
  Testing: 'testing',
  Compatibility: 'compatibility',
  Lifecycle: 'lifecycle',
  Governance: 'governance',
  EvalsHitl: 'evals_hitl',
};
const SKR_UI_DIMS = ['identity', 'security', 'provenance', 'documentation', 'testing', 'compatibility', 'lifecycle', 'governance', 'evals_hitl'];
const skrEmptyDims = () => Object.fromEntries(SKR_UI_DIMS.map((d) => [d, null]));

// Probe the live server. Never throws — returns a status object.
async function skrHealth() {
  try {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), 2000);
    const r = await fetch(`${SKR_API}/health`, { signal: ctrl.signal });
    clearTimeout(timer);
    if (!r.ok) return { healthy: false, endpoint: SKR_API };
    const b = await r.json().catch(() => ({}));
    return { healthy: true, endpoint: SKR_API, protocolVersion: b.protocolVersion };
  } catch (_e) {
    return { healthy: false, endpoint: SKR_API };
  }
}

// Map one raw canonical-index entry to the UI skill model, unrated.
function skrMapRaw(raw, canonicalRoot) {
  return {
    id: raw.name,
    name: raw.name,
    version: null,
    publisher: 'canonical store',
    primitive: null,
    registry: 'local-cache',
    grade: '—',        // unrated until a real assessment runs
    score: null,
    tier: null,
    gal: null,
    signed: 'unknown', // no signature check has been performed
    provenance: false,
    downloads: null,   // no install telemetry
    updated: null,
    sizeKB: null,
    deps: null,
    privacy: null,
    installed: false,
    synopsis: raw.description || `Skill · ${raw.name}`,
    status: raw.status || null,
    dims: skrEmptyDims(),
    caps: { read: false, write: false, network: false, filesystem: false, execution: false },
    ociRef: `file://${canonicalRoot}/${raw.path}`,
    _live: true,
    _assessed: false,
    _assessing: false,
    _path: `${canonicalRoot}/${raw.path}`,
  };
}

// Load the real canonical catalog (same-origin static file). Throws on failure
// so the caller can fall back to the sample corpus.
async function skrLoadCanonical() {
  const r = await fetch('canonical-registry.json');
  if (!r.ok) throw new Error(`canonical index unavailable (${r.status})`);
  const idx = await r.json();
  const root = idx.canonicalRoot || '';
  const raw = Array.isArray(idx.skills) ? idx.skills : [];
  const skills = raw.filter((s) => s.hasSkillMd).map((s) => skrMapRaw(s, root));
  return { root, total: raw.length, withSkillMd: skills.length, skills };
}

// Run a real assessment for one skill. Returns a patch of fields to merge, or
// null on failure (caller keeps the unrated state).
async function skrAssess(skill) {
  try {
    const url = `${SKR_API}/api/v1/skills/assess?path=${encodeURIComponent(skill._path)}`;
    const r = await fetch(url);
    if (!r.ok) return null;
    const body = await r.json();
    const a = body && body.assessment;
    if (!a) return null;
    const dims = skrEmptyDims();
    (a.dimensions || []).forEach((d) => {
      const key = SKR_DIM_MAP[d.dimensionId];
      if (key) dims[key] = Math.round(d.score);
    });
    return {
      grade: a.grade || '—',
      score: a.totalScore != null ? Math.round(a.totalScore) : null,
      dims,
      issues: Array.isArray(a.issues) ? a.issues : [],
      profile: a.profile || null,
      _assessed: true,
      _assessing: false,
    };
  } catch (_e) {
    return null;
  }
}

// Load the server-side pre-assessed catalog in a single request. The server
// assesses the canonical store once at startup and caches it, so this returns
// real grades + dimensions for the whole store without a per-skill fan-out.
// Returns { hydrating, total, assessed, skills:[{ name, grade, score, dims, issues, profile }] }.
async function skrLoadCatalog() {
  const r = await fetch(`${SKR_API}/api/v1/registry/catalog`);
  if (!r.ok) throw new Error(`catalog unavailable (${r.status})`);
  const j = await r.json();
  const skills = (j.skills || []).map((e) => {
    const dims = skrEmptyDims();
    (e.dimensions || []).forEach((d) => {
      const key = SKR_DIM_MAP[d.dimensionId];
      if (key) dims[key] = Math.round(d.score);
    });
    return {
      name: e.name,
      grade: e.grade || '—',
      score: e.totalScore != null ? Math.round(e.totalScore) : null,
      dims,
      issues: e.issues,
      profile: e.profile || null,
    };
  });
  return { hydrating: !!j.hydrating, total: j.total || 0, assessed: j.assessed || 0, skills };
}

// Assess the whole catalog with bounded concurrency. Calls onBatch(results,
// done, total) after each batch, where results is [{ id, patch|null }]. Keeps
// the server from being hammered and lets the UI fill in grades progressively.
async function skrAssessAll(skills, opts) {
  const concurrency = (opts && opts.concurrency) || 8;
  const onBatch = opts && opts.onBatch;
  const stopped = opts && opts.stopped; // () => bool, lets the caller cancel
  let done = 0;
  for (let i = 0; i < skills.length; i += concurrency) {
    if (stopped && stopped()) return;
    const batch = skills.slice(i, i + concurrency);
    const results = await Promise.all(batch.map(async (s) => ({ id: s.id, patch: await skrAssess(s) })));
    done += batch.length;
    if (onBatch) onBatch(results, done, skills.length);
  }
}

Object.assign(window, { SKR_API, skrHealth, skrLoadCanonical, skrLoadCatalog, skrAssess, skrAssessAll, skrMapRaw });
