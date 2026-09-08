import * as vscode from "vscode";
import { getCredential, hasCredential } from "../utils/ociAuth";

export async function packageCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const format = await vscode.window.showQuickPick(
    [
      { label: "$(archive) TAR + Gzip", value: "tar.gz" },
      { label: "$(archive) TAR + Bzip2", value: "tar.bz2" },
      { label: "$(archive) TAR + Zstd", value: "tar.zst" },
      { label: "$(archive) ZIP", value: "zip" },
    ],
    { placeHolder: "Select package format" },
  );
  if (!format) return;
  const terminal = vscode.window.createTerminal("SkillPack Package");
  terminal.sendText(`skillpack package --format ${format.value}`);
  terminal.show();
}

export async function publishCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const registry = await vscode.window.showInputBox({
    prompt: "OCI registry (e.g., ghcr.io/username)",
    placeHolder: "ghcr.io/my-org",
  });
  if (!registry) return;
  const sign = await vscode.window.showQuickPick(
    [
      { label: "$(check) Sign with Sigstore", value: true },
      { label: "$(x) Publish without signing", value: false },
    ],
    { placeHolder: "Signing option" },
  );
  if (sign === undefined) return;
  const terminal = vscode.window.createTerminal("SkillPack Publish");
  const signFlag = sign.value ? "--sign" : "";

  // Use stored credentials if available
  const registryHost = registry.split("/")[0];
  const hasCreds = await hasCredential(registryHost);
  if (hasCreds) {
    const cred = await getCredential(registryHost);
    if (cred?.username && cred.password) {
      terminal.sendText(
        `skillpack publish --registry ${registry} --username ${cred.username} --password ${cred.password} ${signFlag} --no-check`,
      );
      terminal.show();
      return;
    }
  }

  terminal.sendText(
    `skillpack publish --registry ${registry} ${signFlag} --no-check`,
  );
  terminal.show();
}

export async function evalCommand(): Promise<void> {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder open");
    return;
  }
  const suite = await vscode.window.showQuickPick(
    [
      { label: "$(flame) Smoke Suite", value: "smoke" },
      { label: "$(shield) Compliance Suite", value: "compliance" },
      { label: "$(list-flat) All Suites", value: "" },
    ],
    { placeHolder: "Select evaluation suite" },
  );
  if (!suite) return;
  const terminal = vscode.window.createTerminal("SkillPack Eval");
  const suiteFlag = suite.value ? `--suite ${suite.value}` : "";
  terminal.sendText(`skillpack eval ${suiteFlag}`);
  terminal.show();
}
