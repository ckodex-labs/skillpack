// ============================================================
// SkillPack · Mobile surface (responsive web client, C-03 @ 320–430px)
// Touch-first browse → install. Bentography preserved (it's the web app
// on a phone). 44px+ targets, bottom tab nav, install bottom-sheet.
// Rendered inside the iOS bezel; Apple nav bar intentionally omitted.
// ============================================================

const MTab = ({ glyph, label, active }) => (
  <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3, padding: '8px 0', minHeight: 48, justifyContent: 'center' }}>
    <span className="ckr-glyph" aria-hidden="true" style={{ font: "400 17px 'JetBrains Mono', monospace", color: active ? 'var(--ck-accent)' : 'var(--ck-fg-mute)' }}>{glyph}</span>
    <span style={{ font: "600 9px 'Geist', sans-serif", letterSpacing: '.04em', color: active ? 'var(--ck-fg-1)' : 'var(--ck-fg-mute)' }}>{label}</span>
  </div>
);

const MAppBar = () => (
  <div style={{ paddingTop: 54, padding: '54px 16px 0', display: 'flex', alignItems: 'center', gap: 10 }}>
    <img src="../skillpack-registry/assets/mark-a2-favicon.svg" width="24" height="24" alt="" className="ckr-glyph" />
    <div style={{ flex: 1 }}>
      <div style={{ font: "900 15px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '.01em' }}>Registry</div>
      <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>12 skills · 4 registries</div>
    </div>
    <span style={{ width: 40, height: 40, display: 'grid', placeItems: 'center', clipPath: chamfer(7), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)' }}>
      <span className="ckr-glyph" style={{ font: "400 15px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}>⊟</span>
    </span>
  </div>
);

const MCard = ({ s, onTap }) => {
  const prim = PRIMITIVES.find(p => p.id === s.primitive) || {};
  return (
    <article className="ckr-bento" onClick={onTap} style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)', padding: 14 }}>
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 10, marginBottom: 8 }}>
        <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 15px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>{prim.glyph}</span>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ font: "700 14px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{s.name}</div>
          <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>v{s.version} · {s.publisher}</div>
        </div>
        <GradeChip grade={s.grade} score={s.score} />
      </div>
      <p style={{ margin: '0 0 10px', font: "400 11.5px/1.45 'Geist', sans-serif", color: 'var(--ck-fg-2)', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>{s.synopsis}</p>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <GalRail level={s.gal} attested={s.signed === 'signed' && s.provenance} />
        <span style={{ font: "600 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>GAL-{s.gal}</span>
        <button className={`ckr-btn ckr-btn--${s.installed ? 'secondary' : 'primary'} ckr-focusring`} style={{
          marginLeft: 'auto', border: 'none', cursor: 'pointer', minHeight: 40, padding: '0 16px', clipPath: chamfer(7),
          font: "700 11px 'Geist', sans-serif", letterSpacing: '.08em', textTransform: 'uppercase',
          background: s.installed ? 'transparent' : 'var(--ck-accent)', color: s.installed ? 'var(--ck-fg-1)' : 'var(--ck-deep-blue)',
          boxShadow: s.installed ? 'inset 0 0 0 2px var(--ck-stroke)' : 'none',
        }}>{s.installed ? '⊢ Installed' : '↧ Install'}</button>
      </div>
    </article>
  );
};

const MobileShell = ({ children, dimmed }) => (
  <IOSDevice dark>
    <div style={{ minHeight: '100%', background: 'var(--ck-bg-0)', display: 'flex', flexDirection: 'column', filter: dimmed ? 'brightness(.5)' : 'none' }}>
      <MAppBar />
      {/* search */}
      <div style={{ padding: '14px 16px 10px' }}>
        <div className="ckr-input" style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '0 14px', height: 46, background: 'var(--ck-bg-1)', clipPath: chamfer(9), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 45%, transparent)' }}>
          <span className="ckr-glyph" style={{ font: "400 15px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⌕</span>
          <span style={{ font: "400 14px 'Geist', sans-serif", color: 'var(--ck-fg-mute)' }}>Search skills…</span>
        </div>
      </div>
      {/* filter chips */}
      <div style={{ display: 'flex', gap: 8, padding: '0 16px 12px', overflowX: 'auto' }}>
        {['All', 'Assess', 'Create', 'Evolve', 'Bundle'].map((c, i) => (
          <span key={c} className="ckr-chip" style={{ flexShrink: 0, padding: '8px 14px', minHeight: 36, display: 'flex', alignItems: 'center', clipPath: chamfer(6), font: "600 11px 'Geist', sans-serif", letterSpacing: '.04em', background: i === 0 ? 'var(--ck-accent)' : 'transparent', color: i === 0 ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)', boxShadow: i === 0 ? 'none' : 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)' }}>{c}</span>
        ))}
      </div>
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 12, padding: '0 16px 96px' }}>
        {children}
      </div>
    </div>
    {/* bottom tab bar */}
    <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, paddingBottom: 24, background: 'color-mix(in oklab, var(--ck-bg-1) 97%, #000)', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 25%, transparent)', display: 'flex', zIndex: 40 }}>
      <MTab glyph="⊞" label="Registry" active />
      <MTab glyph="⌖" label="Discover" />
      <MTab glyph="⇄" label="Migrate" />
      <MTab glyph="≋" label="Status" />
    </div>
  </IOSDevice>
);

const MobileBrowse = () => (
  <MobileShell>
    {SKILLS.slice(0, 4).map(s => <MCard key={s.id} s={s} />)}
  </MobileShell>
);

const MobileInstall = () => {
  const s = SKILLS[0];
  const prim = PRIMITIVES.find(p => p.id === s.primitive) || {};
  return (
    <div style={{ position: 'relative', width: '100%', height: '100%' }}>
      <MobileShell dimmed>
        {SKILLS.slice(0, 4).map(x => <MCard key={x.id} s={x} />)}
      </MobileShell>
      {/* bottom sheet */}
      <div style={{ position: 'absolute', left: 8, right: 8, bottom: 0, zIndex: 70 }}>
        <div className="ckr-panel" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(16), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 45%, transparent), 0 -20px 50px rgba(0,0,0,.5)', padding: '12px 18px 30px' }}>
          <div aria-hidden="true" style={{ width: 40, height: 5, borderRadius: 3, background: 'color-mix(in oklab, var(--ck-fg-mute) 50%, transparent)', margin: '0 auto 16px' }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 14 }}>
            <span className="ckr-glyph" style={{ font: "600 20px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>{prim.glyph}</span>
            <div style={{ flex: 1 }}>
              <div style={{ font: "700 18px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{s.name}</div>
              <div style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>v{s.version} · {s.publisher} · {prim.label}</div>
            </div>
            <GradeChip grade={s.grade} score={s.score} size="lg" />
          </div>
          <p style={{ margin: '0 0 14px', font: "400 12.5px/1.5 'Geist', sans-serif", color: 'var(--ck-fg-2)' }}>{s.synopsis}</p>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 7, marginBottom: 14 }}>
            <TrustMark state="signed" />
            <TrustMark state="provenance" />
            <Badge kind="teal">{s.privacy}</Badge>
            <Badge kind="mute">GAL-{s.gal} · {s.tier}</Badge>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 7, marginBottom: 16 }}>
            <DimMeter label="security" score={s.dims.security} />
            <DimMeter label="documentation" score={s.dims.documentation} />
            <DimMeter label="provenance" score={s.dims.provenance} />
          </div>
          <button className="ckr-btn ckr-btn--primary ckr-focusring" style={{ width: '100%', border: 'none', cursor: 'pointer', minHeight: 50, clipPath: chamfer(9), background: 'var(--ck-accent)', color: 'var(--ck-deep-blue)', font: "700 13px 'Geist', sans-serif", letterSpacing: '.1em', textTransform: 'uppercase' }}>↧ Install to canonical store</button>
          <div style={{ textAlign: 'center', font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 10 }}>tool://skillpack/install · cosign verified</div>
        </div>
      </div>
    </div>
  );
};

Object.assign(window, { MobileBrowse, MobileInstall });
