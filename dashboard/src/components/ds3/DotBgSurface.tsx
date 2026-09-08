'use client';

/* CKODEX-DS-3 · DotBgSurface — the DotBg v3 drafting canvas as an
   absolutely-positioned background layer for diagrams (GUARDRAILS §7:
   diagrams sit on DotBg). Tracks data-theme so the furniture follows
   the active theme; the budget never appears in a background. */

import { useEffect, useRef, useState } from 'react';
import { ckDotBg, type DotBgTheme } from '../../lib/dotbg';

function currentTheme(): DotBgTheme {
  const t = document.documentElement.getAttribute('data-theme');
  return t === 'vault' || t === 'hc' ? t : 'ledger';
}

export function DotBgSurface() {
  const hostRef = useRef<HTMLDivElement>(null);
  const [markup, setMarkup] = useState('');

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const draw = () => {
      const { width, height } = host.getBoundingClientRect();
      if (width > 0 && height > 0) {
        setMarkup(ckDotBg({ w: Math.round(width), h: Math.round(height), theme: currentTheme() }));
      }
    };

    draw();
    const resize = new ResizeObserver(draw);
    resize.observe(host);
    const themeChange = new MutationObserver(draw);
    themeChange.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    return () => {
      resize.disconnect();
      themeChange.disconnect();
    };
  }, []);

  return (
    <div
      ref={hostRef}
      aria-hidden="true"
      style={{ position: 'absolute', inset: 0, pointerEvents: 'none' }}
      dangerouslySetInnerHTML={{ __html: markup }}
    />
  );
}
