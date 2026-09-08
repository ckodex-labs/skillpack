// ============================================================
// SkillPack · Transparency Fabric — shared diagram vocabulary
// DS-3 brand diagrams: flat paper, ink structure, faint dot field,
// sealed octagon kernels, typed edges (attested head = violet),
// claim chips. Budget hues (rust/violet/red) never sit in a background.
// ============================================================

const TF_MONO = "var(--ck-ff-mono)";
const TF_UI = "var(--ck-ff-ui)";
const TF_DISP = "var(--ck-ff-display)";

// ---- faint engineering dot field (ledger: ink dots @ ~0.12) ----
const DotField = ({ children, style, pad = 28 }) => (
  <div style={{
    position: 'relative',
    background: 'var(--ck-bg-0)',
    backgroundImage: 'radial-gradient(color-mix(in oklab, var(--ck-fg-1) 12%, transparent) 1px, transparent 1.4px)',
    backgroundSize: '18px 18px',
    backgroundPosition: '-1px -1px',
    border: '1px solid var(--ck-hairline)',
    padding: pad,
    ...style,
  }}>{children}</div>
);

// ---- section heading: index · name · kicker ----
const SectionHead = ({ n, name, kicker }) => (
  <div style={{ display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 20 }}>
    <span style={{ font: `500 13px ${TF_MONO}`, color: 'var(--ck-fg-mute)', letterSpacing: '.1em' }}>{n}</span>
    <h2 style={{ margin: 0, font: `400 30px/1.05 ${TF_DISP}`, color: 'var(--ck-fg-1)' }}>{name}</h2>
    <span style={{ font: `500 12px ${TF_MONO}`, color: 'var(--ck-fg-3)', marginLeft: 'auto', textAlign: 'right', maxWidth: 280 }}>{kicker}</span>
  </div>
);

const TFLabel = ({ children, style }) => (
  <div style={{ font: `700 10px ${TF_UI}`, letterSpacing: '.15em', textTransform: 'uppercase', color: 'var(--ck-fg-3)', ...style }}>{children}</div>
);

// verbatim-style invariant line
const Invariant = ({ children }) => (
  <div style={{ display: 'flex', alignItems: 'baseline', gap: 9, padding: '5px 0' }}>
    <span style={{ font: `600 11px ${TF_MONO}`, color: 'var(--ck-proof)', flexShrink: 0 }}>⊢</span>
    <span style={{ font: `600 12px/1.45 ${TF_MONO}`, color: 'var(--ck-fg-1)', letterSpacing: '.01em' }}>{children}</span>
  </div>
);

// ---- octagon kernel node (the sealed-crate mark) ----
const Octagon = ({ size = 116, label, sub, tone = 'var(--ck-fg-1)', fill = 'var(--ck-bg-1)' }) => {
  const c = size * 0.29;
  const clip = `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;
  return (
    <div style={{ width: size, height: size, position: 'relative', flexShrink: 0 }}>
      <div style={{ position: 'absolute', inset: 0, clipPath: clip, background: fill, boxShadow: `inset 0 0 0 2px ${tone}` }} />
      <div style={{ position: 'absolute', inset: 0, display: 'grid', placeItems: 'center', textAlign: 'center', padding: 8 }}>
        <div>
          <div style={{ font: `700 11px ${TF_MONO}`, letterSpacing: '.06em', color: tone, lineHeight: 1.25 }}>{label}</div>
          {sub && <div style={{ font: `500 8.5px ${TF_MONO}`, color: 'var(--ck-fg-mute)', marginTop: 3 }}>{sub}</div>}
        </div>
      </div>
    </div>
  );
};

// ---- a labeled node box (square = quiet, sealed = chamfered) ----
const Node = ({ title, sub, glyph, gtone, sealed, tone, style }) => (
  <div style={{
    position: 'relative', background: 'var(--ck-bg-1)', padding: '11px 13px', minWidth: 0,
    clipPath: sealed ? 'polygon(8px 0,calc(100% - 8px) 0,100% 8px,100% calc(100% - 8px),calc(100% - 8px) 100%,8px 100%,0 calc(100% - 8px),0 8px)' : 'none',
    boxShadow: sealed ? `inset 0 0 0 1.5px ${tone || 'var(--ck-seal)'}` : 'none',
    border: sealed ? 'none' : `1px solid ${tone || 'var(--ck-hairline-strong)'}`,
    ...style,
  }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
      {glyph && <span style={{ font: `600 13px ${TF_MONO}`, color: gtone || 'var(--ck-fg-2)', flexShrink: 0 }}>{glyph}</span>}
      <span style={{ font: `600 12px ${TF_UI}`, color: 'var(--ck-fg-1)' }}>{title}</span>
    </div>
    {sub && <div style={{ font: `500 10px ${TF_MONO}`, color: 'var(--ck-fg-mute)', marginTop: 4, letterSpacing: '.02em' }}>{sub}</div>}
  </div>
);

// ---- claim chip ----
const Chip = ({ children, kind }) => {
  const m = {
    attested:    { c: 'var(--ck-proof)', b: 'var(--ck-proof)' },
    deny:        { c: 'var(--ck-alarm)', b: 'var(--ck-alarm)' },
    observed:    { c: 'var(--ck-fg-1)', b: 'var(--ck-fg-1)' },
    rust:        { c: 'var(--ck-accent)', b: 'var(--ck-accent)' },
    tone:        { c: 'var(--ck-fg-3)', b: 'var(--ck-hairline-strong)' },
  }[kind || 'tone'];
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, font: `600 10px ${TF_MONO}`, letterSpacing: '.07em', textTransform: 'uppercase', color: m.c, boxShadow: `inset 0 0 0 1px ${m.b}`, padding: '4px 9px' }}>{children}</span>
  );
};

// ---- typed edge (SVG) · variants: attested(violet head), kernel(double), deny(red X), promote(chevron) ----
const Edge = ({ variant = 'attested', w = 96, vertical, label }) => {
  const stroke = variant === 'deny' ? 'var(--ck-alarm)' : variant === 'attested' ? 'var(--ck-fg-1)' : 'var(--ck-fg-1)';
  const head = variant === 'attested' ? 'var(--ck-proof)' : variant === 'promote' ? 'var(--ck-fg-1)' : null;
  const L = vertical ? 34 : w;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
      <svg width={vertical ? 18 : L} height={vertical ? L : 18} style={{ overflow: 'visible' }} aria-hidden="true">
        {vertical ? (
          <g>
            <line x1="9" y1="0" x2="9" y2={L - 7} stroke={stroke} strokeWidth="1.25" strokeDasharray={variant === 'kernel' ? '0' : '0'} />
            {variant === 'attested' && [0.34, 0.5, 0.66].map((p, i) => <line key={i} x1="5" y1={L * p} x2="13" y2={L * p} stroke="var(--ck-proof)" strokeWidth="1" />)}
            {variant === 'deny'
              ? <g stroke="var(--ck-alarm)" strokeWidth="1.5"><line x1="4" y1={L - 9} x2="14" y2={L + 1} /><line x1="14" y1={L - 9} x2="4" y2={L + 1} /></g>
              : <path d={`M5 ${L - 8} L9 ${L} L13 ${L - 8}`} fill="none" stroke={head || stroke} strokeWidth="1.5" />}
          </g>
        ) : (
          <g>
            <line x1="0" y1="9" x2={L - 7} y2="9" stroke={stroke} strokeWidth="1.25" />
            {variant === 'kernel' && <line x1="0" y1="12.5" x2={L - 7} y2="12.5" stroke={stroke} strokeWidth="1.25" />}
            {variant === 'attested' && [0.34, 0.5, 0.66].map((p, i) => <line key={i} x1={L * p} y1="5" x2={L * p} y2="13" stroke="var(--ck-proof)" strokeWidth="1" />)}
            {variant === 'deny'
              ? <g stroke="var(--ck-alarm)" strokeWidth="1.5"><line x1={L - 9} y1="4" x2={L + 1} y2="14" /><line x1={L - 9} y1="14" x2={L + 1} y2="4" /></g>
              : <path d={`M${L - 8} 5 L${L} 9 L${L - 8} 13`} fill="none" stroke={head || stroke} strokeWidth="1.5" />}
          </g>
        )}
      </svg>
      {label && <span style={{ font: `500 9px ${TF_MONO}`, color: 'var(--ck-fg-mute)', marginTop: 3, letterSpacing: '.04em' }}>{label}</span>}
    </div>
  );
};

Object.assign(window, { DotField, SectionHead, TFLabel, Invariant, Octagon, Node, Chip, Edge, TF_MONO, TF_UI, TF_DISP });
