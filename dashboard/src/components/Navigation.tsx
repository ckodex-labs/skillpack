'use client';

import Link from 'next/link';
import { usePathname, useSearchParams } from 'next/navigation';
import { Suspense } from 'react';

/* Nav rail items for the console shell. Rendered inside PageShell's
   <nav> landmark; the active item carries ink presence (no hue). */
function NavLinksContent() {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const isAssessment = searchParams.get('view') === 'assessment';

  const links = [
    {
      href: '/',
      label: 'Fleet gallery',
      isActive: pathname === '/' && !isAssessment,
    },
    {
      href: '/?view=assessment',
      label: 'Workspace assessment',
      isActive: pathname === '/' && isAssessment,
    },
    {
      href: '/guide',
      label: 'Deployment guide',
      isActive: pathname === '/guide',
    },
  ];

  return (
    <>
      {links.map((link) => (
        <Link
          key={link.href}
          href={link.href}
          className="ck-nav__item"
          aria-current={link.isActive ? 'page' : undefined}
        >
          {link.label}
        </Link>
      ))}
    </>
  );
}

export function NavLinks() {
  return (
    <Suspense fallback={null}>
      <NavLinksContent />
    </Suspense>
  );
}
