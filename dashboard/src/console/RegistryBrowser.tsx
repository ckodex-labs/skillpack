// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Badge, CkButton, GradeChip, GalRail, TrustMark, Pagination, SectionLabel } from './primitives';
import { PRIMITIVES } from './lib/registry-data';
// ============================================================
// SkillPack Registry · Browser view (controlled by App)
// Search · registry source · facets · sort · list/grid · results.
// ============================================================
const GRADE_ORDER = { 'F': 0, 'D': 1, 'C': 2, 'B': 3, 'A': 4, 'S+': 5 };

// ---- Search field ----
const SearchField = ({ query, setQuery, onClear }) => (
  <div className="ckr-input" style={{
    display: 'flex', alignItems: 'center', gap: 12, padding: '0 16px',
    background: 'var(--ck-bg-1)', clipPath: chamfer(10),
    boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 45%, transparent)',
    height: 52,
  }}>
    <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 16px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⌕</span>
    <input
      type="search" value={query} onChange={(e) => setQuery(e.target.value)}
      placeholder="Search skills by name, publisher, or capability…"
      aria-label="Search the skill registry"
      style={{
        flex: 1, border: 'none', outline: 'none', background: 'transparent',
        color: 'var(--ck-fg-1)', font: "400 15px 'Geist', sans-serif", minWidth: 0,
      }} />
    <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', letterSpacing: '.06em', whiteSpace: 'nowrap' }}>GET /api/v1/skills/search</span>
    {query && (
      <button onClick={onClear} aria-label="Clear search" className="ckr-focusring" style={{
        border: 'none', background: 'transparent', cursor: 'pointer',
        color: 'var(--ck-fg-3)', font: "500 14px 'JetBrains Mono', monospace", padding: 4,
      }}>×</button>
    )}
  </div>
);

// ---- Chip (toggle facet) ----
const Chip = ({ pressed, onClick, children, glyph, tone }) => (
  <button onClick={onClick} aria-pressed={pressed} className="ckr-chip ckr-focusring" style={{
    display: 'inline-flex', alignItems: 'center', gap: 6, padding: '7px 12px',
    border: 'none', cursor: 'pointer', clipPath: chamfer(6),
    font: "600 11px 'Geist', sans-serif", letterSpacing: '.04em',
    background: pressed ? (tone || 'var(--ck-accent)') : 'transparent',
    color: pressed ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)',
    boxShadow: pressed ? 'none' : 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)',
    whiteSpace: 'nowrap',
  }}>
    {glyph && <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 12px 'JetBrains Mono', monospace" }}>{glyph}</span>}
    {children}
  </button>
);

const RegistrySource = ({ registries, reg, setReg }) => (
  <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 14 }}>
    <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 12 }}>Registry source</div>
    <div role="radiogroup" aria-label="Registry source" style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
      {[{ id: 'all', label: 'All registries', host: 'union of indexed sources', trust: 'trusted', note: '' }, ...registries].map(r => {
        const active = reg === r.id;
        const trustGlyph = r.trust === 'trusted' ? '⊢' : r.trust === 'review' ? '⚠' : '◌';
        const trustKind = r.trust === 'trusted' ? 'var(--ck-accent)' : r.trust === 'review' ? 'var(--ck-deny)' : 'var(--ck-fg-mute)';
        return (
          <button key={r.id} role="radio" aria-checked={active} onClick={() => setReg(r.id)}
            className="ckr-row ckr-focusring" style={{
              display: 'flex', alignItems: 'center', gap: 10, padding: '9px 10px', textAlign: 'left',
              border: 'none', cursor: 'pointer', clipPath: chamfer(6),
              background: active ? 'var(--ck-bg-2)' : 'transparent',
              boxShadow: active ? 'inset 0 0 0 1.5px var(--ck-stroke)' : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 18%, transparent)',
            }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 13px 'JetBrains Mono', monospace", color: trustKind, width: 14, textAlign: 'center' }}>{trustGlyph}</span>
            <span style={{ flex: 1, minWidth: 0 }}>
              <span style={{ display: 'block', font: "600 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>{r.label}</span>
              <span style={{ display: 'block', font: "400 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', letterSpacing: '.02em', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{r.host}{r.note ? ` · ${r.note}` : ''}</span>
            </span>
          </button>
        );
      })}
    </div>
  </div>
);

// ---- Result card ----
const ResultCard = ({ s, layout, onSelect }) => {
  const prim = PRIMITIVES.find(p => p.id === s.primitive) || {};
  const trust = s.signed === 'untrusted' ? <TrustMark state="untrusted" />
    : s.signed === 'unsigned' ? <TrustMark state="unsigned" />
    : s.signed === 'unknown' ? <TrustMark state="unknown" />
    : s.provenance ? <TrustMark state="signed" /> : <TrustMark state="signed" />;
  const isGrid = layout === 'grid';
  return (
    <article className="ckr-bento" tabIndex={0} role="button"
      aria-label={`${s.name}${s.version ? ` version ${s.version}` : ''} by ${s.publisher}, grade ${s.grade === '—' ? 'unrated' : s.grade}. Open install panel.`}
      onClick={() => onSelect(s)}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(s); } }}
      style={{
        background: 'var(--ck-bg-1)', clipPath: chamfer(10),
        boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)',
        padding: isGrid ? 18 : '16px 18px', cursor: 'pointer',
        display: isGrid ? 'block' : 'grid',
        gridTemplateColumns: isGrid ? undefined : '1fr auto',
        gap: 16, transition: 'box-shadow .12s ease, transform .12s ease',
      }}
      onMouseEnter={(e) => { e.currentTarget.style.boxShadow = 'inset 0 0 0 2px var(--ck-stroke)'; }}
      onMouseLeave={(e) => { e.currentTarget.style.boxShadow = 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)'; }}
    >
      <div style={{ minWidth: 0 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 8, flexWrap: 'wrap' }}>
          <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 16px 'JetBrains Mono', monospace", color: 'var(--ck-accent)', lineHeight: 1 }}>{prim.glyph}</span>
          <span style={{ font: "700 15px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.01em' }}>{s.name}</span>
          {s.version && <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>v{s.version}</span>}
          <span aria-hidden="true" style={{ color: 'var(--ck-fg-mute)' }}>·</span>
          <span style={{ font: "400 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{s.publisher}</span>
          {s.status && <span className="ckr-badge" style={{ font: "600 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.06em', textTransform: 'uppercase', padding: '2px 7px', boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-fg-mute) 40%, transparent)', clipPath: chamfer(4) }}>{s.status}</span>}
        </div>
        <p style={{ margin: '0 0 12px', font: "400 12.5px 'Geist', sans-serif", color: 'var(--ck-fg-2)', lineHeight: 1.5, maxWidth: 560 }}>{s.synopsis}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
          {prim.label && <Badge kind="kernel" title={`${prim.label} primitive`}><span className="ckr-glyph" aria-hidden="true">{prim.glyph}</span>{prim.label}</Badge>}
          {trust}
          {s.downloads != null && <Badge kind="mute" title={`${s.downloads.toLocaleString()} installs`}>↓ {s.downloads >= 1000 ? (s.downloads / 1000).toFixed(1) + 'k' : s.downloads}</Badge>}
        </div>
      </div>
      <div style={{
        display: 'flex', flexDirection: isGrid ? 'row' : 'column', alignItems: isGrid ? 'center' : 'flex-end',
        gap: isGrid ? 12 : 10, marginTop: isGrid ? 16 : 0,
        justifyContent: isGrid ? 'space-between' : 'flex-start',
        borderTop: isGrid ? '1px solid color-mix(in oklab, var(--ck-stroke) 20%, transparent)' : 'none',
        paddingTop: isGrid ? 14 : 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <GradeChip grade={s.grade} score={s.score} />
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <GalRail level={s.gal || 0} attested={s.signed === 'signed' && s.provenance} />
            <span style={{ font: "600 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.04em' }}>
              {s.gal != null ? `GAL-${s.gal} · ${s.tier}`
                : s._assessing ? 'assessing…'
                : (s._assessed && s.score != null) ? `score ${s.score} / 120`
                : s._live ? 'open to assess' : `GAL-${s.gal} · ${s.tier}`}
            </span>
          </div>
        </div>
        <CkButton variant={s.installed ? 'secondary' : 'primary'} onClick={(e) => { e.stopPropagation(); onSelect(s); }}>
          {s.installed ? '⊢ Installed' : s._live ? '⊳ Open' : '↧ Install'}
        </CkButton>
      </div>
    </article>
  );
};

const REG_PAGE_SIZE = 24;

const RegistryBrowser = ({
  skills, registries, query, setQuery, prim, setPrim, signedOnly, setSignedOnly,
  minGrade, setMinGrade, reg, setReg, sort, setSort, layout, setLayout, onSelect,
}) => {
  const grades = ['F', 'D', 'C', 'B', 'A', 'S+'];
  const [page, setPage] = React.useState(1);
  // Reset to the first page whenever the result set changes.
  React.useEffect(() => { setPage(1); }, [query, prim, signedOnly, minGrade, reg, sort]);
  const pageCount = Math.max(1, Math.ceil(skills.length / REG_PAGE_SIZE));
  const current = Math.min(page, pageCount);
  const pageItems = skills.slice((current - 1) * REG_PAGE_SIZE, current * REG_PAGE_SIZE);
  return (
    <div style={{ padding: '24px 28px', display: 'grid', gridTemplateColumns: '248px 1fr', gap: 22, alignItems: 'start' }}>
      {/* Left rail · registry source + facets */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 16, position: 'sticky', top: 96 }}>
        <RegistrySource registries={registries} reg={reg} setReg={setReg} />
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 14 }}>
          <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 12 }}>Lifecycle primitive</div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            <Chip pressed={prim === 'all'} onClick={() => setPrim('all')}>All</Chip>
            {PRIMITIVES.map(p => (
              <Chip key={p.id} pressed={prim === p.id} onClick={() => setPrim(p.id)} glyph={p.glyph}>{p.label}</Chip>
            ))}
          </div>
          <div style={{ height: 1, background: 'color-mix(in oklab, var(--ck-stroke) 20%, transparent)', margin: '16px 0' }} />
          <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 12 }}>Minimum grade</div>
          <div style={{ display: 'flex', gap: 4 }}>
            {grades.map(g => (
              <button key={g} onClick={() => setMinGrade(minGrade === g ? null : g)} aria-pressed={minGrade === g}
                className="ckr-chip ckr-focusring" title={`Grade ${g} and above`} style={{
                  flex: 1, padding: '7px 0', border: 'none', cursor: 'pointer', clipPath: chamfer(4),
                  font: "800 12px 'Geist', sans-serif",
                  background: minGrade === g ? 'var(--ck-accent)' : 'transparent',
                  color: minGrade === g ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)',
                  boxShadow: minGrade === g ? 'none' : 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 35%, transparent)',
                }}>{g}</button>
            ))}
          </div>
          <div style={{ height: 1, background: 'color-mix(in oklab, var(--ck-stroke) 20%, transparent)', margin: '16px 0' }} />
          <Chip pressed={signedOnly} onClick={() => setSignedOnly(!signedOnly)} glyph="⊢" tone="var(--ck-accent)">Cosign verified only</Chip>
        </div>
      </div>

      {/* Right · search + toolbar + results */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 16, minWidth: 0 }}>
        <SearchField query={query} setQuery={setQuery} onClear={() => setQuery('')} />

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
          <span aria-live="polite" style={{ font: "600 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}>
            {skills.length} result{skills.length === 1 ? '' : 's'}
          </span>
          <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 12 }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: 8, font: "600 10px 'Geist', sans-serif", letterSpacing: '.1em', textTransform: 'uppercase', color: 'var(--ck-fg-3)' }}>
              Sort
              <select value={sort} onChange={(e) => setSort(e.target.value)} aria-label="Sort results"
                className="ckr-input ckr-focusring" style={{
                  border: 'none', background: 'var(--ck-bg-1)', color: 'var(--ck-fg-1)',
                  font: "600 11px 'JetBrains Mono', monospace", padding: '7px 10px', clipPath: chamfer(5),
                  boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)', cursor: 'pointer',
                }}>
                <option value="relevance">relevance</option>
                <option value="grade">grade ↓</option>
                <option value="downloads">installs ↓</option>
                <option value="updated">recently updated</option>
              </select>
            </label>
            <div role="group" aria-label="Layout" style={{ display: 'flex', gap: 2, padding: 2, clipPath: chamfer(5), background: 'color-mix(in oklab, var(--ck-stroke) 22%, transparent)' }}>
              {[{ id: 'list', g: '≣' }, { id: 'grid', g: '⊞' }].map(l => (
                <button key={l.id} onClick={() => setLayout(l.id)} aria-pressed={layout === l.id} aria-selected={layout === l.id}
                  aria-label={`${l.id} layout`} className="ckr-seg ckr-focusring" style={{
                    border: 'none', cursor: 'pointer', padding: '6px 10px', clipPath: chamfer(3),
                    font: "500 13px 'JetBrains Mono', monospace",
                    background: layout === l.id ? 'var(--ck-accent)' : 'var(--ck-bg-0)',
                    color: layout === l.id ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)',
                  }}><span className="ckr-glyph" aria-hidden="true">{l.g}</span></button>
              ))}
            </div>
          </div>
        </div>

        {reg === 'partner-atlas' && (
          <div className="ckr-row" role="status" style={{
            display: 'flex', alignItems: 'center', gap: 10, padding: '12px 14px', clipPath: chamfer(8),
            boxShadow: 'inset 0 0 0 1.5px var(--ck-deny)', background: 'color-mix(in oklab, var(--ck-deny) 8%, transparent)',
          }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>⚠</span>
            <span style={{ font: "500 12px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>
              <strong style={{ color: 'var(--ck-deny)' }}>SKILLPACK_SEC_UNTRUSTED_REGISTRY</strong> — atlas.partner is not in the trust list. Skills install in review mode; signatures must be verified manually.
            </span>
          </div>
        )}

        {skills.length === 0 ? (
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 30%, transparent)', padding: 48, textAlign: 'center' }}>
            <div className="ckr-glyph" aria-hidden="true" style={{ font: "400 28px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginBottom: 12 }}>◌</div>
            <div style={{ font: "700 15px 'Geist', sans-serif", color: 'var(--ck-fg-1)', marginBottom: 6 }}>No skills match</div>
            <div style={{ font: "400 12px 'Geist', sans-serif", color: 'var(--ck-fg-3)' }}>Loosen the grade floor, clear the cosign filter, or widen the registry source.</div>
          </div>
        ) : (
          <React.Fragment>
            <div style={layout === 'grid'
              ? { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 14 }
              : { display: 'flex', flexDirection: 'column', gap: 12 }}>
              {pageItems.map(s => <ResultCard key={s.id} s={s} layout={layout} onSelect={onSelect} />)}
            </div>
            <Pagination page={current} pageCount={pageCount} total={skills.length} pageSize={REG_PAGE_SIZE} onPage={setPage} label="skills" />
          </React.Fragment>
        )}
      </div>
    </div>
  );
};

export { RegistryBrowser, GRADE_ORDER };
