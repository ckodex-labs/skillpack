import type {
  AssessmentResult,
  DimensionScore,
  Issue,
  SkillSummary,
  ClientError,
  Grade,
} from "../generated/client-model";

// Resolution order: runtime window override → build-time public env →
// default. The default targets the server's HTTP port (SKILLPACK_HTTP_PORT,
// 50052) — the REST surface the client speaks. The gRPC port (50051) does not
// serve these REST routes, so it is the wrong default.
const API_URL =
  (typeof window !== "undefined" && (window as any).__SKILLPACK_API_URL__) ||
  process.env.NEXT_PUBLIC_SKILLPACK_API_URL ||
  "http://localhost:50052";

/** CLIENT-SPEC.md §2.3: maximum protocol version this client supports. */
const SUPPORTED_PROTOCOL_VERSION = 1;

export interface HealthResult {
  healthy: boolean;
  protocolVersion?: number;
}

/** CLIENT-SPEC.md §3.1: Health probe with 2s timeout + capability negotiation. */
async function healthProbe(): Promise<HealthResult> {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 2000);
    const resp = await fetch(`${API_URL}/health`, {
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
      protocolVersion > SUPPORTED_PROTOCOL_VERSION
    ) {
      return { healthy: false, protocolVersion };
    }
    return { healthy: true, protocolVersion };
  } catch {
    return { healthy: false };
  }
}

async function fetchWithAuth(
  path: string,
  options: RequestInit = {},
): Promise<Response> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...((options.headers as Record<string, string>) || {}),
  };
  const token =
    typeof window !== "undefined"
      ? (window as any).__SKILLPACK_TOKEN__
      : undefined;
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  return fetch(`${API_URL}${path}`, { ...options, headers });
}

/** Render a structured ClientError for Web per CLIENT-SPEC.md §5.2. */
export function renderClientErrorWeb(err: ClientError): void {
  // Simple console-based rendering; production would use an inline banner component
  const styles: Record<string, string> = {
    fatal: "background:#ef4444;color:#fff;padding:8px 12px;border-radius:4px;",
    error:
      "background:#fee2e2;color:#991b1b;padding:8px 12px;border-radius:4px;",
    warning:
      "background:#fef3c7;color:#92400e;padding:8px 12px;border-radius:4px;",
    info: "background:#f3f4f6;color:#374151;padding:8px 12px;border-radius:4px;",
  };
  console.error(
    `[SkillPack ${err.severity.toUpperCase()}] ${err.code}: ${err.message}`,
  );
}

export type { AssessmentResult, DimensionScore, Issue, SkillSummary, Grade };

/** Unified SkillPack client conforming to CLIENT-SPEC.md transport contract. */
export const skillpackClient = {
  async assess(path: string): Promise<AssessmentResult> {
    const { healthy, protocolVersion } = await healthProbe();
    if (!healthy) {
      if (protocolVersion && protocolVersion > SUPPORTED_PROTOCOL_VERSION) {
        throw new Error(
          `SkillPack server protocol version ${protocolVersion} is not supported. ` +
            `Maximum supported: ${SUPPORTED_PROTOCOL_VERSION}. Please upgrade your client.`,
        );
      }
      throw new Error(
        "SkillPack server is not reachable. Is skillpack-server running?",
      );
    }
    const resp = await fetchWithAuth(
      `/api/v1/skills/assess?path=${encodeURIComponent(path)}`,
    );
    if (!resp.ok) {
      const body = await resp.json().catch(() => ({}));
      if (body.code && body.severity) {
        renderClientErrorWeb(body as ClientError);
      }
      throw new Error(`Assessment failed: ${resp.status}`);
    }
    // The server wraps the result as { assessment: {...} } (mirrors listSkills'
    // { skills: [...] }). Unwrap it so callers get the flat AssessmentResult.
    const body = (await resp.json()) as { assessment?: AssessmentResult };
    if (!body.assessment) {
      throw new Error("Assessment response missing 'assessment' payload");
    }
    return body.assessment;
  },

  async listSkills(): Promise<SkillSummary[]> {
    const resp = await fetchWithAuth("/api/v1/skills");
    if (!resp.ok) throw new Error(`List failed: ${resp.status}`);
    const data = (await resp.json()) as { skills: SkillSummary[] };
    return data.skills || [];
  },

  async migrateSkills(canonicalRoot?: string): Promise<{
    success: boolean;
    message: string;
    totalScanned: number;
    migratedCount: number;
    alreadyCompliantCount: number;
    migratedNames: string[];
  }> {
    const resp = await fetchWithAuth("/migrate/skills", {
      method: "POST",
      body: JSON.stringify({ canonical_root: canonicalRoot }),
    });
    if (!resp.ok) throw new Error(`Migrate skills failed: ${resp.status}`);
    return await resp.json();
  },

  async migrateAgents(): Promise<{
    success: boolean;
    message: string;
    totalScanned: number;
    migratedCount: number;
    alreadyCompliantCount: number;
    migratedNames: string[];
  }> {
    const resp = await fetchWithAuth("/migrate/agents", {
      method: "POST",
    });
    if (!resp.ok) throw new Error(`Migrate agents failed: ${resp.status}`);
    return await resp.json();
  },

  async migrateClaudeAgents(agentsDir?: string): Promise<{
    success: boolean;
    message: string;
    totalScanned: number;
    migratedCount: number;
    alreadyCompliantCount: number;
    migratedNames: string[];
  }> {
    const resp = await fetchWithAuth("/migrate/claude-agents", {
      method: "POST",
      body: JSON.stringify({ agents_dir: agentsDir }),
    });
    if (!resp.ok)
      throw new Error(`Migrate Claude agents failed: ${resp.status}`);
    return await resp.json();
  },

  async migrateHarnesses(): Promise<{
    success: boolean;
    message: string;
    totalScanned: number;
    migratedCount: number;
    alreadyCompliantCount: number;
    migratedNames: string[];
  }> {
    const resp = await fetchWithAuth("/migrate/harnesses", {
      method: "POST",
    });
    if (!resp.ok) throw new Error(`Migrate harnesses failed: ${resp.status}`);
    return await resp.json();
  },

  async searchSkills(params: {
    text?: string;
    min_score?: number;
    sort_by?: string;
    limit?: number;
  }): Promise<{
    skills: SkillSummary[];
    total_count: number;
    page_size: number;
    offset: number;
  }> {
    const query = new URLSearchParams();
    if (params.text) query.set("text", params.text);
    if (params.min_score !== undefined)
      query.set("min_score", String(params.min_score));
    if (params.sort_by) query.set("sort_by", params.sort_by);
    if (params.limit !== undefined) query.set("limit", String(params.limit));
    const resp = await fetchWithAuth(
      `/api/v1/skills/search?${query.toString()}`,
    );
    if (!resp.ok) throw new Error(`Search failed: ${resp.status}`);
    return await resp.json();
  },

  async getSkill(ref: string): Promise<SkillSummary> {
    const resp = await fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(ref)}`,
    );
    if (!resp.ok) throw new Error(`Get skill failed: ${resp.status}`);
    return await resp.json();
  },

  async getSkillProfile(ref: string): Promise<SkillSummary> {
    const resp = await fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(ref)}/profile`,
    );
    if (!resp.ok) throw new Error(`Get profile failed: ${resp.status}`);
    return await resp.json();
  },

  async getSkillCompatibility(
    ref: string,
  ): Promise<{
    skill_ref: string;
    compatibility_score: number;
    capabilities: unknown[];
  }> {
    const resp = await fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(ref)}/compatibility`,
    );
    if (!resp.ok) throw new Error(`Get compatibility failed: ${resp.status}`);
    return await resp.json();
  },

  async getSkillCost(
    ref: string,
  ): Promise<{
    skill_ref: string;
    monthly_cost_usd: number;
    cost_tier: string;
  }> {
    const resp = await fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(ref)}/cost`,
    );
    if (!resp.ok) throw new Error(`Get cost failed: ${resp.status}`);
    return await resp.json();
  },

  async getSkillRating(
    ref: string,
  ): Promise<{
    skill_ref: string;
    rating: string;
    grade: string;
    score: number;
  }> {
    const resp = await fetchWithAuth(
      `/api/v1/skills/${encodeURIComponent(ref)}/rating`,
    );
    if (!resp.ok) throw new Error(`Get rating failed: ${resp.status}`);
    return await resp.json();
  },

  async listIndices(): Promise<
    { id: string; name: string; description: string }[]
  > {
    const resp = await fetchWithAuth("/api/v1/indices");
    if (!resp.ok) throw new Error(`List indices failed: ${resp.status}`);
    const data = (await resp.json()) as { indices: unknown[] };
    return (data.indices || []) as {
      id: string;
      name: string;
      description: string;
    }[];
  },
};
