// ============================================================
// SkillPack Registry · Bentography primitives (extends CKODEX-DS-1)
// Loaded as text/babel. Every interactive/structural element carries a
// ckr-* className so a11y-themes.css can restore borders under forced-colors.
// ============================================================
const { useState: _useState } = React;

const chamfer = (c) =>
  `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;

// ---- Chamfered container ----
const Bento = ({ stroked = true, children, style, pad = 18, accent, className = '', ...rest }) => {
  const ring = stroked
    ? `inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) ${accent ? 70 : 38}%, transparent)`
    : 'none';
  return (
    <div {...rest} className={`ckr-bento ${className}`} style={{
      background: 'var(--ck-bg-1)', clipPath: chamfer(10),
      boxShadow: ring, padding: pad, position: 'relative', ...style,
    }}>
      {children}
    </div>
  );
};

// ---- Badge: icon + text (never color-alone) ----
const Badge = ({ kind = 'kernel', children, title }) => {
  const palette = {
    attested: { bg: 'var(--ck-accent)', fg: 'var(--ck-deep-blue)', ring: 'transparent' },
    kernel:   { bg: 'transparent', fg: 'var(--ck-fg-2)', ring: 'var(--ck-stroke)' },
    witness:  { bg: 'transparent', fg: 'var(--ck-witness)', ring: 'var(--ck-witness)' },
    teal:     { bg: 'transparent', fg: 'var(--ck-link)', ring: 'var(--ck-stroke)' },
    deny:     { bg: 'transparent', fg: 'var(--ck-deny)', ring: 'var(--ck-deny)' },
    mute:     { bg: 'transparent', fg: 'var(--ck-fg-3)', ring: 'var(--ck-fg-mute)' },
  }[kind];
  return (
    <span className="ckr-badge" title={title} style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      font: "700 10px 'JetBrains Mono', monospace",
      letterSpacing: '.07em', textTransform: 'uppercase',
      padding: '4px 9px', background: palette.bg, color: palette.fg,
      clipPath: chamfer(5),
      boxShadow: palette.ring === 'transparent' ? 'none' : `inset 0 0 0 1.5px ${palette.ring}`,
      whiteSpace: 'nowrap',
    }}>{children}</span>
  );
};

// ---- Button ----
const CkButton = ({ variant = 'primary', children, onClick, style, disabled, title, ...rest }) => {
  const palette = {
    primary:   { bg: 'var(--ck-accent)', fg: 'var(--ck-deep-blue)', ring: 'transparent' },
    secondary: { bg: 'transparent', fg: 'var(--ck-fg-1)', ring: 'var(--ck-stroke)' },
    ghost:     { bg: 'transparent', fg: 'var(--ck-link)', ring: 'transparent' },
    deny:      { bg: 'transparent', fg: 'var(--ck-deny)', ring: 'var(--ck-deny)' },
  }[variant];
  return (
    <button onClick={onClick} disabled={disabled} title={title} {...rest}
      className={`ckr-btn ckr-btn--${variant} ckr-focusring`} style={{
        font: "700 11px 'Geist', sans-serif", letterSpacing: '.09em', textTransform: 'uppercase',
        padding: '10px 16px', border: 'none', cursor: disabled ? 'not-allowed' : 'pointer',
        background: palette.bg, color: palette.fg, clipPath: chamfer(8),
        boxShadow: palette.ring === 'transparent' ? 'none' : `inset 0 0 0 2px ${palette.ring}`,
        opacity: disabled ? 0.45 : 1, display: 'inline-flex', alignItems: 'center', gap: 7,
        transition: 'filter .12s ease', ...style,
      }}
      onMouseEnter={(e) => { if (!disabled) e.currentTarget.style.filter = 'brightness(1.12)'; }}
      onMouseLeave={(e) => { e.currentTarget.style.filter = 'none'; }}
    >{children}</button>
  );
};

// ---- GAL rail · 6 segments (text label always supplied alongside) ----
const GalRail = ({ level = 3, width = 84, attested = false }) => (
  <div className="ckr-rail" role="img" aria-label={`Governance assurance level ${level} of 6`}
    style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: 2, width, height: 8 }}>
    {Array.from({ length: 6 }).map((_, i) => (
      <i key={i} className={i < level ? 'on' : ''} style={{
        background: attested ? 'var(--ck-accent)' : 'var(--ck-rail)',
        opacity: i < level ? 1 : 0.22,
      }} />
    ))}
  </div>
);

// ---- Grade chip · S+ A B C D F · letter is primary signal ----
const gradeMeta = {
  'S+': { fg: 'var(--ck-deep-blue)', bg: 'var(--ck-accent)', ring: 'transparent', note: 'exemplary' },
  'A':  { fg: 'var(--ck-link)',     bg: 'transparent', ring: 'var(--ck-stroke)', note: 'strong' },
  'B':  { fg: 'var(--ck-witness)',  bg: 'transparent', ring: 'var(--ck-witness)', note: 'sound' },
  'C':  { fg: 'var(--ck-fg-2)',     bg: 'transparent', ring: 'var(--ck-fg-mute)', note: 'fair' },
  'D':  { fg: 'var(--ck-text-role)',bg: 'transparent', ring: 'var(--ck-text-role)', note: 'weak' },
  'F':  { fg: 'var(--ck-deny)',     bg: 'transparent', ring: 'var(--ck-deny)', note: 'failing' },
  '—':  { fg: 'var(--ck-fg-3)',     bg: 'transparent', ring: 'var(--ck-fg-mute)', note: 'unrated' },
};
const GradeChip = ({ grade = 'B', score, size = 'md' }) => {
  const m = gradeMeta[grade] || gradeMeta['—'];
  const dim = size === 'lg' ? 52 : 40;
  return (
    <div className="ckr-badge" title={`Grade ${grade} · ${m.note}${score != null ? ` · score ${score}` : ''}`}
      role="img" aria-label={`Grade ${grade}, ${m.note}${score != null ? `, score ${score}` : ''}`}
      style={{
        width: dim, height: dim, display: 'grid', placeItems: 'center',
        background: m.bg, color: m.fg, clipPath: chamfer(7),
        boxShadow: m.ring === 'transparent' ? 'none' : `inset 0 0 0 2px ${m.ring}`,
        flexShrink: 0,
      }}>
      <span style={{ font: `800 ${size === 'lg' ? 20 : 16}px 'Geist', sans-serif`, lineHeight: 1, letterSpacing: '-.02em' }}>{grade}</span>
    </div>
  );
};

// ---- Tier flag · L0..L4X (5-sided FSM flag) ----
const TierFlag = ({ size = 40, tier = 'L2' }) => (
  <svg className="ckr-glyph" width={size} height={size} viewBox="0 0 96 96" role="img" aria-label={`Lifecycle tier ${tier}`}>
    <polygon points="48,8 82,28 72,78 24,78 14,28" fill="none" stroke="var(--ck-stroke)" strokeWidth="2.5" />
    <polygon points="48,22 70,36 63,66 33,66 26,36" fill="var(--ck-witness)" opacity=".28" />
    <text x="48" y="56" textAnchor="middle" fill="var(--ck-fg-1)" fontFamily="JetBrains Mono" fontSize="22" fontWeight="700">{tier}</text>
  </svg>
);

// ---- Dimension meter · label + numeric + bar (triple-encoded) ----
const DimMeter = ({ label, score = 80, max = 100 }) => {
  const rated = score != null;
  const pct = rated ? Math.max(0, Math.min(100, (score / max) * 100)) : 0;
  const tone = !rated ? 'transparent' : score >= 90 ? 'var(--ck-accent)' : score >= 70 ? 'var(--ck-link)' : score >= 50 ? 'var(--ck-witness)' : 'var(--ck-deny)';
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '116px 1fr 34px', alignItems: 'center', gap: 10, opacity: rated ? 1 : 0.6 }}>
      <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)', letterSpacing: '.02em' }}>{label}</span>
      <span className="ckr-rail" role="img" aria-label={rated ? `${label}: ${score} of ${max}` : `${label}: unrated`} style={{
        position: 'relative', height: 6, background: 'color-mix(in oklab, var(--ck-fg-mute) 30%, transparent)',
        clipPath: chamfer(2), display: 'block',
      }}>
        <span style={{ position: 'absolute', inset: 0, width: `${pct}%`, background: tone }} />
      </span>
      <span style={{ font: "600 11px 'JetBrains Mono', monospace", color: rated ? 'var(--ck-fg-1)' : 'var(--ck-fg-mute)', textAlign: 'right' }}>{rated ? score : '—'}</span>
    </div>
  );
};

// ---- Trust mark · signature / provenance state, icon + word ----
const TrustMark = ({ state = 'signed' }) => {
  const meta = {
    signed:    { glyph: '⊢', word: 'cosign verified', kind: 'attested' },
    provenance:{ glyph: '⊛', word: 'provenance', kind: 'witness' },
    unsigned:  { glyph: '◌', word: 'unsigned', kind: 'mute' },
    untrusted: { glyph: '⚠', word: 'untrusted registry', kind: 'deny' },
    unknown:   { glyph: '○', word: 'unverified', kind: 'mute' },
  }[state] || {};
  return <Badge kind={meta.kind} title={meta.word}><span className="ckr-glyph" aria-hidden="true">{meta.glyph}</span>{meta.word}</Badge>;
};

// ---- Shared page widgets ----
const SectionLabel = ({ children, style }) => (
  <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', ...style }}>{children}</div>
);

// JSON code panel with light token tinting (mono)
const CodeBlock = ({ json, title, height }) => {
  const text = typeof json === 'string' ? json : JSON.stringify(json, null, 2);
  const lines = text.split('\n');
  const tint = (ln) => {
    // key
    let html = ln.replace(/("[^"]+")(\s*:)/g, '<span style="color:var(--ck-link)">$1</span>$2');
    // string values
    html = html.replace(/:\s*("(?:[^"\\]|\\.)*")/g, (m, p1) => m.replace(p1, `<span style="color:var(--ck-text-role)">${p1}</span>`));
    // booleans / numbers
    html = html.replace(/\b(true|false|null)\b/g, '<span style="color:var(--ck-witness)">$1</span>');
    return html;
  };
  return (
    <div className="ckr-bento" style={{ background: 'var(--ck-bg-2)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 35%, transparent)', overflow: 'hidden' }}>
      {title && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '10px 14px', borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)' }}>
          <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⊞</span>
          <span style={{ font: "600 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}>{title}</span>
        </div>
      )}
      <pre style={{ margin: 0, padding: '14px 16px', overflow: 'auto', maxHeight: height, font: "400 11.5px/1.6 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}>
        {lines.map((ln, i) => (
          <div key={i} style={{ display: 'flex', gap: 14 }}>
            <span aria-hidden="true" style={{ color: 'var(--ck-fg-mute)', userSelect: 'none', width: 20, textAlign: 'right', flexShrink: 0 }}>{i + 1}</span>
            <code style={{ whiteSpace: 'pre' }} dangerouslySetInnerHTML={{ __html: tint(ln) || '&nbsp;' }} />
          </div>
        ))}
      </pre>
    </div>
  );
};

const StatTile = ({ value, label, glyph, tone }) => (
  <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 32%, transparent)', padding: 18 }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
      <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 14px 'JetBrains Mono', monospace", color: tone || 'var(--ck-accent)' }}>{glyph}</span>
      <SectionLabel>{label}</SectionLabel>
    </div>
    <div style={{ font: "800 34px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.02em', lineHeight: 1 }}>{value}</div>
  </div>
);

// ---- Pagination · windowed page controls (shared) ----
const usePageWindow = (page, pageCount, span = 2) => {
  const pages = [];
  const from = Math.max(1, page - span);
  const to = Math.min(pageCount, page + span);
  if (from > 1) { pages.push(1); if (from > 2) pages.push('…'); }
  for (let p = from; p <= to; p++) pages.push(p);
  if (to < pageCount) { if (to < pageCount - 1) pages.push('…'); pages.push(pageCount); }
  return pages;
};

const Pagination = ({ page, pageCount, total, pageSize, onPage, label = 'results' }) => {
  if (pageCount <= 1) return null;
  const start = (page - 1) * pageSize + 1;
  const end = Math.min(total, page * pageSize);
  const pages = usePageWindow(page, pageCount);
  const cell = (content, target, opts = {}) => {
    const { disabled = false, active = false, key } = opts;
    const isGap = content === '…';
    return (
      <button key={key ?? content} onClick={() => !disabled && !isGap && onPage(target)} disabled={disabled || isGap}
        aria-current={active ? 'page' : undefined} aria-label={typeof content === 'string' && !isGap ? content : `Page ${target}`}
        className={isGap ? undefined : 'ckr-focusring'} style={{
          minWidth: 32, height: 32, padding: '0 9px', border: 'none', clipPath: chamfer(4),
          cursor: disabled || isGap ? 'default' : 'pointer',
          font: "600 12px 'JetBrains Mono', monospace",
          background: active ? 'var(--ck-accent)' : 'transparent',
          color: active ? 'var(--ck-deep-blue)' : isGap ? 'var(--ck-fg-mute)' : disabled ? 'var(--ck-fg-mute)' : 'var(--ck-fg-2)',
          boxShadow: active || isGap ? 'none' : 'inset 0 0 0 1.2px color-mix(in oklab, var(--ck-stroke) 40%, transparent)',
          opacity: disabled ? 0.4 : 1,
        }}>{content}</button>
    );
  };
  return (
    <nav aria-label="Pagination" style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap', paddingTop: 4 }}>
      <span aria-live="polite" style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', marginRight: 'auto' }}>
        {start}&ndash;{end} of {total} {label}
      </span>
      {cell('‹ Prev', page - 1, { disabled: page <= 1, key: 'prev' })}
      {pages.map((p, i) => cell(p, p, { active: p === page, key: `p${i}` }))}
      {cell('Next ›', page + 1, { disabled: page >= pageCount, key: 'next' })}
    </nav>
  );
};

Object.assign(window, {
  chamfer, Bento, Badge, CkButton, GalRail, GradeChip, gradeMeta, TierFlag, DimMeter, TrustMark,
  SectionLabel, CodeBlock, StatTile, Pagination,
});
