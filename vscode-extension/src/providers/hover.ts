import * as vscode from 'vscode';

export class SkillHoverProvider implements vscode.HoverProvider {
  provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): vscode.Hover | undefined {
    const range = document.getWordRangeAtPosition(position, /"[^"]+"/);
    if (!range) return undefined;

    const word = document.getText(range).replace(/"/g, '');

    const docs: Record<string, string> = {
      lifecycle:
        '**Lifecycle Operations**\n\nDefine idempotent operations for skill management:\n- `install`: Setup the skill\n- `verify`: Check skill health\n- `upgrade`: Update to new version\n- `uninstall`: Remove the skill',
      capacities:
        '**Skill Capacities**\n\nCapacity types:\n- `mcp`: Model Context Protocol server\n- `oci`: OCI registry\n- `rest`: REST API endpoint',
      metadata:
        '**Skill Metadata**\n\nRequired fields:\n- `name`: Skill identifier\n- `version`: SemVer version\n- `description`: Brief description',
      security:
        '**Security Configuration**\n\nEnable:\n- Sigstore signing\n- SLSA provenance\n- SBOM generation',
    };

    const doc = docs[word];
    if (doc) {
      return new vscode.Hover(new vscode.MarkdownString(doc));
    }

    return undefined;
  }
}
