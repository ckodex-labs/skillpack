'use client';

import { GAL_NAMES } from '../lib/skill-data';

interface GalBadgeProps {
  level: number;
}

/* GAL rank carries no hue. The glyph fills as autonomy rises
   (CNDL safe-set), ink weight steps at GAL-3, and GAL-5 inverts
   as the caution mark — presence without color. */
const GAL_GLYPHS: Record<number, string> = {
  0: '○',
  1: '◐',
  2: '◇',
  3: '◈',
  4: '◆',
  5: '⊘',
};

function galVariant(level: number): string {
  if (level >= 5) return ' gal-badge--caution';
  if (level >= 3) return ' gal-badge--strong';
  return '';
}

export function GalBadge({ level }: GalBadgeProps) {
  const name = GAL_NAMES[level] ?? `GAL-${level}`;
  const glyph = GAL_GLYPHS[level] ?? '○';

  return (
    <span
      className={`gal-badge${galVariant(level)}`}
      title={`Governance Autonomy Level ${level}: ${name}`}
      aria-label={`GAL ${level} — ${name}`}
    >
      <span aria-hidden="true">{glyph}</span>
      GAL {level}
    </span>
  );
}
