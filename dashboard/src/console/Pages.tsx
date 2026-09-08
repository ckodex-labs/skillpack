// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Badge, CkButton, GradeChip, GalRail, DimMeter, SectionLabel, StatTile, gradeMeta } from './primitives';
import { PRIMITIVES, SKILLS, ENDPOINTS, MIGRATION_SOURCES, DIMS } from './lib/registry-data';
// ============================================================
// SkillPack Registry · Overview · My skills · Sync & status
// ============================================================

// ---------- OVERVIEW ----------
const Overview = ({ onNavigate, installedCount, skillCount, corpus }) => {
  const live = corpus && corpus.some(s => s._live);
  const graded = (corpus || []).filter(s => s._assessed && s.score != null);
  const byPrim = PRIMITIVES.map(p => ({ ...p, n: SKILLS.filter(s => s.primitive === p.id).length }));
  const maxN = Math.max(...byPrim.map(p => p.n), 1);
  const tone = { attested: 'var(--ck-accent)', witness: 'var(--ck-witness)', teal: 'var(--ck-link)', deny: 'var(--ck-deny)' };

  // Real grade spread (live mode) for the right-hand panel.
  const spreadGrades = ['S+', 'A', 'B', 'C', 'D', 'F'];
  const spread = spreadGrades.map(g => ({ g, n: graded.filter(s => s.grade === g).length }));
  const maxSpread = Math.max(1, ...spread.map(s => s.n));
  const spreadTone = (g) => (gradeMeta[g] ? (g === 'S+' ? 'var(--ck-accent)' : gradeMeta[g].ring) : 'var(--ck-fg-mute)');

  return (
    <div style={{ padding: '24px 28px', display: 'flex', flexDirection: 'column', gap: 22 }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 14 }}>
        <StatTile glyph="⊞" label="Skills indexed" value={skillCount != null ? skillCount : SKILLS.length} />
        <StatTile glyph="⌖" label="Registries" value={ENDPOINTS.length} tone="var(--ck-link)" />
        {live
          ? <StatTile glyph="◆" label="Assessed" value={graded.length} tone="var(--ck-witness)" />
          : <StatTile glyph="▰" label="Installed" value={installedCount} tone="var(--ck-witness)" />}
        <StatTile glyph="⇄" label="Migration adapters" value={MIGRATION_SOURCES.length} tone="var(--ck-text-role)" />
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1.3fr 1fr', gap: 22, alignItems: 'start' }}>
        {/* activity */}
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 14 }}>
            <SectionLabel>Recent assessments</SectionLabel>
            <span style={{ marginLeft: 'auto', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>from the live catalog</span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {graded
              .slice()
              .sort((a, b) => a.score - b.score)
              .slice(0, 8)
              .map((s, i) => {
                const gm = gradeMeta[s.grade] || {};
                const gColor = s.grade === 'S+' ? 'var(--ck-accent)' : (gm.ring || 'var(--ck-fg-mute)');
                const gGlyph = (s.grade === 'A' || s.grade === 'S+') ? '◆' : s.grade === 'F' ? '⊘' : '○';
                const right = s.profile || 'assessed';
                return (
                  <div key={s.id || i} className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '11px 10px', clipPath: chamfer(6), boxShadow: i % 2 ? 'none' : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 12%, transparent)' }}>
                    <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 13px 'JetBrains Mono', monospace", color: gColor, width: 16, textAlign: 'center' }}>{gGlyph}</span>
                    <span style={{ flex: 1, minWidth: 0 }}>
                      <span style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span style={{ font: "700 12px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{s.name}</span>
                        <Badge kind={s.grade === 'F' ? 'deny' : (s.grade === 'S+' || s.grade === 'A') ? 'attested' : 'mute'}>{s.grade}</Badge>
                      </span>
                      <span style={{ display: 'block', font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 3 }}>score {s.score} / 120</span>
                    </span>
                    <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>{right}</span>
                  </div>
                );
              })}
          </div>
        </div>

        {/* right column */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
            <SectionLabel style={{ marginBottom: 14 }}>{live ? `Grade spread · ${graded.length} assessed` : 'Index by primitive'}</SectionLabel>
            {live ? (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 9 }}>
                {spread.map(({ g, n }) => (
                  <div key={g} style={{ display: 'grid', gridTemplateColumns: '30px 1fr 22px', alignItems: 'center', gap: 10 }}>
                    <span style={{ font: "800 13px 'Geist', sans-serif", color: spreadTone(g), textAlign: 'center' }}>{g}</span>
                    <span className="ckr-rail" style={{ height: 8, background: 'color-mix(in oklab, var(--ck-fg-mute) 26%, transparent)', clipPath: chamfer(2), position: 'relative', display: 'block' }}>
                      <span style={{ position: 'absolute', inset: 0, width: `${(n / maxSpread) * 100}%`, background: spreadTone(g), opacity: g === 'S+' || g === 'F' ? 1 : 0.85 }} />
                    </span>
                    <span style={{ font: "600 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)', textAlign: 'right' }}>{n}</span>
                  </div>
                ))}
                <button onClick={() => onNavigate('quality')} className="ckr-focusring" style={{ marginTop: 6, alignSelf: 'flex-start', border: 'none', background: 'transparent', cursor: 'pointer', color: 'var(--ck-link)', font: "600 10px 'JetBrains Mono', monospace", padding: 2 }}>◆ Open fleet assurance →</button>
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 9 }}>
                {byPrim.map(p => (
                  <div key={p.id} style={{ display: 'grid', gridTemplateColumns: '92px 1fr 18px', alignItems: 'center', gap: 10 }}>
                    <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}><span className="ckr-glyph" aria-hidden="true" style={{ color: 'var(--ck-accent)', marginRight: 6 }}>{p.glyph}</span>{p.label}</span>
                    <span className="ckr-rail" style={{ height: 6, background: 'color-mix(in oklab, var(--ck-fg-mute) 28%, transparent)', clipPath: chamfer(2), position: 'relative', display: 'block' }}>
                      <span style={{ position: 'absolute', inset: 0, width: `${(p.n / maxN) * 100}%`, background: 'var(--ck-stroke)' }} />
                    </span>
                    <span style={{ font: "600 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)', textAlign: 'right' }}>{p.n}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
            <SectionLabel style={{ marginBottom: 12 }}>Jump to</SectionLabel>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
              <CkButton variant="secondary" onClick={() => onNavigate('registry')}>⊞ Browse registry</CkButton>
              <CkButton variant="secondary" onClick={() => onNavigate('discovery')}>⌖ Discovery</CkButton>
              <CkButton variant="secondary" onClick={() => onNavigate('migration')}>⇄ Migrate a skill</CkButton>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// ---------- MY SKILLS ----------
// The local store IS the canonical store (~/Skills/shared). When wired to the
// live daemon, every canonical skill is a local skill — so show them all with
// their real grade + profile. Falls back to the "installed" set otherwise.
const profileLabel = (p) => ((p || '').toLowerCase().includes('cnsb') ? '⊟ cnsb rubric' : '◆ agentskills rubric');

const MY_PAGE_SIZE = 24;
const MySkills = ({ corpus, onSelect }) => {
  const live = corpus.some(s => s._live);
  const mine = live ? corpus : corpus.filter(s => s.installed);
  const graded = mine.filter(s => s._assessed && s.score != null);
  const avg = graded.length ? graded.reduce((a, s) => a + s.score, 0) / graded.length : null;
  const [page, setPage] = React.useState(1);
  const pageCount = Math.max(1, Math.ceil(mine.length / MY_PAGE_SIZE));
  const current = Math.min(page, pageCount);
  const pageItems = mine.slice((current - 1) * MY_PAGE_SIZE, current * MY_PAGE_SIZE);
  return (
    <div style={{ padding: '24px 28px' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 16, flexWrap: 'wrap' }}>
        <Badge kind="attested">▰ {mine.length} in local store</Badge>
        <Badge kind="witness">⊛ ~/Skills/shared</Badge>
        {avg != null && <Badge kind="mute">avg score {avg.toFixed(1)} / 120</Badge>}
        <span style={{ marginLeft: 'auto', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>canonical store</span>
      </div>
      {mine.length === 0 ? (
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 30%, transparent)', padding: 48, textAlign: 'center' }}>
          <div className="ckr-glyph" aria-hidden="true" style={{ font: "400 28px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginBottom: 12 }}>◌</div>
          <div style={{ font: "700 15px 'Geist', sans-serif", color: 'var(--ck-fg-1)', marginBottom: 6 }}>No skills in the local store</div>
          <div style={{ font: "400 12px 'Geist', sans-serif", color: 'var(--ck-fg-3)' }}>Install from the registry to populate your canonical store.</div>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 14 }}>
          {pageItems.map(s => (
            <article key={s.id} className="ckr-bento" tabIndex={0} role="button" onClick={() => onSelect(s)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(s); } }}
              aria-label={`${s.name}, grade ${s.grade === '—' ? 'unrated' : s.grade}. Open detail.`} style={{
                background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)', padding: 16, cursor: 'pointer',
              }}>
              <div style={{ display: 'flex', alignItems: 'flex-start', gap: 10, marginBottom: 10 }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ font: "700 14px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{s.name}</div>
                  <div style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{s.status || 'local'}{s.version ? ` · v${s.version}` : ''}</div>
                </div>
                <GradeChip grade={s.grade} score={s.score} />
              </div>
              <p style={{ margin: '0 0 12px', font: "400 11.5px 'Geist', sans-serif", color: 'var(--ck-fg-2)', lineHeight: 1.5, maxHeight: 34, overflow: 'hidden' }}>{s.synopsis}</p>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 8 }}>
                <Badge kind="mute" title="Assessment rubric applied">{profileLabel(s.profile)}</Badge>
                <span style={{ font: "600 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', whiteSpace: 'nowrap' }}>{s.score != null ? `${s.score} / 120` : 'unrated'}</span>
              </div>
            </article>
          ))}
        </div>
        <Pagination page={current} pageCount={pageCount} total={mine.length} pageSize={MY_PAGE_SIZE} onPage={setPage} label="skills" />
        </div>
      )}
    </div>
  );
};

// ---------- SYNC & STATUS ----------
const SyncStatus = ({ health, meta, corpus }) => {
  health = health || {};
  corpus = corpus || [];
  const endpointHost = (health.endpoint || '').replace(/^https?:\/\//, '');
  const rootPath = meta && meta.root ? meta.root : '~/Skills/shared';
  const total = (meta && meta.total != null) ? meta.total : corpus.length;
  const assessed = corpus.filter(s => s._assessed && s.score != null).length;

  const gradeOrder = ['S+', 'A', 'B', 'C', 'D', 'F'];
  const gradeCounts = gradeOrder
    .map(g => ({ g, n: corpus.filter(s => s.grade === g).length }))
    .filter(x => x.n > 0);

  const cards = [
    {
      glyph: '⊢', label: 'Daemon',
      big: health.healthy ? 'healthy' : 'offline',
      sub: `HTTP ${endpointHost} · protocol v${health.protocolVersion || 1}`,
      tone: health.healthy ? 'var(--ck-accent)' : 'var(--ck-text-role)',
    },
    {
      glyph: '≋', label: 'FSEventStream',
      big: 'watching',
      sub: `${rootPath} · auto-sync on change`,
      tone: 'var(--ck-link)',
    },
    {
      glyph: '⊞', label: 'Storage backend',
      big: 'DuckDB',
      sub: 'in-memory index · canonical store on disk',
      tone: 'var(--ck-witness)',
    },
  ];

  const consumption = [
    { k: 'method', v: 'symlink (reference, never copy)' },
    { k: 'command', v: 'skillpack store sync' },
    { k: 'reconcile', v: 'skillpack store sync --dry-run' },
  ];

  return (
    <div style={{ padding: '24px 28px', display: 'flex', flexDirection: 'column', gap: 22 }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 14 }}>
        {cards.map(c => (
          <div key={c.label} className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)', padding: 18 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
              <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 14px 'JetBrains Mono', monospace", color: c.tone }}>{c.glyph}</span>
              <SectionLabel>{c.label}</SectionLabel>
            </div>
            <div style={{ font: "800 22px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.01em' }}>{c.big}</div>
            <div style={{ font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 6 }}>{c.sub}</div>
          </div>
        ))}
      </div>

      <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
          <SectionLabel>Store fanout · reference model</SectionLabel>
          <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{total} skills fanned out</span>
          <span style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>reference fanout · not a live poll</span>
            <CkButton variant="secondary" onClick={() => {}}>≋ Sync now</CkButton>
          </span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 18, alignItems: 'start' }}>
          {/* real counters */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
              <div style={{ padding: '12px', clipPath: chamfer(7), boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 25%, transparent)' }}>
                <div style={{ font: "800 22px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.01em' }}>{total}</div>
                <div style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 4 }}>total skills</div>
              </div>
              <div style={{ padding: '12px', clipPath: chamfer(7), boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 25%, transparent)' }}>
                <div style={{ font: "800 22px 'Geist', sans-serif", color: 'var(--ck-witness)', letterSpacing: '-.01em' }}>{assessed}</div>
                <div style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 4 }}>assessed</div>
              </div>
            </div>
            {gradeCounts.length > 0 && (
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                {gradeCounts.map(({ g, n }) => (
                  <Badge key={g} kind={g === 'F' ? 'deny' : (g === 'S+' || g === 'A') ? 'attested' : 'mute'}>{g} · {n}</Badge>
                ))}
              </div>
            )}
          </div>

          {/* real consumption model */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {consumption.map((row, i) => (
              <div key={row.k} className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '11px 10px', clipPath: chamfer(6), boxShadow: i % 2 ? 'none' : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 12%, transparent)' }}>
                <span style={{ font: "600 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', width: 74, flexShrink: 0 }}>{row.k}</span>
                <span style={{ flex: 1, minWidth: 0, font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>{row.v}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export { Overview, MySkills, SyncStatus };
