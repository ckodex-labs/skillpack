/* CKODEX-DS-3 · EvidenceMargin — the signature primitive. Evidence is
   never a tab; it is in the margin while the operator works. Renders a
   list of receipts plus the provenance stamp. Receipt color obeys the
   budget: violet rule = proof object exists; red rule = active EP only.
   Requires the vendored DS-3 stylesheets. */

import type { ReactNode } from 'react';
import { ProvenanceStamp } from './ProvenanceStamp';

export type ReceiptKind = 'proof' | 'alarm';

export interface Receipt {
  time: string;
  event: string;
  kind?: ReceiptKind;
  details?: string[];
}

export interface EvidenceMarginProps {
  title?: string;
  entries?: Receipt[];
  /** Digest for the provenance stamp — pass only when a proof object exists. */
  stamp?: string;
  className?: string;
  children?: ReactNode;
}

function receiptModifier(kind: ReceiptKind | undefined): string {
  switch (kind) {
    case 'proof':
      return ' ck-receipt--proof';
    case 'alarm':
      return ' ck-receipt--alarm';
    case undefined:
      return '';
    default: {
      const exhaustive: never = kind;
      return exhaustive;
    }
  }
}

function receiptGlyph(kind: ReceiptKind | undefined): string {
  switch (kind) {
    case 'proof':
      return '◆ ';
    case 'alarm':
      return '⊘ ';
    case undefined:
      return '';
    default: {
      const exhaustive: never = kind;
      return exhaustive;
    }
  }
}

export function EvidenceMargin({
  title = 'Evidence margin',
  entries = [],
  stamp,
  className = '',
  children,
}: EvidenceMarginProps) {
  return (
    <div className={`ck-stack-3 ${className}`.trim()}>
      <div className="ck-label">{title}</div>
      {entries.map((entry, i) => (
        <div key={i} className={`ck-receipt${receiptModifier(entry.kind)}`}>
          <div className="ck-receipt__rule"></div>
          <div className="ck-stack-1">
            <div className="ck-receipt__time">{entry.time}</div>
            <div className="ck-receipt__event">
              {receiptGlyph(entry.kind)}
              {entry.event}
            </div>
            {(entry.details || []).map((detail, j) => (
              <div key={j} className="ck-receipt__detail">
                {detail}
              </div>
            ))}
          </div>
        </div>
      ))}
      {children}
      {stamp ? (
        <div>
          <ProvenanceStamp digest={stamp} />
        </div>
      ) : null}
    </div>
  );
}
