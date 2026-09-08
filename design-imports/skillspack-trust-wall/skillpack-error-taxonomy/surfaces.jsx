// ============================================================
// SkillPack · Error Taxonomy — per-surface renderings
// One canonical ClientError (SKILLPACK_IP_VIOLATION · security · error)
// realized on every surface exactly per its spec §5.1 clientHint.
// DS-3 · red = containment (the error blocks publish) · safe glyphs.
// ============================================================

const ET_MONO = "var(--ck-ff-mono)";
const ET_UI = "var(--ck-ff-ui)";
const DARK = "#0A1322";   // vault ground — terminals stay dark on paper

// ---- shared card frame: surface label · chrome · verbatim hint ----
const SurfaceCard = ({ id, surface, tech, hint, derived, children }) => (
  <figure style={{ margin: 0, display: 'flex', flexDirection: 'column', background: 'var(--ck-bg-1)', border: '1px solid var(--ck-hairline)' }}>
    <figcaption style={{ display: 'flex', alignItems: 'baseline', gap: 8, padding: '11px 14px', borderBottom: '1px solid var(--ck-hairline)' }}>
      <span style={{ font: `700 11px ${ET_MONO}`, letterSpacing: '.06em', color: 'var(--ck-fg-1)' }}>{surface}</span>
      <span style={{ font: `500 10px ${ET_MONO}`, color: 'var(--ck-fg-mute)' }}>{tech}</span>
      {derived
        ? <span className="ck-chip" style={{ marginLeft: 'auto' }}>derived</span>
        : <span style={{ marginLeft: 'auto', font: `600 10px ${ET_MONO}`, color: 'var(--ck-fg-3)' }}>{id}</span>}
    </figcaption>
    <div style={{ flex: 1, minHeight: 196, display: 'flex', flexDirection: 'column' }}>{children}</div>
    <div style={{ padding: '10px 14px', borderTop: '1px solid var(--ck-hairline)', background: 'var(--ck-bg-2)' }}>
      <span style={{ font: `500 10px ${ET_MONO}`, color: 'var(--ck-fg-3)', letterSpacing: '.16em', textTransform: 'uppercase' }}>clientHint</span>
      <div style={{ font: `400 12.5px/1.5 ${ET_UI}`, color: 'var(--ck-fg-2)', marginTop: 3 }}>“{hint}”</div>
    </div>
  </figure>
);

// terminal scaffold (dark) ------------------------------------
const Term = ({ title, children }) => (
  <div style={{ flex: 1, background: DARK, display: 'flex', flexDirection: 'column' }}>
    <div style={{ height: 26, display: 'flex', alignItems: 'center', gap: 6, padding: '0 10px', background: 'rgba(255,255,255,.05)' }}>
      <span style={{ display: 'flex', gap: 5 }}>{['#ff5f57', '#febc2e', '#28c840'].map(c => <span key={c} style={{ width: 9, height: 9, borderRadius: 5, background: c }} />)}</span>
      <span style={{ flex: 1, textAlign: 'center', font: `400 9.5px ${ET_MONO}`, color: 'rgba(234,229,218,.5)' }}>{title}</span>
      <span style={{ width: 33 }} />
    </div>
    <div style={{ flex: 1, padding: '11px 13px', font: `400 11.5px/1.7 ${ET_MONO}` }}>{children}</div>
  </div>
);
const TL = ({ c = 'rgba(234,229,218,.82)', children, indent = 0 }) => (
  <div style={{ color: c, whiteSpace: 'pre-wrap', paddingLeft: indent }}>{children}</div>
);

// 1 · CLI -----------------------------------------------------
const CardCLI = () => (
  <SurfaceCard id="C-01" surface="CLI" tech="rust · clap" hint="Print red text with --force flag suggestion">
    <Term title="zsh — skillpack">
      <TL c="#9A9284"><span style={{ color: '#D2693A' }}>~/skills/thales-helper</span> ❯ skillpack publish</TL>
      <TL c="#F87171"><span style={{ fontWeight: 600 }}>⊘ error</span>  SKILLPACK_IP_VIOLATION  <span style={{ color: '#9A9284' }}>[security]</span></TL>
      <TL c="rgba(234,229,218,.82)" indent={8}>Skill name contains restricted pattern: <span style={{ color: '#F87171' }}>"thales"</span></TL>
      <TL c="#9A9284" indent={8}>→ retry with <span style={{ color: '#D2693A' }}>--force</span> to override (records a waiver)</TL>
      <TL c="#6B6457">exit 78</TL>
    </Term>
  </SurfaceCard>
);

// 2 · VS Code -------------------------------------------------
const CardVSCode = () => (
  <SurfaceCard id="C-02" surface="VS Code" tech="typescript" hint="Show error notification with 'View Details' button">
    <div style={{ flex: 1, background: '#15212E', position: 'relative', overflow: 'hidden' }}>
      {/* faux editor lines */}
      <div style={{ position: 'absolute', inset: 0, padding: '12px 14px', font: `400 11px/1.85 ${ET_MONO}`, color: 'rgba(199,191,175,.22)' }}>
        {['name: thales-helper', 'apiVersion: ckodex.skill/v1.1', 'entry: ./skill.md', 'tier: L1'].map((l, i) => <div key={i}>{l}</div>)}
      </div>
      {/* notification toast */}
      <div style={{ position: 'absolute', right: 12, bottom: 12, width: 250, background: '#101C30', boxShadow: 'inset 0 0 0 1px #2C3D5A, 0 12px 30px rgba(0,0,0,.45)', display: 'flex' }}>
        <span style={{ width: 3, background: '#F87171', flexShrink: 0 }} />
        <div style={{ padding: '10px 12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 7, marginBottom: 5 }}>
            <span style={{ font: `600 12px ${ET_MONO}`, color: '#F87171' }}>⊘</span>
            <span style={{ font: `600 11px ${ET_UI}`, color: '#EAE5DA' }}>Skill name contains a restricted pattern</span>
          </div>
          <div style={{ font: `400 10.5px/1.5 ${ET_UI}`, color: 'rgba(199,191,175,.7)', marginBottom: 9 }}>SKILLPACK_IP_VIOLATION · “thales”</div>
          <div style={{ display: 'flex', gap: 7 }}>
            <span style={{ font: `600 10px ${ET_MONO}`, letterSpacing: '.04em', color: '#0A1322', background: '#D2693A', padding: '5px 9px' }}>VIEW DETAILS</span>
            <span style={{ font: `600 10px ${ET_MONO}`, letterSpacing: '.04em', color: '#C7BFAF', boxShadow: 'inset 0 0 0 1px #2C3D5A', padding: '5px 9px' }}>OVERRIDE</span>
          </div>
        </div>
      </div>
    </div>
  </SurfaceCard>
);

// 3 · Web (Ledger paper banner) -------------------------------
const CardWeb = () => (
  <SurfaceCard id="C-03" surface="Web Dashboard" tech="react" hint="Display inline banner with dismiss action">
    <div style={{ flex: 1, background: 'var(--ck-bg-0)', padding: 14, display: 'flex', flexDirection: 'column', gap: 10 }}>
      {/* the inline banner */}
      <div style={{ display: 'grid', gridTemplateColumns: '2px 1fr', gap: 12, background: 'var(--ck-bg-1)', border: '1px solid color-mix(in oklab, var(--ck-alarm) 40%, transparent)', padding: '11px 13px' }}>
        <span style={{ background: 'var(--ck-alarm)' }} />
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ font: `600 13px ${ET_MONO}`, color: 'var(--ck-alarm)' }}>⊘</span>
            <span style={{ font: `600 12px ${ET_UI}`, color: 'var(--ck-fg-1)' }}>Skill name contains a restricted pattern</span>
            <span style={{ marginLeft: 'auto', font: `400 15px ${ET_UI}`, color: 'var(--ck-fg-mute)', cursor: 'pointer', lineHeight: 1 }}>×</span>
          </div>
          <div style={{ font: `400 11.5px/1.5 ${ET_UI}`, color: 'var(--ck-fg-2)', margin: '5px 0 9px' }}>
            <span style={{ font: `600 10.5px ${ET_MONO}`, color: 'var(--ck-fg-3)' }}>SKILLPACK_IP_VIOLATION</span> — the name asserts <span style={{ fontWeight: 600 }}>“thales”</span>, a restricted pattern.
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <span className="ck-btn ck-btn--primary" style={{ pointerEvents: 'none' }}>Override</span>
            <span className="ck-btn ck-btn--quiet" style={{ pointerEvents: 'none' }}>Dismiss</span>
          </div>
        </div>
      </div>
      <div style={{ font: `500 10px ${ET_MONO}`, color: 'var(--ck-fg-mute)', letterSpacing: '.04em' }}>severity error · stays in place · does not block the page</div>
    </div>
  </SurfaceCard>
);

// 4 · macOS alert sheet ---------------------------------------
const CardMac = () => (
  <SurfaceCard id="C-04" surface="macOS" tech="swiftui" hint="Show alert sheet with 'Override' option">
    <div style={{ flex: 1, background: 'linear-gradient(150deg,#1c2c47,#0a1322)', display: 'grid', placeItems: 'center', padding: 16 }}>
      <div style={{ width: 244, background: 'rgba(246,241,232,.97)', clipPath: 'polygon(8px 0,calc(100% - 8px) 0,100% 8px,100% calc(100% - 8px),calc(100% - 8px) 100%,8px 100%,0 calc(100% - 8px),0 8px)', padding: '16px 16px 13px', textAlign: 'center', boxShadow: '0 24px 50px rgba(0,0,0,.5)' }}>
        <div style={{ width: 34, height: 34, margin: '0 auto 9px', display: 'grid', placeItems: 'center', boxShadow: 'inset 0 0 0 2px var(--ck-alarm)', clipPath: 'polygon(7px 0,calc(100% - 7px) 0,100% 7px,100% calc(100% - 7px),calc(100% - 7px) 100%,7px 100%,0 calc(100% - 7px),0 7px)' }}>
          <span style={{ font: `600 17px ${ET_MONO}`, color: 'var(--ck-alarm)' }}>⊘</span>
        </div>
        <div style={{ font: `600 12.5px ${ET_UI}`, color: '#211B14' }}>Skill name contains a restricted pattern</div>
        <div style={{ font: `400 10.5px/1.45 ${ET_UI}`, color: '#4A4334', margin: '6px 0 13px' }}>“thales” is on the IP boundary list. Publishing requires an override waiver.</div>
        <div style={{ display: 'flex', gap: 7 }}>
          <span style={{ flex: 1, font: `600 11px ${ET_UI}`, color: '#211B14', boxShadow: 'inset 0 0 0 1px #CDC2AC', padding: '6px 0' }}>Cancel</span>
          <span style={{ flex: 1, font: `600 11px ${ET_UI}`, color: '#F6F1E8', background: 'var(--ck-rust)', padding: '6px 0' }}>Override</span>
        </div>
      </div>
    </div>
  </SurfaceCard>
);

// 5 · CNI -----------------------------------------------------
const CardCNI = () => (
  <SurfaceCard id="C-08" surface="CNI" tech="headless · ci/cd" hint="Emit JSON to stderr with exit code 78 (configuration error)">
    <Term title="ci runner — stderr">
      <TL c="#6B6457">$ skillpack-cni publish --json</TL>
      <TL c="#9A9284">{'{'}</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>"code":</span> <span style={{ color: '#F87171' }}>"SKILLPACK_IP_VIOLATION"</span>,</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>"category":</span> <span style={{ color: '#D2693A' }}>"security"</span>, <span style={{ color: '#9A9284' }}>"severity":</span> <span style={{ color: '#F87171' }}>"error"</span>,</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>"details":</span> {'{'} "violations": [<span style={{ color: '#D2693A' }}>"thales"</span>] {'}'}</TL>
      <TL c="#9A9284">{'}'}</TL>
      <TL c="#F87171" >exit 78  <span style={{ color: '#6B6457' }}>· configuration error</span></TL>
    </Term>
  </SurfaceCard>
);

// 6 · MCP (derived) -------------------------------------------
const CardMCP = () => (
  <SurfaceCard id="C-09" surface="MCP" tech="json-rpc · stdio" derived hint="Surface as a JSON-RPC error; security codes map to invalid-params (-32602)">
    <Term title="skillpack-mcp — stdio">
      <TL c="#9A9284"><span style={{ color: '#D2693A' }}>←</span> error {'{'}</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>code:</span> <span style={{ color: '#EAE5DA' }}>-32602</span>,</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>message:</span> <span style={{ color: '#D2693A' }}>"restricted pattern"</span>,</TL>
      <TL indent={12}><span style={{ color: '#9A9284' }}>data:</span> {'{'}</TL>
      <TL indent={24}><span style={{ color: '#9A9284' }}>skillpack:</span> <span style={{ color: '#F87171' }}>"SKILLPACK_IP_VIOLATION"</span>,</TL>
      <TL indent={24}><span style={{ color: '#9A9284' }}>category:</span> <span style={{ color: '#D2693A' }}>"security"</span> {'}'} {'}'}</TL>
    </Term>
  </SurfaceCard>
);

Object.assign(window, { SurfaceCard, CardCLI, CardVSCode, CardWeb, CardMac, CardCNI, CardMCP });
