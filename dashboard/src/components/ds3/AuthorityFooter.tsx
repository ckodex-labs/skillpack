/* CKODEX-DS-3 · AuthorityFooter — the handling band.
   Borrows the classification-banner idea, bound to the budget: the
   level is an assertion. open / internal / restricted carry no hue
   (presence is ink weight + inversion); only `sealed` earns violet
   (a proof object exists — pass `digest`) and only `contained` earns
   red (an active emergency protocol — pass `ep`). The band states
   controlling authority · handling level · environment + mode, with
   the verbatim invariant beside the mode it governs.

   Ported from the reference JSX; the on-demand dock variant is not
   ported because nothing in this app uses it. The proof/EP guards are
   enforced at the type level: `sealed` requires `digest`, `contained`
   requires `ep`. Requires the vendored DS-3 stylesheets. */

import { formatDigest } from './digest-format';

export type AuthorityLevel = 'open' | 'internal' | 'restricted' | 'sealed' | 'contained';

const AUTHORITY_LEVELS: Record<AuthorityLevel, { glyph: string; label: string; handling: string }> = {
  open: { glyph: '○', label: 'Open', handling: 'unrestricted handling' },
  internal: { glyph: '◇', label: 'Internal', handling: 'controlled handling' },
  restricted: { glyph: '◈', label: 'Restricted', handling: 'sensitive · need-to-know' },
  sealed: { glyph: '◆', label: 'Sealed', handling: 'proof-bound environment' },
  contained: { glyph: '⊘', label: 'Contained', handling: 'active emergency protocol' },
};

/* Verbatim invariant — literal, never paraphrased. */
const AUTHORITY_INVARIANT = 'mode changes deployment, not governance semantics';

interface AuthorityFooterBaseProps {
  authority?: string;
  policySet?: string;
  environment?: string;
  mode?: string;
  invariant?: boolean;
  className?: string;
}

/* The palette is the policy: violet requires a proof object, red an EP. */
export type AuthorityFooterProps = AuthorityFooterBaseProps &
  (
    | { level?: 'open' | 'internal' | 'restricted'; digest?: never; ep?: never }
    | { level: 'sealed'; digest: string; ep?: never }
    | { level: 'contained'; ep: string; digest?: never }
  );

export function AuthorityFooter({
  level = 'internal',
  authority = 'ckodex-gov',
  policySet,
  environment = 'gap',
  mode = 'enforce',
  digest,
  ep,
  invariant = true,
  className = '',
}: AuthorityFooterProps) {
  const cfg = AUTHORITY_LEVELS[level];
  const authorityText = `⊢ ${authority}${policySet ? ` / ${policySet}` : ''}`;

  let endK: string;
  let endV: string;
  if (level === 'sealed' && digest) {
    endK = 'policy seal';
    endV = formatDigest(digest);
  } else if (level === 'contained') {
    endK = 'emergency protocol';
    endV = ep ?? 'active';
  } else {
    endK = 'environment';
    endV = `${environment} · mode: ${mode}`;
  }

  return (
    <footer
      className={`ck-authority ck-authority--${level} ${className}`.trim()}
      role="contentinfo"
      aria-label={`Authority footer — ${cfg.label} handling`}
    >
      <div className="ck-authority__zone ck-authority__zone--start">
        <span className="ck-authority__k">authority</span>
        <span className="ck-authority__v" title={authority + (policySet ? ` / ${policySet}` : '')}>
          {authorityText}
        </span>
      </div>
      <div className="ck-authority__center">
        <span className="ck-authority__level">
          <span className="ck-authority__glyph" aria-hidden="true">
            {cfg.glyph}
          </span>
          {cfg.label}
        </span>
        <span className="ck-authority__handling">{cfg.handling}</span>
      </div>
      <div className="ck-authority__zone ck-authority__zone--end">
        <span className="ck-authority__k">{endK}</span>
        <span className="ck-authority__v">{endV}</span>
        {invariant ? <span className="ck-authority__invariant">{AUTHORITY_INVARIANT}</span> : null}
      </div>
    </footer>
  );
}
