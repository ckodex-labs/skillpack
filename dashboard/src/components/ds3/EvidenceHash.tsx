'use client';

/* CKODEX-DS-3 · EvidenceHash — operable digest. Digest format law:
   algorithm prefix preserved, middle-ellipsis truncation, never below
   a 13px hit area. Click to copy; optional ⊛ opens the evidence
   inspector. Never inert grey text. Requires the vendored DS-3
   stylesheets. */

import { useState } from 'react';
import { formatDigest } from './digest-format';

export interface EvidenceHashProps {
  digest: string;
  label?: string;
  onInspect?: (digest: string) => void;
  className?: string;
}

export function EvidenceHash({ digest, label, onInspect, className = '' }: EvidenceHashProps) {
  const [copied, setCopied] = useState(false);
  const display = formatDigest(digest);

  const copy = () => {
    try {
      if (navigator.clipboard) navigator.clipboard.writeText(digest);
    } catch {
      /* clipboard unavailable — still show feedback */
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };

  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 0 }}>
      <button
        type="button"
        className={`ck-hash ${className}`.trim()}
        title={`${digest} · click to copy`}
        onClick={copy}
        aria-label={`copy digest ${digest}`}
      >
        {label ? <span style={{ color: 'var(--ck-fg-mute)' }}>{label} </span> : null}
        {copied ? '⊢ copied' : display}
      </button>
      {onInspect ? (
        <button
          type="button"
          className="ck-hash"
          style={{ borderLeft: 'none' }}
          onClick={() => onInspect(digest)}
          title="open in evidence inspector"
          aria-label="open in evidence inspector"
        >
          ⊛
        </button>
      ) : null}
    </span>
  );
}
