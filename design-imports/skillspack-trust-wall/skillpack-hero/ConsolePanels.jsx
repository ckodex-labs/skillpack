// ============================================================
// SkillPack · Management Console — section panels.
// Each panel renders justice to one governance concept, composed
// from DS-3 tokens and the v4 components. Exported to window.
// ============================================================

const CX_MONO = 'var(--ck-ff-mono)';
const CX_UI = 'var(--ck-ff-ui)';
const CX_DISP = 'var(--ck-ff-display)';

function SecHead({ n, title, kick }) {
  return (
    <div className="cx-sechead">
      {n ? <span style={{ font: `500 13px ${CX_MONO}`, color: 'var(--ck-fg-mute)' }}>{n}</span> : null}
      <h2 className="cx-h2">{title}</h2>
      {kick ? <span className="cx-sechead__kick">{kick}</span> : null}
    </div>
  );
}

function Metrics({ items }) {
  return (
    <div className="cx-metrics">
      {items.map((m) => (
        <div className="cx-metric" key={m.k}>
          <span className="cx-metric__k">{m.k}</span>
          <span className="cx-metric__v" data-tone={m.tone}>{m.v}</span>
          <span className="cx-metric__sub">{m.sub}</span>
        </div>
      ))}
    </div>
  );
}

// ---------- Overview + Root fabric hierarchy ----------
function FabricPanel() {
  const nodes = [
    { d: 0, kind: 'system', name: 'evidence-fabric', state: ['◆ sealed', 'proof'], digest: 'sha256:1d44…f0d3' },
    { d: 1, kind: 'kernel', name: 'domain · governance', state: ['◆ sealed', 'proof'], digest: 'sha256:a0b1…9e0f' },
    { d: 2, kind: 'guardrail', name: 'egress-deny@3.2', state: ['◆ proved', 'proof'], digest: 'sha256:a217…7abc' },
    { d: 2, kind: 'policy', name: 'registry-charter@4', state: ['◆ proved', 'proof'], digest: 'sha256:33f1…7d60' },
    { d: 1, kind: 'kernel', name: 'domain · extraction', state: ['◆ sealed', 'proof'], digest: 'sha256:9f3c…a217' },
    { d: 2, kind: 'skill', name: 'pdf-extract@2.4.1', state: ['◆ proved', 'proof'], digest: 'sha256:9f3c…a217' },
    { d: 2, kind: 'skill', name: 'ocr-lift@1.2.0', state: ['● validated', 'tone'], digest: 'sha256:4b8a…a9e0' },
    { d: 1, kind: 'kernel', name: 'domain · reasoning', state: ['● validated', 'tone'], digest: '—' },
    { d: 2, kind: 'model', name: 'claim-classifier@0.9', state: ['● validated', 'tone'], digest: '—' },
    { d: 2, kind: 'agent', name: 'triage-runner@1.4', state: ['◌ scanned', 'tone'], digest: '—' },
  ];
  return (
    <div className="cx-panel">
      <SecHead n="00" title="Root fabric hierarchy" kick="one sealed root · domains · governed leaves — the books balance top to bottom" />
      <Metrics items={[
        { k: 'Fabric root', v: '◆', sub: 'evidence-fabric@16 · sealed', tone: 'proof' },
        { k: 'Domains', v: '3', sub: 'governance · extraction · reasoning' },
        { k: 'Sealed leaves', v: '5 / 8', sub: '◆ E5 proved' },
        { k: 'Unproven in prod', v: '0', sub: '⊘ egress deny-by-default', tone: 'proof' },
      ]} />
      <div style={{ height: 24 }} />
      <div className="cx-tree">
        {nodes.map((nd, i) => (
          <div className="cx-node" data-depth={nd.d} key={i}>
            {nd.d > 0 ? <span className="cx-node__spine" style={{ left: nd.d === 1 ? 20 : 48 }} /> : null}
            <CkIcon name={nd.kind} size={18} style={{ color: nd.state[1] === 'proof' ? 'var(--ck-proof)' : 'var(--ck-fg-1)' }} />
            <span className="cx-node__name">{nd.name}</span>
            <span className="cx-node__kind">{nd.kind}</span>
            <span className="cx-node__state" data-s={nd.state[1]}>{nd.state[0]}</span>
            <span className="cx-node__digest" style={{ minWidth: 118, textAlign: 'right' }}>{nd.digest}</span>
          </div>
        ))}
      </div>
      <p className="cx-lead"><span className="ck-invariant">nothing crosses a boundary without proof</span> — a leaf is admitted only when its parent domain and the root both resolve to a proof object.</p>
    </div>
  );
}

// ---------- Registry ----------
function RegistryPanel() {
  const all = [
    { kind: 'skill', name: 'pdf-extract', urn: 'urn:ckodex:skill:pdf-extract@2.4.1', owner: 'ckodex-core', version: '2.4.1', sealedAt: '2026-07-02', tier: 'E5', digest: 'sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217' },
    { kind: 'aipack', name: 'regulatory-suite', urn: 'urn:ckodex:aipack:regulatory-suite@1.0', owner: 'gov-tools', version: '1.0.0', sealedAt: '2026-06-28', tier: 'E5', digest: 'sha256:4b8ad2f19c7e6a3d0f52b81e9a4c7d6033f1a9e0' },
    { kind: 'guardrail', name: 'egress-deny', urn: 'urn:ckodex:guardrail:egress-deny@3.2', owner: 'ckodex-gov', version: '3.2.0', sealedAt: '2026-07-05', tier: 'E5', digest: 'sha256:a217f0d3841e2b9c7a56e08d4f1b3c2e99017abc' },
    { kind: 'model', name: 'claim-classifier', urn: 'urn:ckodex:model:claim-classifier@0.9', owner: 'research', version: '0.9.3', tier: 'E3' },
    { kind: 'agent', name: 'triage-runner', urn: 'urn:ckodex:agent:triage-runner@1.4', owner: 'ops', version: '1.4.0', tier: 'E1' },
    { kind: 'skill', name: 'ocr-lift', urn: 'urn:ckodex:skill:ocr-lift@1.2', owner: 'ckodex-core', version: '1.2.0', tier: 'E3' },
  ];
  const kinds = ['all', 'skill', 'aipack', 'guardrail', 'model', 'agent'];
  const [q, setQ] = React.useState('');
  const [filter, setFilter] = React.useState('all');
  const shown = all.filter((c) => (filter === 'all' || c.kind === filter) && (c.name + c.urn).toLowerCase().includes(q.toLowerCase()));
  return (
    <div className="cx-panel">
      <SecHead n="01" title="Registry" kick="every artifact carries its own evidence · a digest earns the ◆ seal" />
      <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'center', marginBottom: 18 }}>
        <input className="cx-field" style={{ maxWidth: 320 }} placeholder="search name or urn…" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Search registry" />
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          {kinds.map((k) => <button key={k} className="cx-chip" aria-pressed={filter === k} onClick={() => setFilter(k)}>{k}</button>)}
        </div>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 16 }}>
        {shown.map((c) => <EntityCard key={c.urn} {...c} ciqr={c.digest ? CiqrPath(c.urn) : undefined} />)}
      </div>
      {shown.length === 0 ? <p className="cx-lead">No artifacts match. ○ the query resolves to nothing.</p> : null}
    </div>
  );
}

// ---------- Assessment (compliance matrix) ----------
function AssessmentPanel() {
  const controls = ['provenance', 'egress', 'determinism', 'redaction', 'replay'];
  const rows = [
    { p: 'pdf-extract', cells: ['attested', 'attested', 'attested', 'attested', 'attested'] },
    { p: 'ocr-lift', cells: ['attested', 'attested', 'claimed', 'attested', 'claimed'] },
    { p: 'claim-classifier', cells: ['attested', 'attested', 'observed', 'claimed', 'observed'] },
    { p: 'triage-runner', cells: ['claimed', 'quarantined', 'observed', 'claimed', 'observed'] },
  ];
  const glyph = { attested: '◆', claimed: '○', observed: '⊢', quarantined: '⊘' };
  return (
    <div className="cx-panel">
      <SecHead n="02" title="Assessment" kick="policies × controls · each cell is a claim-state, colored only by the budget" />
      <div className="cx-matrix" style={{ gridTemplateColumns: `minmax(150px, 1.4fr) repeat(${controls.length}, 1fr)` }}>
        <div className="cx-matrix__cell cx-matrix__cell--head">artifact</div>
        {controls.map((c) => <div key={c} className="cx-matrix__cell cx-matrix__cell--head">{c}</div>)}
        {rows.map((r) => (
          <React.Fragment key={r.p}>
            <div className="cx-matrix__cell cx-matrix__cell--row">{r.p}</div>
            {r.cells.map((s, i) => (
              <div className="cx-matrix__cell" key={i}>
                <span className="cx-matrix__glyph" data-s={s}>{glyph[s]}</span>
                <span className="cx-matrix__note">{s}</span>
              </div>
            ))}
          </React.Fragment>
        ))}
      </div>
      <p className="cx-lead">Only <b style={{ color: 'var(--ck-proof)' }}>◆ attested</b> earns violet — a discharged proof object. <b style={{ color: 'var(--ck-alarm)' }}>⊘ quarantined</b> earns red — an active emergency protocol. Everything else is ink and tone: measuring a control is not, by itself, an assertion.</p>
    </div>
  );
}

// ---------- TrustWall (admission gate) ----------
function TrustWallPanel() {
  const reqs = [
    { name: 'pdf-extract@2.4.1', urn: 'urn:ckodex:skill:pdf-extract@2.4.1', v: 'admit', glyph: '◆', reason: 'proof resolves · sig verified · rekor entry 9f3c' },
    { name: 'regulatory-suite@1.0', urn: 'urn:ckodex:aipack:regulatory-suite@1.0', v: 'admit', glyph: '◆', reason: 'proof resolves · all leaves sealed' },
    { name: 'triage-runner@1.4', urn: 'urn:ckodex:agent:triage-runner@1.4', v: 'hold', glyph: '○', reason: 'E1 scanned · no discharging proof — awaits validation' },
    { name: 'unsigned-scraper@0.1', urn: 'urn:ckodex:skill:unsigned-scraper@0.1', v: 'deny', glyph: '⊘', reason: 'no proof object · egress requested — deny-by-default' },
  ];
  const label = { admit: 'admit', hold: 'hold', deny: 'deny' };
  return (
    <div className="cx-panel">
      <SecHead n="03" title="TrustWall" kick="the wall admits on proof · absence of proof is a denial, not a warning" />
      <div className="cx-wall">
        {reqs.map((r) => (
          <div className="cx-wallrow" key={r.urn}>
            <CkIcon name={r.v === 'deny' ? 'quarantine' : r.v === 'admit' ? 'proof' : 'gate'} size={20} style={{ color: r.v === 'admit' ? 'var(--ck-proof)' : r.v === 'deny' ? 'var(--ck-alarm)' : 'var(--ck-fg-2)' }} />
            <div className="cx-wallrow__id">
              <div className="cx-wallrow__name">{r.name}</div>
              <div className="cx-wallrow__urn">{r.urn}</div>
              <div className="cx-wallrow__reason">{r.reason}</div>
            </div>
            <span className="cx-verdict" data-v={r.v}>{r.glyph} {label[r.v]}</span>
          </div>
        ))}
      </div>
      <p className="cx-lead"><span className="ck-invariant">mode changes deployment, not governance semantics</span> — in monitor mode a denial is logged; in enforce mode it is refused. The verdict is identical.</p>
    </div>
  );
}

Object.assign(window, { SecHead, Metrics, FabricPanel, RegistryPanel, AssessmentPanel, TrustWallPanel, CX_MONO, CX_UI, CX_DISP });
