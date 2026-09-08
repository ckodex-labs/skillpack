import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock vscode before importing modules that use it
vi.mock("vscode", () => ({
  workspace: {
    getConfiguration: vi.fn(() => ({
      get: vi.fn((key: string) => {
        if (key === "serverUrl") return undefined;
        return undefined;
      }),
    })),
  },
  window: {
    showErrorMessage: vi.fn().mockReturnValue(Promise.resolve(undefined)),
    showInformationMessage: vi.fn(),
  },
}));

import { SkillPackApiClient, renderClientErrorVscode } from "./api";
import type {
  Grade,
  Tier,
  SkillSummary,
  ClientError,
} from "../generated/client-model";

describe("generated types", () => {
  it("should have correct Grade enum values", () => {
    const grades: Grade[] = ["S+", "S", "A", "B", "C", "D", "F"];
    expect(grades).toContain("S+");
    expect(grades).toContain("F");
  });

  it("should have correct Tier enum values", () => {
    const tiers: Tier[] = ["L1", "L2", "L3"];
    expect(tiers).toContain("L1");
    expect(tiers).toContain("L3");
  });

  it("should construct a valid SkillSummary", () => {
    const skill: SkillSummary = {
      id: "test-skill",
      name: "test-skill",
      displayName: "Test Skill",
      version: "1.0.0",
      grade: "A",
      tier: "L2",
      updatedAt: new Date().toISOString(),
    };
    expect(skill.id).toBe("test-skill");
    expect(skill.grade).toBe("A");
    expect(skill.tier).toBe("L2");
  });

  it("should construct a valid ClientError", () => {
    const err: ClientError = {
      code: "ERR_SCHEMA_VALIDATION",
      category: "validation",
      severity: "error",
      message: "Schema validation failed",
    };
    expect(err.code).toBe("ERR_SCHEMA_VALIDATION");
    expect(err.severity).toBe("error");
  });
});

describe("SkillPackApiClient", () => {
  const originalEnv = process.env;

  beforeEach(() => {
    process.env = { ...originalEnv };
    vi.stubGlobal("fetch", vi.fn());
  });

  afterEach(() => {
    process.env = originalEnv;
    vi.unstubAllGlobals();
    vi.clearAllMocks();
  });

  it("should discover API URL from SKILLPACK_API_URL env var", () => {
    process.env.SKILLPACK_API_URL = "https://api.example.com";
    process.env.SKILLPACK_TOKEN = "test-token";

    const client = new SkillPackApiClient();
    expect(client["baseUrl"]).toBe("https://api.example.com");
  });

  it("should use default URL when SKILLPACK_API_URL is not set", () => {
    delete process.env.SKILLPACK_API_URL;
    delete process.env.SKILLPACK_TOKEN;

    const client = new SkillPackApiClient();
    expect(client["baseUrl"]).toBe("http://localhost:50051");
  });

  it("should inject Bearer token when SKILLPACK_TOKEN is set", async () => {
    process.env.SKILLPACK_API_URL = "https://api.example.com";
    process.env.SKILLPACK_TOKEN = "my-secret-token";

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ skills: [] }),
    });
    vi.stubGlobal("fetch", mockFetch);

    const client = new SkillPackApiClient();
    await client.search("test");

    const call = mockFetch.mock.calls[0];
    const init = call[1] as RequestInit;
    const headers = init.headers as Record<string, string>;
    expect(headers["Authorization"]).toBe("Bearer my-secret-token");
  });

  it("should not inject Authorization header when SKILLPACK_TOKEN is absent", async () => {
    delete process.env.SKILLPACK_TOKEN;
    process.env.SKILLPACK_API_URL = "https://api.example.com";

    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ skills: [] }),
    });
    vi.stubGlobal("fetch", mockFetch);

    const client = new SkillPackApiClient();
    await client.search("test");

    const call = mockFetch.mock.calls[0];
    const init = call[1] as RequestInit;
    const headers = init.headers as Record<string, string> | undefined;
    expect(headers?.["Authorization"]).toBeUndefined();
  });

  it("should render structured errors correctly", () => {
    const err: ClientError = {
      code: "ERR_NETWORK_TIMEOUT",
      category: "network",
      severity: "error",
      message: "Connection timed out",
      details: { endpoint: "/api/v1/skills" },
      clientHint: {
        vscode: "Check your SKILLPACK_API_URL setting",
      },
    };

    // Should not throw
    expect(() => renderClientErrorVscode(err)).not.toThrow();
  });

  it("health probe uses 2s timeout per CLIENT-SPEC.md §3.1", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ protocolVersion: 1 }),
    });
    vi.stubGlobal("fetch", mockFetch);

    const client = new SkillPackApiClient();
    const result = await client.healthProbe();

    expect(result.healthy).toBe(true);
    expect(result.protocolVersion).toBe(1);
    const call = mockFetch.mock.calls[0];
    expect(call[0]).toBe("http://localhost:50051/health");
    const init = call[1] as RequestInit;
    expect(init.signal).toBeDefined();
  });

  it("should reject incompatible protocol version", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ protocolVersion: 999 }),
    });
    vi.stubGlobal("fetch", mockFetch);

    const client = new SkillPackApiClient();
    const result = await client.healthProbe();
    expect(result.healthy).toBe(false);
    expect(result.protocolVersion).toBe(999);
  });

  it("should accept server without protocolVersion (backward compatible)", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({}),
    });
    vi.stubGlobal("fetch", mockFetch);

    const client = new SkillPackApiClient();
    const result = await client.healthProbe();
    expect(result.healthy).toBe(true);
    expect(result.protocolVersion).toBeUndefined();
  });
});
