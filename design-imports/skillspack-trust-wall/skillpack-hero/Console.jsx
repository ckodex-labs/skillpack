// ============================================================
// SkillPack · Management Console — shell + router + overlays.
// A real DS-3 console: nav rail · main · collapsible Evidence
// Margin · authority footer. Responsive to a single column and a
// slide-in nav on mobile. Profile / Settings / Accessibility drawers.
// ============================================================

const CONSOLE_SECTIONS = [
  { id: 'overview', icon: 'system', label: 'Root fabric', crumb: 'Root fabric hierarchy', render: () => <FabricPanel /> },
  { id: 'registry', icon: 'digest', label: 'Registry', crumb: 'Registry', render: () => <RegistryPanel /> },
  { id: 'assessment', icon: 'matrix', label: 'Assessment', crumb: 'Assessment', render: () => <AssessmentPanel /> },
  { id: 'trustwall', icon: 'gate', label: 'TrustWall', crumb: 'TrustWall', render: () => <TrustWallPanel /> },
  { id: 'io', icon: 'export', label: 'Import / Export', crumb: 'Import / Export', render: () => <ImportExportPanel /> },
  { id: 'migration', icon: 'replay', label: 'Migration', crumb: 'Migration', render: () => <MigrationPanel /> },
  { id: 'transmission', icon: 'chain', label: 'Transmission', crumb: 'Transmission', render: () => <TransmissionPanel /> },
  { id: 'airgap', icon: 'quarantine', label: 'Airgap', crumb: 'Online vs airgap', render: () => <AirgapPanel /> },
];

const ENV_LEVEL = { development: 'open', staging: 'restricted', production: 'sealed', incident: 'contained' };

function Toggle({ on, onChange, label }) {
  return (
    <button type="button" className="cx-toggle" aria-pressed={on} onClick={() => onChange(!on)}>
      <span className="cx-toggle__track"><span className="cx-toggle__knob" /></span>
      <span className="cx-toggle__label">{label}</span>
    </button>
  );
}

function Seg({ value, options, onChange }) {
  return (
    <div className="cx-seg">
      {options.map((o) => <button key={o} className="cx-seg__b" aria-pressed={value === o} onClick={() => onChange(o)}>{o}</button>)}
    </div>
  );
}

function Drawer({ title, onClose, children }) {
  React.useEffect(() => {
    const onKey = (e) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);
  return (
    <React.Fragment>
      <div className="cx-scrim" onClick={onClose} />
      <aside className="cx-drawer" role="dialog" aria-modal="true" aria-label={title}>
        <div className="cx-drawer__head">
          <span className="cx-drawer__title">{title}</span>
          <button className="cx-x" onClick={onClose} aria-label="Close">✕</button>
        </div>
        <div className="cx-drawer__body">{children}</div>
      </aside>
    </React.Fragment>
  );
}

function Setting({ k, desc, children }) {
  return (
    <div className="cx-set">
      <span className="cx-set__k">{k}</span>
      {desc ? <span className="cx-set__desc">{desc}</span> : null}
      <div style={{ marginTop: 2 }}>{children}</div>
    </div>
  );
}

function ProfileDrawer({ onClose }) {
  return (
    <Drawer title="Operator profile" onClose={onClose}>
      <EntityCard kind="agent" name="a. mercer" urn="urn:ckodex:operator:a.mercer" owner="ckodex-gov" version="session·8f2c" tier="E5" digest="sha256:7c21e2a2179f3c1d77e4a20b8f5c6e2a9184d3f0" ciqr={CiqrPath('urn:ckodex:operator:a.mercer')} />
      <Setting k="Roles" desc="what this operator may assert">
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          {['registry-admin', 'sealer', 'auditor'].map((r) => <span key={r} className="cx-chip" aria-pressed="true" style={{ cursor: 'default' }}>{r}</span>)}
        </div>
      </Setting>
      <Setting k="Active session" desc="mode-bound · expires with the policy set">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          <FactLine k="authority" v="⊢ ckodex-gov" />
          <FactLine k="opened" v="2026-07-10 08:02Z" />
          <FactLine k="signing key" v="ed25519:MEUCIQ…" />
        </div>
      </Setting>
      <button className="ck-btn ck-btn--quiet" style={{ width: '100%' }}>Sign out · seal session</button>
    </Drawer>
  );
}

function SettingsDrawer({ onClose, state, set }) {
  return (
    <Drawer title="Console settings" onClose={onClose}>
      <Setting k="Governance mode" desc="mode changes deployment, not governance semantics">
        <Seg value={state.mode} options={['enforce', 'monitor']} onChange={(v) => set({ mode: v })} />
      </Setting>
      <Setting k="Environment" desc="resolves the authority handling level">
        <Seg value={state.env} options={['development', 'staging', 'production', 'incident']} onChange={(v) => set({ env: v })} />
        <div style={{ marginTop: 10 }}><FactLine k="handling level" v={ENV_LEVEL[state.env]} /></div>
      </Setting>
      <Setting k="Policy set" desc="the charter this console enforces against">
        <input className="cx-field" value={state.policy} onChange={(e) => set({ policy: e.target.value })} aria-label="Policy set" />
      </Setting>
      <Setting k="Ledger density" desc="breathing room, or a dense hairline ledger">
        <Toggle on={state.dense} onChange={(v) => set({ dense: v })} label={state.dense ? 'Dense seams' : 'Breathing room'} />
      </Setting>
    </Drawer>
  );
}

function AccessibilityDrawer({ onClose, state, set }) {
  return (
    <Drawer title="Accessibility" onClose={onClose}>
      <Setting k="Theme" desc="every surface resolves in all four — Ledger, Vault, High-contrast, and OS forced-colors">
        <Seg value={state.theme} options={['ledger', 'vault', 'hc']} onChange={(v) => set({ theme: v })} />
      </Setting>
      <Setting k="Text size" desc="scales the whole console on the proof-grid">
        <Seg value={state.textScale} options={['S', 'M', 'L']} onChange={(v) => set({ textScale: v })} />
      </Setting>
      <Setting k="Motion" desc="the Resolve ceremony and panel transitions">
        <Toggle on={state.reduce} onChange={(v) => set({ reduce: v })} label={state.reduce ? 'Reduced' : 'Full motion'} />
      </Setting>
      <Setting k="Focus & forced colors">
        <p className="cx-set__desc" style={{ margin: 0 }}>Focus rings use the accent (gold under HC). When your OS requests forced colors, the system palette always wins — the console remaps ground, ink, and links to system keywords automatically.</p>
      </Setting>
    </Drawer>
  );
}

function IconBtn({ icon, label, onClick }) {
  return <button className="cx-iconbtn" onClick={onClick} aria-label={label} title={label}><CkIcon name={icon} size={17} /></button>;
}

function Console() {
  const [section, setSection] = React.useState('overview');
  const [drawer, setDrawer] = React.useState(null);
  const [navOpen, setNavOpen] = React.useState(false);
  const [state, setState] = React.useState({ mode: 'enforce', env: 'production', policy: 'registry-charter@4', dense: false, theme: 'ledger', textScale: 'M', reduce: false });
  const set = (patch) => setState((s) => ({ ...s, ...patch }));

  React.useEffect(() => { document.documentElement.setAttribute('data-theme', state.theme); }, [state.theme]);
  React.useEffect(() => { document.documentElement.style.fontSize = state.textScale === 'S' ? '14px' : state.textScale === 'L' ? '18px' : '16px'; }, [state.textScale]);
  React.useEffect(() => { document.documentElement.classList.toggle('cx-reduce', state.reduce); }, [state.reduce]);

  const sec = CONSOLE_SECTIONS.find((s) => s.id === section);
  const level = ENV_LEVEL[state.env];
  const receipts = [
    { time: '08:02:11', event: 'Approved', details: ['operator: a.mercer', 'policy: ' + state.policy], kind: 'routine' },
    { time: '08:14:02', event: 'Attested', details: ['pdf-extract@2.4.1', 'sha256:9f3c…a217'], kind: 'proof' },
    { time: '09:41:07', event: 'Signed', details: ['ed25519:MEUCIQ…', 'rekor: entry 9f3c'], kind: 'proof' },
    { time: '10:22:40', event: 'Rejected', details: ['unsigned-scraper@0.1', 'no proof · egress'], kind: 'alarm' },
    { time: '11:03:18', event: 'Exported', details: ['bundle · internal', 'digest carried'], kind: 'routine' },
  ];

  const go = (id) => { setSection(id); setNavOpen(false); };

  return (
    <div className={'ck-shell' + (state.dense ? ' ck-dense' : '')} data-navopen={navOpen ? 'true' : 'false'}>
      <a className="ckr-skip" href="#cx-main">Skip to content</a>
      <header className="ck-shell__header">
        <div className="cx-headrow" style={{ minWidth: 0 }}>
          <button className="cx-iconbtn cx-navtrigger" aria-label="Open navigation" onClick={() => setNavOpen(true)}><CkIcon name="matrix" size={17} /></button>
          <div className="ck-masthead" style={{ minWidth: 0 }}>
            <span className="ck-masthead__brand">SKILLPACK</span>
            <span className="ck-masthead__crumb" style={{ font: `500 12px ${CX_MONO}`, color: 'var(--ck-fg-3)' }}>{sec.crumb}</span>
            <span className="ck-masthead__urn cx-masthead-urn" style={{ font: `500 11px ${CX_MONO}`, color: 'var(--ck-fg-mute)' }}>urn:ckodex:console/16.0</span>
          </div>
        </div>
        <div className="cx-headrow">
          <IconBtn icon="agent" label="Operator profile" onClick={() => setDrawer('profile')} />
          <IconBtn icon="policy" label="Console settings" onClick={() => setDrawer('settings')} />
          <IconBtn icon="gauge" label="Accessibility" onClick={() => setDrawer('a11y')} />
          <div role="radiogroup" aria-label="Theme" style={{ display: 'inline-flex', border: '1px solid var(--ck-hairline-strong)' }}>
            {[['ledger', 'L'], ['vault', 'V'], ['hc', 'HC']].map(([v, l], i) => (
              <button key={v} role="radio" aria-checked={state.theme === v} onClick={() => set({ theme: v })}
                style={{ cursor: 'pointer', font: `600 10px ${CX_MONO}`, letterSpacing: '.06em', padding: '7px 10px', border: 'none', borderLeft: i ? '1px solid var(--ck-hairline-strong)' : 'none', background: state.theme === v ? 'var(--ck-fg-1)' : 'transparent', color: state.theme === v ? 'var(--ck-bg-1)' : 'var(--ck-fg-3)' }}>{l}</button>
            ))}
          </div>
        </div>
      </header>

      {navOpen ? <div className="cx-navscrim" onClick={() => setNavOpen(false)} /> : null}
      <nav className="ck-shell__nav" aria-label="Primary">
        <button className="cx-iconbtn cx-nav__close" aria-label="Close navigation" onClick={() => setNavOpen(false)}>✕</button>
        <div style={{ font: `600 9px ${CX_MONO}`, letterSpacing: '.16em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', padding: '4px 20px 12px' }}>Governance</div>
        {CONSOLE_SECTIONS.map((s) => (
          <a key={s.id} className={'ck-nav__item' + (section === s.id ? ' ck-nav__item--active' : '')} href="#" aria-current={section === s.id ? 'page' : undefined} onClick={(e) => { e.preventDefault(); go(s.id); }}>
            <CkIcon name={s.icon} size={18} />
            <span style={{ font: `500 13px ${CX_UI}` }}>{s.label}</span>
          </a>
        ))}
        <div style={{ marginTop: 'auto', padding: 20, display: 'flex', flexDirection: 'column', gap: 8 }}>
          <a href="index.html" style={{ font: `500 11px ${CX_MONO}`, color: 'var(--ck-fg-3)', textDecoration: 'none', borderBottom: 'none' }}>← ecosystem</a>
          <a href="glory.html" style={{ font: `500 11px ${CX_MONO}`, color: 'var(--ck-fg-3)', textDecoration: 'none', borderBottom: 'none' }}>skill dossier →</a>
          <a href="vscode.html" style={{ font: `500 11px ${CX_MONO}`, color: 'var(--ck-fg-3)', textDecoration: 'none', borderBottom: 'none' }}>vs code extension →</a>
        </div>
      </nav>

      <main className="ck-shell__main" id="cx-main">{sec.render()}</main>

      <aside className="ck-shell__margin" aria-label="Evidence margin">
        <CollapsibleMargin title="Evidence margin" entries={receipts} stamp="sha256:9f3c…a217" storageKey="skillpack-console-margin" />
      </aside>

      <footer className="ck-shell__footer">
        <span>⊢ {state.policy.split('@')[0]} · {state.env} · mode: {state.mode}</span>
        <span style={{ display: 'flex', gap: 14, alignItems: 'center' }}>
          <span style={{ color: level === 'sealed' ? 'var(--ck-proof)' : level === 'contained' ? 'var(--ck-alarm)' : 'var(--ck-fg-mute)' }}>{level === 'sealed' ? '◆' : level === 'contained' ? '⊘' : '◇'} {level}</span>
          <span className="ck-invariant" style={{ color: 'var(--ck-fg-3)' }}>mode changes deployment, not governance semantics</span>
        </span>
      </footer>

      {drawer === 'profile' ? <ProfileDrawer onClose={() => setDrawer(null)} /> : null}
      {drawer === 'settings' ? <SettingsDrawer onClose={() => setDrawer(null)} state={state} set={set} /> : null}
      {drawer === 'a11y' ? <AccessibilityDrawer onClose={() => setDrawer(null)} state={state} set={set} /> : null}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Console />);
