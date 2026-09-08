import * as vscode from "vscode";
import { state } from "./state";
import type { MockAssessment } from "../providers/treeView";

export async function assessSkill(uri: vscode.Uri): Promise<void> {
  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Assessing skill...",
      cancellable: false,
    },
    async () => {
      await new Promise((resolve) => setTimeout(resolve, 800));

      state.diagnosticCollection!.clear();

      const assessment: MockAssessment = {
        grade: "B",
        score: 82.5,
        dimensions: [
          { name: "Structure", score: 90 },
          { name: "Security", score: 75 },
          { name: "Documentation", score: 80 },
          { name: "Lifecycle", score: 65 },
          { name: "Governance", score: 70 },
        ],
        issues: [
          { message: "Consider adding lifecycle operations", severity: "info" },
          { message: "Missing STRIDE threat model", severity: "warning" },
        ],
      };

      const diagnostics: vscode.Diagnostic[] = assessment.issues.map(
        (issue, i) => {
          const range = new vscode.Range(i, 0, i, 10);
          const severity =
            issue.severity === "warning"
              ? vscode.DiagnosticSeverity.Warning
              : vscode.DiagnosticSeverity.Information;
          const diag = new vscode.Diagnostic(range, issue.message, severity);
          diag.source = "SkillPack";
          return diag;
        },
      );

      const skillMdPath = vscode.Uri.joinPath(uri, "SKILL.md");
      state.diagnosticCollection!.set(skillMdPath, diagnostics);

      updateStatusBar(assessment.grade, assessment.score);

      vscode.window.showInformationMessage(
        `Assessment complete: Grade ${assessment.grade} (${assessment.score}/100)`,
      );
    },
  );
}

export function updateStatusBar(grade: string, score: number): void {
  const gradeIcons: Record<string, string> = {
    A: "$(star-full) ",
    B: "$(star-half) ",
    C: "$(circle-outline) ",
    D: "$(warning) ",
    F: "$(error) ",
  };

  state.statusBarItem!.text = `${gradeIcons[grade] ?? ""}SkillPack: ${grade}`;
  state.statusBarItem!.tooltip = `Skill Quality: ${score}/100\nClick for full report`;
  state.statusBarItem!.show();
}

export async function updateWorkspaceContext(): Promise<void> {
  const files = await vscode.workspace.findFiles("**/SKILL.md", null, 1);
  vscode.commands.executeCommand(
    "setContext",
    "workspaceHasSkill",
    files.length > 0,
  );

  if (files.length > 0) {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (workspaceFolder) {
      await assessSkill(workspaceFolder.uri);
    }
  }
}
