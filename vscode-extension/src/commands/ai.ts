import * as vscode from "vscode";
import { state } from "../utils/state";
import {
  getWizardHtml,
  getAIChatHtml,
  getImprovementSuggestionsHtml,
} from "../utils/html";

export async function newSkillWizard(): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    "skillpackWizard",
    "Create New Skill",
    vscode.ViewColumn.Active,
    { enableScripts: true, retainContextWhenHidden: true },
  );

  panel.webview.html = getWizardHtml();

  panel.webview.onDidReceiveMessage(async (message) => {
    if (message.command === "createSkill") {
      await createSkillFromWizard(message.data);
      panel.dispose();
    }
  });
}

export async function askAICommand(): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      { label: "$(question) How do I add MCP capacity?", value: "mcp" },
      {
        label: "$(question) How do I add lifecycle operations?",
        value: "lifecycle",
      },
      { label: "$(question) How do I add examples?", value: "examples" },
      {
        label: "$(question) How do I improve security score?",
        value: "security",
      },
      { label: "$(question) How do I publish my skill?", value: "publish" },
      { label: "$(edit) Ask a custom question...", value: "custom" },
    ],
    { placeHolder: "What would you like help with?" },
  );

  if (!choice) return;

  if (choice.value === "custom") {
    const question = await vscode.window.showInputBox({
      prompt: "Ask the AI about skill authoring",
      placeHolder: "How do I...",
    });
    if (question) {
      showAIChat(question);
    }
  } else {
    showAIChat(getPresetQuestion(choice.value));
  }
}

export async function improveSkillCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const panel = vscode.window.createWebviewPanel(
    "skillpackImprove",
    "Improve Skill Quality",
    vscode.ViewColumn.Beside,
    { enableScripts: true },
  );
  panel.webview.html = getImprovementSuggestionsHtml();
}

export async function generateCapacityCommand(): Promise<void> {
  const capacityType = await vscode.window.showQuickPick(
    [
      {
        label: "$(plug) MCP Server",
        description: "Model Context Protocol server",
        value: "mcp",
      },
      {
        label: "$(database) OCI Registry",
        description: "Container registry integration",
        value: "oci",
      },
      {
        label: "$(globe) REST API",
        description: "REST API endpoint",
        value: "rest",
      },
      {
        label: "$(terminal) CLI",
        description: "Command-line interface",
        value: "cli",
      },
      {
        label: "$(file-code) Custom",
        description: "Custom capacity type",
        value: "custom",
      },
    ],
    { placeHolder: "Select capacity type to generate" },
  );
  if (!capacityType) return;
  await generateCapacityScaffold(capacityType.value);
}

export async function addExampleCommand(): Promise<void> {
  const exampleType = await vscode.window.showQuickPick(
    [
      { label: "$(play) Basic Usage", value: "basic" },
      { label: "$(beaker) Advanced Configuration", value: "advanced" },
      { label: "$(bug) Error Handling", value: "error" },
      { label: "$(zap) Performance Optimization", value: "performance" },
    ],
    { placeHolder: "Select example type" },
  );
  if (!exampleType) return;
  await addExampleToSkill(exampleType.value);
}

export async function generateTestsCommand(): Promise<void> {
  vscode.window.showInformationMessage("Generating test scaffolding...");
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) return;
  const edit = new vscode.WorkspaceEdit();
  const testsDir = vscode.Uri.joinPath(workspaceFolder.uri, "tests");
  const testFile = vscode.Uri.joinPath(testsDir, "skill.test.ts");
  edit.createFile(testFile, { ignoreIfExists: true });
  edit.insert(
    testFile,
    new vscode.Position(0, 0),
    `import { describe, it, expect } from 'vitest';

describe('Skill Tests', () => {
    describe('Lifecycle Operations', () => {
        it('should install successfully', async () => {
            expect(true).toBe(true);
        });
        it('should verify after install', async () => {
            expect(true).toBe(true);
        });
    });
    describe('Capacity Tests', () => {
        it('should respond to health check', async () => {
            expect(true).toBe(true);
        });
    });
});
`,
  );
  await vscode.workspace.applyEdit(edit);
  vscode.window.showInformationMessage("Created tests/skill.test.ts");
}

export async function openDocsCommand(): Promise<void> {
  vscode.env.openExternal(
    vscode.Uri.parse("https://skillpack.dev/docs/authoring-guide"),
  );
}

// Scaffold generators

async function createSkillFromWizard(data: {
  name: string;
  description: string;
  author: string;
  capacities: string[];
  hasLifecycle: boolean;
}): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) return;

  const edit = new vscode.WorkspaceEdit();
  const skillMd = vscode.Uri.joinPath(workspaceFolder.uri, "SKILL.md");
  edit.createFile(skillMd, { ignoreIfExists: true });
  edit.insert(
    skillMd,
    new vscode.Position(0, 0),
    `---
name: ${data.name}
version: 0.1.0
description: ${data.description}
author: ${data.author}
---

# ${data.name}

${data.description}

## Usage

\`\`\`bash
skillpack install ${data.name}
\`\`\`

## Examples

See the \`examples/\` directory for usage examples.

## Configuration

Configure via \`.skillpack.json\` or environment variables.

## Security

This skill follows CKODEX security guidelines.
`,
  );

  const skillJson = vscode.Uri.joinPath(workspaceFolder.uri, "skill.cnsb.json");
  const manifest: any = {
    $schema: "./schemas/cnsb.schema.json",
    metadata: {
      name: data.name,
      version: "0.1.0",
      description: data.description,
      author: data.author,
    },
  };
  if (data.hasLifecycle) {
    manifest.lifecycle = {
      install: { command: "npm ci", timeout: "5m" },
      verify: { command: "npm test" },
      uninstall: { command: "rm -rf node_modules" },
    };
  }
  if (data.capacities.includes("mcp")) {
    manifest.capacities = [
      {
        type: "mcp",
        server: { command: "node", args: ["capacities/mcp/server.js"] },
      },
    ];
  }
  edit.createFile(skillJson, { ignoreIfExists: true });
  edit.insert(
    skillJson,
    new vscode.Position(0, 0),
    JSON.stringify(manifest, null, 2),
  );

  const exampleFile = vscode.Uri.joinPath(
    workspaceFolder.uri,
    "examples",
    "basic.md",
  );
  edit.createFile(exampleFile, { ignoreIfExists: true });
  edit.insert(
    exampleFile,
    new vscode.Position(0, 0),
    `# Basic Usage Example\n\n$\`\`\`typescript\nimport { ${data.name.replace(/-/g, "_")} } from '${data.name}';\nconst skill = new ${data.name.replace(/-/g, "_")}();\nawait skill.run();\n\`\`\`\n`,
  );

  await vscode.workspace.applyEdit(edit);
  const doc = await vscode.workspace.openTextDocument(skillMd);
  await vscode.window.showTextDocument(doc);
  vscode.window.showInformationMessage(`Created skill: ${data.name}`);
}

async function generateCapacityScaffold(type: string): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) return;

  const edit = new vscode.WorkspaceEdit();
  if (type === "mcp") {
    const serverFile = vscode.Uri.joinPath(
      workspaceFolder.uri,
      "capacities",
      "mcp",
      "server.ts",
    );
    edit.createFile(serverFile, { ignoreIfExists: true });
    edit.insert(
      serverFile,
      new vscode.Position(0, 0),
      `#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { ListToolsRequestSchema, CallToolRequestSchema } from '@modelcontextprotocol/sdk/types.js';

const server = new Server({ name: 'skill-mcp', version: '0.1.0' }, { capabilities: { tools: {} } });
server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: [{ name: 'skill_action', description: 'Performs the main skill action', inputSchema: { type: 'object', properties: { input: { type: 'string' } }, required: ['input'] } }],
}));
server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;
    if (name === 'skill_action') {
        return { content: [{ type: 'text', text: \`Processed: \${args?.input ?? ''}\` }] };
    }
    throw new Error(\`Unknown tool: \${name}\`);
});
async function main() { const transport = new StdioServerTransport(); await server.connect(transport); }
main().catch(console.error);
`,
    );
    vscode.window.showInformationMessage("Created capacities/mcp/server.ts");
  }
  await vscode.workspace.applyEdit(edit);
}

async function addExampleToSkill(type: string): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) return;

  const edit = new vscode.WorkspaceEdit();
  const exampleFile = vscode.Uri.joinPath(
    workspaceFolder.uri,
    "examples",
    `${type}.md`,
  );

  const examples: Record<string, string> = {
    basic: `# Basic Usage\n\n\`\`\`typescript\nconst result = await skill.run({ input: 'hello' });\nconsole.log(result);\n\`\`\`\n`,
    advanced: `# Advanced Configuration\n\n\`\`\`typescript\nconst skill = new Skill({ timeout: 30000, retries: 3, cache: true });\nconst result = await skill.run({ input: 'complex data', options: { streaming: true } });\n\`\`\`\n`,
    error: `# Error Handling\n\n\`\`\`typescript\ntry {\n    await skill.run({ input: 'data' });\n} catch (error) {\n    if (error instanceof SkillError) {\n        console.error('Skill error:', error.code, error.message);\n    } else { throw error; }\n}\n\`\`\`\n`,
    performance: `# Performance Optimization\n\n\`\`\`typescript\nconst results = await skill.runBatch([\n    { input: 'item1' }, { input: 'item2' }, { input: 'item3' },\n], { concurrency: 5 });\n\`\`\`\n`,
  };

  edit.createFile(exampleFile, { ignoreIfExists: true });
  edit.insert(
    exampleFile,
    new vscode.Position(0, 0),
    examples[type] ?? examples["basic"],
  );

  await vscode.workspace.applyEdit(edit);
  const doc = await vscode.workspace.openTextDocument(exampleFile);
  await vscode.window.showTextDocument(doc);
}

// AI Chat helpers

function showAIChat(initialQuestion?: string): void {
  if (state.aiChatPanel) {
    state.aiChatPanel.reveal();
  } else {
    state.aiChatPanel = vscode.window.createWebviewPanel(
      "skillpackAI",
      "SkillPack AI Assistant",
      vscode.ViewColumn.Beside,
      { enableScripts: true, retainContextWhenHidden: true },
    );
    state.aiChatPanel.onDidDispose(() => {
      state.aiChatPanel = undefined;
    });
  }

  state.aiChatPanel.webview.html = getAIChatHtml(initialQuestion);

  state.aiChatPanel.webview.onDidReceiveMessage(async (message: any) => {
    if (message.command === "sendMessage") {
      const response = await getAIResponse(message.text);
      state.aiChatPanel?.webview.postMessage({
        command: "response",
        text: response,
      });
    } else if (message.command === "applyCode") {
      await applyCodeSuggestion(message.code, message.filename);
    }
  });
}

async function getAIResponse(question: string): Promise<string> {
  const responses: Record<string, string> = {
    mcp: `To add an MCP capacity to your skill:\n\n1. Create the capacity directory:\n\`\`\`bash\nmkdir -p capacities/mcp\n\`\`\`\n\n2. Add to your skill.cnsb.json:\n\`\`\`json\n{ "capacities": [{ "type": "mcp", "server": { "command": "node", "args": ["capacities/mcp/server.js"] } }] }\n\`\`\`\n\n3. Implement the MCP server in capacities/mcp/server.js`,
    lifecycle: `Add lifecycle operations to your skill.cnsb.json:\n\`\`\`json\n{ "lifecycle": { "install": { "command": "npm ci", "timeout": "5m" }, "verify": { "command": "npm test" }, "uninstall": { "command": "rm -rf node_modules" } } }\n\`\`\``,
    security: `To improve your security score:\n\n1. Add a threat model in security/threat-model.yaml\n2. Enable Sigstore signing in your CI workflow\n3. Generate SLSA provenance at build time\n4. Add SBOM generation to your build`,
  };

  const lowerQ = question.toLowerCase();
  for (const [key, response] of Object.entries(responses)) {
    if (lowerQ.includes(key)) return response;
  }

  return `I can help you with skill authoring! Topics:\n- Adding MCP capacities\n- Configuring lifecycle operations\n- Improving security\n- Adding examples and documentation\n- Setting up tests and CI/CD`;
}

async function applyCodeSuggestion(
  code: string,
  filename: string,
): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) return;
  const edit = new vscode.WorkspaceEdit();
  const uri = vscode.Uri.joinPath(workspaceFolder.uri, filename);
  edit.createFile(uri, { ignoreIfExists: false, overwrite: true });
  edit.insert(uri, new vscode.Position(0, 0), code);
  await vscode.workspace.applyEdit(edit);
  const doc = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(doc);
  vscode.window.showInformationMessage(`Created ${filename}`);
}

function getPresetQuestion(topic: string): string {
  const questions: Record<string, string> = {
    mcp: "How do I add an MCP capacity to my skill?",
    lifecycle: "How do I add lifecycle operations?",
    examples: "How do I add examples to my skill?",
    security: "How do I improve my security score?",
    publish: "How do I publish my skill to a registry?",
  };
  return questions[topic] ?? "How do I author a skill?";
}
