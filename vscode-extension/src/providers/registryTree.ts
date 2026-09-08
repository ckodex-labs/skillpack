import * as vscode from "vscode";
import type { SkillSummary } from "../generated/client-model";

export class RegistryTreeProvider implements vscode.TreeDataProvider<RegistryTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<
    RegistryTreeItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private registries: Map<string, SkillSummary[]> = new Map();

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  addRegistry(name: string, skills: SkillSummary[]): void {
    this.registries.set(name, skills);
    this.refresh();
  }

  getTreeItem(element: RegistryTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: RegistryTreeItem): RegistryTreeItem[] {
    if (!element) {
      const items: RegistryTreeItem[] = [];
      for (const [name] of this.registries) {
        items.push(
          new RegistryTreeItem(
            name,
            "",
            vscode.TreeItemCollapsibleState.Expanded,
            "registry",
          ),
        );
      }
      if (items.length === 0) {
        items.push(
          new RegistryTreeItem(
            "Click to discover skills",
            "",
            vscode.TreeItemCollapsibleState.None,
            "empty",
          ),
        );
      }
      return items;
    }

    if (element.contextValue === "registry") {
      const skills = this.registries.get(element.label as string) ?? [];
      return skills.map(
        (s) =>
          new RegistryTreeItem(
            s.displayName || s.name,
            `${s.grade} \u2022 ${s.tier}`,
            vscode.TreeItemCollapsibleState.None,
            "skill",
            s.id,
          ),
      );
    }

    return [];
  }
}

export class RegistryTreeItem extends vscode.TreeItem {
  constructor(
    label: string,
    private readonly value: string,
    collapsibleState: vscode.TreeItemCollapsibleState,
    context: string,
    public readonly skillRef?: string,
  ) {
    super(label, collapsibleState);
    this.description = value;
    this.contextValue = context;

    const icons: Record<string, string> = {
      registry: "database",
      skill: "package",
      empty: "search",
    };

    this.iconPath = new vscode.ThemeIcon(icons[context] || "file");

    if (context === "skill" && skillRef) {
      this.command = {
        command: "skillpack.installSkill",
        title: "Install Skill",
        arguments: [skillRef],
      };
    } else if (context === "empty") {
      this.command = {
        command: "skillpack.discoverSkills",
        title: "Discover Skills",
      };
    }
  }
}
