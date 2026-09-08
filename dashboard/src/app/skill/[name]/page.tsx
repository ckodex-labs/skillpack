// @ts-nocheck
'use client';

// Live per-skill detail page. Deep-linkable full-page view of a canonical
// skill's real assessment (grade, score, 9 dimensions, issues), fetched live
// from the daemon. Distinct from the static /skills/[skillId] demo hero pages.

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { SKR_API } from '../../../console/lib/live';
import { chamfer, Bento, Badge, GradeChip, DimMeter, SectionLabel } from '../../../console/primitives';

const DIM_LABEL = {
  IdentityAndManifest: 'identity',
  Security: 'security',
  Provenance: 'provenance',
  Documentation: 'documentation',
  Testing: 'testing',
  Compatibility: 'compatibility',
  Lifecycle: 'lifecycle',
  Governance: 'governance',
  EvalsHitl: 'evals_hitl',
};

const sevTone = (s) =>
  s === 'error' ? 'var(--ck-deny)' : s === 'warning' ? 'var(--ck-text-role)' : 'var(--ck-fg-mute)';

export default function LiveSkillPage() {
  const params = useParams();
  const name = decodeURIComponent(Array.isArray(params.name) ? params.name[0] : params.name || '');
  const [state, setState] = useState({ phase: 'loading', data: null, error: null });

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const r = await fetch(`${SKR_API}/api/v1/skills/assess?path=${encodeURIComponent(name)}`);
        if (!r.ok) throw new Error(`assess failed (${r.status})`);
        const body = await r.json();
        if (!body.assessment) throw new Error('no assessment payload');
        if (alive) setState({ phase: 'ok', data: body.assessment, error: null });
      } catch (e) {
        if (alive) setState({ phase: 'error', data: null, error: String(e.message || e) });
      }
    })();
    return () => {
      alive = false;
    };
  }, [name]);

  const a = state.data;
  const dims = a
    ? (a.dimensions || []).map((d) => ({ label: DIM_LABEL[d.dimensionId] || d.dimensionId, score: Math.round(d.score) }))
    : [];
  const issues = a ? a.issues || [] : [];

  return (
    <div style={{ minHeight: '100vh', background: 'var(--ck-bg-0)', color: 'var(--ck-fg-1)' }}>
      <header
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 16,
          padding: '18px 32px',
          borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)',
        }}
      >
        <Link href="/" style={{ textDecoration: 'none' }}>
          <span
            className="ckr-focusring"
            style={{
              font: "600 11px 'JetBrains Mono', monospace",
              letterSpacing: '.08em',
              color: 'var(--ck-link)',
              padding: '6px 10px',
              clipPath: chamfer(5),
              boxShadow: 'inset 0 0 0 1.2px color-mix(in oklab, var(--ck-stroke) 40%, transparent)',
            }}
          >
            ‹ console
          </span>
        </Link>
        <div>
          <div
            style={{
              font: "700 10px 'Geist', sans-serif",
              letterSpacing: '.14em',
              textTransform: 'uppercase',
              color: 'var(--ck-fg-3)',
            }}
          >
            Canonical store · live assessment
          </div>
          <h1 style={{ margin: 0, font: "400 30px var(--ck-ff-display, 'Instrument Serif'), serif", color: 'var(--ck-fg-1)' }}>
            {name}
          </h1>
        </div>
      </header>

      <main id="main-content" style={{ maxWidth: 980, margin: '0 auto', padding: '28px 32px 80px' }}>
        {state.phase === 'loading' && (
          <div style={{ font: "500 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', display: 'flex', gap: 9, alignItems: 'center' }}>
            <span className="ckr-glyph ckr-spin" aria-hidden="true">◐</span> running live assessment via {SKR_API.replace(/^https?:\/\//, '')} …
          </div>
        )}

        {state.phase === 'error' && (
          <Bento pad={22}>
            <SectionLabel style={{ marginBottom: 8 }}>Assessment unavailable</SectionLabel>
            <div style={{ font: "500 12px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>⊭ {state.error}</div>
            <div style={{ font: "400 12px 'Geist', sans-serif", color: 'var(--ck-fg-3)', marginTop: 8 }}>
              The daemon may be offline, or no skill named <code>{name}</code> exists in the canonical store.
            </div>
          </Bento>
        )}

        {state.phase === 'ok' && a && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
            {/* score header */}
            <Bento pad={24}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 20, flexWrap: 'wrap' }}>
                <GradeChip grade={a.grade} score={a.totalScore != null ? Math.round(a.totalScore) : null} size="lg" />
                <div style={{ flex: 1, minWidth: 200 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap', marginBottom: 6 }}>
                    <span style={{ font: "700 15px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>Grade {a.grade}</span>
                    <span style={{ font: "500 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>
                      score {a.totalScore != null ? Math.round(a.totalScore) : '—'} / 120
                    </span>
                  </div>
                  <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
                    <Badge kind="mute" title="Assessment rubric">
                      {(a.profile || '').toLowerCase().includes('cnsb') ? '⊟ cnsb rubric' : '◆ agentskills rubric'}
                    </Badge>
                    <Badge kind={issues.length ? 'witness' : 'attested'} title="Issues found">
                      ⚑ {issues.length} issue{issues.length === 1 ? '' : 's'}
                    </Badge>
                    <Badge kind="mute" title="Store path">{a.skillId || name}</Badge>
                  </div>
                </div>
              </div>
            </Bento>

            {/* 9-dimension breakdown */}
            <Bento pad={22}>
              <SectionLabel style={{ marginBottom: 16 }}>9-dimension assessment</SectionLabel>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                {dims.map((d) => (
                  <DimMeter key={d.label} label={d.label} score={d.score} />
                ))}
              </div>
            </Bento>

            {/* issues */}
            {issues.length > 0 && (
              <Bento pad={22}>
                <SectionLabel style={{ marginBottom: 14 }}>Issues · {issues.length}</SectionLabel>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {issues.map((iss, i) => (
                    <div
                      key={i}
                      className="ckr-row"
                      style={{
                        display: 'flex',
                        alignItems: 'flex-start',
                        gap: 12,
                        padding: '10px 12px',
                        clipPath: chamfer(6),
                        boxShadow: `inset 0 0 0 1px color-mix(in oklab, ${sevTone(iss.severity)} 40%, transparent)`,
                      }}
                    >
                      <span
                        aria-hidden="true"
                        style={{ font: "600 12px 'JetBrains Mono', monospace", color: sevTone(iss.severity), width: 54, flexShrink: 0, textTransform: 'uppercase', letterSpacing: '.04em' }}
                      >
                        {iss.severity}
                      </span>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div style={{ font: "400 12.5px 'Geist', sans-serif", color: 'var(--ck-fg-1)', lineHeight: 1.5 }}>{iss.message}</div>
                        {(iss.path || iss.line != null) && (
                          <div style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 3 }}>
                            {iss.path}
                            {iss.line != null ? `:${iss.line}` : ''} · {iss.dimensionId}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </Bento>
            )}
          </div>
        )}
      </main>
    </div>
  );
}
