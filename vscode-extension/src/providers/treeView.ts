import * as vscode from 'vscode';

export interface MockAssessment {
  grade: string;
  score: number;
  dimensions: { name: string; score: number }[];
  issues: { message: string; severity: string }[];
}

export class SkillPackTreeProvider implements vscode.TreeDataProvider<SkillPackTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<SkillPackTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private assessment: MockAssessment | null = null;

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  setAssessment(assessment: MockAssessment): void {
    this.assessment = assessment;
    this.refresh();
  }

  getTreeItem(element: SkillPackTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: SkillPackTreeItem): SkillPackTreeItem[] {
    if (!element) {
      return [
        new SkillPackTreeItem('Grade', this.assessment?.grade ?? '—', vscode.TreeItemCollapsibleState.None, 'grade'),
        new SkillPackTreeItem('Score', `${this.assessment?.score ?? 0}/100`, vscode.TreeItemCollapsibleState.None, 'score'),
        new SkillPackTreeItem('Dimensions', '', vscode.TreeItemCollapsibleState.Expanded, 'dimensions'),
        new SkillPackTreeItem('Issues', `${this.assessment?.issues?.length ?? 0}`, vscode.TreeItemCollapsibleState.Collapsed, 'issues'),
        new SkillPackTreeItem('AI Assistant', 'Ask for help', vscode.TreeItemCollapsibleState.None, 'ai'),
      ];
    }

    if (element.contextValue === 'dimensions') {
      return (this.assessment?.dimensions ?? []).map(
        (d) => new SkillPackTreeItem(d.name, `${d.score}%`, vscode.TreeItemCollapsibleState.None, 'dimension'),
      );
    }

    if (element.contextValue === 'issues') {
      return (this.assessment?.issues ?? []).map(
        (i) => new SkillPackTreeItem(i.message, i.severity, vscode.TreeItemCollapsibleState.None, 'issue'),
      );
    }

    return [];
  }
}

export class SkillPackTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    private readonly value: string,
    collapsibleState: vscode.TreeItemCollapsibleState,
    context: string,
  ) {
    super(label, collapsibleState);
    this.description = value;
    this.contextValue = context;

    const icons: Record<string, string> = {
      grade: 'star-full',
      score: 'symbol-number',
      dimensions: 'list-tree',
      issues: 'warning',
      dimension: 'check',
      issue: 'issue-opened',
      ai: 'hubot',
    };

    this.iconPath = new vscode.ThemeIcon(icons[context] ?? 'circle-outline');

    if (context === 'ai') {
      this.command = { command: 'skillpack.askAI', title: 'Ask AI' };
    }
  }
}
