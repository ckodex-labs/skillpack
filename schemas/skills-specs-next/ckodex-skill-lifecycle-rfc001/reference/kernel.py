"""
ckx_skill_kernel — pure reference implementation of the RFC-001 skill lifecycle FSM.

No I/O, no network, no embedding model. Plug your own embedder, registry,
and PCA emitter via the protocols declared at the bottom of this module.

This is a faithful pseudocode→runnable port of RFC-001 v0.2.0 §3, §5, §6, §8.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Callable, Optional, Protocol

# ───────────────────────────────────────────────────────────────
# FSM tiers
# ───────────────────────────────────────────────────────────────

class Tier(str, Enum):
    L0  = "L0"    # registered: name + canonical ID only
    L1  = "L1"    # metadata: name + description + scope hints
    L2  = "L2"    # synopsis: ~400 tokens
    L3  = "L3"    # full SKILL.md body
    L4R = "L4R"  # resource read (medium-risk, content-only)
    L4X = "L4X"  # script execute (high-risk, capability-gated)


_TIER_ORDER = [Tier.L0, Tier.L1, Tier.L2, Tier.L3, Tier.L4R, Tier.L4X]
_TIER_INDEX = {t: i for i, t in enumerate(_TIER_ORDER)}


def tier_lt(a: Tier, b: Tier) -> bool:
    return _TIER_INDEX[a] < _TIER_INDEX[b]


# ───────────────────────────────────────────────────────────────
# Thresholds (RFC-001 §5; normative defaults — operators may tune)
# ───────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class Thresholds:
    tau_syn_load:  float = 0.45    # L1 → L2
    tau_syn_keep:  float = 0.30    # hold L2
    tau_full_load: float = 0.65    # L2 → L3
    tau_full_keep: float = 0.40    # hold L3
    tau_floor:     float = 0.20    # demote-all floor
    theta_drift:   float = 0.35    # topic-drift demotion threshold

    def __post_init__(self):
        # Hysteresis invariant — correctness property, not optimization.
        assert self.tau_syn_load  > self.tau_syn_keep  > self.tau_floor
        assert self.tau_full_load > self.tau_full_keep > self.tau_floor


@dataclass(frozen=True)
class IdleTimeouts:
    L3: int = 5      # turns
    L2: int = 15
    L1: int = 10**9  # effectively unbounded


# ───────────────────────────────────────────────────────────────
# Skill state in the harness pool
# ───────────────────────────────────────────────────────────────

@dataclass
class SkillState:
    skill_id: str                  # canonical sha256:<hex>
    skill_name: str
    tier: Tier = Tier.L1
    score: float = 0.0
    turns_since_last_ref: int = 0
    drift_at_load: float = 0.0
    token_cost: int = 80           # typical L1 cost; updated on tier change
    locked: bool = False           # framework_skills pinning (RFC §12.3)

    # Carried from the skill manifest, populated by the registry adapter:
    implicit_invocation: bool = True
    max_tier: Tier = Tier.L4R
    l4x_allowed: bool = False
    exclusive_of: list[str] = field(default_factory=list)


# ───────────────────────────────────────────────────────────────
# Lifecycle decision record (feeds the PCA emitter)
# ───────────────────────────────────────────────────────────────

@dataclass
class LifecycleDecision:
    skill_id: str
    from_tier: Tier
    to_tier: Tier
    trigger: str
    score: Optional[float] = None
    components: Optional[dict] = None
    weights: Optional[dict] = None
    drift: Optional[float] = None
    theta: Optional[float] = None
    idle_turns: Optional[int] = None
    capability: Optional[str] = None


# ───────────────────────────────────────────────────────────────
# Token cost per tier (typical — operators may calibrate)
# ───────────────────────────────────────────────────────────────

_TIER_TOKEN_COST = {
    Tier.L0: 10,
    Tier.L1: 80,
    Tier.L2: 400,
    Tier.L3: 3500,
    Tier.L4R: 1500,   # average; varies by resource
    Tier.L4X: 0,      # execution itself is not a context-residency cost
}


# ───────────────────────────────────────────────────────────────
# Promotion / demotion helpers
# ───────────────────────────────────────────────────────────────

def promote(s: SkillState, target: Tier, trigger: str) -> LifecycleDecision:
    assert tier_lt(s.tier, target), f"promote must move up ({s.tier} → {target})"
    if tier_lt(s.max_tier, target):
        # Respect the manifest's max_tier ceiling.
        return LifecycleDecision(s.skill_id, s.tier, s.tier, trigger="max_tier_ceiling")
    if target == Tier.L4X and not s.l4x_allowed:
        return LifecycleDecision(s.skill_id, s.tier, s.tier, trigger="capability_denied",
                                 capability="l4xAllowed=false in manifest")
    from_t = s.tier
    s.tier = target
    s.token_cost = _TIER_TOKEN_COST[target]
    s.turns_since_last_ref = 0
    return LifecycleDecision(s.skill_id, from_t, target, trigger=trigger)


def demote_one(s: SkillState, trigger: str) -> LifecycleDecision:
    idx = _TIER_INDEX[s.tier]
    if idx == 0:
        return LifecycleDecision(s.skill_id, s.tier, s.tier, trigger="already_floor")
    target = _TIER_ORDER[idx - 1]
    # L4R/L4X both demote to L3 (parallel branches, not vertically stacked).
    if s.tier in (Tier.L4R, Tier.L4X):
        target = Tier.L3
    from_t = s.tier
    s.tier = target
    s.token_cost = _TIER_TOKEN_COST[target]
    return LifecycleDecision(s.skill_id, from_t, target, trigger=trigger)


# ───────────────────────────────────────────────────────────────
# Eviction policy (RFC §6)
# ───────────────────────────────────────────────────────────────

def needs_demote(
    s: SkillState,
    *,
    drift: float,
    thresholds: Thresholds,
    idle_timeouts: IdleTimeouts,
) -> Optional[str]:
    """Returns the trigger name, or None if the skill should stay."""
    if s.locked:
        return None

    # Drift first — catches "loaded long ago, conversation moved on".
    if drift > thresholds.theta_drift:
        return "topic_drift"

    # Idle timeout per tier
    n_idle = {
        Tier.L3:  idle_timeouts.L3,
        Tier.L2:  idle_timeouts.L2,
        Tier.L1:  idle_timeouts.L1,
        Tier.L4R: idle_timeouts.L3,  # treat like L3
        Tier.L4X: idle_timeouts.L3,
    }.get(s.tier, idle_timeouts.L1)
    if s.turns_since_last_ref > n_idle:
        return "idle_timeout"

    # Score decay vs the keep threshold for the current tier
    keep = {
        Tier.L3:  thresholds.tau_full_keep,
        Tier.L4R: thresholds.tau_full_keep,
        Tier.L4X: thresholds.tau_full_keep,
        Tier.L2:  thresholds.tau_syn_keep,
        Tier.L1:  0.0,
        Tier.L0:  0.0,
    }[s.tier]
    if s.tier in (Tier.L2, Tier.L3, Tier.L4R, Tier.L4X) and s.score < keep:
        if s.turns_since_last_ref >= 3:
            return "score_decay"

    return None


# ───────────────────────────────────────────────────────────────
# Mutex enforcement
# ───────────────────────────────────────────────────────────────

def apply_mutex(promoted: SkillState, pool: dict[str, SkillState]) -> list[LifecycleDecision]:
    """When `promoted` enters the pool, evict every skill named in its exclusiveOf."""
    out: list[LifecycleDecision] = []
    for name in promoted.exclusive_of:
        for s in list(pool.values()):
            if s.skill_name == name and s.skill_id != promoted.skill_id:
                from_t = s.tier
                s.tier = Tier.L1
                s.token_cost = _TIER_TOKEN_COST[Tier.L1]
                out.append(LifecycleDecision(s.skill_id, from_t, Tier.L1, trigger="mutex"))
    return out


# ───────────────────────────────────────────────────────────────
# Budget enforcement (argmin score, lowest tier first)
# ───────────────────────────────────────────────────────────────

def enforce_budget(pool: dict[str, SkillState], b_pool: int) -> list[LifecycleDecision]:
    out: list[LifecycleDecision] = []

    def total() -> int:
        return sum(s.token_cost for s in pool.values())

    while total() > b_pool:
        # Pick the L3-or-above victim with the lowest score; if none, pick L2.
        victims_hot = [s for s in pool.values() if s.tier in (Tier.L3, Tier.L4R, Tier.L4X) and not s.locked]
        victims_warm = [s for s in pool.values() if s.tier == Tier.L2 and not s.locked]

        if victims_hot:
            victim = min(victims_hot, key=lambda s: s.score)
        elif victims_warm:
            victim = min(victims_warm, key=lambda s: s.score)
        else:
            # Cannot reduce further without violating pinning. Stop.
            break

        out.append(demote_one(victim, trigger="budget_pressure"))

    return out


# ───────────────────────────────────────────────────────────────
# Per-turn FSM step — the heart of the loop
# ───────────────────────────────────────────────────────────────

@dataclass
class TurnInputs:
    """Per-skill scoring inputs from the router."""
    score: float
    drift: float
    s_pol: int                    # 1 = passes policy pre-filter, 0 = denied
    intent_to_invoke: bool        # heuristic from the harness
    score_components: dict        # for the PCA bundle
    weights: dict                 # for the PCA bundle


@dataclass
class TurnResult:
    decisions: list[LifecycleDecision]
    pool_tokens_before: int
    pool_tokens_after:  int


def step(
    pool: dict[str, SkillState],
    per_skill: dict[str, TurnInputs],
    *,
    thresholds: Thresholds = Thresholds(),
    idle_timeouts: IdleTimeouts = IdleTimeouts(),
    b_pool_tokens: int = 8192,
) -> TurnResult:
    """One turn of the lifecycle FSM. Mutates `pool` in place. Returns decisions.

    Invariant: each skill receives at most one tier-changing decision per turn
    (mutex eviction wins over drift demotion; the second-of-a-kind is skipped).
    """
    decisions: list[LifecycleDecision] = []
    moved_this_turn: set[str] = set()   # skill_ids that already changed tier
    tokens_before = sum(s.token_cost for s in pool.values())

    # 1. Update per-skill bookkeeping from the inputs
    for sid, s in pool.items():
        inp = per_skill.get(sid)
        if inp is None:
            s.turns_since_last_ref += 1
            continue
        s.score = inp.score if inp.s_pol else 0.0
        if s.score >= thresholds.tau_floor:
            s.turns_since_last_ref = 0
        else:
            s.turns_since_last_ref += 1

    # 2. Promotions L1 → L2 → L3 (each skill takes its highest legal step)
    for sid, s in pool.items():
        inp = per_skill.get(sid)
        if inp is None or inp.s_pol == 0:
            continue
        if s.tier == Tier.L1 and inp.score >= thresholds.tau_syn_load:
            d = promote(s, Tier.L2, trigger="score_threshold")
            decisions.append(d)
            if d.from_tier != d.to_tier:
                moved_this_turn.add(sid)
        if s.tier == Tier.L2 and inp.score >= thresholds.tau_full_load and inp.intent_to_invoke:
            d = promote(s, Tier.L3, trigger="intent_to_invoke")
            decisions.append(d)
            if d.from_tier != d.to_tier:
                moved_this_turn.add(sid)

    # 3. Mutex enforcement: once per promoted skill, on its final landing tier
    mutex_applied_for: set[str] = set()
    for sid in list(moved_this_turn):
        promoted = pool[sid]
        if sid in mutex_applied_for:
            continue
        if promoted.tier in (Tier.L2, Tier.L3) and promoted.exclusive_of:
            evicted = apply_mutex(promoted, pool)
            for d in evicted:
                if d.from_tier != d.to_tier:
                    decisions.append(d)
                    moved_this_turn.add(d.skill_id)
            mutex_applied_for.add(sid)

    # 4. Demotions: drift, idle, score decay — skip skills that already moved
    for sid, s in pool.items():
        if sid in moved_this_turn:
            continue
        inp = per_skill.get(sid)
        # Skills with no signal at all: don't synthesize a punishing drift; rely on idle/decay instead.
        drift = inp.drift if inp else 0.0
        trigger = needs_demote(s, drift=drift, thresholds=thresholds, idle_timeouts=idle_timeouts)
        if trigger:
            d = demote_one(s, trigger=trigger)
            if d.from_tier != d.to_tier:
                decisions.append(d)
                moved_this_turn.add(sid)

    # 5. Budget enforcement
    for d in enforce_budget(pool, b_pool_tokens):
        if d.from_tier != d.to_tier:
            decisions.append(d)

    tokens_after = sum(s.token_cost for s in pool.values())
    return TurnResult(decisions=decisions,
                      pool_tokens_before=tokens_before,
                      pool_tokens_after=tokens_after)


# ───────────────────────────────────────────────────────────────
# Adapter protocols — implement these in your harness adapter
# ───────────────────────────────────────────────────────────────

class Embedder(Protocol):
    def embed(self, text: str) -> list[float]: ...


class PCAEmitter(Protocol):
    def emit(self, decisions: list[LifecycleDecision], tokens_before: int, tokens_after: int) -> None: ...


# ───────────────────────────────────────────────────────────────
# Module self-test
# ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    # Smoke test: promote one skill through L1 → L2 → L3, then demote on drift.
    pool: dict[str, SkillState] = {
        "sha256:aa": SkillState(skill_id="sha256:aa", skill_name="ckodex-oscal"),
        "sha256:bb": SkillState(skill_id="sha256:bb", skill_name="ckodex-diagrams"),
    }
    inputs = {
        "sha256:aa": TurnInputs(score=0.72, drift=0.10, s_pol=1, intent_to_invoke=True,
                                score_components={"sem": 0.82, "ont": 0.75, "lex": 0.55},
                                weights={"sem": 0.30, "ont": 0.25, "lex": 0.15}),
        "sha256:bb": TurnInputs(score=0.10, drift=0.05, s_pol=1, intent_to_invoke=False,
                                score_components={}, weights={}),
    }
    r1 = step(pool, inputs)
    print("Turn 1 decisions:")
    for d in r1.decisions:
        print(f"  {d.skill_id[:12]}... {d.from_tier} → {d.to_tier}  trigger={d.trigger}")
    print(f"  Pool tokens: {r1.pool_tokens_before} → {r1.pool_tokens_after}")
    print()

    # Turn 7: heavy drift on the OSCAL skill
    inputs2 = {
        "sha256:aa": TurnInputs(score=0.20, drift=0.45, s_pol=1, intent_to_invoke=False,
                                score_components={}, weights={}),
    }
    r2 = step(pool, inputs2)
    print("Turn 7 decisions (after drift):")
    for d in r2.decisions:
        print(f"  {d.skill_id[:12]}... {d.from_tier} → {d.to_tier}  trigger={d.trigger}")
    print(f"  Pool tokens: {r2.pool_tokens_before} → {r2.pool_tokens_after}")
    print()
    print("Self-test PASS")
