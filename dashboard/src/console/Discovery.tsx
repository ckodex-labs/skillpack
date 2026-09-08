// @ts-nocheck
'use client';
import React from 'react';
import { chamfer, Bento, Badge, CkButton, SectionLabel, CodeBlock } from './primitives';
// ============================================================
// SkillPack Registry · Discovery page
// REAL connection probe against the local daemon (fetch /health,
// measured latency, real protocol) · federated-registry roster with the
// local daemon as the only live entry · live manifest · add-by-URL that
// really attempts /.well-known/skillpack.json.
// ============================================================
const { useState: _useStateD, useRef: _useRefD, useEffect: _useEffectD } = React;

// Real probe steps — driven by the async fetch progress, not timers.
const PROBE_STEPS = [
  { id: 'resolve', title: 'resolve endpoint', detail: 'locate local daemon base URL', glyph: '⌖' },
  { id: 'http',    title: 'HTTP GET /health', detail: 'fetch health, measure latency', glyph: '→' },
  { id: 'parse',   title: 'parse protocol',   detail: 'read protocolVersion from body', glyph: '⊢' },
];

// status: 'idle' | 'running' | 'done' | 'fail'
const ConnectSequence = ({ status, running, onProbe, target, latencyMs, protocol, error, done }) => {
  const failed = status.some(s => s === 'fail');
  return (
    <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
        <SectionLabel>Connection sequence</SectionLabel>
        <span style={{ marginLeft: 'auto', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>live · GET /health</span>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        {PROBE_STEPS.map((s, i) => {
          const st = status[i];
          const cur = st === 'running';
          const isDone = st === 'done';
          const isFail = st === 'fail';
          const tone = isFail ? 'var(--ck-deny)' : isDone ? 'var(--ck-link)' : cur ? 'var(--ck-accent)' : 'var(--ck-fg-mute)';
          const glyph = isFail ? '⊭' : isDone ? '⊢' : s.glyph;
          return (
            <div key={s.id} className="ckr-row" style={{ display: 'flex', alignItems: 'flex-start', gap: 12, padding: '9px 0', opacity: st === 'idle' ? 0.45 : 1 }}>
              <span className={cur ? 'ckr-glyph ckr-spin' : 'ckr-glyph'} aria-hidden="true" style={{ font: "600 14px 'JetBrains Mono', monospace", color: tone, width: 20, textAlign: 'center', flexShrink: 0, display: 'inline-block' }}>{glyph}</span>
              <div style={{ flex: 1 }}>
                <div style={{ font: "600 12px 'JetBrains Mono', monospace", color: st === 'idle' ? 'var(--ck-fg-3)' : 'var(--ck-fg-1)' }}>{s.title}</div>
                <div style={{ font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 2 }}>{s.detail}</div>
              </div>
              {i === 1 && isDone && latencyMs != null && <Badge kind="kernel" title="Measured round-trip latency">{latencyMs}ms</Badge>}
              {i === 2 && isDone && <Badge kind="teal" title="Negotiated protocol">v{protocol}</Badge>}
            </div>
          );
        })}
      </div>
      {failed && error && (
        <div style={{ marginTop: 12, padding: '10px 12px', clipPath: chamfer(6), background: 'color-mix(in oklab, var(--ck-deny) 10%, transparent)', boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-deny) 40%, transparent)', font: "500 11px 'JetBrains Mono', monospace", color: 'var(--ck-deny)' }}>
          ⊭ probe failed · {error}
        </div>
      )}
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 14 }}>
        <CkButton variant={running ? 'secondary' : 'primary'} onClick={onProbe} disabled={running}>
          {running ? '∿ Probing…' : done ? '↻ Re-probe' : '⌖ Probe ' + target}
        </CkButton>
        {done && !running && !failed && <Badge kind="attested" title="Handshake complete">⊢ connected</Badge>}
        {done && !running && failed && <Badge kind="deny" title="Handshake failed">⊭ offline</Badge>}
      </div>
    </div>
  );
};

const stripScheme = (u) => (u || '').replace(/^https?:\/\//, '').replace(/\/$/, '');

const Discovery = ({ health, apiBase, registries }) => {
  const h = health || {};
  const regs = registries || [];
  const proto = h.protocolVersion || 1;
  const daemonHost = stripScheme(apiBase);

  const [status, setStatus] = _useStateD(['idle', 'idle', 'idle']);
  const [running, setRunning] = _useStateD(false);
  const [done, setDone] = _useStateD(false);
  const [latencyMs, setLatencyMs] = _useStateD(null);
  const [probedProto, setProbedProto] = _useStateD(null);
  const [error, setError] = _useStateD(null);

  const [newUrl, setNewUrl] = _useStateD('');
  const [discovering, setDiscovering] = _useStateD(false);
  const [discoverResult, setDiscoverResult] = _useStateD(null); // { ok, text }
  const abortRef = _useRefD(null);

  _useEffectD(() => () => { if (abortRef.current) abortRef.current.abort(); }, []);

  const setStep = (i, v) => setStatus(prev => { const n = prev.slice(); n[i] = v; return n; });

  // REAL probe: fetch apiBase/health, measure latency, read protocolVersion.
  const probe = async () => {
    setRunning(true); setDone(false); setError(null);
    setLatencyMs(null); setProbedProto(null);
    setStatus(['running', 'idle', 'idle']);

    const ctrl = new AbortController();
    abortRef.current = ctrl;
    const timeout = setTimeout(() => ctrl.abort(), 3000);

    try {
      // Step 0 — resolve endpoint
      if (!apiBase) throw new Error('no API base configured');
      setStep(0, 'done');

      // Step 1 — HTTP GET /health (measured)
      setStep(1, 'running');
      const t0 = performance.now();
      const res = await fetch(`${apiBase}/health`, { signal: ctrl.signal, headers: { accept: 'application/json' } });
      const t1 = performance.now();
      const ms = Math.round(t1 - t0);
      setLatencyMs(ms);
      if (!res.ok) throw new Error(`HTTP ${res.status} ${res.statusText || ''}`.trim());
      setStep(1, 'done');

      // Step 2 — parse protocol
      setStep(2, 'running');
      let body = {};
      try { body = await res.json(); } catch (_) { body = {}; }
      const pv = body && body.protocolVersion != null ? body.protocolVersion : (h.protocolVersion || 1);
      setProbedProto(pv);
      setStep(2, 'done');
    } catch (e) {
      const msg = e && e.name === 'AbortError' ? 'timeout after 3000ms' : (e && e.message) || String(e);
      setError(msg);
      // Mark the first not-yet-done step as failed.
      setStatus(prev => {
        const n = prev.slice();
        const idx = n.findIndex(s => s === 'running' || s === 'idle');
        if (idx >= 0) n[idx] = 'fail';
        return n;
      });
    } finally {
      clearTimeout(timeout);
      setRunning(false);
      setDone(true);
      abortRef.current = null;
    }
  };

  // REAL discover: attempt to fetch the well-known manifest from a URL.
  const discover = async () => {
    const url = newUrl.trim().replace(/\/$/, '');
    if (!url) return;
    setDiscovering(true); setDiscoverResult(null);
    const ctrl = new AbortController();
    const timeout = setTimeout(() => ctrl.abort(), 3000);
    try {
      const res = await fetch(`${url}/.well-known/skillpack.json`, { signal: ctrl.signal, headers: { accept: 'application/json' } });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const txt = await res.text();
      setDiscoverResult({ ok: true, text: `⊢ reachable · ${txt.length} bytes` });
    } catch (e) {
      const msg = e && e.name === 'AbortError' ? 'timeout' : (e && e.message) || String(e);
      setDiscoverResult({ ok: false, text: `⊭ unreachable / CORS · ${msg}` });
    } finally {
      clearTimeout(timeout);
      setDiscovering(false);
    }
  };

  const shownProto = probedProto != null ? probedProto : proto;

  // Live manifest built from the real daemon facts.
  const manifest = {
    registry: 'ckodex-skillpack (local)',
    endpoint: apiBase,
    protocol: 'v' + shownProto,
    transports: ['http', 'grpc'],
    auth: 'Bearer · SKILLPACK_API_TOKEN',
    health: h.healthy ? 'healthy' : 'offline',
  };

  const capabilities = ['assess', 'catalog', 'health', 'sse'];

  return (
    <div style={{ padding: '24px 28px', display: 'grid', gridTemplateColumns: '1.15fr 1fr', gap: 22, alignItems: 'start' }}>
      {/* LEFT */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18, minWidth: 0 }}>
        <ConnectSequence
          status={status} running={running} done={done} onProbe={probe}
          target="local · daemon" latencyMs={latencyMs} protocol={shownProto} error={error}
        />

        {/* federated registries */}
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 14 }}>
            <SectionLabel>Federated registries</SectionLabel>
            <span style={{ marginLeft: 'auto', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>1 live · {regs.length} demo</span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {/* LOCAL DAEMON — the only real entry */}
            <div className="ckr-row" style={{
              display: 'grid', gridTemplateColumns: '18px 1fr auto', alignItems: 'center', gap: 12, padding: '12px 12px',
              clipPath: chamfer(7), background: 'var(--ck-bg-2)', boxShadow: 'inset 0 0 0 1.5px var(--ck-stroke)',
            }}>
              <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 13px 'JetBrains Mono', monospace", color: 'var(--ck-accent)', textAlign: 'center' }}>⊢</span>
              <span style={{ minWidth: 0 }}>
                <span style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                  <span style={{ font: "700 13px 'Geist', sans-serif", color: 'var(--ck-fg-1)' }}>local · daemon</span>
                  <span style={{ font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{daemonHost}</span>
                </span>
                <span style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 6, flexWrap: 'wrap' }}>
                  <Badge kind="kernel" title="http transport">http</Badge>
                  <Badge kind="kernel" title="grpc transport">grpc</Badge>
                  <span style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.04em' }}>real endpoint · {h.probed ? 'probed' : 'not probed'}</span>
                </span>
              </span>
              <span style={{ textAlign: 'right' }}>
                <span style={{ display: 'flex', alignItems: 'center', gap: 6, justifyContent: 'flex-end' }}>
                  <span aria-hidden="true" style={{ width: 7, height: 7, background: h.healthy ? 'var(--ck-accent)' : 'var(--ck-deny)', display: 'inline-block', clipPath: chamfer(2) }} />
                  <span style={{ font: "600 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-2)' }}>{h.healthy ? 'healthy' : 'offline'}</span>
                </span>
                <span style={{ display: 'block', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 4 }}>
                  {latencyMs != null ? `${latencyMs}ms · ` : ''}proto v{shownProto}
                </span>
              </span>
            </div>

            {/* DEMO PEERS — honestly not reachable */}
            {regs.map(r => {
              const review = r.trust === 'review';
              return (
                <div key={r.id} className="ckr-row" style={{
                  display: 'grid', gridTemplateColumns: '18px 1fr auto', alignItems: 'center', gap: 12, padding: '12px 12px',
                  clipPath: chamfer(7), background: 'transparent', opacity: 0.72,
                  boxShadow: review
                    ? 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-deny) 34%, transparent)'
                    : 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 20%, transparent)',
                }}>
                  <span className="ckr-glyph" aria-hidden="true" style={{ font: "600 13px 'JetBrains Mono', monospace", color: review ? 'var(--ck-deny)' : 'var(--ck-fg-mute)', textAlign: 'center' }}>{review ? '⚠' : '◌'}</span>
                  <span style={{ minWidth: 0 }}>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                      <span style={{ font: "700 13px 'Geist', sans-serif", color: 'var(--ck-fg-2)' }}>{r.label}</span>
                      <span style={{ font: "400 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>{r.host}</span>
                    </span>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 6, flexWrap: 'wrap' }}>
                      {review && <Badge kind="deny" title="Trust policy: review required">review · untrusted</Badge>}
                      <span style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-3)', letterSpacing: '.04em' }}>{r.note || 'demo peer'}</span>
                    </span>
                  </span>
                  <span style={{ textAlign: 'right' }}>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 6, justifyContent: 'flex-end' }}>
                      <span aria-hidden="true" style={{ width: 7, height: 7, background: 'var(--ck-fg-mute)', display: 'inline-block', clipPath: chamfer(2) }} />
                      <span style={{ font: "600 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>not reachable</span>
                    </span>
                    <span style={{ display: 'block', font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', marginTop: 4 }}>demo peer · no data</span>
                  </span>
                </div>
              );
            })}
          </div>

          {/* add registry */}
          <div style={{ height: 1, background: 'color-mix(in oklab, var(--ck-stroke) 20%, transparent)', margin: '16px 0' }} />
          <SectionLabel style={{ marginBottom: 10 }}>Add registry by URL</SectionLabel>
          <div style={{ display: 'flex', gap: 8 }}>
            <div className="ckr-input" style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 10, padding: '0 12px', background: 'var(--ck-bg-2)', clipPath: chamfer(7), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 40%, transparent)', height: 42 }}>
              <span className="ckr-glyph" aria-hidden="true" style={{ font: "500 13px 'JetBrains Mono', monospace", color: 'var(--ck-accent)' }}>⌖</span>
              <input value={newUrl} onChange={(e) => setNewUrl(e.target.value)} placeholder="https://registry.example.org" aria-label="Registry URL to discover"
                style={{ flex: 1, border: 'none', outline: 'none', background: 'transparent', color: 'var(--ck-fg-1)', font: "400 13px 'JetBrains Mono', monospace", minWidth: 0 }} />
              <span style={{ font: "500 9px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)', whiteSpace: 'nowrap' }}>+ /.well-known/skillpack.json</span>
            </div>
            <CkButton variant="secondary" disabled={!newUrl.trim() || discovering} onClick={discover} title="Fetch the well-known manifest">
              {discovering ? '∿ …' : 'Discover'}
            </CkButton>
          </div>
          {discoverResult && (
            <div style={{
              marginTop: 10, padding: '9px 12px', clipPath: chamfer(6),
              background: discoverResult.ok ? 'color-mix(in oklab, var(--ck-accent) 10%, transparent)' : 'color-mix(in oklab, var(--ck-deny) 10%, transparent)',
              boxShadow: `inset 0 0 0 1.5px color-mix(in oklab, ${discoverResult.ok ? 'var(--ck-accent)' : 'var(--ck-deny)'} 38%, transparent)`,
              font: "500 11px 'JetBrains Mono', monospace", color: discoverResult.ok ? 'var(--ck-fg-1)' : 'var(--ck-deny)',
            }}>
              {discoverResult.text}
            </div>
          )}
        </div>
      </div>

      {/* RIGHT · manifest + capabilities */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18, position: 'sticky', top: 96, minWidth: 0 }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
            <SectionLabel>Manifest</SectionLabel>
            <span style={{ font: "500 10px 'JetBrains Mono', monospace", color: 'var(--ck-fg-mute)' }}>live · {daemonHost}</span>
          </div>
          <CodeBlock title="local daemon manifest" json={manifest} height={360} />
        </div>
        <div className="ckr-bento" style={{ background: 'var(--ck-bg-1)', clipPath: chamfer(10), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 38%, transparent)', padding: 18 }}>
          <SectionLabel style={{ marginBottom: 12 }}>Negotiated capabilities</SectionLabel>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 7 }}>
            {capabilities.map(f => <Badge key={f} kind="teal" title={`Capability: ${f}`}><span className="ckr-glyph" aria-hidden="true">⊢</span>{f}</Badge>)}
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10, marginTop: 16 }}>
            <KV k="protocol" v={`v${shownProto}`} />
            <KV k="transport" v="http → grpc" />
            <KV k="auth" v="Bearer · SKILLPACK_API_TOKEN" />
            <KV k="health" v={h.healthy ? '⊢ healthy' : '⊭ offline'} />
          </div>
        </div>
      </div>
    </div>
  );
};

const KV = ({ k, v }) => (
  <div style={{ padding: '10px 12px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1px color-mix(in oklab, var(--ck-stroke) 22%, transparent)' }}>
    <div style={{ font: "600 9px 'Geist', sans-serif", letterSpacing: '.1em', textTransform: 'uppercase', color: 'var(--ck-fg-3)' }}>{k}</div>
    <div style={{ font: "600 11px 'JetBrains Mono', monospace", color: 'var(--ck-fg-1)', marginTop: 4 }}>{v}</div>
  </div>
);

export { Discovery };
