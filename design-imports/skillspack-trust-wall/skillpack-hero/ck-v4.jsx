// ============================================================
// CKODEX-DS-3 · v4 components ported for the browser (no bundler)
// CkIcon · EntityCard · CkCiqr (real QR encoder) + CiqrPath.
// Source: DS component .jsx — `export` stripped, globals assigned.
// Requires: React, styles.css, ck-v4.css.
// ============================================================

// ---------- CkIcon ----------
const CK_ICON_PATHS = {
  kernel:
    '<path d="M8.5 3 H15.5 L21 8.5 V15.5 L15.5 21 H8.5 L3 15.5 V8.5 Z"/>' +
    '<circle cx="12" cy="12" r="2" fill="currentColor" stroke="none"/>',
  gate: '<path d="M4.5 6 H19.5"/><path d="M7 6 V20"/><path d="M17 6 V20"/><path d="M7 13 H17"/>',
  margin: '<path d="M4 4 H20 V20 H4 Z"/><path d="M14.5 4 V20"/><path d="M16.4 8 H18.6"/><path d="M16.4 12 H18.6"/>',
  proof: '<path d="M12 2.5 L21.5 12 L12 21.5 L2.5 12 Z"/><path d="M6.6 6.6 L9.2 9.2"/>',
  chain:
    '<path d="M6 8.5 L9.5 12 L6 15.5 L2.5 12 Z"/><path d="M12 8.5 L15.5 12 L12 15.5 L8.5 12 Z"/><path d="M18 8.5 L21.5 12 L18 15.5 L14.5 12 Z"/>',
  digest: '<path d="M4 5 H20 V19 H4 Z"/><path d="M8 9.5 H16"/><path d="M8 12 H16"/><path d="M8 14.5 H16"/>',
  policy:
    '<path d="M5.5 3.5 H18.5 V20.5 H5.5 Z"/><path d="M8 8 H16"/><path d="M8 12 H15"/><path d="M8 15.5 H13"/><circle cx="15.8" cy="17.4" r="1.2" fill="currentColor" stroke="none"/>',
  receipt: '<path d="M5 4.5 V19.5"/><path d="M8 8 H18"/><path d="M8 12 H16"/><path d="M8 16 H12"/>',
  disclosure: '<path d="M4 8 H20 V20 H4 Z"/><path d="M4 8 L12 3 L20 8"/><path d="M8 13.5 H16"/>',
  export: '<path d="M4 9 H13 V20 H4 Z"/><path d="M11 13 L21 3"/><path d="M15 3 H21 V9"/>',
  replay: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7 V12 L15.5 14"/><path d="M5.7 7.4 L3.6 9 L5.9 10.4"/>',
  quarantine:
    '<path d="M4 8 V4 H8"/><path d="M16 4 H20 V8"/><path d="M20 16 V20 H16"/><path d="M8 20 H4 V16"/><circle cx="12" cy="12" r="2" fill="currentColor" stroke="none"/>',
  audit:
    '<path d="M6 4 V20"/><circle cx="6" cy="8" r="1.5" fill="currentColor" stroke="none"/><circle cx="6" cy="13" r="1.5" fill="currentColor" stroke="none"/><circle cx="6" cy="18" r="1.5" fill="currentColor" stroke="none"/><path d="M9.5 8 H18"/><path d="M9.5 13 H16"/><path d="M9.5 18 H14"/>',
  gauge: '<path d="M4 16 A8 8 0 0 1 20 16"/><path d="M12 16 L16.5 9.5"/><circle cx="12" cy="16" r="1.5" fill="currentColor" stroke="none"/>',
  matrix:
    '<path d="M4 4 H20 V20 H4 Z"/><path d="M4 9.33 H20"/><path d="M4 14.66 H20"/><path d="M9.33 4 V20"/><path d="M14.66 4 V20"/><rect x="14.66" y="4" width="5.34" height="5.33" fill="currentColor" stroke="none"/>',
  trend: '<path d="M4 19 H20"/><path d="M4 16 L8 11 L12 13 L16 6 L20 9"/><circle cx="16" cy="6" r="1.4" fill="currentColor" stroke="none"/>',
  system:
    '<path d="M3.5 3.5 H20.5 V20.5 H3.5 Z"/><path d="M8.5 8.5 H15.5 V15.5 H8.5 Z"/><path d="M12 3.5 V8.5"/><path d="M12 15.5 V20.5"/><path d="M3.5 12 H8.5"/><path d="M15.5 12 H20.5"/>',
  agent: '<circle cx="12" cy="12" r="8.5"/><path d="M12 12 L17.3 6.7"/><circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none"/>',
  skill: '<path d="M5 8.5 L12 4.5 L19 8.5"/><path d="M5 13.25 L12 9.25 L19 13.25"/><path d="M5 18 L12 14 L19 18"/>',
  model: '<path d="M4 4 H20 V20 H4 Z"/><path d="M4 4 L20 20"/><path d="M12 4 L20 12"/><path d="M4 12 L12 20"/>',
  aipack: '<path d="M4 6.5 H20 V20 H4 Z"/><path d="M4 10.5 H20"/><path d="M12 10.5 V20"/><circle cx="12" cy="8.5" r="1.2" fill="currentColor" stroke="none"/>',
  guardrail: '<path d="M3.5 8.5 H20.5"/><path d="M3.5 13 H20.5"/><path d="M6.5 8.5 V20"/><path d="M12 8.5 V20"/><path d="M17.5 8.5 V20"/>',
};
function CkIcon({ name, size = 20, strokeWidth = 1.5, label, className = '', style, ...rest }) {
  const inner = CK_ICON_PATHS[name];
  const cls = ('ck-icon ck-icon--' + name + (className ? ' ' + className : '')).trim();
  const a11y = label ? { role: 'img', 'aria-label': label } : { 'aria-hidden': 'true' };
  return React.createElement('svg', {
    className: cls, width: size, height: size, viewBox: '0 0 24 24', fill: 'none',
    stroke: 'currentColor', strokeWidth, strokeLinecap: 'square', strokeLinejoin: 'miter',
    style, ...a11y, ...rest, dangerouslySetInnerHTML: { __html: inner || '' },
  });
}

// ---------- CkCiqr — real QR (byte mode, EC-L, v1–5, mask 0) ----------
const CIQR_EXP = new Uint8Array(512), CIQR_LOG = new Uint8Array(256);
(function () { let x = 1; for (let i = 0; i < 255; i++) { CIQR_EXP[i] = x; CIQR_LOG[x] = i; x <<= 1; if (x & 0x100) x ^= 0x11d; } for (let i = 255; i < 512; i++) CIQR_EXP[i] = CIQR_EXP[i - 255]; })();
function ciqrPolyMul(a, b) { const r = new Array(a.length + b.length - 1).fill(0); for (let i = 0; i < a.length; i++) for (let j = 0; j < b.length; j++) if (a[i] && b[j]) r[i + j] ^= CIQR_EXP[(CIQR_LOG[a[i]] + CIQR_LOG[b[j]]) % 255]; return r; }
function ciqrRS(data, ecLen) { let gen = [1]; for (let i = 0; i < ecLen; i++) gen = ciqrPolyMul(gen, [1, CIQR_EXP[i]]); const res = data.concat(new Array(ecLen).fill(0)); for (let i = 0; i < data.length; i++) { const f = res[i]; if (f === 0) continue; const lf = CIQR_LOG[f]; for (let j = 0; j < gen.length; j++) res[i + j] ^= CIQR_EXP[(lf + CIQR_LOG[gen[j]]) % 255]; } return res.slice(data.length); }
function ciqrFormatBits(maskId) { const data = (0b01 << 3) | maskId; const G = 0b10100110111; const blen = (v) => { let l = 0; while (v) { l++; v >>>= 1; } return l; }; let d = data << 10; while (blen(d) >= blen(G)) d ^= G << (blen(d) - blen(G)); return ((data << 10) | d) ^ 0b101010000010010; }
const CIQR_VERSIONS = [{ v: 1, data: 19, ec: 7, align: 0 }, { v: 2, data: 34, ec: 10, align: 18 }, { v: 3, data: 55, ec: 15, align: 22 }, { v: 4, data: 80, ec: 20, align: 26 }, { v: 5, data: 108, ec: 26, align: 30 }];
function ciqrMatrix(payload) {
  const bytes = new TextEncoder().encode(String(payload));
  const V = CIQR_VERSIONS.find((t) => bytes.length <= t.data - 2); if (!V) return null;
  const size = 17 + 4 * V.v; const bits = [];
  const push = (val, n) => { for (let i = n - 1; i >= 0; i--) bits.push((val >> i) & 1); };
  push(4, 4); push(bytes.length, 8); bytes.forEach((b) => push(b, 8));
  push(0, Math.min(4, V.data * 8 - bits.length)); while (bits.length % 8) bits.push(0);
  const data = []; for (let i = 0; i < bits.length; i += 8) { let b = 0; for (let j = 0; j < 8; j++) b = (b << 1) | bits[i + j]; data.push(b); }
  const PADS = [0xec, 0x11]; let pi = 0; while (data.length < V.data) data.push(PADS[pi++ % 2]);
  const cw = data.concat(ciqrRS(data, V.ec));
  const m = Array.from({ length: size }, () => new Array(size).fill(null));
  const finder = (r0, c0) => { for (let r = -1; r <= 7; r++) for (let c = -1; c <= 7; c++) { const rr = r0 + r, cc = c0 + c; if (rr < 0 || rr >= size || cc < 0 || cc >= size) continue; m[rr][cc] = (r >= 0 && r <= 6 && (c === 0 || c === 6)) || (c >= 0 && c <= 6 && (r === 0 || r === 6)) || (r >= 2 && r <= 4 && c >= 2 && c <= 4); } };
  finder(0, 0); finder(size - 7, 0); finder(0, size - 7);
  for (let i = 8; i < size - 8; i++) { if (m[i][6] === null) m[i][6] = i % 2 === 0; if (m[6][i] === null) m[6][i] = i % 2 === 0; }
  if (V.align) { const p = V.align; for (let r = -2; r <= 2; r++) for (let c = -2; c <= 2; c++) m[p + r][p + c] = Math.max(Math.abs(r), Math.abs(c)) !== 1; }
  const fmt = ciqrFormatBits(0);
  for (let i = 0; i < 15; i++) { const mod = ((fmt >> i) & 1) === 1; if (i < 6) m[i][8] = mod; else if (i < 8) m[i + 1][8] = mod; else m[size - 15 + i][8] = mod; if (i < 8) m[8][size - i - 1] = mod; else if (i < 9) m[8][15 - i] = mod; else m[8][14 - i] = mod; }
  m[size - 8][8] = true;
  let inc = -1, row = size - 1, bitIndex = 7, byteIndex = 0;
  for (let col = size - 1; col > 0; col -= 2) { if (col === 6) col--; while (true) { for (let c = 0; c < 2; c++) { if (m[row][col - c] === null) { let dark = false; if (byteIndex < cw.length) dark = ((cw[byteIndex] >>> bitIndex) & 1) === 1; if ((row + (col - c)) % 2 === 0) dark = !dark; m[row][col - c] = dark; bitIndex--; if (bitIndex === -1) { byteIndex++; bitIndex = 7; } } } row += inc; if (row < 0 || row >= size) { row -= inc; inc = -inc; break; } } }
  return { m, size };
}
function CiqrPath(payload) {
  const q = ciqrMatrix(payload); if (!q) return null;
  let path = ''; for (let r = 0; r < q.size; r++) for (let c = 0; c < q.size; c++) if (q.m[r][c]) path += 'M' + c + ' ' + r + 'h1v1h-1z';
  return { path, modules: q.size };
}
function CkCiqr({ payload, size = 110, attested = false, label = 'CIQR', caption = 'scan to resolve canonical evidence', className = '', ...rest }) {
  const q = React.useMemo(() => CiqrPath(payload), [payload]);
  return (
    <figure className={('ck-ciqr' + (attested ? ' ck-ciqr--attested' : '') + (className ? ' ' + className : '')).trim()} {...rest}>
      {q ? (
        <svg className="ck-ciqr__code" style={{ width: size + 'px', height: size + 'px' }} viewBox={'0 0 ' + q.modules + ' ' + q.modules} role="img" aria-label={label + ' — ' + caption} shapeRendering="crispEdges" xmlns="http://www.w3.org/2000/svg">
          <path d={q.path} fill="currentColor"></path>
        </svg>
      ) : (
        <div className="ck-ciqr__code ck-ciqr__code--overflow" style={{ width: size + 'px', height: size + 'px' }}>⊘</div>
      )}
      <figcaption className="ck-ciqr__caption">
        <span className="ck-ciqr__mark">{attested ? '◆ ' : ''}{label}</span>
        <span className="ck-ciqr__note">{q ? caption : 'payload exceeds v5 · EC-L'}</span>
      </figcaption>
    </figure>
  );
}

// ---------- EntityCard ----------
const ENTITY_KINDS = {
  system: { label: 'System', mark: CK_ICON_PATHS.system },
  agent: { label: 'Agent', mark: CK_ICON_PATHS.agent },
  skill: { label: 'Skill', mark: CK_ICON_PATHS.skill },
  model: { label: 'Model', mark: CK_ICON_PATHS.model },
  aipack: { label: 'AIPack', mark: CK_ICON_PATHS.aipack },
  guardrail: { label: 'Guardrail', mark: CK_ICON_PATHS.guardrail },
};
const ENTITY_TIERS = {
  E0: { word: 'claimed', glyph: '○', k: 'none' },
  E1: { word: 'scanned', glyph: '◌', k: 'none' },
  E3: { word: 'validated', glyph: '●', k: 'mid' },
  E5: { word: 'proved', glyph: '◆', k: 'proof' },
};
function fmtEntityDigest(d) { if (!d) return ''; const m = String(d).match(/^([a-z0-9-]+:)(.+)$/i); const prefix = m ? m[1] : ''; const hex = m ? m[2] : String(d); return prefix + (hex.length > 12 ? hex.slice(0, 4) + '…' + hex.slice(-4) : hex); }
function EntityCopyBtn({ value }) {
  const [done, setDone] = React.useState(false);
  const onCopy = () => { const fin = () => { setDone(true); setTimeout(() => setDone(false), 1400); }; if (navigator.clipboard) navigator.clipboard.writeText(value).then(fin, fin); else fin(); };
  return <button type="button" className="ck-entity__copy" data-done={done ? 'true' : 'false'} onClick={onCopy}>{done ? 'Copied' : 'Copy'}</button>;
}
function EntityCard({ kind, name, urn, version, owner, sealedAt, tier, digest, ciqr, className = '', children, ...rest }) {
  const k = ENTITY_KINDS[kind]; const t = tier ? ENTITY_TIERS[tier] : null; const attested = !!digest;
  return (
    <article className={('ck-entity' + (attested ? ' ck-sealed' : '') + (className ? ' ' + className : '')).trim()} data-kind={kind} {...rest}>
      <header className="ck-entity__head">
        <svg className="ck-entity__mark" width={20} height={20} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5} strokeLinecap="square" strokeLinejoin="miter" aria-hidden="true" dangerouslySetInnerHTML={{ __html: k ? k.mark : '' }}></svg>
        <span className="ck-entity__kind">{k ? k.label : String(kind)}</span>
        {t ? <span className="ck-entity__tier" data-tier={t.k}><span aria-hidden="true">{t.glyph}</span> {tier} · {t.word}</span> : null}
      </header>
      <h3 className="ck-entity__name">{name}</h3>
      {urn ? <div className="ck-entity__urn"><span className="ck-entity__urnval">{urn}</span><EntityCopyBtn value={urn} /></div> : null}
      {(owner || version || sealedAt) ? (
        <dl className="ck-entity__facts">
          {owner ? <React.Fragment><dt>owner</dt><dd>{owner}</dd></React.Fragment> : null}
          {version ? <React.Fragment><dt>version</dt><dd>{version}</dd></React.Fragment> : null}
          {sealedAt ? <React.Fragment><dt>sealed</dt><dd>{sealedAt}</dd></React.Fragment> : null}
        </dl>
      ) : null}
      {children}
      <footer className="ck-entity__foot">
        {attested ? <span className="ck-stamp" title={digest}><span aria-hidden="true">◆</span> sealed · {fmtEntityDigest(digest)}</span> : <span className="ck-entity__unattested">○ unattested</span>}
        {ciqr ? <svg className="ck-entity__ciqr" viewBox={'0 0 ' + ciqr.modules + ' ' + ciqr.modules} role="img" aria-label="CIQR — scan to resolve canonical evidence" shapeRendering="crispEdges" xmlns="http://www.w3.org/2000/svg"><path d={ciqr.path} fill="currentColor"></path></svg> : null}
      </footer>
    </article>
  );
}

// ---------- CollapsibleMargin — the Evidence Margin, properly collapsible ----------
// The shipped bundle's `collapsible` prop is a stale build with no CSS; this
// re-implements it on the real ck-receipt / ck-stamp classes and persists the
// collapsed state, so the margin folds to a 48px rail (never a tab).
function CollapsibleMargin({ title = 'Evidence margin', entries = [], stamp, storageKey = 'ck-margin', children, defaultCollapsed = false }) {
  const [collapsed, setCollapsed] = React.useState(() => { try { const v = localStorage.getItem(storageKey); return v === null ? defaultCollapsed : v === '1'; } catch (e) { return defaultCollapsed; } });
  React.useEffect(() => { try { localStorage.setItem(storageKey, collapsed ? '1' : '0'); } catch (e) {} }, [collapsed, storageKey]);
  const toggle = () => setCollapsed((c) => !c);
  const glyph = <CkIcon name="margin" size={19} />;
  if (collapsed) {
    return (
      <div className="ckh-margin ckh-margin--collapsed" data-collapsed="true">
        <button type="button" className="ckh-margin__toggle" onClick={toggle} aria-expanded="false" aria-label="Expand evidence margin">{glyph}</button>
        <div className="ckh-margin__rail">{title}</div>
        {stamp ? <div className="ckh-margin__railstamp" title={'sealed · ' + stamp} aria-hidden="true">◆</div> : null}
      </div>
    );
  }
  return (
    <div className="ckh-margin" data-collapsed="false">
      <div className="ckh-margin__head"><div className="ck-label">{title}</div><button type="button" className="ckh-margin__toggle" onClick={toggle} aria-expanded="true" aria-label="Collapse evidence margin">{glyph}</button></div>
      {entries.map((en, i) => {
        const mod = en.kind === 'proof' ? ' ck-receipt--proof' : en.kind === 'alarm' ? ' ck-receipt--alarm' : '';
        return (
          <div key={i} className={'ck-receipt' + mod}>
            <div className="ck-receipt__rule"></div>
            <div className="ck-stack-1">
              <div className="ck-receipt__time">{en.time}</div>
              <div className="ck-receipt__event">{en.kind === 'proof' ? '◆ ' : en.kind === 'alarm' ? '⊘ ' : ''}{en.event}</div>
              {(en.details || []).map((d, j) => <div key={j} className="ck-receipt__detail">{d}</div>)}
            </div>
          </div>
        );
      })}
      {children}
      {stamp ? <div><span className="ck-stamp" title={stamp}><span aria-hidden="true">◆</span> sealed · {stamp}</span></div> : null}
    </div>
  );
}

Object.assign(window, { CkIcon, CK_ICON_PATHS, CkCiqr, CiqrPath, EntityCard, ENTITY_KINDS, CollapsibleMargin });
