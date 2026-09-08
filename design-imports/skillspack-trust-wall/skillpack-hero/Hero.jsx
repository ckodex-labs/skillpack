// ============================================================
// SkillPack · Ecosystem Hero — the front door of the registry.
// DS-3 Evidence Editorial. Warm paper, ink structure, a closed
// budget. The story: a skill is not trusted because it is popular —
// it is trusted because it is proved. Every noun in the supply
// chain is a governed artifact carrying its own evidence.
// ============================================================

const H_MONO = 'var(--ck-ff-mono)';
const H_UI = 'var(--ck-ff-ui)';
const H_DISP = 'var(--ck-ff-display)';

// ---- theme switch (ledger / vault / hc) ----
function ThemeSwitch() {
  const [theme, setTheme] = React.useState(() => document.documentElement.getAttribute('data-theme') || 'ledger');
  React.useEffect(() => { document.documentElement.setAttribute('data-theme', theme); }, [theme]);
  const opts = [['ledger', 'Ledger'], ['vault', 'Vault'], ['hc', 'HC']];
  return (
    <div role="radiogroup" aria-label="Theme" style={{ display: 'inline-flex', border: '1px solid var(--ck-hairline-strong)' }}>
      {opts.map(([v, l], i) => (
        <button key={v} role="radio" aria-checked={theme === v} onClick={() => setTheme(v)}
          style={{ cursor: 'pointer', font: `600 10px ${H_MONO}`, letterSpacing: '.1em', textTransform: 'uppercase', padding: '7px 12px', border: 'none', borderLeft: i ? '1px solid var(--ck-hairline-strong)' : 'none', background: theme === v ? 'var(--ck-fg-1)' : 'transparent', color: theme === v ? 'var(--ck-bg-1)' : 'var(--ck-fg-3)' }}>{l}</button>
      ))}
    </div>
  );
}

// ---- faint engineering dot field ----
const HeroDots = ({ children, style }) => (
  <div style={{ position: 'relative', backgroundImage: 'radial-gradient(color-mix(in oklab, var(--ck-fg-1) 11%, transparent) 1px, transparent 1.4px)', backgroundSize: '22px 22px', backgroundPosition: '-1px -1px', ...style }}>{children}</div>
);

const Kicker = ({ children, style }) => (
  <div style={{ font: `600 10px ${H_MONO}`, letterSpacing: '.2em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', ...style }}>{children}</div>
);

// ---- supply-chain lifecycle rail ----
function SupplyChain() {
  const steps = [
    ['skill', 'Author', 'a skill is written', '○ claimed'],
    ['digest', 'Scan', 'checks run, digest taken', '◌ scanned'],
    ['gauge', 'Validate', 'conformance vectors pass', '● validated'],
    ['proof', 'Seal', 'kernel discharges the proof', '◆ proved'],
    ['chain', 'Register', 'attestation lands in the registry', '◆ published'],
    ['export', 'Install', 'the consumer verifies before run', '⊢ admitted'],
  ];
  return (
    <div style={{ display: 'grid', gridTemplateColumns: `repeat(${steps.length}, 1fr)`, border: '1px solid var(--ck-hairline-strong)', borderRight: 'none' }}>
      {steps.map(([icon, name, desc, state], i) => {
        const proved = state.startsWith('◆') || state.startsWith('⊢');
        return (
          <div key={name} style={{ position: 'relative', padding: '20px 18px 18px', borderRight: '1px solid var(--ck-hairline-strong)', background: 'var(--ck-bg-1)', display: 'flex', flexDirection: 'column', gap: 12, minHeight: 168 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <CkIcon name={icon} size={22} style={{ color: proved && (icon === 'proof' || icon === 'chain') ? 'var(--ck-proof)' : 'var(--ck-fg-1)' }} />
              <span style={{ font: `600 10px ${H_MONO}`, color: 'var(--ck-fg-mute)' }}>{String(i + 1).padStart(2, '0')}</span>
            </div>
            <div style={{ marginTop: 'auto' }}>
              <div style={{ font: `400 19px/1 ${H_DISP}`, color: 'var(--ck-fg-1)' }}>{name}</div>
              <div style={{ font: `500 11px/1.4 ${H_MONO}`, color: 'var(--ck-fg-3)', marginTop: 6, letterSpacing: '.01em' }}>{desc}</div>
            </div>
            <div style={{ font: `600 10px ${H_MONO}`, letterSpacing: '.06em', textTransform: 'uppercase', color: proved ? 'var(--ck-proof)' : 'var(--ck-fg-mute)', borderTop: '1px solid var(--ck-hairline)', paddingTop: 9 }}>{state}</div>
          </div>
        );
      })}
    </div>
  );
}

// ---- the governed nouns, as entity cards ----
function EcosystemCards() {
  const q = (s) => CiqrPath(s);
  const cards = [
    { kind: 'skill', name: 'pdf-extract', urn: 'urn:ckodex:skill:pdf-extract@2.4.1', owner: 'ckodex-core', version: '2.4.1', sealedAt: '2026-07-02', tier: 'E5', digest: 'sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217', ciqr: q('urn:ckodex:skill:pdf-extract@2.4.1') },
    { kind: 'aipack', name: 'regulatory-suite', urn: 'urn:ckodex:aipack:regulatory-suite@1.0', owner: 'gov-tools', version: '1.0.0', sealedAt: '2026-06-28', tier: 'E5', digest: 'sha256:4b8ad2f19c7e6a3d0f52b81e9a4c7d6033f1a9e0', ciqr: q('urn:ckodex:aipack:regulatory-suite@1.0') },
    { kind: 'guardrail', name: 'egress-deny', urn: 'urn:ckodex:guardrail:egress-deny@3.2', owner: 'ckodex-gov', version: '3.2.0', sealedAt: '2026-07-05', tier: 'E5', digest: 'sha256:a217f0d3841e2b9c7a56e08d4f1b3c2e99017abc', ciqr: q('urn:ckodex:guardrail:egress-deny@3.2') },
    { kind: 'model', name: 'claim-classifier', urn: 'urn:ckodex:model:claim-classifier@0.9', owner: 'research', version: '0.9.3', tier: 'E3' },
    { kind: 'agent', name: 'triage-runner', urn: 'urn:ckodex:agent:triage-runner@1.4', owner: 'ops', version: '1.4.0', tier: 'E1' },
    { kind: 'system', name: 'evidence-fabric', urn: 'urn:ckodex:system:evidence-fabric@16', owner: 'platform', version: '16.0.0', sealedAt: '2026-07-01', tier: 'E5', digest: 'sha256:1d4477e9a20b8f5c6e2a91b3c2e99017a9e0f0d3', ciqr: q('urn:ckodex:system:evidence-fabric@16') },
  ];
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
      {cards.map((c) => <EntityCard key={c.name} {...c} />)}
    </div>
  );
}

function Metric({ figure, label, sub, tone }) {
  return (
    <div style={{ padding: '22px 20px', background: 'var(--ck-bg-1)', border: '1px solid var(--ck-hairline-strong)', display: 'flex', flexDirection: 'column', gap: 6, minHeight: 132, justifyContent: 'space-between' }}>
      <Kicker>{label}</Kicker>
      <div>
        <div style={{ font: `400 46px/0.95 ${H_DISP}`, color: tone || 'var(--ck-fg-1)', fontVariantNumeric: 'tabular-nums' }}>{figure}</div>
        <div style={{ font: `500 11px/1.4 ${H_MONO}`, color: 'var(--ck-fg-3)', marginTop: 8 }}>{sub}</div>
      </div>
    </div>
  );
}

function Hero() {
  const heroCiqr = CiqrPath('urn:ckodex:registry:skillpack · verify at rekor.ckodex.com');
  return (
    <div style={{ maxWidth: 1240, margin: '0 auto', padding: '0 40px' }}>
      {/* masthead */}
      <header style={{ display: 'flex', alignItems: 'center', gap: 20, padding: '22px 0', borderBottom: '1px solid var(--ck-hairline)' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 12 }}>
          <span style={{ font: `700 15px ${H_MONO}`, letterSpacing: '.18em', color: 'var(--ck-fg-1)' }}>SKILLPACK</span>
          <span style={{ font: `500 11px ${H_MONO}`, color: 'var(--ck-fg-mute)' }}>the governed skills registry</span>
        </div>
        <nav style={{ marginLeft: 'auto', display: 'flex', gap: 22, alignItems: 'center' }}>
          {['Registry', 'Skills', 'Guardrails', 'Docs'].map((l) => (
            <a key={l} href="#" style={{ font: `500 12px ${H_UI}`, color: 'var(--ck-fg-2)', textDecoration: 'none', borderBottom: 'none' }}>{l}</a>
          ))}
          <a href="glory.html" style={{ font: `600 12px ${H_UI}`, color: 'var(--ck-accent)', textDecoration: 'none', borderBottom: 'none' }}>View a skill →</a>
          <ThemeSwitch />
        </nav>
      </header>

      {/* hero */}
      <HeroDots style={{ borderBottom: '2px solid var(--ck-fg-1)', margin: '0 -40px', padding: '0 40px' }}>
        <div style={{ padding: '76px 0 68px', display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) auto', gap: 56, alignItems: 'center' }}>
          <div>
            <Kicker style={{ marginBottom: 22 }}>Ckodex v16 · evidence fabric</Kicker>
            <h1 style={{ margin: 0, font: `400 clamp(46px, 6vw, 76px)/0.98 ${H_DISP}`, color: 'var(--ck-fg-1)', letterSpacing: '-0.01em' }}>
              A skill earns trust<br />by being <em style={{ fontStyle: 'italic', color: 'var(--ck-proof)' }}>proved</em>,<br />never by being popular.
            </h1>
            <p style={{ margin: '26px 0 0', maxWidth: '58ch', font: `400 16px/1.62 ${H_UI}`, color: 'var(--ck-fg-2)' }}>
              SkillPack is the registry where every skill, pack, guardrail, and model moves through the same closed lifecycle — authored, scanned, validated, and sealed with a kernel proof. Nothing crosses a boundary without an evidence bundle. Online or airgapped, the books always balance.
            </p>
            <div style={{ display: 'flex', gap: 12, marginTop: 32, alignItems: 'center', flexWrap: 'wrap' }}>
              <a href="glory.html" className="ck-btn ck-btn--primary" style={{ textDecoration: 'none' }}>Browse the registry</a>
              <a href="#lifecycle" className="ck-btn ck-btn--quiet" style={{ textDecoration: 'none' }}>How sealing works</a>
              <span className="ck-invariant" style={{ font: `600 11px ${H_MONO}`, color: 'var(--ck-fg-3)', marginLeft: 8 }}>nothing crosses a boundary without proof</span>
            </div>
          </div>
          <div style={{ flexShrink: 0 }}>
            <CkCiqr payload={'urn:ckodex:registry:skillpack · verify at rekor.ckodex.com'} size={150} attested label="Registry CIQR" caption="scan to resolve the canonical registry root" />
          </div>
        </div>
      </HeroDots>

      {/* metrics strip */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, margin: '40px 0 8px' }}>
        <Metric figure="1,284" label="Sealed artifacts" sub="◆ E5 · kernel-proved" tone="var(--ck-proof)" />
        <Metric figure="6" label="Governed kinds" sub="skill · aipack · guardrail · model · agent · system" />
        <Metric figure="100%" label="Provenance coverage" sub="every install verified pre-run" />
        <Metric figure="0" label="Unproven in prod" sub="⊘ egress deny-by-default" />
      </div>

      {/* lifecycle */}
      <section id="lifecycle" style={{ margin: '56px 0 0' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 18 }}>
          <span style={{ font: `500 13px ${H_MONO}`, color: 'var(--ck-fg-mute)' }}>01</span>
          <h2 style={{ margin: 0, font: `400 30px ${H_DISP}`, color: 'var(--ck-fg-1)' }}>The supply chain is a closed lifecycle</h2>
          <span style={{ marginLeft: 'auto', font: `500 12px ${H_MONO}`, color: 'var(--ck-fg-3)', maxWidth: 300, textAlign: 'right' }}>six stages · one evidence trail · no skips</span>
        </div>
        <SupplyChain />
      </section>

      {/* the registry */}
      <section style={{ margin: '56px 0 0' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 18 }}>
          <span style={{ font: `500 13px ${H_MONO}`, color: 'var(--ck-fg-mute)' }}>02</span>
          <h2 style={{ margin: 0, font: `400 30px ${H_DISP}`, color: 'var(--ck-fg-1)' }}>Every noun is a governed artifact</h2>
          <span style={{ marginLeft: 'auto', font: `500 12px ${H_MONO}`, color: 'var(--ck-fg-3)', maxWidth: 320, textAlign: 'right' }}>a digest earns the ◆ seal · without one, the card says ○ unattested</span>
        </div>
        <EcosystemCards />
      </section>

      <div style={{ height: 72 }} />
    </div>
  );
}

// mount hero + a provenance authority footer over the whole page
function HeroPage() {
  const proof = {
    urn: 'urn:ckodex:registry:skillpack@16.0',
    signature: 'ed25519:MEUCIQD9f3c1d77e4a20b8f5c6e2a9184d3f0b7',
    evbRef: 'urn:ckodex:evb:skillpack-root',
    rekorUrl: 'https://rekor.ckodex.com/api/v1/log/entries/9f3c',
    sealedAt: '2026-07-06 08:14:02Z',
    ciqr: CiqrPath('urn:ckodex:registry:skillpack@16.0'),
  };
  return (
    <React.Fragment>
      <a className="ckr-skip" href="#lifecycle">Skip to content</a>
      <Hero />
      <AuthorityFooter
        level="sealed"
        classification="public"
        tier="E5"
        name="SkillPack registry root · ckodex-gov"
        authority="ckodex-gov"
        policySet="registry-charter"
        environment="production"
        mode="enforce"
        proof={proof}
        digest={proof.signature && 'sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217'}
        copyForAI
        defaultExpanded={false}
      />
    </React.Fragment>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<HeroPage />);
