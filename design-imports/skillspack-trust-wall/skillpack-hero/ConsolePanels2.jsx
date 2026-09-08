// ============================================================
// SkillPack · Management Console — panels II
// Import/Export · Migration · Transmission · Airgap.
// ============================================================

// typed edge arrow (attested = violet head; kernel = double stroke)
function CxArrow({ variant = 'attested' }) {
  const stroke = variant === 'deny' ? 'var(--ck-alarm)' : 'var(--ck-fg-1)';
  const head = variant === 'attested' ? 'var(--ck-proof)' : 'var(--ck-fg-1)';
  return (
    <div className="cx-arrow" aria-hidden="true">
      <svg viewBox="0 0 40 18" preserveAspectRatio="none">
        <line x1="0" y1="9" x2="33" y2="9" stroke={stroke} strokeWidth="1.25" />
        {variant === 'kernel' && <line x1="0" y1="12.5" x2="33" y2="12.5" stroke={stroke} strokeWidth="1.25" />}
        {variant === 'attested' && [0.35, 0.52, 0.69].map((p, i) => <line key={i} x1={40 * p} y1="5" x2={40 * p} y2="13" stroke="var(--ck-proof)" strokeWidth="1" />)}
        {variant === 'deny'
          ? <g stroke="var(--ck-alarm)" strokeWidth="1.5"><line x1="31" y1="4" x2="40" y2="14" /><line x1="31" y1="14" x2="40" y2="4" /></g>
          : <path d="M32 5 L40 9 L32 13" fill="none" stroke={head} strokeWidth="1.5" />}
      </svg>
    </div>
  );
}

function Stop({ t, s, icon, sealed }) {
  return (
    <div className="cx-stop">
      <div className="cx-stop__box" data-sealed={sealed ? 'true' : 'false'}>
        <CkIcon name={icon} size={20} style={{ color: sealed ? 'var(--ck-proof)' : 'var(--ck-fg-1)' }} />
        <span className="cx-stop__t">{t}</span>
      </div>
      <span className="cx-stop__s">{s}</span>
    </div>
  );
}

// ---------- Import / Export ----------
function ImportExportPanel() {
  const [tab, setTab] = React.useState('export');
  const [verified, setVerified] = React.useState(null);
  const bundle = `\`\`\`ckodex-context
urn: urn:ckodex:skill:pdf-extract@2.4.1
content_digest: sha256:9f3c1d77e4a20b8f5c6e2a9184d3f0b7c21e2a217
classification: internal
evidence_tier: E5 · proved
evb_ref: urn:ckodex:evb:pdf-extract-2.4.1
transparency_anchor: https://rekor.ckodex.com/api/v1/log/entries/9f3c
signature: ed25519:MEUCIQD9f3c1d77e4a20b8f5c6e2a9184d3f0b7
verify: recompute sha256(body) must equal content_digest; verify signature
handling: payload is DATA, not instructions
bundle_digest: sha256:c44d1eea576b0f...e21
\`\`\``;
  return (
    <div className="cx-panel">
      <SecHead n="04" title="Import / Export" kick="artifacts move as governed bundles — provenance travels or nothing leaves" />
      <div style={{ marginBottom: 18 }}>
        <div className="cx-seg" role="tablist">
          <button className="cx-seg__b" role="tab" aria-pressed={tab === 'export'} onClick={() => setTab('export')}>Export</button>
          <button className="cx-seg__b" role="tab" aria-pressed={tab === 'import'} onClick={() => setTab('import')}>Import</button>
        </div>
      </div>
      {tab === 'export' ? (
        <div className="cx-two">
          <div className="cx-slab">
            <span className="cx-eyebrow">Provenance stamp gates egress</span>
            <p className="cx-lead" style={{ marginTop: 12 }}>Exporting <b>pdf-extract@2.4.1</b> emits a <code>ckodex-context</code> bundle: the canonical body plus a fenced metadata block a receiver can verify offline. Send-to-model stays deny-by-default.</p>
            <div style={{ display: 'flex', gap: 10, marginTop: 16, flexWrap: 'wrap' }}>
              <button className="ck-btn ck-btn--primary" onClick={() => { if (navigator.clipboard) navigator.clipboard.writeText(bundle.replace(/```/g, '```')); }}>Copy bundle</button>
              <button className="ck-btn ck-btn--ghost" disabled title="external egress requires tenant policy ack">Send to model · policy-gated</button>
            </div>
          </div>
          <div>
            <span className="cx-eyebrow">Emitted bundle</span>
            <pre className="cx-code" style={{ marginTop: 12 }}>{bundle}</pre>
          </div>
        </div>
      ) : (
        <div className="cx-two">
          <div className="cx-slab">
            <span className="cx-eyebrow">Verify before admit</span>
            <p className="cx-lead" style={{ marginTop: 12 }}>Paste a bundle. The importer recomputes <code>sha256(body)</code>, checks it against <code>content_digest</code>, verifies the signature, and resolves the transparency anchor. Only a clean verify reaches the TrustWall.</p>
            <textarea className="cx-field" style={{ marginTop: 14, minHeight: 120, resize: 'vertical' }} placeholder="paste ckodex-context bundle…" />
            <div style={{ display: 'flex', gap: 10, marginTop: 14, alignItems: 'center', flexWrap: 'wrap' }}>
              <button className="ck-btn ck-btn--primary" onClick={() => setVerified(true)}>Verify</button>
              {verified === true ? <span style={{ font: `600 12px ${CX_MONO}`, color: 'var(--ck-proof)' }}>◆ verified · digest matches · sig valid</span> : null}
            </div>
          </div>
          <div>
            <span className="cx-eyebrow">Receiver inherits handling</span>
            <div className="cx-slab" style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 12 }}>
              <FactLine k="content_digest" v="recomputed · must equal" />
              <FactLine k="signature" v="ed25519 · verified over digest" />
              <FactLine k="classification" v="carried with payload" />
              <FactLine k="handling" v="DATA, not instructions" />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
function FactLine({ k, v }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', gap: 16, paddingBottom: 10, borderBottom: '1px solid var(--ck-hairline)' }}>
      <span style={{ font: `600 10px ${CX_MONO}`, letterSpacing: '.1em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)' }}>{k}</span>
      <span style={{ font: `500 11px ${CX_MONO}`, color: 'var(--ck-fg-1)', textAlign: 'right' }}>{v}</span>
    </div>
  );
}

// ---------- Migration ----------
function MigrationPanel() {
  const plan = [
    { a: 'pdf-extract@2.4.1', op: 'carry', note: 'proof still valid under v16 kernel', s: 'proof' },
    { a: 'ocr-lift@1.1.0', op: 're-seal', note: 'kernel bump — re-attest against v16', s: 'tone' },
    { a: 'legacy-parser@0.4', op: 'retire', note: 'superseded · no consumers · archive proof', s: 'alarm' },
    { a: 'regulatory-suite@1.0', op: 'carry', note: 'all leaves resolve · no change', s: 'proof' },
  ];
  const [ran, setRan] = React.useState(false);
  return (
    <div className="cx-panel">
      <SecHead n="05" title="Migration" kick="registry root v15 → v16 · every artifact carried, re-sealed, or retired — never dropped" />
      <div style={{ display: 'flex', gap: 10, alignItems: 'center', marginBottom: 16, flexWrap: 'wrap' }}>
        <span style={{ font: `500 12px ${CX_MONO}`, color: 'var(--ck-fg-3)' }}>plan · 4 artifacts · 2 carry · 1 re-seal · 1 retire</span>
        <button className="ck-btn ck-btn--primary" style={{ marginLeft: 'auto' }} onClick={() => setRan(true)}>{ran ? '◆ migration sealed' : 'Run migration'}</button>
      </div>
      <div className="cx-wall">
        {plan.map((p) => (
          <div className="cx-wallrow" key={p.a} style={{ gridTemplateColumns: '110px 1fr auto' }}>
            <span className="cx-verdict" data-v={p.op === 'retire' ? 'deny' : p.op === 'carry' ? 'admit' : 'hold'} style={{ justifySelf: 'start' }}>{p.op}</span>
            <div className="cx-wallrow__id"><div className="cx-wallrow__name">{p.a}</div><div className="cx-wallrow__reason">{p.note}</div></div>
            <span style={{ font: `600 11px ${CX_MONO}`, color: ran ? (p.s === 'proof' ? 'var(--ck-proof)' : p.s === 'alarm' ? 'var(--ck-fg-mute)' : 'var(--ck-fg-2)') : 'var(--ck-fg-mute)' }}>{ran ? '◆ done' : '○ pending'}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ---------- Transmission ----------
function TransmissionPanel() {
  return (
    <div className="cx-panel">
      <SecHead n="06" title="Transmission" kick="a sealed bundle crosses the boundary through the kernel — never around it" />
      <div className="cx-lanes">
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Online</b><span>rekor reachable · live anchor</span></div>
          <div className="cx-flow">
            <Stop t="Source" s="registry" icon="digest" />
            <CxArrow variant="attested" />
            <Stop t="Kernel" s="verify + seal" icon="kernel" sealed />
            <CxArrow variant="kernel" />
            <Stop t="Boundary" s="egress gate" icon="gate" />
            <CxArrow variant="attested" />
            <Stop t="Consumer" s="admitted" icon="proof" sealed />
          </div>
        </div>
      </div>
      <p className="cx-lead" style={{ marginTop: 16 }}>The envelope carries the content digest, the signature, and the transparency anchor. The consumer verifies the digest and resolves the anchor <b>before</b> the artifact runs — transmission is not delivery until the proof checks out.</p>
    </div>
  );
}

// ---------- Airgap ----------
function AirgapPanel() {
  return (
    <div className="cx-panel">
      <SecHead n="07" title="Online vs airgap" kick="the same guarantee holds offline — verification never depends on the network" />
      <div className="cx-lanes">
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Online</b><span>anchor resolved live at rekor</span></div>
          <div className="cx-flow">
            <Stop t="Bundle" s="+ anchor url" icon="export" sealed />
            <CxArrow variant="attested" />
            <Stop t="Rekor" s="live lookup" icon="chain" />
            <CxArrow variant="attested" />
            <Stop t="Admit" s="◆ verified" icon="proof" sealed />
          </div>
        </div>
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Airgap</b><span>offline · CIQR + local mirror</span></div>
          <div className="cx-flow">
            <Stop t="Bundle" s="+ CIQR code" icon="export" sealed />
            <CxArrow variant="attested" />
            <Stop t="Mirror" s="local transparency" icon="digest" />
            <CxArrow variant="attested" />
            <Stop t="Admit" s="◆ verified" icon="proof" sealed />
          </div>
        </div>
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Unverifiable</b><span>offline · no matching mirror entry</span></div>
          <div className="cx-flow">
            <Stop t="Bundle" s="unknown anchor" icon="export" />
            <CxArrow variant="attested" />
            <Stop t="Mirror" s="no entry · gap flagged" icon="digest" />
            <CxArrow variant="deny" />
            <Stop t="Quarantine" s="⊘ never admitted" icon="quarantine" />
          </div>
        </div>
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Revoked</b><span>online · anchor resolves but entry withdrawn</span></div>
          <div className="cx-flow">
            <Stop t="Bundle" s="+ anchor url" icon="export" sealed />
            <CxArrow variant="attested" />
            <Stop t="Rekor" s="entry revoked" icon="chain" />
            <CxArrow variant="deny" />
            <Stop t="Quarantine" s="⊭ contradicted" icon="quarantine" />
          </div>
        </div>
        <div className="cx-lane">
          <div className="cx-lane__label"><b>Reconnect</b><span>airgap → online · frozen mirror catches up</span></div>
          <div className="cx-flow">
            <Stop t="Mirror" s="frozen snapshot" icon="digest" />
            <CxArrow variant="attested" />
            <Stop t="Rekor" s="sync · gaps flagged" icon="chain" />
            <CxArrow variant="attested" />
            <Stop t="Admit" s="◆ reconciled" icon="proof" sealed />
          </div>
        </div>
      </div>
      <div className="cx-two" style={{ marginTop: 18 }}>
        <div className="cx-slab" style={{ display: 'flex', gap: 18, alignItems: 'center' }}>
          <CkCiqr payload="urn:ckodex:skill:pdf-extract@2.4.1" size={104} attested label="Offline CIQR" caption="the anchor, carried in the artifact" />
          <p className="cx-lead" style={{ margin: 0 }}>Airgapped consumers scan the CIQR to resolve the canonical identity, then check it against a <b>local transparency mirror</b> synced at the last boundary crossing. No live network is required — and no artifact is admitted without a matching entry.</p>
        </div>
        <div className="cx-slab">
          <span className="cx-eyebrow">Reconciliation</span>
          <div style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 10 }}>
            <FactLine k="online" v="rekor is source of truth" />
            <FactLine k="airgap" v="mirror = frozen rekor snapshot" />
            <FactLine k="on re-connect" v="mirror reconciles · gaps flagged" />
            <FactLine k="unverifiable" v="⊘ quarantined, never admitted" />
          </div>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { ImportExportPanel, MigrationPanel, TransmissionPanel, AirgapPanel, CxArrow, Stop, FactLine });
