// ============================================================
// SkillPack · Skill Glory — the dossier for one governed skill.
// A real DS-3 app shell (console: nav rail · main · Evidence Margin)
// with the AuthorityFooter provenance band. Everything is composed
// from the design system's own components — nothing hand-rolled that
// the DS already owns.
// ============================================================

const { PageShell, EvidenceMargin, StateChip, QuietCard, Button } = window.CkodexDesignSystem_042c8d;

const G_MONO = 'var(--ck-ff-mono)';
const G_UI = 'var(--ck-ff-ui)';
const G_DISP = 'var(--ck-ff-display)';

function ThemeChip() {
  const [theme, setTheme] = React.useState(() => document.documentElement.getAttribute('data-theme') || 'ledger');
  React.useEffect(() => { document.documentElement.setAttribute('data-theme', theme); }, [theme]);
  const opts = [['ledger', 'Ledger'], ['vault', 'Vault'], ['hc', 'HC']];
  return (
    <div role="radiogroup" aria-label="Theme" style={{ display: 'inline-flex', border: '1px solid var(--ck-hairline-strong)' }}>
      {opts.map(([v, l], i) => (
        <button key={v} role="radio" aria-checked={theme === v} onClick={() => setTheme(v)}
          style={{ cursor: 'pointer', font: `600 10px ${G_MONO}`, letterSpacing: '.09em', textTransform: 'uppercase', padding: '6px 11px', border: 'none', borderLeft: i ? '1px solid var(--ck-hairline-strong)' : 'none', background: theme === v ? 'var(--ck-fg-1)' : 'transparent', color: theme === v ? 'var(--ck-bg-1)' : 'var(--ck-fg-3)' }}>{l}</button>
      ))}
    </div>
  );
}

const Eyebrow = ({ children, style }) => (
  <div style={{ font: `600 10px ${G_MONO}`, letterSpacing: '.18em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', ...style }}>{children}</div>
);

// nav rail
function Nav() {
  const items = [
    ['skill', 'pdf-extract', true],
    ['chain', 'Attestation chain', false],
    ['gauge', 'Conformance', false],
    ['guardrail', 'Guardrails', false],
    ['audit', 'Audit trail', false],
    ['export', 'Install', false],
  ];
  return (
    <React.Fragment>
      <div style={{ font: `600 9px ${G_MONO}`, letterSpacing: '.16em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', padding: '4px 20px 12px' }}>Dossier</div>
      {items.map(([icon, label, active]) => (
        <a key={label} className={'ck-nav__item' + (active ? ' ck-nav__item--active' : '')} href="#" aria-current={active ? 'page' : undefined}>
          <CkIcon name={icon} size={18} />
          <span style={{ font: `500 13px ${G_UI}` }}>{label}</span>
        </a>
      ))}
      <div style={{ marginTop: 'auto', padding: '20px' }}>
        <a href="index.html" style={{ font: `500 11px ${G_MONO}`, color: 'var(--ck-fg-3)', textDecoration: 'none', borderBottom: 'none' }}>← back to registry</a>
      </div>
    </React.Fragment>
  );
}

// lifecycle progress
function Lifecycle() {
  const stages = ['claimed', 'scanned', 'validated', 'proved', 'published', 'admitted'];
  const done = 6;
  return (
    <div style={{ display: 'flex', gap: 0, border: '1px solid var(--ck-hairline-strong)' }}>
      {stages.map((s, i) => {
        const proved = i >= 3;
        return (
          <div key={s} style={{ flex: 1, padding: '11px 14px', borderRight: i < stages.length - 1 ? '1px solid var(--ck-hairline)' : 'none', background: i < done ? 'var(--ck-bg-1)' : 'var(--ck-bg-0)' }}>
            <div style={{ font: `600 9px ${G_MONO}`, color: 'var(--ck-fg-mute)' }}>{String(i + 1).padStart(2, '0')}</div>
            <div style={{ font: `600 11px ${G_MONO}`, letterSpacing: '.04em', textTransform: 'uppercase', color: proved ? 'var(--ck-proof)' : 'var(--ck-fg-2)', marginTop: 5 }}>{proved ? '◆' : '○'} {s}</div>
          </div>
        );
      })}
    </div>
  );
}

function FactRow({ k, v, mono = true, tone }) {
  return (
    <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: 20, padding: '10px 0', borderBottom: '1px solid var(--ck-hairline)' }}>
      <span style={{ font: `600 10px ${G_MONO}`, letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', flexShrink: 0 }}>{k}</span>
      <span style={{ font: mono ? `500 12px ${G_MONO}` : `400 13px ${G_UI}`, color: tone || 'var(--ck-fg-1)', textAlign: 'right' }}>{v}</span>
    </div>
  );
}

function Capability({ icon, title, desc }) {
  return (
    <QuietCard style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 9, minHeight: 118 }}>
      <CkIcon name={icon} size={20} style={{ color: 'var(--ck-fg-1)' }} />
      <div style={{ font: `600 13px ${G_UI}`, color: 'var(--ck-fg-1)' }}>{title}</div>
      <div style={{ font: `400 12px/1.5 ${G_UI}`, color: 'var(--ck-fg-3)' }}>{desc}</div>
    </QuietCard>
  );
}

function GloryMain() {
  const digest = 'sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217';
  return (
    <div id="dossier" style={{ display: 'flex', flexDirection: 'column', gap: 40 }}>
      {/* identity */}
      <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0,1fr) auto', gap: 40, alignItems: 'start' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
            <CkIcon name="skill" size={26} style={{ color: 'var(--ck-fg-1)' }} />
            <Eyebrow style={{ margin: 0 }}>Skill · sealed artifact</Eyebrow>
            <StateChip state="attested">attested</StateChip>
          </div>
          <h1 style={{ margin: 0, font: `400 52px/1 ${G_DISP}`, color: 'var(--ck-fg-1)' }}>pdf-extract</h1>
          <div style={{ font: `500 13px ${G_MONO}`, color: 'var(--ck-fg-3)', marginTop: 12 }}>urn:ckodex:skill:pdf-extract@2.4.1 · owner ckodex-core</div>
          <p style={{ margin: '20px 0 0', maxWidth: '60ch', font: `400 15px/1.65 ${G_UI}`, color: 'var(--ck-fg-2)' }}>
            Extracts structured text, tables, and figure anchors from PDF documents into a canonical evidence body. Deterministic over its input: the same document yields the same content digest, so downstream attestations stay stable. Runs kernel-sealed — the extraction cannot reach the network, and its output carries the proof of how it was produced.
          </p>
          <div style={{ display: 'flex', gap: 10, marginTop: 24, flexWrap: 'wrap' }}>
            <Button variant="primary">Install · verify first</Button>
            <Button variant="quiet">Inspect evidence bundle</Button>
          </div>
        </div>
        <CkCiqr payload={'urn:ckodex:skill:pdf-extract@2.4.1'} size={132} attested label="Skill CIQR" caption="scan to resolve the sealed bundle" />
      </div>

      {/* lifecycle */}
      <div>
        <Eyebrow style={{ marginBottom: 12 }}>Lifecycle · all stages discharged</Eyebrow>
        <Lifecycle />
      </div>

      {/* two-column: facts + capabilities */}
      <div style={{ display: 'grid', gridTemplateColumns: '340px 1fr', gap: 40, alignItems: 'start' }}>
        <div>
          <Eyebrow style={{ marginBottom: 6 }}>Identity &amp; proof</Eyebrow>
          <FactRow k="version" v="2.4.1" />
          <FactRow k="kind" v="skill · deterministic" />
          <FactRow k="evidence tier" v="◆ E5 · proved" tone="var(--ck-proof)" />
          <FactRow k="content digest" v="sha256:9f3c…a217" tone="var(--ck-proof)" />
          <FactRow k="sealed" v="2026-07-02 09:41Z" />
          <FactRow k="transparency" v="rekor.ckodex.com/…/9f3c" />
        </div>
        <div>
          <Eyebrow style={{ marginBottom: 12 }}>Interface</Eyebrow>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: 12 }}>
            <Capability icon="digest" title="Canonical body" desc="Headings, paragraphs, fenced code — whitespace-normalized for stable hashing." />
            <Capability icon="matrix" title="Table extraction" desc="Grid regions recovered as row/column cells with anchor coordinates." />
            <Capability icon="proof" title="Sealed output" desc="Every result ships with its content digest and the proof of production." />
            <Capability icon="quarantine" title="No egress" desc="Runs with the network boundary closed — egress deny-by-default." />
            <Capability icon="replay" title="Deterministic" desc="Same input, same digest — replayable and independently verifiable." />
            <Capability icon="gauge" title="Conformance" desc="Passes the full extraction vector set at each promotion gate." />
          </div>
        </div>
      </div>

      {/* guardrails enforced */}
      <div>
        <Eyebrow style={{ marginBottom: 12 }}>Guardrails enforced at run</Eyebrow>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(230px, 1fr))', gap: 16 }}>
          <EntityCard kind="guardrail" name="egress-deny" urn="urn:ckodex:guardrail:egress-deny@3.2" owner="ckodex-gov" version="3.2.0" sealedAt="2026-07-05" tier="E5" digest="sha256:a217f0d3841e2b9c7a56e08d4f1b3c2e99017abc" ciqr={CiqrPath('urn:ckodex:guardrail:egress-deny@3.2')} />
          <EntityCard kind="guardrail" name="pii-redact" urn="urn:ckodex:guardrail:pii-redact@2.0" owner="privacy" version="2.0.1" sealedAt="2026-06-30" tier="E5" digest="sha256:33f1a9e04b8ad2f19c7e6a3d0f52b81e9a4c7d60" ciqr={CiqrPath('urn:ckodex:guardrail:pii-redact@2.0')} />
          <EntityCard kind="guardrail" name="size-cap" urn="urn:ckodex:guardrail:size-cap@1.1" owner="platform" version="1.1.0" tier="E3" />
        </div>
      </div>
    </div>
  );
}

function GloryPage() {
  const receipts = [
    { time: '2026-07-02 09:38:04', event: 'Generated', details: ['author: ckodex-core', 'src: pdf-extract@2.4.1'], kind: 'routine' },
    { time: '2026-07-02 09:39:12', event: 'Evaluated', details: ['conformance: 418/418 vectors', 'gate: promote/e3'], kind: 'routine' },
    { time: '2026-07-02 09:40:51', event: 'Attested', details: ['kernel: ck-kernel@16', 'sha256:9f3c…a217'], kind: 'proof' },
    { time: '2026-07-02 09:41:07', event: 'Signed', details: ['ed25519:MEUCIQ…', 'rekor: entry 9f3c'], kind: 'proof' },
    { time: '2026-07-02 09:41:20', event: 'Approved', details: ['policy: registry-charter', 'mode: enforce'], kind: 'routine' },
  ];
  const proof = {
    urn: 'urn:ckodex:skill:pdf-extract@2.4.1',
    signature: 'ed25519:MEUCIQD9f3c1d77e4a20b8f5c6e2a9184d3f0b7',
    evbRef: 'urn:ckodex:evb:pdf-extract-2.4.1',
    rekorUrl: 'https://rekor.ckodex.com/api/v1/log/entries/9f3c',
    sealedAt: '2026-07-02 09:41:07Z',
    digest: 'sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217',
    ciqr: CiqrPath('urn:ckodex:skill:pdf-extract@2.4.1'),
  };
  return (
    <React.Fragment>
      <a className="ckr-skip" href="#dossier">Skip to content</a>
      <PageShell
        variant="console"
        brand="SKILLPACK"
        crumb="Skill dossier"
        urn="urn:ckodex:skill:pdf-extract@2.4.1"
        headerRight={<ThemeChip />}
        nav={<Nav />}
        margin={<EvidenceMargin title="Evidence margin" entries={receipts} stamp="sha256:9f3c…a217" />}
        footerLeft="SkillPack v1.0 · CKODEX v16"
        footerRight="6 receipts · 1 proof object"
      >
        <GloryMain />
      </PageShell>
      <AuthorityFooter
        level="sealed"
        classification="internal"
        tier="E5"
        name="pdf-extract@2.4.1 · ckodex-core"
        authority="ckodex-gov"
        policySet="registry-charter"
        environment="production"
        mode="enforce"
        proof={proof}
        digest={proof.digest}
        artifact="#dossier"
        copyForAI
      />
    </React.Fragment>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<GloryPage />);
