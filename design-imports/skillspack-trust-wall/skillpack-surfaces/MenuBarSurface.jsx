// ============================================================
// SkillPack · macOS Menu Bar surface (C-04)
// 320px fixed popover. Feature matrix: Install M, Sync M, Status M,
// Lifecycle S — but Registry search/browse MUST NOT. So: daemon status,
// agent sync, install-by-reference, lifecycle approvals. Daemon state
// survives; UI reconnects (offline strategy).
// ============================================================

const MenuChip = ({ children }) => (
  <span style={{ font: "400 12.5px 'JetBrains Mono', monospace", color: 'rgba(255,255,255,.82)', padding: '0 9px' }}>{children}</span>
);

const PopSection = ({ title, right, children }) => (
  <div style={{ padding: '12px 14px', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 16%, transparent)' }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
      <div style={{ font: "700 9px 'Geist', sans-serif", letterSpacing: '.13em', color: 'var(--ck-fg-3)', textTransform: 'uppercase' }}>{title}</div>
      {right && <span style={{ marginLeft: 'auto' }}>{right}</span>}
    </div>
    {children}
  </div>
);

const MenuBarSurface = () => (
  <div data-screen-label="macos menu bar · skillpack" style={{ width: '100%', height: '100%', position: 'relative', overflow: 'hidden', background: 'linear-gradient(150deg, #0a2747 0%, #00152b 45%, #07221c 100%)', fontFamily: 'var(--ck-ff-body)' }}>
    {/* subtle wallpaper orbs */}
    <div aria-hidden="true" style={{ position: 'absolute', top: -80, left: -40, width: 320, height: 320, borderRadius: '50%', background: 'radial-gradient(circle, color-mix(in oklab, var(--ck-teal) 22%, transparent), transparent 70%)' }} />
    <div aria-hidden="true" style={{ position: 'absolute', bottom: -60, right: 40, width: 260, height: 260, borderRadius: '50%', background: 'radial-gradient(circle, color-mix(in oklab, var(--ck-lavender) 16%, transparent), transparent 70%)' }} />

    {/* menu bar */}
    {/* content layer — flow-based so it survives the canvas focus overlay */}
    <div style={{ position: 'relative', zIndex: 1, height: '100%', display: 'flex', flexDirection: 'column' }}>
    {/* menu bar */}
    <div style={{ height: 26, background: 'rgba(3,13,28,.92)', display: 'flex', alignItems: 'center', padding: '0 12px', flexShrink: 0 }}>
      <span style={{ font: "700 13px 'JetBrains Mono', monospace", color: '#fff', marginRight: 14 }}></span>
      <span style={{ font: "700 12.5px 'Geist', sans-serif", color: '#fff', marginRight: 16 }}>SkillsUI</span>
      <MenuChip>File</MenuChip><MenuChip>View</MenuChip><MenuChip>Store</MenuChip>
      <span style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 14 }}>
        {/* highlighted skillpack menu icon */}
        <span style={{ display: 'flex', alignItems: 'center', gap: 6, background: 'rgba(255,255,255,.16)', padding: '3px 8px', borderRadius: 5 }}>
          <span className="ckr-glyph" style={{ font: "500 13px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⊞</span>
          <span style={{ width: 6, height: 6, borderRadius: 4, background: 'var(--ck-accent)' }} />
        </span>
        <span style={{ font: "400 12px 'JetBrains Mono', monospace", color: 'rgba(255,255,255,.8)' }}>􀙇</span>
        <span style={{ font: "400 11.5px 'JetBrains Mono', monospace", color: 'rgba(255,255,255,.8)' }}>100%</span>
        <span style={{ font: "500 12px 'JetBrains Mono', monospace", color: '#fff' }}>14:12</span>
      </span>
    </div>

    {/* popover */}
    <div style={{ display: 'flex', justifyContent: 'flex-end', padding: '8px 16px 0' }}>
      <div style={{ width: 320, position: 'relative' }}>
      {/* notch */}
      <div aria-hidden="true" style={{ position: 'absolute', top: -6, right: 30, width: 14, height: 14, background: 'var(--ck-bg-1)', transform: 'rotate(45deg)', boxShadow: 'inset 1.5px 1.5px 0 color-mix(in oklab, var(--ck-stroke) 40%, transparent)' }} />
      <div className="ckr-panel" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(12), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 42%, transparent), 0 20px 50px rgba(0,0,0,.5)', overflow: 'hidden' }}>
        {/* header */}
        <div style={{ padding: '14px', display: 'flex', alignItems: 'center', gap: 10 }}>
          <img src="../skillpack-registry/assets/mark-a2-favicon.svg" width="24" height="24" alt="" className="ckr-glyph" />
          <div style={{ flex: 1 }}>
            <div style={{ font: "900 13px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '.02em' }}>SkillPack</div>
            <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>daemon · XPC + gRPC :50051</div>
          </div>
          <Badge kind="attested" title="Daemon healthy">⊢ healthy</Badge>
        </div>

        {/* sync */}
        <PopSection title="Agent sync" right={<span style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>last 14:12</span>}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
            <span style={{ font: "800 22px 'Geist', sans-serif", color: 'var(--ck-fg-1)', letterSpacing: '-.02em' }}>11<span style={{ font: "500 13px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}> / 12</span></span>
            <div style={{ flex: 1 }}>
              <span className="ckr-rail" style={{ display: 'block', height: 6, background: 'color-mix(in oklab, var(--ck-fg-mute) 28%, transparent)', clipPath: chamfer(2), position: 'relative' }}>
                <span style={{ position: 'absolute', inset: 0, width: '92%', background: 'var(--ck-accent)' }} />
              </span>
              <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 5 }}>1 pending · Continue (symlink)</div>
            </div>
          </div>
          <CkButton variant="secondary" style={{ width: '100%', justifyContent: 'center', padding: '9px 0' }}>≋ Sync now</CkButton>
        </PopSection>

        {/* install by reference (no search/browse) */}
        <PopSection title="Install by reference">
          <div className="ckr-input" style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 10px', height: 34, background: 'var(--ck-bg-2)', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)', marginBottom: 8 }}>
            <span className="ckr-glyph" style={{ font: "400 11px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>↧</span>
            <span style={{ flex: 1, font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>oci://reg.ckodex.org/skills/proof-audit</span>
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <CkButton variant="primary" style={{ flex: 1, justifyContent: 'center', padding: '9px 0' }}>↧ Install</CkButton>
            <CkButton variant="ghost" style={{ justifyContent: 'center', padding: '9px 12px' }}>⎘ Clipboard</CkButton>
          </div>
          <div style={{ font: "400 9px 'Geist', sans-serif", color: 'var(--ck-fg-mute)', marginTop: 8, lineHeight: 1.4 }}>Browsing the registry is done from the web or VS Code — the menu bar installs by reference only.</div>
        </PopSection>

        {/* lifecycle approval */}
        <PopSection title="Lifecycle" right={<Badge kind="witness">1 gate</Badge>}>
          <div className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '9px 10px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 28%, transparent)' }}>
            <span className="ckr-glyph" style={{ font: "600 13px 'JetBrains Mono', monospace", color: 'var(--ck-witness)' }}>⇗</span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ font: "700 11px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>kernel-policy</div>
              <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>promote L3 → L4 · staging</div>
            </div>
            <button className="ckr-btn ckr-btn--primary ckr-focusring" style={{ border: 'none', background: 'var(--ck-accent)', color: 'var(--ck-deep-blue)', cursor: 'pointer', font: "700 9px 'Geist', sans-serif", letterSpacing: '.06em', textTransform: 'uppercase', padding: '6px 10px', clipPath: chamfer(5) }}>Approve</button>
          </div>
        </PopSection>

        {/* footer */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '11px 14px', borderTop: '1px solid color-mix(in oklab, var(--ck-stroke) 16%, transparent)' }}>
          <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-link)', cursor: 'pointer' }}>Open dashboard ↗</span>
          <span style={{ marginLeft: 'auto', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', cursor: 'pointer' }}>Quit</span>
        </div>
      </div>
      </div>
    </div>
    </div>
  </div>
);

Object.assign(window, { MenuBarSurface });
