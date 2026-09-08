// ============================================================
// SkillPack · CLI surface (C-01) + headless CNI gate (C-08)
// CLI: Rust (clap) · 80-col terminal · keyboard · queue+retry offline.
//   Feature matrix: most ops MUST — the most capable surface.
// CNI: headless CI/CD client · stdin/env in · structured JSON out +
//   exit codes (78 = config error). No UI; this is a quality gate.
// Safe-set glyphs only — ⊢ asserted · ⊭ refuted · ⊘ denied · ◆ grade.
// ============================================================

// ---------- shared terminal atoms ----------
const MONO = "'JetBrains Mono', monospace";
const Ln = ({ c = 'var(--ck-fg-2)', children, indent = 0, style }) => (
  <div style={{ font: `400 12.5px/19px ${MONO}`, color: c, whiteSpace: 'pre-wrap', paddingLeft: indent, ...style }}>{children}</div>
);
const Prompt = ({ cwd, cmd, cursor }) => (
  <div style={{ font: `400 12.5px/19px ${MONO}`, whiteSpace: 'pre-wrap', marginTop: 4 }}>
    <span style={{ color: 'var(--ck-witness)' }}>{cwd}</span>
    <span style={{ color: 'var(--ck-accent)' }}> ❯ </span>
    <span style={{ color: 'var(--ck-fg-1)' }}>{cmd}</span>
    {cursor && <span style={{ display: 'inline-block', width: 7, height: 14, marginLeft: 1, background: 'var(--ck-accent)', verticalAlign: '-2px' }} />}
  </div>
);
// ascii score bar — 12 cells, triple-encoded (glyph + number + bar)
const Bar = ({ label, score, glyph, gtone, note, ntone }) => {
  const fill = Math.round((score / 100) * 12);
  const tone = score >= 90 ? 'var(--ck-accent)' : score >= 70 ? 'var(--ck-link)' : score >= 50 ? 'var(--ck-witness)' : 'var(--ck-deny)';
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 0, font: `400 12.5px/20px ${MONO}` }}>
      <span style={{ color: gtone, width: 16 }}>{glyph}</span>
      <span style={{ color: 'var(--ck-fg-2)', width: 124, display: 'inline-block' }}>{label}</span>
      <span style={{ color: 'var(--ck-fg-1)', width: 28, textAlign: 'right', display: 'inline-block' }}>{score}</span>
      <span style={{ color: tone, marginLeft: 12, letterSpacing: '-1px' }}>{'█'.repeat(fill)}<span style={{ color: 'color-mix(in oklab, var(--ck-fg-mute) 40%, transparent)' }}>{'░'.repeat(12 - fill)}</span></span>
      {note && <span style={{ color: ntone || 'var(--ck-fg-mute)', marginLeft: 14 }}>{note}</span>}
    </div>
  );
};

// ============================================================
// C-01 · interactive CLI — macOS terminal window
// ============================================================
const CLISurface = () => (
  <div data-screen-label="cli · skillpack" style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column', background: 'linear-gradient(155deg,#06203c 0%, #00152b 60%, #04190f 100%)', overflow: 'hidden', fontFamily: MONO }}>
    {/* window chrome */}
    <div style={{ height: 38, background: 'rgba(3,13,28,.86)', display: 'flex', alignItems: 'center', padding: '0 13px', flexShrink: 0, gap: 8, boxShadow: 'inset 0 -1px 0 color-mix(in oklab, var(--ck-stroke) 22%, transparent)' }}>
      <span style={{ display: 'flex', gap: 8 }}>
        {['#ff5f57', '#febc2e', '#28c840'].map(c => <span key={c} style={{ width: 12, height: 12, borderRadius: 6, background: c }} />)}
      </span>
      <span style={{ flex: 1, textAlign: 'center', font: `400 11.5px ${MONO}`, color: 'rgba(255,255,255,.62)' }}>skillpack — zsh — 80×40</span>
      <span style={{ width: 56 }} />
    </div>

    {/* terminal body */}
    <div style={{ flex: 1, overflow: 'hidden', padding: '14px 18px 16px', display: 'flex', flexDirection: 'column', gap: 1 }}>
      {/* command 1 — assess */}
      <Prompt cwd="~/skills/my-formatter" cmd="skillpack assess . --watch" />
      <Ln c="var(--ck-link)">▶ assess · my-formatter · 9 dimensions · AssessStream :50051</Ln>
      <div style={{ marginTop: 3 }}>
        <Bar glyph="⊢" gtone="var(--ck-accent)" label="identity" score={88} />
        <Bar glyph="⊢" gtone="var(--ck-accent)" label="documentation" score={78} />
        <Bar glyph="⊢" gtone="var(--ck-accent)" label="compatibility" score={84} />
        <Bar glyph="⊢" gtone="var(--ck-accent)" label="governance" score={81} />
        <Bar glyph="⊭" gtone="var(--ck-witness)" label="testing" score={62} note="1 issue" ntone="var(--ck-witness)" />
        <Bar glyph="⊘" gtone="var(--ck-deny)" label="provenance" score={40} note="no attestation" ntone="var(--ck-deny)" />
      </div>
      <Ln c="var(--ck-text-role)" style={{ marginTop: 4, fontWeight: 600 }}>◆ grade C (72) · tier L1 · GAL-3 · 2 issues</Ln>
      <Ln c="var(--ck-fg-mute)" indent={16}>→ skillpack report --format html · skillpack grade --explain</Ln>

      {/* command 2 — publish denied */}
      <Prompt cwd="~/skills/my-formatter" cmd="skillpack publish oci://reg.ckodex.org/skills/my-formatter" />
      <Ln c="var(--ck-deny)" style={{ fontWeight: 600 }}>⊘ denied — SKILLPACK_SEC_NO_PROVENANCE  <span style={{ color: 'var(--ck-fg-mute)', fontWeight: 400 }}>[security · error]</span></Ln>
      <Ln c="var(--ck-fg-2)" indent={16}>skill name asserts no attestation; publish gate requires a signed lock.</Ln>
      <Ln c="var(--ck-fg-mute)" indent={16}>remediation: skillpack lock --sign  then re-run publish</Ln>
      <Ln c="var(--ck-fg-mute)" indent={16}>retry with --force to override (records a waiver) · exit 78</Ln>

      {/* command 3 — install, live prompt */}
      <Prompt cwd="~/skills/my-formatter" cmd="skillpack install proof-audit@1.4.2" cursor />
      <Ln c="var(--ck-link)">↧ resolving · reg.ckodex.org · cosign ⊢ verified · Rekor #4471902</Ln>
      <Ln c="var(--ck-accent)">⊢ installed proof-audit 1.4.2 → ~/Skills/shared · synced 12/12 agents</Ln>
    </div>

    {/* status line */}
    <div style={{ height: 24, background: 'var(--ck-accent)', display: 'flex', alignItems: 'center', padding: '0 14px', gap: 16, flexShrink: 0, font: `600 10.5px ${MONO}`, color: 'var(--ck-deep-blue)' }}>
      <span>skillpack 1.0.0</span>
      <span>⊢ CKODEX v16</span>
      <span>in-proc + gRPC :50051</span>
      <span style={{ marginLeft: 'auto' }}>queue 0</span>
      <span>--no-color · --quiet ready</span>
    </div>
  </div>
);

// ============================================================
// C-08 · headless CNI — CI/CD runner log (non-interactive gate)
// ============================================================
const CNIStep = ({ state, label, detail }) => {
  const m = {
    pass: { g: '⊢', c: 'var(--ck-accent)' },
    fail: { g: '⊘', c: 'var(--ck-deny)' },
    warn: { g: '⊭', c: 'var(--ck-witness)' },
    run:  { g: '◐', c: 'var(--ck-link)' },
  }[state];
  return (
    <div style={{ display: 'flex', alignItems: 'baseline', gap: 10, padding: '5px 0' }}>
      <span style={{ font: `500 13px ${MONO}`, color: m.c, width: 14 }}>{m.g}</span>
      <span style={{ flex: 1, font: `500 12px ${MONO}`, color: 'var(--ck-fg-1)' }}>{label}
        {detail && <span style={{ color: 'var(--ck-fg-mute)', fontWeight: 400 }}>  {detail}</span>}
      </span>
    </div>
  );
};

const CNISurface = () => (
  <div data-screen-label="cni · ci gate" style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column', background: 'var(--ck-bg-0)', overflow: 'hidden', fontFamily: 'var(--ck-ff-body)' }}>
    {/* runner header */}
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '13px 16px', background: 'var(--ck-bg-1)', boxShadow: 'inset 0 -1px 0 color-mix(in oklab, var(--ck-stroke) 20%, transparent)', flexShrink: 0 }}>
      <span style={{ width: 9, height: 9, background: 'var(--ck-deny)', clipPath: chamfer(2), flexShrink: 0 }} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ font: "700 12px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>skillpack-cni · quality gate</div>
        <div style={{ font: `500 9.5px ${MONO}`, color: 'var(--ck-fg-mute)' }}>.github/workflows/skills.yml · job: gate · runner ubuntu-24.04</div>
      </div>
      <Badge kind="deny">⊘ exit 78</Badge>
    </div>

    {/* invocation + env */}
    <div style={{ padding: '12px 16px 8px', background: 'var(--ck-bg-0)' }}>
      <Ln c="var(--ck-fg-mute)" style={{ fontSize: 11.5 }}>$ SKILLPACK_API_URL=$CK_URL SKILLPACK_TOKEN=*** \</Ln>
      <Ln c="var(--ck-fg-1)" indent={10} style={{ fontSize: 11.5 }}>skillpack-cni gate --skill ./my-formatter --policy prod --json</Ln>
    </div>

    {/* gate steps */}
    <div style={{ flex: 1, overflow: 'hidden', padding: '6px 16px', minHeight: 0 }}>
      <div style={{ font: "700 9px 'Geist', sans-serif", letterSpacing: '.13em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 4 }}>Gate · policy prod</div>
      <CNIStep state="pass" label="capability negotiation" detail="schema v1 · gRPC" />
      <CNIStep state="pass" label="ip boundary check" detail="no restricted patterns" />
      <CNIStep state="warn" label="assess · grade C (72)" detail="≥ B required → soft" />
      <CNIStep state="fail" label="provenance" detail="no attestation · blocks promote" />

      {/* json on stderr */}
      <div style={{ marginTop: 10, background: 'var(--ck-bg-1)', clipPath: chamfer(8), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-deny) 38%, transparent)', padding: '11px 13px' }}>
        <div style={{ font: `600 9px ${MONO}`, color: 'var(--ck-fg-mute)', marginBottom: 7, letterSpacing: '.04em' }}>stderr · application/json</div>
        <Ln c="var(--ck-fg-3)" style={{ fontSize: 11.5, lineHeight: '17px' }}>{`{`}</Ln>
        <Ln c="var(--ck-fg-2)" indent={14} style={{ fontSize: 11.5, lineHeight: '17px' }}>{`"code": `}<span style={{ color: 'var(--ck-deny)' }}>"SKILLPACK_IP_VIOLATION"</span>{`,`}</Ln>
        <Ln c="var(--ck-fg-2)" indent={14} style={{ fontSize: 11.5, lineHeight: '17px' }}>{`"category": `}<span style={{ color: 'var(--ck-witness)' }}>"security"</span>{`, `}<span style={{ color: 'var(--ck-fg-2)' }}>{`"severity": `}</span><span style={{ color: 'var(--ck-deny)' }}>"error"</span>{`,`}</Ln>
        <Ln c="var(--ck-fg-2)" indent={14} style={{ fontSize: 11.5, lineHeight: '17px' }}>{`"clientHint": { "cni": `}<span style={{ color: 'var(--ck-accent)' }}>"exit 78"</span>{` }`}</Ln>
        <Ln c="var(--ck-fg-3)" style={{ fontSize: 11.5, lineHeight: '17px' }}>{`}`}</Ln>
      </div>
    </div>

    {/* exit line */}
    <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '11px 16px', background: 'var(--ck-bg-1)', boxShadow: 'inset 0 1px 0 color-mix(in oklab, var(--ck-stroke) 20%, transparent)', flexShrink: 0 }}>
      <span style={{ font: `600 11px ${MONO}`, color: 'var(--ck-deny)' }}>⊘ gate failed · process exited 78</span>
      <span style={{ marginLeft: 'auto', font: `500 10px ${MONO}`, color: 'var(--ck-fg-mute)' }}>fail-fast · non-interactive · no retry</span>
    </div>
  </div>
);

Object.assign(window, { CLISurface, CNISurface });
