"""
ckx_skill_lifecycle reference implementation — RFC-001 v0.2.0.

  kernel.py   — FSM + thresholds + per-turn step() loop (pure)
  scoring.py  — multi-signal scoring functions (pure)

This package is the harness-neutral core. Adapter modules (codex,
claude-code, microsoft-agent-framework, generic-fs) wire embedders,
registries, and PCA emitters around these primitives.
"""
from .kernel import (
    Tier, SkillState, Thresholds, IdleTimeouts,
    LifecycleDecision, TurnInputs, TurnResult,
    promote, demote_one, apply_mutex, enforce_budget, step,
    Embedder, PCAEmitter,
)
from .scoring import (
    SignalWeights, SkillSignals,
    compute_score, lexical_score, cosine,
    ontology_score, recency_decay, user_signal, policy_gate,
)

__all__ = [
    # kernel
    "Tier", "SkillState", "Thresholds", "IdleTimeouts",
    "LifecycleDecision", "TurnInputs", "TurnResult",
    "promote", "demote_one", "apply_mutex", "enforce_budget", "step",
    "Embedder", "PCAEmitter",
    # scoring
    "SignalWeights", "SkillSignals",
    "compute_score", "lexical_score", "cosine",
    "ontology_score", "recency_decay", "user_signal", "policy_gate",
]

__version__ = "0.2.0"
