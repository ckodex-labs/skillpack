// ============================================================
// SkillPack Registry · Install drawer
// Skill detail + 9-dimension assessment + capability disclosure +
// trust verification + streaming install (pull→verify→boundary→sync).
// Maps to tool://skillpack/install { ociRef } and the AssessStream /
// SyncAgentsStream event model. Risky skills hit a security gate first.
// ============================================================
const { useEffect: _useEffect2, useState: _useState2, useRef: _useRef2 } = React;

const CAP_META = {
  read:       { glyph: '◰', label: 'Read' },
  write:      { glyph: '◳', label: 'Write' },
  network:    { glyph: '≋', label: 'Network' },
  filesystem: { glyph: '⊟', label: 'Filesystem' },
  execution:  { glyph: '⚙', label: 'Execution' },
};

const PRIVACY_META = {
  'debug-local':      { tone: 'mute',    note: 'dev tenants · local harness' },
  'audit-private':    { tone: 'witness', note: 'spec default · staging + prod' },
  'public-anchor':    { tone: 'teal',    note: 'transparency-log anchored' },
  'regulated-export': { tone: 'deny',    note: 'sovereign · export-controlled' },
};

const InstallStep = ({ state, glyph, title, detail }) => {
  // state: pending | active | done | warn | fail
  const tone = {
    pending: 'var(--ck-fg-mute)', active: 'var(--ck-accent)', done: 'var(--ck-link)',
    warn: 'var(--ck-text-role)', fail: 'var(--ck-deny)',
  }[state];
  const mark = { pending: glyph, active: glyph, done: '⊢', warn: '⚠', fail: '✕' }[state];
  return (
    <div className="ckr-row" style={{ display: 'flex', alignItems: 'flex-start', gap: 12, padding: '10px 0', opacity: state === 'pending' ? 0.5 : 1 }}>
      <span className={state === 'active' ? 'ckr-glyph ckr-spin' : 'ckr-glyph'} aria-hidden="true" style={{
        font: "600 14px 'JetBrains Mono', monospace", color: tone, width: 18, textAlign: 'center', flexShrink: 0,
        display: 'inline-block',
      }}>{mark}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ font: "600 12px 'JetBrains Mono', monospace", color: state === 'pending' ? 'var(--ck-fg-3)' : 'var(--ck-fg-1)' }}>{title}</div>
        {detail && <div style={{ font: "400 10.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 2, wordBreak: 'break-all' }}>{detail}</div>}
      </div>
    </div>
  );
};

const InstallDrawer = ({ skill, onClose, onInstalled }) => {
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const risky = skill && (skill.signed === 'untrusted' || skill.signed === 'unsigned');
  const [phase, setPhase] = _useState2('idle');      // idle | gate | running | done | failed
  const [step, setStep] = _useState2(-1);
  const closeRef = _useRef2(null);
  const timers = _useRef2([]);

  _useEffect2(() => {
    if (closeRef.current) closeRef.current.focus();
    const onKey = (e) => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('keydown', onKey);
    return () => { document.removeEventListener('keydown', onKey); timers.current.forEach(clearTimeout); };
  }, []);

  if (!skill) return null;
  const prim = PRIMITIVES.find(p => p.id === skill.primitive) || {};
  const reg = REGISTRIES.find(r => r.id === skill.registry) || {};
  const priv = PRIVACY_META[skill.privacy] || {};

  const STEPS = [
    { glyph: '↧', title: 'Resolve · OCI pull', detail: skill.ociRef },
    { glyph: '⊢', title: 'Verify · cosign signature', detail: risky ? 'no valid signature found' : 'signature chains to ckodex root' },
    { glyph: '⊟', title: 'Boundary · IPGuard check', detail: 'POST /api/v1/boundary-check · scope ok' },
    { glyph: '≋', title: 'Sync · fanout to 12 agents', detail: 'symlink × 11 · mdc index × 1' },
  ];
  const failAt = risky ? 1 : -1;

  const run = () => {
    setPhase('running'); setStep(0);
    const delays = reduceMotion ? [0, 0, 0, 0] : [700, 900, 700, 900];
    let acc = 0;
    STEPS.forEach((_, i) => {
      acc += delays[i];
      timers.current.push(setTimeout(() => {
        if (failAt === i) { setStep(i); setPhase('failed'); return; }
        if (i === STEPS.length - 1) { setStep(i + 1); setPhase('done'); }
        else setStep(i + 1);
      }, acc));
    });
  };

  const onPrimary = () => {
    if (skill.installed) return;
    if (risky && phase === 'idle') { setPhase('gate'); return; }
    run();
  };

  const stepState = (i) => {
    if (phase === 'idle' || phase === 'gate') return 'pending';
    if (phase === 'failed') { if (i < failAt) return 'done'; if (i === failAt) return 'fail'; return 'pending'; }
    if (i < step) return 'done';
    if (i === step) return 'active';
    return 'pending';
  };

  return (
    <div role="presentation" onClick={onClose} style={{
      position: 'fixed', inset: 0, zIndex: 50, display: 'flex', justifyContent: 'flex-end',
      background: 'color-mix(in oklab, var(--ck-deep-blue) 62%, transparent)',
    }}>
      <section role="dialog" aria-modal="true" aria-label={`Install ${skill.name}`}
        onClick={(e) => e.stopPropagation()} className="ckr-panel" style={{
          width: 'min(520px, 94vw)', height: '100vh', overflowY: 'auto',
          background: 'var(--ck-bg-0)', boxShadow: 'inset 1.5px 0 0 var(--ck-stroke)',
          display: 'flex', flexDirection: 'column',
        }}>
        {/* header */}
        <div style={{ padding: '22px 24px 18px', borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)', position: 'sticky', top: 0, background: 'var(--ck-bg-0)', zIndex: 2 }}>
          <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12 }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 20px 'JetBrains Mono', monospace", color: 'var(--ck-accent)', lineHeight: 1.2 }}>{prim.glyph}</span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <h2 style={{ margin: 0, font: "700 22px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.01em' }}>{skill.name}</h2>
              <div style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 3 }}>{[skill.version ? `v${skill.version}` : null, skill.publisher, prim.label].filter(Boolean).join(' · ')}</div>
            </div>
            <button ref={closeRef} onClick={onClose} aria-label="Close install panel" className="ckr-focusring" style={{
              border: 'none', background: 'transparent', cursor: 'pointer', color: 'var(--ck-fg-2)',
              font: "400 18px 'JetBrains Mono', monospace", padding: 6, clipPath: chamfer(4),
            }}>✕</button>
          </div>
          <p style={{ margin: '14px 0 0', font: "400 13px 'Geist', sans-serif", color: 'var(--ck-fg-2)', lineHeight: 1.55 }}>{skill.synopsis}</p>
        </div>

        <div style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 20 }}>
          {/* score row */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <GradeChip grade={skill.grade} score={skill.score} size="lg" />
            <div style={{ flex: 1 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                <span style={{ font: "700 13px 'Geist', sans-serif", color: 'var(--ck-fg-1)', whiteSpace: 'nowrap' }}>{skill.score != null ? `Grade ${skill.grade}` : (skill._assessing ? 'Assessing…' : 'Unrated')}</span>
                {skill.score != null && <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', whiteSpace: 'nowrap' }}>score {skill.score} / 120</span>}
              </div>
              <GalRail level={skill.gal || 0} width={140} attested={skill.signed === 'signed' && skill.provenance} />
              <div style={{ font: "600 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.04em', marginTop: 5 }}>
                {skill.gal != null ? `GAL-${skill.gal} · tier ${skill.tier}` : (skill._assessing ? 'running live assessment…' : 'live assessment · GET /api/v1/skills/assess')}
              </div>
            </div>
          </div>

          {/* trust */}
          <div>
            <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 10 }}>Trust &amp; provenance</div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
              <TrustMark state={skill.signed === 'untrusted' ? 'untrusted' : skill.signed === 'unsigned' ? 'unsigned' : skill.signed === 'unknown' ? 'unknown' : 'signed'} />
              {skill.provenance ? <TrustMark state="provenance" /> : <Badge kind="mute" title="No provenance attestation">◌ no provenance</Badge>}
              <Badge kind={reg.trust === 'trusted' ? 'attested' : reg.trust === 'review' ? 'deny' : 'mute'} title={reg.host}>
                <span className="ckr-glyph" aria-hidden="true">{reg.trust === 'trusted' ? '⊢' : reg.trust === 'review' ? '⚠' : '◌'}</span>{reg.label}
              </Badge>
              {skill.privacy && <Badge kind={priv.tone} title={priv.note}>{skill.privacy}</Badge>}
              {skill.profile && <Badge kind="mute" title="Assessment rubric applied to this skill">{(skill.profile || '').toLowerCase().includes('cnsb') ? '⊟ cnsb rubric' : '◆ agentskills rubric'}</Badge>}
            </div>
          </div>

          {/* capabilities */}
          <div>
            <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 10 }}>Requested capabilities</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: 6 }}>
              {Object.keys(CAP_META).map(k => {
                const on = skill.caps[k];
                return (
                  <div key={k} className="ckr-chip" title={`${CAP_META[k].label}: ${on ? 'granted' : 'not requested'}`} style={{
                    display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 5, padding: '10px 4px', clipPath: chamfer(5),
                    boxShadow: `inset 0 0 0 1.5px ${on ? 'var(--ck-stroke)' : 'color-mix(in oklab, var(--ck-fg-mute) 30%, transparent)'}`,
                    background: on ? 'color-mix(in oklab, var(--ck-stroke) 10%, transparent)' : 'transparent', opacity: on ? 1 : 0.5,
                  }}>
                    <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 14px 'JetBrains Mono', monospace", color: on ? 'var(--ck-accent)' : 'var(--ck-fg-mute)' }}>{CAP_META[k].glyph}</span>
                    <span style={{ font: "600 8.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)', letterSpacing: '.02em' }}>{CAP_META[k].label}</span>
                    <span className="ckr-sr">{on ? 'granted' : 'not requested'}</span>
                  </div>
                );
              })}
            </div>
          </div>

          {/* 9-dimension assessment */}
          <div>
            <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 12 }}>9-dimension assessment</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 9 }}>
              {DIMS.map(d => <DimMeter key={d} label={d} score={skill.dims[d]} />)}
            </div>
          </div>

          {/* install zone */}
          <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 16 }}>
            {phase === 'gate' && (
              <div role="alertdialog" aria-label="Security gate" style={{ marginBottom: 14 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                  <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 15px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>⚠</span>
                  <span style={{ font: "700 12px 'Geist', sans-serif", color: 'var(--ck-deny)' }}>
                    {skill.signed === 'untrusted' ? 'SKILLPACK_SEC_UNTRUSTED_REGISTRY' : 'SKILLPACK_SEC_INVALID_SIGNATURE'}
                  </span>
                </div>
                <p style={{ margin: 0, font: "400 12px 'Geist', sans-serif", color: 'var(--ck-fg-2)', lineHeight: 1.5 }}>
                  {skill.signed === 'untrusted'
                    ? `${reg.label} is not in the trust list and this artifact has no verifiable cosign chain. Installing requires an explicit override and will be recorded in the audit log.`
                    : 'This artifact is unsigned — its evidence signature could not be verified against any known signer. Override to install in debug-local mode only.'}
                </p>
              </div>
            )}

            {(phase === 'running' || phase === 'done' || phase === 'failed') && (
              <div aria-live="polite" style={{ marginBottom: 14 }}>
                {STEPS.map((s, i) => <InstallStep key={i} state={stepState(i)} glyph={s.glyph} title={s.title} detail={s.detail} />)}
                {phase === 'done' && (
                  <div className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 8, padding: '10px 12px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px var(--ck-accent)' }}>
                    <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⊢</span>
                    <span style={{ font: "500 11.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>installed → ~/Skills/shared/{skill.name}</span>
                  </div>
                )}
                {phase === 'failed' && (
                  <div className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 8, padding: '10px 12px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px var(--ck-deny)', background: 'color-mix(in oklab, var(--ck-deny) 8%, transparent)' }}>
                    <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>✕</span>
                    <span style={{ font: "500 11.5px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>halted at verify · exit 77 · quarantined as shadow skill</span>
                  </div>
                )}
              </div>
            )}

            <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
              {phase === 'idle' && !skill.installed && (
                <CkButton variant={risky ? 'deny' : 'primary'} onClick={onPrimary} className="ckr-focusring">
                  {risky ? '⚠ Review & install' : '↧ Install to canonical store'}
                </CkButton>
              )}
              {phase === 'idle' && skill.installed && (
                <CkButton variant="secondary" disabled>⊢ Already installed</CkButton>
              )}
              {phase === 'gate' && (
                <React.Fragment>
                  <CkButton variant="deny" onClick={run}>Override &amp; continue</CkButton>
                  <CkButton variant="ghost" onClick={() => setPhase('idle')}>Cancel</CkButton>
                </React.Fragment>
              )}
              {phase === 'running' && <CkButton variant="secondary" disabled>↧ Installing…</CkButton>}
              {phase === 'done' && (
                <React.Fragment>
                  <CkButton variant="primary" onClick={() => { onInstalled && onInstalled(skill.id); onClose(); }}>⊢ Done</CkButton>
                  <CkButton variant="ghost" onClick={onClose}>Close</CkButton>
                </React.Fragment>
              )}
              {phase === 'failed' && (
                <React.Fragment>
                  <CkButton variant="secondary" onClick={() => { setPhase('idle'); setStep(-1); }}>Back</CkButton>
                  <CkButton variant="ghost" onClick={onClose}>Close</CkButton>
                </React.Fragment>
              )}
              <span style={{ marginLeft: 'auto', font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', letterSpacing: '.04em' }}>tool://skillpack/install</span>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

Object.assign(window, { InstallDrawer });
