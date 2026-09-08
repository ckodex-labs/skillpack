// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Bento, Badge, CkButton, SectionLabel } from './primitives';
// ============================================================
// SkillPack Registry · Shell surfaces: Splash · Auth · Profile
// A real boot flow: brand splash while the daemon is probed → operator
// sign-in (bearer token, which the client actually sends) → console.
// ============================================================

// ---------- SPLASH ----------
const Splash = ({ health }) => {
  const line = !health.probed
    ? 'connecting to daemon…'
    : health.healthy
      ? `daemon healthy · protocol v${health.protocolVersion || 1} · loading catalog…`
      : 'daemon offline · entering sample mode';
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--ck-bg-0)', display: 'grid', placeItems: 'center', zIndex: 100 }}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 26 }}>
        <img src="/console-assets/mark-a2-favicon.svg" width="72" height="72" alt="" className="ckr-glyph" />
        <div style={{ textAlign: 'center' }}>
          <div style={{ font: "700 22px 'JetBrains Mono', monospace", letterSpacing: '.26em', color: 'var(--ck-fg-1)' }}>SKILLPACK</div>
          <div style={{ font: "500 11px 'JetBrains Mono', monospace", letterSpacing: '.14em', color: 'var(--ck-fg-mute)', marginTop: 7 }}>the governed skills registry</div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 9, font: "500 11px 'JetBrains Mono', monospace", color: health.probed && !health.healthy ? 'var(--ck-text-role)' : 'var(--ck-fg-3)' }}>
          <span className="ckr-glyph ckr-spin" aria-hidden="true" style={{ display: 'inline-block' }}>◐</span>
          {line}
        </div>
      </div>
    </div>
  );
};

// ---------- AUTH ----------
const Auth = ({ health, onEnter }) => {
  const [token, setToken] = React.useState('');
  const endpoint = (health.endpoint || 'http://localhost:50052').replace(/^https?:\/\//, '');
  const submit = (e) => { if (e) e.preventDefault(); onEnter(token.trim() || null); };
  return (
    <div style={{ position: 'fixed', inset: 0, background: 'var(--ck-bg-0)', display: 'grid', placeItems: 'center', zIndex: 100, padding: 24 }}>
      <form onSubmit={submit} className="ckr-bento" style={{
        width: 'min(460px, 94vw)', background: 'var(--ck-bg-1)', clipPath: chamfer(12),
        boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)', padding: '34px 34px 28px',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 22 }}>
          <img src="/console-assets/mark-a2-favicon.svg" width="30" height="30" alt="" className="ckr-glyph" />
          <div>
            <div style={{ font: "700 13px 'JetBrains Mono', monospace", letterSpacing: '.18em', color: 'var(--ck-fg-1)' }}>SKILLPACK</div>
            <div style={{ font: "500 9px 'JetBrains Mono', monospace", letterSpacing: '.1em', color: 'var(--ck-fg-mute)' }}>web dashboard · C-03</div>
          </div>
          <div style={{ marginLeft: 'auto' }}>
            <Badge kind={health.healthy ? 'attested' : 'mute'} title={endpoint}>
              <span className="ckr-glyph" aria-hidden="true">{health.healthy ? '⊢' : '◌'}</span>{health.healthy ? 'daemon up' : 'offline'}
            </Badge>
          </div>
        </div>

        <h2 style={{ margin: '0 0 6px', font: "400 26px var(--ck-ff-display, 'Instrument Serif'), serif", color: 'var(--ck-fg-1)' }}>Operator sign-in</h2>
        <p style={{ margin: '0 0 20px', font: "400 12.5px 'Geist', sans-serif", color: 'var(--ck-fg-2)', lineHeight: 1.55 }}>
          Present a bearer token to authorize mutations, or continue &mdash; this daemon runs with token auth disabled, so read + assess are open.
        </p>

        <label htmlFor="skr-token" style={{ display: 'block', font: "700 10px 'Geist', sans-serif", letterSpacing: '.12em', textTransform: 'uppercase', color: 'var(--ck-fg-3)', marginBottom: 8 }}>API token · optional</label>
        <input id="skr-token" type="password" value={token} onChange={(e) => setToken(e.target.value)} autoComplete="off"
          placeholder="paste SKILLPACK_API_TOKEN, or leave blank" className="ckr-input ckr-focusring" style={{
            width: '100%', boxSizing: 'border-box', border: 'none', background: 'var(--ck-bg-0)', color: 'var(--ck-fg-1)',
            font: "400 13px 'JetBrains Mono', monospace", padding: '12px 14px', clipPath: chamfer(6),
            boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 45%, transparent)', marginBottom: 20,
          }} />

        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <CkButton variant="primary" onClick={submit}>⊢ Enter console</CkButton>
          <span style={{ marginLeft: 'auto', font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{token.trim() ? 'bearer token → Authorization' : 'no token · read-only-safe'}</span>
        </div>
      </form>
    </div>
  );
};

// ---------- PROFILE ----------
const Profile = ({ session, health, theme, onTheme, onSignOut, onOpenHero }) => {
  const endpoint = (health.endpoint || 'http://localhost:50052').replace(/^https?:\/\//, '');
  const card = (children) => (
    <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 34%, transparent)', padding: 20 }}>{children}</div>
  );
  const row = (k, v, tone) => (
    <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: 12, padding: '7px 0' }}>
      <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>{k}</span>
      <span style={{ font: "600 12px 'JetBrains Mono', monospace", color: tone || 'var(--ck-fg-1)', textAlign: 'right' }}>{v}</span>
    </div>
  );
  return (
    <div style={{ padding: '24px 28px', display: 'flex', flexDirection: 'column', gap: 22 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
        <div className="ckr-badge" style={{ width: 56, height: 56, display: 'grid', placeItems: 'center', background: 'var(--ck-accent)', color: 'var(--ck-deep-blue, #0A1322)', clipPath: chamfer(9), font: "800 22px 'Geist', sans-serif" }}>
          {(session && session.operator ? session.operator[0] : 'O').toUpperCase()}
        </div>
        <div>
          <div style={{ font: "700 18px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{session && session.operator ? session.operator : 'local operator'}</div>
          <div style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{session && session.token ? 'authenticated · bearer token' : 'anonymous · read + assess'}</div>
        </div>
        <div style={{ marginLeft: 'auto' }}>
          <CkButton variant="deny" onClick={onSignOut}>⊘ Sign out</CkButton>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 18, alignItems: 'start' }}>
        {card(<React.Fragment>
          <SectionLabel style={{ marginBottom: 12 }}>Connection</SectionLabel>
          {row('daemon', health.healthy ? 'healthy' : 'offline', health.healthy ? 'var(--ck-accent)' : 'var(--ck-text-role)')}
          {row('endpoint', endpoint)}
          {row('protocol', `v${health.protocolVersion || 1}`)}
          {row('token', session && session.token ? 'present' : 'none')}
        </React.Fragment>)}
        {card(<React.Fragment>
          <SectionLabel style={{ marginBottom: 12 }}>Preferences</SectionLabel>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '7px 0' }}>
            <span style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)' }}>theme</span>
            <div role="group" aria-label="Theme" style={{ display: 'flex', gap: 2, padding: 2, clipPath: chamfer(5), background: 'color-mix(in oklab, var(--ck-stroke) 25%, transparent)' }}>
              {['ledger', 'vault', 'hc'].map(t => (
                <button key={t} onClick={() => onTheme(t)} aria-pressed={theme === t} className="ckr-seg ckr-focusring" style={{
                  font: "700 9px 'JetBrains Mono', monospace", letterSpacing: '.08em', textTransform: 'uppercase',
                  padding: '6px 10px', border: 'none', cursor: 'pointer', clipPath: chamfer(3),
                  background: theme === t ? 'var(--ck-accent)' : 'var(--ck-bg-0)',
                  color: theme === t ? 'var(--ck-deep-blue, #0A1322)' : 'var(--ck-fg-2)',
                }}>{t}</button>
              ))}
            </div>
          </div>
          {row('storage', '~/Skills/shared')}
          <div style={{ marginTop: 12, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
            <a href="../skillpack-hero/index.html" className="ckr-focusring" style={{ textDecoration: 'none' }}>
              <CkButton variant="secondary">◆ View the hero</CkButton>
            </a>
            <a href="../index.html" className="ckr-focusring" style={{ textDecoration: 'none' }}>
              <CkButton variant="ghost">≡ All surfaces</CkButton>
            </a>
          </div>
        </React.Fragment>)}
      </div>
    </div>
  );
};

export { Splash, Auth, Profile };
