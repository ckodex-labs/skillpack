import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import type { AssessmentResult, ClientError } from "../generated/client-model";

/** Dynamically import client so window globals are set before module evaluation. */
async function loadClient() {
  return import("./client");
}

describe("skillpackClient transport", () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.resetModules();
    globalThis.fetch = vi.fn();
    (globalThis as any).window = globalThis;
    (globalThis as any).__SKILLPACK_API_URL__ = "https://dash.test";
    (globalThis as any).__SKILLPACK_TOKEN__ = "dash-token";
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
    delete (globalThis as any).window;
    delete (globalThis as any).__SKILLPACK_API_URL__;
    delete (globalThis as any).__SKILLPACK_TOKEN__;
    vi.clearAllMocks();
  });

  it("should discover API URL from window global", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ skills: [] }),
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    await skillpackClient.listSkills();

    const url = mockFetch.mock.calls[0][0] as string;
    expect(url).toContain("https://dash.test");
  });

  it("should inject Bearer token from window global", async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ skills: [] }),
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    await skillpackClient.listSkills();

    const init = mockFetch.mock.calls[0][1] as RequestInit;
    const headers = init.headers as Record<string, string>;
    expect(headers["Authorization"]).toBe("Bearer dash-token");
  });

  it("should not inject Authorization when token is absent", async () => {
    delete (globalThis as any).__SKILLPACK_TOKEN__;
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ skills: [] }),
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    await skillpackClient.listSkills();

    const init = mockFetch.mock.calls[0][1] as RequestInit;
    const headers = init.headers as Record<string, string> | undefined;
    expect(headers?.["Authorization"]).toBeUndefined();
  });

  it("should health probe before assess", async () => {
    const mockFetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes("/health")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ protocolVersion: 1 }),
        });
      }
      return Promise.resolve({
        ok: true,
        json: () =>
          Promise.resolve({
            assessment: {
              id: "test-001",
              skillId: "test-skill",
              skillPath: "/test",
              grade: "A",
              totalScore: 85,
              dimensions: [],
              issues: [],
              assessedAt: new Date().toISOString(),
            } as AssessmentResult,
          }),
      });
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    const result = await skillpackClient.assess("/test");

    expect(result.grade).toBe("A");
    const healthCall = mockFetch.mock.calls.find((c) =>
      (c[0] as string).includes("/health"),
    );
    expect(healthCall).toBeDefined();
  });

  it("should reject server with incompatible protocol version", async () => {
    const mockFetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes("/health")) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ protocolVersion: 999 }),
        });
      }
      return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    await expect(skillpackClient.assess("/test")).rejects.toThrow(
      "protocol version 999 is not supported",
    );
  });

  it("should accept server without protocol version (backward compatible)", async () => {
    const mockFetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes("/health")) {
        return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
      }
      return Promise.resolve({
        ok: true,
        json: () =>
          Promise.resolve({
            assessment: {
              id: "test-002",
              skillId: "test-skill",
              skillPath: "/test",
              grade: "B",
              totalScore: 70,
              dimensions: [],
              issues: [],
              assessedAt: new Date().toISOString(),
            } as AssessmentResult,
          }),
      });
    });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    const result = await skillpackClient.assess("/test");
    expect(result.grade).toBe("B");
  });

  it("should throw when server is unreachable", async () => {
    const mockFetch = vi.fn().mockResolvedValue({ ok: false });
    globalThis.fetch = mockFetch;

    const { skillpackClient } = await loadClient();
    await expect(skillpackClient.assess("/test")).rejects.toThrow(
      "SkillPack server is not reachable",
    );
  });
});

describe("renderClientErrorWeb", () => {
  it("should render structured errors without throwing", async () => {
    const { renderClientErrorWeb } = await loadClient();
    const err: ClientError = {
      code: "ERR_SCHEMA_VALIDATION",
      category: "validation",
      severity: "error",
      message: "Schema validation failed",
    };

    expect(() => renderClientErrorWeb(err)).not.toThrow();
  });
});
