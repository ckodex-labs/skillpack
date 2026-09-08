import * as vscode from "vscode";
import type { RegistryTreeProvider } from "../providers/registryTree";

export const state = {
  extensionContext: undefined as vscode.ExtensionContext | undefined,
  diagnosticCollection: undefined as vscode.DiagnosticCollection | undefined,
  statusBarItem: undefined as vscode.StatusBarItem | undefined,
  aiChatPanel: undefined as vscode.WebviewPanel | undefined,
  registryTreeProvider: undefined as RegistryTreeProvider | undefined,
};

// Backward-compatible direct exports
export const diagnosticCollection = () => state.diagnosticCollection!;
export const statusBarItem = () => state.statusBarItem!;
export const aiChatPanel = () => state.aiChatPanel;
export const registryTreeProvider = () => state.registryTreeProvider;
