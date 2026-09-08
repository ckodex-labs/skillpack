/* CKODEX-DS-3 · Button — square, mono-label. The chamfer is sealed-only,
   so buttons carry no cut corners. `primary` (rust) is the focal action —
   the budget allows at most two per view. `emergency` (red) requires an
   active emergency protocol. Requires the vendored DS-3 stylesheets. */

import type { ButtonHTMLAttributes } from 'react';

export type ButtonVariant = 'primary' | 'quiet' | 'ghost' | 'emergency';

export interface Ds3ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

export function Button({
  variant = 'quiet',
  type = 'button',
  className = '',
  children,
  ...rest
}: Ds3ButtonProps) {
  const cls = ['ck-btn', `ck-btn--${variant}`, className].filter(Boolean).join(' ');
  return (
    <button type={type} className={cls} {...rest}>
      {children}
    </button>
  );
}
