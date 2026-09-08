// ============================================================
// SkillPack · Error Taxonomy — document
// DS-3 Ledger editorial surface. Shows the canonical ClientError
// (spec §5.1) as the sealed source of truth, the severity ladder
// (§5.2), and the same error realized across all six surfaces.
// The Evidence Margin keeps the books: reject (red) → waiver (violet).
// ============================================================

const { useState: useStateET } = React;
const MONO = "var(--ck-ff-mono)";
const UI = "var(--ck-ff-ui)";

// ---- canonical error, verbatim from the spec ----
const ERR = {
  code: 'SKILLPACK_IP_VIOLATION',
  category: 'security',
  severity: 'error',
  message: 'Skill name contains restricted pattern',
  violations: ['thales'],
};

const Label = ({ children, style }) => (
  <div style={{ font: `700 10px ${UI}`, letterSpacing: '.15em', textTransform: 'uppercase', color: 'var(--ck-fg-3)', ...style }}>{children}</div>
);

// ---- the canonical error · a sealed source-of-truth card ----
const CanonicalCard = () => (
  <div className="ck-sealed" style={{ padding: '22px 24px' }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap', marginBottom: 16 }}>
      <span style={{ font: `700 13px ${MONO}`, letterSpacing: '.02em', color: 'var(--ck-fg-1)' }}>{ERR.code}</span>
      <span className="ck-chip ck-chip--quarantined">⊘ security</span>
      <span className="ck-chip" style={{ borderColor: 'var(--ck-alarm)', color: 'var(--ck-alarm)' }}>error</span>
      <span className="ck-hash" style={{ marginLeft: 'auto' }}>sha256:9f3c…a217 ⊛</span>
    </div>
    {/* the JSON object */}
    <pre style={{ margin: 0, font: `400 12.5px/1.75 ${MONO}`, color: 'var(--ck-fg-2)', whiteSpace: 'pre-wrap' }}>
{`{
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"code"</span>{`:       `}<span style={{ color: 'var(--ck-alarm)' }}>"SKILLPACK_IP_VIOLATION"</span>{`,
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"category"</span>{`:   `}<span style={{ color: 'var(--ck-accent)' }}>"security"</span>{`,
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"severity"</span>{`:   `}<span style={{ color: 'var(--ck-alarm)' }}>"error"</span>{`,
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"message"</span>{`:    `}<span style={{ color: 'var(--ck-fg-1)' }}>"Skill name contains restricted pattern"</span>{`,
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"details"</span>{`:    { "violations": [`}<span style={{ color: 'var(--ck-accent)' }}>"thales"</span>{`] },
  `}<span style={{ color: 'var(--ck-fg-3)' }}>"clientHint"</span>{`: { cli, vscode, web, macos, cni }
}`}
    </pre>
    <div style={{ marginTop: 16, paddingTop: 14, borderTop: '1px solid var(--ck-hairline)', font: `400 13px/1.6 ${UI}`, color: 'var(--ck-fg-2)' }}>
      One object, generated once at the boundary. Each surface reads the same <span style={{ font: `500 12px ${MONO}`, color: 'var(--ck-fg-1)' }}>code</span> · <span style={{ font: `500 12px ${MONO}`, color: 'var(--ck-fg-1)' }}>severity</span> and renders its own <span style={{ font: `500 12px ${MONO}`, color: 'var(--ck-fg-1)' }}>clientHint</span> — never its own copy.
    </div>
  </div>
);

// ---- severity ladder (§5.2) ----
const SEV = [
  { k: 'fatal',   m: 'Unrecoverable; abort',     cli: 'exit 1 + red', cni: 'exit 1', tone: 'var(--ck-alarm)' },
  { k: 'error',   m: 'Operation failed',          cli: 'red text',     cni: 'exit 78', tone: 'var(--ck-alarm)', on: true },
  { k: 'warning', m: 'Degraded but continues',    cli: 'tone text',    cni: 'exit 0', tone: 'var(--ck-proof)' },
  { k: 'info',    m: 'Informational',             cli: 'gray text',    cni: 'stdout', tone: 'var(--ck-fg-3)' },
];
const SeverityLadder = () => (
  <div className="ck-quiet" style={{ padding: '18px 20px' }}>
    <Label style={{ marginBottom: 14 }}>Severity ladder · §5.2</Label>
    <div style={{ display: 'flex', flexDirection: 'column' }}>
      {SEV.map(s => (
        <div key={s.k} style={{ display: 'grid', gridTemplateColumns: '88px 1fr auto', alignItems: 'center', gap: 12, padding: '9px 0', borderBottom: '1px solid var(--ck-hairline)', opacity: s.on ? 1 : 0.7 }}>
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 7, font: `600 12px ${MONO}`, color: s.tone }}>
            <span style={{ width: 6, height: 6, background: s.tone, flexShrink: 0 }} />{s.k}
          </span>
          <span style={{ font: `400 12.5px ${UI}`, color: 'var(--ck-fg-2)' }}>{s.m}</span>
          <span style={{ font: `500 10.5px ${MONO}`, color: 'var(--ck-fg-mute)', whiteSpace: 'nowrap' }}>{s.cli} · {s.cni}</span>
        </div>
      ))}
    </div>
    <div style={{ marginTop: 12, font: `400 11.5px/1.5 ${UI}`, color: 'var(--ck-fg-3)' }}>
      The card below is severity <span style={{ font: `600 11px ${MONO}`, color: 'var(--ck-alarm)' }}>error</span> — failed, but recoverable with an override waiver.
    </div>
  </div>
);

// ---- Evidence Margin receipts for this event ----
const Receipt = ({ t, ev, detail, kind }) => (
  <div className={`ck-receipt${kind ? ' ck-receipt--' + kind : ''}`} style={{ paddingBottom: 12 }}>
    <span className="ck-receipt__rule" />
    <div style={{ paddingLeft: 0 }}>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
        <span className="ck-receipt__event">{ev}</span>
        <span className="ck-receipt__time" style={{ marginLeft: 'auto' }}>{t}</span>
      </div>
      <div className="ck-receipt__detail" style={{ marginTop: 2 }}>{detail}</div>
    </div>
  </div>
);
const EvidenceAside = () => (
  <aside style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
    <Label>Evidence margin</Label>
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <Receipt t="14:02:11" ev="generated" detail="publish requested · thales-helper" />
      <Receipt t="14:02:11" ev="evaluated" detail="IP boundary check · 1 pattern" />
      <Receipt t="14:02:11" ev="rejected" kind="alarm" detail="SKILLPACK_IP_VIOLATION · blocked" />
      <Receipt t="14:03:40" ev="signed" kind="proof" detail="override waiver · maintainer key" />
      <Receipt t="14:03:41" ev="approved" kind="proof" detail="publish proceeds with waiver" />
    </div>
    <div className="ck-stamp" style={{ marginTop: 4 }}>◆ waiver sealed · GAL-3</div>
    <div style={{ font: `400 11px/1.5 ${UI}`, color: 'var(--ck-fg-mute)' }}>
      The margin keeps the books: a red rejection and the violet waiver that lifted it are both permanent receipts.
    </div>
  </aside>
);

// ---- the six surfaces ----
const SurfaceGrid = () => (
  <section>
    <div style={{ display: 'flex', alignItems: 'baseline', gap: 12, marginBottom: 18 }}>
      <h2 style={{ margin: 0, font: `400 26px ${UI}`, fontFamily: 'var(--ck-ff-display)', color: 'var(--ck-fg-1)' }}>One error, six surfaces</h2>
      <span style={{ font: `500 11px ${MONO}`, color: 'var(--ck-fg-mute)' }}>each renders its own clientHint</span>
    </div>
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(338px, 1fr))', gap: 16 }}>
      <CardCLI /><CardVSCode /><CardWeb /><CardMac /><CardCNI /><CardMCP />
    </div>
  </section>
);

const ErrorTaxonomy = () => (
  <div style={{ maxWidth: 1200, margin: '0 auto', padding: '0 40px 80px' }}>
    {/* masthead */}
    <header style={{ display: 'flex', alignItems: 'flex-end', gap: 20, padding: '36px 0 26px', borderBottom: '1px solid var(--ck-hairline)', marginBottom: 32 }}>
      <div style={{ flex: 1 }}>
        <Label style={{ marginBottom: 8 }}>Unified client model · §5</Label>
        <h1 style={{ margin: 0, font: `400 46px/1.02 var(--ck-ff-display)`, color: 'var(--ck-fg-1)' }}>Error taxonomy</h1>
        <p className="ck-body" style={{ margin: '12px 0 0', maxWidth: '60ch' }}>A structured error is generated once at the boundary and carries a per-client hint. The palette is the policy: a security violation that blocks a publish earns the red mark — and the waiver that lifts it earns violet.</p>
      </div>
      <div style={{ textAlign: 'right' }}>
        <div style={{ font: `700 12px ${MONO}`, letterSpacing: '.18em', color: 'var(--ck-fg-1)' }}>SKILLPACK</div>
        <div style={{ font: `500 10px ${MONO}`, color: 'var(--ck-fg-mute)', marginTop: 4 }}>urn:ckodex:spec:client/0.1.0</div>
      </div>
    </header>

    {/* canonical + severity + receipts */}
    <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0,1fr) 304px', gap: 40, alignItems: 'start', marginBottom: 48 }}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
        <CanonicalCard />
        <SeverityLadder />
      </div>
      <EvidenceAside />
    </div>

    <SurfaceGrid />

    <footer style={{ marginTop: 48, paddingTop: 18, borderTop: '1px solid var(--ck-hairline)', display: 'flex', gap: 16, font: `500 11px ${MONO}`, color: 'var(--ck-fg-mute)' }}>
      <span>SkillPack v1.0 · CKODEX v16</span>
      <span>client-error-codes.md</span>
      <span style={{ marginLeft: 'auto' }}>5 hinted surfaces · 1 derived (MCP)</span>
    </footer>
  </div>
);

ReactDOM.createRoot(document.getElementById('root')).render(<ErrorTaxonomy />);
