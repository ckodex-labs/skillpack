/* CKODEX-DS-3 · QuietCard — the default container. Quiet surfaces are
   SQUARE with a hairline; `sealed` swaps to the chamfered contour — the
   only place the cut corner survives. Seeing it must mean an attested /
   sealed state exists. Requires the vendored DS-3 stylesheets. */

import type { HTMLAttributes } from 'react';

export interface QuietCardProps extends HTMLAttributes<HTMLDivElement> {
  sealed?: boolean;
}

export function QuietCard({ sealed = false, className = '', children, ...rest }: QuietCardProps) {
  const cls = [sealed ? 'ck-sealed' : 'ck-quiet', className].filter(Boolean).join(' ');
  return (
    <div className={cls} {...rest}>
      {children}
    </div>
  );
}
