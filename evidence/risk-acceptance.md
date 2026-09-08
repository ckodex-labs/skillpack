# Risk Acceptance — SkillPack Alpha

> Date: 2026-06-04  
> Release: 1.0.0-alpha.1  
> Reviewer: Alpha readiness review team  
> Status: Accepted for trusted, local/private alpha use only

## Risk Accepted

**Default-open authentication on the HTTP/gRPC server when `SKILLPACK_API_TOKEN` is unset.**

### Rationale

- The primary use case for the alpha is local development and trusted private evaluation.
- Requiring a token for every local run would add friction to the onboarding experience.
- The server now defaults to binding on `127.0.0.1` only; exposing it to a network requires an explicit `SKILLPACK_BIND_ALL=1` flag.
- A loud `WARN` (and `ERROR` if `BIND_ALL` is set without a token) is emitted at startup.
- Mutation/migration endpoints are clearly documented as requiring a token for any non-local deployment.

### Conditions of Acceptance

1. The server MUST NOT be exposed on a non-loopback interface without `SKILLPACK_API_TOKEN` set.
2. Alpha users MUST be informed that auth is disabled by default and that they are responsible for setting a token before any network exposure.
3. This risk acceptance MUST be re-evaluated before any beta or production release.

### Mitigations in Place

- Server binds to `127.0.0.1` by default (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/bin/server.rs:39-44`).
- `SKILLPACK_BIND_ALL=1` required for `0.0.0.0` binding.
- Startup warning logs when auth is disabled (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/bin/server.rs:71-75`).
- CORS denies all cross-origin requests by default unless `SKILLPACK_CORS_ORIGINS` is explicitly set (`@/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/crates/skillpack-api/src/http_server.rs:130-153`).

### Sign-off

This risk is accepted for the `1.0.0-alpha.1` release under the conditions above.
