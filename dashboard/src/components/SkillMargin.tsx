import type { FableSkill } from '../lib/skill-data';
import { EvidenceMargin, type Receipt } from './ds3';

interface SkillMarginProps {
  skill: FableSkill;
  /** Content digest of the skill record — the proof object. */
  digest: string;
  /** UTC timestamp at which the digest was generated (page render). */
  generatedAt: string;
}

/* The Evidence Margin for skill pages: receipts derived from the
   skill record, persistent in the <aside> — never a tab. The violet
   rule appears only on the digest receipt (a proof object exists);
   the provenance stamp gates the record the same way. */
export function SkillMargin({ skill, digest, generatedAt }: SkillMarginProps) {
  const scores = Object.values(skill.scorecard);
  const meanScore = scores.reduce((a, b) => a + b, 0) / scores.length;
  const attested =
    skill.status === 'verified' &&
    skill.trust.signature === 'required' &&
    skill.trust.provenance === 'required';

  const receipts: Receipt[] = [
    {
      time: generatedAt,
      event: 'generated · content digest',
      kind: 'proof',
      details: [digest.slice(0, 19) + '…'],
    },
    {
      time: generatedAt,
      event: 'evaluated · trust posture',
      details: [
        `sbom ${skill.trust.sbom} · signature ${skill.trust.signature}`,
        `provenance ${skill.trust.provenance} · sandbox ${skill.trust.sandbox}`,
        `network ${skill.trust.network.replace(/_/g, ' ')} · secrets ${skill.trust.secrets}`,
      ],
    },
    {
      time: generatedAt,
      event: 'evaluated · scorecard',
      details: [`${scores.length} dimensions · mean ${meanScore.toFixed(1)}`],
    },
  ];

  return <EvidenceMargin entries={receipts} stamp={attested ? digest : undefined} />;
}
