// @ts-nocheck
'use client';
import React from 'react';
// ============================================================
// SkillPack Registry · App controller + view router
// Wired to the running skillpack-server via live.js: real canonical catalog,
import { Sidebar, Header } from './Shell';
import { RegistryBrowser, GRADE_ORDER } from './RegistryBrowser';
import { InstallDrawer } from './InstallDrawer';
import { Discovery } from './Discovery';
import { Migration } from './Migration';
import { Overview, MySkills, SyncStatus } from './Pages';
import { Quality } from './Quality';
import { Splash, Auth, Profile } from './Boot';
import { SKILLS, REGISTRIES, ENDPOINTS } from './lib/registry-data';
import { SKR_API, skrHealth, skrLoadCanonical, skrLoadCatalog, skrAssess, skrAssessAll } from './lib/live';

const LS = typeof window !== 'undefined' ? window.localStorage : { getItem: () => null, setItem: () => {}, removeItem: () => {} };

// live /health, and on-open per-skill assessment. Falls back to the curated
// sample corpus (SKILLS) if the server or catalog is unreachable.
// ============================================================
const { useState, useEffect, useMemo } = React;

const VIEW_META = {
  overview:  { eyebrow: 'Web dashboard · C-03', title: 'Overview' },
  registry:  { eyebrow: 'Install from registry', title: 'Registry browser' },
  quality:   { eyebrow: 'Quality · assurance', title: 'Fleet assurance' },
  discovery: { eyebrow: 'Federation', title: 'Discovery' },
  migration: { eyebrow: 'Evolve · sandboxed', title: 'Migration' },
  skills:    { eyebrow: 'Local store', title: 'My skills' },
  status:    { eyebrow: 'Local store', title: 'Sync & status' },
  profile:   { eyebrow: 'Session', title: 'Operator' },
};

const App = () => {
  const VALID_THEMES = ['ledger', 'vault', 'hc'];
  const [theme, setTheme] = useState(() => {
    const t = LS.getItem('skr-theme');
    return VALID_THEMES.includes(t) ? t : 'ledger';
  });
  const [view, setView] = useState(() => LS.getItem('skr-view') || 'overview');

  // Boot flow: splash → auth → console.
  const [phase, setPhase] = useState('boot'); // boot | auth | app
  const [session, setSession] = useState(() => {
    const t = LS.getItem('skr-token');
    if (t) { window.__SKILLPACK_TOKEN__ = t; return { token: t, operator: 'operator' }; }
    return null;
  });

  // live wiring
  const [liveSkills, setLiveSkills] = useState(null);   // null → sample mode
  const [liveMeta, setLiveMeta] = useState(null);       // { total, withSkillMd, root }
  const [health, setHealth] = useState({ probed: false, healthy: false });
  const [assessProgress, setAssessProgress] = useState(null); // { done, total } | null

  // registry state
  const [query, setQuery] = useState('');
  const [prim, setPrim] = useState('all');
  const [signedOnly, setSignedOnly] = useState(false);
  const [minGrade, setMinGrade] = useState(null);
  const [reg, setReg] = useState('all');
  const [sort, setSort] = useState('relevance');
  const [layout, setLayout] = useState(() => LS.getItem('skr-layout') || 'list');
  const [selected, setSelected] = useState(null);
  const [installed, setInstalled] = useState(() => new Set(SKILLS.filter(s => s.installed).map(s => s.id)));

  useEffect(() => { document.documentElement.setAttribute('data-theme', theme); LS.setItem('skr-theme', theme); }, [theme]);
  useEffect(() => { LS.setItem('skr-view', view); }, [view]);
  useEffect(() => { LS.setItem('skr-layout', layout); }, [layout]);

  // Probe the live server, load the real canonical catalog, then progressively
  // assess every skill so the grid fills with real grades.
  useEffect(() => {
    let alive = true;
    (async () => {
      let healthy = false;
      if (skrHealth) {
        const h = await skrHealth();
        healthy = !!h.healthy;
        if (alive) setHealth({ ...h, probed: true });
      }
      if (!skrLoadCanonical) return;
      let res;
      try {
        res = await skrLoadCanonical();
      } catch (_e) {
        return; // keep sample corpus
      }
      if (!alive || !res.skills.length) return;
      setLiveSkills(res.skills);
      setLiveMeta(res);

      if (!healthy) return;

      // Overlay real grades onto the (description-carrying) index rows by name.
      const mergeCatalog = (cat) => {
        const byName = new Map(cat.skills.map(s => [s.name, s]));
        setLiveSkills(prev => prev && prev.map(s => {
          const c = byName.get(s.name);
          return c ? { ...s, grade: c.grade, score: c.score, dims: c.dims, issues: c.issues, profile: c.profile, _assessed: true, _assessing: false } : s;
        }));
        setAssessProgress({ done: cat.assessed, total: cat.total });
      };

      // Preferred: the server-side pre-assessed catalog — one request, real
      // grades, cached. Poll while the server is still hydrating.
      if (skrLoadCatalog) {
        try {
          let cat = await skrLoadCatalog();
          if (cat.total || cat.skills.length) {
            if (alive) mergeCatalog(cat);
            let tries = 0;
            while (alive && cat.hydrating && tries < 40) {
              await new Promise(r => setTimeout(r, 1500));
              cat = await skrLoadCatalog();
              if (alive) mergeCatalog(cat);
              tries += 1;
            }
            return; // catalog path handled the grading
          }
        } catch (_e) { /* fall through to client-side fan-out */ }
      }

      // Fallback (older server without the catalog endpoint): assess client-side.
      if (skrAssessAll) {
        setAssessProgress({ done: 0, total: res.skills.length });
        skrAssessAll(res.skills, {
          concurrency: 8,
          stopped: () => !alive,
          onBatch: (results, done, total) => {
            if (!alive) return;
            const byId = new Map(results.map(r => [r.id, r.patch]));
            setLiveSkills(prev => prev && prev.map(s => {
              if (!byId.has(s.id)) return s;
              const patch = byId.get(s.id);
              return patch ? { ...s, ...patch } : { ...s, _assessed: true, _assessing: false };
            }));
            setAssessProgress({ done, total });
          },
        });
      }
    })();
    return () => { alive = false; };
  }, []);

  // Leave the splash once the daemon has been probed; go to sign-in unless a
  // session was already restored from storage.
  useEffect(() => {
    if (phase !== 'boot' || !health.probed) return;
    const id = setTimeout(() => setPhase(session ? 'app' : 'auth'), 600);
    return () => clearTimeout(id);
  }, [phase, health.probed, session]);

  const enterConsole = (token) => {
    if (token) {
      window.__SKILLPACK_TOKEN__ = token;
      LS.setItem('skr-token', token);
      setSession({ token, operator: 'operator' });
    } else {
      setSession({ token: null, operator: 'local operator' });
    }
    setPhase('app');
  };
  const signOut = () => {
    delete window.__SKILLPACK_TOKEN__;
    LS.removeItem('skr-token');
    setSession(null);
    setView('overview');
    setPhase('auth');
  };

  const live = liveSkills != null;
  const base = liveSkills || SKILLS;

  // Open a skill; if it is a live canonical entry, run a real assessment and
  // merge the result so the drawer shows true grade + 9 dimensions.
  const handleSelect = (skill) => {
    setSelected(skill);
    if (!skill || !skill._live || skill._assessed || skill._assessing) return;
    if (!health.healthy || !skrAssess) return;
    setLiveSkills(prev => (prev || []).map(s => s.id === skill.id ? { ...s, _assessing: true } : s));
    skrAssess(skill).then(patch => {
      setLiveSkills(prev => (prev || []).map(s => s.id === skill.id
        ? { ...s, ...(patch || { _assessing: false, _assessed: true }) }
        : s));
    });
  };

  const corpus = useMemo(() => base.map(s => ({ ...s, installed: installed.has(s.id) })), [base, installed]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    let list = corpus.filter(s => {
      if (reg !== 'all' && s.registry !== reg) return false;
      if (prim !== 'all' && s.primitive !== prim) return false;
      if (signedOnly && s.signed !== 'signed') return false;
      if (minGrade && GRADE_ORDER[s.grade] != null && GRADE_ORDER[s.grade] < GRADE_ORDER[minGrade]) return false;
      if (q) {
        const caps = s.caps ? Object.keys(s.caps).filter(k => s.caps[k]).join(' ') : '';
        const hay = `${s.name} ${s.publisher || ''} ${s.synopsis} ${s.primitive || ''} ${s.status || ''} ${caps}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
    const num = (v) => (v == null ? -1 : v);
    const cmp = {
      relevance: (a, b) => num(b.score) - num(a.score) || a.name.localeCompare(b.name),
      grade: (a, b) => (GRADE_ORDER[b.grade] ?? -1) - (GRADE_ORDER[a.grade] ?? -1) || num(b.score) - num(a.score),
      downloads: (a, b) => num(b.downloads) - num(a.downloads),
      updated: (a, b) => (b.updated || '').localeCompare(a.updated || ''),
    }[sort];
    return [...list].sort(cmp);
  }, [corpus, query, prim, signedOnly, minGrade, reg, sort]);

  const selectedLive = selected ? corpus.find(s => s.id === selected.id) : null;
  const installedCount = corpus.filter(s => s.installed).length;
  const skillCount = base.length;
  const meta = VIEW_META[view] || VIEW_META.overview;

  const subtitle = {
    overview:  `${skillCount} skills · ${ENDPOINTS.length} registries · ${installedCount} installed`,
    registry:  query.trim() ? `query "${query.trim()}" · ${filtered.length} match${filtered.length === 1 ? '' : 'es'}` : `${filtered.length} skills · 4 registries`,
    quality:   `9-dimension assessment across ${corpus.filter(s => s._assessed && s.score != null).length} assessed skills`,
    discovery: '.well-known federation · capability negotiation · spec §3.1',
    migration: 'foreign format → ckodex.skill/v1.1 · isolated sandbox · dry-run first',
    skills:    live ? `${base.length} in local store · ~/Skills/shared` : `${installedCount} installed · synced to 12 agents`,
    status:    'daemon · FSEventStream · 12-agent fanout',
    profile:   session && session.token ? 'authenticated · bearer token' : 'anonymous · read + assess',
  }[view];

  // Honest data-source banner: live catalog vs curated sample.
  const banner = (() => {
    if (!health.probed) return null;
    const on = live && health.healthy;
    const endpoint = health.endpoint ? health.endpoint.replace(/^https?:\/\//, '') : '';
    const assessing = assessProgress && assessProgress.done < assessProgress.total;
    const graded = assessProgress ? `${assessProgress.done}/${assessProgress.total} assessed` : 'assessments run on open';
    const msg = on
      ? assessing
        ? `Live · assessing canonical catalog from ${endpoint} · ${graded}…`
        : `Live · ${liveMeta ? liveMeta.withSkillMd : skillCount} canonical skills from ${endpoint} · ${assessProgress ? 'all graded' : 'grades run on open'}`
      : health.healthy
        ? 'Live server reachable · canonical catalog empty · showing curated sample'
        : 'Sample data · skillpack-server not reachable — grades and dimensions are illustrative';
    return (
      <div role="status" style={{
        display: 'flex', alignItems: 'center', gap: 9, padding: '8px 28px',
        background: on ? 'color-mix(in oklab, var(--ck-accent) 8%, transparent)' : 'color-mix(in oklab, var(--ck-fg-mute) 12%, transparent)',
        borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 20%, transparent)',
        font: "600 10.5px 'JetBrains Mono', monospace", letterSpacing: '.03em',
        color: on ? 'var(--ck-fg-1)' : 'var(--ck-fg-2)',
      }}>
        <span aria-hidden="true" style={{ color: on ? 'var(--ck-accent)' : 'var(--ck-fg-mute)' }}>{on ? '⊢' : '◌'}</span>
        {msg}
      </div>
    );
  })();

  if (phase === 'boot') return <Splash health={health} />;
  if (phase === 'auth') return <Auth health={health} onEnter={enterConsole} />;

  return (
    <div style={{ display: 'flex', minHeight: '100vh', background: 'var(--ck-bg-0)', color: 'var(--ck-fg-1)' }}>
      <a href="#main" className="ckr-skip">Skip to content</a>
      <Sidebar current={view} onNavigate={setView} health={health} session={session} />
      <main id="main" data-screen-label={`web dashboard · ${view}`} style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        <Header eyebrow={meta.eyebrow} title={meta.title} subtitle={subtitle} theme={theme} onTheme={setTheme} />
        {banner}

        {view === 'overview' && <Overview onNavigate={setView} installedCount={installedCount} skillCount={skillCount} corpus={corpus} />}
        {view === 'registry' && (
          <RegistryBrowser
            skills={filtered} registries={REGISTRIES}
            query={query} setQuery={setQuery}
            prim={prim} setPrim={setPrim}
            signedOnly={signedOnly} setSignedOnly={setSignedOnly}
            minGrade={minGrade} setMinGrade={setMinGrade}
            reg={reg} setReg={setReg}
            sort={sort} setSort={setSort}
            layout={layout} setLayout={setLayout}
            onSelect={handleSelect}
          />
        )}
        {view === 'quality' && <Quality corpus={corpus} onSelect={handleSelect} />}
        {view === 'discovery' && <Discovery health={health} apiBase={SKR_API} registries={REGISTRIES} />}
        {view === 'migration' && <Migration corpus={corpus} onSelect={handleSelect} />}
        {view === 'skills' && <MySkills corpus={corpus} onSelect={handleSelect} />}
        {view === 'status' && <SyncStatus health={health} meta={liveMeta} corpus={corpus} />}
        {view === 'profile' && <Profile session={session} health={health} theme={theme} onTheme={setTheme} onSignOut={signOut} />}
      </main>
      {selectedLive && (
        <InstallDrawer
          skill={selectedLive}
          onClose={() => setSelected(null)}
          onInstalled={(id) => setInstalled(prev => new Set(prev).add(id))}
        />
      )}
    </div>
  );
};

export default App;
