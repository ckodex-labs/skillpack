# EXAMPLE — anatomy of a reference doc

> This file is the shape of a reference doc, not a real one. When you adapt
> the template, replace it with one reference per protocol surface
> (API endpoint catalog, package-type matrix, CLI flag tables, RBAC model,
> etc.). References are loaded by the agent **on demand**, not on every
> invocation — that is the point of progressive disclosure.

---

## When the agent should read this file

- The user asks a question that lands in the "Routing table" row that names
  this reference.
- The orchestrator agent is about to chain two scripts and needs the protocol
  detail (endpoint catalog, RBAC matrix, fail-mode taxonomy) to decide the
  next step.
- A `jq` query against an envelope returns an error code that maps to a
  protocol-level concept documented here.

The agent should **not** read this file:

- On every conversation turn — the SKILL.md body should be enough for the
  golden paths.
- To answer "what does this template do?" — that lives in SKILL.md.
- To find an example invocation — that lives in `scripts/<name>.sh --help`.

---

## Section 1 — Protocol summary

One paragraph describing the protocol surface this reference covers. For
example, in a JFrog reference: "Artifactory exposes a JSON REST API rooted at
`/artifactory/api/`. Authentication is by bearer token or OIDC token
exchange; static credentials are forbidden in CI."

---

## Section 2 — Endpoint / surface catalog

Tabular reference for every surface the agent might dispatch against. Keep it
flat — agents read tables far better than nested prose.

| ID    | Surface                       | Method | Auth        | Idempotent | Notes                       |
| ----- | ----------------------------- | ------ | ----------- | ---------- | --------------------------- |
| EX-01 | `/api/example/list`           | GET    | Bearer/OIDC | yes        | Pagination via `?cursor=`   |
| EX-02 | `/api/example/{id}`           | GET    | Bearer/OIDC | yes        | 404 → `NOT_FOUND` envelope  |
| EX-03 | `/api/example/{id}/promote`   | POST   | Bearer/OIDC | no         | Write path; defaults dry-run |

For the template's `hello.sh` there is no protocol — this section is shape
demonstration, not literal content.

---

## Section 3 — Fail-mode catalog

Every script that talks to this protocol must map upstream errors onto the
envelope's `error.code` namespace. Catalog the mapping here so the agent does
not have to infer it.

| Upstream signal             | Envelope `error.code`         | Retriable | Action                                  |
| --------------------------- | ----------------------------- | --------- | --------------------------------------- |
| HTTP 401 / 403              | `AUTH_FAILED`                 | false     | die() — stderr, exit 1. Bad credential. |
| HTTP 404                    | `NOT_FOUND`                   | false     | die() — stderr, exit 1. Caller error.   |
| HTTP 408 / 502 / 503 / 504  | `UPSTREAM_TRANSIENT`          | true      | emit_error() — stdout, exit 0. Retry.   |
| HTTP 429                    | `RATE_LIMITED`                | true      | emit_error() — stdout, exit 0. Backoff. |
| TLS validation failure      | `TLS_VALIDATION_FAILED`       | false     | die() — never silently downgrade.       |
| Connection refused          | `CONNECT_REFUSED`             | true      | emit_error() — stdout, exit 0.          |
| Body fails schema           | `INVALID_RESPONSE_SHAPE`      | false     | die() — protocol drift, escalate.       |

The codes for `hello.sh` are `SIMULATED_TRANSPORT_ERROR` and `SIMULATED_FATAL` —
deliberately synthetic so the test gate can assert against them.

---

## Section 4 — Example chain

The agent should treat each step's verification as a `jq -e` against the
envelope, never a string match against prose.

1. **Discover** — `scripts/hello.sh --name demo` →
   `jq -e '.ok == true and .data.name == "demo"'`.
2. **Simulate retriable failure** — `scripts/hello.sh --name demo --fail-mode retriable` →
   `jq -e '.ok == false and .error.retriable == true'`. The agent retries
   (template caps at one retry; production adapters use bounded exponential
   backoff with jitter).
3. **Simulate fatal failure** — `scripts/hello.sh --name demo --fail-mode fatal` →
   `jq -e '.ok == false and .error.retriable == false'` on **stderr**. The
   agent surfaces the envelope to the user and stops — fatal config errors
   are never retried.

---

## Section 5 — Safety notes specific to this protocol

- **HTTPS only.** Every URL flag must call `require_https`. The template has
  no URL flag; production adapters must wire this in.
- **Token never on CLI.** Pass tokens via env (`<ADAPTER>_TOKEN`), masked in
  all diagnostics with `mask_token`.
- **OIDC for CI.** Static credentials forbidden — use OIDC token exchange.
  Token TTL ≤ 3600s for CI roles.
- **Dry-run default on write paths.** Set `DRY_RUN=1` as the default; require
  `--no-dry-run` to flip. The template's `hello.sh` is read-only and does
  not need this.
- **Signature verification before any import / apply / promote.** Never
  `--skip-verify` in prod, even on internal hops.

---

## Section 6 — Anti-patterns (do not do this in a reference)

| Anti-pattern                                            | Why it breaks agents                            |
| ------------------------------------------------------- | ----------------------------------------------- |
| Prose-only endpoint descriptions                        | Agents read tables far better than paragraphs.  |
| Mixed retriable / fatal in one row without a column     | Agents cannot infer routing without the column. |
| Examples that string-match envelope text                | Drifts on every wording change — assert shape. |
| Embedding raw tokens in examples                        | Leaks into agent context and downstream logs.   |
| Hedging vocab ("should work", "probably retriable")     | VWP P-SC-001 — no hedging in technical claims. |
| Marketing vocab ("robust error handling", "seamless")   | VWP P-SC-002 — banned in reference docs.       |
| Unscoped "always" / "never" without a baseline          | VWP P-SC-007 — bind every absolute to evidence. |

---

## Adapting this file

When you clone the template:

1. Rename `EXAMPLE.md` to match the protocol surface (e.g. `JFROG_API.md`,
   `GITLAB_API.md`, `K8S_RBAC.md`).
2. Replace Sections 1–5 with real protocol content.
3. Add one row per real endpoint / surface; keep Section 3's fail-mode
   catalog exhaustive — every upstream error class the script can encounter
   must map to a single envelope code.
4. Cross-reference from the SKILL.md "References" section and the agent's
   "Routing table" so the on-demand load is discoverable.
5. Cite this reference in `agent.test.sh` Section 3 (routing-target check) so
   removing the reference without updating the routing table fails CI.
