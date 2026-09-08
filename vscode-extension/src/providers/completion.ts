import * as vscode from 'vscode';

export class SkillCompletionProvider implements vscode.CompletionItemProvider {
  provideCompletionItems(
    _document: vscode.TextDocument,
    _position: vscode.Position,
  ): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];

    const lifecycleItem = new vscode.CompletionItem(
      'lifecycle',
      vscode.CompletionItemKind.Snippet,
    );
    lifecycleItem.insertText = new vscode.SnippetString(`"lifecycle": {
  "install": {
    "command": "\${1:npm ci}",
    "timeout": "\${2:5m}"
  },
  "verify": {
    "command": "\${3:npm test}"
  },
  "uninstall": {
    "command": "\${4:rm -rf node_modules}"
  }
}`);
    lifecycleItem.documentation = 'Add lifecycle operations (install, verify, uninstall)';
    items.push(lifecycleItem);

    const mcpItem = new vscode.CompletionItem(
      'capacity:mcp',
      vscode.CompletionItemKind.Snippet,
    );
    mcpItem.insertText = new vscode.SnippetString(`{
  "type": "mcp",
  "server": {
    "command": "\${1:node}",
    "args": ["\${2:capacities/mcp/server.js}"]
  },
  "tools": [
    {
      "name": "\${3:skill_action}",
      "description": "\${4:Performs an action}"
    }
  ]
}`);
    mcpItem.documentation = 'Add MCP (Model Context Protocol) capacity';
    items.push(mcpItem);

    const exampleItem = new vscode.CompletionItem(
      'example',
      vscode.CompletionItemKind.Snippet,
    );
    exampleItem.insertText = new vscode.SnippetString(`{
  "name": "\${1:basic-usage}",
  "description": "\${2:Basic usage example}",
  "code": "\${3:// Example code here}"
}`);
    exampleItem.documentation = 'Add an example';
    items.push(exampleItem);

    const securityItem = new vscode.CompletionItem(
      'security',
      vscode.CompletionItemKind.Snippet,
    );
    securityItem.insertText = new vscode.SnippetString(`"security": {
  "attestation": {
    "sigstore": true,
    "slsa_level": \${1:3}
  },
  "sbom": {
    "format": "cyclonedx",
    "generate_on_build": true
  }
}`);
    securityItem.documentation = 'Add security configuration';
    items.push(securityItem);

    return items;
  }
}

export class SkillMarkdownCompletionProvider implements vscode.CompletionItemProvider {
  provideCompletionItems(): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];

    const sections = [
      { name: 'Description', desc: 'Brief description of the skill' },
      { name: 'Usage', desc: 'How to use the skill' },
      { name: 'Examples', desc: 'Usage examples' },
      { name: 'Configuration', desc: 'Configuration options' },
      { name: 'Security', desc: 'Security considerations' },
      { name: 'Changelog', desc: 'Version history' },
    ];

    for (const section of sections) {
      const item = new vscode.CompletionItem(
        `## ${section.name}`,
        vscode.CompletionItemKind.Snippet,
      );
      item.insertText = new vscode.SnippetString(`## ${section.name}\n\n\${0}`);
      item.documentation = section.desc;
      items.push(item);
    }

    return items;
  }
}
