// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Badge } from './primitives';
// ============================================================
// SkillPack Registry · Shell (Sidebar + Header)
// Web Dashboard chrome. Registry is the live view; sibling nav items
// are present for IA truth but disabled in this prototype.
// ============================================================

const NAV = [
  { id: 'overview',  label: 'Overview',     glyph: '◈', live: true, group: 'console' },
  { id: 'registry',  label: 'Registry',     glyph: '⊞', live: true, group: 'console' },
  { id: 'quality',   label: 'Assurance',    glyph: '◆', live: true, group: 'console' },
  { id: 'discovery', label: 'Discovery',    glyph: '⌖', live: true, group: 'federate' },
  { id: 'migration', label: 'Migration',    glyph: '⇄', live: true, group: 'federate' },
  { id: 'skills',    label: 'My skills',    glyph: '▰', live: true, group: 'local' },
  { id: 'status',    label: 'Sync & status',glyph: '≋', live: true, group: 'local' },
  { id: 'profile',   label: 'Operator',     glyph: '●', live: true, group: 'session' },
];
const GROUPS = [
  { id: 'console',  label: 'Console' },
  { id: 'federate', label: 'Federation' },
  { id: 'local',    label: 'Local store' },
  { id: 'session',  label: 'Session' },
];

const Sidebar = ({ current, onNavigate, health }) => (
  <aside className="ckr-panel" style={{
    width: 224, flexShrink: 0, background: 'var(--ck-bg-0)',
    borderRight: '1px solid color-mix(in oklab, var(--ck-stroke) 25%, transparent)',
    padding: '20px 14px', display: 'flex', flexDirection: 'column', gap: 4,
    position: 'sticky', top: 0, height: '100vh', boxSizing: 'border-box',
  }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '0 8px 18px' }}>
      <img src="/console-assets/mark-a2-favicon.svg" width="26" height="26" alt="" className="ckr-glyph" />
      <div>
        <div style={{ font: "700 13px 'JetBrains Mono', monospace", letterSpacing: '.18em', color: 'var(--ck-fg-1)' }}>SKILLPACK</div>
        <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', letterSpacing: '.1em' }}>web dashboard · C-03</div>
      </div>
    </div>
    <nav aria-label="Primary" style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
      {GROUPS.map(g => (
        <React.Fragment key={g.id}>
          <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', padding: '10px 8px 6px', textTransform: 'uppercase' }}>{g.label}</div>
          {NAV.filter(it => it.group === g.id).map(it => {
            const active = current === it.id;
            return (
              <button key={it.id} onClick={() => onNavigate(it.id)} aria-current={active ? 'page' : undefined}
                title={it.label} className="ckr-row ckr-focusring" style={{
                  display: 'flex', alignItems: 'center', gap: 10, padding: '9px 10px',
                  background: active ? 'var(--ck-bg-1)' : 'transparent',
                  color: active ? 'var(--ck-fg-1)' : 'var(--ck-fg-2)',
                  border: 'none', cursor: 'pointer',
                  font: "500 12px 'Geist', sans-serif", textAlign: 'left',
                  clipPath: chamfer(6),
                  boxShadow: active ? 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 50%, transparent)' : 'none',
                }}>
                <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 13px 'JetBrains Mono', monospace", color: active ? 'var(--ck-accent)' : 'var(--ck-fg-3)', width: 14, textAlign: 'center' }}>{it.glyph}</span>
                <span>{it.label}</span>
                {active && <span aria-hidden="true" style={{ marginLeft: 'auto', font: "600 8px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>⊢</span>}
              </button>
            );
          })}
        </React.Fragment>
      ))}
    </nav>
    <div style={{ marginTop: 'auto', paddingTop: 18 }}>
      <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', padding: '0 8px 8px', textTransform: 'uppercase' }}>Connection</div>
      {(() => {
        const probed = health && health.probed;
        const up = health && health.healthy;
        const glyph = !probed ? '◌' : up ? '⊢' : '⚠';
        const glyphColor = !probed ? 'var(--ck-fg-mute)' : up ? 'var(--ck-accent)' : 'var(--ck-deny)';
        const endpoint = (health && health.endpoint) ? health.endpoint.replace(/^https?:\/\//, '') : ':50052';
        const line2 = !probed ? 'probing…' : up ? `healthy · protocol v${(health && health.protocolVersion) || 1}` : 'unreachable · using sample data';
        return (
          <div className="ckr-row" style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px', clipPath: chamfer(6), boxShadow: `inset 0 0 0 1px ${up ? 'color-mix(in oklab, var(--ck-accent) 40%, transparent)' : 'color-mix(in oklab, var(--ck-stroke) 30%, transparent)'}` }}>
            <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 12px 'JetBrains Mono', monospace", color: glyphColor }}>{glyph}</span>
            <div>
              <div style={{ font: "600 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)' }}>HTTP · {endpoint}</div>
              <div style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{line2}</div>
            </div>
          </div>
        );
      })()}
    </div>
  </aside>
);

const THEMES = [
  { id: 'ledger', label: 'Ledger', sub: 'paper · canonical' },
  { id: 'vault',  label: 'Vault', sub: 'night shift' },
  { id: 'hc',     label: 'HC', sub: 'high contrast' },
];

const Header = ({ eyebrow, title, subtitle, theme, onTheme, badges }) => (
  <header style={{
    display: 'flex', alignItems: 'center', gap: 16, padding: '20px 28px',
    borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)',
    position: 'sticky', top: 0, zIndex: 20, background: 'var(--ck-bg-0)',
  }}>
    <div style={{ flex: 1, minWidth: 0 }}>
      <div style={{ font: "700 10px 'Geist', sans-serif", letterSpacing: '.14em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 2 }}>{eyebrow}</div>
      <h1 style={{ margin: 0, font: "400 30px var(--ck-ff-display)", lineHeight: 1.08, letterSpacing: '0', color: 'var(--ck-fg-1)' }}>{title}</h1>
      {subtitle && <div style={{ font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', letterSpacing: '.04em', marginTop: 4 }}>{subtitle}</div>}
    </div>
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }} aria-hidden="true">
      {badges || (<React.Fragment>
        <Badge kind="attested" title="Governance assurance level 5">⊢ GAL-5</Badge>
        <Badge kind="witness" title="Three active witnesses">⊛ 3 witnesses</Badge>
      </React.Fragment>)}
    </div>
    <div role="group" aria-label="Theme" style={{
      display: 'flex', gap: 2, padding: 2, clipPath: chamfer(6),
      background: 'color-mix(in oklab, var(--ck-stroke) 25%, transparent)',
    }}>
      {THEMES.map(t => (
        <button key={t.id} onClick={() => onTheme(t.id)} aria-pressed={theme === t.id} title={t.sub}
          className="ckr-seg ckr-focusring" aria-selected={theme === t.id} style={{
            font: "700 10px 'JetBrains Mono', monospace", letterSpacing: '.08em',
            padding: '7px 12px', border: 'none', cursor: 'pointer',
            background: theme === t.id ? 'var(--ck-accent)' : 'var(--ck-bg-0)',
            color: theme === t.id ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)',
            clipPath: chamfer(4),
          }}>{t.label}</button>
      ))}
    </div>
  </header>
);

export { Sidebar, Header };
