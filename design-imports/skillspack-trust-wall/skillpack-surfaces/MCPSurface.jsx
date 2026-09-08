// ============================================================
// SkillPack · MCP Server surface (C-09)
// Headless · Rust · MCP protocol over stdio/sse · exposes skills
// as MCP *resources* + a detail *tool* for AI hosts (Claude, Copilot).
//   Feature matrix: MCP resource exposure = MUST · skill detail = SHOULD ·
//   assess / install / publish / registry-search = MUST NOT (read-only bridge).
// DS-3 Vault · safe-set glyphs only (→ ← ⊢ ◆ ▸ ⊘).
// Unique-named atoms (Mcp*) — top-level consts share one script scope.
// ============================================================

const MCP_MONO = "'JetBrains Mono', monospace";

// one stdio frame · dir = '→' host→server (request) · '←' server→host (response)
const McpFrame = ({ dir, children, tone }) => (
  <div style={{ display: 'flex', gap: 9, alignItems: 'baseline', padding: '1px 0' }}>
    <span style={{ font: `600 12px ${MCP_MONO}`, color: dir === '→' ? 'var(--ck-link)' : 'var(--ck-accent)', width: 12, flexShrink: 0 }}>{dir}</span>
    <span style={{ font: `400 12px/18px ${MCP_MONO}`, color: tone || 'var(--ck-fg-2)', whiteSpace: 'pre-wrap', minWidth: 0 }}>{children}</span>
  </div>
);
const McpKey = ({ children }) => <span style={{ color: 'var(--ck-fg-3)' }}>{children}</span>;
const McpStr = ({ children }) => <span style={{ color: 'var(--ck-accent)' }}>{children}</span>;
const McpUri = ({ children }) => <span style={{ color: 'var(--ck-proof)' }}>{children}</span>;

// a resource row in the catalog rail
const McpResource = ({ uri, grade, gtone, tier }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 9, padding: '7px 0', boxShadow: 'inset 0 -1px 0 color-mix(in oklab, var(--ck-stroke) 16%, transparent)' }}>
    <span style={{ font: `600 11px ${MCP_MONO}`, color: 'var(--ck-proof)', flexShrink: 0 }}>▸</span>
    <span style={{ flex: 1, minWidth: 0, font: `400 11px ${MCP_MONO}`, color: 'var(--ck-fg-2)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{uri}</span>
    <span style={{ font: `700 11px ${MCP_MONO}`, color: gtone, flexShrink: 0 }}>{grade}</span>
    <span style={{ font: `500 9.5px ${MCP_MONO}`, color: 'var(--ck-fg-mute)', flexShrink: 0, width: 22, textAlign: 'right' }}>{tier}</span>
  </div>
);

const MCPSurface = () => (
  <div data-screen-label="mcp server · skillpack" style={{ width: '100%', height: '100%', display: 'flex', flexDirection: 'column', background: 'var(--ck-bg-0)', overflow: 'hidden', fontFamily: 'var(--ck-ff-ui)' }}>
    {/* server header — connected host + transport */}
    <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '13px 18px', background: 'var(--ck-bg-1)', boxShadow: 'inset 0 -1px 0 color-mix(in oklab, var(--ck-stroke) 22%, transparent)', flexShrink: 0 }}>
      <span style={{ width: 9, height: 9, borderRadius: 5, background: 'var(--ck-accent)', flexShrink: 0 }} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ font: "700 13px var(--ck-ff-ui)", color: 'var(--ck-fg-1)' }}>skillpack-mcp · C-09</div>
        <div style={{ font: `500 10px ${MCP_MONO}`, color: 'var(--ck-fg-mute)', letterSpacing: '.02em' }}>transport stdio · MCP 2025-06-18 · read-only bridge</div>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 11px', clipPath: chamfer(6), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-proof) 45%, transparent)' }}>
        <span style={{ font: `600 11px ${MCP_MONO}`, color: 'var(--ck-proof)' }}>⊢</span>
        <span style={{ font: `600 10.5px ${MCP_MONO}`, color: 'var(--ck-fg-1)' }}>host: claude</span>
      </div>
    </div>

    {/* body — stdio session (left) + exposed catalog (right) */}
    <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '1fr 304px', minHeight: 0 }}>
      {/* JSON-RPC stdio session */}
      <div style={{ padding: '14px 18px', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 1 }}>
        <div style={{ font: "700 9px var(--ck-ff-ui)", letterSpacing: '.13em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 8 }}>JSON-RPC · stdio</div>

        <McpFrame dir="→"><McpKey>initialize</McpKey> {'{ '}clientInfo: {'{ '}name: <McpStr>"claude"</McpStr>, version: <McpStr>"3.7"</McpStr>{' }'} {'}'}</McpFrame>
        <McpFrame dir="←"><McpKey>result</McpKey> {'{ '}serverInfo: skillpack-mcp <McpStr>1.0.0</McpStr>,</McpFrame>
        <McpFrame dir="←" tone="var(--ck-fg-3)">{'       '}capabilities: {'{ '}resources: {'{'} subscribe: <McpStr>true</McpStr> {'}'}, tools: {'{}'} {'}'} {'}'}</McpFrame>

        <div style={{ height: 9 }} />
        <McpFrame dir="→"><McpKey>resources/list</McpKey></McpFrame>
        <McpFrame dir="←"><McpKey>result</McpKey> {'{ '}resources: <span style={{ color: 'var(--ck-fg-1)' }}>12</span> exposed {'}'}  <span style={{ color: 'var(--ck-fg-mute)' }}>→ catalog ▸</span></McpFrame>

        <div style={{ height: 9 }} />
        <McpFrame dir="→"><McpKey>resources/read</McpKey> {'{ '}uri: <McpUri>"skill://ckodex/writing-skills"</McpUri> {'}'}</McpFrame>
        <McpFrame dir="←"><McpKey>result</McpKey> {'{ '}contents: [ SKILL.md ],</McpFrame>
        <McpFrame dir="←" tone="var(--ck-fg-3)">{'       '}meta: {'{ '}<span style={{ color: 'var(--ck-accent)' }}>◆ grade S+</span> · tier L4 · GAL-6 · cosign <span style={{ color: 'var(--ck-accent)' }}>⊢</span> {'}'} {'}'}</McpFrame>

        <div style={{ height: 9 }} />
        <McpFrame dir="→"><McpKey>tools/call</McpKey> {'{ '}name: <McpStr>"get_skill_detail"</McpStr>,</McpFrame>
        <McpFrame dir="→" tone="var(--ck-fg-3)">{'           '}arguments: {'{ '}skill: <McpStr>"my-formatter"</McpStr> {'}'} {'}'}</McpFrame>
        <McpFrame dir="←"><McpKey>result</McpKey> {'{ '}<span style={{ color: 'var(--ck-fg-1)' }}>◆ grade C (72)</span> · tier L1 · 2 issues {'}'}</McpFrame>

        {/* read-only boundary note */}
        <div style={{ marginTop: 'auto', display: 'flex', alignItems: 'center', gap: 9, paddingTop: 12 }}>
          <span style={{ font: `600 12px ${MCP_MONO}`, color: 'var(--ck-deny)' }}>⊘</span>
          <span style={{ font: `500 10.5px ${MCP_MONO}`, color: 'var(--ck-fg-mute)' }}>assess · install · publish · search — not exposed (MUST NOT)</span>
        </div>
      </div>

      {/* exposed resource catalog */}
      <div style={{ borderLeft: '1px solid color-mix(in oklab, var(--ck-stroke) 22%, transparent)', background: 'var(--ck-bg-1)', padding: '14px 16px', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
        <div style={{ font: "700 9px var(--ck-ff-ui)", letterSpacing: '.13em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', marginBottom: 4 }}>Exposed resources</div>
        <div style={{ overflow: 'hidden' }}>
          <McpResource uri="skill://ckodex/writing-skills" grade="S+" gtone="var(--ck-accent)" tier="L4" />
          <McpResource uri="skill://ckodex/conformance" grade="A" gtone="var(--ck-proof)" tier="L3" />
          <McpResource uri="skill://ckodex/proof-audit" grade="A" gtone="var(--ck-proof)" tier="L3" />
          <McpResource uri="skill://ckodex/my-formatter" grade="C" gtone="var(--ck-fg-2)" tier="L1" />
        </div>

        <div style={{ font: "700 9px var(--ck-ff-ui)", letterSpacing: '.13em', color: 'var(--ck-fg-3)', textTransform: 'uppercase', margin: '16px 0 8px' }}>Tools</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 9, padding: '9px 11px', clipPath: chamfer(7), boxShadow: 'inset 0 0 0 1.5px color-mix(in oklab, var(--ck-stroke) 30%, transparent)' }}>
          <span style={{ font: `600 12px ${MCP_MONO}`, color: 'var(--ck-accent)' }}>⊹</span>
          <div style={{ minWidth: 0 }}>
            <div style={{ font: `600 11px ${MCP_MONO}`, color: 'var(--ck-fg-1)' }}>get_skill_detail</div>
            <div style={{ font: `500 9px ${MCP_MONO}`, color: 'var(--ck-fg-mute)' }}>read-only · SHOULD</div>
          </div>
        </div>

        <div style={{ marginTop: 'auto', paddingTop: 14, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ font: `500 9.5px ${MCP_MONO}`, color: 'var(--ck-fg-mute)' }}>gRPC :50051 · token SKILLPACK_MCP_TOKEN</span>
        </div>
      </div>
    </div>

    {/* footer status */}
    <div style={{ display: 'flex', alignItems: 'center', gap: 14, padding: '10px 18px', background: 'var(--ck-bg-1)', boxShadow: 'inset 0 1px 0 color-mix(in oklab, var(--ck-stroke) 22%, transparent)', flexShrink: 0, font: `500 10px ${MCP_MONO}`, color: 'var(--ck-fg-mute)' }}>
      <span style={{ color: 'var(--ck-accent)' }}>⊢ serving</span>
      <span>12 resources · 1 tool</span>
      <span style={{ marginLeft: 'auto' }}>subscribe: on · keepalive 30s</span>
    </div>
  </div>
);

Object.assign(window, { MCPSurface });
