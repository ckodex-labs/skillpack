# RFC-001: Progressive Skill Lifecycle Runtime Profile for Agent Harnesses

| Field         | Value                                                                |
|---------------|----------------------------------------------------------------------|
| Status        | Draft                                                                |
| Version       | 0.2.0                                                                |
| Authors       | Ckodex Labs                                                          |
| Scope         | Upstream-compatible runtime profile; no upstream change required     |
| Target spec   | `agent-skills` v1-compatible / CKODEX skill manifest v1.1            |
| Framework     | CKODEX v16.0                                                         |
| Date          | 2026-05-23                                                           |
| Supersedes    | RFC-001 v0.1.0 (internal draft)                                      |
| Related       | `skill.v1.schema.json`, `skillsbundle.v1.schema.json`                |
| Review GAL    | 1 (assisted, post-hoc external review integrated)                    |

---

## 0. Abstract

Agent Skills v1 defines a three-stage authoring/disclosure model:
metadata, full instructions, and resources. Some implementations
treat script execution as a distinct fourth stage. However, the public
specification does not define a runtime lifecycle FSM with demotion,
eviction, hysteresis, skill-pool budgeting, ontology-aware routing,
or attested lifecycle decisions.

This RFC proposes a **runtime profile** that any v1-compatible
harness can adopt without changes to the upstream `SKILL.md` format.
The profile defines a six-tier progressive-disclosure FSM (with
distinct resource-read and script-execute tiers), a multi-signal
scoring function with hysteresis and normalization, an explicit
eviction policy including topic-drift, and an optional JSON-LD
ontology block carried in the CKODEX companion manifest
(`skill.json`). It adds a four-mode privacy model for PCA-attested
lifecycle decisions and a capability-negotiation handshake for
cross-harness interop. The profile is fully backward-compatible:
v1 skills work unchanged, degrading gracefully to lexical + semantic
scoring only.

---

## 1. Motivation

### 1.1 Baseline (corrected)

Agent Skills v1 specifies progressive disclosure across three stages:
metadata loaded at startup, full `SKILL.md` body loaded on
activation, and optional resources (`scripts/`, `references/`,
`assets/`) read on demand. Some harnesses further separate script
execution into a distinct fourth stage with elevated permission
requirements. Startup bloat is generally bounded by harness-level
caps. **What the public spec does not define** is a runtime
lifecycle: no demotion path, no eviction policy, no hysteresis, no
skill-pool token budget, no ontology-aware routing, and no signed
lifecycle decisions.

### 1.2 Observed Failure

The post-selection failure mode is real and serious: harnesses
promote skills to full-body residency on weak signal and never
demote. Over a long session with 10+ installed skills, accumulated
skill bodies displace the user's actual working context. Worst-case
skill-induced context occupancy is on the order of N × 3000 tokens,
spent on skills the user invoked once or not at all.

### 1.3 Goals

| Goal                                          | Mechanism                                       |
|-----------------------------------------------|-------------------------------------------------|
| Bound the token cost of the skill pool         | Six-tier FSM + budget enforcement               |
| Load skills only when justified                | Multi-signal scoring + thresholds               |
| Unload skills when no longer justified         | Demotion + drift-based eviction                 |
| Separate resource read from script execution   | Distinct L4R and L4X tiers with different gates |
| Avoid tier-flap near decision boundaries       | Hysteresis (`τ_load > τ_keep`)                  |
| Preserve audit trail with privacy guarantees   | PCA modes (debug → audit → public → regulated)  |
| Backward-compatible with `agent-skills` v1     | All new fields optional; SKILL.md untouched     |

### 1.4 Non-Goals

- Does **not** modify the upstream `SKILL.md` body or frontmatter format.
- Does **not** mandate a specific embedding model or vector store.
- Does **not** prescribe harness UI for skill state visibility.
- Does **not** require any change to upstream `agent-skills` v1.

---

## 2. Locked Defaults

| Decision                          | Choice                                              | Rationale                                         |
|-----------------------------------|-----------------------------------------------------|---------------------------------------------------|
| L2 synopsis source                | Build-time precomputed                              | Deterministic, signable, auditable                |
| Ontology block placement          | CKODEX companion manifest (`skill.json`) only       | Upstream `SKILL.md` stays portable                |
| Synopsis requirement strength     | SHOULD (CKODEX-shipped: MUST; third-party: MAY)     | Ecosystem-friendly; CKODEX release discipline preserved |
| PCA privacy default               | `audit-private`                                     | Per-turn routing is not public-anchor by default  |
| Score range                       | All signals normalized to `[0,1]`; composite ≤ 1.0  | Comparable, debuggable, anti-overflow             |

---

## 3. Six-Tier Lifecycle FSM

```
  L0   ──►  L1   ──►  L2    ──►  L3    ──►  L4R   (resource read)
  reg.     meta.    syn.       instr.    └──►  L4X   (script execute, separate gate)

  ◄────────────────────────────────────────  (demotion paths to L0..L3)
```

### 3.1 State Definitions

| Tier | Content                              | Typical tokens | Risk class | Required gate                                |
|------|--------------------------------------|----------------|------------|----------------------------------------------|
| L0   | Name + canonical ID                  | ~10            | none       | registry budget                              |
| L1   | Name + description + scope hints     | ~80            | low        | scope + framework compatibility              |
| L2   | Synopsis + ontology summary          | ~400           | low        | `score ≥ τ_syn_load`                         |
| L3   | Full `SKILL.md` body                 | ~3000–5000     | medium     | `score ≥ τ_full_load ∧ intent_to_invoke`     |
| L4R  | Selected references/assets read      | per-file       | medium     | resource allowlist + path policy             |
| L4X  | Bundled script/tool executed         | per-tool       | **high**   | capability policy + sandbox + provenance + PCA |

The L4R/L4X split is essential: reading `references/OSCAL.md` and
executing `scripts/validate.py` are not the same risk class.
Aligning them under a single L4 conflates a disclosure decision with
a permission decision.

### 3.2 Transitions

**Promotion chain:**

| From | To  | Trigger                                                            |
|------|-----|--------------------------------------------------------------------|
| L0   | L1  | Registry scan + scope filter                                       |
| L1   | L2  | `score ≥ τ_syn_load`                                               |
| L2   | L3  | `score ≥ τ_full_load ∧ intent_to_invoke`                           |
| L3   | L4R | Referenced resource needed                                         |
| L3   | L4X | Executable needed AND `capability_policy.admits(skill, tool)`      |

**Demotion chain:**

| From | To  | Trigger                                                            |
|------|-----|--------------------------------------------------------------------|
| L4X  | L3  | Execution complete OR policy revoked                               |
| L4R  | L3  | Resource no longer needed                                          |
| L3   | L2  | `idle ≥ N_full ∨ score < τ_full_keep ∨ task_complete ∨ drift ≥ θ_drift` |
| L2   | L1  | `idle ≥ N_syn ∨ drift ≥ θ_drift ∨ score < τ_syn_keep`              |
| L1   | L0  | `registry_pressure ∧ rank_below_floor`                             |
| any  | L1  | `emergency_evict(context_overflow ≥ 0.85)`                         |

**Emergency:**

| Event                  | Action                                                      |
|------------------------|-------------------------------------------------------------|
| `EP-001 STOP_ALL`      | All → L1; freeze registry; revoke all L4X                   |
| `EP-002 QUARANTINE`    | All quarantined-agent skills → L0; preserve forensic snapshot |
| `EP-013 FW_INTEGRITY`  | Version-incompatible skills auto-evict to L0                |
| `EP-014 CONTAINMENT`   | Snapshot full skill-pool state to signed evidence, then evict |

### 3.3 The L2 Synopsis Tier

L2 is the load-bearing addition. A precomputed bounded synopsis —
"what this does / when it triggers / what it produces / what it
explicitly does *not* cover" — stays warm cheaply and gates the
expensive L3 promotion. Most skills currently flapping between L1
and L3 are L2-relevant but never L3-needed.

The synopsis MUST be a single string field at `metadata.synopsis`
(see Appendix A) bounded to 2048 characters, generated at bundle
build time, and SHOULD be a faithful compression of the `SKILL.md`
body. Validator §10.4 checks synopsis–body consistency via semantic
diff.

---

## 4. Multi-Signal Scoring

### 4.1 Score Function (normalized)

```
SCORE(s, ctx) : Real ∈ [0,1]

  S_lex  := BM25(s.triggers ⊕ s.keywords, recent_K_turns)            ∈ [0,1]
  S_sem  := cos(embed(ctx_window), embed(s.description ⊕ s.synopsis)) ∈ [0,1]
  S_ont  := clamp01( (jaccard(ctx.subjects, s.ontology.subjects)
                     + 0.5 · jaccard(ctx.subjects, s.ontology.related)) / 1.5 )
  S_co   := P(s | currently_loaded_skills)                            ∈ [0,1]
  S_rec  := exp(-λ · turns_since_last_reference)                      ∈ [0,1]
  S_usr  ∈ {0.0, 0.6, 0.8, 1.0}                                       // graded, see §4.3
  S_pol  ∈ {0, 1}                                                     // hard gate

  score  := Σᵢ wᵢ · Sᵢ        if S_pol = 1
         := 0                  if S_pol = 0
```

### 4.2 Default Weights

| Signal  | Weight | Rationale                                                          |
|---------|--------|--------------------------------------------------------------------|
| S_sem   | 0.30   | Strongest single signal; embedding captures intent                 |
| S_ont   | 0.25   | Structural, not lexical; CKODEX differentiator                     |
| S_lex   | 0.15   | Cheap, catches explicit keyword matches                            |
| S_co    | 0.10   | "Skill A often precedes skill B" patterns                          |
| S_rec   | 0.10   | Recent reference predicts re-reference                             |
| S_usr   | 0.10   | Honors explicit user intent without bypassing policy               |

Weights are tunable per harness profile and MUST be recorded in any
lifecycle PCA bundle.

### 4.3 User Signal (graded)

```
S_usr := 1.0   explicit_mention (e.g. "use ckodex-oscal")
       | 0.8   pinned by repo/admin config
       | 0.6   memory preference (past sessions)
       | 0.0   no user signal
```

Explicit mention bypasses lexical uncertainty but **does not bypass
policy** (`S_pol`). A user requesting a skill their `GAL` does not
admit still fails the pre-filter; the harness MUST surface the
policy denial rather than silently promoting.

### 4.4 Policy Pre-Filter (`S_pol`)

A skill fails the pre-filter and is excluded from candidacy if any of:

- `skill.ontology.galMinimum > current_gal`
- `skill.governance.proofRequired ∧ ¬proof_pipeline_available`
- Skill scope ⊄ current `(tenant, env, workspace, project)`
- `skill.metadata.version` incompatible with harness framework version
- `skill.runtime.implicitInvocation == false ∧ ¬explicit_mention`

### 4.5 Score-Function Properties

- **Bounded.** `score ∈ [0,1]` for all inputs.
- **Monotonic in signal.** Increasing any non-policy signal cannot decrease the score.
- **Policy-dominated.** Failing pre-filter zeros the score regardless of all other signals.
- **Explainable.** The component vector and weight vector are reproducible from the PCA bundle.

---

## 5. Hysteresis & Thresholds

```
τ_syn_load   = 0.45   // L1 → L2
τ_syn_keep   = 0.30   // hold L2
τ_full_load  = 0.65   // L2 → L3
τ_full_keep  = 0.40   // hold L3
τ_floor      = 0.20   // demote all
θ_drift      = 0.35   // topic-drift demotion threshold

invariant:   τ_load > τ_keep > τ_floor    // anti-flap
band:        [τ_keep, τ_load]              // skill remains in current tier
```

Hysteresis is a correctness property, not an optimization. Without
`τ_load > τ_keep`, a skill scoring near the boundary thrashes every
turn, burning re-load tokens and producing audit noise.

---

## 6. Eviction Policy

A loaded skill is demoted or evicted when any of the following hold:

| Trigger          | Condition                                                              |
|------------------|------------------------------------------------------------------------|
| Score decay      | `score < τ_keep(current_tier) ∧ idle ≥ 3 turns`                        |
| Idle timeout     | `idle_turns > N_idle(tier)`, where `N_idle = {L3:5, L2:15, L1:∞}`      |
| Topic drift      | `cos(ctx_now, ctx_at_load) < θ_drift`                                  |
| Budget pressure  | `Σ tokens(loaded) > B_pool` → evict argmin score, lowest tier first    |
| Task complete    | Explicit end signal OR `skill.exit_criteria_met`                       |
| Mutex            | `load(B) ∧ B ∈ A.ontology.exclusiveOf` → atomic evict A                |
| Framework break  | `skill.version ⊭ harness.framework.compat`                             |
| L4X completion   | Script execution returned → L4X → L3                                   |
| L4X revocation   | Capability policy revoked mid-execution → SIGTERM + L4X → L3           |

Topic drift catches the "loaded 20 turns ago, conversation moved
on" failure that recency and score-decay alone miss. Track the
centroid embedding of the last `K` turns; when a skill's home-topic
embedding (its `(description ⊕ synopsis)` embedding at load time)
is far from the current centroid, demote regardless of recency.

---

## 7. CKODEX Companion Manifest (`skill.json` v1.1)

The upstream `SKILL.md` format is unchanged. CKODEX-aware harnesses
additionally read a sibling companion manifest, `skill.json`, that
carries semantic and governance extensions. Harnesses without
CKODEX support ignore the companion and rely on `SKILL.md`
exclusively — the skill remains fully functional.

### 7.1 Layout

```
skill-root/
  SKILL.md                # upstream-compatible (unchanged)
  skill.json              # CKODEX companion (this RFC's extensions)
  references/             # per upstream spec
  scripts/                # per upstream spec
  assets/                 # per upstream spec
```

### 7.2 Example

```jsonc
{
  "apiVersion": "ckodex.org/skill/v1.1",
  "kind": "Skill",
  "metadata": {
    "name": "ckodex-oscal",
    "description": "Author, read, validate, convert, or reason about OSCAL...",
    "synopsis": "When the user mentions OSCAL, NIST 800-53/171, FedRAMP, SSP, POA&M, control catalogs, profiles, baselines, component definitions, or compliance-as-code: this skill resolves OSCAL profiles, validates catalog/profile/SSP documents against NIST schemas, converts between XML/JSON/YAML, and emits UCA attestations. NOT a substitute for cortaix-csr (factory baseline) or framework-specific ISO/PCI/HIPAA skills.",
    "version": "1.0.0"
  },
  "ontology": {
    "@context": "https://ckodex.org/skill-context/v1",
    "subjects":    ["oscal", "nist:800-53", "compliance", "fedramp"],
    "produces":    ["ckodex:UCA"],
    "consumes":    ["oscal:catalog", "oscal:profile"],
    "related":     ["cortaix-csr", "ckodex-announcements"],
    "exclusiveOf": [],
    "triggers":    ["OSCAL", "SSP", "POA&M", "FedRAMP", "control catalog"],
    "galMinimum":  2,
    "proofTypes":  ["UCA"]
  },
  "runtime": {
    "implicitInvocation": true,
    "minTier":            "L1",
    "maxTier":            "L4R",
    "l4xAllowed":         false
  },
  "governance": {
    "scope": {
      "tenant":      "*",
      "environment": ["dev", "staging", "prod"],
      "workspace":   "*"
    },
    "requiresTipForCrossTenant": true
  }
}
```

### 7.3 Harness Capabilities Unlocked

1. **Pre-warm via `produces`/`consumes` edges.** Loading a UCA-producer
   nudges UCA-consumers from L1 to L2.
2. **Mutex enforcement via `exclusiveOf`.** Loaded skills listed in
   the `exclusiveOf` array of a newly-loaded skill MUST be evicted
   atomically.
3. **Policy pre-filter via `galMinimum` and `proofTypes`.** Candidates
   failing pre-filter are dropped before scoring.
4. **Structural relatedness via `subjects` / `related`.** Jaccard
   over `subjects` plus weighted overlap with `related` supplies
   `S_ont` without an embedding round-trip.
5. **Implicit-invocation control via `runtime.implicitInvocation`.**
   Skills with `implicitInvocation: false` MUST NOT be auto-promoted;
   only explicit mention promotes them.
6. **Max-tier ceiling via `runtime.maxTier`.** A skill declaring
   `maxTier: "L4R"` cannot be promoted to L4X regardless of intent.

### 7.4 Backward Compatibility

Skills without an `ontology` block: `S_ont := 0`; policy pre-filter
passes by default; harness synthesizes L2 content from
`description` plus first bounded section of `SKILL.md`. The
remaining signals (`S_lex`, `S_sem`, `S_co`, `S_rec`, `S_usr`)
carry the full scoring load.

---

## 8. Reference Algorithm

```python
# Pseudocode. A real implementation is per-harness.

def on_user_turn(msg: str, harness_state: HarnessState) -> Response:
    ctx_emb = embed(harness_state.recent_K_turns + [msg])
    candidates = [
        s for s in harness_state.registry
        if policy_pre_filter(s, harness_state)   # S_pol = 1
    ]
    for s in candidates:
        s.score = compute_score(s, ctx_emb, harness_state)

    # Promotions (L1 → L2 → L3)
    for s in candidates:
        if s.tier == L1 and s.score >= TAU_SYN_LOAD:
            promote(s, L2)
        if s.tier == L2 and s.score >= TAU_FULL_LOAD and intent_to_invoke(s, msg):
            promote(s, L3)

    # L3 → L4R / L4X handled inline during response generation
    # (driven by explicit file references / tool calls).

    # Demotions
    for s in harness_state.loaded:
        if drift(s, ctx_emb) > THETA_DRIFT or idle_turns(s) > N_IDLE[s.tier]:
            demote(s, s.tier - 1)
        elif s.score < TAU_KEEP[s.tier]:
            demote(s, s.tier - 1)

    # Budget enforcement
    while total_tokens(harness_state.loaded) > B_POOL:
        victim = argmin_score(harness_state.loaded, prefer_tier=L3)
        evict(victim)

    # Lifecycle attestation (privacy mode honored)
    if harness_state.gal >= 3:
        emit_pca(LifecycleDecisionBundle(harness_state,
                                         mode=harness_state.pca_privacy_mode))

    return respond_using(harness_state.loaded_at_L2_or_higher)
```

Per-turn cost target: <50 ms p95 for N ≤ 50 skills with cached
embeddings (see §16 acceptance metrics).

---

## 9. PCA Evidence for Lifecycle Decisions (privacy-mode-aware)

Per-turn skill routing decisions contain user-intent signal.
Anchoring all of them to a public transparency log is wrong.
Sigstore/Rekor remains the right tool at the **artifact/release**
boundary — for runtime routing evidence, this RFC defines a
four-mode privacy model.

### 9.1 Privacy Modes

| Mode               | Content                                                                    | Anchored to               |
|--------------------|----------------------------------------------------------------------------|---------------------------|
| `debug-local`      | Raw scores, skill names, triggers, ctx-embedding hash                      | Local file only           |
| `audit-private`    | Skill IDs, scores rounded to 2 decimals, no prompt terms                   | Internal CKODEX log       |
| `public-anchor`    | Bundle hash, policy version, transition counts; no skill names unless public | Rekor                     |
| `regulated-export` | Redacted full evidence with access-control + retention labels              | Regulated evidence store  |

Default mode is `audit-private`. Operators MAY downgrade to
`debug-local` for development or upgrade to `regulated-export` for
compliance contexts. `public-anchor` is opt-in only.

### 9.2 Bundle Schema (audit-private example)

```jsonc
{
  "predicate":     "ckodex/skill-lifecycle@v1",
  "subject":       "urn:ckodex:harness:<harness-id>:turn:<turn-uuid>",
  "timestamp":     "2026-05-23T14:32:11.123456789Z",
  "privacy_mode":  "audit-private",
  "decisions": [
    {
      "skill_id":   "sha256:7f3a...c2",
      "from_tier":  "L1",
      "to_tier":    "L2",
      "trigger":    "score_threshold",
      "score":      0.47,
      "components": { "lex": 0.20, "sem": 0.60, "ont": 0.50, "co": 0.10, "rec": 0.00, "usr": 0.00 },
      "weights":    { "lex": 0.15, "sem": 0.30, "ont": 0.25, "co": 0.10, "rec": 0.10, "usr": 0.10 }
    }
  ],
  "context_pool_tokens_before": 4820,
  "context_pool_tokens_after":  2110,
  "budget_max":                 8192,
  "policy_version":             "ckodex-skill-lifecycle/v1.1"
}
```

In `audit-private` mode the `skill_id` is the digest, not the
human-readable name; the harness keeps a name↔digest map locally.
In `public-anchor` mode the digest is the only identifier published.

---

## 10. Validator Hooks (v1.1)

The CKODEX skill validator gains the following checks for v1.1
manifests. Target: **0 failures / 0 warnings**.

| §    | Check                                                                                |
|------|--------------------------------------------------------------------------------------|
| §10.1 | `metadata.synopsis` present and ≤ 2048 chars (warning only for third-party v1.1)    |
| §10.2 | If `ontology` present: `@context` resolves; predicates within allowed set           |
| §10.3 | `ontology.subjects`/`related`/`exclusiveOf` are unique-item arrays                  |
| §10.4 | `synopsis` semantically consistent with `SKILL.md` body (cosine ≥ 0.55)             |
| §10.5 | `ontology.galMinimum` ∈ `[0..5]`                                                    |
| §10.6 | `ontology.proofTypes` ⊆ enum from `skill.v1` `governance.proofTypes`                |
| §10.7 | `runtime.maxTier` ∈ `{L1, L2, L3, L4R, L4X}` and `≥ runtime.minTier`                |
| §10.8 | If `runtime.l4xAllowed: true` then `capabilities.tools[].risk` declared for each tool |

---

## 11. Migration v1 → v1.1

| Step | Action                                                                                          |
|------|-------------------------------------------------------------------------------------------------|
| 1    | Bump `apiVersion` to `ckodex.org/skill/v1.1` in `skill.json` only. `SKILL.md` unchanged.        |
| 2    | Run `gen_bundle.py --generate-synopsis` to populate `metadata.synopsis` (CKODEX-shipped: MUST). |
| 3    | Add optional `ontology` block (recommended).                                                    |
| 4    | Add optional `runtime` block (recommended for skills that must restrict implicit invocation).    |
| 5    | `validate.sh` re-runs to 0F/0W under v1.1 rules.                                                |
| 6    | Re-pack bundle; supersedes v1 artifact via standard semver/promotion path.                      |

**Compatibility matrix:**

| Skill manifest | Harness v1.0 | Harness v1.1 (CKODEX) |
|----------------|--------------|------------------------|
| v1             | full         | full + degraded routing (no S_ont) |
| v1.1           | reads metadata + body (ignores ontology) | full RFC-001 routing |

---

## 12. CKODEX-Specific Bindings

### 12.1 GAL Gates

| GAL  | Lifecycle proof requirement                                       |
|------|-------------------------------------------------------------------|
| 0–2  | PCA optional; transitions logged but not attested                 |
| 3    | PCA mandatory; default mode `audit-private`                       |
| 4    | PCA + UCA cross-reference on every L3 promotion                   |
| 5    | PCA + UCA + ZKP on every L3 promotion; lifecycle replayable       |

### 12.2 Emergency Protocol Hooks

See §3.2 emergency row. Additionally:

- `EP-009` (ACCESSIBILITY_EMERGENCY): cognitive-load-heavy skills demote one tier.
- `EP-012` (SUSTAINABILITY_VIOLATION): high-cost L4X execution paused.

### 12.3 Framework Integrity

`AG-014 FrameworkIntegrator` MUST be pinned at L2 minimum and is
exempt from idle-timeout eviction. The harness MUST refuse to load
any skill whose `version` field is incompatible with the active
framework version (drives `EP-013`).

### 12.4 Tenant Scoping

`load_candidate ⟺ skill.scope ⊆ current(tenant, env, ws, proj)`.
Cross-tenant promotion requires TIP (Tenant Isolation Proof).

---

## 13. Conformance Vectors

Twelve canonical test vectors. A v1.1-conformant harness MUST pass
all twelve.

| #  | Vector                          | Validates                                          |
|----|---------------------------------|----------------------------------------------------|
| A  | Cold start, OSCAL query         | L1→L2→L3 promotion on combined signal              |
| B  | Topic drift after 6 turns        | Drift demotion overrides recency                   |
| C  | Budget pressure                  | Eviction order: argmin score, lowest tier first    |
| D  | Mutex                            | `exclusiveOf` triggers atomic eviction             |
| E  | Explicit invocation bypass       | `$skill` mention promotes to L3 unless policy denies |
| F  | Implicit invocation disabled     | `implicitInvocation: false` blocks auto-promotion  |
| G  | Malicious ontology               | Invalid `@context` or forbidden predicate → validation fail |
| H  | Privacy mode                     | `public-anchor` bundle contains no prompt-derived subjects |
| I  | Script execution gate            | L4X denied when tool capability missing            |
| J  | Version skew                     | Incompatible harness/skill versions evict before scoring |
| K  | Duplicate skill names            | Canonical digest-based ID prevents collision       |
| L  | Synopsis mismatch                | Synopsis–body cosine < 0.55 → validator warning    |

### 13.1 Vector A — Cold Start, OSCAL Query

```
Registry:   13 CKODEX skills
GAL:        2
Turn 1:     "Can you validate this SSP against NIST 800-53 rev 5?"

Expected:
  ckodex-oscal:    S_lex=0.55  S_sem=0.82  S_ont=0.75  →  score=0.66  →  L1→L2→L3
  cortaix-csr:     S_lex=0.15  S_sem=0.45  S_ont=0.40  →  score=0.32  →  L1→L2
  ckodex-diagrams: S_lex=0.05  S_sem=0.18  S_ont=0.10  →  score=0.10  →  L1 (no change)
  others:          score < 0.20                       →  L1 (no change)
```

### 13.2 Vector B — Topic Drift

```
State at turn 7: ckodex-oscal at L3, loaded at turn 1 with score=0.71.
Turn 7: "Actually let's switch — can you draft a release announcement for v16.0?"

Expected:
  ckodex-oscal:        drift = 0.42 > θ_drift = 0.35  →  demote L3→L2
  ckodex-announcements: S_lex=0.60 S_sem=0.78 S_ont=0.70  →  score=0.71  →  L1→L2→L3
```

### 13.3 Vector C — Budget Pressure

```
State: 5 skills at L3, total = 17 400 tokens. B_pool = 8 192.
Expected: evict argmin-score in L3 until total ≤ 8 192. Emit one PCA
          decision per eviction with trigger="budget_pressure".
```

### 13.4 Vector D — Mutex

```
Registry: ckodex-foundationdb at L2.
New skill X declares ontology.exclusiveOf = ["ckodex-foundationdb"].
Turn: explicit X-shaped query.

Expected:
  X:                    promoted to L3.
  ckodex-foundationdb:  evicted atomically with X's promotion.
  PCA logs:             one bundle covering both transitions with trigger="mutex".
```

### 13.5 Vector E — Explicit Invocation Bypass

```
Turn: "Use ckodex-diagrams to render a proof-chain diagram."

Expected:
  ckodex-diagrams:  S_usr = 1.0 (explicit). policy_pre_filter passes.
                    Promoted L1→L2→L3 in one turn even if S_lex/S_sem/S_ont
                    individually below thresholds.
  If GAL < ontology.galMinimum:
                    policy denial; harness surfaces error;
                    skill remains at L1; no silent promotion.
```

### 13.6 Vector F — Implicit Invocation Disabled

```
Skill manifest: runtime.implicitInvocation = false.
Turn: generic query containing skill's trigger words but no explicit mention.

Expected:
  Skill: remains at L1. No auto-promotion regardless of score.
  Only $skill explicit mention promotes it.
```

### 13.7 Vector G — Malicious Ontology

```
Skill ships with ontology.@context = "https://attacker.example/ctx".
Or ontology.proofTypes includes "XYZ" not in enum.

Expected:
  validate.sh:  fail at §10.2 or §10.6.
  Bundle:       not installable.
```

### 13.8 Vector H — Privacy Mode

```
PCA emitted in mode "public-anchor".

Expected bundle contents:
  - bundle hash, policy_version, transition counts: present
  - skill_id as digest (sha256:...): present
  - skill human-readable name: ABSENT unless skill is on the public-skills allowlist
  - score components or weights: ABSENT
  - prompt-derived subjects or context embedding: ABSENT
```

### 13.9 Vector I — Script Execution Gate

```
Skill requests L3 → L4X transition. Required tool capability not granted.

Expected:
  L4X transition: denied.
  PCA: emits trigger="capability_denied" with the missing capability.
  L3 promotion: retained; user response uses L3 content only.
```

### 13.10 Vector J — Version Skew

```
Skill manifest: metadata.version = "0.9.0" incompatible with harness
framework v16.0.

Expected:
  Skill: evicted to L0 before scoring runs.
  EP-013 FRAMEWORK_INTEGRITY_BREACH: triggered.
  PCA: emits trigger="framework_break".
```

### 13.11 Vector K — Duplicate Skill Names

```
Two skills install with metadata.name = "pdf-processing" but different
content/authors.

Expected:
  Canonical ID = sha256(name ⊕ namespace ⊕ issuer).
  Both load independently under distinct canonical IDs.
  Display name disambiguation: <name>@<short-digest>.
```

### 13.12 Vector L — Synopsis Mismatch

```
Skill author edits SKILL.md body without regenerating synopsis.
Validator runs.

Expected:
  cosine(embed(synopsis), embed(SKILL.md body)) < 0.55.
  validate.sh: §10.4 warning (CKODEX-shipped: failure).
  Remediation: re-run gen_bundle.py --generate-synopsis.
```

---

## 14. Threat Model

| Threat                            | Mitigation                                                              |
|-----------------------------------|-------------------------------------------------------------------------|
| Skill squatting / name collision  | Canonical ID = `sha256(name ⊕ namespace ⊕ issuer)`                      |
| Malicious synopsis                | Build-time generation + synopsis–body semantic-diff check (Validator §10.4) |
| Ontology poisoning                | Signed ontology, fixed `@context`, predicate allowlist (Validator §10.2) |
| Prompt-induced overpromotion      | Hysteresis + `intent_to_invoke` gate + policy pre-filter                |
| Skill persistence attack          | Idle timeout + drift eviction + `task_complete` demotion                |
| Cross-tenant leakage              | TIP required before cross-tenant skill promotion                        |
| Script abuse (L4X)                | Sandbox + least privilege + capability allowlist + no ambient secrets   |
| Evidence leakage (PCA)            | Privacy modes + redaction + retention class                             |
| Implicit-invocation hijack        | `runtime.implicitInvocation: false` blocks auto-promotion               |
| Framework downgrade               | `EP-013` evicts version-incompatible skills before scoring              |

---

## 15. Capability Negotiation Handshake

For cross-harness interop, a v1.1-aware harness publishes a
capability descriptor at session start:

```jsonc
{
  "skill_runtime_capabilities": {
    "supports_synopsis_tier":      true,
    "supports_ontology_scoring":   true,
    "supports_hysteresis":         true,
    "supports_eviction":           true,
    "supports_lifecycle_pca":      true,
    "supports_l4x_sandbox":        true,
    "pca_privacy_modes":           ["debug-local", "audit-private"],
    "max_skill_pool_tokens":       8192,
    "default_weights":             { "lex": 0.15, "sem": 0.30, "ont": 0.25, "co": 0.10, "rec": 0.10, "usr": 0.10 }
  }
}
```

A skill MAY declare minimum capability requirements via
`metadata.annotations["ckodex.org/requires-capability"]`. A harness
that does not advertise the required capability MUST refuse to load
the skill rather than load with degraded behavior.

---

## 16. Acceptance Metrics

| Metric                                  | Target                                                |
|-----------------------------------------|-------------------------------------------------------|
| `skill_pool_token_reduction`            | ≥ 60% reduction vs no-demotion baseline               |
| `false_positive_L3_promotions`          | ≤ 5% on conformance suite                             |
| `false_negative_invocations`            | ≤ 3% for explicit-skill tasks                         |
| `skill_flap_rate`                       | ≤ 1 tier oscillation per 20 turns                     |
| `p95_router_latency`                    | ≤ 50 ms for N ≤ 50 skills with cached embeddings      |
| `evidence_privacy_violation_rate`       | 0 raw prompt fragments in `public-anchor` PCA mode    |
| `validator_pass_rate_v1_compat`         | 100% for CKODEX-shipped v1.1 skills                   |

Measurement methodology: a fixed-content conformance corpus
(`ckodex-skill-lifecycle-conformance-corpus.tgz`, separate
deliverable) replays a deterministic sequence of turns against a
test registry; metrics computed from emitted PCA bundles in
`debug-local` mode.

---

## 17. Implementation Architecture (sketch)

The runtime profile is implemented as a small embeddable kernel
plus harness-specific adapters, following the CKODEX three-split
pattern (kernel pure / validation reusable / adapters
transport-specific).

```
ckx-skill-kernel        // pure FSM, scoring, thresholds, budget planner
ckx-skill-registry      // scans SKILL.md + skill.json + OCI bundles
ckx-skill-validation    // JSON Schema + ontology validation + synopsis/body consistency
ckx-skill-router        // BM25 + embeddings + ontology graph + co-occurrence + recency
ckx-skill-evidence      // PCA bundle generation, privacy modes, signing, anchoring
ckx-skill-adapters      // codex, claude-code, microsoft-agent-framework, generic-fs
```

Full module specification deferred to **RFC-002: CKODEX Skill
Runtime Reference Implementation**.

---

## 18. Future Work

| Item                              | Target RFC / version                          |
|-----------------------------------|-----------------------------------------------|
| Reference Python harness module   | RFC-002                                       |
| Federated co-occurrence priors    | v1.2 (privacy-preserving cross-tenant learning) |
| Per-skill telemetry feedback      | v1.2                                          |
| Probabilistic L2.5 tier           | v1.2 (partial SKILL.md section load)          |
| Skill-graph compositional proofs  | v1.3 (`(p₁ ⊗ p₂)` over invocation chains)     |
| ZKP-attested ontology claims      | v1.3                                          |

---

## Appendix A — Schema Delta (`skill.v1.1.schema.json`)

Composition-safe via `unevaluatedProperties: false` per JSON Schema
2020-12 guidance. The v1 schema uses `additionalProperties: false`
internally; `allOf`-extension with `unevaluatedProperties` is the
spec-compliant way to add new top-level keys without breaking
closure.

```jsonc
{
  "$id": "https://schemas.ckodex.org/skill.v1.1.schema.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CKODEX Skill Manifest v1.1",
  "type": "object",
  "allOf": [
    { "$ref": "https://schemas.ckodex.org/skill.v1.schema.json" }
  ],
  "properties": {
    "apiVersion": {
      "type": "string",
      "enum": ["ckodex.org/skill/v1", "ckodex.org/skill/v1.1"]
    },
    "metadata": {
      "type": "object",
      "properties": {
        "synopsis": {
          "type": "string",
          "minLength": 1,
          "maxLength": 2048,
          "description": "Precomputed L2 synopsis (RFC-001 §3.3)."
        }
      }
    },
    "ontology": { "$ref": "#/$defs/SkillOntology" },
    "runtime":  { "$ref": "#/$defs/SkillRuntime" }
  },
  "unevaluatedProperties": false,
  "$defs": {
    "SkillOntology": {
      "type": "object",
      "required": ["@context"],
      "additionalProperties": false,
      "properties": {
        "@context":    { "const": "https://ckodex.org/skill-context/v1" },
        "subjects":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "produces":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "consumes":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "related":     { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "exclusiveOf": { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "triggers":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "galMinimum":  { "type": "integer", "minimum": 0, "maximum": 5 },
        "proofTypes":  {
          "type": "array",
          "items": { "type": "string", "enum": ["PCA", "UCA", "TIP", "ZKP", "DCA", "TKP", "WCAG", "CLP", "UFP"] },
          "uniqueItems": true
        }
      }
    },
    "SkillRuntime": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "implicitInvocation": { "type": "boolean", "default": true },
        "minTier":            { "type": "string", "enum": ["L0", "L1", "L2", "L3", "L4R", "L4X"] },
        "maxTier":            { "type": "string", "enum": ["L1", "L2", "L3", "L4R", "L4X"] },
        "l4xAllowed":         { "type": "boolean", "default": false }
      }
    }
  }
}
```

## Appendix B — Reference Score Computation (Python, normalized)

```python
import math
from dataclasses import dataclass

@dataclass
class SignalWeights:
    lex: float = 0.15
    sem: float = 0.30
    ont: float = 0.25
    co:  float = 0.10
    rec: float = 0.10
    usr: float = 0.10

@dataclass
class SkillSignals:
    s_lex: float   # ∈ [0,1]
    s_sem: float   # ∈ [0,1]
    s_ont: float   # ∈ [0,1] (already normalized; see ontology_score)
    s_co:  float   # ∈ [0,1]
    s_rec: float   # ∈ [0,1]
    s_usr: float   # ∈ {0.0, 0.6, 0.8, 1.0}
    s_pol: int     # ∈ {0,1} hard gate

def compute_score(sig: SkillSignals, w: SignalWeights = SignalWeights()) -> float:
    if sig.s_pol == 0:
        return 0.0
    return (
        w.lex * sig.s_lex +
        w.sem * sig.s_sem +
        w.ont * sig.s_ont +
        w.co  * sig.s_co  +
        w.rec * sig.s_rec +
        w.usr * sig.s_usr
    )

def recency_decay(turns_since_ref: int, lam: float = 0.15) -> float:
    return math.exp(-lam * turns_since_ref)

def ontology_score(ctx_subjects: set[str],
                   skill_subjects: set[str],
                   related_subjects: set[str]) -> float:
    """Returns S_ont normalized to [0,1]."""
    def jaccard(a: set, b: set) -> float:
        if not a or not b:
            return 0.0
        return len(a & b) / len(a | b)
    raw = jaccard(ctx_subjects, skill_subjects) + 0.5 * jaccard(ctx_subjects, related_subjects)
    return min(1.0, raw / 1.5)

def user_signal(explicit_mention: bool,
                pinned: bool,
                memory_preference: bool) -> float:
    if explicit_mention:    return 1.0
    if pinned:              return 0.8
    if memory_preference:   return 0.6
    return 0.0
```

## Appendix C — PCA Privacy Mode Decision Tree

```
need_to_record_lifecycle_decision()
  ├── is_local_dev_session? ─────────► debug-local
  ├── is_regulated_tenant?  ─────────► regulated-export
  ├── is_public_release_artifact? ───► public-anchor
  └── default ────────────────────────► audit-private
```

## Appendix D — Capability Negotiation Schema (separate file)

```jsonc
{
  "$id": "https://schemas.ckodex.org/skill-runtime-capabilities.v1.schema.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Skill Runtime Capabilities",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "supports_synopsis_tier":    { "type": "boolean" },
    "supports_ontology_scoring": { "type": "boolean" },
    "supports_hysteresis":       { "type": "boolean" },
    "supports_eviction":         { "type": "boolean" },
    "supports_lifecycle_pca":    { "type": "boolean" },
    "supports_l4x_sandbox":      { "type": "boolean" },
    "pca_privacy_modes": {
      "type": "array",
      "items": { "type": "string", "enum": ["debug-local", "audit-private", "public-anchor", "regulated-export"] },
      "uniqueItems": true
    },
    "max_skill_pool_tokens": { "type": "integer", "minimum": 0 },
    "default_weights": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "lex": { "type": "number", "minimum": 0, "maximum": 1 },
        "sem": { "type": "number", "minimum": 0, "maximum": 1 },
        "ont": { "type": "number", "minimum": 0, "maximum": 1 },
        "co":  { "type": "number", "minimum": 0, "maximum": 1 },
        "rec": { "type": "number", "minimum": 0, "maximum": 1 },
        "usr": { "type": "number", "minimum": 0, "maximum": 1 }
      }
    }
  }
}
```

---

## Review Disposition (v0.1.0 → v0.2.0)

| Reviewer comment                                              | Status                                  |
|---------------------------------------------------------------|-----------------------------------------|
| Correct two-tier baseline claim                               | ACCEPTED (§1.1)                         |
| Split L4 into L4R (read) and L4X (execute)                    | ACCEPTED (§3)                           |
| `synopsis` MUST → SHOULD (CKODEX-shipped: MUST)               | ACCEPTED (§2, §10.1)                    |
| Top-level upstream pollution                                  | CLARIFIED — `skill.json` is already CKODEX companion; SKILL.md unchanged (§7) |
| Schema composition: `unevaluatedProperties: false`            | ACCEPTED (Appendix A)                   |
| Normalize ontology score; graded `S_usr`                      | ACCEPTED (§4.1, §4.3, Appendix B)       |
| PCA privacy modes                                             | ACCEPTED (§9, Appendix C)               |
| Reframe as runtime profile, not upstream replacement          | ACCEPTED (title, abstract, scope)       |
| Threat model section                                          | ACCEPTED (§14)                          |
| Capability negotiation handshake                              | ACCEPTED (§15, Appendix D)              |
| Acceptance metrics with numeric targets                       | ACCEPTED (§16)                          |
| Conformance vectors E..L                                      | ACCEPTED (§13.5..§13.12)                |
| Implementation architecture (kernel + adapters)               | ACCEPTED as §17 sketch; full module → RFC-002 |
| OpenAI Codex 2% / 8000-char cap citation                      | DEFERRED — structural claim kept; specific numbers pending primary-source verification |
| Rename `skill.json` → `skill.ckodex.json`                     | REJECTED — breaks 10 shipped CKODEX bundles; scope clarified inline (§7.1) |

---

*End of RFC-001 v0.2.0.*
