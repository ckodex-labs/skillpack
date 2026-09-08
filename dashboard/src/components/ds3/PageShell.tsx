/* CKODEX-DS-3 · PageShell — composition is semantic. Renders real
   landmarks (<header> <nav> <main> <aside> <footer>) on the shell grid.
   The Evidence Margin is an <aside>, always rightmost, never a tab.
   variants: console (nav · main · margin) · document (main · margin) ·
   reading (single measured column; receipts collapse to the footer).

   Ported from the reference JSX with one adaptation: the shell footer
   is passed as a node so the AuthorityFooter (the handling band) can
   be the page footer, per GUARDRAILS §4a. Requires the vendored DS-3
   stylesheets. */

import type { ReactNode } from 'react';

export type ShellVariant = 'console' | 'document' | 'reading';

export interface PageShellProps {
  variant?: ShellVariant;
  brand?: string;
  crumb?: ReactNode;
  urn?: string;
  headerRight?: ReactNode;
  nav?: ReactNode;
  margin?: ReactNode;
  /** Shell footer — typically an AuthorityFooter. */
  footer: ReactNode;
  className?: string;
  children: ReactNode;
}

export function PageShell({
  variant = 'console',
  brand = 'CKODEX',
  crumb,
  urn,
  headerRight,
  nav,
  margin,
  footer,
  className = '',
  children,
}: PageShellProps) {
  const cls = [
    'ck-shell',
    variant !== 'console' ? `ck-shell--${variant}` : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={cls}>
      <header className="ck-shell__header">
        <div className="ck-masthead">
          <span className="ck-masthead__brand">{brand}</span>
          {crumb ? <span className="ck-masthead__crumb">{crumb}</span> : null}
          {urn ? <span className="ck-masthead__urn">{urn}</span> : null}
        </div>
        {headerRight ? <div className="ck-row-3">{headerRight}</div> : null}
      </header>
      {variant === 'console' && nav ? (
        <nav className="ck-shell__nav" aria-label="Primary">
          {nav}
        </nav>
      ) : null}
      <main id="main-content" className="ck-shell__main">
        {children}
      </main>
      {variant !== 'reading' && margin ? (
        <aside className="ck-shell__margin" aria-label="Evidence margin">
          {margin}
        </aside>
      ) : null}
      {footer}
    </div>
  );
}
