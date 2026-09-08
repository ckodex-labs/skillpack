import * as vscode from 'vscode';
import { listRegistries } from '../utils/ociAuth';

export class OciRegistryTreeProvider implements vscode.TreeDataProvider<OciRegistryTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<OciRegistryTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: OciRegistryTreeItem): vscode.TreeItem {
    return element;
  }

  async getChildren(_element?: OciRegistryTreeItem): Promise<OciRegistryTreeItem[]> {
    const registries = await listRegistries();
    if (registries.length === 0) {
      return [
        new OciRegistryTreeItem(
          'No logged-in registries',
          '',
          vscode.TreeItemCollapsibleState.None,
          'empty',
        ),
      ];
    }
    return registries.map(
      (r: string) => new OciRegistryTreeItem(r, '', vscode.TreeItemCollapsibleState.None, 'registry'),
    );
  }
}

export class OciRegistryTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    value: string,
    collapsibleState: vscode.TreeItemCollapsibleState,
    context: string,
  ) {
    super(label, collapsibleState);
    this.description = value;
    this.contextValue = context;
    const icons: Record<string, string> = {
      registry: 'database',
      empty: 'info',
    };
    this.iconPath = new vscode.ThemeIcon(icons[context] ?? 'circle-outline');
    if (context === 'empty') {
      this.command = {
        command: 'skillpack.ociLogin',
        title: 'Login to Registry',
      };
    }
  }
}
