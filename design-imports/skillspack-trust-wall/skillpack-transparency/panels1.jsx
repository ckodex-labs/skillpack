// ============================================================
// Transparency Fabric · panels 1–3
// Fabric Kernel · Transparency Graph · Transparency Exchange (STX)
// ============================================================

const TwoCol = ({ aside, children }) => (
  <div style={{ display: 'grid', gridTemplateColumns: '256px minmax(0,1fr)', gap: 32, alignItems: 'start' }}>
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>{aside}</div>
    <div style={{ minWidth: 0 }}>{children}</div>
  </div>
);
const Def = ({ children }) => (
  <p style={{ margin: 0, font: `400 14px/1.6 ${TF_UI}`, color: 'var(--ck-fg-2)' }}>{children}</p>
);

// ============================================================
// 01 · Transparency Fabric Kernel
// ============================================================
const FabricKernel = () => (
  <section>
    <SectionHead n="01" name="Transparency Fabric Kernel" kicker="the sealed core every exchange passes through" />
    <TwoCol aside={
      <React.Fragment>
        <Def>A sealed computation crate that mediates every transparency operation. Nothing reaches a consumer except through the kernel — and the kernel refuses any call that would break an invariant.</Def>
        <div style={{ borderTop: '1px solid var(--ck-hairline)', paddingTop: 12 }}>
          <TFLabel style={{ marginBottom: 8 }}>Enforced invariants</TFLabel>
          <Invariant>every artifact carries a provenance envelope</Invariant>
          <Invariant>no boundary crossing without an attestation</Invariant>
          <Invariant>the ledger is append-only</Invariant>
        </div>
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          <Chip kind="observed">sealed crate</Chip><Chip kind="attested">GAL-5</Chip><Chip kind="tone">in-proc + gRPC</Chip>
        </div>
      </React.Fragment>
    }>
      <DotField>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr auto 1fr', alignItems: 'center', gap: 0 }}>
          {/* producers */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <TFLabel style={{ marginBottom: 2 }}>Producers</TFLabel>
            <Node title="Authors" sub="skillpack init · craft" glyph="✎" />
            <Node title="CI / CNI" sub="assess · package" glyph="⊟" />
          </div>
          {/* kernel with in/out edges */}
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <Edge variant="attested" w={56} label="attest" />
            <Octagon size={132} label="FABRIC" sub="KERNEL · v16" tone="var(--ck-fg-1)" />
            <Edge variant="attested" w={56} label="serve" />
          </div>
          {/* consumers */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12, alignItems: 'flex-end' }}>
            <TFLabel style={{ marginBottom: 2 }}>Consumers</TFLabel>
            <Node title="Agents" sub="MCP · install" glyph="◆" gtone="var(--ck-proof)" style={{ width: 168 }} />
            <Node title="Registries" sub="OCI · STX" glyph="⊞" style={{ width: 168 }} />
          </div>
        </div>
        <div style={{ marginTop: 22, paddingTop: 16, borderTop: '1px dashed var(--ck-hairline-strong)', display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
          <span style={{ font: `600 10px ${TF_MONO}`, color: 'var(--ck-alarm)' }}>⊘</span>
          <span style={{ font: `500 11.5px ${TF_UI}`, color: 'var(--ck-fg-3)' }}>a call that omits provenance is refused at the boundary — the kernel returns <span style={{ font: `600 11px ${TF_MONO}`, color: 'var(--ck-alarm)' }}>SKILLPACK_SEC_NO_PROVENANCE</span>, never a partial artifact.</span>
        </div>
      </DotField>
    </TwoCol>
  </section>
);

// ============================================================
// 02 · Transparency Graph
// ============================================================
const GRAPH = [
  { t: 'Skill', s: 'my-formatter', g: '◆', gt: 'var(--ck-fg-2)' },
  { t: 'Digest', s: 'sha256:9f3c…', g: '⊛', gt: 'var(--ck-fg-2)' },
  { t: 'PCA', s: 'producer attest', g: '⊢', gt: 'var(--ck-proof)', sealed: true, tone: 'var(--ck-proof)' },
  { t: 'UCA', s: 'usage attest', g: '⊢', gt: 'var(--ck-proof)', sealed: true, tone: 'var(--ck-proof)' },
  { t: 'Witness', s: '3 co-signers', g: '⊛', gt: 'var(--ck-proof)' },
  { t: 'Rekor', s: 'log #4471902', g: '◆', gt: 'var(--ck-proof)', sealed: true, tone: 'var(--ck-proof)' },
];
const TransparencyGraph = () => (
  <section>
    <SectionHead n="02" name="Transparency Graph" kicker="every claim traced to a verifiable root" />
    <TwoCol aside={
      <React.Fragment>
        <Def>The append-only provenance DAG. Each skill resolves to a digest, a chain of attestations, the witnesses that co-signed them, and a transparency-log entry. Any past state is reconstructable; no node is ever rewritten.</Def>
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          <Chip kind="attested">violet = proof object</Chip><Chip kind="tone">append-only</Chip>
        </div>
      </React.Fragment>
    }>
      <DotField pad={24}>
        <div style={{ display: 'flex', alignItems: 'center', overflowX: 'auto', paddingBottom: 6 }}>
          {GRAPH.map((node, i) => (
            <React.Fragment key={node.t}>
              <div style={{ flexShrink: 0, width: 124 }}>
                <Node title={node.t} sub={node.s} glyph={node.g} gtone={node.gt} sealed={node.sealed} tone={node.tone} />
              </div>
              {i < GRAPH.length - 1 && <Edge variant="attested" w={46} />}
            </React.Fragment>
          ))}
        </div>
        <div style={{ marginTop: 18, display: 'flex', gap: 22, font: `500 10px ${TF_MONO}`, color: 'var(--ck-fg-mute)', flexWrap: 'wrap' }}>
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 7 }}><svg width="26" height="10"><line x1="0" y1="5" x2="18" y2="5" stroke="var(--ck-fg-1)" strokeWidth="1.25"/><line x1="9" y1="2" x2="9" y2="8" stroke="var(--ck-proof)" strokeWidth="1"/><path d="M18 2 L24 5 L18 8" fill="none" stroke="var(--ck-proof)" strokeWidth="1.5"/></svg> attested edge · violet head + hash ticks</span>
          <span>◆ sealed node = attestation present</span>
        </div>
      </DotField>
    </TwoCol>
  </section>
);

// ============================================================
// 03 · Transparency Exchange (STX)
// ============================================================
const MODES = [
  { m: 'public', d: 'full envelope', c: 'var(--ck-proof)' },
  { m: 'redacted', d: 'digests only', c: 'var(--ck-fg-1)' },
  { m: 'sealed', d: 'encrypted body', c: 'var(--ck-accent)' },
  { m: 'private', d: 'metadata withheld', c: 'var(--ck-fg-3)' },
];
const TRANSPORTS = ['REST / JSON', 'gRPC / Connect', 'GraphQL'];
const TransparencyExchange = () => (
  <section>
    <SectionHead n="03" name="Transparency Exchange" kicker="STX v0.1.0 · publish & subscribe transparency artifacts" />
    <TwoCol aside={
      <React.Fragment>
        <Def>The protocol that moves transparency artifacts between fabrics. A producer publishes once; subscribers receive the envelope filtered to the privacy mode their trust tier permits — across any supported transport.</Def>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 7 }}>
          {/* flow */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <Node title="Producer" style={{ flex: 1 }} />
          </div>
          <div style={{ display: 'flex', justifyContent: 'center' }}><Edge variant="attested" vertical /></div>
          <Node title="STX · privacy filter" glyph="⊟" sealed tone="var(--ck-accent)" />
          <div style={{ display: 'flex', justifyContent: 'center' }}><Edge variant="promote" vertical /></div>
          <Node title="Subscriber" sub="receives permitted view" style={{ flex: 1 }} />
        </div>
      </React.Fragment>
    }>
      <DotField pad={0} style={{ overflow: 'hidden' }}>
        {/* header row: privacy modes */}
        <div style={{ display: 'grid', gridTemplateColumns: '140px repeat(4, 1fr)', borderBottom: '1px solid var(--ck-hairline-strong)' }}>
          <div style={{ padding: '12px 14px' }}><TFLabel>Transport ↓ · Mode →</TFLabel></div>
          {MODES.map(mo => (
            <div key={mo.m} style={{ padding: '12px 12px', borderLeft: '1px solid var(--ck-hairline)' }}>
              <div style={{ font: `700 11px ${TF_MONO}`, color: mo.c, letterSpacing: '.04em' }}>{mo.m}</div>
              <div style={{ font: `500 9.5px ${TF_MONO}`, color: 'var(--ck-fg-mute)', marginTop: 3 }}>{mo.d}</div>
            </div>
          ))}
        </div>
        {/* transport rows */}
        {TRANSPORTS.map((tr, ri) => (
          <div key={tr} style={{ display: 'grid', gridTemplateColumns: '140px repeat(4, 1fr)', borderBottom: ri < TRANSPORTS.length - 1 ? '1px solid var(--ck-hairline)' : 'none' }}>
            <div style={{ padding: '14px', font: `600 12px ${TF_MONO}`, color: 'var(--ck-fg-1)' }}>{tr}</div>
            {MODES.map((mo, ci) => {
              // private over GraphQL is the only withheld combo (illustrative)
              const ok = !(mo.m === 'private' && tr === 'GraphQL');
              return (
                <div key={mo.m} style={{ padding: '14px', borderLeft: '1px solid var(--ck-hairline)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <span style={{ font: `600 13px ${TF_MONO}`, color: ok ? mo.c : 'var(--ck-alarm)' }}>{ok ? '⊢' : '⊘'}</span>
                </div>
              );
            })}
          </div>
        ))}
      </DotField>
    </TwoCol>
  </section>
);

Object.assign(window, { TwoCol, Def, FabricKernel, TransparencyGraph, TransparencyExchange });
