'use client';

import type { FableSkill } from '../lib/skill-data';
import { TrustStrip } from './TrustStrip';
import Link from 'next/link';

interface SkillHeroProps {
  skill: FableSkill;
}

/* One focal action per hero (rust budget: ≤2 per view, and the
   active nav item may take the second). Everything else is quiet
   or ghost. */
export function SkillHero({ skill }: SkillHeroProps) {
  return (
    <section className="skill-hero" aria-label="Skill hero">
      <div className="ck-label skill-hero-eyebrow">{skill.identity.category} skill</div>

      <h1 className="ck-display skill-hero-title">{skill.name}</h1>
      <p className="skill-hero-tagline">{skill.tagline}</p>

      <TrustStrip status={skill.status} gal={skill.gal} trust={skill.trust} />

      <div className="skill-hero-actions">
        <Link href={`/skills/${skill.id}/topology`} className="ck-btn ck-btn--primary">
          View topology
        </Link>
        <Link href={`/skills/${skill.id}/evidence`} className="ck-btn ck-btn--quiet">
          Evidence
        </Link>
        <Link href="/guide" className="ck-btn ck-btn--ghost">
          Guide
        </Link>
      </div>
    </section>
  );
}
