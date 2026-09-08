// ============================================================
// SkillPack · Client Surfaces — design canvas
// VS Code (C-02) · macOS Menu Bar (C-04) · Mobile responsive web (C-03)
// ============================================================

const SurfacesApp = () => (
  <DesignCanvas>
    <DCSection id="cli" title="CLI · C-01 + headless CNI · C-08 + MCP · C-09" subtitle="Rust (clap) · 80-col terminal · most capable surface · CNI is the non-interactive CI/CD gate · MCP exposes skills as read-only resources to AI hosts">
      <DCArtboard id="cli-main" label="skillpack · interactive terminal" width={920} height={620} style={{ background: '#00152b' }}>
        <CLISurface />
      </DCArtboard>
      <DCArtboard id="cni-main" label="skillpack-cni · CI/CD gate (headless)" width={720} height={620} style={{ background: 'var(--ck-bg-0)' }}>
        <CNISurface />
      </DCArtboard>
      <DCArtboard id="mcp-main" label="skillpack-mcp · MCP server (headless)" width={920} height={520} style={{ background: 'var(--ck-bg-0)' }}>
        <MCPSurface />
      </DCArtboard>
    </DCSection>

    <DCSection id="vscode" title="VS Code Extension · C-02" subtitle="300px sidebar + editor · auto-assess on save · AI-assisted authoring · inline install">
      <DCArtboard id="vscode-main" label="Editor · SKILL.md + assessment" width={1180} height={760} style={{ background: 'var(--ck-bg-0)' }}>
        <VSCodeSurface />
      </DCArtboard>
    </DCSection>

    <DCSection id="menubar" title="macOS Menu Bar · C-04" subtitle="320px fixed popover · status · sync · install-by-reference · lifecycle · NO registry search/browse">
      <DCArtboard id="menubar-main" label="Menu bar popover" width={640} height={560} style={{ background: '#00152b' }}>
        <MenuBarSurface />
      </DCArtboard>
    </DCSection>

    <DCSection id="mobile" title="Mobile · responsive web (C-03)" subtitle="Touch-first · 44px+ targets · bottom tab nav · browse → install sheet">
      <DCArtboard id="mobile-browse" label="Browse" width={402} height={874} style={{ background: 'transparent', boxShadow: 'none', borderRadius: 0, overflow: 'visible' }}>
        <MobileBrowse />
      </DCArtboard>
      <DCArtboard id="mobile-install" label="Install sheet" width={402} height={874} style={{ background: 'transparent', boxShadow: 'none', borderRadius: 0, overflow: 'visible' }}>
        <MobileInstall />
      </DCArtboard>
    </DCSection>
  </DesignCanvas>
);

ReactDOM.createRoot(document.getElementById('root')).render(<SurfacesApp />);
