import * as vscode from 'vscode';

const SERVICE_PREFIX = 'skillpack-oci';

export interface OciCredential {
  username?: string;
  password?: string;
  token?: string;
}

export async function storeCredential(registry: string, credential: OciCredential): Promise<void> {
  const key = `${SERVICE_PREFIX}:${registry}`;
  await vscode.workspace.getConfiguration().update(key, JSON.stringify(credential), true);
}

export async function getCredential(registry: string): Promise<OciCredential | null> {
  const key = `${SERVICE_PREFIX}:${registry}`;
  const value = vscode.workspace.getConfiguration().get<string>(key);
  if (!value) return null;
  try {
    return JSON.parse(value) as OciCredential;
  } catch {
    return null;
  }
}

export async function deleteCredential(registry: string): Promise<void> {
  const key = `${SERVICE_PREFIX}:${registry}`;
  await vscode.workspace.getConfiguration().update(key, undefined, true);
}

export async function listRegistries(): Promise<string[]> {
  const config = vscode.workspace.getConfiguration();
  const listKey = `${SERVICE_PREFIX}:__registries`;
  return config.get<string[]>(listKey) ?? [];
}

export async function trackRegistry(registry: string): Promise<void> {
  const listKey = `${SERVICE_PREFIX}:__registries`;
  const config = vscode.workspace.getConfiguration();
  const list = config.get<string[]>(listKey) ?? [];
  if (!list.includes(registry)) {
    await config.update(listKey, [...list, registry], true);
  }
}

export async function untrackRegistry(registry: string): Promise<void> {
  const listKey = `${SERVICE_PREFIX}:__registries`;
  const config = vscode.workspace.getConfiguration();
  const list = config.get<string[]>(listKey) ?? [];
  await config.update(listKey, list.filter((r: string) => r !== registry), true);
}

export async function hasCredential(registry: string): Promise<boolean> {
  return (await getCredential(registry)) !== null;
}
