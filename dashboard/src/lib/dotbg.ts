/* ============================================================
 * CKODEX-DS-3 · DotBg v3 — premium engineering canvas.
 * Vendored from the ckodex-design skill (dotbg.js), ported from
 * the window-global IIFE to a typed ES module. Logic unchanged.
 *
 * "Evidence Editorial": paper is FLAT; the canvas is drafting
 * furniture in ink/tone. The semantic budget (rust / violet /
 * red) NEVER appears in a background.
 *
 * Layers:
 *   1. Solid base fill          theme ground
 *   2. Minor dot grid           whisper-fine, r 0.6, spacing 24
 *   3. Major crosshair ticks    + marks every 4th node (96pt)
 *   4. Corner accent clusters   graded lattice in TONE, fades in
 *   5. Engineering L-brackets   precise corner registration
 *   6. Tonal lift               barely-there top sheen
 * ============================================================ */

export type DotBgTheme = 'ledger' | 'vault' | 'hc';

interface DotBgThemeSpec {
  base: string;
  dot: string;
  dotOp: number;
  cross: string;
  crossOp: number;
  fur: string;
  brkOp: number;
  furScale: number;
  lift: string;
  liftOp: number;
  vig: boolean;
  vigCol: string;
  vigOp: number;
}

const THEMES: Record<DotBgTheme, DotBgThemeSpec> = {
  ledger: {
    base: '#F6F1E8', dot: '#211B14', dotOp: 0.07,
    cross: '#211B14', crossOp: 0.16,
    fur: '#6E6457', brkOp: 0.38, furScale: 0.7,
    lift: '#FFFFFF', liftOp: 0.05, vig: false, vigCol: '#211B14', vigOp: 0,
  },
  vault: {
    base: '#0A1322', dot: '#EAE5DA', dotOp: 0.08,
    cross: '#EAE5DA', crossOp: 0.18,
    fur: '#9A9284', brkOp: 0.34, furScale: 0.7,
    lift: '#22304A', liftOp: 0.22, vig: true, vigCol: '#02060E', vigOp: 0.34,
  },
  hc: {
    base: '#000000', dot: '#FFFFFF', dotOp: 0.45,
    cross: '#FFFFFF', crossOp: 0.85,
    fur: '#FFFFFF', brkOp: 1.0, furScale: 1.0,
    lift: '#000000', liftOp: 0, vig: false, vigCol: '#000000', vigOp: 0,
  },
};

export interface DotBgOptions {
  w?: number;
  h?: number;
  theme?: DotBgTheme;
}

export function ckDotBg(opts: DotBgOptions = {}): string {
  const w = opts.w || 320;
  const h = opts.h || 200;
  const t = THEMES[opts.theme || 'ledger'];
  const fur = t.fur;
  const S = 24;      /* minor dot spacing (pt) */
  const MAJ = S * 4; /* major crosshair spacing (96pt) */
  const R = 0.6;     /* minor dot radius */
  const isHc = opts.theme === 'hc';
  const uid = 'dbg' + Math.random().toString(36).slice(2, 8);

  /* major crosshair ticks — a precise + at every 4th node */
  const ca = 2.4;
  const cw = isHc ? 1.0 : 0.75;
  const cc = MAJ / 2;
  const crosshair =
    `<path d="M ${cc - ca} ${cc} H ${cc + ca} M ${cc} ${cc - ca} V ${cc + ca}" ` +
    `stroke="${t.cross}" stroke-width="${cw}" opacity="${t.crossOp}" ` +
    `stroke-linecap="square" fill="none"/>`;

  /* corner clusters — 4x4 lattice anchored at each corner */
  const N = 4;
  const corners = [
    { x: S * 0.5, y: S * 0.5, dx: 1, dy: 1 },
    { x: w - S * 0.5, y: S * 0.5, dx: -1, dy: 1 },
    { x: S * 0.5, y: h - S * 0.5, dx: 1, dy: -1 },
    { x: w - S * 0.5, y: h - S * 0.5, dx: -1, dy: -1 },
  ];
  let clusters = '';
  corners.forEach((c) => {
    for (let i = 0; i < N; i++) {
      for (let j = 0; j < N; j++) {
        const dist = Math.sqrt(i * i + j * j);
        const r = 1.5 - dist * 0.28;
        const op = 0.72 - dist * 0.18;
        if (r <= 0.2 || op <= 0.04) continue;
        const cx = c.x + c.dx * i * S;
        const cy = c.y + c.dy * j * S;
        clusters += `<circle cx="${cx.toFixed(1)}" cy="${cy.toFixed(1)}" r="${r.toFixed(2)}" fill="${fur}" opacity="${(op * t.furScale).toFixed(3)}"/>`;
      }
    }
  });

  /* engineering L-brackets — corner registration, inset 14pt */
  const ins = 14;
  const arm = 8;
  const bw = isHc ? 1.5 : 1.1;
  const brk = (x: number, y: number, sx: number, sy: number) =>
    `<path d="M ${x} ${y + sy * arm} V ${y} H ${x + sx * arm}" fill="none" stroke="${fur}" stroke-width="${bw}" opacity="${t.brkOp}" stroke-linecap="square"/>`;
  const brackets =
    brk(ins, ins, 1, 1) +
    brk(w - ins, ins, -1, 1) +
    brk(ins, h - ins, 1, -1) +
    brk(w - ins, h - ins, -1, -1);

  /* tonal lift — soft sheen from the top, no heavy vignette */
  const lift =
    t.liftOp > 0
      ? `<linearGradient id="${uid}-lift" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="${t.lift}" stop-opacity="${t.liftOp}"/>
      <stop offset="42%" stop-color="${t.lift}" stop-opacity="0"/>
    </linearGradient>`
      : '';
  const liftRect = t.liftOp > 0 ? `<rect width="${w}" height="${h}" fill="url(#${uid}-lift)"/>` : '';

  /* vignette — Vault only; paper and HC stay flat */
  const vig = t.vig
    ? `<radialGradient id="${uid}-vig" cx="50%" cy="46%" r="74%">
      <stop offset="52%" stop-color="${t.vigCol}" stop-opacity="0"/>
      <stop offset="100%" stop-color="${t.vigCol}" stop-opacity="${t.vigOp}"/>
    </radialGradient>`
    : '';
  const vigRect = t.vig ? `<rect width="${w}" height="${h}" fill="url(#${uid}-vig)"/>` : '';

  return `
<svg class="ck-dotbg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="none" aria-hidden="true">
  <defs>
    <pattern id="${uid}-dots" width="${S}" height="${S}" patternUnits="userSpaceOnUse">
      <circle class="ck-dotbg__dot" cx="${S / 2}" cy="${S / 2}" r="${R}" fill="${t.dot}" opacity="${t.dotOp}"/>
    </pattern>
    <pattern id="${uid}-cross" width="${MAJ}" height="${MAJ}" patternUnits="userSpaceOnUse">
      ${crosshair}
    </pattern>
    ${lift}
    ${vig}
  </defs>
  <rect width="${w}" height="${h}" fill="${t.base}"/>
  <rect width="${w}" height="${h}" fill="url(#${uid}-dots)"/>
  <rect width="${w}" height="${h}" fill="url(#${uid}-cross)"/>
  ${clusters}
  ${liftRect}
  ${vigRect}
  ${brackets}
</svg>`.trim();
}
