# RFC-001: Progressive Skill Lifecycle for Agent Harnesses

| Field        | Value                                                       |
|--------------|-------------------------------------------------------------|
| Status       | Draft                                                       |
| Version      | 0.1.0                                                       |
| Authors      | Ckodex Labs                                                 |
| Target spec  | `agent-skills` v1 → v1.1                                    |
| Framework    | CKODEX v16.0                                                |
| Date         | 2026-05-23                                                  |
| Supersedes   | —                                                           |
| Related      | `skill.v1.schema.json`, `skillsbundle.v1.schema.json`       |

---

## 0. Abstract

The current `agent-skills` v1 specification defines two skill states in the
agent's context — metadata (`name + description`, ~80 tokens) and full
(`SKILL.md` body, ~3000–5000 tokens) — and exactly one transition direction
(metadata → full). It defines no eviction policy, no multi-signal load
heuristic, and no ontological signal beyond the free-text `description`.

The observed failure mode is context bloat: harnesses promote skills on
weak signal, never demote, and accumulate idle skill bodies that displace
the user's actual working context. This RFC proposes a five-tier
progressive-disclosure FSM, a multi-signal scoring function with
hysteresis, an explicit eviction policy, and an optional JSON-LD ontology
block that lets harnesses reason over a skill graph. The proposal is
fully backward-compatible: v1 skills work unchanged, degrading gracefully
to lexical + semantic scoring only.

---

## 1. Motivation

### 1.1 Observed Failure

Harnesses currently load all available skills' metadata at session start
(~80 tokens × N skills) and promote any skill whose description
lexically overlaps the user turn. Once promoted, the full `SKILL.md`
body stays resident for the remainder of the session. With N=13 skills
and an average body of 3000 tokens, worst-case skill-induced context
occupancy is ~39 000 tokens — a non-trivial fraction of every modern
model's context window, spent on skills the user never invoked.

### 1.2 Root Causes

1. **Two states are too few.** There is no intermediate tier between
   `description` (~80t) and `SKILL.md` body (~3000t+), so any signal
   strong enough to promote past metadata commits to the full body.
2. **No reverse arrow.** The spec defines no demotion or eviction.
3. **One signal.** The `description` field is the entire load oracle.
   No embedding similarity, no co-occurrence prior, no ontology, no
   recency, no policy gate.
4. **No budget concept.** The harness has no notion of a skill-pool
   token budget separate from conversation context.

### 1.3 Goals

| Goal                                          | Mechanism                            |
|-----------------------------------------------|--------------------------------------|
| Bound the token cost of the skill pool         | Five-tier FSM + budget enforcement   |
| Load skills only when justified                | Multi-signal scoring + thresholds    |
| Unload skills when no longer justified         | Demotion + drift-based eviction      |
| Avoid tier-flap near decision boundaries       | Hysteresis                           |
| Preserve audit trail                           | PCA-attested lifecycle transitions   |
| Backward-compatible with `agent-skills` v1     | All new fields optional              |

### 1.4 Non-Goals

- This RFC does **not** change the `SKILL.md` body format.
- It does **not** mandate a specific embedding model or vector store.
- It does **not** prescribe harness UI for skill state visibility.

---

## 2. Locked Defaults

Three forks identified during exploration are resolved as follows:

| Decision                     | Choice                                | Rationale                                      |
|------------------------------|---------------------------------------|------------------------------------------------|
| L2 synopsis source           | Build-time precomputed                | Deterministic, signable, auditable             |
| Ontology block placement     | Top-level `ontology` in `skill.json`  | Clean schema, justifies v1 → v1.1 bump         |
| Score transparency           | PCA bundle only                       | Proof-native, no operator UI noise             |

---

## 3. Five-Tier Lifecycle FSM

```
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │  L0_registered  ──►  L1_metadata  ──►  L2_synopsis  ──►  L3_full  ──►  L4_resources
  │   (~10t/skill)        (~80t)            (~400t)            (~3-5kt)          (per-file)
  │                                                             │
  │  ◄──────────────  ◄────────────────  ◄──────────────  ◄──────────  (demotion paths)
  │                                                             │
  └─────────────────────────────────────────────────────────────┘
```

### 3.1 State Definitions

| Tier | Content                          | Typical tokens | Source                              |
|------|----------------------------------|----------------|-------------------------------------|
| L0   | Name only                        | ~10            | Disk registry index                 |
| L1   | Name + description               | ~80            | `skill.json` / SKILL.md frontmatter |
| L2   | Compressed synopsis              | ~400           | `metadata.synopsis` (precomputed)   |
| L3   | Full `SKILL.md` body             | ~3000–5000     | `SKILL.md`                          |
| L4   | Referenced scripts/refs/assets   | per-file       | `resources.*` paths                 |

### 3.2 Transitions

| From | To  | Trigger                                                      |
|------|-----|--------------------------------------------------------------|
| L0   | L1  | Registry scan within budget                                  |
| L1   | L2  | `score ≥ τ_syn_load`                                         |
| L2   | L3  | `score ≥ τ_full_load ∧ intent_to_invoke`                     |
| L3   | L4  | Explicit file reference in `SKILL.md`                        |
| L3   | L2  | `idle ≥ N_full ∨ score < τ_full_keep`                        |
| L2   | L1  | `idle ≥ N_syn ∨ drift ≥ θ_drift`                             |
| L1   | L0  | `registry_pressure ∧ rank_below_floor`                       |
| any  | L1  | `emergency_evict(context_overflow ≥ 0.85)`                   |

### 3.3 The L2 Synopsis Tier

L2 is the load-bearing addition. A precomputed ~400-token block —
"what this does / when it triggers / what it produces / what it
explicitly does *not* cover" — stays warm cheaply and gates the
expensive L3 promotion. Most skills currently flapping between L1 and
L3 are L2-relevant but never L3-needed.

The synopsis MUST be a single string field at `metadata.synopsis`
(see Appendix A schema delta) bounded to 2048 characters. It MUST be
generated by the skill author at bundle build time (e.g. in
`gen_bundle.py`) and SHOULD be a faithful compression of the
`SKILL.md` body, not marketing copy.

---

## 4. Multi-Signal Scoring

### 4.1 Score Function

```
SCORE(s, ctx) : Real ∈ [0,1]

  S_lex  := BM25(s.triggers ⊕ s.keywords, recent_K_turns)
  S_sem  := cos(embed(ctx_window), embed(s.description ⊕ s.synopsis))
  S_ont  := jaccard(ctx.subjects, s.ontology.subjects)
            ⊕ 0.5 · graph_overlap(ctx.subjects, s.ontology.related)
  S_co   := P(s | currently_loaded_skills)         // co-occurrence prior
  S_rec  := exp(-λ · turns_since_last_reference)   // recency decay
  S_usr  := explicit_mention ∨ pinned ∨ memory_preference   // ∈ {0,1}
  S_pol  := policy_gate(GAL, tenant, env, ws)              // hard gate

  score  := Σᵢ wᵢ · Sᵢ        if S_pol = 1
         := 0                  if S_pol = 0
```

### 4.2 Default Weights

| Signal  | Weight | Rationale                                                  |
|---------|--------|------------------------------------------------------------|
| S_sem   | 0.30   | Strongest single signal; embedding captures intent         |
| S_ont   | 0.25   | CKODEX differentiator; structural, not lexical             |
| S_lex   | 0.15   | Cheap, catches explicit keyword matches                    |
| S_co    | 0.10   | Captures "skill A often precedes skill B" patterns         |
| S_rec   | 0.10   | Skill referenced recently is more likely to be referenced again |
| S_usr   | 0.10   | Honors explicit user intent                                |

Weights are tunable per harness profile. The harness MUST publish the
active weight vector in any lifecycle PCA bundle.

### 4.3 Policy Pre-Filter

`S_pol` is a hard gate, not a weighted score. A skill fails the
pre-filter and is excluded from candidacy if any of:

- `skill.governance.galMinimum > current_gal`
- `skill.governance.proofRequired ∧ ¬proof_pipeline_available`
- skill scope ⊄ current `(tenant, env, workspace, project)`
- `skill.metadata.version` incompatible with harness framework version

---

## 5. Hysteresis & Thresholds

```
τ_syn_load   = 0.45   // L1 → L2
τ_syn_keep   = 0.30   // hold L2
τ_full_load  = 0.65   // L2 → L3
τ_full_keep  = 0.40   // hold L3
τ_floor      = 0.20   // demote all

invariant:   τ_load > τ_keep > τ_floor    // anti-flap
band:        [τ_keep, τ_load]              // skill remains in current tier
```

Hysteresis is non-negotiable. Without `τ_load > τ_keep`, a skill
scoring near the boundary thrashes between tiers every turn, burning
re-load tokens and producing audit noise.

---

## 6. Eviction Policy

A loaded skill is demoted or evicted when any of the following hold:

| Trigger          | Condition                                                  |
|------------------|------------------------------------------------------------|
| Score decay      | `score < τ_keep(current_tier) ∧ idle ≥ 3 turns`            |
| Idle timeout     | `idle_turns > N_idle(tier)`, where `N_idle = {L3:5, L2:15, L1:∞}` |
| Topic drift      | `cos(ctx_now, ctx_at_load) < θ_drift` (default `θ_drift = 0.35`) |
| Budget pressure  | `Σ tokens(loaded) > B_pool` → evict argmin score, lowest tier first |
| Task complete    | Explicit end signal OR `skill.exit_criteria_met`           |
| Mutex            | `load(B) ∧ B ∈ A.ontology.exclusiveOf` → evict A           |
| Framework break  | `skill.version ⊭ harness.framework.compat`                 |

Topic drift is the most important trigger: it catches the
"loaded 20 turns ago, conversation moved on" failure mode that
recency and score-decay alone miss. The harness MUST track the
centroid embedding of the last K turns; when a skill's home-topic
embedding (its description+synopsis embedding at load time) is far
from the current centroid, demote regardless of recency.

---

## 7. JSON-LD Ontology Block (skill.v1.1)

Skills already carry semantics in their `description`. The v1.1 spec
exposes that semantics structurally.

### 7.1 Example

```jsonc
{
  "apiVersion": "ckodex.org/skill/v1.1",
  "kind": "Skill",
  "metadata": {
    "name": "ckodex-oscal",
    "description": "Author, read, validate, convert, or reason about OSCAL...",
    "synopsis": "When the user mentions OSCAL, NIST 800-53/171, FedRAMP, SSP, POA&M, control catalogs, profiles, baselines, component definitions, or compliance-as-code: this skill resolves OSCAL profiles, validates catalog/profile/SSP documents against NIST schemas, converts between XML/JSON/YAML, and emits UCA attestations for downstream proof pipelines. NOT a substitute for cortaix-csr (factory baseline) or cortaix-iso (ISO 27001).",
    "version": "1.0.0"
  },
  "ontology": {
    "@context": "https://ckodex.org/skill-context/v1",
    "ckodex:subjects":    ["oscal", "nist:800-53", "compliance", "fedramp"],
    "ckodex:produces":    ["ckodex:UCA"],
    "ckodex:consumes":    ["oscal:catalog", "oscal:profile"],
    "ckodex:related":     ["cortaix-csr", "ckodex-announcements"],
    "ckodex:exclusiveOf": [],
    "ckodex:triggers":    ["OSCAL", "SSP", "POA&M", "FedRAMP", "control catalog"],
    "ckodex:gal-min":     2,
    "ckodex:proof-types": ["UCA"]
  }
}
```

### 7.2 Harness Capabilities Unlocked

1. **Pre-warm via `produces`/`consumes` edges.** Loading `cortaix-csr`
   (produces UCA) nudges `ckodex-oscal` (consumes nothing UCA-shaped
   directly, but produces UCA itself) from L1 to L2.
2. **Mutex enforcement via `exclusiveOf`.** A harness MUST evict any
   loaded skill listed in the `exclusiveOf` array of a newly-loaded
   skill, atomically.
3. **Policy pre-filter via `gal-min` and `proof-types`.** Candidates
   failing `gal-min ≤ current_gal` are dropped before scoring.
4. **Structural relatedness via `subjects` / `related`.** Jaccard
   over `subjects`, weighted graph-distance over `related`, supplies
   `S_ont` without requiring an embedding round-trip.

### 7.3 Backward Compatibility

Skills without an `ontology` block degrade gracefully: `S_ont := 0`
and the policy pre-filter passes by default. The remaining signals
(`S_lex`, `S_sem`, `S_co`, `S_rec`, `S_usr`) carry the full scoring
load — exactly as a harness without ontology awareness would compute.

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

    # Promotions
    for s in candidates:
        if s.tier == L1 and s.score >= TAU_SYN_LOAD:
            promote(s, L2, harness_state)
        if s.tier == L2 and s.score >= TAU_FULL_LOAD and intent_to_invoke(s, msg):
            promote(s, L3, harness_state)

    # Demotions
    for s in harness_state.loaded:
        if drift(s, ctx_emb) > THETA_DRIFT or idle_turns(s) > N_IDLE[s.tier]:
            demote(s, s.tier - 1, harness_state)
        elif s.score < TAU_KEEP[s.tier]:
            demote(s, s.tier - 1, harness_state)

    # Budget enforcement
    while total_tokens(harness_state.loaded) > B_POOL:
        victim = argmin_score(harness_state.loaded, prefer_tier=L3)
        evict(victim, harness_state)

    # CKODEX-only: attest the lifecycle decisions
    if harness_state.gal >= 3:
        emit_pca(LifecycleDecisionBundle(harness_state))

    return respond_using(harness_state.loaded_at_L2_or_higher)
```

Per-turn cost: ~N cosine sims + lexical scan + ontology lookups + drift
check. For N=13 skills, dominated by embedding inference; total
overhead measured at <50 ms on a typical harness.

---

## 9. PCA Evidence Schema for Lifecycle Decisions

When `current_gal ≥ 3`, every lifecycle transition MUST be entered into
a Proof-Carrying Action bundle:

```jsonc
{
  "predicate": "ckodex/skill-lifecycle@v1",
  "subject":   "urn:ckodex:harness:<harness-id>:turn:<turn-uuid>",
  "timestamp": "2026-05-23T14:32:11.123456789Z",
  "decisions": [
    {
      "skill":       "ckodex-oscal",
      "from_tier":   "L1",
      "to_tier":     "L2",
      "trigger":     "score_threshold",
      "score":       0.47,
      "components":  { "S_lex": 0.2, "S_sem": 0.6, "S_ont": 0.5, "S_co": 0.1, "S_rec": 0.0, "S_usr": 0.0 },
      "weights":     { "lex": 0.15, "sem": 0.30, "ont": 0.25, "co": 0.10, "rec": 0.10, "usr": 0.10 }
    },
    {
      "skill":       "cortaix-csr",
      "from_tier":   "L3",
      "to_tier":     "L2",
      "trigger":     "topic_drift",
      "drift":       0.41,
      "theta":       0.35,
      "idle_turns":  7
    }
  ],
  "context_pool_tokens_before": 4820,
  "context_pool_tokens_after":  2110,
  "budget_max":                 8192
}
```

The bundle MUST be signed (cosign) and anchored to Rekor following the
standard CKODEX PCA discipline.

---

## 10. Validator Hooks (v1.1)

The `agent-skills` validator gains four new checks for v1.1 skills:

| §   | Check                                                                  |
|-----|------------------------------------------------------------------------|
| §1  | `metadata.synopsis` present (warn if absent) and ≤ 2048 chars          |
| §2  | If `ontology` present, `@context` resolves and all referenced predicates are valid |
| §3  | `ontology.subjects`, `ontology.related`, `ontology.exclusiveOf` are unique-item arrays |
| §4  | `ontology.gal-min` ∈ [0..5]; `ontology.proof-types` ⊆ enum from skill.v1 governance.proofTypes |

Target: **0 failures / 0 warnings** per the standard CKODEX
shipped-skill criterion.

---

## 11. Migration v1 → v1.1

| Step | Action                                                                |
|------|-----------------------------------------------------------------------|
| 1    | Skill author bumps `apiVersion` to `ckodex.org/skill/v1.1`.           |
| 2    | Skill author runs `gen_bundle.py --generate-synopsis` to populate `metadata.synopsis`. |
| 3    | Skill author adds optional `ontology` block (recommended but not required). |
| 4    | `validate.sh` re-runs to 0F/0W under v1.1 rules.                      |
| 5    | Bundle is re-packed; supersedes the v1 artifact via standard semver/promotion path. |

v1 skills remain installable and runnable in v1.1-aware harnesses; they
participate in scoring with `S_ont := 0` and an empty `synopsis` (the
harness falls back to truncating the description for L2 content).

---

## 12. CKODEX-Specific Bindings

These bindings sit outside the upstream `agent-skills` spec and are
honored only by CKODEX-aware harnesses.

### 12.1 GAL Gates

| GAL  | Lifecycle proof requirement                                       |
|------|-------------------------------------------------------------------|
| 0–2  | PCA optional; transitions logged but not attested                 |
| 3    | PCA mandatory on every transition                                 |
| 4    | PCA + UCA cross-reference on every L3 promotion                   |
| 5    | PCA + UCA + ZKP on every L3 promotion; lifecycle replayable       |

### 12.2 Emergency Protocols

- `EP-001` (STOP_ALL_AGENTS): all skills → L1, freeze registry.
- `EP-002` (QUARANTINE_AGENT): quarantined agent's skill pool → L0.
- `EP-013` (FRAMEWORK_INTEGRITY_BREACH): version-incompatible skills auto-evict.
- `EP-014` (CONTAINMENT_BREACH): full skill pool snapshot to forensic evidence bundle before eviction.

### 12.3 Framework Integrity

`AG-014 FrameworkIntegrator` MUST be pinned at L2 minimum and is
exempt from idle-timeout eviction. The harness MUST refuse to load
any skill whose `version` field is incompatible with the active
framework version.

### 12.4 Tenant Scoping

`load_candidate ⟺ skill.scope ⊆ current(tenant, env, ws, proj)`.
Cross-tenant promotion requires a TIP (Tenant Isolation Proof).

---

## 13. Test Vectors

### 13.1 Vector A — Cold Start, OSCAL Query

```
Registry:   13 CKODEX skills
GAL:        2
Turn 1:     "Can you validate this SSP against NIST 800-53 rev 5?"

Expected:
  ckodex-oscal:    S_lex=0.55, S_sem=0.82, S_ont=0.75  →  score=0.66  →  L1 → L2 → L3
  cortaix-csr:     S_lex=0.15, S_sem=0.45, S_ont=0.40  →  score=0.32  →  L1 → L2
  ckodex-diagrams: S_lex=0.05, S_sem=0.18, S_ont=0.10  →  score=0.10  →  L1 (no change)
  others:          score < 0.20                         →  L1 (no change)
```

### 13.2 Vector B — Topic Drift After 6 Turns

```
State at turn 7: ckodex-oscal at L3, score=0.71 at load (turn 1).
Turn 7: "Actually let's switch — can you draft a release announcement for v16.0?"

Expected:
  ckodex-oscal:        drift(ctx_now, ctx_at_load) = 0.42 > θ_drift = 0.35
                       → demote L3 → L2
  ckodex-announcements: S_lex=0.60, S_sem=0.78, S_ont=0.70 → score=0.71
                       → L1 → L2 → L3
```

### 13.3 Vector C — Budget Pressure

```
State: 5 skills at L3, total skill-pool tokens = 17 400, B_pool = 8 192.
Expected: evict argmin-score in L3 until total ≤ B_pool; emit PCA budget_pressure entry per eviction.
```

### 13.4 Vector D — Mutex

```
Registry: ckodex-foundationdb at L2, ckodex-redis-hypothetical declares
  ontology.exclusiveOf = ["ckodex-foundationdb"].
Turn: explicit Redis-shaped query.

Expected:
  ckodex-redis-hypothetical: promoted to L3.
  ckodex-foundationdb:       evicted atomically with promotion; PCA logs mutex trigger.
```

---

## 14. Future Work

| Item                              | v1.2 target?                                |
|-----------------------------------|---------------------------------------------|
| Federated co-occurrence priors    | Learn `S_co` across tenants with privacy preservation |
| Per-skill telemetry feedback      | Skill author publishes preferred thresholds  |
| Probabilistic L2.5 tier           | Partial body load (sections of `SKILL.md`)   |
| Skill-graph compositional proofs  | `(p₁ ⊗ p₂)` over a chain of skill invocations |
| ZKP-attested ontology claims      | Prove ontology consistency without revealing topic vector |

---

## Appendix A — Schema Delta (`skill.v1.1.schema.json`)

```jsonc
{
  "$id": "https://schemas.ckodex.org/skill.v1.1.schema.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CKODEX Skill Manifest v1.1",
  "allOf": [
    { "$ref": "https://schemas.ckodex.org/skill.v1.schema.json" }
  ],
  "properties": {
    "apiVersion": {
      "type": "string",
      "enum": ["ckodex.org/skill/v1", "ckodex.org/skill/v1.1"]
    },
    "metadata": {
      "properties": {
        "synopsis": {
          "type": "string",
          "minLength": 1,
          "maxLength": 2048,
          "description": "Precomputed L2 synopsis: a faithful ~400-token compression of SKILL.md body suitable for the synopsis tier of the progressive disclosure FSM (RFC-001)."
        }
      }
    },
    "ontology": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "@context": {
          "type": "string",
          "format": "uri",
          "const": "https://ckodex.org/skill-context/v1"
        },
        "ckodex:subjects":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:produces":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:consumes":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:related":     { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:exclusiveOf": { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:triggers":    { "type": "array", "items": { "type": "string", "maxLength": 64 }, "uniqueItems": true },
        "ckodex:gal-min":     { "type": "integer", "minimum": 0, "maximum": 5 },
        "ckodex:proof-types": {
          "type": "array",
          "items": { "type": "string", "enum": ["PCA", "UCA", "TIP", "ZKP", "DCA", "TKP", "WCAG", "CLP", "UFP"] },
          "uniqueItems": true
        }
      }
    }
  }
}
```

## Appendix B — Reference Score Computation (Python)

```python
import math
from dataclasses import dataclass
from typing import Iterable

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
    s_sem: float
    s_ont: float
    s_co:  float
    s_rec: float
    s_usr: float
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
    def jaccard(a: set, b: set) -> float:
        if not a or not b: return 0.0
        return len(a & b) / len(a | b)
    return jaccard(ctx_subjects, skill_subjects) + 0.5 * jaccard(ctx_subjects, related_subjects)
```

---

## Acknowledgments

This RFC consolidates observations from CKODEX v15.0/v16.0 production
deployments where harnesses with N ≥ 10 installed skills exhibited
context occupancy >25% from idle skill bodies. The five-tier FSM,
synopsis tier, and topic-drift trigger emerged from those traces.

---

*End of RFC-001.*
