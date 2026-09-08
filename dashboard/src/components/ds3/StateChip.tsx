/* CKODEX-DS-3 · StateChip — the six epistemic claim states, closed set.
   Glyphs from the CNDL safe-set. Color obeys the budget: only attested
   earns violet, only quarantined earns red. Requires the vendored
   DS-3 stylesheets. */

import type { HTMLAttributes, ReactNode } from 'react';

export type ClaimState =
  | 'observed'
  | 'inferred'
  | 'claimed'
  | 'attested'
  | 'contradicted'
  | 'quarantined';

const CLAIM_GLYPHS: Record<ClaimState, string> = {
  observed: '⊢',
  inferred: '⇝',
  claimed: '○',
  attested: '◆',
  contradicted: '⊭',
  quarantined: '⊘',
};

export interface StateChipProps extends HTMLAttributes<HTMLSpanElement> {
  state?: ClaimState;
  glyph?: string;
  children?: ReactNode;
}

export function StateChip({ state = 'observed', glyph, className = '', children, ...rest }: StateChipProps) {
  const g = glyph !== undefined ? glyph : CLAIM_GLYPHS[state];
  const cls = ['ck-chip', `ck-chip--${state}`, className].filter(Boolean).join(' ');
  return (
    <span className={cls} {...rest}>
      {g ? <span aria-hidden="true">{g}</span> : null}
      {children !== undefined && children !== null ? children : state}
    </span>
  );
}
