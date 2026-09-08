/* @ds-bundle: {"format":3,"namespace":"CkodexDesignSystem_042c8d","components":[{"name":"Button","sourcePath":"components/Button.jsx"},{"name":"EvidenceHash","sourcePath":"components/EvidenceHash.jsx"},{"name":"EvidenceMargin","sourcePath":"components/EvidenceMargin.jsx"},{"name":"PageShell","sourcePath":"components/PageShell.jsx"},{"name":"ProvenanceStamp","sourcePath":"components/ProvenanceStamp.jsx"},{"name":"QuietCard","sourcePath":"components/QuietCard.jsx"},{"name":"StateChip","sourcePath":"components/StateChip.jsx"}],"sourceHashes":{"components/Button.jsx":"34c9647fda88","components/EvidenceHash.jsx":"a7b342914354","components/EvidenceMargin.jsx":"9cfb0a71ffcd","components/PageShell.jsx":"906a115b0e58","components/ProvenanceStamp.jsx":"c7d3232aba6f","components/QuietCard.jsx":"f9b81787944b","components/StateChip.jsx":"bfd4d3a1eb2e","dotbg.js":"68e92ecfd6cb","redesign-v2/console-v2-primitives.jsx":"d571e22e543b","redesign-v2/console-v2-shell.jsx":"06b674426447","redesign-v2/console-v2-views.jsx":"c1a54216f65d","redesign-v2/hero-scenes.js":"8f83f8ad7c2c","redesign-v2/tweaks-panel.jsx":"6591467622ed","ui_kits/governance-console/App.jsx":"28dbdb437dd1","ui_kits/governance-console/ArtifactsView.jsx":"6b9737a2f69c","ui_kits/governance-console/AttestationReview.jsx":"e096c3a804e6","ui_kits/governance-console/Header.jsx":"cdcbc76a25c4","ui_kits/governance-console/KernelDetail.jsx":"8a38d9a7681b","ui_kits/governance-console/Overview.jsx":"686ed31768a1","ui_kits/governance-console/PolicyView.jsx":"23912accb24b","ui_kits/governance-console/RekorView.jsx":"65929cb76963","ui_kits/governance-console/Sidebar.jsx":"748b6f19dc9c","ui_kits/governance-console/primitives.jsx":"7fa0b429e55c"},"inlinedExternals":[],"unexposedExports":[]} */

(() => {

const __ds_ns = (window.CkodexDesignSystem_042c8d = window.CkodexDesignSystem_042c8d || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · Button — square, mono-label. The chamfer is sealed-only,
   so buttons carry no cut corners. `primary` (rust) is the focal action —
   the budget allows at most two per view. `emergency` (red) requires an
   active emergency protocol. Requires styles.css. */
function Button({
  variant = 'quiet',
  type = 'button',
  className = '',
  children,
  ...rest
}) {
  const cls = ['ck-btn', 'ck-btn--' + variant, className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement("button", _extends({
    type: type,
    className: cls
  }, rest), children);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/Button.jsx", error: String((e && e.message) || e) }); }

// components/EvidenceHash.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · EvidenceHash — operable digest. Digest format law:
   algorithm prefix preserved, middle-ellipsis truncation, never below
   a 13px hit area. Click to copy; optional ⊛ opens the evidence
   inspector. Never inert grey text. Requires styles.css. */
function EvidenceHash({
  digest = '',
  label,
  onInspect,
  className = '',
  ...rest
}) {
  const [copied, setCopied] = React.useState(false);

  /* keep the algorithm prefix, middle-ellipsize the hex */
  const m = digest.match(/^([a-z0-9-]+:)(.+)$/i);
  const prefix = m ? m[1] : '';
  const hex = m ? m[2] : digest;
  const display = prefix + (hex.length > 12 ? hex.slice(0, 4) + '…' + hex.slice(-4) : hex);
  const copy = () => {
    try {
      if (navigator.clipboard) navigator.clipboard.writeText(digest);
    } catch (e) {/* clipboard unavailable — still show feedback */}
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };
  return /*#__PURE__*/React.createElement("span", _extends({
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 0
    }
  }, rest), /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: ('ck-hash ' + className).trim(),
    title: digest + ' · click to copy',
    onClick: copy,
    "aria-label": 'copy digest ' + digest
  }, label ? /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-mute)'
    }
  }, label, " ") : null, copied ? '⊢ copied' : display), onInspect ? /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "ck-hash",
    style: {
      borderLeft: 'none'
    },
    onClick: () => onInspect(digest),
    title: "open in evidence inspector",
    "aria-label": "open in evidence inspector"
  }, "\u229B") : null);
}
Object.assign(__ds_scope, { EvidenceHash });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/EvidenceHash.jsx", error: String((e && e.message) || e) }); }

// components/EvidenceMargin.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · EvidenceMargin — the signature primitive. Evidence is
   never a tab; it is in the margin while the operator works. Renders a
   list of receipts plus the provenance stamp. Receipt color obeys the
   budget: violet rule = proof object exists; red rule = active EP only.
   Requires styles.css. */
function EvidenceMargin({
  title = 'Evidence margin',
  entries = [],
  stamp,
  className = '',
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ('ck-stack-3 ' + className).trim()
  }, rest), /*#__PURE__*/React.createElement("div", {
    className: "ck-label"
  }, title), entries.map((en, i) => {
    const mod = en.kind === 'proof' ? ' ck-receipt--proof' : en.kind === 'alarm' ? ' ck-receipt--alarm' : '';
    return /*#__PURE__*/React.createElement("div", {
      key: i,
      className: 'ck-receipt' + mod
    }, /*#__PURE__*/React.createElement("div", {
      className: "ck-receipt__rule"
    }), /*#__PURE__*/React.createElement("div", {
      className: "ck-stack-1"
    }, /*#__PURE__*/React.createElement("div", {
      className: "ck-receipt__time"
    }, en.time), /*#__PURE__*/React.createElement("div", {
      className: "ck-receipt__event"
    }, en.kind === 'proof' ? '◆ ' : en.kind === 'alarm' ? '⊘ ' : '', en.event), (en.details || []).map((d, j) => /*#__PURE__*/React.createElement("div", {
      key: j,
      className: "ck-receipt__detail"
    }, d))));
  }), children, stamp ? /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    className: "ck-stamp",
    title: stamp
  }, /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true"
  }, "\u25C6"), "sealed \xB7 ", stamp)) : null);
}
Object.assign(__ds_scope, { EvidenceMargin });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/EvidenceMargin.jsx", error: String((e && e.message) || e) }); }

// components/PageShell.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · PageShell — composition is semantic. Renders real
   landmarks (<header> <nav> <main> <aside> <footer>) on the shell grid.
   The Evidence Margin is an <aside>, always rightmost, never a tab.
   variants: console (nav · main · margin) · document (main · margin) ·
   reading (single measured column). Requires styles.css. */
function PageShell({
  variant = 'console',
  brand = 'CKODEX',
  crumb,
  urn,
  headerRight,
  nav,
  margin,
  footerLeft,
  footerRight,
  className = '',
  children,
  ...rest
}) {
  const cls = ('ck-shell' + (variant !== 'console' ? ' ck-shell--' + variant : '') + ' ' + className).trim();
  return /*#__PURE__*/React.createElement("div", _extends({
    className: cls
  }, rest), /*#__PURE__*/React.createElement("header", {
    className: "ck-shell__header"
  }, /*#__PURE__*/React.createElement("div", {
    className: "ck-masthead"
  }, /*#__PURE__*/React.createElement("span", {
    className: "ck-masthead__brand"
  }, brand), crumb ? /*#__PURE__*/React.createElement("span", {
    className: "ck-masthead__crumb"
  }, crumb) : null, urn ? /*#__PURE__*/React.createElement("span", {
    className: "ck-masthead__urn"
  }, urn) : null), headerRight ? /*#__PURE__*/React.createElement("div", {
    className: "ck-row-3"
  }, headerRight) : null), variant === 'console' && nav ? /*#__PURE__*/React.createElement("nav", {
    className: "ck-shell__nav",
    "aria-label": "Primary"
  }, nav) : null, /*#__PURE__*/React.createElement("main", {
    className: "ck-shell__main"
  }, children), variant !== 'reading' && margin ? /*#__PURE__*/React.createElement("aside", {
    className: "ck-shell__margin",
    "aria-label": "Evidence margin"
  }, margin) : null, /*#__PURE__*/React.createElement("footer", {
    className: "ck-shell__footer"
  }, /*#__PURE__*/React.createElement("span", null, footerLeft), /*#__PURE__*/React.createElement("span", null, footerRight)));
}
Object.assign(__ds_scope, { PageShell });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/PageShell.jsx", error: String((e && e.message) || e) }); }

// components/ProvenanceStamp.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · ProvenanceStamp — the ExportGate's signature.
   The artifact leaves with an evidence envelope, or it does not leave.
   Violet because a stamp asserts a proof object exists. Requires styles.css. */
function ProvenanceStamp({
  digest = '',
  verb = 'sealed',
  className = '',
  ...rest
}) {
  const m = digest.match(/^([a-z0-9-]+:)(.+)$/i);
  const prefix = m ? m[1] : '';
  const hex = m ? m[2] : digest;
  const display = prefix + (hex.length > 12 ? hex.slice(0, 4) + '…' + hex.slice(-4) : hex);
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ('ck-stamp ' + className).trim(),
    title: digest
  }, rest), /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true"
  }, "\u25C6"), verb, " \xB7 ", display);
}
Object.assign(__ds_scope, { ProvenanceStamp });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/ProvenanceStamp.jsx", error: String((e && e.message) || e) }); }

// components/QuietCard.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · QuietCard — the default container. Quiet surfaces are
   SQUARE with a hairline; `sealed` swaps to the chamfered contour — the
   only place the cut corner survives. Seeing it must mean an attested /
   sealed state exists. Requires styles.css. */
function QuietCard({
  sealed = false,
  style,
  className = '',
  children,
  ...rest
}) {
  const cls = ((sealed ? 'ck-sealed' : 'ck-quiet') + ' ' + className).trim();
  return /*#__PURE__*/React.createElement("div", _extends({
    className: cls,
    style: style
  }, rest), children);
}
Object.assign(__ds_scope, { QuietCard });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/QuietCard.jsx", error: String((e && e.message) || e) }); }

// components/StateChip.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
/* CKODEX-DS-3 · StateChip — the six epistemic claim states, closed set.
   Glyphs from the CNDL safe-set. Color obeys the budget: only attested
   earns violet, only quarantined earns red. Requires styles.css. */
const CLAIM_GLYPHS = {
  observed: '⊢',
  inferred: '⇝',
  claimed: '○',
  attested: '◆',
  contradicted: '⊭',
  quarantined: '⊘'
};
function StateChip({
  state = 'observed',
  glyph,
  className = '',
  children,
  ...rest
}) {
  const g = glyph !== undefined ? glyph : CLAIM_GLYPHS[state] || '';
  const cls = ('ck-chip ck-chip--' + state + ' ' + className).trim();
  return /*#__PURE__*/React.createElement("span", _extends({
    className: cls
  }, rest), g ? /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true"
  }, g) : null, children !== undefined && children !== null ? children : state);
}
Object.assign(__ds_scope, { StateChip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/StateChip.jsx", error: String((e && e.message) || e) }); }

// dotbg.js
try { (() => {
/* ============================================================
 * CKODEX-DS-3 · DotBg v2 — engineering-canvas background
 * Refined for "Evidence Editorial": paper is FLAT (no vignette);
 * dots and furniture are ink/tone — the semantic budget (rust /
 * violet / red) never appears in a background.
 *   ① Solid base fill        theme ground
 *   ② Dot grid               <pattern> circles, r 1.0, spacing 24
 *   ③ Corner accent clusters 4×4 lattice in TONE, fades inward
 *   ④ Engineering ticks      8pt L-brackets in tone @ theme op
 *   (⑤ vignette — Vault only, very subtle; Ledger and HC are flat)
 *
 * Usage (vanilla):
 *   el.innerHTML = CkDotBg({ w: 320, h: 200, theme: 'ledger' });
 * Usage (React):
 *   <div dangerouslySetInnerHTML={{__html: CkDotBg({w,h,theme})}} />
 *
 * themes: 'ledger' (default) · 'vault' · 'hc'
 * Back-compat aliases: light → ledger, dark → vault.
 * `accent` overrides the furniture color — structural use only;
 * never pass rust/violet/red (the budget is for assertions).
 * forced-colors: wrap in an element that maps fills to ButtonText.
 * ============================================================ */
(function (root) {
  const THEMES = {
    ledger: {
      base: '#F6F1E8',
      dot: '#211B14',
      dotOp: 0.12,
      fur: '#6E6457',
      brkOp: 0.45,
      vig: false,
      vigCol: '#211B14',
      vigOp: 0
    },
    vault: {
      base: '#0A1322',
      dot: '#EAE5DA',
      dotOp: 0.14,
      fur: '#9A9284',
      brkOp: 0.40,
      vig: true,
      vigCol: '#000000',
      vigOp: 0.30
    },
    hc: {
      base: '#000000',
      dot: '#FFFFFF',
      dotOp: 0.60,
      fur: '#FFFFFF',
      brkOp: 1.00,
      vig: false,
      vigCol: '#000000',
      vigOp: 0
    }
  };
  const ALIAS = {
    light: 'ledger',
    dark: 'vault'
  };
  function CkDotBg(opts) {
    opts = opts || {};
    const w = opts.w || 320;
    const h = opts.h || 200;
    const key = ALIAS[opts.theme] || opts.theme;
    const t = THEMES[key] || THEMES.ledger;
    const fur = opts.accent || t.fur; /* furniture color — structural only */
    const S = 24; /* dot spacing (pt) */
    const R = 1.0; /* base dot radius */
    const uid = 'dbg' + Math.random().toString(36).slice(2, 8);

    /* ③ corner clusters — 4×4 lattice anchored at each corner,
          r = 1.7 − dist·0.30 · op = 0.85 − dist·0.20 (dist in grid units) */
    const N = 4;
    const corners = [{
      x: S * 0.5,
      y: S * 0.5,
      dx: 1,
      dy: 1
    }, {
      x: w - S * 0.5,
      y: S * 0.5,
      dx: -1,
      dy: 1
    }, {
      x: S * 0.5,
      y: h - S * 0.5,
      dx: 1,
      dy: -1
    }, {
      x: w - S * 0.5,
      y: h - S * 0.5,
      dx: -1,
      dy: -1
    }];
    let clusters = '';
    corners.forEach(c => {
      for (let i = 0; i < N; i++) {
        for (let j = 0; j < N; j++) {
          const dist = Math.sqrt(i * i + j * j);
          const r = 1.7 - dist * 0.30;
          const op = 0.85 - dist * 0.20;
          if (r <= 0.2 || op <= 0.05) continue;
          const cx = c.x + c.dx * i * S;
          const cy = c.y + c.dy * j * S;
          clusters += `<circle cx="${cx.toFixed(1)}" cy="${cy.toFixed(1)}" r="${r.toFixed(2)}" fill="${fur}" opacity="${(op * (key === 'hc' ? 1 : 0.8)).toFixed(2)}"/>`;
        }
      }
    });

    /* ④ engineering tick marks — 8pt L-brackets inset 14pt */
    const ins = 14,
      arm = 8,
      bw = 1.25;
    const brk = (x, y, sx, sy) => `<path d="M ${x} ${y + sy * arm} V ${y} H ${x + sx * arm}" fill="none" stroke="${fur}" stroke-width="${bw}" opacity="${t.brkOp}"/>`;
    const brackets = brk(ins, ins, 1, 1) + brk(w - ins, ins, -1, 1) + brk(ins, h - ins, 1, -1) + brk(w - ins, h - ins, -1, -1);

    /* ⑤ vignette — Vault only; paper stays flat */
    const vig = t.vig ? `<radialGradient id="${uid}-vig" cx="50%" cy="50%" r="72%">
      <stop offset="55%" stop-color="${t.vigCol}" stop-opacity="0"/>
      <stop offset="100%" stop-color="${t.vigCol}" stop-opacity="${t.vigOp}"/>
    </radialGradient>` : '';
    const vigRect = t.vig ? `<rect width="${w}" height="${h}" fill="url(#${uid}-vig)"/>` : '';
    return `
<svg class="ck-dotbg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="none" aria-hidden="true">
  <defs>
    <pattern id="${uid}-dots" width="${S}" height="${S}" patternUnits="userSpaceOnUse">
      <circle class="ck-dotbg__dot" cx="${S / 2}" cy="${S / 2}" r="${R}" fill="${t.dot}" opacity="${t.dotOp}"/>
    </pattern>
    ${vig}
  </defs>
  <rect width="${w}" height="${h}" fill="${t.base}"/>
  <rect width="${w}" height="${h}" fill="url(#${uid}-dots)"/>
  ${clusters}
  ${vigRect}
  ${brackets}
</svg>`.trim();
  }
  root.CkDotBg = CkDotBg;
})(typeof window !== 'undefined' ? window : this);
})(); } catch (e) { __ds_ns.__errors.push({ path: "dotbg.js", error: String((e && e.message) || e) }); }

// redesign-v2/console-v2-primitives.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
// Console v2 primitives — quiet bentography (B5: contour demoted).
// Loaded as text/babel. Exports to window at the end.
const cv2Chamfer = (c = 10) => `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;

// Quiet card — default container. sealed=true restores the 2.5px contour
// (the ONLY surfaces that wear it: attested/sealed state carriers).
const QuietCard = ({
  sealed = false,
  pad = 24,
  style,
  children,
  ...rest
}) => /*#__PURE__*/React.createElement("div", {
  className: "ck2-elev",
  style: {
    minWidth: 0
  }
}, /*#__PURE__*/React.createElement("div", _extends({}, rest, {
  className: `ck2-card${sealed ? ' ck2-card--sealed' : ''}`,
  style: {
    padding: pad,
    ...style
  }
}), children));

// State chip — F-03 fix. Routine states are ink-on-quiet; `moment` fills
// yellow for the attestation moment and decays back after 4s.
const StateChip = ({
  kind = 'quiet',
  moment = false,
  children
}) => {
  const cls = moment ? 'ck2-chip ck2-chip--attested' : kind === 'deny' ? 'ck2-chip ck2-chip--deny' : kind === 'witness' ? 'ck2-chip ck2-chip--witness' : 'ck2-chip';
  return /*#__PURE__*/React.createElement("span", {
    className: cls,
    style: {
      transition: 'background 280ms, color 280ms, border-color 280ms'
    }
  }, children);
};

// Evidence chip — F-04 fix. Click = copy; the ⊛ affordance opens the inspector.
const HashChip = ({
  digest,
  short,
  onInspect,
  label
}) => {
  const display = short || digest.slice(0, 12) + '…' + digest.slice(-8);
  const copy = e => {
    e.stopPropagation();
    if (navigator.clipboard) navigator.clipboard.writeText(digest).catch(() => {});
    window.cv2Toast && window.cv2Toast(`copied · ${display}`);
  };
  const inspect = e => {
    e.stopPropagation();
    onInspect && onInspect();
  };
  return /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 0
    }
  }, /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "ck2-hash",
    title: `${digest} · click to copy`,
    onClick: copy,
    "aria-label": `copy digest ${digest}`
  }, label ? /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-mute)'
    }
  }, label, " ") : null, display), onInspect ? /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "ck2-hash",
    style: {
      borderLeft: 'none'
    },
    onClick: inspect,
    title: "open in evidence inspector",
    "aria-label": "open in evidence inspector"
  }, "\u229B") : null);
};

// GAL rail — unchanged vocabulary (it was right in v1)
const GalRailV2 = ({
  level = 3,
  width = 96,
  attested = false,
  title
}) => /*#__PURE__*/React.createElement("div", {
  title: title || `GAL ${level} of 6`,
  style: {
    display: 'grid',
    gridTemplateColumns: 'repeat(6, 1fr)',
    gap: 2,
    width,
    height: 7
  }
}, Array.from({
  length: 6
}).map((_, i) => /*#__PURE__*/React.createElement("i", {
  key: i,
  style: {
    background: attested ? 'var(--ck-accent)' : 'var(--ck-rail)',
    opacity: i < level ? 1 : 0.22
  }
})));

// Glyphs — primitives vocabulary at icon scale
const GlyphDiamond = ({
  s = 18,
  nucleus = false,
  color = 'var(--ck-stroke)'
}) => /*#__PURE__*/React.createElement("svg", {
  width: s,
  height: s,
  viewBox: "0 0 20 20",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M10 1.5 L18.5 10 L10 18.5 L1.5 10 Z",
  fill: "none",
  stroke: color,
  strokeWidth: "1.5"
}), nucleus && /*#__PURE__*/React.createElement("path", {
  d: "M10 6.5 L13.5 10 L10 13.5 L6.5 10 Z",
  fill: "var(--ck-accent)"
}));
const GlyphOctagon = ({
  s = 18,
  color = 'var(--ck-stroke)'
}) => /*#__PURE__*/React.createElement("svg", {
  width: s,
  height: s,
  viewBox: "0 0 20 20",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M6.5 1.5 H13.5 L18.5 6.5 V13.5 L13.5 18.5 H6.5 L1.5 13.5 V6.5 Z",
  fill: "none",
  stroke: color,
  strokeWidth: "1.5"
}));
const GlyphFlag = ({
  s = 18,
  color = 'var(--ck-stroke)'
}) => /*#__PURE__*/React.createElement("svg", {
  width: s,
  height: s,
  viewBox: "0 0 20 20",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M10 1.5 L17.5 6 L15.3 17 H4.7 L2.5 6 Z",
  fill: "none",
  stroke: color,
  strokeWidth: "1.5"
}));

// Edge glyph for feeds: ⊢ attested · ⇝ promoting · ⊘ denied · ◐ witnessed
const edgeGlyphMap = {
  attested: {
    g: '⊢',
    c: 'var(--ck-accent)'
  },
  promoting: {
    g: '⇝',
    c: 'var(--ck-stroke)'
  },
  denied: {
    g: '⊘',
    c: 'var(--ck-deny)'
  },
  witnessed: {
    g: '◐',
    c: 'var(--ck-witness)'
  },
  sealed: {
    g: '∎',
    c: 'var(--ck-fg-mute)'
  },
  drafted: {
    g: '◌',
    c: 'var(--ck-fg-mute)'
  },
  reviewed: {
    g: '◐',
    c: 'var(--ck-fg-3)'
  }
};
const EdgeGlyph = ({
  kind
}) => {
  const e = edgeGlyphMap[kind] || edgeGlyphMap.sealed;
  return /*#__PURE__*/React.createElement("span", {
    "aria-hidden": "true",
    style: {
      color: e.c,
      fontFamily: 'var(--ck-ff-mono)',
      fontSize: 13
    }
  }, e.g);
};

// Typed-arrow legend — F-10 fix: the arrow grammar is teachable in place.
const ArrowLegend = ({
  open
}) => !open ? null : /*#__PURE__*/React.createElement("div", {
  className: "cv2-legend",
  role: "dialog",
  "aria-label": "typed arrow legend"
}, /*#__PURE__*/React.createElement("div", {
  className: "lg-row"
}, /*#__PURE__*/React.createElement("svg", {
  width: "56",
  height: "12",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "4",
  x2: "56",
  y2: "4",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "8",
  x2: "56",
  y2: "8",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
})), "kernel-flow"), /*#__PURE__*/React.createElement("div", {
  className: "lg-row"
}, /*#__PURE__*/React.createElement("svg", {
  width: "56",
  height: "12",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: "46",
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "16",
  y1: "1",
  x2: "16",
  y2: "11",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "30",
  y1: "1",
  x2: "30",
  y2: "11",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("polygon", {
  points: "46,1 56,6 46,11",
  fill: "var(--ck-stroke)"
})), "attested"), /*#__PURE__*/React.createElement("div", {
  className: "lg-row"
}, /*#__PURE__*/React.createElement("svg", {
  width: "56",
  height: "12",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: "44",
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("polyline", {
  points: "44,1 54,6 44,11",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25",
  fill: "none"
})), "promotion"), /*#__PURE__*/React.createElement("div", {
  className: "lg-row"
}, /*#__PURE__*/React.createElement("svg", {
  width: "56",
  height: "12",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: "42",
  y2: "6",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "44",
  y1: "1",
  x2: "54",
  y2: "11",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.5"
}), /*#__PURE__*/React.createElement("line", {
  x1: "54",
  y1: "1",
  x2: "44",
  y2: "11",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.5"
})), "deny"), /*#__PURE__*/React.createElement("div", {
  className: "lg-row"
}, /*#__PURE__*/React.createElement("svg", {
  width: "56",
  height: "12",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: "48",
  y2: "6",
  stroke: "var(--ck-accent)",
  strokeWidth: "2"
}), /*#__PURE__*/React.createElement("line", {
  x1: "50",
  y1: "1",
  x2: "50",
  y2: "11",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.5"
}), /*#__PURE__*/React.createElement("line", {
  x1: "53",
  y1: "1",
  x2: "53",
  y2: "11",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.5"
}), /*#__PURE__*/React.createElement("line", {
  x1: "56",
  y1: "1",
  x2: "56",
  y2: "11",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.5"
})), "emergency"));

// Gate diamond at diagram scale, with Resolve beat layers
const GateDiamond = ({
  size = 120,
  resolving = false,
  decided = null
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 120 120",
  className: resolving ? 'cv2-resolving' : ''
}, /*#__PURE__*/React.createElement("polygon", {
  className: "rb1",
  points: "60,8 112,60 60,112 8,60",
  fill: "none",
  stroke: decided === 'deny' ? 'var(--ck-deny)' : 'var(--ck-stroke)',
  strokeWidth: "2.5"
}), /*#__PURE__*/React.createElement("polygon", {
  className: "rb2",
  points: "60,24 96,60 60,96 24,60",
  fill: "none",
  stroke: "var(--ck-witness)",
  strokeWidth: "1",
  opacity: ".8"
}), decided !== 'deny' ? /*#__PURE__*/React.createElement("polygon", {
  className: "rb3",
  points: "60,40 80,60 60,80 40,60",
  fill: decided === 'attested' ? 'var(--ck-accent)' : 'none',
  stroke: "var(--ck-accent)",
  strokeWidth: decided === 'attested' ? 0 : 1
}) : /*#__PURE__*/React.createElement("g", {
  className: "rb3"
}, /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "48",
  x2: "72",
  y2: "72",
  stroke: "var(--ck-deny)",
  strokeWidth: "2"
}), /*#__PURE__*/React.createElement("line", {
  x1: "72",
  y1: "48",
  x2: "48",
  y2: "72",
  stroke: "var(--ck-deny)",
  strokeWidth: "2"
})), /*#__PURE__*/React.createElement("g", {
  className: "rb4"
}, /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "0",
  x2: "60",
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "114",
  y1: "60",
  x2: "120",
  y2: "60",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "114",
  x2: "60",
  y2: "120",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "60",
  x2: "3",
  y2: "60",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
})));

// Proof-chain nodes at diagram fidelity — 96 viewBox, canonical strokes
// (2.5px outer, 1px inner encapsulation), nucleus, cardinal ticks (NW half).
const ProofDiamond = ({
  size = 88,
  witness = false
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 96 96",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("polygon", {
  points: "48,6 90,48 48,90 6,48",
  fill: "none",
  stroke: witness ? 'var(--ck-witness)' : 'var(--ck-stroke)',
  strokeWidth: "2.5"
}), /*#__PURE__*/React.createElement("polygon", {
  points: "48,18 78,48 48,78 18,48",
  fill: "none",
  stroke: witness ? 'var(--ck-witness)' : 'var(--ck-stroke)',
  strokeWidth: "0.75",
  opacity: ".6"
}), /*#__PURE__*/React.createElement("polygon", {
  points: "48,30 66,48 48,66 30,48",
  fill: "var(--ck-accent)"
}), /*#__PURE__*/React.createElement("circle", {
  cx: "48",
  cy: "48",
  r: "3.5",
  fill: "var(--ck-deep-blue)"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "0",
  x2: "48",
  y2: "5",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "91",
  y1: "48",
  x2: "96",
  y2: "48",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "91",
  x2: "48",
  y2: "96",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "48",
  x2: "3",
  y2: "48",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}));
const ProofOctagon = ({
  size = 88
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 96 96",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M20,6 H76 L90,20 V76 L76,90 H20 L6,76 V20 Z",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "2.5"
}), /*#__PURE__*/React.createElement("path", {
  d: "M24,10 H72 L86,24 V72 L72,86 H24 L10,72 V24 Z",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1",
  opacity: ".45"
}), /*#__PURE__*/React.createElement("line", {
  x1: "36",
  y1: "90",
  x2: "36",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "90",
  x2: "48",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "90",
  x2: "60",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}));
// Attested edge — double stroke + hash ticks + teal head. Both proof-chain
// edges are attested edges; promotion chevrons belong to gates, not the chain.
const ChainArrowAttested = ({
  width = 140
}) => /*#__PURE__*/React.createElement("svg", {
  width: width,
  height: "16",
  viewBox: `0 0 ${width} 16`,
  style: {
    flexShrink: 1,
    minWidth: 60
  },
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: width - 12,
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "10",
  x2: width - 12,
  y2: "10",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), [0.25, 0.5, 0.75].map((f, i) => /*#__PURE__*/React.createElement("line", {
  key: i,
  x1: (width - 12) * f,
  y1: "2",
  x2: (width - 12) * f,
  y2: "14",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.25"
})), /*#__PURE__*/React.createElement("polygon", {
  points: `${width - 12},1 ${width},8 ${width - 12},15`,
  fill: "var(--ck-stroke)"
}));
Object.assign(window, {
  cv2Chamfer,
  QuietCard,
  StateChip,
  HashChip,
  GalRailV2,
  GlyphDiamond,
  GlyphOctagon,
  GlyphFlag,
  EdgeGlyph,
  ArrowLegend,
  GateDiamond,
  ProofDiamond,
  ProofOctagon,
  ChainArrowAttested
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "redesign-v2/console-v2-primitives.jsx", error: String((e && e.message) || e) }); }

// redesign-v2/console-v2-shell.jsx
try { (() => {
// Console v2 — shell: sidebar, chrome header, command bar (⌘K),
// evidence inspector drawer, toast, tweaks.
const {
  useState,
  useEffect,
  useRef,
  useCallback
} = React;
const CV2_VIEWS = {
  overview: {
    title: 'Overview',
    sub: 'last resolve · 14:12 utc · 5 kernels · 1 gate open',
    glyph: '◆'
  },
  kernel: {
    title: 'kernel-core',
    sub: 'v 1.4.0 · rev 892f · GAL-6 attested',
    glyph: '⊗'
  },
  attest: {
    title: 'Attestation review',
    sub: 'gate-7712 · promote kernel-policy → production',
    glyph: '◇'
  }
};
const CV2_CMDS = [{
  k: '◆',
  label: 'Go to overview',
  type: 'surface',
  act: go => go('overview')
}, {
  k: '⊗',
  label: 'kernel-core',
  type: 'kernel',
  act: go => go('kernel')
}, {
  k: '◇',
  label: 'gate-7712 · attestation review',
  type: 'gate',
  act: go => go('attest')
}, {
  k: '⊗',
  label: 'kernel-policy',
  type: 'kernel',
  act: go => go('kernel')
}, {
  k: '⊢',
  label: 'Attest + promote gate-7712',
  type: 'action',
  act: go => go('attest')
}, {
  k: '≡',
  label: 'rekor #4812 · kernel-core@1.4.0',
  type: 'anchor',
  act: (go, inspect) => inspect({
    type: 'rekor',
    id: '#4812',
    digest: 'sha256:a42f9c01e0b3d8f2a6c41e9b7d5a3f80c2e6b4a1'
  })
}, {
  k: '⊛',
  label: 'Export AIBOM',
  type: 'action',
  act: () => window.cv2Toast('AIBOM export queued · 423 artifacts')
}];

/* ---------- toast ---------- */
const Toast = () => {
  const [msg, setMsg] = useState(null);
  const tRef = useRef(null);
  useEffect(() => {
    window.cv2Toast = m => {
      setMsg(m);
      clearTimeout(tRef.current);
      tRef.current = setTimeout(() => setMsg(null), 2600);
    };
    return () => {
      clearTimeout(tRef.current);
      delete window.cv2Toast;
    };
  }, []);
  return /*#__PURE__*/React.createElement("div", {
    className: `cv2-toast${msg ? ' show' : ''}`,
    role: "status"
  }, msg && /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u220E"), msg));
};

/* ---------- evidence inspector (L2 / L3) ---------- */
const EvidenceInspector = ({
  item,
  onClose
}) => {
  const [tab, setTab] = useState('summary');
  const closeRef = useRef(null);
  useEffect(() => {
    if (item) {
      setTab('summary');
      closeRef.current && closeRef.current.focus();
    }
  }, [item]);
  useEffect(() => {
    const onKey = e => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [onClose]);
  const digest = item ? item.digest : '';
  const name = item ? item.id || item.type : '';
  const raw = item ? JSON.stringify({
    kind: item.type,
    subject: name,
    digest,
    signatures: [{
      keyid: 'fulcio:1f8a',
      sig: 'MEUCIQDx…'
    }, {
      keyid: 'gal-svc:0b42',
      sig: 'MEYCIQCm…'
    }],
    rekor: {
      logIndex: 4812,
      treeSize: 4812,
      rootHash: 'b8d3f1a9…04e72c5d',
      inclusionProof: ['c1a4…', '9e0d…', '44af…']
    },
    policy: {
      verdict: 'pass',
      compiled: '12 rules',
      mode: 'M3'
    }
  }, null, 2) : '';
  return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("div", {
    className: `cv2-scrim${item ? ' open' : ''}`,
    onClick: onClose
  }), /*#__PURE__*/React.createElement("aside", {
    className: `cv2-drawer${item ? ' open' : ''}`,
    role: "dialog",
    "aria-modal": "true",
    "aria-label": "evidence inspector",
    "aria-hidden": !item
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-drawer-head"
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-stroke)',
      fontFamily: 'var(--ck-ff-mono)'
    }
  }, "\u229B"), /*#__PURE__*/React.createElement("h2", null, "Evidence \xB7 ", name), /*#__PURE__*/React.createElement("button", {
    className: "x",
    ref: closeRef,
    onClick: onClose,
    "aria-label": "close inspector"
  }, "\u2715")), /*#__PURE__*/React.createElement("div", {
    className: "cv2-drawer-body"
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-tabs",
    role: "tablist"
  }, /*#__PURE__*/React.createElement("button", {
    role: "tab",
    "aria-selected": tab === 'summary',
    className: tab === 'summary' ? 'on' : '',
    onClick: () => setTab('summary')
  }, "summary"), /*#__PURE__*/React.createElement("button", {
    role: "tab",
    "aria-selected": tab === 'raw',
    className: tab === 'raw' ? 'on' : '',
    onClick: () => setTab('raw')
  }, "raw record")), tab === 'summary' ? /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("dl", {
    className: "cv2-kv"
  }, /*#__PURE__*/React.createElement("dt", null, "digest"), /*#__PURE__*/React.createElement("dd", null, digest), /*#__PURE__*/React.createElement("dt", null, "signatures"), /*#__PURE__*/React.createElement("dd", null, "fulcio:1f8a \u22A2", /*#__PURE__*/React.createElement("br", null), "gal-svc:0b42 \u22A2"), /*#__PURE__*/React.createElement("dt", null, "witnesses"), /*#__PURE__*/React.createElement("dd", null, "4 signed \xB7 threshold 3"), /*#__PURE__*/React.createElement("dt", null, "rekor index"), /*#__PURE__*/React.createElement("dd", null, "#4812 \xB7 tree 4,812"), /*#__PURE__*/React.createElement("dt", null, "inclusion"), /*#__PURE__*/React.createElement("dd", {
    style: {
      color: 'var(--ck-text-role)'
    }
  }, "\u22A8 proof verified \xB7 14:12 utc"), /*#__PURE__*/React.createElement("dt", null, "policy"), /*#__PURE__*/React.createElement("dd", null, "12 rules compiled \xB7 pass \xB7 M3")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 10,
      flexWrap: 'wrap'
    }
  }, /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => {
      navigator.clipboard && navigator.clipboard.writeText(digest).catch(() => {});
      window.cv2Toast('copied · full digest');
    }
  }, "\u229B copy digest"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => setTab('raw')
  }, "\u2261 raw record"))) : /*#__PURE__*/React.createElement("pre", {
    className: "cv2-raw"
  }, raw))));
};

/* ---------- command bar ---------- */
const CommandBar = ({
  open,
  onClose,
  go,
  inspect
}) => {
  const [q, setQ] = useState('');
  const [sel, setSel] = useState(0);
  const inputRef = useRef(null);
  const list = CV2_CMDS.filter(c => c.label.toLowerCase().includes(q.toLowerCase()));
  useEffect(() => {
    if (open) {
      setQ('');
      setSel(0);
      setTimeout(() => inputRef.current && inputRef.current.focus(), 60);
    }
  }, [open]);
  const onKey = e => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSel(s => Math.min(s + 1, list.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSel(s => Math.max(s - 1, 0));
    } else if (e.key === 'Enter' && list[sel]) {
      list[sel].act(go, inspect);
      onClose();
    }
  };
  return /*#__PURE__*/React.createElement("div", {
    className: `cv2-cmdk${open ? ' open' : ''}`,
    onClick: onClose,
    "aria-hidden": !open
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-cmdk-box",
    role: "dialog",
    "aria-label": "command bar",
    onClick: e => e.stopPropagation()
  }, /*#__PURE__*/React.createElement("input", {
    ref: inputRef,
    value: q,
    onChange: e => {
      setQ(e.target.value);
      setSel(0);
    },
    onKeyDown: onKey,
    placeholder: "jump to kernel, gate, anchor \xB7 run action \xB7 paste a digest prefix",
    "aria-label": "command query"
  }), /*#__PURE__*/React.createElement("div", {
    className: "cv2-cmdk-list"
  }, list.map((c, i) => /*#__PURE__*/React.createElement("button", {
    key: c.label,
    className: `cv2-cmdk-item${i === sel ? ' sel' : ''}`,
    onMouseEnter: () => setSel(i),
    onClick: () => {
      c.act(go, inspect);
      onClose();
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "k"
  }, c.k), c.label, /*#__PURE__*/React.createElement("span", {
    className: "tp"
  }, c.type))), list.length === 0 && /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '16px 14px',
      font: '12.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)'
    }
  }, "no match \xB7 \u22AD"))));
};

/* ---------- app ---------- */
const CV2_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "dark",
  "density": "comfortable",
  "v1contours": false
} /*EDITMODE-END*/;
const ConsoleV2App = () => {
  const [view, setView] = useState('overview');
  const [inspecting, setInspecting] = useState(null);
  const [cmdOpen, setCmdOpen] = useState(false);
  const [t, setTweak] = useTweaks(CV2_TWEAK_DEFAULTS);
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', t.theme);
    document.documentElement.style.setProperty('--cv2-row', t.density === 'compact' ? '48px' : '64px');
    document.body.classList.toggle('cv2-v1mode', !!t.v1contours);
  }, [t]);
  useEffect(() => {
    const onKey = e => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCmdOpen(o => !o);
      }
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, []);
  const go = useCallback(v => {
    setView(v in CV2_VIEWS ? v : 'overview');
  }, []);
  const inspect = useCallback(item => setInspecting(item), []);
  const meta = CV2_VIEWS[view];
  return /*#__PURE__*/React.createElement("div", {
    className: "cv2-app",
    "data-screen-label": `console v2 · ${view}`
  }, /*#__PURE__*/React.createElement("nav", {
    className: "cv2-side",
    "aria-label": "surfaces"
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: "../assets/logos/mark-a2-favicon.svg",
    alt: ""
  }), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement("b", null, "CKODEX"), /*#__PURE__*/React.createElement("small", null, "DS-2 v2.0.0-rc1"))), /*#__PURE__*/React.createElement("p", {
    className: "cv2-navlbl"
  }, "surface"), /*#__PURE__*/React.createElement("div", {
    className: "cv2-nav"
  }, /*#__PURE__*/React.createElement("button", {
    className: view === 'overview' ? 'on' : '',
    onClick: () => go('overview')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u25C6"), "Overview"), /*#__PURE__*/React.createElement("button", {
    className: view === 'kernel' ? 'on' : '',
    onClick: () => go('kernel')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u2297"), "Kernels"), /*#__PURE__*/React.createElement("button", {
    className: view === 'attest' ? 'on' : '',
    onClick: () => go('attest')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u25C7"), "Attestation"), /*#__PURE__*/React.createElement("button", {
    onClick: () => window.cv2Toast('policy surface · follows the collection template')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u22A2"), "Policy"), /*#__PURE__*/React.createElement("button", {
    onClick: () => window.cv2Toast('artifacts surface · follows the collection template')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u229B"), "Artifacts"), /*#__PURE__*/React.createElement("button", {
    onClick: () => window.cv2Toast('rekor surface · follows the collection template')
  }, /*#__PURE__*/React.createElement("span", {
    className: "gl"
  }, "\u2261"), "Rekor log")), /*#__PURE__*/React.createElement("div", {
    className: "cv2-side-foot"
  }, "density ladder", /*#__PURE__*/React.createElement("br", null), "L0 glance \u2192 L3 verify")), /*#__PURE__*/React.createElement("div", {
    className: "cv2-main"
  }, /*#__PURE__*/React.createElement("header", {
    className: "cv2-head"
  }, /*#__PURE__*/React.createElement("h1", null, meta.title), /*#__PURE__*/React.createElement("span", {
    className: "sub"
  }, meta.sub), /*#__PURE__*/React.createElement("div", {
    className: "cv2-head-right"
  }, /*#__PURE__*/React.createElement("button", {
    className: "cv2-kbtn",
    onClick: () => setCmdOpen(true)
  }, "search & act ", /*#__PURE__*/React.createElement("kbd", null, "\u2318K")), /*#__PURE__*/React.createElement("div", {
    className: "cv2-mode",
    tabIndex: 0,
    "aria-label": "governance mode M3 promotion"
  }, /*#__PURE__*/React.createElement("svg", {
    width: "16",
    height: "16",
    viewBox: "0 0 20 20",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M10 1.5 L17.5 6 L15.3 17 H4.7 L2.5 6 Z",
    fill: "none",
    stroke: "var(--ck-witness)",
    strokeWidth: "1.5"
  })), "M3 \xB7 promotion", /*#__PURE__*/React.createElement("div", {
    className: "pop"
  }, /*#__PURE__*/React.createElement("span", {
    className: "ck2-invariant"
  }, "mode changes deployment, not governance semantics"), /*#__PURE__*/React.createElement("p", null, "M3 since 14:02 utc. Attested kernels are production-eligible. Gate thresholds are unchanged."))))), /*#__PURE__*/React.createElement("main", {
    className: "cv2-body"
  }, view === 'overview' && /*#__PURE__*/React.createElement(OverviewV2, {
    go: go,
    inspect: inspect
  }), view === 'kernel' && /*#__PURE__*/React.createElement(KernelDetailV2, {
    inspect: inspect
  }), view === 'attest' && /*#__PURE__*/React.createElement(AttestReviewV2, {
    inspect: inspect
  }))), /*#__PURE__*/React.createElement(EvidenceInspector, {
    item: inspecting,
    onClose: () => setInspecting(null)
  }), /*#__PURE__*/React.createElement(CommandBar, {
    open: cmdOpen,
    onClose: () => setCmdOpen(false),
    go: go,
    inspect: inspect
  }), /*#__PURE__*/React.createElement(Toast, null), /*#__PURE__*/React.createElement(TweaksPanel, null, /*#__PURE__*/React.createElement(TweakSection, {
    label: "Theme"
  }), /*#__PURE__*/React.createElement(TweakRadio, {
    label: "Palette",
    value: t.theme,
    options: ['dark', 'light', 'hc'],
    onChange: v => setTweak('theme', v)
  }), /*#__PURE__*/React.createElement(TweakSection, {
    label: "Density"
  }), /*#__PURE__*/React.createElement(TweakRadio, {
    label: "Rows",
    value: t.density,
    options: ['comfortable', 'compact'],
    onChange: v => setTweak('density', v)
  }), /*#__PURE__*/React.createElement(TweakSection, {
    label: "Compare"
  }), /*#__PURE__*/React.createElement(TweakToggle, {
    label: "v1 contours (pre-B5)",
    value: t.v1contours,
    onChange: v => setTweak('v1contours', v)
  })));
};
ReactDOM.createRoot(document.getElementById('root')).render(/*#__PURE__*/React.createElement(ConsoleV2App, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "redesign-v2/console-v2-shell.jsx", error: String((e && e.message) || e) }); }

// redesign-v2/console-v2-views.jsx
try { (() => {
// Console v2 — views: Overview, Kernel detail, Attestation review.
const CV2_DATA = {
  kernels: [{
    id: 'kernel-core',
    gal: 6,
    state: 'attested',
    digest: 'sha256:a42f9c01e0b3d8f2a6c41e9b7d5a3f80c2e6b4a1',
    age: '2h',
    ver: '1.4.0',
    rev: '892f'
  }, {
    id: 'kernel-attest',
    gal: 5,
    state: 'attested',
    digest: 'sha256:c91d4b2a7e6f0a3c5d8b1e4f9a2c6d309e7b5a4c',
    age: '5h',
    ver: '1.3.9',
    rev: '7c1a'
  }, {
    id: 'kernel-policy',
    gal: 4,
    state: 'promoting',
    digest: 'sha256:77e09f3c2b6a1d4e8f0c3a5b9d2e6f407c1a8b3d',
    age: '21m',
    ver: '1.4.0',
    rev: '3b9e'
  }, {
    id: 'kernel-rekor',
    gal: 3,
    state: 'reviewed',
    digest: 'sha256:12ab8e5d3c7f0b4a6e9d2c5f8a1b4e708d3c6f9a',
    age: '1d',
    ver: '1.2.4',
    rev: 'e44d'
  }, {
    id: 'kernel-sandbox',
    gal: 2,
    state: 'drafted',
    digest: 'sha256:3f09d72b5e8a1c4f7b0d3e6a9c2f5b801e4d7a0c',
    age: '3d',
    ver: '0.9.1',
    rev: '01f2'
  }],
  edges: [{
    kind: 'attested',
    subj: 'kernel-core',
    t: '14:12',
    rel: '2m'
  }, {
    kind: 'promoting',
    subj: 'kernel-policy',
    t: '14:08',
    rel: '6m'
  }, {
    kind: 'denied',
    subj: 'kernel-sandbox',
    t: '14:01',
    rel: '13m'
  }, {
    kind: 'witnessed',
    subj: 'kernel-attest',
    t: '13:54',
    rel: '20m'
  }],
  attEdges: [{
    from: 'source',
    to: 'builder',
    kind: 'attested',
    actor: 'build-7',
    t: '13:58'
  }, {
    from: 'builder',
    to: 'signer',
    kind: 'attested',
    actor: 'fulcio',
    t: '14:02'
  }, {
    from: 'signer',
    to: 'rekor',
    kind: 'promotion',
    actor: 'gal-svc',
    t: '14:09'
  }, {
    from: 'rekor',
    to: 'anchor',
    kind: 'kernel-flow',
    actor: '—',
    t: '14:12'
  }],
  reviewers: [{
    id: 'ewald.k',
    state: 'attested',
    t: '13:41'
  }, {
    id: 'morita.s',
    state: 'attested',
    t: '13:52'
  }, {
    id: 'chen.j',
    state: 'pending',
    t: null
  }]
};
const cv2StateChipKind = s => s === 'denied' ? 'deny' : s === 'witnessed' ? 'witness' : 'quiet';

/* ---------------- Overview ---------------- */
const OverviewV2 = ({
  go,
  inspect
}) => {
  const [legend, setLegend] = React.useState(false);
  return /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid"
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-stats"
  }, /*#__PURE__*/React.createElement(QuietCard, null, /*#__PURE__*/React.createElement("div", {
    className: "cv2-stat",
    style: {
      padding: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "n"
  }, "5", /*#__PURE__*/React.createElement("small", null, "/5")), /*#__PURE__*/React.createElement("span", {
    className: "l"
  }, "kernels attested"))), /*#__PURE__*/React.createElement(QuietCard, null, /*#__PURE__*/React.createElement("div", {
    className: "cv2-stat",
    style: {
      padding: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "n"
  }, "1"), /*#__PURE__*/React.createElement("span", {
    className: "l"
  }, "gate open \xB7 gate-7712"))), /*#__PURE__*/React.createElement(QuietCard, null, /*#__PURE__*/React.createElement("div", {
    className: "cv2-stat",
    style: {
      padding: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "n"
  }, "4,812"), /*#__PURE__*/React.createElement("span", {
    className: "l"
  }, "anchor entries \xB7 depth 8"))), /*#__PURE__*/React.createElement(QuietCard, null, /*#__PURE__*/React.createElement("div", {
    className: "cv2-stat",
    style: {
      padding: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "n"
  }, "14:12"), /*#__PURE__*/React.createElement("span", {
    className: "l"
  }, "last resolve \xB7 2m ago")))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '2fr 1fr',
      gap: 20,
      alignItems: 'start'
    }
  }, /*#__PURE__*/React.createElement(QuietCard, {
    pad: 28
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'baseline',
      gap: 14,
      marginBottom: 20
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      font: '600 17px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      margin: 0
    }
  }, "Proof chain"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '11.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)'
    }
  }, "PCA \u2192 UCA \u2192 REKOR"), /*#__PURE__*/React.createElement("div", {
    className: "cv2-legend-wrap",
    style: {
      marginLeft: 'auto'
    }
  }, /*#__PURE__*/React.createElement("button", {
    className: "cv2-kbtn",
    onClick: () => setLegend(!legend),
    "aria-expanded": legend
  }, "\u22B2\u22B3 edge legend"), /*#__PURE__*/React.createElement(ArrowLegend, {
    open: legend
  }))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      gap: 16,
      padding: '12px 4px 4px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(ProofDiamond, {
    size: 88
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '600 12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)',
      marginTop: 8,
      letterSpacing: '.1em'
    }
  }, "PCA"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '11px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)',
      marginTop: 2
    }
  }, "provenance")), /*#__PURE__*/React.createElement(ChainArrowAttested, {
    width: 150
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(ProofDiamond, {
    size: 88
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '600 12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)',
      marginTop: 8,
      letterSpacing: '.1em'
    }
  }, "UCA"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '11px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)',
      marginTop: 2
    }
  }, "usage")), /*#__PURE__*/React.createElement(ChainArrowAttested, {
    width: 150
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(ProofOctagon, {
    size: 88
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '600 12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)',
      marginTop: 8,
      letterSpacing: '.1em'
    }
  }, "REKOR"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '11px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)',
      marginTop: 2
    }
  }, "anchor")))), /*#__PURE__*/React.createElement(QuietCard, {
    pad: 24
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      font: '600 14px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      margin: '0 0 6px',
      letterSpacing: '.02em'
    }
  }, "Recent edges"), /*#__PURE__*/React.createElement("div", null, CV2_DATA.edges.map(e => /*#__PURE__*/React.createElement("div", {
    className: "cv2-edge",
    key: e.subj + e.t
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: e.kind
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-1)'
    }
  }, e.kind), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-3)'
    }
  }, e.subj), /*#__PURE__*/React.createElement("span", {
    className: "t"
  }, e.rel, " ago")))))), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl"
  }, "kernels \xB7 5"), /*#__PURE__*/React.createElement(QuietCard, {
    pad: 0
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-rows"
  }, CV2_DATA.kernels.map(k => /*#__PURE__*/React.createElement("div", {
    className: "cv2-row",
    key: k.id,
    role: "button",
    tabIndex: 0,
    "aria-label": `open ${k.id}`,
    onClick: () => go('kernel', k.id),
    onKeyDown: e => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        go('kernel', k.id);
      }
    }
  }, /*#__PURE__*/React.createElement("span", {
    className: "nm",
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }
  }, /*#__PURE__*/React.createElement(GlyphOctagon, {
    s: 15
  }), k.id), /*#__PURE__*/React.createElement(GalRailV2, {
    level: k.gal,
    width: 100,
    title: `GAL ${k.gal} of 6`
  }), /*#__PURE__*/React.createElement("span", null, /*#__PURE__*/React.createElement(StateChip, {
    kind: cv2StateChipKind(k.state)
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: k.state
  }), k.state)), /*#__PURE__*/React.createElement("span", {
    className: "sig",
    onClick: e => e.stopPropagation()
  }, /*#__PURE__*/React.createElement(HashChip, {
    digest: k.digest,
    onInspect: () => inspect({
      type: 'kernel',
      ...k
    })
  })), /*#__PURE__*/React.createElement("span", {
    className: "age"
  }, k.age, " ago")))))));
};

/* ---------------- Kernel detail ---------------- */
const KernelDetailV2 = ({
  inspect
}) => {
  const k = CV2_DATA.kernels[0];
  return /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid",
    style: {
      gridTemplateColumns: '1fr 320px',
      alignItems: 'start'
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid"
  }, /*#__PURE__*/React.createElement(QuietCard, {
    sealed: true,
    pad: 28
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 24,
      alignItems: 'center',
      flexWrap: 'wrap'
    }
  }, /*#__PURE__*/React.createElement("svg", {
    width: "84",
    height: "84",
    viewBox: "0 0 96 96",
    "aria-hidden": "true"
  }, /*#__PURE__*/React.createElement("path", {
    d: "M20,6 H76 L90,20 V76 L76,90 H20 L6,76 V20 Z",
    fill: "none",
    stroke: "var(--ck-stroke)",
    strokeWidth: "2.5"
  }), /*#__PURE__*/React.createElement("path", {
    d: "M24,10 H72 L86,24 V72 L72,86 H24 L10,72 V24 Z",
    fill: "none",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1",
    opacity: ".45"
  }), /*#__PURE__*/React.createElement("polygon", {
    points: "48,32 64,48 48,64 32,48",
    fill: "var(--ck-accent)"
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1,
      minWidth: 220
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: '600 24px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      letterSpacing: '-0.014em'
    }
  }, "kernel-core"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)',
      margin: '6px 0 12px'
    }
  }, "v ", k.ver, " \xB7 rev ", k.rev, " \xB7 sealed 2h ago"), /*#__PURE__*/React.createElement(HashChip, {
    digest: k.digest,
    onInspect: () => inspect({
      type: 'kernel',
      ...k
    })
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'right'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: '600 56px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      lineHeight: 1,
      letterSpacing: '-0.02em'
    }
  }, "6", /*#__PURE__*/React.createElement("span", {
    style: {
      fontSize: 22,
      color: 'var(--ck-fg-3)',
      fontWeight: 500
    }
  }, "/6")), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '11px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)',
      letterSpacing: '.1em',
      textTransform: 'uppercase'
    }
  }, "governance assurance"), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 10,
      display: 'flex',
      justifyContent: 'flex-end'
    }
  }, /*#__PURE__*/React.createElement(GalRailV2, {
    level: 6,
    width: 120,
    attested: true
  }))))), /*#__PURE__*/React.createElement(QuietCard, {
    pad: 28
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      font: '600 17px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      margin: '0 0 4px'
    }
  }, "Attestation edges"), /*#__PURE__*/React.createElement("p", {
    style: {
      font: '13px var(--ck-ff-body)',
      color: 'var(--ck-fg-3)',
      margin: '0 0 18px'
    }
  }, "Six attested edges, four witness signatures. Promotion is reversible only through an emergency edge."), /*#__PURE__*/React.createElement("div", null, CV2_DATA.attEdges.map((e, i) => /*#__PURE__*/React.createElement("div", {
    key: i,
    style: {
      display: 'grid',
      gridTemplateColumns: '110px 1fr 110px 90px 60px',
      gap: 16,
      alignItems: 'center',
      padding: '13px 0',
      borderBottom: i < CV2_DATA.attEdges.length - 1 ? '1px solid var(--ck2-hairline)' : 'none'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)',
      textAlign: 'right'
    }
  }, e.from), /*#__PURE__*/React.createElement("svg", {
    height: "16",
    width: "100%",
    preserveAspectRatio: "none",
    viewBox: "0 0 100 16",
    "aria-label": `${e.kind} edge`
  }, e.kind === 'kernel-flow' && /*#__PURE__*/React.createElement("g", null, /*#__PURE__*/React.createElement("line", {
    x1: "0",
    y1: "6",
    x2: "100",
    y2: "6",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1"
  }), /*#__PURE__*/React.createElement("line", {
    x1: "0",
    y1: "10",
    x2: "100",
    y2: "10",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1"
  })), e.kind === 'attested' && /*#__PURE__*/React.createElement("g", null, /*#__PURE__*/React.createElement("line", {
    x1: "0",
    y1: "8",
    x2: "88",
    y2: "8",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1"
  }), /*#__PURE__*/React.createElement("line", {
    x1: "25",
    y1: "2",
    x2: "25",
    y2: "14",
    stroke: "var(--ck-accent)",
    strokeWidth: "1.25"
  }), /*#__PURE__*/React.createElement("line", {
    x1: "50",
    y1: "2",
    x2: "50",
    y2: "14",
    stroke: "var(--ck-accent)",
    strokeWidth: "1.25"
  }), /*#__PURE__*/React.createElement("line", {
    x1: "75",
    y1: "2",
    x2: "75",
    y2: "14",
    stroke: "var(--ck-accent)",
    strokeWidth: "1.25"
  }), /*#__PURE__*/React.createElement("polygon", {
    points: "88,2 100,8 88,14",
    fill: "var(--ck-stroke)"
  })), e.kind === 'promotion' && /*#__PURE__*/React.createElement("g", null, /*#__PURE__*/React.createElement("line", {
    x1: "0",
    y1: "8",
    x2: "88",
    y2: "8",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1.25"
  }), /*#__PURE__*/React.createElement("polyline", {
    points: "84,2 96,8 84,14",
    stroke: "var(--ck-stroke)",
    strokeWidth: "1.25",
    fill: "none"
  }))), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)'
    }
  }, e.to), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '11.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)'
    }
  }, e.actor), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '11px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)',
      textAlign: 'right'
    }
  }, e.t)))))), /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid",
    style: {
      position: 'sticky',
      top: 84
    }
  }, /*#__PURE__*/React.createElement(QuietCard, {
    pad: 24
  }, /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl",
    style: {
      marginTop: 0
    }
  }, "actions"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => window.cv2Toast && window.cv2Toast('witness requested · 2 of 5 notified')
  }, "\u25D0 request witness"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => inspect({
      type: 'kernel',
      ...k
    })
  }, "\u229B open evidence"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--deny",
    onClick: () => window.cv2Toast && window.cv2Toast('revocation requires an emergency edge')
  }, "\u2298 revoke seal"))), /*#__PURE__*/React.createElement(QuietCard, {
    pad: 24
  }, /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl",
    style: {
      marginTop: 0
    }
  }, "witnesses \xB7 4"), ['fulcio-ca', 'gal-svc', 'ewald.k', 'morita.s'].map(w => /*#__PURE__*/React.createElement("div", {
    className: "cv2-edge",
    key: w
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: "witnessed"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-1)'
    }
  }, w), /*#__PURE__*/React.createElement("span", {
    className: "t"
  }, "signed"))))));
};

/* ---------------- Attestation review ---------------- */
const AttestReviewV2 = ({
  inspect
}) => {
  const [decision, setDecision] = React.useState(null); // null | attested | deny
  const [resolving, setResolving] = React.useState(false);
  const [confirmDeny, setConfirmDeny] = React.useState(false);
  const [denyText, setDenyText] = React.useState('');
  const [chipMoment, setChipMoment] = React.useState(false);
  const attest = () => {
    setResolving(true);
    setDecision('attested');
    setChipMoment(true);
    window.cv2Toast && window.cv2Toast('⊢ gate-7712 attested · anchored at #4813');
    setTimeout(() => setResolving(false), 1900); // Resolve completes; end-state is plain CSS
    setTimeout(() => setChipMoment(false), 4000); // the moment decays — F-03
  };
  const deny = () => {
    if (denyText.trim() !== 'gate-7712') return;
    setResolving(true);
    setDecision('deny');
    setConfirmDeny(false);
    setTimeout(() => setResolving(false), 1900);
    window.cv2Toast && window.cv2Toast('⊘ gate-7712 denied · recorded at #4813');
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid",
    style: {
      gridTemplateColumns: '1fr 300px',
      alignItems: 'start'
    }
  }, /*#__PURE__*/React.createElement(QuietCard, {
    sealed: decision === 'attested',
    pad: 32
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'baseline',
      gap: 14,
      marginBottom: 8
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      font: '600 19px var(--ck-ff-display)',
      color: 'var(--ck-fg-1)',
      margin: 0
    }
  }, "Gate \xB7 promote kernel-policy"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '11.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-mute)',
      marginLeft: 'auto'
    }
  }, "GATE-7712 \xB7 M3")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-around',
      padding: '20px 0 8px',
      flexWrap: 'wrap',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(GlyphOctagon, {
    s: 72
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-2)',
      marginTop: 8
    }
  }, "kernel-policy")), /*#__PURE__*/React.createElement(GateDiamond, {
    size: 128,
    resolving: resolving,
    decided: decision
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center',
      opacity: decision === 'attested' ? 1 : .55
    }
  }, /*#__PURE__*/React.createElement(GlyphOctagon, {
    s: 72,
    color: decision === 'attested' ? 'var(--ck-accent)' : 'var(--ck-stroke)'
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-2)',
      marginTop: 8
    }
  }, "production"))), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center',
      marginBottom: 24
    }
  }, /*#__PURE__*/React.createElement(StateChip, {
    kind: decision === 'deny' ? 'deny' : 'quiet',
    moment: chipMoment
  }, decision === null ? /*#__PURE__*/React.createElement(React.Fragment, null, "\u25CB pending") : decision === 'attested' ? /*#__PURE__*/React.createElement(React.Fragment, null, "\u22A2 attested") : /*#__PURE__*/React.createElement(React.Fragment, null, "\u2298 denied"))), /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl"
  }, "evidence"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 10,
      marginBottom: 28
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12,
      flexWrap: 'wrap'
    }
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: "attested"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '13px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)'
    }
  }, "provenance"), /*#__PURE__*/React.createElement(HashChip, {
    digest: CV2_DATA.kernels[2].digest,
    onInspect: () => inspect({
      type: 'gate-evidence',
      id: 'provenance',
      digest: CV2_DATA.kernels[2].digest
    })
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: "attested"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '13px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)'
    }
  }, "usage envelope"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)'
    }
  }, "policy-safe \u2200 inputs")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: "witnessed"
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '13px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)'
    }
  }, "witness"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)'
    }
  }, "2 of 3 required \xB7 chen.j pending")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: decision ? 'attested' : 'drafted'
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '13px var(--ck-ff-mono)',
      color: 'var(--ck-fg-1)'
    }
  }, "rekor anchor"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12px var(--ck-ff-mono)',
      color: decision ? 'var(--ck-text-role)' : 'var(--ck-fg-3)'
    }
  }, decision ? '#4813 · anchored' : 'pending decision'))), decision === null && !confirmDeny && /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 12,
      flexWrap: 'wrap',
      borderTop: '1px solid var(--ck2-hairline)',
      paddingTop: 20
    }
  }, /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--primary",
    onClick: attest
  }, "\u22A2 attest + promote"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => window.cv2Toast && window.cv2Toast('◐ witness requested from chen.j')
  }, "\u25D0 request witness"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--deny",
    style: {
      marginLeft: 'auto'
    },
    onClick: () => setConfirmDeny(true)
  }, "\u2298 deny")), confirmDeny && /*#__PURE__*/React.createElement("div", {
    style: {
      borderTop: '1px solid var(--ck2-hairline)',
      paddingTop: 20,
      display: 'flex',
      gap: 12,
      alignItems: 'center',
      flexWrap: 'wrap'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      font: '12.5px var(--ck-ff-mono)',
      color: 'var(--ck-deny)'
    }
  }, "deny is gate-final \xB7 type ", /*#__PURE__*/React.createElement("b", null, "gate-7712"), " to confirm"), /*#__PURE__*/React.createElement("input", {
    value: denyText,
    onChange: e => setDenyText(e.target.value),
    placeholder: "gate-7712",
    "aria-label": "type gate id to confirm deny",
    style: {
      font: '12.5px var(--ck-ff-mono)',
      background: 'var(--ck-bg-0)',
      border: '1px solid var(--ck2-hairline-strong)',
      color: 'var(--ck-fg-1)',
      padding: '8px 12px',
      width: 130
    }
  }), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--deny",
    disabled: denyText.trim() !== 'gate-7712',
    style: {
      opacity: denyText.trim() === 'gate-7712' ? 1 : .45
    },
    onClick: deny
  }, "\u2298 confirm deny"), /*#__PURE__*/React.createElement("button", {
    className: "cv2-btn cv2-btn--quiet",
    onClick: () => {
      setConfirmDeny(false);
      setDenyText('');
    }
  }, "cancel")), decision !== null && /*#__PURE__*/React.createElement("div", {
    style: {
      borderTop: '1px solid var(--ck2-hairline)',
      paddingTop: 20,
      font: '12.5px var(--ck-ff-mono)',
      color: 'var(--ck-fg-3)'
    }
  }, "decision recorded \xB7 ", /*#__PURE__*/React.createElement("button", {
    className: "ck2-hash",
    onClick: () => inspect({
      type: 'rekor',
      id: '#4813',
      digest: 'sha256:b8d3f1a9c2e64b7a8f0d3c5e9a1b4f6d04e72c5d'
    })
  }, "view anchor #4813 \u229B"))), /*#__PURE__*/React.createElement("div", {
    className: "cv2-grid",
    style: {
      position: 'sticky',
      top: 84
    }
  }, /*#__PURE__*/React.createElement(QuietCard, {
    pad: 24
  }, /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl",
    style: {
      marginTop: 0
    }
  }, "reviewers \xB7 2 of 3"), CV2_DATA.reviewers.map(r => /*#__PURE__*/React.createElement("div", {
    className: "cv2-edge",
    key: r.id
  }, /*#__PURE__*/React.createElement(EdgeGlyph, {
    kind: r.state === 'attested' ? 'attested' : 'drafted'
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-1)'
    }
  }, r.id), /*#__PURE__*/React.createElement("span", {
    className: "t"
  }, r.state === 'attested' ? `⊢ ${r.t}` : 'pending')))), /*#__PURE__*/React.createElement(QuietCard, {
    pad: 24
  }, /*#__PURE__*/React.createElement("p", {
    className: "cv2-sec-lbl",
    style: {
      marginTop: 0
    }
  }, "invariant"), /*#__PURE__*/React.createElement("span", {
    className: "ck2-invariant"
  }, "mode changes deployment, not governance semantics"), /*#__PURE__*/React.createElement("p", {
    style: {
      font: '12.5px var(--ck-ff-body)',
      color: 'var(--ck-fg-3)',
      margin: '10px 0 0',
      lineHeight: 1.55
    }
  }, "Attesting in M3 changes where kernel-policy runs. It does not change what the gate enforces."))));
};
Object.assign(window, {
  CV2_DATA,
  OverviewV2,
  KernelDetailV2,
  AttestReviewV2
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "redesign-v2/console-v2-views.jsx", error: String((e && e.message) || e) }); }

// redesign-v2/hero-scenes.js
try { (() => {
// CKODEX hero scenes — Canvas-2D stand-ins for the three specified
// Three.js scenes (see Design System v2 Spec §5). Same compositions,
// same palette, same motion law: translate + opacity only. No rotation,
// no pulsing, no glow. Static single frame under prefers-reduced-motion.
(function () {
  'use strict';

  const REDUCED = window.matchMedia('(prefers-reduced-motion: reduce)');
  function readPalette() {
    const cs = getComputedStyle(document.documentElement);
    const v = (n, fb) => (cs.getPropertyValue(n) || fb).trim() || fb;
    return {
      stroke: v('--ck-stroke', '#38a3a5'),
      accent: v('--ck-accent', '#fcca46'),
      witness: v('--ck-witness', '#bc96e6'),
      fgMute: v('--ck-fg-mute', '#4A5B75'),
      bg: v('--ck-bg-0', '#00152b')
    };
  }

  // deterministic PRNG so the composition is stable across reloads
  function mulberry32(a) {
    return function () {
      a |= 0;
      a = a + 0x6D2B79F5 | 0;
      let t = Math.imul(a ^ a >>> 15, 1 | a);
      t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t;
      return ((t ^ t >>> 14) >>> 0) / 4294967296;
    };
  }

  /* ---------- scene 1 · zero-trust mesh ---------- */
  function meshScene(W, H, pal) {
    const rnd = mulberry32(7712);
    const N = 42;
    const nodes = [];
    for (let i = 0; i < N; i++) {
      nodes.push({
        x: 0.08 + rnd() * 0.84,
        y: 0.08 + rnd() * 0.84,
        dx: (rnd() - 0.5) * 0.012,
        dy: (rnd() - 0.5) * 0.012
      });
    }
    // nearest-neighbour edges, fixed at init
    const edges = [];
    for (let i = 0; i < N; i++) {
      const d = nodes.map((n, j) => ({
        j,
        d: (n.x - nodes[i].x) ** 2 + (n.y - nodes[i].y) ** 2
      })).sort((a, b) => a.d - b.d).slice(1, 4);
      for (const {
        j
      } of d) {
        if (!edges.some(e => e.a === j && e.b === i)) edges.push({
          a: i,
          b: j,
          born: rnd() * 14
        });
      }
    }
    return function draw(ctx, t) {
      for (const n of nodes) {
        n.x += n.dx * 0.016;
        n.y += n.dy * 0.016;
        if (n.x < 0.05 || n.x > 0.95) n.dx *= -1;
        if (n.y < 0.05 || n.y > 0.95) n.dy *= -1;
      }
      const P = n => [n.x * W, n.y * H];
      // edges attest in over time, cycle 14s
      const cyc = t % 14;
      ctx.lineWidth = 1;
      for (const e of edges) {
        const age = cyc - e.born;
        if (age < 0) continue;
        const prog = Math.min(1, age / 0.9); // edge draws in over 0.9s
        const [ax, ay] = P(nodes[e.a]);
        const [bx, by] = P(nodes[e.b]);
        ctx.strokeStyle = pal.stroke;
        ctx.globalAlpha = 0.16 + 0.2 * prog;
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(ax + (bx - ax) * prog, ay + (by - ay) * prog);
        ctx.stroke();
        // hash ticks on fully attested edges
        if (prog >= 1) {
          ctx.globalAlpha = 0.5;
          const mx = (ax + bx) / 2,
            my = (ay + by) / 2;
          const len = Math.hypot(bx - ax, by - ay) || 1;
          const px = -(by - ay) / len,
            py = (bx - ax) / len;
          ctx.beginPath();
          ctx.moveTo(mx - px * 3, my - py * 3);
          ctx.lineTo(mx + px * 3, my + py * 3);
          ctx.stroke();
        }
      }
      // nodes
      for (let i = 0; i < N; i++) {
        const [x, y] = P(nodes[i]);
        ctx.globalAlpha = 0.85;
        ctx.fillStyle = pal.stroke;
        ctx.fillRect(x - 1.5, y - 1.5, 3, 3);
      }
      // one migrating nucleus — the node currently being attested
      const k = Math.floor(cyc / 2) % N;
      const [nx, ny] = P(nodes[k]);
      ctx.globalAlpha = 1;
      ctx.strokeStyle = pal.accent;
      ctx.lineWidth = 1.5;
      ctx.strokeRect(nx - 6, ny - 6, 12, 12);
      ctx.fillStyle = pal.accent;
      ctx.beginPath();
      ctx.moveTo(nx, ny - 3.4);
      ctx.lineTo(nx + 3.4, ny);
      ctx.lineTo(nx, ny + 3.4);
      ctx.lineTo(nx - 3.4, ny);
      ctx.closePath();
      ctx.fill();
      ctx.globalAlpha = 1;
    };
  }

  /* ---------- scene 2 · evidence bundle ---------- */
  function bundleScene(W, H, pal) {
    const cx = W / 2,
      cy = H / 2;
    const R = Math.min(W, H) * 0.30;
    const r = R / 1.793; // the sealed-diamond signature ratio
    const dia = R_ => [[0, -R_], [R_, 0], [0, R_], [-R_, 0]];
    return function draw(ctx, t) {
      const cyc = t % 9;
      // outer diamond strokes draw in, one edge per beat
      const pts = dia(R);
      ctx.lineWidth = 2;
      ctx.strokeStyle = pal.stroke;
      for (let i = 0; i < 4; i++) {
        const prog = Math.max(0, Math.min(1, (cyc - i * 0.45) / 0.45));
        if (prog <= 0) continue;
        const [x1, y1] = pts[i],
          [x2, y2] = pts[(i + 1) % 4];
        ctx.globalAlpha = 0.9;
        ctx.beginPath();
        ctx.moveTo(cx + x1, cy + y1);
        ctx.lineTo(cx + x1 + (x2 - x1) * prog, cy + y1 + (y2 - y1) * prog);
        ctx.stroke();
      }
      // inner diamond (witness) fades in at beat 2
      const wAlpha = Math.max(0, Math.min(1, (cyc - 2.2) / 0.8));
      if (wAlpha > 0) {
        ctx.globalAlpha = 0.8 * wAlpha;
        ctx.strokeStyle = pal.witness;
        ctx.lineWidth = 1;
        const ip = dia(r);
        ctx.beginPath();
        ctx.moveTo(cx + ip[0][0], cy + ip[0][1]);
        for (let i = 1; i <= 4; i++) ctx.lineTo(cx + ip[i % 4][0], cy + ip[i % 4][1]);
        ctx.stroke();
      }
      // nucleus at beat 3
      const nAlpha = Math.max(0, Math.min(1, (cyc - 3.4) / 0.6));
      if (nAlpha > 0) {
        ctx.globalAlpha = nAlpha;
        ctx.fillStyle = pal.accent;
        const s = 7;
        ctx.beginPath();
        ctx.moveTo(cx, cy - s);
        ctx.lineTo(cx + s, cy);
        ctx.lineTo(cx, cy + s);
        ctx.lineTo(cx - s, cy);
        ctx.closePath();
        ctx.fill();
      }
      // proof ticks travel along 4 horizontal rails toward the diamond (anchor beat)
      ctx.lineWidth = 1;
      for (let lane = 0; lane < 4; lane++) {
        const ly = cy + (lane - 1.5) * (R * 0.42);
        const fromLeft = lane % 2 === 0;
        const x0 = fromLeft ? 0 : W;
        const x1 = fromLeft ? cx - R - 24 : cx + R + 24;
        ctx.globalAlpha = 0.18;
        ctx.strokeStyle = pal.fgMute;
        ctx.beginPath();
        ctx.moveTo(x0, ly);
        ctx.lineTo(x1, ly);
        ctx.stroke();
        // three ticks per rail, staggered
        for (let k = 0; k < 3; k++) {
          const ph = (t * 0.12 + k * 0.33 + lane * 0.17) % 1;
          const tx = x0 + (x1 - x0) * ph;
          ctx.globalAlpha = 0.7 * (1 - ph * 0.4);
          ctx.strokeStyle = pal.stroke;
          ctx.beginPath();
          ctx.moveTo(tx, ly - 4);
          ctx.lineTo(tx, ly + 4);
          ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
    };
  }

  /* ---------- scene 3 · hash rail field ---------- */
  function hashFieldScene(W, H, pal) {
    const gap = 26;
    const cols = Math.ceil(W / gap),
      rows = Math.ceil(H / gap);
    const rnd = mulberry32(4812);
    const jitter = [];
    for (let i = 0; i < cols * rows; i++) jitter.push(rnd());
    return function draw(ctx, t) {
      const waveX = (t * 0.07 % 1.4 - 0.2) * W; // verification wave sweeps L→R
      ctx.lineWidth = 1;
      for (let i = 0; i < cols; i++) {
        for (let j = 0; j < rows; j++) {
          const x = i * gap + gap / 2,
            y = j * gap + gap / 2;
          const d = Math.abs(x - waveX);
          const lit = Math.max(0, 1 - d / (W * 0.18));
          const base = 0.1 + jitter[i * rows + j] * 0.1;
          const h = 4 + lit * 5;
          ctx.globalAlpha = base + lit * 0.65;
          ctx.strokeStyle = lit > 0.85 && jitter[i * rows + j] > 0.93 ? pal.accent : pal.stroke;
          ctx.beginPath();
          ctx.moveTo(x, y - h / 2);
          ctx.lineTo(x, y + h / 2);
          ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
    };
  }
  const SCENES = {
    mesh: meshScene,
    bundle: bundleScene,
    hashfield: hashFieldScene
  };
  function CkHeroStage(canvas) {
    let raf = null,
      draw = null,
      pal = readPalette(),
      W = 0,
      H = 0,
      kind = 'mesh';
    let start = performance.now();
    const ctx = canvas.getContext('2d');
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    function resize() {
      const rect = canvas.getBoundingClientRect();
      W = rect.width;
      H = rect.height;
      canvas.width = Math.round(W * dpr);
      canvas.height = Math.round(H * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      build();
    }
    function build() {
      if (W > 0 && H > 0) {
        draw = SCENES[kind](W, H, pal);
        frame(true);
      }
    }
    function frame(force) {
      if (!draw) return;
      ctx.clearRect(0, 0, W, H);
      const t = REDUCED.matches ? 6.5 : (performance.now() - start) / 1000;
      draw(ctx, t);
      if (REDUCED.matches && !force) stop();
    }
    function loop() {
      frame();
      raf = requestAnimationFrame(loop);
    }
    function startLoop() {
      stop();
      if (REDUCED.matches) {
        frame(true);
        return;
      } // single static frame
      raf = requestAnimationFrame(loop);
    }
    function stop() {
      if (raf) {
        cancelAnimationFrame(raf);
        raf = null;
      }
    }
    window.addEventListener('resize', resize);
    REDUCED.addEventListener?.('change', startLoop);
    return {
      setScene(k) {
        if (SCENES[k]) {
          kind = k;
          start = performance.now();
          build();
          startLoop();
        }
      },
      refreshPalette() {
        pal = readPalette();
        build();
      },
      start() {
        resize();
        startLoop();
      },
      stop
    };
  }
  window.CkHeroStage = CkHeroStage;
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "redesign-v2/hero-scenes.js", error: String((e && e.message) || e) }); }

// redesign-v2/tweaks-panel.jsx
try { (() => {
// @ds-adherence-ignore -- omelette starter scaffold (raw elements/hex/px by design)

/* BEGIN USAGE */
// tweaks-panel.jsx
// Reusable Tweaks shell + form-control helpers.
// Exports (to window): useTweaks, TweaksPanel, TweakSection, TweakRow, TweakSlider,
//   TweakToggle, TweakRadio, TweakSelect, TweakText, TweakNumber, TweakColor, TweakButton.
//
// Owns the host protocol (listens for __activate_edit_mode / __deactivate_edit_mode,
// posts __edit_mode_available / __edit_mode_set_keys / __edit_mode_dismissed) so
// individual prototypes don't re-roll it. Ships a consistent set of controls so you
// don't hand-draw <input type="range">, segmented radios, steppers, etc.
//
// Usage (in an HTML file that loads React + Babel):
//
//   const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
//     "primaryColor": "#D97757",
//     "palette": ["#D97757", "#29261b", "#f6f4ef"],
//     "fontSize": 16,
//     "density": "regular",
//     "dark": false
//   }/*EDITMODE-END*/;
//
//   function App() {
//     const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
//     return (
//       <div style={{ fontSize: t.fontSize, color: t.primaryColor }}>
//         Hello
//         <TweaksPanel>
//           <TweakSection label="Typography" />
//           <TweakSlider label="Font size" value={t.fontSize} min={10} max={32} unit="px"
//                        onChange={(v) => setTweak('fontSize', v)} />
//           <TweakRadio  label="Density" value={t.density}
//                        options={['compact', 'regular', 'comfy']}
//                        onChange={(v) => setTweak('density', v)} />
//           <TweakSection label="Theme" />
//           <TweakColor  label="Primary" value={t.primaryColor}
//                        options={['#D97757', '#2A6FDB', '#1F8A5B', '#7A5AE0']}
//                        onChange={(v) => setTweak('primaryColor', v)} />
//           <TweakColor  label="Palette" value={t.palette}
//                        options={[['#D97757', '#29261b', '#f6f4ef'],
//                                  ['#475569', '#0f172a', '#f1f5f9']]}
//                        onChange={(v) => setTweak('palette', v)} />
//           <TweakToggle label="Dark mode" value={t.dark}
//                        onChange={(v) => setTweak('dark', v)} />
//         </TweaksPanel>
//       </div>
//     );
//   }
//
// TweakRadio is the segmented control for 2–3 short options (auto-falls-back to
// TweakSelect past ~16/~10 chars per label); reach for TweakSelect directly when
// options are many or long. For color tweaks always curate 3-4 options rather than
// a free picker; an option can also be a whole 2–5 color palette (the stored value
// is the array). The Tweak* controls are a floor, not a ceiling — build custom
// controls inside the panel if a tweak calls for UI they don't cover.
/* END USAGE */
// ─────────────────────────────────────────────────────────────────────────────

const __TWEAKS_STYLE = `
  .twk-panel{position:fixed;right:16px;bottom:16px;z-index:2147483646;width:280px;
    max-height:calc(100vh - 32px);display:flex;flex-direction:column;
    transform:scale(var(--dc-inv-zoom,1));transform-origin:bottom right;
    background:rgba(250,249,247,.78);color:#29261b;
    -webkit-backdrop-filter:blur(24px) saturate(160%);backdrop-filter:blur(24px) saturate(160%);
    border:.5px solid rgba(255,255,255,.6);border-radius:14px;
    box-shadow:0 1px 0 rgba(255,255,255,.5) inset,0 12px 40px rgba(0,0,0,.18);
    font:11.5px/1.4 ui-sans-serif,system-ui,-apple-system,sans-serif;overflow:hidden}
  .twk-hd{display:flex;align-items:center;justify-content:space-between;
    padding:10px 8px 10px 14px;cursor:move;user-select:none}
  .twk-hd b{font-size:12px;font-weight:600;letter-spacing:.01em}
  .twk-x{appearance:none;border:0;background:transparent;color:rgba(41,38,27,.55);
    width:22px;height:22px;border-radius:6px;cursor:default;font-size:13px;line-height:1}
  .twk-x:hover{background:rgba(0,0,0,.06);color:#29261b}
  .twk-body{padding:2px 14px 14px;display:flex;flex-direction:column;gap:10px;
    overflow-y:auto;overflow-x:hidden;min-height:0;
    scrollbar-width:thin;scrollbar-color:rgba(0,0,0,.15) transparent}
  .twk-body::-webkit-scrollbar{width:8px}
  .twk-body::-webkit-scrollbar-track{background:transparent;margin:2px}
  .twk-body::-webkit-scrollbar-thumb{background:rgba(0,0,0,.15);border-radius:4px;
    border:2px solid transparent;background-clip:content-box}
  .twk-body::-webkit-scrollbar-thumb:hover{background:rgba(0,0,0,.25);
    border:2px solid transparent;background-clip:content-box}
  .twk-row{display:flex;flex-direction:column;gap:5px}
  .twk-row-h{flex-direction:row;align-items:center;justify-content:space-between;gap:10px}
  .twk-lbl{display:flex;justify-content:space-between;align-items:baseline;
    color:rgba(41,38,27,.72)}
  .twk-lbl>span:first-child{font-weight:500}
  .twk-val{color:rgba(41,38,27,.5);font-variant-numeric:tabular-nums}

  .twk-sect{font-size:10px;font-weight:600;letter-spacing:.06em;text-transform:uppercase;
    color:rgba(41,38,27,.45);padding:10px 0 0}
  .twk-sect:first-child{padding-top:0}

  .twk-field{appearance:none;box-sizing:border-box;width:100%;min-width:0;height:26px;padding:0 8px;
    border:.5px solid rgba(0,0,0,.1);border-radius:7px;
    background:rgba(255,255,255,.6);color:inherit;font:inherit;outline:none}
  .twk-field:focus{border-color:rgba(0,0,0,.25);background:rgba(255,255,255,.85)}
  select.twk-field{padding-right:22px;
    background-image:url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'><path fill='rgba(0,0,0,.5)' d='M0 0h10L5 6z'/></svg>");
    background-repeat:no-repeat;background-position:right 8px center}

  .twk-slider{appearance:none;-webkit-appearance:none;width:100%;height:4px;margin:6px 0;
    border-radius:999px;background:rgba(0,0,0,.12);outline:none}
  .twk-slider::-webkit-slider-thumb{-webkit-appearance:none;appearance:none;
    width:14px;height:14px;border-radius:50%;background:#fff;
    border:.5px solid rgba(0,0,0,.12);box-shadow:0 1px 3px rgba(0,0,0,.2);cursor:default}
  .twk-slider::-moz-range-thumb{width:14px;height:14px;border-radius:50%;
    background:#fff;border:.5px solid rgba(0,0,0,.12);box-shadow:0 1px 3px rgba(0,0,0,.2);cursor:default}

  .twk-seg{position:relative;display:flex;padding:2px;border-radius:8px;
    background:rgba(0,0,0,.06);user-select:none}
  .twk-seg-thumb{position:absolute;top:2px;bottom:2px;border-radius:6px;
    background:rgba(255,255,255,.9);box-shadow:0 1px 2px rgba(0,0,0,.12);
    transition:left .15s cubic-bezier(.3,.7,.4,1),width .15s}
  .twk-seg.dragging .twk-seg-thumb{transition:none}
  .twk-seg button{appearance:none;position:relative;z-index:1;flex:1;border:0;
    background:transparent;color:inherit;font:inherit;font-weight:500;min-height:22px;
    border-radius:6px;cursor:default;padding:4px 6px;line-height:1.2;
    overflow-wrap:anywhere}

  .twk-toggle{position:relative;width:32px;height:18px;border:0;border-radius:999px;
    background:rgba(0,0,0,.15);transition:background .15s;cursor:default;padding:0}
  .twk-toggle[data-on="1"]{background:#34c759}
  .twk-toggle i{position:absolute;top:2px;left:2px;width:14px;height:14px;border-radius:50%;
    background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.25);transition:transform .15s}
  .twk-toggle[data-on="1"] i{transform:translateX(14px)}

  .twk-num{display:flex;align-items:center;box-sizing:border-box;min-width:0;height:26px;padding:0 0 0 8px;
    border:.5px solid rgba(0,0,0,.1);border-radius:7px;background:rgba(255,255,255,.6)}
  .twk-num-lbl{font-weight:500;color:rgba(41,38,27,.6);cursor:ew-resize;
    user-select:none;padding-right:8px}
  .twk-num input{flex:1;min-width:0;height:100%;border:0;background:transparent;
    font:inherit;font-variant-numeric:tabular-nums;text-align:right;padding:0 8px 0 0;
    outline:none;color:inherit;-moz-appearance:textfield}
  .twk-num input::-webkit-inner-spin-button,.twk-num input::-webkit-outer-spin-button{
    -webkit-appearance:none;margin:0}
  .twk-num-unit{padding-right:8px;color:rgba(41,38,27,.45)}

  .twk-btn{appearance:none;height:26px;padding:0 12px;border:0;border-radius:7px;
    background:rgba(0,0,0,.78);color:#fff;font:inherit;font-weight:500;cursor:default}
  .twk-btn:hover{background:rgba(0,0,0,.88)}
  .twk-btn.secondary{background:rgba(0,0,0,.06);color:inherit}
  .twk-btn.secondary:hover{background:rgba(0,0,0,.1)}

  .twk-swatch{appearance:none;-webkit-appearance:none;width:56px;height:22px;
    border:.5px solid rgba(0,0,0,.1);border-radius:6px;padding:0;cursor:default;
    background:transparent;flex-shrink:0}
  .twk-swatch::-webkit-color-swatch-wrapper{padding:0}
  .twk-swatch::-webkit-color-swatch{border:0;border-radius:5.5px}
  .twk-swatch::-moz-color-swatch{border:0;border-radius:5.5px}

  .twk-chips{display:flex;gap:6px}
  .twk-chip{position:relative;appearance:none;flex:1;min-width:0;height:46px;
    padding:0;border:0;border-radius:6px;overflow:hidden;cursor:default;
    box-shadow:0 0 0 .5px rgba(0,0,0,.12),0 1px 2px rgba(0,0,0,.06);
    transition:transform .12s cubic-bezier(.3,.7,.4,1),box-shadow .12s}
  .twk-chip:hover{transform:translateY(-1px);
    box-shadow:0 0 0 .5px rgba(0,0,0,.18),0 4px 10px rgba(0,0,0,.12)}
  .twk-chip[data-on="1"]{box-shadow:0 0 0 1.5px rgba(0,0,0,.85),
    0 2px 6px rgba(0,0,0,.15)}
  .twk-chip>span{position:absolute;top:0;bottom:0;right:0;width:34%;
    display:flex;flex-direction:column;box-shadow:-1px 0 0 rgba(0,0,0,.1)}
  .twk-chip>span>i{flex:1;box-shadow:0 -1px 0 rgba(0,0,0,.1)}
  .twk-chip>span>i:first-child{box-shadow:none}
  .twk-chip svg{position:absolute;top:6px;left:6px;width:13px;height:13px;
    filter:drop-shadow(0 1px 1px rgba(0,0,0,.3))}
`;

// ── useTweaks ───────────────────────────────────────────────────────────────
// Single source of truth for tweak values. setTweak persists via the host
// (__edit_mode_set_keys → host rewrites the EDITMODE block on disk).
function useTweaks(defaults) {
  const [values, setValues] = React.useState(defaults);
  // Accepts either setTweak('key', value) or setTweak({ key: value, ... }) so a
  // useState-style call doesn't write a "[object Object]" key into the persisted
  // JSON block.
  const setTweak = React.useCallback((keyOrEdits, val) => {
    const edits = typeof keyOrEdits === 'object' && keyOrEdits !== null ? keyOrEdits : {
      [keyOrEdits]: val
    };
    setValues(prev => ({
      ...prev,
      ...edits
    }));
    window.parent.postMessage({
      type: '__edit_mode_set_keys',
      edits
    }, '*');
    // Same-window signal so in-page listeners (deck-stage rail thumbnails)
    // can react — the parent message only reaches the host, not peers.
    window.dispatchEvent(new CustomEvent('tweakchange', {
      detail: edits
    }));
  }, []);
  return [values, setTweak];
}

// ── TweaksPanel ─────────────────────────────────────────────────────────────
// Floating shell. Registers the protocol listener BEFORE announcing
// availability — if the announce ran first, the host's activate could land
// before our handler exists and the toolbar toggle would silently no-op.
// The close button posts __edit_mode_dismissed so the host's toolbar toggle
// flips off in lockstep; the host echoes __deactivate_edit_mode back which
// is what actually hides the panel.
function TweaksPanel({
  title = 'Tweaks',
  children
}) {
  const [open, setOpen] = React.useState(false);
  const dragRef = React.useRef(null);
  const offsetRef = React.useRef({
    x: 16,
    y: 16
  });
  const PAD = 16;
  const clampToViewport = React.useCallback(() => {
    const panel = dragRef.current;
    if (!panel) return;
    const w = panel.offsetWidth,
      h = panel.offsetHeight;
    const maxRight = Math.max(PAD, window.innerWidth - w - PAD);
    const maxBottom = Math.max(PAD, window.innerHeight - h - PAD);
    offsetRef.current = {
      x: Math.min(maxRight, Math.max(PAD, offsetRef.current.x)),
      y: Math.min(maxBottom, Math.max(PAD, offsetRef.current.y))
    };
    panel.style.right = offsetRef.current.x + 'px';
    panel.style.bottom = offsetRef.current.y + 'px';
  }, []);
  React.useEffect(() => {
    if (!open) return;
    clampToViewport();
    if (typeof ResizeObserver === 'undefined') {
      window.addEventListener('resize', clampToViewport);
      return () => window.removeEventListener('resize', clampToViewport);
    }
    const ro = new ResizeObserver(clampToViewport);
    ro.observe(document.documentElement);
    return () => ro.disconnect();
  }, [open, clampToViewport]);
  React.useEffect(() => {
    const onMsg = e => {
      const t = e?.data?.type;
      if (t === '__activate_edit_mode') setOpen(true);else if (t === '__deactivate_edit_mode') setOpen(false);
    };
    window.addEventListener('message', onMsg);
    window.parent.postMessage({
      type: '__edit_mode_available'
    }, '*');
    return () => window.removeEventListener('message', onMsg);
  }, []);
  const dismiss = () => {
    setOpen(false);
    window.parent.postMessage({
      type: '__edit_mode_dismissed'
    }, '*');
  };
  const onDragStart = e => {
    const panel = dragRef.current;
    if (!panel) return;
    const r = panel.getBoundingClientRect();
    const sx = e.clientX,
      sy = e.clientY;
    const startRight = window.innerWidth - r.right;
    const startBottom = window.innerHeight - r.bottom;
    const move = ev => {
      offsetRef.current = {
        x: startRight - (ev.clientX - sx),
        y: startBottom - (ev.clientY - sy)
      };
      clampToViewport();
    };
    const up = () => {
      window.removeEventListener('mousemove', move);
      window.removeEventListener('mouseup', up);
    };
    window.addEventListener('mousemove', move);
    window.addEventListener('mouseup', up);
  };
  if (!open) return null;
  return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("style", null, __TWEAKS_STYLE), /*#__PURE__*/React.createElement("div", {
    ref: dragRef,
    className: "twk-panel",
    "data-omelette-chrome": "",
    style: {
      right: offsetRef.current.x,
      bottom: offsetRef.current.y
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "twk-hd",
    onMouseDown: onDragStart
  }, /*#__PURE__*/React.createElement("b", null, title), /*#__PURE__*/React.createElement("button", {
    className: "twk-x",
    "aria-label": "Close tweaks",
    onMouseDown: e => e.stopPropagation(),
    onClick: dismiss
  }, "\u2715")), /*#__PURE__*/React.createElement("div", {
    className: "twk-body"
  }, children)));
}

// ── Layout helpers ──────────────────────────────────────────────────────────

function TweakSection({
  label,
  children
}) {
  return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("div", {
    className: "twk-sect"
  }, label), children);
}
function TweakRow({
  label,
  value,
  children,
  inline = false
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: inline ? 'twk-row twk-row-h' : 'twk-row'
  }, /*#__PURE__*/React.createElement("div", {
    className: "twk-lbl"
  }, /*#__PURE__*/React.createElement("span", null, label), value != null && /*#__PURE__*/React.createElement("span", {
    className: "twk-val"
  }, value)), children);
}

// ── Controls ────────────────────────────────────────────────────────────────

function TweakSlider({
  label,
  value,
  min = 0,
  max = 100,
  step = 1,
  unit = '',
  onChange
}) {
  return /*#__PURE__*/React.createElement(TweakRow, {
    label: label,
    value: `${value}${unit}`
  }, /*#__PURE__*/React.createElement("input", {
    type: "range",
    className: "twk-slider",
    min: min,
    max: max,
    step: step,
    value: value,
    onChange: e => onChange(Number(e.target.value))
  }));
}
function TweakToggle({
  label,
  value,
  onChange
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "twk-row twk-row-h"
  }, /*#__PURE__*/React.createElement("div", {
    className: "twk-lbl"
  }, /*#__PURE__*/React.createElement("span", null, label)), /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: "twk-toggle",
    "data-on": value ? '1' : '0',
    role: "switch",
    "aria-checked": !!value,
    onClick: () => onChange(!value)
  }, /*#__PURE__*/React.createElement("i", null)));
}
function TweakRadio({
  label,
  value,
  options,
  onChange
}) {
  const trackRef = React.useRef(null);
  const [dragging, setDragging] = React.useState(false);
  // The active value is read by pointer-move handlers attached for the lifetime
  // of a drag — ref it so a stale closure doesn't fire onChange for every move.
  const valueRef = React.useRef(value);
  valueRef.current = value;

  // Segments wrap mid-word once per-segment width runs out. The track is
  // ~248px (280 panel − 28 body pad − 4 seg pad), each button loses 12px
  // to its own padding, and 11.5px system-ui averages ~6.3px/char — so 2
  // options fit ~16 chars each, 3 fit ~10. Past that (or >3 options), fall
  // back to a dropdown rather than wrap.
  const labelLen = o => String(typeof o === 'object' ? o.label : o).length;
  const maxLen = options.reduce((m, o) => Math.max(m, labelLen(o)), 0);
  const fitsAsSegments = maxLen <= ({
    2: 16,
    3: 10
  }[options.length] ?? 0);
  if (!fitsAsSegments) {
    // <select> emits strings — map back to the original option value so the
    // fallback stays type-preserving (numbers, booleans) like the segment path.
    const resolve = s => {
      const m = options.find(o => String(typeof o === 'object' ? o.value : o) === s);
      return m === undefined ? s : typeof m === 'object' ? m.value : m;
    };
    return /*#__PURE__*/React.createElement(TweakSelect, {
      label: label,
      value: value,
      options: options,
      onChange: s => onChange(resolve(s))
    });
  }
  const opts = options.map(o => typeof o === 'object' ? o : {
    value: o,
    label: o
  });
  const idx = Math.max(0, opts.findIndex(o => o.value === value));
  const n = opts.length;
  const segAt = clientX => {
    const r = trackRef.current.getBoundingClientRect();
    const inner = r.width - 4;
    const i = Math.floor((clientX - r.left - 2) / inner * n);
    return opts[Math.max(0, Math.min(n - 1, i))].value;
  };
  const onPointerDown = e => {
    setDragging(true);
    const v0 = segAt(e.clientX);
    if (v0 !== valueRef.current) onChange(v0);
    const move = ev => {
      if (!trackRef.current) return;
      const v = segAt(ev.clientX);
      if (v !== valueRef.current) onChange(v);
    };
    const up = () => {
      setDragging(false);
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
  return /*#__PURE__*/React.createElement(TweakRow, {
    label: label
  }, /*#__PURE__*/React.createElement("div", {
    ref: trackRef,
    role: "radiogroup",
    onPointerDown: onPointerDown,
    className: dragging ? 'twk-seg dragging' : 'twk-seg'
  }, /*#__PURE__*/React.createElement("div", {
    className: "twk-seg-thumb",
    style: {
      left: `calc(2px + ${idx} * (100% - 4px) / ${n})`,
      width: `calc((100% - 4px) / ${n})`
    }
  }), opts.map(o => /*#__PURE__*/React.createElement("button", {
    key: o.value,
    type: "button",
    role: "radio",
    "aria-checked": o.value === value
  }, o.label))));
}
function TweakSelect({
  label,
  value,
  options,
  onChange
}) {
  return /*#__PURE__*/React.createElement(TweakRow, {
    label: label
  }, /*#__PURE__*/React.createElement("select", {
    className: "twk-field",
    value: value,
    onChange: e => onChange(e.target.value)
  }, options.map(o => {
    const v = typeof o === 'object' ? o.value : o;
    const l = typeof o === 'object' ? o.label : o;
    return /*#__PURE__*/React.createElement("option", {
      key: v,
      value: v
    }, l);
  })));
}
function TweakText({
  label,
  value,
  placeholder,
  onChange
}) {
  return /*#__PURE__*/React.createElement(TweakRow, {
    label: label
  }, /*#__PURE__*/React.createElement("input", {
    className: "twk-field",
    type: "text",
    value: value,
    placeholder: placeholder,
    onChange: e => onChange(e.target.value)
  }));
}
function TweakNumber({
  label,
  value,
  min,
  max,
  step = 1,
  unit = '',
  onChange
}) {
  const clamp = n => {
    if (min != null && n < min) return min;
    if (max != null && n > max) return max;
    return n;
  };
  const startRef = React.useRef({
    x: 0,
    val: 0
  });
  const onScrubStart = e => {
    e.preventDefault();
    startRef.current = {
      x: e.clientX,
      val: value
    };
    const decimals = (String(step).split('.')[1] || '').length;
    const move = ev => {
      const dx = ev.clientX - startRef.current.x;
      const raw = startRef.current.val + dx * step;
      const snapped = Math.round(raw / step) * step;
      onChange(clamp(Number(snapped.toFixed(decimals))));
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "twk-num"
  }, /*#__PURE__*/React.createElement("span", {
    className: "twk-num-lbl",
    onPointerDown: onScrubStart
  }, label), /*#__PURE__*/React.createElement("input", {
    type: "number",
    value: value,
    min: min,
    max: max,
    step: step,
    onChange: e => onChange(clamp(Number(e.target.value)))
  }), unit && /*#__PURE__*/React.createElement("span", {
    className: "twk-num-unit"
  }, unit));
}

// Relative-luminance contrast pick — checkmarks drawn over a swatch need to
// read on both #111 and #fafafa without per-option configuration. Hex input
// only (#rgb / #rrggbb); named or rgb()/hsl() colors fall through to "light".
function __twkIsLight(hex) {
  const h = String(hex).replace('#', '');
  const x = h.length === 3 ? h.replace(/./g, c => c + c) : h.padEnd(6, '0');
  const n = parseInt(x.slice(0, 6), 16);
  if (Number.isNaN(n)) return true;
  const r = n >> 16 & 255,
    g = n >> 8 & 255,
    b = n & 255;
  return r * 299 + g * 587 + b * 114 > 148000;
}
const __TwkCheck = ({
  light
}) => /*#__PURE__*/React.createElement("svg", {
  viewBox: "0 0 14 14",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M3 7.2 5.8 10 11 4.2",
  fill: "none",
  strokeWidth: "2.2",
  strokeLinecap: "round",
  strokeLinejoin: "round",
  stroke: light ? 'rgba(0,0,0,.78)' : '#fff'
}));

// TweakColor — curated color/palette picker. Each option is either a single
// hex string or an array of 1-5 hex strings; the card adapts — a lone color
// renders solid, a palette renders colors[0] as the hero (left ~2/3) with the
// rest stacked in a sharp column on the right. onChange emits the
// option in the shape it was passed (string stays string, array stays array).
// Without options it falls back to the native color input for back-compat.
function TweakColor({
  label,
  value,
  options,
  onChange
}) {
  if (!options || !options.length) {
    return /*#__PURE__*/React.createElement("div", {
      className: "twk-row twk-row-h"
    }, /*#__PURE__*/React.createElement("div", {
      className: "twk-lbl"
    }, /*#__PURE__*/React.createElement("span", null, label)), /*#__PURE__*/React.createElement("input", {
      type: "color",
      className: "twk-swatch",
      value: value,
      onChange: e => onChange(e.target.value)
    }));
  }
  // Native <input type=color> emits lowercase hex per the HTML spec, so
  // compare case-insensitively. String() guards JSON.stringify(undefined),
  // which returns the primitive undefined (no .toLowerCase).
  const key = o => String(JSON.stringify(o)).toLowerCase();
  const cur = key(value);
  return /*#__PURE__*/React.createElement(TweakRow, {
    label: label
  }, /*#__PURE__*/React.createElement("div", {
    className: "twk-chips",
    role: "radiogroup"
  }, options.map((o, i) => {
    const colors = Array.isArray(o) ? o : [o];
    const [hero, ...rest] = colors;
    const sup = rest.slice(0, 4);
    const on = key(o) === cur;
    return /*#__PURE__*/React.createElement("button", {
      key: i,
      type: "button",
      className: "twk-chip",
      role: "radio",
      "aria-checked": on,
      "data-on": on ? '1' : '0',
      "aria-label": colors.join(', '),
      title: colors.join(' · '),
      style: {
        background: hero
      },
      onClick: () => onChange(o)
    }, sup.length > 0 && /*#__PURE__*/React.createElement("span", null, sup.map((c, j) => /*#__PURE__*/React.createElement("i", {
      key: j,
      style: {
        background: c
      }
    }))), on && /*#__PURE__*/React.createElement(__TwkCheck, {
      light: __twkIsLight(hero)
    }));
  })));
}
function TweakButton({
  label,
  onClick,
  secondary = false
}) {
  return /*#__PURE__*/React.createElement("button", {
    type: "button",
    className: secondary ? 'twk-btn secondary' : 'twk-btn',
    onClick: onClick
  }, label);
}
Object.assign(window, {
  useTweaks,
  TweaksPanel,
  TweakSection,
  TweakRow,
  TweakSlider,
  TweakToggle,
  TweakRadio,
  TweakSelect,
  TweakText,
  TweakNumber,
  TweakColor,
  TweakButton
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "redesign-v2/tweaks-panel.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/App.jsx
try { (() => {
// App shell
const {
  useState,
  useEffect
} = React;
const App = () => {
  const [theme, setTheme] = useState(() => localStorage.getItem('ck-theme') || 'dark');
  const [view, setView] = useState(() => localStorage.getItem('ck-view') || 'overview');
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('ck-theme', theme);
  }, [theme]);
  useEffect(() => {
    localStorage.setItem('ck-view', view);
  }, [view]);
  const titles = {
    overview: {
      title: 'Overview',
      subtitle: 'last resolve · 14:12 utc · 5 kernels · 1 gate open'
    },
    kernels: {
      title: 'kernel-core',
      subtitle: 'v 1.4.0 · rev 892f · GAL-6 attested'
    },
    attest: {
      title: 'Attestation review',
      subtitle: 'gate-7712 · promote kernel-policy → production'
    },
    policy: {
      title: 'Policy',
      subtitle: 'M3 · promotion · 12 policies compiled'
    },
    artifacts: {
      title: 'Artifacts',
      subtitle: '423 signed · 7 pending attestation'
    },
    rekor: {
      title: 'Rekor log',
      subtitle: 'anchor tree · depth 8 · 4,812 entries'
    }
  };
  const nav = id => {
    setView(id === 'kernels' ? 'kernels' : id === 'attest' ? 'attest' : id);
  };
  return /*#__PURE__*/React.createElement("div", {
    "data-screen-label": `console · ${view}`,
    style: {
      display: 'flex',
      minHeight: '100vh',
      background: 'var(--ck-bg-0)',
      color: 'var(--ck-fg-1)'
    }
  }, /*#__PURE__*/React.createElement(Sidebar, {
    current: view,
    onNavigate: nav
  }), /*#__PURE__*/React.createElement("main", {
    style: {
      flex: 1,
      display: 'flex',
      flexDirection: 'column'
    }
  }, /*#__PURE__*/React.createElement(Header, {
    title: titles[view].title,
    subtitle: titles[view].subtitle,
    theme: theme,
    onTheme: setTheme
  }), view === 'overview' && /*#__PURE__*/React.createElement(Overview, null), view === 'kernels' && /*#__PURE__*/React.createElement(KernelDetail, null), view === 'attest' && /*#__PURE__*/React.createElement(AttestationReview, null), view === 'policy' && /*#__PURE__*/React.createElement(PolicyView, null), view === 'artifacts' && /*#__PURE__*/React.createElement(ArtifactsView, null), view === 'rekor' && /*#__PURE__*/React.createElement(RekorView, null)));
};
ReactDOM.createRoot(document.getElementById('root')).render(/*#__PURE__*/React.createElement(App, null));
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/App.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/ArtifactsView.jsx
try { (() => {
// Artifacts view — inventory grid with trust + xBoM summaries
const ArtifactsView = () => {
  const artifacts = [{
    name: 'atlas-triage-7',
    kind: 'ai-system',
    gal: 5,
    state: 'attested',
    digest: 'sha256:a42f…c01e',
    comps: 5
  }, {
    name: 'kernel-core',
    kind: 'kernel',
    gal: 6,
    state: 'attested',
    digest: 'sha256:c91d…4b2a',
    comps: 3
  }, {
    name: 'kernel-policy',
    kind: 'kernel',
    gal: 4,
    state: 'promoting',
    digest: 'sha256:77e0…9f3c',
    comps: 4
  }, {
    name: 'guard-eval-pack',
    kind: 'dataset',
    gal: 3,
    state: 'witnessed',
    digest: 'sha256:12ab…8e5d',
    comps: 2
  }, {
    name: 'triage-lora',
    kind: 'weights',
    gal: 6,
    state: 'attested',
    digest: 'sha256:3f09…d72b',
    comps: 1
  }, {
    name: 'sandbox-runner',
    kind: 'kernel',
    gal: 2,
    state: 'pending',
    digest: 'sha256:5e71…b0c4',
    comps: 6
  }];
  const stMap = {
    attested: 'attested',
    promoting: 'witness',
    witnessed: 'witness',
    pending: 'mute'
  };
  const glyph = {
    attested: '⊢',
    promoting: '⇝',
    witnessed: '⊛',
    pending: '◌'
  };
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '24px 28px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      marginBottom: 16
    }
  }, /*#__PURE__*/React.createElement(Badge, {
    kind: "attested"
  }, "\u2297 423 sealed"), /*#__PURE__*/React.createElement(Badge, {
    kind: "witness"
  }, "\u229B 12 witnessing"), /*#__PURE__*/React.createElement(Badge, {
    kind: "mute"
  }, "\u25CC 7 pending"), /*#__PURE__*/React.createElement("div", {
    style: {
      marginLeft: 'auto'
    }
  }, /*#__PURE__*/React.createElement(CkButton, {
    variant: "ghost"
  }, "\u220E Export AIBOM"))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: 'repeat(3, 1fr)',
      gap: 14
    }
  }, artifacts.map(a => /*#__PURE__*/React.createElement(Bento, {
    key: a.name,
    pad: 16
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'flex-start',
      gap: 10,
      marginBottom: 12
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      font: "600 16px 'JetBrains Mono', monospace",
      color: 'var(--ck-accent)',
      lineHeight: 1
    }
  }, a.kind === 'kernel' ? '▰' : a.kind === 'dataset' ? '≡' : a.kind === 'weights' ? '◈' : '◇'), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 13px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, a.name), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.06em',
      textTransform: 'uppercase'
    }
  }, a.kind))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      marginBottom: 10
    }
  }, /*#__PURE__*/React.createElement(GalRail, {
    level: a.gal,
    attested: a.state === 'attested',
    width: 84
  }), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "600 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-2)'
    }
  }, "GAL-", a.gal)), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "400 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-3)',
      marginBottom: 12
    }
  }, a.digest), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between'
    }
  }, /*#__PURE__*/React.createElement(Badge, {
    kind: stMap[a.state]
  }, glyph[a.state], " ", a.state), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, a.comps, " components"))))));
};
Object.assign(window, {
  ArtifactsView
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/ArtifactsView.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/AttestationReview.jsx
try { (() => {
// Attestation review — gate decision with deny branch
const AttestationReview = () => {
  const [decision, setDecision] = React.useState(null);
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '24px 28px',
      display: 'grid',
      gridTemplateColumns: '1fr 300px',
      gap: 20
    }
  }, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'baseline',
      marginBottom: 16
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: 0,
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Gate \xB7 promote kernel-policy"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em'
    }
  }, "GATE-7712 \xB7 M3")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr 140px 1fr',
      alignItems: 'center',
      gap: 14,
      padding: '8px 0 18px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(Octagon, {
    size: 72
  }, /*#__PURE__*/React.createElement("polygon", {
    points: "48,34 64,48 48,62 32,48",
    fill: "var(--ck-accent)"
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)',
      marginTop: 6
    }
  }, "kernel-policy")), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(Diamond, {
    size: 96,
    nucleusFilled: decision === 'attest'
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)',
      marginTop: 6
    }
  }, "gate")), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'inline-block',
      position: 'relative'
    }
  }, /*#__PURE__*/React.createElement(Octagon, {
    size: 72
  }), decision === 'deny' && /*#__PURE__*/React.createElement("svg", {
    width: "72",
    height: "72",
    style: {
      position: 'absolute',
      inset: 0
    },
    viewBox: "0 0 72 72"
  }, /*#__PURE__*/React.createElement("line", {
    x1: "18",
    y1: "18",
    x2: "54",
    y2: "54",
    stroke: "var(--ck-deny)",
    strokeWidth: "3"
  }), /*#__PURE__*/React.createElement("line", {
    x1: "54",
    y1: "18",
    x2: "18",
    y2: "54",
    stroke: "var(--ck-deny)",
    strokeWidth: "3"
  }))), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 10px 'JetBrains Mono', monospace",
      color: decision === 'deny' ? 'var(--ck-deny)' : 'var(--ck-fg-1)',
      marginTop: 6
    }
  }, decision === 'deny' ? 'refused' : 'production'))), /*#__PURE__*/React.createElement("div", {
    className: "ck-invariant",
    style: {
      fontSize: 11,
      padding: '8px 12px',
      background: 'color-mix(in oklab, var(--ck-accent) 10%, transparent)',
      boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-accent) 40%, transparent)',
      clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)',
      marginBottom: 16
    }
  }, "mode changes deployment, not governance semantics"), /*#__PURE__*/React.createElement("h4", {
    style: {
      margin: '6px 0 8px',
      font: "700 11px 'Lato', sans-serif",
      color: 'var(--ck-fg-3)',
      letterSpacing: '.14em',
      textTransform: 'uppercase'
    }
  }, "Evidence"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gap: 8,
      font: "400 12px/1.5 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-2)'
    }
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-accent)'
    }
  }, "\u22A2"), " provenance \xB7 sha256:77e0\u20269f3c"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-accent)'
    }
  }, "\u22A2"), " usage envelope \xB7 policy-safe \u2200 inputs"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-witness)'
    }
  }, "\u229B"), " witness \xB7 3 of 5 required"), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-mute)'
    }
  }, "\u25D0"), " rekor anchor \xB7 pending")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 10,
      marginTop: 20
    }
  }, /*#__PURE__*/React.createElement(CkButton, {
    variant: "primary",
    onClick: () => setDecision('attest')
  }, "\u22A2 Attest + promote"), /*#__PURE__*/React.createElement(CkButton, {
    variant: "secondary",
    onClick: () => setDecision(null)
  }, "\u229B Request witness"), /*#__PURE__*/React.createElement(CkButton, {
    variant: "deny",
    onClick: () => setDecision('deny')
  }, "\u2298 Deny"))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 10
    }
  }, "Decision"), decision === null && /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "\u25CC pending"), decision === 'attest' && /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 13px 'JetBrains Mono', monospace",
      color: 'var(--ck-accent)'
    }
  }, "\u22A2 attested \xB7 promoted"), decision === 'deny' && /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 13px 'JetBrains Mono', monospace",
      color: 'var(--ck-deny)'
    }
  }, "\u2298 denied \xB7 refused")), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 10
    }
  }, "Reviewers"), ['ewald.k', 'morita.s', 'chen.j'].map(r => /*#__PURE__*/React.createElement("div", {
    key: r,
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      padding: '6px 0',
      font: "500 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-witness)'
    }
  }, "\u229B"), r)))));
};
Object.assign(window, {
  AttestationReview
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/AttestationReview.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/Header.jsx
try { (() => {
// Header — page title, theme switcher, attestation state
const Header = ({
  title,
  subtitle,
  theme,
  onTheme
}) => {
  const themes = [{
    id: 'dark',
    label: 'Dark'
  }, {
    id: 'light',
    label: 'Light'
  }, {
    id: 'hc',
    label: 'HC'
  }];
  return /*#__PURE__*/React.createElement("header", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 16,
      padding: '20px 28px',
      borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 2
    }
  }, "Governance console"), /*#__PURE__*/React.createElement("h1", {
    style: {
      margin: 0,
      font: "700 24px 'Roboto', sans-serif",
      letterSpacing: '-.01em',
      color: 'var(--ck-fg-1)'
    }
  }, title), subtitle && /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em',
      marginTop: 4
    }
  }, subtitle)), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 8
    }
  }, /*#__PURE__*/React.createElement(Badge, {
    kind: "attested"
  }, "\u22A2 GAL-5"), /*#__PURE__*/React.createElement(Badge, {
    kind: "witness"
  }, "\u229B 3 witnesses")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 2,
      padding: 2,
      clipPath: 'polygon(6px 0, calc(100% - 6px) 0, 100% 6px, 100% calc(100% - 6px), calc(100% - 6px) 100%, 6px 100%, 0 calc(100% - 6px), 0 6px)',
      background: 'color-mix(in oklab, var(--ck-stroke) 25%, transparent)'
    }
  }, themes.map(t => /*#__PURE__*/React.createElement("button", {
    key: t.id,
    onClick: () => onTheme(t.id),
    style: {
      font: "700 10px 'JetBrains Mono', monospace",
      letterSpacing: '.08em',
      padding: '6px 12px',
      border: 'none',
      cursor: 'pointer',
      background: theme === t.id ? 'var(--ck-accent)' : 'var(--ck-bg-0)',
      color: theme === t.id ? 'var(--ck-deep-blue)' : 'var(--ck-fg-2)',
      clipPath: 'polygon(4px 0, calc(100% - 4px) 0, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 0 calc(100% - 4px), 0 4px)'
    }
  }, t.label))));
};
Object.assign(window, {
  Header
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/Header.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/KernelDetail.jsx
try { (() => {
// Kernel detail view — single kernel, GAL rail at scale, hash-tick flow
const KernelDetail = () => /*#__PURE__*/React.createElement("div", {
  style: {
    padding: '24px 28px',
    display: 'grid',
    gridTemplateColumns: '260px 1fr',
    gap: 20
  }
}, /*#__PURE__*/React.createElement(Bento, {
  style: {
    textAlign: 'center',
    padding: 24
  }
}, /*#__PURE__*/React.createElement(Octagon, {
  size: 160
}, /*#__PURE__*/React.createElement("polygon", {
  points: "48,34 64,48 48,62 32,48",
  fill: "var(--ck-accent)"
}), /*#__PURE__*/React.createElement("circle", {
  cx: "48",
  cy: "48",
  r: "4",
  fill: "var(--ck-deep-blue)"
})), /*#__PURE__*/React.createElement("div", {
  style: {
    font: "700 16px 'Roboto', sans-serif",
    color: 'var(--ck-fg-1)',
    marginTop: 12
  }
}, "kernel-core"), /*#__PURE__*/React.createElement("div", {
  style: {
    font: "500 11px 'JetBrains Mono', monospace",
    color: 'var(--ck-fg-mute)',
    letterSpacing: '.04em',
    marginTop: 4
  }
}, "v 1.4.0 \xB7 rev 892f"), /*#__PURE__*/React.createElement("div", {
  style: {
    display: 'flex',
    justifyContent: 'center',
    gap: 6,
    marginTop: 14
  }
}, /*#__PURE__*/React.createElement(Badge, {
  kind: "attested"
}, "\u22A2 GAL-6"), /*#__PURE__*/React.createElement(Badge, {
  kind: "witness"
}, "\u229B 5"))), /*#__PURE__*/React.createElement("div", {
  style: {
    display: 'flex',
    flexDirection: 'column',
    gap: 16
  }
}, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("h3", {
  style: {
    margin: '0 0 14px',
    font: "500 16px 'Roboto', sans-serif",
    color: 'var(--ck-fg-1)'
  }
}, "Governance assurance level"), /*#__PURE__*/React.createElement("div", {
  style: {
    display: 'grid',
    gridTemplateColumns: '60px 1fr 80px',
    alignItems: 'center',
    gap: 14
  }
}, /*#__PURE__*/React.createElement("span", {
  style: {
    font: "700 32px 'Roboto', sans-serif",
    color: 'var(--ck-accent)'
  }
}, "6", /*#__PURE__*/React.createElement("span", {
  style: {
    font: "500 14px 'JetBrains Mono', monospace",
    color: 'var(--ck-fg-mute)'
  }
}, "/6")), /*#__PURE__*/React.createElement(GalRail, {
  level: 6,
  attested: true,
  width: "100%"
}), /*#__PURE__*/React.createElement(Badge, {
  kind: "attested"
}, "\u22A2 attested")), /*#__PURE__*/React.createElement("div", {
  style: {
    marginTop: 14,
    font: "400 12px 'Lato', sans-serif",
    color: 'var(--ck-fg-2)'
  }
}, "The kernel has accumulated six attested edges and four witness signatures. Promotion is now reversible only through an emergency edge.")), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("h3", {
  style: {
    margin: '0 0 16px',
    font: "500 16px 'Roboto', sans-serif",
    color: 'var(--ck-fg-1)'
  }
}, "Attestation edges"), /*#__PURE__*/React.createElement("div", {
  style: {
    display: 'grid',
    gap: 12
  }
}, [{
  a: 'source',
  b: 'builder',
  type: 'attested'
}, {
  a: 'builder',
  b: 'signer',
  type: 'attested'
}, {
  a: 'signer',
  b: 'rekor',
  type: 'promotion'
}, {
  a: 'rekor',
  b: 'anchor',
  type: 'kernel'
}].map((e, i) => /*#__PURE__*/React.createElement("div", {
  key: i,
  style: {
    display: 'grid',
    gridTemplateColumns: '80px 1fr 80px',
    alignItems: 'center',
    gap: 14
  }
}, /*#__PURE__*/React.createElement("span", {
  style: {
    font: "600 11px 'JetBrains Mono', monospace",
    color: 'var(--ck-fg-1)',
    textAlign: 'right'
  }
}, e.a), e.type === 'attested' && /*#__PURE__*/React.createElement(ArrowAttested, {
  width: 220
}), e.type === 'promotion' && /*#__PURE__*/React.createElement(ArrowPromotion, {
  width: 220
}), e.type === 'kernel' && /*#__PURE__*/React.createElement(ArrowKernelFlow, {
  width: 220
}), /*#__PURE__*/React.createElement("span", {
  style: {
    font: "600 11px 'JetBrains Mono', monospace",
    color: 'var(--ck-fg-1)'
  }
}, e.b)))))));
Object.assign(window, {
  KernelDetail
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/KernelDetail.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/Overview.jsx
try { (() => {
// Overview view — kernel roster, proof chain, Orb anchor
const Overview = () => {
  const kernels = [{
    name: 'kernel-core',
    gal: 6,
    state: 'attested',
    sig: 'sha256:a42f…c01e'
  }, {
    name: 'kernel-attest',
    gal: 5,
    state: 'attested',
    sig: 'sha256:c91d…4b2a'
  }, {
    name: 'kernel-policy',
    gal: 4,
    state: 'promoting',
    sig: 'sha256:77e0…9f3c'
  }, {
    name: 'kernel-rekor',
    gal: 3,
    state: 'reviewed',
    sig: 'sha256:12ab…8e5d'
  }, {
    name: 'kernel-sandbox',
    gal: 2,
    state: 'drafted',
    sig: 'sha256:3f09…d72b'
  }];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '24px 28px',
      display: 'grid',
      gridTemplateColumns: '1fr 320px',
      gap: 20
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '10px 16px',
      background: 'color-mix(in oklab, var(--ck-accent) 12%, transparent)',
      boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-accent) 45%, transparent)',
      clipPath: 'polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px)'
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "ck-invariant",
    style: {
      fontSize: 11
    }
  }, "mode changes deployment, not governance semantics")), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'baseline',
      marginBottom: 14
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: 0,
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Proof chain"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em'
    }
  }, "PCA \u2192 UCA \u2192 REKOR")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: '4px 6px'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(Diamond, {
    size: 64
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)',
      marginTop: 6,
      letterSpacing: '.04em'
    }
  }, "PCA"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "provenance")), /*#__PURE__*/React.createElement(ArrowAttested, {
    width: 160
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(Diamond, {
    size: 64
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)',
      marginTop: 6,
      letterSpacing: '.04em'
    }
  }, "UCA"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "usage")), /*#__PURE__*/React.createElement(ArrowAttested, {
    width: 160
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      textAlign: 'center'
    }
  }, /*#__PURE__*/React.createElement(Octagon, {
    size: 64
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)',
      marginTop: 6,
      letterSpacing: '.04em'
    }
  }, "REKOR"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "anchor")))), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'baseline',
      marginBottom: 12
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: 0,
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Kernels"), /*#__PURE__*/React.createElement(CkButton, {
    variant: "ghost",
    style: {
      padding: '6px 10px'
    }
  }, "\u22A2 Attest all")), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr 90px 130px 1fr 80px',
      gap: 10,
      font: "700 9px 'Lato', sans-serif",
      color: 'var(--ck-fg-3)',
      letterSpacing: '.14em',
      textTransform: 'uppercase',
      padding: '6px 0',
      borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 20%, transparent)'
    }
  }, /*#__PURE__*/React.createElement("span", null, "Name"), /*#__PURE__*/React.createElement("span", null, "GAL"), /*#__PURE__*/React.createElement("span", null, "State"), /*#__PURE__*/React.createElement("span", null, "Signature"), /*#__PURE__*/React.createElement("span", null)), kernels.map(k => /*#__PURE__*/React.createElement("div", {
    key: k.name,
    style: {
      display: 'grid',
      gridTemplateColumns: '1fr 90px 130px 1fr 80px',
      gap: 10,
      alignItems: 'center',
      padding: '10px 0',
      borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 10%, transparent)'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      font: "600 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, k.name), /*#__PURE__*/React.createElement(GalRail, {
    level: k.gal,
    attested: k.state === 'attested',
    width: 72
  }), /*#__PURE__*/React.createElement(Badge, {
    kind: k.state === 'attested' ? 'attested' : k.state === 'promoting' ? 'witness' : 'kernel'
  }, k.state === 'attested' ? '⊢' : k.state === 'promoting' ? '⇝' : '◐', " ", k.state), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "400 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-3)'
    }
  }, k.sig), /*#__PURE__*/React.createElement(CkButton, {
    variant: "ghost",
    style: {
      padding: '6px 10px',
      fontSize: 9
    }
  }, "inspect \u2192"))))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement(Bento, {
    style: {
      textAlign: 'center',
      paddingTop: 20,
      paddingBottom: 20
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 12
    }
  }, "Focal anchor"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'center'
    }
  }, /*#__PURE__*/React.createElement(Orb, {
    size: 180
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em',
      marginTop: 12
    }
  }, "policy surface \xB7 stable")), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 10
    }
  }, "Recent edges"), [{
    glyph: '⊢',
    t: 'attested',
    k: 'kernel-core',
    time: '14:12'
  }, {
    glyph: '⇝',
    t: 'promoting',
    k: 'kernel-policy',
    time: '14:08'
  }, {
    glyph: '⊘',
    t: 'denied',
    k: 'kernel-sandbox',
    time: '14:01',
    deny: true
  }, {
    glyph: '⊛',
    t: 'witnessed',
    k: 'kernel-attest',
    time: '13:54'
  }].map((e, i) => /*#__PURE__*/React.createElement("div", {
    key: i,
    style: {
      display: 'grid',
      gridTemplateColumns: '16px 1fr 50px',
      gap: 10,
      alignItems: 'center',
      padding: '6px 0',
      borderBottom: i < 3 ? '1px solid color-mix(in oklab, var(--ck-stroke) 10%, transparent)' : 'none'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      font: "600 12px 'JetBrains Mono', monospace",
      color: e.deny ? 'var(--ck-deny)' : 'var(--ck-accent)'
    }
  }, e.glyph), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, e.t, " ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-mute)'
    }
  }, "\xB7 ", e.k)), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      textAlign: 'right'
    }
  }, e.time))))));
};
Object.assign(window, {
  Overview
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/Overview.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/PolicyView.jsx
try { (() => {
// Policy view — FSM modes + open gates
const PolicyView = () => {
  const modes = [{
    m: 'M0',
    name: 'sealed',
    desc: 'no deployment · governance frozen',
    active: false
  }, {
    m: 'M1',
    name: 'draft',
    desc: 'local only · no external edges',
    active: false
  }, {
    m: 'M2',
    name: 'review',
    desc: 'witnessed · staging deployment',
    active: false
  }, {
    m: 'M3',
    name: 'promotion',
    desc: 'attested · production eligible',
    active: true
  }];
  const gates = [{
    id: 'gate-7712',
    target: 'kernel-policy',
    state: 'open',
    glyph: '◐',
    kind: 'witness'
  }, {
    id: 'gate-7708',
    target: 'kernel-core',
    state: 'passed',
    glyph: '⊢',
    kind: 'attested'
  }, {
    id: 'gate-7705',
    target: 'kernel-sandbox',
    state: 'denied',
    glyph: '⊘',
    kind: 'deny'
  }];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '24px 28px',
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      gap: 20
    }
  }, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: '0 0 4px',
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Governance modes"), /*#__PURE__*/React.createElement("div", {
    className: "ck-invariant",
    style: {
      fontSize: 10,
      marginBottom: 14
    }
  }, "mode changes deployment, not governance semantics"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 10
    }
  }, modes.map(mo => /*#__PURE__*/React.createElement("div", {
    key: mo.m,
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 14,
      padding: '10px 12px',
      background: mo.active ? 'color-mix(in oklab, var(--ck-accent) 12%, transparent)' : 'transparent',
      boxShadow: mo.active ? 'inset 0 0 0 1px color-mix(in oklab, var(--ck-accent) 45%, transparent)' : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 20%, transparent)',
      clipPath: 'polygon(8px 0,calc(100% - 8px) 0,100% 8px,100% calc(100% - 8px),calc(100% - 8px) 100%,8px 100%,0 calc(100% - 8px),0 8px)'
    }
  }, /*#__PURE__*/React.createElement(FsmFlag, {
    size: 36,
    mode: mo.m
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, mo.name), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "400 11px 'Lato', sans-serif",
      color: 'var(--ck-fg-2)'
    }
  }, mo.desc)), mo.active && /*#__PURE__*/React.createElement(Badge, {
    kind: "attested"
  }, "\u22A2 active"))))), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: '0 0 14px',
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Gates"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 12
    }
  }, gates.map(g => /*#__PURE__*/React.createElement("div", {
    key: g.id,
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 14,
      padding: '4px 0',
      borderBottom: '1px solid color-mix(in oklab, var(--ck-stroke) 12%, transparent)'
    }
  }, /*#__PURE__*/React.createElement(Diamond, {
    size: 48,
    nucleusFilled: g.state === 'passed',
    gated: false
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      flex: 1
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, g.id), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "400 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "\u2192 ", g.target)), /*#__PURE__*/React.createElement(Badge, {
    kind: g.kind
  }, g.glyph, " ", g.state)))), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 16,
      display: 'flex',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement(CkButton, {
    variant: "primary"
  }, "\u22A2 Compile policy"), /*#__PURE__*/React.createElement(CkButton, {
    variant: "ghost"
  }, "\u2200 Dry-run"))));
};
Object.assign(window, {
  PolicyView
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/PolicyView.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/RekorView.jsx
try { (() => {
// Rekor log view — append-only anchor entries with proof-chain spine
const RekorView = () => {
  const entries = [{
    idx: 4812,
    ts: '14:12:07',
    kind: 'attested',
    art: 'kernel-core@1.4.0',
    hash: 'a42f…c01e',
    glyph: '⊢',
    kc: 'attested'
  }, {
    idx: 4811,
    ts: '14:11:52',
    kind: 'promotion',
    art: 'kernel-policy@1.4.0',
    hash: '77e0…9f3c',
    glyph: '⇝',
    kc: 'witness'
  }, {
    idx: 4810,
    ts: '14:08:31',
    kind: 'witnessed',
    art: 'guard-eval-pack@v3',
    hash: '12ab…8e5d',
    glyph: '⊛',
    kc: 'witness'
  }, {
    idx: 4809,
    ts: '14:02:18',
    kind: 'attested',
    art: 'triage-lora@r12',
    hash: '3f09…d72b',
    glyph: '⊢',
    kc: 'attested'
  }, {
    idx: 4808,
    ts: '13:54:09',
    kind: 'denied',
    art: 'sandbox-runner@dev',
    hash: '5e71…b0c4',
    glyph: '⊘',
    kc: 'deny'
  }, {
    idx: 4807,
    ts: '13:48:44',
    kind: 'sealed',
    art: 'kernel-attest@1.3.9',
    hash: '9c4e…77a1',
    glyph: '⊗',
    kc: 'mute'
  }];
  return /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '24px 28px',
      display: 'grid',
      gridTemplateColumns: '1fr 260px',
      gap: 20
    }
  }, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'baseline',
      marginBottom: 14
    }
  }, /*#__PURE__*/React.createElement("h3", {
    style: {
      margin: 0,
      font: "500 16px 'Roboto', sans-serif",
      color: 'var(--ck-fg-1)'
    }
  }, "Anchor log"), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em'
    }
  }, "append-only \xB7 depth 8")), /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'relative'
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      position: 'absolute',
      left: 7,
      top: 8,
      bottom: 8,
      width: 2,
      background: 'color-mix(in oklab, var(--ck-stroke) 35%, transparent)'
    }
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 2
    }
  }, entries.map((e, i) => /*#__PURE__*/React.createElement("div", {
    key: e.idx,
    style: {
      display: 'grid',
      gridTemplateColumns: '16px 64px 1fr 130px',
      gap: 14,
      alignItems: 'center',
      padding: '10px 0',
      borderBottom: i < entries.length - 1 ? '1px solid color-mix(in oklab, var(--ck-stroke) 10%, transparent)' : 'none'
    }
  }, /*#__PURE__*/React.createElement("span", {
    style: {
      width: 16,
      height: 16,
      display: 'grid',
      placeItems: 'center',
      zIndex: 1,
      background: 'var(--ck-bg-1)',
      font: "600 11px 'JetBrains Mono', monospace",
      color: e.kc === 'deny' ? 'var(--ck-deny)' : e.kc === 'mute' ? 'var(--ck-fg-mute)' : 'var(--ck-accent)'
    }
  }, e.glyph), /*#__PURE__*/React.createElement("span", {
    style: {
      font: "600 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, "#", e.idx), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, e.art), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "400 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)'
    }
  }, "sha256:", e.hash, " \xB7 ", e.ts, " utc")), /*#__PURE__*/React.createElement("div", {
    style: {
      justifySelf: 'end'
    }
  }, /*#__PURE__*/React.createElement(Badge, {
    kind: e.kc
  }, e.glyph, " ", e.kind))))))), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }
  }, /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 12
    }
  }, "Tree head"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "900 28px 'Roboto', sans-serif",
      color: 'var(--ck-accent)',
      lineHeight: 1
    }
  }, "4,812"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      marginTop: 4
    }
  }, "entries \xB7 depth 8"), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 14,
      font: "400 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-2)',
      wordBreak: 'break-all'
    }
  }, "root ", /*#__PURE__*/React.createElement("span", {
    style: {
      color: 'var(--ck-fg-mute)'
    }
  }, "sha256:"), /*#__PURE__*/React.createElement("br", null), "b8d3f1a9\u202604e72c5d")), /*#__PURE__*/React.createElement(Bento, null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      textTransform: 'uppercase',
      marginBottom: 10
    }
  }, "Consistency"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      font: "600 12px 'JetBrains Mono', monospace",
      color: 'var(--ck-accent)'
    }
  }, /*#__PURE__*/React.createElement("span", null, "\u220E"), " proof verified"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "400 10px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      marginTop: 6
    }
  }, "last checkpoint 14:12 utc"))));
};
Object.assign(window, {
  RekorView
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/RekorView.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/Sidebar.jsx
try { (() => {
// Sidebar — primary chrome
const Sidebar = ({
  current,
  onNavigate
}) => {
  const items = [{
    id: 'overview',
    label: 'Overview',
    glyph: '◈'
  }, {
    id: 'kernels',
    label: 'Kernels',
    glyph: '▰'
  }, {
    id: 'attest',
    label: 'Attestation',
    glyph: '◇'
  }, {
    id: 'policy',
    label: 'Policy',
    glyph: '⊢'
  }, {
    id: 'artifacts',
    label: 'Artifacts',
    glyph: '⊗'
  }, {
    id: 'rekor',
    label: 'Rekor log',
    glyph: '≡'
  }];
  return /*#__PURE__*/React.createElement("aside", {
    style: {
      width: 220,
      flexShrink: 0,
      background: 'var(--ck-bg-0)',
      borderRight: '1px solid color-mix(in oklab, var(--ck-stroke) 25%, transparent)',
      padding: '20px 14px',
      display: 'flex',
      flexDirection: 'column',
      gap: 4
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      padding: '0 8px 18px'
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/logos/mark-a2-favicon.svg",
    width: "26",
    height: "26",
    alt: ""
  }), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "900 14px 'Roboto', sans-serif",
      letterSpacing: '.02em',
      color: 'var(--ck-fg-1)'
    }
  }, "CKODEX"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.1em'
    }
  }, "DS-1 v1.4.0"))), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      padding: '8px 8px 6px',
      textTransform: 'uppercase'
    }
  }, "Surface"), items.map(it => {
    const active = current === it.id;
    return /*#__PURE__*/React.createElement("button", {
      key: it.id,
      onClick: () => onNavigate(it.id),
      style: {
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        padding: '8px 10px',
        background: active ? 'var(--ck-bg-1)' : 'transparent',
        color: active ? 'var(--ck-fg-1)' : 'var(--ck-fg-2)',
        border: 'none',
        cursor: 'pointer',
        font: "500 12px 'Lato', sans-serif",
        textAlign: 'left',
        clipPath: 'polygon(6px 0, calc(100% - 6px) 0, 100% 6px, 100% calc(100% - 6px), calc(100% - 6px) 100%, 6px 100%, 0 calc(100% - 6px), 0 6px)',
        boxShadow: active ? 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 45%, transparent)' : 'none'
      }
    }, /*#__PURE__*/React.createElement("span", {
      style: {
        font: "500 13px 'JetBrains Mono', monospace",
        color: active ? 'var(--ck-accent)' : 'var(--ck-fg-3)',
        width: 14,
        textAlign: 'center'
      }
    }, it.glyph), /*#__PURE__*/React.createElement("span", null, it.label), active && /*#__PURE__*/React.createElement("span", {
      style: {
        marginLeft: 'auto',
        font: "600 8px 'JetBrains Mono', monospace",
        color: 'var(--ck-fg-mute)',
        letterSpacing: '.08em'
      }
    }, "\u22A2"));
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      marginTop: 'auto',
      paddingTop: 18
    }
  }, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "700 10px 'Lato', sans-serif",
      letterSpacing: '.14em',
      color: 'var(--ck-fg-3)',
      padding: '0 8px 6px',
      textTransform: 'uppercase'
    }
  }, "Mode"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 10,
      padding: '8px 10px'
    }
  }, /*#__PURE__*/React.createElement(FsmFlag, {
    size: 34,
    mode: "M3"
  }), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    style: {
      font: "600 11px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-1)'
    }
  }, "M3 \xB7 promotion"), /*#__PURE__*/React.createElement("div", {
    style: {
      font: "500 9px 'JetBrains Mono', monospace",
      color: 'var(--ck-fg-mute)',
      letterSpacing: '.04em'
    }
  }, "since 14:02 UTC")))));
};
Object.assign(window, {
  Sidebar
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/Sidebar.jsx", error: String((e && e.message) || e) }); }

// ui_kits/governance-console/primitives.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
// Bentography primitives · inline JSX (loaded as text/babel)
// Chamfered bento, diamond gate, octagon kernel, 5-sided FSM flag, 6-segment
// GAL rail, five typed arrows, the Orb focal anchor.

const Bento = ({
  stroked = true,
  children,
  style,
  pad = 18,
  ...rest
}) => {
  const c = 10;
  const clip = `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;
  const ring = stroked ? 'inset 0 0 0 2.5px var(--ck-stroke), inset 0 0 0 4.5px color-mix(in oklab, var(--ck-stroke) 28%, transparent)' : 'none';
  return /*#__PURE__*/React.createElement("div", _extends({}, rest, {
    style: {
      background: 'var(--ck-bg-1)',
      clipPath: clip,
      boxShadow: ring,
      padding: pad,
      position: 'relative',
      ...style
    }
  }), children);
};
const Diamond = ({
  size = 96,
  gated = true,
  label,
  nucleusFilled = true
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 96 96"
}, /*#__PURE__*/React.createElement("polygon", {
  points: "48,6 90,48 48,90 6,48",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "2.5"
}), gated && /*#__PURE__*/React.createElement("polygon", {
  points: "48,18 78,48 48,78 18,48",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "0.75",
  opacity: ".6"
}), nucleusFilled && /*#__PURE__*/React.createElement("polygon", {
  points: "48,28 68,48 48,68 28,48",
  fill: "var(--ck-accent)"
}), /*#__PURE__*/React.createElement("circle", {
  cx: "48",
  cy: "48",
  r: "4",
  fill: "var(--ck-deep-blue)"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "0",
  x2: "48",
  y2: "5",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "91",
  y1: "48",
  x2: "96",
  y2: "48",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "91",
  x2: "48",
  y2: "96",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "48",
  x2: "3",
  y2: "48",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), label && /*#__PURE__*/React.createElement("text", {
  x: "48",
  y: "100",
  textAnchor: "middle",
  fill: "var(--ck-fg-mute)",
  fontFamily: "JetBrains Mono",
  fontSize: "8",
  letterSpacing: ".04em"
}, label));
const Octagon = ({
  size = 96,
  children
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 96 96"
}, /*#__PURE__*/React.createElement("path", {
  d: "M20,6 H76 L90,20 V76 L76,90 H20 L6,76 V20 Z",
  fill: "var(--ck-bg-1)",
  stroke: "var(--ck-stroke)",
  strokeWidth: "2.5"
}), /*#__PURE__*/React.createElement("path", {
  d: "M24,10 H72 L86,24 V72 L72,86 H24 L10,72 V24 Z",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1",
  opacity: ".45"
}), children, /*#__PURE__*/React.createElement("line", {
  x1: "36",
  y1: "90",
  x2: "36",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "48",
  y1: "90",
  x2: "48",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "90",
  x2: "60",
  y2: "94",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}));
const FsmFlag = ({
  size = 56,
  mode = 'M3'
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 96 96"
}, /*#__PURE__*/React.createElement("polygon", {
  points: "48,8 82,28 72,78 24,78 14,28",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "2.5"
}), /*#__PURE__*/React.createElement("polygon", {
  points: "48,22 70,36 63,66 33,66 26,36",
  fill: "var(--ck-witness)",
  opacity: ".3"
}), /*#__PURE__*/React.createElement("text", {
  x: "48",
  y: "54",
  textAnchor: "middle",
  fill: "var(--ck-fg-1)",
  fontFamily: "JetBrains Mono",
  fontSize: "14",
  fontWeight: "700"
}, mode));

// GAL tick rail · 6 <rect>s, never Unicode rail glyphs
const GalRail = ({
  level = 3,
  width = 96,
  attested = false
}) => {
  const slots = Array.from({
    length: 6
  });
  return /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'grid',
      gridTemplateColumns: 'repeat(6, 1fr)',
      gap: 2,
      width,
      height: 8
    }
  }, slots.map((_, i) => /*#__PURE__*/React.createElement("i", {
    key: i,
    style: {
      background: attested ? 'var(--ck-accent)' : 'var(--ck-rail)',
      opacity: i < level ? 1 : 0.22
    }
  })));
};

// Typed arrow · five variants
const ArrowKernelFlow = ({
  width = 120
}) => /*#__PURE__*/React.createElement("svg", {
  width: width,
  height: "14",
  viewBox: `0 0 ${width} 14`
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "5",
  x2: width,
  y2: "5",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "9",
  x2: width,
  y2: "9",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}));
const ArrowAttested = ({
  width = 160
}) => /*#__PURE__*/React.createElement("svg", {
  width: width,
  height: "16",
  viewBox: `0 0 ${width} 16`
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "6",
  x2: width - 12,
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "10",
  x2: width - 12,
  y2: "10",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), [0.25, 0.5, 0.75].map((f, i) => /*#__PURE__*/React.createElement("line", {
  key: i,
  x1: width * f,
  y1: "2",
  x2: width * f,
  y2: "14",
  stroke: "var(--ck-accent)",
  strokeWidth: "1.25"
})), /*#__PURE__*/React.createElement("polygon", {
  points: `${width - 12},1 ${width},8 ${width - 12},15`,
  fill: "var(--ck-stroke)"
}));
const ArrowDeny = ({
  width = 120
}) => /*#__PURE__*/React.createElement("svg", {
  width: width,
  height: "16",
  viewBox: `0 0 ${width} 16`
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "8",
  x2: width - 14,
  y2: "8",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("line", {
  x1: width - 14,
  y1: "2",
  x2: width - 2,
  y2: "14",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.5"
}), /*#__PURE__*/React.createElement("line", {
  x1: width - 2,
  y1: "2",
  x2: width - 14,
  y2: "14",
  stroke: "var(--ck-deny)",
  strokeWidth: "1.5"
}));
const ArrowPromotion = ({
  width = 120
}) => /*#__PURE__*/React.createElement("svg", {
  width: width,
  height: "16",
  viewBox: `0 0 ${width} 16`
}, /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "8",
  x2: width - 10,
  y2: "8",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25"
}), /*#__PURE__*/React.createElement("polyline", {
  points: `${width - 16},2 ${width - 4},8 ${width - 16},14`,
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25",
  fill: "none"
}));

// The Orb · single focal anchor, breathing
const Orb = ({
  size = 160
}) => /*#__PURE__*/React.createElement("svg", {
  width: size,
  height: size,
  viewBox: "0 0 120 120"
}, /*#__PURE__*/React.createElement("circle", {
  className: "ck-orb__ring",
  cx: "60",
  cy: "60",
  r: "52",
  fill: "none",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1.25",
  opacity: ".5"
}), /*#__PURE__*/React.createElement("circle", {
  className: "ck-orb__ring",
  cx: "60",
  cy: "60",
  r: "36",
  fill: "none",
  stroke: "var(--ck-witness)",
  strokeWidth: "1",
  opacity: ".65",
  style: {
    animationDelay: '-0.6s'
  }
}), /*#__PURE__*/React.createElement("circle", {
  className: "ck-orb__ring",
  cx: "60",
  cy: "60",
  r: "20",
  fill: "none",
  stroke: "var(--ck-accent)",
  strokeWidth: "1",
  style: {
    animationDelay: '-1.2s'
  }
}), /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "0",
  x2: "60",
  y2: "6",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "60",
  y1: "114",
  x2: "60",
  y2: "120",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "0",
  y1: "60",
  x2: "6",
  y2: "60",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("line", {
  x1: "114",
  y1: "60",
  x2: "120",
  y2: "60",
  stroke: "var(--ck-stroke)",
  strokeWidth: "1"
}), /*#__PURE__*/React.createElement("circle", {
  cx: "60",
  cy: "60",
  r: "3",
  fill: "var(--ck-accent)"
}));

// Badge (chamfered, safe-set glyph only)
const Badge = ({
  kind = 'kernel',
  children
}) => {
  const palette = {
    attested: {
      bg: 'var(--ck-accent)',
      fg: 'var(--ck-deep-blue)',
      ring: 'transparent'
    },
    kernel: {
      bg: 'transparent',
      fg: 'var(--ck-stroke)',
      ring: 'var(--ck-stroke)'
    },
    witness: {
      bg: 'transparent',
      fg: 'var(--ck-witness)',
      ring: 'var(--ck-witness)'
    },
    deny: {
      bg: 'transparent',
      fg: 'var(--ck-deny)',
      ring: 'var(--ck-deny)'
    },
    mute: {
      bg: 'transparent',
      fg: 'var(--ck-fg-mute)',
      ring: 'var(--ck-fg-mute)'
    }
  }[kind];
  const c = 5;
  const clip = `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;
  return /*#__PURE__*/React.createElement("span", {
    style: {
      display: 'inline-flex',
      alignItems: 'center',
      gap: 5,
      font: "700 10px 'JetBrains Mono', monospace",
      letterSpacing: '.08em',
      textTransform: 'uppercase',
      padding: '4px 10px',
      background: palette.bg,
      color: palette.fg,
      clipPath: clip,
      boxShadow: palette.ring === 'transparent' ? 'none' : `inset 0 0 0 1.5px ${palette.ring}`
    }
  }, children);
};

// Button · chamfered, no pill
const CkButton = ({
  variant = 'primary',
  children,
  onClick,
  style
}) => {
  const c = 8;
  const clip = `polygon(${c}px 0, calc(100% - ${c}px) 0, 100% ${c}px, 100% calc(100% - ${c}px), calc(100% - ${c}px) 100%, ${c}px 100%, 0 calc(100% - ${c}px), 0 ${c}px)`;
  const palette = {
    primary: {
      bg: 'var(--ck-accent)',
      fg: 'var(--ck-deep-blue)',
      ring: 'transparent'
    },
    secondary: {
      bg: 'transparent',
      fg: 'var(--ck-fg-1)',
      ring: 'var(--ck-stroke)'
    },
    ghost: {
      bg: 'transparent',
      fg: 'var(--ck-stroke)',
      ring: 'transparent'
    },
    deny: {
      bg: 'transparent',
      fg: 'var(--ck-deny)',
      ring: 'var(--ck-deny)'
    }
  }[variant];
  return /*#__PURE__*/React.createElement("button", {
    onClick: onClick,
    style: {
      font: "700 11px 'Lato', sans-serif",
      letterSpacing: '.1em',
      textTransform: 'uppercase',
      padding: '10px 16px',
      border: 'none',
      cursor: 'pointer',
      background: palette.bg,
      color: palette.fg,
      clipPath: clip,
      boxShadow: palette.ring === 'transparent' ? 'none' : `inset 0 0 0 2px ${palette.ring}`,
      ...style
    }
  }, children);
};
Object.assign(window, {
  Bento,
  Diamond,
  Octagon,
  FsmFlag,
  GalRail,
  ArrowKernelFlow,
  ArrowAttested,
  ArrowDeny,
  ArrowPromotion,
  Orb,
  Badge,
  CkButton
});
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/governance-console/primitives.jsx", error: String((e && e.message) || e) }); }

__ds_ns.Button = __ds_scope.Button;

__ds_ns.EvidenceHash = __ds_scope.EvidenceHash;

__ds_ns.EvidenceMargin = __ds_scope.EvidenceMargin;

__ds_ns.PageShell = __ds_scope.PageShell;

__ds_ns.ProvenanceStamp = __ds_scope.ProvenanceStamp;

__ds_ns.QuietCard = __ds_scope.QuietCard;

__ds_ns.StateChip = __ds_scope.StateChip;

})();
