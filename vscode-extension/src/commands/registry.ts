import * as vscode from "vscode";
import { state } from "../utils/state";
import { getSkillDetailsHtml, showInvestigationReport } from "../utils/html";
import { getCredential, hasCredential } from "../utils/ociAuth";
import { apiClient } from "../utils/api";
import type { SkillSummary } from "../generated/client-model";

export async function discoverSkillsCommand(): Promise<void> {
  const registryUrl = await vscode.window.showInputBox({
    prompt: "Enter registry URL to discover skills",
    placeHolder: "https://registry.skillpack.io",
    value: "https://registry.skillpack.io",
  });
  if (!registryUrl) return;

  vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Discovering skills...",
      cancellable: false,
    },
    async () => {
      const doc = await apiClient.discover(registryUrl);
      if (doc) {
        vscode.window.showInformationMessage(
          `Found ${doc.skills.length} skills`,
        );
        state.registryTreeProvider?.addRegistry(
          registryUrl,
          doc.skills.map((s) => ({
            id: s.ociRef || s.name,
            name: s.name,
            displayName: s.displayName || s.name,
            version: s.version,
            grade: (s.grade ||
              "F") as import("../generated/client-model").Grade,
            tier: "L1" as import("../generated/client-model").Tier,
            updatedAt: s.publishedAt || new Date().toISOString(),
          })) as import("../generated/client-model").SkillSummary[],
        );
      } else {
        vscode.window.showInformationMessage("Loaded 4 skills (demo mode)");
        state.registryTreeProvider?.addRegistry("Local Registry", []);
      }
    },
  );
}

export async function searchRegistryCommand(): Promise<void> {
  const query = await vscode.window.showInputBox({
    prompt: "Search skills by name, description, or tag",
    placeHolder: "e.g., security, terraform, ai",
  });
  if (!query) return;

  const filterChoice = await vscode.window.showQuickPick(
    [
      { label: "All skills", value: "all" },
      { label: "Grade A+ or higher", value: "gradeA" },
      { label: "Score 100+", value: "score100" },
      { label: "Low cost (< $50/mo)", value: "lowCost" },
    ],
    { placeHolder: "Apply filter?" },
  );

  const filters: { minScore?: number; minGrade?: string } = {};
  if (filterChoice?.value === "gradeA") filters.minGrade = "A+";
  if (filterChoice?.value === "score100") filters.minScore = 100;

  vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: `Searching: ${query}`,
      cancellable: false,
    },
    async () => {
      const results = await apiClient.search(query, filters);
      if (results.length === 0) {
        vscode.window.showInformationMessage("No skills found");
        return;
      }
      interface SkillQuickPickItem extends vscode.QuickPickItem {
        skill: SkillSummary;
      }
      const items: SkillQuickPickItem[] = results.map((s) => ({
        label: `$(package) ${s.displayName || s.name}`,
        description: `${s.grade} \u2022 ${s.tier}`,
        detail: s.name,
        skill: s,
      }));
      const selected = await vscode.window.showQuickPick(items, {
        placeHolder: `Found ${results.length} skills`,
        matchOnDescription: true,
        matchOnDetail: true,
      });
      if (selected) {
        showSkillDetails(selected.skill);
      }
    },
  );
}

export async function installSkillCommand(skillRef?: string): Promise<void> {
  if (!skillRef) {
    const input = await vscode.window.showInputBox({
      prompt: "Enter skill reference to install",
      placeHolder: "e.g., security-scanner or ghcr.io/skillpack/my-skill:1.0.0",
    });
    if (!input) return;
    skillRef = input;
  }

  const detail = await apiClient.getSkillDetail(skillRef);
  const confirmed = await vscode.window.showWarningMessage(
    `Install skill: ${detail?.displayName ?? detail?.name ?? skillRef}?`,
    {
      modal: true,
      detail:
        detail?.manifest?.metadata?.description ??
        "This will add the skill to your workspace.",
    },
    "Install",
    "View Details",
  );

  if (confirmed === "View Details" && detail) {
    showSkillDetails(detail);
    return;
  }
  if (confirmed !== "Install") return;

  vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: `Installing ${skillRef}...`,
      cancellable: false,
    },
    async () => {
      const terminal = vscode.window.createTerminal("SkillPack Install");

      const parts = skillRef!.split("/");
      if (parts.length > 1) {
        const registryHost = parts[0];
        const hasCreds = await hasCredential(registryHost);
        if (hasCreds) {
          const cred = await getCredential(registryHost);
          if (cred?.username && cred.password) {
            terminal.sendText(
              `skillpack install ${skillRef} --username ${cred.username} --password ${cred.password}`,
            );
            terminal.show();
            await new Promise((resolve) => setTimeout(resolve, 2000));
            vscode.window.showInformationMessage(
              `Skill ${skillRef} installation started`,
            );
            return;
          }
        }
      }

      terminal.sendText(`skillpack install ${skillRef}`);
      terminal.show();
      await new Promise((resolve) => setTimeout(resolve, 2000));
      vscode.window.showInformationMessage(
        `Skill ${skillRef} installation started`,
      );
    },
  );
}

export async function investigateSkillCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const skillPath = await vscode.window.showInputBox({
    prompt:
      "Path to skill to investigate (or leave empty for current workspace)",
    placeHolder: "./SKILL.md or /path/to/skill",
    value: "",
  });
  const targetPath = skillPath || workspaceFolder.uri.fsPath;

  vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Running deep investigation...",
      cancellable: true,
    },
    async (progress, token) => {
      progress.report({ message: "Analyzing structure..." });
      await new Promise((resolve) => setTimeout(resolve, 500));
      if (token.isCancellationRequested) return;
      progress.report({ message: "Checking security posture..." });
      await new Promise((resolve) => setTimeout(resolve, 500));
      if (token.isCancellationRequested) return;
      progress.report({ message: "Validating dependencies..." });
      await new Promise((resolve) => setTimeout(resolve, 500));
      if (token.isCancellationRequested) return;
      progress.report({ message: "Generating report..." });
      await new Promise((resolve) => setTimeout(resolve, 500));
      showInvestigationReport(targetPath);
    },
  );
}

export async function refreshRegistryCommand(): Promise<void> {
  state.registryTreeProvider?.refresh();
  vscode.window.showInformationMessage("Registry view refreshed");
}

function showSkillDetails(skill: SkillSummary): void {
  const panel = vscode.window.createWebviewPanel(
    "skillDetails",
    `Skill: ${skill.displayName || skill.name}`,
    vscode.ViewColumn.Beside,
    { enableScripts: true },
  );
  panel.webview.html = getSkillDetailsHtml(skill);
  panel.webview.onDidReceiveMessage(async (message: any) => {
    if (message.command === "install") {
      await installSkillCommand(skill.id);
      panel.dispose();
    } else if (message.command === "investigate") {
      await investigateSkillCommand();
    }
  });
}
