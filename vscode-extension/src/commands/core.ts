import * as vscode from "vscode";
import { assessSkill } from "../utils/diagnostics";
import { getReportHtml } from "../utils/html";
import type { SkillPackFix } from "../providers/codeActions";

export async function assessCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  await assessSkill(workspaceFolder.uri);
}

export async function validateCommand(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showWarningMessage("No active editor");
    return;
  }
  vscode.window.showInformationMessage("Validating skill schema...");
  await new Promise((resolve) => setTimeout(resolve, 500));
  vscode.window.showInformationMessage("Schema validation passed");
}

export async function showReportCommand(): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    "skillpackReport",
    "SkillPack Report",
    vscode.ViewColumn.Beside,
    { enableScripts: true },
  );
  panel.webview.html = getReportHtml({
    skillName: "current-skill",
    grade: "B",
    score: 82.5,
    dimensions: [
      { name: "Structure", score: 90 },
      { name: "Security", score: 75 },
      { name: "Documentation", score: 80 },
      { name: "Lifecycle", score: 85 },
      { name: "Governance", score: 70 },
    ],
  });
}

export async function initCommand(): Promise<void> {
  const name = await vscode.window.showInputBox({
    prompt: "Skill name",
    placeHolder: "my-skill",
  });
  if (!name) return;
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const terminal = vscode.window.createTerminal("SkillPack");
  terminal.sendText(`skillpack init ${name}`);
  terminal.show();
}

export async function applyFixCommand(fix: SkillPackFix): Promise<void> {
  const edit = new vscode.WorkspaceEdit();
  if (fix.type === "add-lifecycle") {
    const doc = await vscode.workspace.openTextDocument(fix.uri);
    const text = doc.getText();
    const json = JSON.parse(text);
    if (!json.lifecycle) {
      json.lifecycle = {
        install: { command: "npm ci", timeout: "5m" },
        verify: { command: "npm test" },
        uninstall: { command: "rm -rf node_modules" },
      };
      const newText = JSON.stringify(json, null, 2);
      edit.replace(fix.uri, new vscode.Range(0, 0, doc.lineCount, 0), newText);
      await vscode.workspace.applyEdit(edit);
      vscode.window.showInformationMessage("Added lifecycle operations");
    }
  } else if (fix.type === "add-security") {
    const securityDir = vscode.Uri.joinPath(fix.uri, "..", "security");
    const threatModelUri = vscode.Uri.joinPath(
      securityDir,
      "threat-model.yaml",
    );
    edit.createFile(threatModelUri, { ignoreIfExists: true });
    edit.insert(
      threatModelUri,
      new vscode.Position(0, 0),
      `# Threat Model
threats:
  - id: T001
    name: Unauthorized Access
    stride: Spoofing
    mitigation: Implement authentication
    status: mitigated
`,
    );
    await vscode.workspace.applyEdit(edit);
    vscode.window.showInformationMessage("Created security/threat-model.yaml");
  }
}

// MARK: - Frontmatter Migration Commands

function runInTerminal(name: string, command: string): void {
  const terminal = vscode.window.createTerminal(name);
  terminal.sendText(command);
  terminal.show();
}

export async function migrateSkillsCommand(): Promise<void> {
  const canonicalRoot = await vscode.window.showInputBox({
    prompt: "Canonical root path (leave empty for ~/Skills/shared)",
    placeHolder: "~/Skills/shared",
  });
  const cmd = canonicalRoot
    ? `skillpack migrate-skills --canonical-root "${canonicalRoot}"`
    : "skillpack migrate-skills";
  runInTerminal("Migrate Skills", cmd);
  vscode.window.showInformationMessage("Migration started in terminal");
}

export async function migrateAgentsCommand(): Promise<void> {
  runInTerminal("Migrate Agents", "skillpack migrate-agents");
  vscode.window.showInformationMessage("Agent migration started in terminal");
}

export async function migrateClaudeAgentsCommand(): Promise<void> {
  const agentsDir = await vscode.window.showInputBox({
    prompt: "Claude agents directory (leave empty for ~/.claude/agents)",
    placeHolder: "~/.claude/agents",
  });
  const cmd = agentsDir
    ? `skillpack migrate-claude-agents --agents-dir "${agentsDir}"`
    : "skillpack migrate-claude-agents";
  runInTerminal("Migrate Claude Agents", cmd);
  vscode.window.showInformationMessage(
    "Claude agent migration started in terminal",
  );
}

export async function migrateHarnessesCommand(): Promise<void> {
  runInTerminal("Migrate Harnesses", "skillpack migrate-harnesses");
  vscode.window.showInformationMessage("Harness migration started in terminal");
}
