// ============================================================
// Transparency Fabric · panels 4–6
// TrustWall · Transition Station · Online vs Airgap
// (uses TwoCol, Def, kit primitives)
// ============================================================

// ============================================================
// 04 · TrustWall
// ============================================================
const WALL = [
  { name: 'writing-skills', reg: 'reg.ckodex.org', sig: 1, prov: 1, trust: 1 },
  { name: 'proof-audit', reg: 'reg.ckodex.org', sig: 1, prov: 1, trust: 1 },
  { name: 'fast-lint', reg: 'skills.sh', sig: 1, prov: 0, trust: 1 },
  { name: 'atlas-helper', reg: 'atlas.partner', sig: 0, prov: 0, trust: 0 },
];
const Mark = ({ ok }) => (
  <span style={{ font: `600 13px ${TF_MONO}`, color: ok ? 'var(--ck-proof)' : 'var(--ck-alarm)' }}>{ok ? '⊢' : '⊘'}</span>
);
const TrustWall = () => (
  <section>
    <SectionHead n="04" name="TrustWall" kicker="the admission boundary · signed ∧ provenanced ∧ trusted" />
    <TwoCol aside={
      <React.Fragment>
        <Def>The policy wall between an untrusted registry and the local store. A candidate is admitted only if all three predicates hold; any miss drops it to review or denies it outright.</Def>
        <div style={{ borderTop: '1px solid var(--ck-hairline)', paddingTop: 12 }}>
          <TFLabel style={{ marginBottom: 8 }}>Admission predicate</TFLabel>
          <Invariant>cosign signature verifies</Invariant>
          <Invariant>provenance envelope present</Invariant>
          <Invariant>registry on the trust list</Invariant>
        </div>
      </React.Fragment>
    }>
      <DotField pad={0} style={{ overflow: 'hidden' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr 0.7fr 0.7fr 0.7fr 1fr', borderBottom: '1px solid var(--ck-hairline-strong)', background: 'var(--ck-bg-1)' }}>
          {['Candidate', 'Registry', 'Sig', 'Prov', 'Trust', 'Verdict'].map((h, i) => (
            <div key={h} style={{ padding: '11px 14px', borderLeft: i ? '1px solid var(--ck-hairline)' : 'none' }}><TFLabel>{h}</TFLabel></div>
          ))}
        </div>
        {WALL.map((r, ri) => {
          const admit = r.sig && r.prov && r.trust;
          const review = !admit && r.sig && r.trust;
          const verdict = admit ? { t: 'admit', c: 'var(--ck-proof)', g: '⊢' } : review ? { t: 'review', c: 'var(--ck-accent)', g: '○' } : { t: 'deny', c: 'var(--ck-alarm)', g: '⊘' };
          return (
            <div key={r.name} style={{ display: 'grid', gridTemplateColumns: '1.4fr 1fr 0.7fr 0.7fr 0.7fr 1fr', borderBottom: ri < WALL.length - 1 ? '1px solid var(--ck-hairline)' : 'none', alignItems: 'center' }}>
              <div style={{ padding: '13px 14px', font: `600 12px ${TF_MONO}`, color: 'var(--ck-fg-1)' }}>{r.name}</div>
              <div style={{ padding: '13px 14px', borderLeft: '1px solid var(--ck-hairline)', font: `500 11px ${TF_MONO}`, color: 'var(--ck-fg-3)' }}>{r.reg}</div>
              <div style={{ padding: '13px 14px', borderLeft: '1px solid var(--ck-hairline)', textAlign: 'center' }}><Mark ok={r.sig} /></div>
              <div style={{ padding: '13px 14px', borderLeft: '1px solid var(--ck-hairline)', textAlign: 'center' }}><Mark ok={r.prov} /></div>
              <div style={{ padding: '13px 14px', borderLeft: '1px solid var(--ck-hairline)', textAlign: 'center' }}><Mark ok={r.trust} /></div>
              <div style={{ padding: '13px 14px', borderLeft: `2px solid ${verdict.c}`, display: 'flex', alignItems: 'center', gap: 7 }}>
                <span style={{ font: `600 12px ${TF_MONO}`, color: verdict.c }}>{verdict.g}</span>
                <span style={{ font: `600 11px ${TF_MONO}`, color: verdict.c, textTransform: 'uppercase', letterSpacing: '.06em' }}>{verdict.t}</span>
              </div>
            </div>
          );
        })}
      </DotField>
    </TwoCol>
  </section>
);

// ============================================================
// 05 · Transition Station
// ============================================================
const Platform = ({ env, tier, glyph }) => (
  <div style={{ flex: 1, minWidth: 0, textAlign: 'center' }}>
    <Octagon size={92} label={env} sub={tier} tone="var(--ck-fg-1)" />
  </div>
);
const Gate = ({ cond, locked }) => (
  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 7, padding: '0 4px' }}>
    <Edge variant={locked ? 'deny' : 'promote'} w={64} />
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 5, font: `600 9px ${TF_MONO}`, letterSpacing: '.05em', textTransform: 'uppercase', color: locked ? 'var(--ck-alarm)' : 'var(--ck-proof)', boxShadow: `inset 0 0 0 1px ${locked ? 'var(--ck-alarm)' : 'var(--ck-proof)'}`, padding: '3px 7px', whiteSpace: 'nowrap' }}>{locked ? '⊘' : '⊢'} {cond}</span>
  </div>
);
const TransitionStation = () => (
  <section>
    <SectionHead n="05" name="Transition Station" kicker="lifecycle promotion · each crossing is a gate" />
    <TwoCol aside={
      <React.Fragment>
        <Def>Where a skill moves between environments. A transition is never automatic — it is a gate that consumes evidence: an assessment grade, an attestation, the right witnesses. Fail the gate and the artifact stays on its platform.</Def>
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          <Chip kind="attested">promote</Chip><Chip kind="deny">blocked · waiver</Chip><Chip kind="tone">supersede</Chip>
        </div>
      </React.Fragment>
    }>
      <DotField>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 4 }}>
          <Platform env="DEV" tier="L1 · draft" />
          <Gate cond="assess ≥ C" />
          <Platform env="STAGING" tier="L3 · review" />
          <Gate cond="3 witnesses" locked />
          <Platform env="PROD" tier="L4 · sealed" />
        </div>
        <div style={{ marginTop: 20, paddingTop: 14, borderTop: '1px dashed var(--ck-hairline-strong)', display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ font: `600 11px ${TF_MONO}`, color: 'var(--ck-alarm)' }}>⊘ staging → prod is held</span>
          <span style={{ font: `500 11.5px ${TF_UI}`, color: 'var(--ck-fg-3)' }}>promotion to L4 needs 3 co-signing witnesses; 2 present. The station records the attempt and waits — it never silently downgrades the gate.</span>
        </div>
      </DotField>
    </TwoCol>
  </section>
);

// ============================================================
// 06 · Online vs Airgap
// ============================================================
const OA = [
  { k: 'Connectivity', on: 'gRPC :50051 · live', air: 'none · sealed bundle' },
  { k: 'Verify against', on: 'Rekor public log', air: 'embedded attestations' },
  { k: 'Witnesses', on: 'fetched & co-signed live', air: 'co-signs bundled at seal' },
  { k: 'Freshness', on: 'real-time revocation', air: 'as-of seal timestamp' },
  { k: 'Trust root', on: 'TUF · Fulcio CA', air: 'pinned offline keyring' },
  { k: 'On failure', on: 'retry · queue · reconnect', air: 'fail-closed' },
];
const OnlineAirgap = () => (
  <section>
    <SectionHead n="06" name="Online vs Airgap" kicker="two trust postures · same evidence guarantees" />
    <TwoCol aside={
      <React.Fragment>
        <Def>The fabric runs in two postures. Online, trust is checked against a live transparency log. Airgapped, the bundle carries its own sealed evidence envelope and is verified against a pinned offline root. Either way, an artifact arrives proven — or it does not arrive.</Def>
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          <Chip kind="attested">⊢ online</Chip><Chip kind="rust">⊟ airgap</Chip>
        </div>
      </React.Fragment>
    }>
      <DotField pad={0} style={{ overflow: 'hidden' }}>
        {/* header */}
        <div style={{ display: 'grid', gridTemplateColumns: '150px 1fr 1fr', borderBottom: '1px solid var(--ck-hairline-strong)', background: 'var(--ck-bg-1)' }}>
          <div style={{ padding: '12px 16px' }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '12px 16px', borderLeft: '1px solid var(--ck-hairline)', boxShadow: 'inset 0 -2px 0 var(--ck-proof)' }}>
            <span style={{ font: `600 14px ${TF_MONO}`, color: 'var(--ck-proof)' }}>⊢</span>
            <span style={{ font: `700 13px ${TF_UI}`, color: 'var(--ck-fg-1)' }}>Online</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '12px 16px', borderLeft: '1px solid var(--ck-hairline)', background: 'color-mix(in oklab, var(--ck-accent) 8%, transparent)', boxShadow: 'inset 0 -2px 0 var(--ck-accent)' }}>
            <span style={{ font: `600 14px ${TF_MONO}`, color: 'var(--ck-accent)' }}>⊟</span>
            <span style={{ font: `700 13px ${TF_UI}`, color: 'var(--ck-fg-1)' }}>Airgap</span>
          </div>
        </div>
        {/* rows */}
        {OA.map((r, i) => (
          <div key={r.k} style={{ display: 'grid', gridTemplateColumns: '150px 1fr 1fr', borderBottom: i < OA.length - 1 ? '1px solid var(--ck-hairline)' : 'none', alignItems: 'stretch' }}>
            <div style={{ padding: '13px 16px', font: `600 10.5px ${TF_MONO}`, letterSpacing: '.04em', textTransform: 'uppercase', color: 'var(--ck-fg-mute)', display: 'flex', alignItems: 'center' }}>{r.k}</div>
            <div style={{ padding: '13px 16px', borderLeft: '1px solid var(--ck-hairline)', font: `500 12.5px/1.4 ${TF_UI}`, color: 'var(--ck-fg-2)' }}>{r.on}</div>
            <div style={{ padding: '13px 16px', borderLeft: '1px solid var(--ck-hairline)', background: 'color-mix(in oklab, var(--ck-accent) 5%, transparent)', font: `500 12.5px/1.4 ${TF_UI}`, color: 'var(--ck-fg-2)' }}>{r.air}</div>
          </div>
        ))}
      </DotField>
    </TwoCol>
  </section>
);

Object.assign(window, { TrustWall, TransitionStation, OnlineAirgap });
