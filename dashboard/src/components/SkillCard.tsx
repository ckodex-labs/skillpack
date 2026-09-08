'use client';

import Link from 'next/link';
import type { FableSkill } from '../lib/skill-data';
import { GalBadge } from './GalBadge';
import { StateChip } from './ds3';
import { skillClaimState } from './TrustStrip';

interface SkillCardProps {
  skill: FableSkill;
}

export function SkillCard({ skill }: SkillCardProps) {
  return (
    <Link
      href={`/skills/${skill.id}`}
      className="skill-card"
      aria-label={`${skill.name} — ${skill.tagline}`}
    >
      <div className="skill-card-header">
        <span className="ck-label">{skill.identity.category}</span>
        <StateChip state={skillClaimState(skill.status, skill.trust)} />
      </div>

      <h2 className="skill-card-name">{skill.name}</h2>
      <p className="skill-card-tagline">{skill.tagline}</p>

      <div className="skill-card-footer">
        <GalBadge level={skill.gal} />
        <span className="skill-card-enter">enter →</span>
      </div>
    </Link>
  );
}
