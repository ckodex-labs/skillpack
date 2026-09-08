// ============================================================
// SkillPack · VS Code Extension surface (C-02)
// 300px sidebar + editor. Unique powers: auto-assess on save,
// AI-assisted authoring, inline registry install. Feature matrix: most
// ops MUST. Chrome is a generic IDE; SkillPack panels carry CKODEX accents.
// ============================================================

const vsTone = { S: 'var(--ck-fg-3)' };

// activity bar
const VSActivity = () => {
  const icons = [{ g: '▰', on: false }, { g: '⌕', on: false }, { g: '⎇', on: false }, { g: '⊞', on: true }, { g: '⚙', on: false }];
  return (
    <div style={{ width: 52, background: 'color-mix(in oklab, var(--ck-deep-blue) 60%, #000)', display: 'flex', flexDirection: 'column', alignItems: 'center', paddingTop: 10, gap: 4, flexShrink: 0 }}>
      {icons.map((it, i) => (
        <div key={i} style={{ position: 'relative', width: 52, height: 44, display: 'grid', placeItems: 'center' }}>
          {it.on && <span style={{ position: 'absolute', left: 0, top: 8, bottom: 8, width: 2, background: 'var(--ck-accent)' }} />}
          <span className="ckr-glyph" style={{ font: "400 19px 'JetBrains Mono', monospace", color: it.on ? 'var(--ck-fg-1)' : 'var(--ck-fg-mute)' }}>{it.g}</span>
        </div>
      ))}
      <div style={{ marginTop: 'auto', marginBottom: 10 }}><span className="ckr-glyph" style={{ font: "400 18px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>⌂</span></div>
    </div>
  );
};

const TreeRow = ({ depth = 0, glyph, name, grade, badge, badgeKind, dot, active }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 7, padding: '4px 10px', paddingLeft: 10 + depth * 14, background: active ? 'color-mix(in oklab, var(--ck-stroke) 16%, transparent)' : 'transparent', boxShadow: active ? 'inset 2px 0 0 var(--ck-accent)' : 'none' }}>
    <span className="ckr-glyph" aria-hidden="true" style={{ font: "400 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', width: 12 }}>{glyph}</span>
    <span style={{ flex: 1, font: "400 12.5px 'JetBrains Mono', monospace", color: active ? 'var(--ck-fg-1)' : 'var(--ck-fg-2)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{name}{dot && <span style={{ color: 'var(--ck-accent)', marginLeft: 5 }}>●</span>}</span>
    {grade && <GradeChip grade={grade} size="md" />}
    {badge && <Badge kind={badgeKind || 'mute'}>{badge}</Badge>}
  </div>
);

const VSSidebar = () => (
  <div style={{ width: 300, background: 'var(--ck-bg-1)', flexShrink: 0, display: 'flex', flexDirection: 'column', borderRight: '1px solid color-mix(in oklab, var(--ck-stroke) 18%, transparent)', overflow: 'hidden' }}>
    <div style={{ padding: '12px 14px 8px', display: 'flex', alignItems: 'center', gap: 8 }}>
      <span style={{ font: "700 11px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-2)', textTransform: 'uppercase' }}>SkillPack</span>
      <span style={{ marginLeft: 'auto', font: "400 13px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>⋯</span>
    </div>
    {/* workspace skills tree */}
    <div style={{ padding: '6px 0' }}>
      <div style={{ padding: '4px 12px', font: "700 9px 'Geist', sans-serif", letterSpacing: '.12em', color: 'var(--ck-fg-3)', textTransform: 'uppercase' }}>Workspace skills</div>
      <TreeRow glyph="▾" name="skills/" />
      <TreeRow depth={1} glyph="◆" name="writing-skills" grade="S+" />
      <TreeRow depth={1} glyph="◆" name="conformance" grade="A" badge="◆" badgeKind="attested" />
      <TreeRow depth={1} glyph="◆" name="my-formatter" grade="C" dot active />
    </div>
    {/* assessment of open file */}
    <div style={{ padding: '8px 12px', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 16%, transparent)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
        <div style={{ font: "700 9px 'Geist', sans-serif", letterSpacing: '.12em', color: 'var(--ck-fg-3)', textTransform: 'uppercase' }}>Assessment</div>
        <span style={{ marginLeft: 'auto', font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>my-formatter</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 12 }}>
        <GradeChip grade="C" score={72} size="lg" />
        <div>
          <GalRail level={3} width={120} />
          <div style={{ font: "600 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', marginTop: 5 }}>GAL-3 · tier L1 · 2 issues</div>
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 7, marginBottom: 10 }}>
        <DimMeter label="documentation" score={78} />
        <DimMeter label="testing" score={62} />
        <DimMeter label="provenance" score={40} />
      </div>
      <div className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 9, padding: '8px 10px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-accent) 55%, transparent)' }}>
        <span className="ckr-glyph" style={{ font: "600 12px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⟳</span>
        <span style={{ flex: 1, font: "600 11px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>Auto-assess on save</span>
        <span style={{ width: 30, height: 16, background: 'var(--ck-accent)', clipPath: chamfer(3), position: 'relative' }}>
          <span style={{ position: 'absolute', right: 2, top: 2, width: 12, height: 12, background: 'var(--ck-deep-blue)' }} />
        </span>
      </div>
    </div>
    {/* registry */}
    <div style={{ padding: '10px 12px', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 16%, transparent)', marginTop: 'auto' }}>
      <div style={{ font: "700 9px 'Geist', sans-serif", letterSpacing: '.12em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 10 }}>Registry</div>
      <div className="ckr-input" style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 10px', height: 34, background: 'var(--ck-bg-2)', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)', marginBottom: 10 }}>
        <span className="ckr-glyph" style={{ font: "400 12px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⌕</span>
        <span style={{ font: "400 11px 'Geist', sans-serif", color: 'var(--ck-fg-mute)' }}>search skills…</span>
      </div>
      {[{ n: 'proof-audit', g: 'A' }, { n: 'design-lint', g: 'B' }].map(r => (
        <div key={r.n} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 4px' }}>
          <span className="ckr-glyph" style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⊢</span>
          <span style={{ flex: 1, font: "500 11.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>{r.n}</span>
          <GradeChip grade={r.g} size="md" />
          <button className="ckr-btn ckr-btn--ghost" style={{ border: 'none', background: 'transparent', cursor: 'pointer', font: "700 9px 'Geist', sans-serif", letterSpacing: '.08em', textTransform: 'uppercase', color: 'var(--ck-link)', padding: '4px 6px' }}>↥ Install</button>
        </div>
      ))}
    </div>
  </div>
);

// editor
const codeLines = [
  { n: 1, t: 'lens', s: '⊨ Assess   ⊢ Grade   ↥ Publish   ◆ Craft with AI' },
  { n: 1, t: 'fm', s: '---' },
  { n: 2, t: 'fm', s: 'apiVersion: ckodex.skill/v1.1' },
  { n: 3, t: 'fm', s: 'name: my-formatter' },
  { n: 4, t: 'fmw', s: 'synopsis: "Formats code blocks and stuff in files."' },
  { n: 5, t: 'fm', s: 'ontology: { primitive: evolve }' },
  { n: 6, t: 'fm', s: '---' },
  { n: 7, t: 'b', s: '' },
  { n: 8, t: 'h', s: '# My Formatter' },
  { n: 9, t: 'b', s: '' },
  { n: 10, t: 'p', s: 'Use when reformatting fenced code blocks across a' },
  { n: 11, t: 'p', s: 'workspace. Detects language and applies the project' },
  { n: 12, t: 'p', s: 'style guide.' },
];

const VSEditor = () => (
  <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, background: 'var(--ck-bg-0)' }}>
    {/* tabs */}
    <div style={{ display: 'flex', alignItems: 'stretch', background: 'var(--ck-bg-1)', height: 38, flexShrink: 0 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 16px', background: 'var(--ck-bg-0)', borderTop: '2px solid var(--ck-accent)', font: "400 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>
        <span className="ckr-glyph" style={{ color: 'var(--ck-accent)' }}>◆</span> SKILL.md <span style={{ color: 'var(--ck-accent)' }}>●</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 16px', font: "400 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>
        <span className="ckr-glyph">{'{}'}</span> skill.json
      </div>
    </div>
    {/* breadcrumb */}
    <div style={{ padding: '6px 16px', font: "400 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 12%, transparent)' }}>skills <span style={{ opacity: .5 }}>›</span> my-formatter <span style={{ opacity: .5 }}>›</span> SKILL.md</div>
    {/* code body */}
    <div style={{ flex: 1, overflow: 'hidden', position: 'relative', padding: '8px 0' }}>
      {codeLines.map((ln, i) => {
        if (ln.t === 'lens') return (
          <div key={i} style={{ display: 'flex', paddingLeft: 56, marginBottom: 3, gap: 16, font: "600 10.5px 'Geist', sans-serif", letterSpacing: '.04em', color: 'var(--ck-link)' }}>
            <span style={{ cursor: 'pointer' }}>⚡ Assess</span>
            <span style={{ cursor: 'pointer' }}>⊢ Grade</span>
            <span style={{ cursor: 'pointer' }}>↥ Publish</span>
            <span style={{ cursor: 'pointer', color: 'var(--ck-witness)' }}>◆ Craft with AI</span>
          </div>
        );
        const col = ln.t === 'fm' || ln.t === 'fmw' ? 'var(--ck-fg-2)' : ln.t === 'h' ? 'var(--ck-accent)' : ln.t === 'p' ? 'var(--ck-fg-1)' : 'var(--ck-fg-2)';
        return (
          <div key={i} style={{ display: 'flex', minHeight: 21, position: 'relative', background: ln.t === 'fmw' ? 'color-mix(in oklab, var(--ck-text-role) 9%, transparent)' : 'transparent' }}>
            <span style={{ width: 44, textAlign: 'right', paddingRight: 12, font: "400 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', flexShrink: 0, userSelect: 'none' }}>{ln.n}</span>
            <code style={{ font: "400 12.5px/21px 'JetBrains Mono', monospace", color: col, whiteSpace: 'pre', textDecoration: ln.t === 'fmw' ? 'underline wavy var(--ck-text-role)' : 'none', textUnderlineOffset: 4, textDecorationThickness: 1 }}>{ln.s}</code>
            {ln.t === 'fmw' && (
              <span style={{ position: 'absolute', right: 14, top: 1, display: 'flex', alignItems: 'center', gap: 6, font: "600 10px 'Geist', sans-serif", color: 'var(--ck-text-role)' }}>
                <span className="ckr-glyph">◆</span> AI: tighten synopsis · third-person, ≤140 <span style={{ color: 'var(--ck-fg-mute)' }}>⌥⏎</span>
              </span>
            )}
          </div>
        );
      })}
    </div>
    {/* panel */}
    <div style={{ height: 168, background: 'var(--ck-bg-1)', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 18%, transparent)', flexShrink: 0, display: 'flex', flexDirection: 'column' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 18, padding: '8px 16px', font: "700 10px 'Geist', sans-serif", letterSpacing: '.1em', textTransform: 'uppercase' }}>
        <span style={{ color: 'var(--ck-fg-3)' }}>Problems <span style={{ color: 'var(--ck-text-role)' }}>2</span></span>
        <span style={{ color: 'var(--ck-fg-3)' }}>Output</span>
        <span style={{ color: 'var(--ck-fg-1)', boxShadow: 'inset 0 -2px 0 var(--ck-accent)', paddingBottom: 6 }}>SkillPack</span>
        <span style={{ marginLeft: 'auto', font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', textTransform: 'none', letterSpacing: 0 }}>AssessStream · :50051</span>
      </div>
      <div style={{ flex: 1, overflow: 'hidden', padding: '4px 16px', font: "400 11px/17px 'JetBrains Mono', monospace" }}>
        {[
          ['var(--ck-fg-mute)', '$ skillpack assess my-formatter --watch'],
          ['var(--ck-link)', '▶ started · 9 dimensions'],
          ['var(--ck-fg-2)', '  ⊢ identity 88   ⊢ documentation 78   ⚠ testing 62'],
          ['var(--ck-deny)', '  ⊘ provenance 40 · no attestation (SKILLPACK_SEC_NO_PROVENANCE)'],
          ['var(--ck-text-role)', '◆ grade C (72) · tier L1 · 2 issues → see Problems'],
        ].map((l, i) => <div key={i} style={{ color: l[0] }}>{l[1]}</div>)}
      </div>
    </div>
  </div>
);

const VSCodeSurface = () => (
  <div data-screen-label="vscode · skillpack" style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column', background: 'var(--ck-bg-0)', overflow: 'hidden', fontFamily: 'var(--ck-ff-body)' }}>
    {/* title bar */}
    <div style={{ height: 36, background: 'color-mix(in oklab, var(--ck-deep-blue) 55%, #000)', display: 'flex', alignItems: 'center', padding: '0 12px', flexShrink: 0, gap: 8 }}>
      <span style={{ display: 'flex', gap: 7 }}>
        {['#ff5f57', '#febc2e', '#28c840'].map(c => <span key={c} style={{ width: 11, height: 11, borderRadius: 6, background: c }} />)}
      </span>
      <span style={{ flex: 1, textAlign: 'center', font: "400 11.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>my-formatter — skillpack-workspace</span>
      <span style={{ width: 52 }} />
    </div>
    <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
      <VSActivity />
      <VSSidebar />
      <VSEditor />
    </div>
    {/* status bar */}
    <div style={{ height: 24, background: 'var(--ck-accent)', display: 'flex', alignItems: 'center', padding: '0 12px', gap: 16, flexShrink: 0, font: "600 10.5px 'JetBrains Mono', monospace", color: 'var(--ck-deep-blue)' }}>
      <span>⊢ CKODEX · M3</span>
      <span>◆ grade C</span>
      <span>GAL-3</span>
      <span style={{ marginLeft: 'auto' }}>⟳ auto-assess: on</span>
      <span>ckodex.skill v1.1</span>
      <span>UTF-8</span>
    </div>
  </div>
);

Object.assign(window, { VSCodeSurface });
