// ============================================================
// SkillPack Registry · Evolve page (re-assess a store skill)
// Pick a real canonical skill → re-run the LIVE 9-dimension
// assessment (window.skrAssess) → read the real grade / dims /
// issues → open it in the registry drawer.
//
// HONEST SCOPE: the only real operation here is the live
// re-assessment and the grade/dimensions/issues it returns.
// Sandbox spawn, dry-run diff and promotion are NOT performed —
// the "evolve pipeline" (migrate → promote → supersede) is shown
// as a static capability preview, never as work that occurred.
// ============================================================
const { useState: _useStateM } = React;

const PIPELINE = [
  { id: 'load',   glyph: '⌖', title: 'Load skill',         detail: 'select a canonical store skill' },
  { id: 'assess', glyph: '⊢', title: 'Re-assess (live)',   detail: 'skrAssess · schema + 9-dimension' },
  { id: 'result', glyph: '▤', title: '9-dimension result', detail: 'real grade · dims · issue count' },
  { id: 'open',   glyph: '⇗', title: 'Open in registry',   detail: 'inspect the live detail drawer' },
];

// Static, honest description of the broader evolve capability. This is a
// preview of what the evolve primitive is designed to do — it is NOT a live
// sandbox and none of these rows reflect a run that happened.
const EVOLVE_CAP = [
  { k: 'live now',   v: 're-assess', tone: 'attested' },
  { k: 'sandbox',    v: 'preview',   tone: 'mute' },
  { k: 'dry-run diff', v: 'preview', tone: 'mute' },
  { k: 'promote/supersede', v: 'preview', tone: 'mute' },
];

const Migration = ({ corpus, onSelect }) => {
  const skills = Array.isArray(corpus) ? corpus : [];
  const [query, setQuery] = _useStateM('');
  const [selectedId, setSelectedId] = _useStateM(skills[0] ? skills[0].id : null);
  const [phase, setPhase] = _useStateM('idle');   // idle | running | done | error
  const [result, setResult] = _useStateM(null);   // real skrAssess patch

  const selected = skills.find(s => s.id === selectedId) || null;

  const filtered = (() => {
    const q = query.trim().toLowerCase();
    const base = q ? skills.filter(s => (s.name || '').toLowerCase().includes(q)) : skills;
    return base.slice(0, 12);
  })();

  const pick = (id) => { setSelectedId(id); setPhase('idle'); setResult(null); };

  const run = async () => {
    if (!selected || typeof window.skrAssess !== 'function') return;
    setPhase('running'); setResult(null);
    const patch = await window.skrAssess(selected);
    if (patch) { setResult(patch); setPhase('done'); }
    else { setResult(null); setPhase('error'); }
  };

  // pipeline index states, driven by the REAL async phase
  const stepState = (i) => {
    if (phase === 'idle') return i === 0 ? 'ready' : 'pending';
    if (phase === 'running') { if (i === 0) return 'done'; if (i === 1) return 'active'; return 'pending'; }
    if (phase === 'error')   { if (i === 0) return 'done'; if (i === 1) return 'error'; return 'pending'; }
    // done
    if (i <= 2) return 'done';
    return 'ready';
  };

  const issueCount = result ? (Array.isArray(result.issues) ? result.issues.length : (result.issues || 0)) : 0;

  return (
    <div style={{ padding: '24px 28px', display: 'grid', gridTemplateColumns: '300px 1fr', gap: 22, alignItems: 'start' }}>
      {/* LEFT · capability preview + pipeline */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18, position: 'sticky', top: 96 }}>
        <div className="ckr-bento" style={{ background: 'color-mix(in oklab, var(--ck-witness) 8%, var(--ck-bg-1))', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-witness) 50%, transparent)', padding: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: 'var(--ck-witness)' }}>◳</span>
            <SectionLabel style={{ color: 'var(--ck-witness)' }}>Evolve capability</SectionLabel>
            <span style={{ marginLeft: 'auto' }}>
              <Badge kind="witness" title="Capability preview — not a live sandbox">◌ preview</Badge>
            </span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 7 }}>
            {EVOLVE_CAP.map(row => (
              <div key={row.k} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ flex: 1, font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.02em' }}>{row.k}</span>
                <span style={{ font: "600 10px 'JetBrains Mono', monospace", color: row.tone === 'attested' ? 'var(--ck-accent)' : 'var(--ck-fg-mute)' }}>{row.v}</span>
              </div>
            ))}
          </div>
          <div style={{ font: "400 10px 'Geist', sans-serif", color: 'var(--ck-fg-mute)', marginTop: 12, lineHeight: 1.5 }}>
            Only re-assessment is live. Sandboxed migration and promotion are a preview of the evolve primitive, not operations performed here.
          </div>
        </div>

        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 16 }}>
          <SectionLabel style={{ marginBottom: 14 }}>Pipeline</SectionLabel>
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {PIPELINE.map((p, i) => {
              const st = stepState(i);
              const tone = st === 'done' ? 'var(--ck-link)'
                : st === 'active' ? 'var(--ck-accent)'
                : st === 'error' ? 'var(--ck-deny)'
                : st === 'ready' ? 'var(--ck-fg-2)' : 'var(--ck-fg-mute)';
              const glyph = st === 'done' ? '⊢' : st === 'error' ? '⊗' : p.glyph;
              const lineDone = st === 'done';
              return (
                <div key={p.id} style={{ display: 'flex', gap: 10, alignItems: 'flex-start', position: 'relative', paddingBottom: i < PIPELINE.length - 1 ? 14 : 0 }}>
                  {i < PIPELINE.length - 1 && <span aria-hidden="true" style={{ position: 'absolute', left: 9, top: 20, bottom: 0, width: 1.5, background: lineDone ? 'var(--ck-link)' : 'color-mix(in oklab, var(--ck-stroke) 25%, transparent)' }} />}
                  <span className={st === 'active' ? 'ckr-glyph ckr-spin' : 'ckr-glyph'} aria-hidden="true" style={{ font: "600 12px 'JetBrains Mono', monospace", color: tone, width: 19, height: 19, display: 'grid', placeItems: 'center', flexShrink: 0, background: 'var(--ck-bg-1)', clipPath: chamfer(4), boxShadow: `inset 0 0 0 1.5px ${tone}`, zIndex: 1 }}>{glyph}</span>
                  <div style={{ flex: 1, opacity: (st === 'pending') ? 0.5 : 1 }}>
                    <div style={{ font: "600 11.5px 'JetBrains Mono', monospace", color: st === 'pending' ? 'var(--ck-fg-3)' : 'var(--ck-fg-1)' }}>{p.title}</div>
                    <div style={{ font: "400 9.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 2 }}>{p.detail}</div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* RIGHT · source selector + live result */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18, minWidth: 0 }}>
        {/* source selector · real store skills */}
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 12 }}>
            <SectionLabel>Select a store skill to re-assess</SectionLabel>
            <span style={{ marginLeft: 'auto' }}><Badge kind="mute" title="Canonical skills in corpus">{skills.length} skills</Badge></span>
          </div>
          <input
            type="text" value={query} onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter by name…" aria-label="Filter skills by name"
            className="ckr-focusring"
            style={{
              width: '100%', boxSizing: 'border-box', padding: '9px 12px', marginBottom: 10,
              font: "500 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)',
              background: 'var(--ck-bg-2)', border: 'none', clipPath: chamfer(6),
              boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 30%, transparent)',
            }}
          />
          <div role="radiogroup" aria-label="Store skill" style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 8 }}>
            {filtered.map(s => {
              const on = selectedId === s.id;
              return (
                <button key={s.id} role="radio" aria-checked={on} onClick={() => pick(s.id)} className="ckr-chip ckr-focusring" title={s.synopsis || s.name} style={{
                  textAlign: 'left', padding: '10px 12px', border: 'none', cursor: 'pointer', clipPath: chamfer(7),
                  display: 'flex', alignItems: 'center', gap: 9,
                  background: on ? 'var(--ck-bg-2)' : 'transparent',
                  boxShadow: on ? 'inset 0 0 0 1.5px var(--ck-stroke)' : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 22%, transparent)',
                }}>
                  <GradeChip grade={s.grade || '—'} score={s.score} />
                  <span style={{ minWidth: 0, font: "700 12px 'Geist', sans-serif", color: 'var(--ck-fg-1)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{s.name}</span>
                </button>
              );
            })}
            {filtered.length === 0 && (
              <div style={{ gridColumn: '1 / -1', font: "400 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', padding: '8px 2px' }}>No skills match “{query}”.</div>
            )}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 14 }}>
            <CkButton variant={phase === 'running' ? 'secondary' : 'primary'} onClick={run} disabled={phase === 'running' || !selected}>
              {phase === 'running' ? '⊢ Assessing…' : phase === 'idle' ? '⊢ Re-assess (live)' : '↻ Re-assess again'}
            </CkButton>
            {selected && <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>target · {selected.name}</span>}
            <span style={{ marginLeft: 'auto', font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>skrAssess · /skills/assess</span>
          </div>
        </div>

        {/* live assessment result */}
        {phase === 'error' && (
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px var(--ck-deny)', padding: 16, display: 'flex', alignItems: 'center', gap: 10 }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>⊗</span>
            <span style={{ font: "500 11.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>Live assessment unavailable — the assessment service did not return a result. No grade was produced.</span>
          </div>
        )}

        {phase === 'done' && result && (
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
              <SectionLabel>Live re-assessment · {selected ? selected.name : ''}</SectionLabel>
              <Badge kind="attested" title="Real assessment result">⊢ live</Badge>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 16, flexWrap: 'wrap', marginBottom: 16 }}>
              <GradeChip grade={result.grade || '—'} score={result.score} size="lg" />
              <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                {result.score != null && <Badge kind="teal" title="Total score">score {result.score}</Badge>}
                {result.profile && <Badge kind="mute" title="Assessment profile">profile {result.profile}</Badge>}
                <Badge kind={issueCount ? 'witness' : 'attested'} title="Issues from assessment">⚑ {issueCount} issue{issueCount === 1 ? '' : 's'}</Badge>
              </div>
              <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 10 }}>
                <CkButton variant="ghost" disabled title="Promotion is a capability preview — not performed here">⇗ Promote (preview)</CkButton>
                <CkButton variant="primary" onClick={() => selected && onSelect && onSelect(selected)}>⇗ Open in registry</CkButton>
              </div>
            </div>
            <SectionLabel style={{ marginBottom: 10 }}>9-dimension breakdown</SectionLabel>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 9 }}>
              {DIMS.map(dim => (
                <DimMeter key={dim} label={dim} score={result.dims ? result.dims[dim] : null} />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

Object.assign(window, { Migration });
