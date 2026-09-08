'use client';

/* CKODEX-DS-3 · ThemeSwitch — cycles data-theme through the three
   author-selectable themes (ledger · vault · hc) and persists the
   choice. Forced-colors is the fourth mandatory theme; it activates
   from the OS via media query and is never gated here. */

import { useEffect, useSyncExternalStore } from 'react';

const THEMES = ['ledger', 'vault', 'hc'] as const;
export type ThemeName = (typeof THEMES)[number];

export const THEME_STORAGE_KEY = 'ck-theme';

function isThemeName(value: string | null): value is ThemeName {
  return value !== null && (THEMES as readonly string[]).includes(value);
}

/* The <html data-theme> attribute is the source of truth; the switch
   subscribes to it instead of duplicating it in component state. */
function subscribeToTheme(onChange: () => void): () => void {
  const observer = new MutationObserver(onChange);
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme'],
  });
  return () => observer.disconnect();
}

function readTheme(): ThemeName {
  const current = document.documentElement.getAttribute('data-theme');
  return isThemeName(current) ? current : 'ledger';
}

export function ThemeSwitch() {
  const theme = useSyncExternalStore(subscribeToTheme, readTheme, (): ThemeName => 'ledger');

  /* The pre-paint restore script sets data-theme before first paint, but
     React 19 hydration strips attributes it does not render on <html>.
     Re-apply the persisted theme once hydration is done. */
  useEffect(() => {
    let stored: string | null = null;
    try {
      stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    } catch {
      return;
    }
    if (isThemeName(stored) && stored !== readTheme()) {
      document.documentElement.setAttribute('data-theme', stored);
    }
  }, []);

  const cycle = () => {
    const next = THEMES[(THEMES.indexOf(theme) + 1) % THEMES.length];
    document.documentElement.setAttribute('data-theme', next);
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch {
      /* storage unavailable — theme still applies for this page */
    }
  };

  return (
    <button
      type="button"
      className="ck-btn ck-btn--ghost"
      onClick={cycle}
      title="Cycle theme: ledger, vault, hc. Forced-colors follows the OS."
      aria-label={`Theme: ${theme}. Activate to cycle to the next theme.`}
    >
      theme · {theme}
    </button>
  );
}
