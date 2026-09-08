import * as vscode from "vscode";
import { state } from "./utils/state";
import { assessSkill, updateWorkspaceContext } from "./utils/diagnostics";
import {
  SkillPackCodeActionProvider,
  SkillCompletionProvider,
  SkillMarkdownCompletionProvider,
  SkillHoverProvider,
  SkillPackTreeProvider,
  RegistryTreeProvider,
  OciRegistryTreeProvider,
} from "./providers";
import {
  assessCommand,
  validateCommand,
  showReportCommand,
  initCommand,
  applyFixCommand,
  migrateSkillsCommand,
  migrateAgentsCommand,
  migrateClaudeAgentsCommand,
  migrateHarnessesCommand,
  packageCommand,
  publishCommand,
  evalCommand,
  newSkillWizard,
  askAICommand,
  improveSkillCommand,
  generateCapacityCommand,
  addExampleCommand,
  generateTestsCommand,
  openDocsCommand,
  discoverSkillsCommand,
  searchRegistryCommand,
  installSkillCommand,
  investigateSkillCommand,
  refreshRegistryCommand,
  ociLoginCommand,
  ociLogoutCommand,
  ociListRegistriesCommand,
} from "./commands";

export function activate(context: vscode.ExtensionContext): void {
  console.log("SkillPack extension activated");
  state.extensionContext = context;

  state.diagnosticCollection =
    vscode.languages.createDiagnosticCollection("skillpack");
  context.subscriptions.push(state.diagnosticCollection);

  state.statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100,
  );
  state.statusBarItem.command = "skillpack.showReport";
  context.subscriptions.push(state.statusBarItem);

  context.subscriptions.push(
    vscode.commands.registerCommand("skillpack.assess", assessCommand),
    vscode.commands.registerCommand("skillpack.validate", validateCommand),
    vscode.commands.registerCommand("skillpack.showReport", showReportCommand),
    vscode.commands.registerCommand("skillpack.init", initCommand),
    vscode.commands.registerCommand("skillpack.applyFix", applyFixCommand),
    vscode.commands.registerCommand(
      "skillpack.migrateSkills",
      migrateSkillsCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.migrateAgents",
      migrateAgentsCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.migrateClaudeAgents",
      migrateClaudeAgentsCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.migrateHarnesses",
      migrateHarnessesCommand,
    ),
    vscode.commands.registerCommand("skillpack.newSkill", newSkillWizard),
    vscode.commands.registerCommand("skillpack.askAI", askAICommand),
    vscode.commands.registerCommand(
      "skillpack.improveSkill",
      improveSkillCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.generateCapacity",
      generateCapacityCommand,
    ),
    vscode.commands.registerCommand("skillpack.addExample", addExampleCommand),
    vscode.commands.registerCommand(
      "skillpack.generateTests",
      generateTestsCommand,
    ),
    vscode.commands.registerCommand("skillpack.openDocs", openDocsCommand),
    vscode.commands.registerCommand(
      "skillpack.discoverSkills",
      discoverSkillsCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.searchRegistry",
      searchRegistryCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.installSkill",
      installSkillCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.investigateSkill",
      investigateSkillCommand,
    ),
    vscode.commands.registerCommand(
      "skillpack.refreshRegistry",
      refreshRegistryCommand,
    ),
    vscode.commands.registerCommand("skillpack.package", packageCommand),
    vscode.commands.registerCommand("skillpack.publish", publishCommand),
    vscode.commands.registerCommand("skillpack.eval", evalCommand),
    vscode.commands.registerCommand("skillpack.ociLogin", ociLoginCommand),
    vscode.commands.registerCommand("skillpack.ociLogout", ociLogoutCommand),
    vscode.commands.registerCommand(
      "skillpack.ociListRegistries",
      ociListRegistriesCommand,
    ),
  );

  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider(
      [{ language: "markdown" }, { language: "json" }, { language: "yaml" }],
      new SkillPackCodeActionProvider(),
      {
        providedCodeActionKinds:
          SkillPackCodeActionProvider.providedCodeActionKinds,
      },
    ),
  );

  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      { language: "json", pattern: "**/skill.cnsb.json" },
      new SkillCompletionProvider(),
      '"',
      ":",
    ),
    vscode.languages.registerCompletionItemProvider(
      { language: "markdown", pattern: "**/SKILL.md" },
      new SkillMarkdownCompletionProvider(),
      "#",
      "-",
    ),
  );

  context.subscriptions.push(
    vscode.languages.registerHoverProvider(
      [{ language: "json", pattern: "**/skill.cnsb.json" }],
      new SkillHoverProvider(),
    ),
  );

  const treeProvider = new SkillPackTreeProvider();
  vscode.window.createTreeView("skillpackView", {
    treeDataProvider: treeProvider,
  });
  context.subscriptions.push(
    vscode.commands.registerCommand("skillpack.refreshView", () =>
      treeProvider.refresh(),
    ),
  );

  state.registryTreeProvider = new RegistryTreeProvider();
  vscode.window.createTreeView("skillpackRegistryView", {
    treeDataProvider: state.registryTreeProvider,
  });

  const ociTreeProvider = new OciRegistryTreeProvider();
  vscode.window.createTreeView("skillpackOciView", {
    treeDataProvider: ociTreeProvider,
  });
  context.subscriptions.push(
    vscode.commands.registerCommand("skillpack.refreshOciView", () =>
      ociTreeProvider.refresh(),
    ),
  );

  const config = vscode.workspace.getConfiguration("skillpack");
  if (config.get("autoAssess")) {
    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument(
        async (doc: vscode.TextDocument) => {
          if (
            doc.fileName.endsWith("SKILL.md") ||
            doc.fileName.endsWith("skill.cnsb.json") ||
            doc.fileName.endsWith("eval.yml")
          ) {
            await assessSkill(doc.uri);
          }
        },
      ),
    );
  }

  updateWorkspaceContext();
}

export function deactivate(): void {
  state.diagnosticCollection?.dispose();
  state.statusBarItem?.dispose();
  state.aiChatPanel?.dispose();
}
