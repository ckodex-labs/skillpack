'use client';

import Link from 'next/link';
import type { FableSkill } from '../../../lib/skill-data';
import { SkillHero } from '../../../components/SkillHero';
import { AuthorityPath } from '../../../components/AuthorityPath';
import { FunFact } from '../../../components/FunFact';
import { CapabilityList } from '../../../components/CapabilityList';
import { SkillEconomics } from '../../../components/SkillEconomics';
import { FailureStory } from '../../../components/FailureStory';
import { SkillQR } from '../../../components/SkillQR';
import { SkillTopology } from '../../../components/SkillTopology';

interface SkillHeroPageProps {
  skill: FableSkill;
  digest: string;
}

export function SkillHeroPage({ skill, digest }: SkillHeroPageProps) {
  return (
    <>
      <Link href="/" className="back-link">
        ← Back to fleet
      </Link>

      <SkillHero skill={skill} />

      <section className="section" aria-label="Authority path">
        <div className="ck-label section-label">Authority path</div>
        <AuthorityPath authority={skill.authority} />
      </section>

      <section className="section" aria-label="Topology">
        <div className="ck-label section-label">Topology</div>
        <SkillTopology skillName={skill.name} digest={digest} />
      </section>

      <section className="section" aria-label="Field note">
        <FunFact text={skill.identity.funFact} />
      </section>

      <section className="section" aria-label="Capabilities and boundaries">
        <div className="ck-label section-label">Capabilities &amp; boundaries</div>
        <CapabilityList capabilities={skill.capabilities} boundaries={skill.boundaries} />
      </section>

      <section className="section" aria-label="Economics and QR activation">
        <div className="section-grid">
          <div>
            <div className="ck-label section-label">Economics</div>
            <SkillEconomics economics={skill.economics} />
          </div>
          <div>
            <div className="ck-label section-label">QR activation</div>
            <SkillQR target={skill.identity.qrTarget} label={`Scan to activate ${skill.name}`} />
          </div>
        </div>
      </section>

      <section className="section" aria-label="Scorecard">
        <div className="ck-label section-label">Scorecard</div>
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

      <section className="section" aria-label="Failure modes">
        <FailureStory text={skill.failureStory} />
      </section>

      <section className="section" aria-label="Guide">
        <Link href="/guide" className="ck-btn ck-btn--quiet">
          Read the deployment guide
        </Link>
      </section>
    </>
  );
}
