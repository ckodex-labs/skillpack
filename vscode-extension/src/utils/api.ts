import * as vscode from "vscode";
import type {
  SkillSummary,
  SkillDetail,
  RegistryEntry,
  ClientError,
} from "../generated/client-model";

/** Per-client rendering hint for ClientError.vscode */
export interface VscodeErrorHint {
  message: string;
  button?: { label: string; command: string };
}

/** Render a structured ClientError for VS Code per CLIENT-SPEC.md §5.2. */
export function renderClientErrorVscode(err: ClientError): void {
  const hint = err.clientHint?.vscode;
  switch (err.severity) {
    case "fatal":
      vscode.window.showErrorMessage(`SkillPack Fatal: ${err.message}`);
      break;
    case "error":
      if (hint) {
        vscode.window
          .showErrorMessage(err.message, "View Details")
          .then((choice) => {
            if (choice === "View Details") {
              vscode.window.showInformationMessage(hint);
            }
          });
      } else {
        vscode.window.showErrorMessage(err.message);
      }
      break;
    case "warning":
      vscode.window.showWarningMessage(err.message);
      break;
    case "info":
      vscode.window.showInformationMessage(err.message);
      break;
  }
}

export class SkillPackApiClient {
  private baseUrl: string;

  constructor() {
    // CLIENT-SPEC.md §3.1: Discovery via SKILLPACK_API_URL env var, fallback to config
    const envUrl = process.env.SKILLPACK_API_URL;
    const config = vscode.workspace.getConfiguration("skillpack");
    this.baseUrl =
      envUrl || config.get("serverUrl") || "http://localhost:50051";
  }

  /// CLIENT-SPEC.md §6.1: Env var takes priority; fallback to VS Code SecretStorage.
  private async getToken(): Promise<string | undefined> {
    if (process.env.SKILLPACK_TOKEN) {
      return process.env.SKILLPACK_TOKEN;
    }
    const secrets = (await import("./state")).state.extensionContext?.secrets;
    if (secrets) {
      return secrets.get("skillpack.token");
    }
    return undefined;
  }

  private async fetchWithAuth(
    path: string,
    options: RequestInit = {},
  ): Promise<Response> {
    const headers: Record<string, string> = {
      ...((options.headers as Record<string, string>) || {}),
    };
    const token = await this.getToken();
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }
    return fetch(`${this.baseUrl}${path}`, { ...options, headers });
  }

  /** CLIENT-SPEC.md §2.3: maximum protocol version this client supports. */
  private readonly SUPPORTED_PROTOCOL_VERSION = 1;

  /** CLIENT-SPEC.md §3.1: Health probe with 2s timeout + capability negotiation. */
  async healthProbe(): Promise<{ healthy: boolean; protocolVersion?: number }> {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 2000);
      const resp = await fetch(`${this.baseUrl}/health`, {
        signal: controller.signal,
      });
      clearTimeout(timeout);
      if (!resp.ok) return { healthy: false };
      const body = (await resp.json().catch(() => ({}))) as Record<
        string,
        unknown
      >;
      const protocolVersion =
        typeof body.protocolVersion === "number"
          ? body.protocolVersion
          : undefined;
      if (
        protocolVersion !== undefined &&
        protocolVersion > this.SUPPORTED_PROTOCOL_VERSION
      ) {
        return { healthy: false, protocolVersion };
      }
      return { healthy: true, protocolVersion };
    } catch {
      return { healthy: false };
    }
  }

  async search(
    query: string,
    filters?: { minScore?: number; minGrade?: string; tags?: string[] },
  ): Promise<SkillSummary[]> {
    const params = new URLSearchParams();
    if (query) params.set("q", query);
    if (filters?.minScore) params.set("min_score", String(filters.minScore));
    if (filters?.minGrade) params.set("min_grade", filters.minGrade);
    if (filters?.tags) params.set("tags", filters.tags.join(","));

    const response = await this.fetchWithAuth(
      `/api/v1/skills/search?${params}`,
    );
    if (!response.ok) throw new Error(`API error: ${response.status}`);
    const data = (await response.json()) as { skills: SkillSummary[] };
    return data.skills || [];
  }

  async getSkillDetail(skillRef: string): Promise<SkillDetail | null> {
    const response = await this.fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(skillRef)}`,
    );
    if (!response.ok) return null;
    return (await response.json()) as SkillDetail;
  }

  async discover(
    registryUrl: string,
  ): Promise<{ skills: RegistryEntry[] } | null> {
    try {
      const response = await fetch(`${registryUrl}/.well-known/skills.json`);
      if (!response.ok) return null;
      return (await response.json()) as { skills: RegistryEntry[] };
    } catch {
      return null;
    }
  }
}

export const apiClient = new SkillPackApiClient();
