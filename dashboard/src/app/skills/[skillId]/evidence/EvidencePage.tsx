'use client';

import Link from 'next/link';
import type { FableSkill } from '../../../../lib/skill-data';
import { TrustStrip } from '../../../../components/TrustStrip';
import { EvidenceHash } from '../../../../components/ds3';

interface EvidencePageProps {
  skill: FableSkill;
  digest: string;
}

interface EvidenceRow {
  label: string;
  value: string;
}

/* Trust posture rows: present/required assert (ink), everything else
   is tone. Denial is governance, not an emergency — no red. */
function rowGlyph(value: string): string {
  if (value === 'present' || value === 'required') return '⊢';
  if (value === 'denied' || value.includes('denied')) return '⊘';
  return '○';
}

function EvidenceRows({ rows, withGlyphs = false }: { rows: EvidenceRow[]; withGlyphs?: boolean }) {
  return (
    <div className="section-card">
      {rows.map((row) => (
        <div
          key={row.label}
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 'var(--ck-sp-3)',
            padding: 'var(--ck-sp-2) 0',
            borderBottom: '1px solid var(--ck-hairline)',
          }}
        >
          <span className="ck-label">{row.label}</span>
          <span className="ck-evidence">
            {withGlyphs ? <span aria-hidden="true">{rowGlyph(row.value)} </span> : null}
            {row.value.replace(/_/g, ' ')}
          </span>
        </div>
      ))}
    </div>
  );
}

export function EvidencePage({ skill, digest }: EvidencePageProps) {
  return (
    <>
      <Link href={`/skills/${skill.id}`} className="back-link">
        ← Back to {skill.name}
      </Link>

      <h1 className="ck-h1" style={{ margin: '0 0 var(--ck-sp-3)' }}>
        Evidence — {skill.name}
      </h1>

      <div style={{ marginBottom: 'var(--ck-sp-4)' }}>
        <EvidenceHash digest={digest} label="content" />
      </div>

      <TrustStrip status={skill.status} gal={skill.gal} trust={skill.trust} />

      <section className="section" style={{ marginTop: 'var(--ck-sp-6)' }}>
        <div className="ck-label section-label">Trust posture</div>
        <EvidenceRows
          withGlyphs
          rows={[
            { label: 'SBOM', value: skill.trust.sbom },
            { label: 'AI-BOM', value: skill.trust.aiBom },
            { label: 'Signature', value: skill.trust.signature },
            { label: 'Provenance', value: skill.trust.provenance },
            { label: 'Sandbox', value: skill.trust.sandbox },
            { label: 'Network', value: skill.trust.network },
            { label: 'Secrets', value: skill.trust.secrets },
          ]}
        />
      </section>

      <section className="section">
        <div className="ck-label section-label">Scorecard metrics</div>
        <div className="scorecard">
          <div className="scorecard-grid">
            {Object.entries(skill.scorecard).map(([key, value]) => (
              <div key={key} className="scorecard-metric">
                <div className="scorecard-metric-value">{value}</div>
                <div className="ck-label scorecard-metric-label">
                  {key.replace(/([A-Z])/g, ' $1').trim()}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="section">
        <div className="ck-label section-label">Authority chain</div>
        <EvidenceRows
          rows={Object.entries(skill.authority).map(([key, value]) => ({
            label: key,
            value,
          }))}
        />
      </section>
    </>
  );
}
