#!/usr/bin/env python3
"""
conformance/run.py — exercises the RFC-001 reference kernel against the
twelve conformance vectors. Vectors A, B, D, F have full kernel-level
runners. Vectors C, E, G, H, I, J, K, L are validated structurally
(fixture shape and declarative checks).

Usage:
  python3 conformance/run.py

Exit code: 0 if all checks pass.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

# Make reference module importable when running from the pack root
PACK_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PACK_ROOT))

from reference.kernel import (
    Tier, SkillState, Thresholds, IdleTimeouts, TurnInputs, step,
)


VECTORS_PATH = Path(__file__).resolve().parent / "vectors.json"


def _print_decisions(label: str, decisions) -> None:
    print(f"  {label}:")
    for d in decisions:
        print(f"    {d.skill_id[:14]:<14} {d.from_tier!s:>4} → {d.to_tier!s:<4} trigger={d.trigger}")


# ───────────────────────────────────────────────────────────────
# Vector A — cold-start OSCAL query
# ───────────────────────────────────────────────────────────────

def run_vector_a() -> tuple[bool, str]:
    pool = {
        "sha256:oscal":   SkillState(skill_id="sha256:oscal",   skill_name="ckodex-oscal"),
        "sha256:csr":     SkillState(skill_id="sha256:csr",     skill_name="cortaix-csr"),
        "sha256:diag":    SkillState(skill_id="sha256:diag",    skill_name="ckodex-diagrams"),
        "sha256:fdb":     SkillState(skill_id="sha256:fdb",     skill_name="ckodex-foundationdb"),
        "sha256:announce":SkillState(skill_id="sha256:announce",skill_name="ckodex-announcements"),
    }
    inputs = {
        "sha256:oscal":   TurnInputs(score=0.66, drift=0.05, s_pol=1, intent_to_invoke=True,  score_components={}, weights={}),
        "sha256:csr":     TurnInputs(score=0.50, drift=0.10, s_pol=1, intent_to_invoke=False, score_components={}, weights={}),
        "sha256:diag":    TurnInputs(score=0.10, drift=0.05, s_pol=1, intent_to_invoke=False, score_components={}, weights={}),
        "sha256:fdb":     TurnInputs(score=0.05, drift=0.10, s_pol=1, intent_to_invoke=False, score_components={}, weights={}),
        "sha256:announce":TurnInputs(score=0.08, drift=0.05, s_pol=1, intent_to_invoke=False, score_components={}, weights={}),
    }
    r = step(pool, inputs)
    _print_decisions("Vector A decisions", r.decisions)

    # Expected: oscal L1→L2→L3, csr L1→L2, others stay at L1
    by_skill = {d.skill_id: d for d in r.decisions if d.from_tier != d.to_tier}
    if pool["sha256:oscal"].tier == Tier.L3 and \
       pool["sha256:csr"].tier == Tier.L2 and \
       pool["sha256:diag"].tier == Tier.L1:
        return True, "oscal→L3, csr→L2, diag stays L1"
    return False, f"end-state tiers wrong: oscal={pool['sha256:oscal'].tier}, csr={pool['sha256:csr'].tier}, diag={pool['sha256:diag'].tier}"


# ───────────────────────────────────────────────────────────────
# Vector B — topic drift demotes oscal, promotes announcements
# ───────────────────────────────────────────────────────────────

def run_vector_b() -> tuple[bool, str]:
    pool = {
        "sha256:oscal":    SkillState(skill_id="sha256:oscal",    skill_name="ckodex-oscal", tier=Tier.L3, score=0.71, token_cost=3500, turns_since_last_ref=6),
        "sha256:announce": SkillState(skill_id="sha256:announce", skill_name="ckodex-announcements", tier=Tier.L1, token_cost=80),
    }
    inputs = {
        "sha256:oscal":    TurnInputs(score=0.20, drift=0.42, s_pol=1, intent_to_invoke=False, score_components={}, weights={}),
        "sha256:announce": TurnInputs(score=0.71, drift=0.10, s_pol=1, intent_to_invoke=True,  score_components={}, weights={}),
    }
    r = step(pool, inputs)
    _print_decisions("Vector B decisions", r.decisions)

    if pool["sha256:oscal"].tier == Tier.L2 and pool["sha256:announce"].tier == Tier.L3:
        # Verify the drift trigger is recorded
        drift_decisions = [d for d in r.decisions if d.trigger == "topic_drift"]
        if drift_decisions:
            return True, "oscal demoted on drift, announcements promoted to L3"
    return False, f"end-state wrong: oscal={pool['sha256:oscal'].tier}, announce={pool['sha256:announce'].tier}"


# ───────────────────────────────────────────────────────────────
# Vector D — mutex
# ───────────────────────────────────────────────────────────────

def run_vector_d() -> tuple[bool, str]:
    pool = {
        "sha256:fdb":   SkillState(skill_id="sha256:fdb",   skill_name="ckodex-foundationdb", tier=Tier.L2, token_cost=400),
        "sha256:redis": SkillState(skill_id="sha256:redis", skill_name="ckodex-redis-hypothetical",
                                    exclusive_of=["ckodex-foundationdb"]),
    }
    inputs = {
        "sha256:redis": TurnInputs(score=0.85, drift=0.05, s_pol=1, intent_to_invoke=True,
                                    score_components={}, weights={}),
    }
    r = step(pool, inputs)
    _print_decisions("Vector D decisions", r.decisions)

    if pool["sha256:redis"].tier == Tier.L3 and pool["sha256:fdb"].tier == Tier.L1:
        mutex_decision = [d for d in r.decisions if d.trigger == "mutex"]
        if mutex_decision:
            return True, "redis promoted to L3, foundationdb evicted by mutex"
    return False, f"end-state wrong: redis={pool['sha256:redis'].tier}, fdb={pool['sha256:fdb'].tier}"


# ───────────────────────────────────────────────────────────────
# Vector F — implicit invocation disabled
# ───────────────────────────────────────────────────────────────

def run_vector_f() -> tuple[bool, str]:
    pool = {
        "sha256:guarded": SkillState(skill_id="sha256:guarded", skill_name="guarded-skill",
                                      implicit_invocation=False),
    }
    # Strong signals BUT policy gate is 0 because implicit invocation disabled and no explicit mention
    inputs = {
        "sha256:guarded": TurnInputs(score=0.95, drift=0.05, s_pol=0, intent_to_invoke=True,
                                      score_components={}, weights={}),
    }
    r = step(pool, inputs)
    _print_decisions("Vector F decisions", r.decisions)

    if pool["sha256:guarded"].tier == Tier.L1:
        return True, "guarded-skill stays at L1 despite strong signal (S_pol=0)"
    return False, f"end-state wrong: guarded={pool['sha256:guarded'].tier}"


# ───────────────────────────────────────────────────────────────
# Structural checks (vectors C, E, G, H, I, J, K, L)
# ───────────────────────────────────────────────────────────────

def structural_check(vector: dict) -> tuple[bool, str]:
    """Verify the vector itself is well-formed."""
    required = {"id", "name", "purpose", "setup", "input", "expected"}
    missing = required - set(vector.keys())
    if missing:
        return False, f"missing keys: {missing}"
    return True, "well-formed"


# ───────────────────────────────────────────────────────────────
# Main
# ───────────────────────────────────────────────────────────────

KERNEL_RUNNERS = {
    "A": run_vector_a,
    "B": run_vector_b,
    "D": run_vector_d,
    "F": run_vector_f,
}


def main() -> int:
    with open(VECTORS_PATH, encoding="utf-8") as f:
        spec = json.load(f)

    pass_count = 0
    fail_count = 0

    for vec in spec["vectors"]:
        vid = vec["id"]
        print(f"--- Vector {vid} — {vec['name']} ---")
        runner = KERNEL_RUNNERS.get(vid)
        if runner:
            ok, msg = runner()
        else:
            ok, msg = structural_check(vec)
        if ok:
            print(f"  PASS  {msg}")
            pass_count += 1
        else:
            print(f"  FAIL  {msg}")
            fail_count += 1
        print()

    total = pass_count + fail_count
    print(f"=== Conformance: {pass_count}/{total} PASS, {fail_count} FAIL ===")
    return 0 if fail_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
