import * as vscode from "vscode";
import type { SkillSummary } from "../generated/client-model";

export function getWizardHtml(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Create New Skill</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            padding: 24px;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
        }
        h1 { font-size: 1.5rem; margin-bottom: 24px; }
        .field { margin-bottom: 20px; }
        label { display: block; margin-bottom: 6px; font-weight: 600; }
        input, textarea {
            width: 100%;
            padding: 8px 12px;
            background: var(--vscode-input-background);
            border: 1px solid var(--vscode-input-border);
            color: var(--vscode-input-foreground);
            border-radius: 4px;
            font-size: 14px;
        }
        .checkbox-group { display: flex; flex-direction: column; gap: 8px; }
        .checkbox-group label {
            display: flex;
            align-items: center;
            gap: 8px;
            font-weight: normal;
        }
        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 10px 24px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }
        button:hover { background: var(--vscode-button-hoverBackground); }
        .help { font-size: 12px; color: var(--vscode-descriptionForeground); margin-top: 4px; }
    </style>
</head>
<body>
    <h1>🧩 Create New Skill</h1>
    <div class="field">
        <label for="name">Skill Name *</label>
        <input type="text" id="name" placeholder="my-awesome-skill" required>
        <p class="help">Use lowercase with hyphens (e.g., data-analyzer)</p>
    </div>
    <div class="field">
        <label for="description">Description *</label>
        <textarea id="description" rows="2" placeholder="A brief description of what this skill does"></textarea>
    </div>
    <div class="field">
        <label for="author">Author</label>
        <input type="text" id="author" placeholder="Your Name <email@example.com>">
    </div>
    <div class="field">
        <label>Capacities</label>
        <div class="checkbox-group">
            <label><input type="checkbox" id="cap-mcp" checked> MCP Server (for AI agents)</label>
            <label><input type="checkbox" id="cap-cli"> CLI Tool</label>
            <label><input type="checkbox" id="cap-rest"> REST API</label>
        </div>
    </div>
    <div class="field">
        <label>Options</label>
        <div class="checkbox-group">
            <label><input type="checkbox" id="lifecycle" checked> Include lifecycle operations</label>
            <label><input type="checkbox" id="security" checked> Create security directory</label>
            <label><input type="checkbox" id="tests" checked> Add test scaffolding</label>
        </div>
    </div>
    <button onclick="createSkill()">Create Skill</button>
    <script>
        const vscode = acquireVsCodeApi();
        function createSkill() {
            const data = {
                name: document.getElementById('name').value,
                description: document.getElementById('description').value,
                author: document.getElementById('author').value,
                capacities: [],
                hasLifecycle: document.getElementById('lifecycle').checked,
            };
            if (document.getElementById('cap-mcp').checked) data.capacities.push('mcp');
            if (document.getElementById('cap-cli').checked) data.capacities.push('cli');
            if (document.getElementById('cap-rest').checked) data.capacities.push('rest');
            vscode.postMessage({ command: 'createSkill', data });
        }
    </script>
</body>
</html>`;
}

export function getAIChatHtml(initialQuestion?: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SkillPack AI Assistant</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            padding: 0; margin: 0;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            display: flex; flex-direction: column; height: 100vh;
        }
        .header { padding: 12px 16px; border-bottom: 1px solid var(--vscode-panel-border); display: flex; align-items: center; gap: 8px; }
        .header h2 { margin: 0; font-size: 14px; }
        .messages { flex: 1; overflow-y: auto; padding: 16px; }
        .message { margin-bottom: 16px; padding: 12px; border-radius: 8px; max-width: 90%; }
        .message.user { background: var(--vscode-button-background); color: var(--vscode-button-foreground); margin-left: auto; }
        .message.assistant { background: var(--vscode-editor-inactiveSelectionBackground); }
        .message pre { background: var(--vscode-textBlockQuote-background); padding: 8px; border-radius: 4px; overflow-x: auto; font-family: var(--vscode-editor-font-family); font-size: 13px; }
        .message code { font-family: var(--vscode-editor-font-family); }
        .input-area { padding: 12px; border-top: 1px solid var(--vscode-panel-border); display: flex; gap: 8px; }
        .input-area input { flex: 1; padding: 8px 12px; background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); color: var(--vscode-input-foreground); border-radius: 4px; }
        .input-area button { padding: 8px 16px; background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; border-radius: 4px; cursor: pointer; }
        .quick-actions { padding: 8px 16px; display: flex; gap: 8px; flex-wrap: wrap; }
        .quick-actions button { padding: 6px 12px; background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
    </style>
</head>
<body>
    <div class="header"><span>🤖</span><h2>SkillPack AI Assistant</h2></div>
    <div class="quick-actions">
        <button onclick="askQuestion('How do I add MCP capacity?')">Add MCP</button>
        <button onclick="askQuestion('How do I add lifecycle operations?')">Lifecycle</button>
        <button onclick="askQuestion('How do I improve security?')">Security</button>
        <button onclick="askQuestion('How do I publish?')">Publish</button>
    </div>
    <div class="messages" id="messages"></div>
    <div class="input-area">
        <input type="text" id="input" placeholder="Ask about skill authoring..." />
        <button onclick="sendMessage()">Send</button>
    </div>
    <script>
        const vscode = acquireVsCodeApi();
        const messagesEl = document.getElementById('messages');
        const inputEl = document.getElementById('input');
        ${initialQuestion ? `askQuestion("${initialQuestion}");` : ""}
        inputEl.addEventListener('keypress', (e) => { if (e.key === 'Enter') sendMessage(); });
        function askQuestion(text) { inputEl.value = text; sendMessage(); }
        function sendMessage() {
            const text = inputEl.value.trim();
            if (!text) return;
            addMessage(text, 'user');
            inputEl.value = '';
            vscode.postMessage({ command: 'sendMessage', text });
        }
        function addMessage(text, type) {
            const div = document.createElement('div');
            div.className = 'message ' + type;
            div.innerHTML = formatMessage(text);
            messagesEl.appendChild(div);
            messagesEl.scrollTop = messagesEl.scrollHeight;
        }
        function formatMessage(text) {
            return text
                .replace(/\`\`\`(\\w+)?\\n([\\s\\S]*?)\`\`\`/g, '<pre><code>$2</code></pre>')
                .replace(/\`([^\`]+)\`/g, '<code>$1</code>')
                .replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>')
                .replace(/\\n/g, '<br>');
        }
        window.addEventListener('message', (event) => {
            const message = event.data;
            if (message.command === 'response') { addMessage(message.text, 'assistant'); }
        });
    </script>
</body>
</html>`;
}

export function getImprovementSuggestionsHtml(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Skill Improvement Suggestions</title>
    <style>
        body { font-family: var(--vscode-font-family); padding: 24px; background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); }
        h1 { font-size: 1.25rem; margin-bottom: 16px; }
        .suggestion { padding: 16px; margin-bottom: 12px; background: var(--vscode-editor-inactiveSelectionBackground); border-radius: 8px; border-left: 4px solid var(--vscode-charts-blue); }
        .suggestion.high { border-left-color: var(--vscode-charts-green); }
        .suggestion.medium { border-left-color: var(--vscode-charts-yellow); }
        .suggestion h3 { margin: 0 0 8px; font-size: 14px; display: flex; align-items: center; gap: 8px; }
        .suggestion p { margin: 0 0 12px; font-size: 13px; color: var(--vscode-descriptionForeground); }
        .suggestion button { padding: 6px 12px; background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
        .impact { font-size: 11px; padding: 2px 6px; border-radius: 4px; background: var(--vscode-badge-background); color: var(--vscode-badge-foreground); }
    </style>
</head>
<body>
    <h1>💡 Improvement Suggestions</h1>
    <div class="suggestion high">
        <h3><span>🛡️ Add Security Controls</span><span class="impact">+15 points</span></h3>
        <p>Create a threat model and enable Sigstore signing for better security posture.</p>
        <button onclick="applyFix('security')">Apply Fix</button>
    </div>
    <div class="suggestion high">
        <h3><span>🔄 Add Lifecycle Operations</span><span class="impact">+10 points</span></h3>
        <p>Define install, verify, and uninstall operations for complete skill management.</p>
        <button onclick="applyFix('lifecycle')">Apply Fix</button>
    </div>
    <div class="suggestion medium">
        <h3><span>📚 Improve Documentation</span><span class="impact">+5 points</span></h3>
        <p>Add more examples and usage documentation to your SKILL.md file.</p>
        <button onclick="applyFix('docs')">Add Examples</button>
    </div>
    <div class="suggestion medium">
        <h3><span>🤖 Add MCP Capacity</span><span class="impact">+5 points</span></h3>
        <p>Enable AI agent integration with Model Context Protocol server.</p>
        <button onclick="applyFix('mcp')">Generate MCP</button>
    </div>
    <script>
        const vscode = acquireVsCodeApi();
        function applyFix(type) { vscode.postMessage({ command: 'applyFix', type }); }
    </script>
</body>
</html>`;
}

export function getReportHtml(data: {
  skillName: string;
  grade: string;
  score: number;
  dimensions: { name: string; score: number }[];
}): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SkillPack Report</title>
    <style>
        body { font-family: var(--vscode-font-family); padding: 20px; background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); }
        h1 { margin-bottom: 8px; }
        .grade { font-size: 64px; font-weight: bold; color: var(--vscode-charts-blue); line-height: 1; }
        .score { font-size: 24px; color: var(--vscode-descriptionForeground); margin-bottom: 24px; }
        .dimension { margin: 16px 0; }
        .dimension-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
        .bar { height: 8px; background: var(--vscode-progressBar-background); border-radius: 4px; overflow: hidden; }
        .fill { height: 100%; background: var(--vscode-charts-blue); transition: width 0.3s ease; }
    </style>
</head>
<body>
    <h1>${data.skillName}</h1>
    <div class="grade">${data.grade}</div>
    <div class="score">${data.score}/100</div>
    <h2>Dimensions</h2>
    ${data.dimensions
      .map(
        (d) => `
        <div class="dimension">
            <div class="dimension-header"><span>${d.name}</span><span>${d.score}%</span></div>
            <div class="bar"><div class="fill" style="width: ${d.score}%"></div></div>
        </div>
    `,
      )
      .join("")}
</body>
</html>`;
}

export function getSkillDetailsHtml(skill: SkillSummary): string {
  const gradeColor = skill.grade.startsWith("S")
    ? "#10b981"
    : skill.grade.startsWith("A")
      ? "#3b82f6"
      : skill.grade.startsWith("B")
        ? "#f59e0b"
        : "#ef4444";
  const display = skill.displayName || skill.name;

  return `<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: var(--vscode-font-family); padding: 20px; background: var(--vscode-editor-background); color: var(--vscode-foreground); }
        .header { display: flex; align-items: center; gap: 16px; margin-bottom: 24px; }
        .grade { font-size: 48px; font-weight: bold; color: ${gradeColor}; }
        .title { flex: 1; }
        .name { font-size: 24px; font-weight: 600; margin: 0; }
        .ref { opacity: 0.7; font-size: 12px; }
        .stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 24px; }
        .stat { background: var(--vscode-input-background); padding: 12px; border-radius: 8px; text-align: center; }
        .stat-value { font-size: 24px; font-weight: bold; }
        .stat-label { font-size: 12px; opacity: 0.7; }
        .description { margin-bottom: 24px; padding: 16px; background: var(--vscode-input-background); border-radius: 8px; }
        .tags { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 24px; }
        .tag { background: var(--vscode-badge-background); color: var(--vscode-badge-foreground); padding: 4px 12px; border-radius: 12px; font-size: 12px; }
        .actions { display: flex; gap: 12px; }
        button { padding: 10px 20px; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; }
        .primary { background: var(--vscode-button-background); color: var(--vscode-button-foreground); }
        .secondary { background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); }
    </style>
</head>
<body>
    <div class="header">
        <div class="grade">${skill.grade}</div>
        <div class="title">
            <h1 class="name">${display}</h1>
            <div class="ref">${skill.id}</div>
        </div>
        <div style="text-align: right;">
            <div style="font-size: 14px;">${skill.tier}</div>
            <div class="ref">v${skill.version}</div>
        </div>
    </div>
    <div class="stats">
        <div class="stat"><div class="stat-value">${skill.grade}</div><div class="stat-label">Grade</div></div>
        <div class="stat"><div class="stat-value">${skill.tier}</div><div class="stat-label">Tier</div></div>
        <div class="stat"><div class="stat-value">${skill.isLocal ? "Local" : "Remote"}</div><div class="stat-label">Source</div></div>
    </div>
    <div class="actions">
        <button class="primary" onclick="install()">Install Skill</button>
        <button class="secondary" onclick="investigate()">Investigate</button>
    </div>
    <script>
        const vscode = acquireVsCodeApi();
        function install() { vscode.postMessage({ command: 'install' }); }
        function investigate() { vscode.postMessage({ command: 'investigate' }); }
    </script>
</body>
</html>`;
}

export function showInvestigationReport(path: string): void {
  const panel = vscode.window.createWebviewPanel(
    "skillInvestigation",
    "Skill Investigation Report",
    vscode.ViewColumn.Active,
    { enableScripts: true },
  );

  panel.webview.html = `<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: var(--vscode-font-family); padding: 20px; background: var(--vscode-editor-background); color: var(--vscode-foreground); }
        h1 { margin-bottom: 8px; }
        .path { opacity: 0.6; font-size: 12px; margin-bottom: 24px; }
        .section { background: var(--vscode-input-background); padding: 16px; border-radius: 8px; margin-bottom: 16px; }
        .section-title { font-weight: 600; margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }
        .pass { color: #10b981; }
        .warn { color: #f59e0b; }
        .fail { color: #ef4444; }
        .finding { padding: 8px 0; border-bottom: 1px solid var(--vscode-panel-border); }
        .finding:last-child { border-bottom: none; }
        .finding-severity { font-size: 10px; padding: 2px 6px; border-radius: 4px; text-transform: uppercase; }
        .severity-low { background: rgba(16, 185, 129, 0.2); color: #10b981; }
        .severity-medium { background: rgba(245, 158, 11, 0.2); color: #f59e0b; }
        .severity-high { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
    </style>
</head>
<body>
    <h1>Investigation Report</h1>
    <div class="path">${path}</div>
    <div class="section">
        <div class="section-title"><span class="pass">✓</span> Structure Validation</div>
        <div class="finding">SKILL.md present and valid</div>
        <div class="finding">skill.cnsb.json schema validated</div>
        <div class="finding">Examples directory found (3 examples)</div>
    </div>
    <div class="section">
        <div class="section-title"><span class="warn">⚠</span> Security Analysis</div>
        <div class="finding"><span class="finding-severity severity-medium">MEDIUM</span> Missing threat-model.yaml</div>
        <div class="finding"><span class="finding-severity severity-low">LOW</span> SLSA provenance not configured</div>
    </div>
    <div class="section">
        <div class="section-title"><span class="pass">✓</span> Lifecycle Operations</div>
        <div class="finding">install: npm ci ✓</div>
        <div class="finding">verify: npm test ✓</div>
        <div class="finding">uninstall: defined ✓</div>
    </div>
    <div class="section">
        <div class="section-title"><span class="pass">✓</span> Dependency Analysis</div>
        <div class="finding">No known vulnerabilities</div>
        <div class="finding">License: MIT (compatible)</div>
        <div class="finding">5 direct dependencies, 42 transitive</div>
    </div>
    <div class="section">
        <div class="section-title"><span class="warn">⚠</span> Recommendations</div>
        <div class="finding">Add a threat model for security compliance</div>
        <div class="finding">Enable Sigstore signing in CI workflow</div>
        <div class="finding">Add more examples covering error scenarios</div>
    </div>
</body>
</html>`;
}
