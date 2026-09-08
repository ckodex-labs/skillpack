// ============================================================
// SkillPack · Transparency Fabric — document assembly
// DS-3 Ledger architecture brief. Six concepts, one fabric.
// ============================================================

const Divider = () => <div style={{ height: 1, background: 'var(--ck-hairline)', margin: '52px 0' }} />;

const TransparencyFabric = () => (
  <div style={{ maxWidth: 1200, margin: '0 auto', padding: '0 40px 96px' }}>
    {/* masthead */}
    <header style={{ display: 'flex', alignItems: 'flex-end', gap: 24, padding: '40px 0 28px', borderBottom: '2px solid var(--ck-fg-1)', marginBottom: 14 }}>
      <div style={{ flex: 1 }}>
        <div style={{ font: `700 10px ${TF_UI}`, letterSpacing: '.16em', textTransform: 'uppercase', color: 'var(--ck-fg-3)', marginBottom: 10 }}>CKODEX v16 · transparency subsystem</div>
        <h1 style={{ margin: 0, font: `400 52px/1 ${TF_DISP}`, color: 'var(--ck-fg-1)' }}>Transparency Fabric</h1>
        <p className="ck-body" style={{ margin: '14px 0 0', maxWidth: '64ch' }}>Six components, one guarantee: nothing crosses a boundary without proof. The kernel enforces it, the graph records it, the exchange moves it, the wall admits on it, the station gates on it — online or airgapped, the books always balance.</p>
      </div>
      <div style={{ textAlign: 'right', flexShrink: 0 }}>
        <div style={{ font: `700 13px ${TF_MONO}`, letterSpacing: '.18em', color: 'var(--ck-fg-1)' }}>SKILLPACK</div>
        <div style={{ font: `500 10px ${TF_MONO}`, color: 'var(--ck-fg-mute)', marginTop: 5 }}>urn:ckodex:fabric/16.0</div>
      </div>
    </header>

    {/* component index strip */}
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginBottom: 48 }}>
      {[
        ['01', 'Fabric Kernel'], ['02', 'Transparency Graph'], ['03', 'Transparency Exchange'],
        ['04', 'TrustWall'], ['05', 'Transition Station'], ['06', 'Online vs Airgap'],
      ].map(([n, t]) => (
        <span key={n} style={{ display: 'inline-flex', alignItems: 'baseline', gap: 8, padding: '7px 12px', border: '1px solid var(--ck-hairline)', font: `500 12px ${TF_UI}`, color: 'var(--ck-fg-2)' }}>
          <span style={{ font: `600 10px ${TF_MONO}`, color: 'var(--ck-fg-mute)' }}>{n}</span>{t}
        </span>
      ))}
    </div>

    <FabricKernel />
    <Divider />
    <TransparencyGraph />
    <Divider />
    <TransparencyExchange />
    <Divider />
    <TrustWall />
    <Divider />
    <TransitionStation />
    <Divider />
    <OnlineAirgap />

    <footer style={{ marginTop: 60, paddingTop: 18, borderTop: '1px solid var(--ck-hairline)', display: 'flex', gap: 16, font: `500 11px ${TF_MONO}`, color: 'var(--ck-fg-mute)', flexWrap: 'wrap' }}>
      <span>SkillPack v1.0 · CKODEX v16 · STX v0.1.0</span>
      <span className="ck-invariant" style={{ color: 'var(--ck-fg-3)' }}>nothing crosses a boundary without proof</span>
      <span style={{ marginLeft: 'auto' }}>6 components · 1 fabric</span>
    </footer>
  </div>
);

ReactDOM.createRoot(document.getElementById('root')).render(<TransparencyFabric />);
