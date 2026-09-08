// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Bento, Badge, CkButton, GradeChip, DimMeter, SectionLabel, StatTile, gradeMeta } from './primitives';
import { DIMS } from './lib/registry-data';
// ============================================================
// SkillPack Registry · Fleet Assurance (QA)
// Computed live from the assessed corpus: grade distribution, fleet score,
// per-dimension fleet averages (where the whole fleet is weakest), and the
// skills that most need attention. Everything here is real — derived from the
// canonical store's 9-dimension assessments.
// ============================================================

const QA_GRADES = ['S+', 'A', 'B', 'C', 'D', 'F'];

const Quality = ({ corpus, onSelect }) => {
  const assessed = corpus.filter(s => s._assessed && s.score != null);
  const n = assessed.length;
  const avg = n ? assessed.reduce((a, s) => a + s.score, 0) / n : 0;

  const hist = Object.fromEntries(QA_GRADES.map(g => [g, assessed.filter(s => s.grade === g).length]));
  const maxHist = Math.max(1, ...Object.values(hist));
  const weak = assessed.filter(s => s.grade === 'D' || s.grade === 'F').length;

  // Per-dimension fleet average (over assessed skills that reported the dim).
  const dimAvg = DIMS.map(d => {
    const vals = assessed.map(s => s.dims[d]).filter(v => v != null);
    return { dim: d, avg: vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : 0 };
  }).sort((a, b) => a.avg - b.avg); // weakest first

  const bottom = [...assessed].sort((a, b) => a.score - b.score).slice(0, 10);

  const gradeTone = (g) => (gradeMeta[g] ? (g === 'S+' ? 'var(--ck-accent)' : gradeMeta[g].ring) : 'var(--ck-fg-mute)');

  return (
    <div style={{ padding: '24px 28px', display: 'flex', flexDirection: 'column', gap: 22 }}>
      {/* headline stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 14 }}>
        <StatTile glyph="◆" label="Fleet score" value={n ? avg.toFixed(1) : '—'} />
        <StatTile glyph="⊞" label="Assessed" value={`${n}/${corpus.length}`} tone="var(--ck-link)" />
        <StatTile glyph="⊢" label="A grade or better" value={hist['S+'] + hist['A']} tone="var(--ck-accent)" />
        <StatTile glyph="⚠" label="Needs work · D–F" value={weak} tone="var(--ck-deny)" />
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 22, alignItems: 'start' }}>
        {/* grade distribution */}
        <Bento pad={20}>
          <SectionLabel style={{ marginBottom: 16 }}>Grade distribution · {n} skills</SectionLabel>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {QA_GRADES.map(g => {
              const c = hist[g];
              const pct = (c / maxHist) * 100;
              return (
                <div key={g} style={{ display: 'grid', gridTemplateColumns: '30px 1fr 34px', alignItems: 'center', gap: 12 }}>
                  <span style={{ font: "800 14px 'Geist', sans-serif", color: gradeTone(g), textAlign: 'center' }}>{g}</span>
                  <span className="ckr-rail" role="img" aria-label={`${g}: ${c} skills`} style={{
                    position: 'relative', height: 12, background: 'color-mix(in oklab, var(--ck-fg-mute) 24%, transparent)',
                    clipPath: chamfer(3), display: 'block',
                  }}>
                    <span style={{ position: 'absolute', inset: 0, width: `${pct}%`, background: gradeTone(g), opacity: g === 'S+' || g === 'F' ? 1 : 0.85 }} />
                  </span>
                  <span style={{ font: "600 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)', textAlign: 'right' }}>{c}</span>
                </div>
              );
            })}
          </div>
        </Bento>

        {/* dimension fleet averages */}
        <Bento pad={20}>
          <SectionLabel style={{ marginBottom: 16 }}>Fleet average by dimension · weakest first</SectionLabel>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {dimAvg.map(({ dim, avg: a }) => <DimMeter key={dim} label={dim} score={Math.round(a)} />)}
          </div>
        </Bento>
      </div>

      {/* needs attention */}
      <Bento pad={20}>
        <SectionLabel style={{ marginBottom: 14 }}>Needs attention · lowest-scoring skills</SectionLabel>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {bottom.map(s => (
            <button key={s.id} onClick={() => onSelect(s)} className="ckr-row ckr-focusring" style={{
              display: 'grid', gridTemplateColumns: 'auto 1fr auto', alignItems: 'center', gap: 14, textAlign: 'left',
              padding: '10px 12px', border: 'none', cursor: 'pointer', clipPath: chamfer(6), background: 'transparent',
              boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 22%, transparent)',
            }}>
              <GradeChip grade={s.grade} score={s.score} />
              <span style={{ minWidth: 0 }}>
                <span style={{ display: 'block', font: "700 13px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>{s.name}</span>
                <span style={{ display: 'block', font: "400 11px 'Geist', sans-serif", color: 'var(--ck-fg-3)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 640 }}>{s.synopsis}</span>
              </span>
              <span style={{ font: "600 12px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)', whiteSpace: 'nowrap' }}>{s.score} / 120 · {s.issues != null ? `${s.issues} issues` : ''}</span>
            </button>
          ))}
        </div>
      </Bento>
    </div>
  );
};

export { Quality };
