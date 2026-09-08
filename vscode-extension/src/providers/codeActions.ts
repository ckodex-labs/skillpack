import * as vscode from 'vscode';

export interface SkillPackFix {
  type: 'add-lifecycle' | 'add-security' | 'add-mcp';
  uri: vscode.Uri;
}

export class SkillPackCodeActionProvider implements vscode.CodeActionProvider {
  static readonly providedCodeActionKinds = [
    vscode.CodeActionKind.QuickFix,
    vscode.CodeActionKind.Source,
  ];

  provideCodeActions(
    document: vscode.TextDocument,
    _range: vscode.Range,
    context: vscode.CodeActionContext,
  ): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];

    for (const diagnostic of context.diagnostics) {
      if (diagnostic.source !== 'SkillPack') continue;

      if (diagnostic.message.includes('lifecycle')) {
        const action = new vscode.CodeAction(
          '$(gear) Add lifecycle operations',
          vscode.CodeActionKind.QuickFix,
        );
        action.command = {
          command: 'skillpack.applyFix',
          title: 'Add lifecycle operations',
          arguments: [{ type: 'add-lifecycle', uri: document.uri }],
        };
        action.isPreferred = true;
        actions.push(action);
      }

      if (
        diagnostic.message.includes('security') ||
        diagnostic.message.includes('threat')
      ) {
        const action = new vscode.CodeAction(
          '$(shield) Create threat model',
          vscode.CodeActionKind.QuickFix,
        );
        action.command = {
          command: 'skillpack.applyFix',
          title: 'Create threat model',
          arguments: [{ type: 'add-security', uri: document.uri }],
        };
        actions.push(action);
      }
    }

    const askAI = new vscode.CodeAction(
      '$(hubot) Ask AI for help',
      vscode.CodeActionKind.Source,
    );
    askAI.command = { command: 'skillpack.askAI', title: 'Ask AI' };
    actions.push(askAI);

    const improve = new vscode.CodeAction(
      '$(lightbulb) Improve skill quality',
      vscode.CodeActionKind.Source,
    );
    improve.command = {
      command: 'skillpack.improveSkill',
      title: 'Improve Skill',
    };
    actions.push(improve);

    return actions;
  }
}
