// ============================================================
// SkillPack · VS Code extension — the developer surface.
// A neutral editor frame (title / activity / explorer / editor /
// panel / status) whose extension views carry the DS-3 evidence
// vocabulary: a registry tree with claim states, a webview dossier
// with EntityCard + CIQR, inline proof decorations, and a sealed
// status-bar item. Defaults to Vault (the natural IDE ground).
// ============================================================

const { EntityCard, CkIcon, CkCiqr, CiqrPath, CollapsibleMargin } = window;
const V_MONO = 'var(--ck-ff-mono)';
const V_UI = 'var(--ck-ff-ui)';

function VThemeSwitch() {
  const [t, setT] = React.useState(() => document.documentElement.getAttribute('data-theme') || 'vault');
  React.useEffect(() => { document.documentElement.setAttribute('data-theme', t); }, [t]);
  return (
    <div className="vs-themeswitch" role="radiogroup" aria-label="Theme">
      {[['ledger', 'L'], ['vault', 'V'], ['hc', 'HC']].map(([v, l]) => (
        <button key={v} role="radio" aria-checked={t === v} onClick={() => setT(v)}>{l}</button>
      ))}
    </div>
  );
}

const REGISTRY_TREE = [
  { grp: 'skills', items: [
    { name: 'pdf-extract', s: ['◆', 'proof'], sealed: true },
    { name: 'ocr-lift', s: ['●', 'tone'] },
    { name: 'table-lift', s: ['○', 'tone'] },
  ] },
  { grp: 'guardrails', items: [
    { name: 'egress-deny', s: ['◆', 'proof'], sealed: true },
    { name: 'pii-redact', s: ['◆', 'proof'], sealed: true },
  ] },
  { grp: 'agents', items: [
    { name: 'triage-runner', s: ['◌', 'tone'] },
    { name: 'unsigned-scraper', s: ['⊘', 'alarm'] },
  ] },
];

function Sidebar({ selected, onSelect }) {
  return (
    <div className="vs-side">
      <div className="vs-side__head"><span>SkillPack registry</span><CkIcon name="chain" size={14} style={{ color: 'var(--ck-fg-mute)' }} /></div>
      <div className="vs-tree">
        {REGISTRY_TREE.map((g) => (
          <div key={g.grp}>
            <div style={{ font: `600 9px ${V_MONO}`, letterSpacing: '.14em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', padding: '8px 14px 4px' }}>{g.grp}</div>
            {g.items.map((it) => (
              <button key={it.name} className="vs-trow" aria-selected={selected === it.name} onClick={() => onSelect(it.name)} style={{ paddingLeft: 22 }}>
                <CkIcon name={g.grp === 'skills' ? 'skill' : g.grp === 'guardrails' ? 'guardrail' : 'agent'} size={15} style={{ color: it.s[1] === 'proof' ? 'var(--ck-proof)' : it.s[1] === 'alarm' ? 'var(--ck-alarm)' : 'var(--ck-fg-2)' }} />
                <span className="vs-trow__name">{it.name}</span>
                <span className="vs-trow__state" data-s={it.s[1]}>{it.s[0]}</span>
              </button>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function CodePane() {
  const L = (n, content, deco) => ({ n, content, deco });
  const lines = [
    L(1, <span><span className="c"># urn:ckodex:skill:pdf-extract@2.4.1</span></span>),
    L(2, <span><span className="k">name</span><span className="s">: pdf-extract</span></span>),
    L(3, <span><span className="k">version</span><span className="s">: 2.4.1</span></span>),
    L(4, <span><span className="k">kind</span><span className="s">: skill</span></span>),
    L(5, <span><span className="k">deterministic</span><span className="s">: true</span></span>, ['◆ replayable · digest stable', 'proof']),
    L(6, <span><span className="k">egress</span><span className="s">: deny</span></span>, ['◆ guardrail egress-deny@3.2', 'proof']),
    L(7, <span> </span>),
    L(8, <span><span className="k">evidence</span><span className="s">:</span></span>),
    L(9, <span>{'  '}<span className="k">tier</span><span className="s">: E5</span></span>, ['◆ proved · kernel-checked', 'proof']),
    L(10, <span>{'  '}<span className="k">digest</span><span className="p">: sha256:9f3c…a217</span></span>),
    L(11, <span>{'  '}<span className="k">signature</span><span className="s">: ed25519:MEUCIQ…</span></span>, ['◆ verified', 'proof']),
    L(12, <span>{'  '}<span className="k">anchor</span><span className="s">: rekor.ckodex.com/…/9f3c</span></span>),
    L(13, <span> </span>),
    L(14, <span><span className="k">requires</span><span className="s">:</span></span>),
    L(15, <span>{'  - '}<span className="s">urn:ckodex:model:claim-classifier@0.9</span></span>, ['● validated · not sealed', 'tone']),
  ];
  return (
    <div className="vs-code">
      <div className="vs-gutter">{lines.map((l) => <div key={l.n}>{l.n}</div>)}</div>
      <div className="vs-lines">
        {lines.map((l) => (
          <div key={l.n}>{l.content}{l.deco ? <span className="vs-deco" data-s={l.deco[1]}>{l.deco[0]}</span> : null}</div>
        ))}
      </div>
    </div>
  );
}

function Webview() {
  return (
    <div className="vs-webview">
      <div className="vs-wv__banner"><CkIcon name="proof" size={16} style={{ color: 'var(--ck-proof)' }} /> Skill evidence · resolved from the sealed bundle</div>
      <EntityCard kind="skill" name="pdf-extract" urn="urn:ckodex:skill:pdf-extract@2.4.1" owner="ckodex-core" version="2.4.1" sealedAt="2026-07-02" tier="E5" digest="sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217" ciqr={CiqrPath('urn:ckodex:skill:pdf-extract@2.4.1')} />
      <div style={{ display: 'flex', gap: 18, alignItems: 'center', flexWrap: 'wrap' }}>
        <CkCiqr payload="urn:ckodex:skill:pdf-extract@2.4.1" size={96} attested label="CIQR" caption="scan to resolve the bundle" />
        <div style={{ display: 'flex', flexDirection: 'column', gap: 9, minWidth: 140 }}>
          <button className="ck-btn ck-btn--primary">Verify &amp; install</button>
          <button className="ck-btn ck-btn--quiet">Open dossier</button>
          <button className="ck-btn ck-btn--ghost">Copy for AI</button>
        </div>
      </div>
      <div className="ck-label" style={{ marginTop: 4 }}>Receipts</div>
      <CollapsibleMargin
        title="Evidence margin"
        storageKey="vscode-margin"
        entries={[
          { time: '09:40:51', event: 'Attested', details: ['kernel: ck-kernel@16'], kind: 'proof' },
          { time: '09:41:07', event: 'Signed', details: ['rekor: entry 9f3c'], kind: 'proof' },
          { time: '11:20:03', event: 'Evaluated', details: ['conformance 418/418'], kind: 'routine' },
        ]}
        stamp="sha256:9f3c…a217"
      />
    </div>
  );
}

function Panel() {
  return (
    <div className="vs-panel">
      <div className="vs-panel__head">
        <button className="vs-panel__tab" aria-selected="true">SkillPack</button>
        <button className="vs-panel__tab" aria-selected="false">Problems</button>
        <button className="vs-panel__tab" aria-selected="false">Output</button>
      </div>
      <div className="vs-log">
        <div>[09:41] resolve urn:ckodex:skill:pdf-extract@2.4.1</div>
        <div>[09:41] recompute sha256(body) → 9f3c…a217 · <b className="p">matches content_digest</b></div>
        <div>[09:41] verify ed25519 signature over digest · <b className="p">valid</b></div>
        <div>[09:41] resolve transparency anchor rekor/9f3c · <b className="p">present</b></div>
        <div>[09:41] tier E5 · <b className="p">◆ proved</b> — admit</div>
        <div>[10:22] resolve urn:ckodex:skill:unsigned-scraper@0.1</div>
        <div>[10:22] no proof object · egress requested · <b className="a">⊘ deny-by-default</b></div>
      </div>
    </div>
  );
}

function StatusBar() {
  return (
    <div className="vs-status">
      <span className="vs-status__seal">◆ sealed</span>
      <span className="vs-status__item">E5 · proved</span>
      <span className="vs-status__item">mode: enforce</span>
      <span className="vs-status__item">rekor ✓</span>
      <span className="vs-status__spacer" />
      <span className="vs-status__item">sha256:9f3c…a217</span>
      <span className="vs-status__item">Ln 6, Col 12</span>
      <span className="vs-status__item">YAML</span>
    </div>
  );
}

function VSCode() {
  const [selected, setSelected] = React.useState('pdf-extract');
  const [act, setAct] = React.useState('skillpack');
  const [sideOpen, setSideOpen] = React.useState(false);
  const acts = [['digest', 'explorer'], ['skill', 'skillpack'], ['gauge', 'search'], ['chain', 'source control'], ['export', 'run']];
  return (
    <div className="vs" data-sideopen={sideOpen ? 'true' : 'false'}>
      <div className="vs-title">
        <div className="vs-dots"><i /><i /><i /></div>
        <button className="vs-act vs-sidetoggle" aria-label="Toggle sidebar" onClick={() => setSideOpen((v) => !v)} style={{ width: 26, height: 22 }}><CkIcon name="matrix" size={14} /></button>
        <span className="vs-title__name">pdf-extract.skill.yaml — skillpack-registry — VS Code</span>
        <VThemeSwitch />
      </div>
      <div className="vs-body">
        <div className="vs-activity">
          {acts.map(([icon, id], i) => (
            <button key={id} className="vs-act" aria-pressed={act === id} aria-label={id} data-badge={i === 1 ? '7' : undefined} onClick={() => { setAct(id); setSideOpen(true); }} style={{ position: 'relative' }}>
              <CkIcon name={icon} size={20} />
            </button>
          ))}
          <div style={{ marginTop: 'auto', display: 'flex', flexDirection: 'column', gap: 6 }}>
            <a href="console.html" className="vs-act" aria-label="console" title="Management console"><CkIcon name="gate" size={20} /></a>
            <a href="index.html" className="vs-act" aria-label="ecosystem" title="Ecosystem"><CkIcon name="system" size={20} /></a>
          </div>
        </div>
        <Sidebar selected={selected} onSelect={(n) => { setSelected(n); setSideOpen(false); }} />
        <div className="vs-editor">
          <div className="vs-tabs">
            <button className="vs-tab" aria-selected="true"><span className="vs-tab__dot" />{selected}.skill.yaml</button>
            <button className="vs-tab" aria-selected="false">SKILL.md</button>
          </div>
          <div className="vs-pane vs-splitcols" style={{ gridTemplateColumns: '1fr 380px' }}>
            <div style={{ minWidth: 0, display: 'flex', flexDirection: 'column' }}>
              <div style={{ flex: 1, overflow: 'auto' }}><CodePane /></div>
              <Panel />
            </div>
            <div style={{ borderLeft: '1px solid var(--ck-hairline)', minWidth: 0, overflow: 'hidden' }}><Webview /></div>
          </div>
        </div>
      </div>
      <StatusBar />
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<VSCode />);
