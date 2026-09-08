'use client';

import Link from 'next/link';
import type { FableSkill } from '../../../../lib/skill-data';
import { SkillTopology } from '../../../../components/SkillTopology';
import { TrustStrip } from '../../../../components/TrustStrip';

interface TopologyPageProps {
  skill: FableSkill;
  digest: string;
}

const LEGEND: { label: string; value: string }[] = [
  { label: 'Inputs', value: 'Catalogs, profiles, and source artifacts' },
  { label: 'Skill core', value: 'Processing engine — kernel double-stroke' },
  { label: 'Policy gate', value: 'OPA policy check, GAL verification' },
  { label: 'Evidence', value: 'DCA attestation, proof generation' },
  { label: 'Receipt', value: 'Signed SHA-256 output artifact — attested edge' },
];

export function TopologyPage({ skill, digest }: TopologyPageProps) {
  return (
    <>
      <Link href={`/skills/${skill.id}`} className="back-link">
        ← Back to {skill.name}
      </Link>

      <h1 className="ck-h1" style={{ margin: '0 0 var(--ck-sp-3)' }}>
        Topology — {skill.name}
      </h1>
      <p style={{ color: 'var(--ck-fg-3)', margin: '0 0 var(--ck-sp-5)' }}>
        Data flow: Inputs → Skill Core → Policy Gate → Evidence → Receipt
      </p>

      <TrustStrip status={skill.status} gal={skill.gal} trust={skill.trust} />

      <section
        className="section"
        style={{ marginTop: 'var(--ck-sp-6)' }}
        aria-label="Full topology diagram"
      >
        <SkillTopology skillName={skill.name} digest={digest} />
      </section>

      <section className="section">
        <div className="ck-label section-label">Topology legend</div>
        <div className="section-card">
          {LEGEND.map((row) => (
            <div
              key={row.label}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'baseline',
                gap: 'var(--ck-sp-3)',
                padding: 'var(--ck-sp-2) 0',
                borderBottom: '1px solid var(--ck-hairline)',
              }}
            >
              <span className="ck-label">{row.label}</span>
              <span className="ck-evidence">{row.value}</span>
            </div>
          ))}
        </div>
      </section>
    </>
  );
}
