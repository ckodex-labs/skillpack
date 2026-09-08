'use client';

import type { FableSkill, SkillTrust } from '../lib/skill-data';
import { GalBadge } from './GalBadge';
import { StateChip, type ClaimState } from './ds3';

interface TrustStripProps {
  status: FableSkill['status'];
  gal: number;
  trust: SkillTrust;
}

/* Claim state for a skill record. `attested` (violet) only when the
   record carries attestation metadata: verified status backed by
   required signature and provenance. Routine states are tone. */
export function skillClaimState(status: FableSkill['status'], trust: SkillTrust): ClaimState {
  switch (status) {
    case 'verified':
      return trust.signature === 'required' && trust.provenance === 'required'
        ? 'attested'
        : 'observed';
    case 'draft':
      return 'claimed';
    case 'deprecated':
      return 'contradicted';
    default: {
      const exhaustive: never = status;
      return exhaustive;
    }
  }
}

function TrustBadge({ label, value }: { label: string; value: string }) {
  /* Present / required / denied are all routine governance facts —
     tone and ink, never traffic-light color. Denial is governance,
     not an emergency. */
  const asserted = value === 'present' || value === 'required';
  const glyph = asserted ? '⊢' : value === 'denied' || value.includes('denied') ? '⊘' : '○';

  return (
    <span
      className={`trust-badge${asserted ? ' trust-badge--asserted' : ''}`}
      title={`${label}: ${value.replace(/_/g, ' ')}`}
    >
      <span aria-hidden="true">{glyph}</span>
      {label}
    </span>
  );
}

export function TrustStrip({ status, gal, trust }: TrustStripProps) {
  return (
    <div className="trust-strip" role="list" aria-label="Trust indicators">
      <span role="listitem">
        <StateChip state={skillClaimState(status, trust)} />
      </span>

      <span role="listitem">
        <GalBadge level={gal} />
      </span>

      <span role="listitem">
        <TrustBadge label="SBOM" value={trust.sbom} />
      </span>
      <span role="listitem">
        <TrustBadge label="SIGNATURE" value={trust.signature} />
      </span>
      <span role="listitem">
        <TrustBadge label="PROVENANCE" value={trust.provenance} />
      </span>
      <span role="listitem">
        <TrustBadge label={`SANDBOX: ${trust.sandbox.toUpperCase()}`} value="present" />
      </span>
      <span role="listitem">
        <TrustBadge
          label={`NETWORK: ${trust.network.replace(/_/g, ' ').toUpperCase()}`}
          value={trust.network.includes('denied') ? 'denied' : 'present'}
        />
      </span>
    </div>
  );
}
