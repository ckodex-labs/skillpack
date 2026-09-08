import * as vscode from 'vscode';
import {
  storeCredential,
  deleteCredential,
  listRegistries,
  trackRegistry,
  untrackRegistry,
  getCredential,
} from '../utils/ociAuth';

export async function ociLoginCommand(): Promise<void> {
  const registry = await vscode.window.showInputBox({
    prompt: 'OCI registry to log in to',
    placeHolder: 'ghcr.io',
    value: 'ghcr.io',
  });
  if (!registry) return;

  const username = await vscode.window.showInputBox({
    prompt: `Username for ${registry}`,
    placeHolder: 'username',
  });
  if (!username) return;

  const password = await vscode.window.showInputBox({
    prompt: `Password or token for ${registry}`,
    placeHolder: 'ghp_xxx or password',
    password: true,
  });
  if (!password) return;

  await vscode.window.withProgress(
    { location: vscode.ProgressLocation.Notification, title: `Logging in to ${registry}...` },
    async () => {
      await storeCredential(registry, { username, password });
      await trackRegistry(registry);
      vscode.window.showInformationMessage(`Logged in to ${registry}`);
    },
  );
}

export async function ociLogoutCommand(): Promise<void> {
  const registries = await listRegistries();
  if (registries.length === 0) {
    vscode.window.showInformationMessage('No logged-in registries');
    return;
  }

  const registry = await vscode.window.showQuickPick(
    [...registries.map((r) => ({ label: r })), { label: '$(globe) Other...' }],
    { placeHolder: 'Select registry to log out from' },
  );
  if (!registry) return;

  let target = registry.label;
  if (target === '$(globe) Other...') {
    const input = await vscode.window.showInputBox({
      prompt: 'Registry to log out from',
      placeHolder: 'ghcr.io',
    });
    if (!input) return;
    target = input;
  }

  await deleteCredential(target);
  await untrackRegistry(target);
  vscode.window.showInformationMessage(`Logged out from ${target}`);
}

export async function ociListRegistriesCommand(): Promise<void> {
  const registries = await listRegistries();
  if (registries.length === 0) {
    vscode.window.showInformationMessage('No logged-in registries');
    return;
  }

  const items = await Promise.all(
    registries.map(async (r) => {
      const cred = await getCredential(r);
      return {
        label: `$(database) ${r}`,
        description: cred?.username ? `user: ${cred.username}` : 'token auth',
      };
    }),
  );

  await vscode.window.showQuickPick(items, { placeHolder: 'Logged-in registries' });
}
