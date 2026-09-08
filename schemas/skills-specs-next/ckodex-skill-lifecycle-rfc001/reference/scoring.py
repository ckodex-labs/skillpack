"""
ckx_skill_scoring — RFC-001 §4 multi-signal scoring, normalized to [0,1].

Pure functions. No I/O. Plug your own embedding model and registry.

Signals:
  S_lex   BM25-style lexical match of triggers/keywords against recent turns
  S_sem   cosine(embed(ctx), embed(description ⊕ synopsis))
  S_ont   structural Jaccard over ontology subjects + 0.5·related, clamped /1.5
  S_co    co-occurrence prior P(skill | currently_loaded)
  S_rec   exp(-λ · turns_since_last_reference)
  S_usr   graded {0, 0.6, 0.8, 1.0} — explicit / pinned / memory / none
  S_pol   hard gate ∈ {0, 1}

Composite is policy-dominated: S_pol = 0 zeros the score regardless of all
other signals. Composite ∈ [0,1] always.
"""
from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Iterable


# ───────────────────────────────────────────────────────────────
# Default weights (RFC §4.2)
# ───────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class SignalWeights:
    lex: float = 0.15
    sem: float = 0.30
    ont: float = 0.25
    co:  float = 0.10
    rec: float = 0.10
    usr: float = 0.10

    def __post_init__(self):
        s = self.lex + self.sem + self.ont + self.co + self.rec + self.usr
        assert abs(s - 1.0) < 1e-9, f"weights must sum to 1.0, got {s}"


# ───────────────────────────────────────────────────────────────
# Signal sample
# ───────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class SkillSignals:
    s_lex: float       # ∈ [0,1]
    s_sem: float       # ∈ [0,1]
    s_ont: float       # ∈ [0,1]  (already normalized by ontology_score)
    s_co:  float       # ∈ [0,1]
    s_rec: float       # ∈ [0,1]
    s_usr: float       # ∈ {0.0, 0.6, 0.8, 1.0}
    s_pol: int         # ∈ {0, 1}

    def __post_init__(self):
        for name in ("s_lex", "s_sem", "s_ont", "s_co", "s_rec", "s_usr"):
            v = getattr(self, name)
            assert 0.0 <= v <= 1.0, f"{name} out of range: {v}"
        assert self.s_pol in (0, 1)


# ───────────────────────────────────────────────────────────────
# Composite score (RFC §4.1)
# ───────────────────────────────────────────────────────────────

def compute_score(sig: SkillSignals, w: SignalWeights = SignalWeights()) -> float:
    if sig.s_pol == 0:
        return 0.0
    score = (
        w.lex * sig.s_lex +
        w.sem * sig.s_sem +
        w.ont * sig.s_ont +
        w.co  * sig.s_co  +
        w.rec * sig.s_rec +
        w.usr * sig.s_usr
    )
    # The weighted sum is bounded in [0,1] by construction (each component
    # in [0,1], weights sum to 1.0). The clamp here is belt-and-suspenders.
    return max(0.0, min(1.0, score))


# ───────────────────────────────────────────────────────────────
# Individual signal helpers
# ───────────────────────────────────────────────────────────────

def lexical_score(triggers: Iterable[str], recent_turns: str) -> float:
    """A tiny BM25-ish scorer for stdlib-only use. Replace with a real BM25
    implementation (e.g. rank_bm25) in production."""
    haystack = recent_turns.lower()
    triggers = [t.lower() for t in triggers if t]
    if not triggers:
        return 0.0
    hits = sum(1 for t in triggers if t in haystack)
    # Saturating: hitting half the triggers gives ~0.5, all gives ~1.0
    return min(1.0, hits / max(1, len(triggers)))


def cosine(a: list[float], b: list[float]) -> float:
    """Cosine similarity, clamped to [0,1] (treat negatives as 0 for routing)."""
    if not a or not b or len(a) != len(b):
        return 0.0
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(y * y for y in b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return max(0.0, min(1.0, dot / (na * nb)))


def ontology_score(
    ctx_subjects: set[str],
    skill_subjects: set[str],
    related_subjects: set[str] = frozenset(),
) -> float:
    """RFC §4.1 ontology score: jaccard(ctx, subjects) + 0.5·jaccard(ctx, related),
    normalized to [0,1] via /1.5 clamp."""
    def jaccard(a: set, b: set) -> float:
        if not a or not b:
            return 0.0
        inter = len(a & b)
        union = len(a | b)
        return inter / union if union else 0.0
    raw = jaccard(ctx_subjects, skill_subjects) + 0.5 * jaccard(ctx_subjects, related_subjects)
    return min(1.0, raw / 1.5)


def recency_decay(turns_since_ref: int, lam: float = 0.15) -> float:
    """Exponential decay. λ=0.15 ⇒ half-life ~4.6 turns."""
    if turns_since_ref < 0:
        return 1.0
    return math.exp(-lam * turns_since_ref)


def user_signal(
    *,
    explicit_mention: bool = False,
    pinned: bool = False,
    memory_preference: bool = False,
) -> float:
    """Graded user signal per RFC §4.3."""
    if explicit_mention:    return 1.0
    if pinned:              return 0.8
    if memory_preference:   return 0.6
    return 0.0


def policy_gate(
    *,
    skill_gal_minimum: int,
    current_gal: int,
    skill_scope: set[tuple[str, str]],
    current_scope: set[tuple[str, str]],
    skill_implicit_invocation: bool,
    explicit_mention: bool,
    framework_compatible: bool,
) -> int:
    """RFC §4.4 hard policy gate. Returns 1 (admit) or 0 (deny)."""
    if skill_gal_minimum > current_gal:
        return 0
    if not framework_compatible:
        return 0
    if not skill_scope.issubset(current_scope):
        return 0
    if not skill_implicit_invocation and not explicit_mention:
        return 0
    return 1


# ───────────────────────────────────────────────────────────────
# Module self-test
# ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    # Replicates RFC §13.1 Vector A — Cold Start, OSCAL Query.
    # Signals chosen so the composite exceeds τ_full_load=0.65, justifying
    # L1→L2→L3 promotion when paired with intent_to_invoke=True.
    sig_oscal = SkillSignals(s_lex=0.55, s_sem=0.82, s_ont=0.75,
                              s_co=0.10, s_rec=0.95, s_usr=0.60, s_pol=1)
    score = compute_score(sig_oscal)
    print(f"ckodex-oscal score = {score:.3f}  (expected ≥ τ_full_load=0.65)")
    assert score >= 0.65, f"Vector A expected promotion to L3, got {score:.3f}"

    sig_diag = SkillSignals(s_lex=0.05, s_sem=0.18, s_ont=0.10,
                            s_co=0.10, s_rec=0.0, s_usr=0.0, s_pol=1)
    score_d = compute_score(sig_diag)
    print(f"ckodex-diagrams score = {score_d:.3f} (expected below τ_floor=0.20)")
    assert score_d < 0.20

    # Policy gate denies regardless of strong signals
    sig_pol = SkillSignals(s_lex=1.0, s_sem=1.0, s_ont=1.0,
                           s_co=1.0, s_rec=1.0, s_usr=1.0, s_pol=0)
    assert compute_score(sig_pol) == 0.0

    # Ontology score normalization
    ctx     = {"oscal", "nist:800-53", "compliance"}
    subj    = {"oscal", "nist:800-53", "fedramp"}
    related = {"cortaix-csr"}
    s_ont = ontology_score(ctx, subj, related)
    print(f"S_ont(ctx, subjects={subj}, related={related}) = {s_ont:.3f}")
    assert 0.0 <= s_ont <= 1.0

    # User signal grading
    assert user_signal(explicit_mention=True) == 1.0
    assert user_signal(pinned=True) == 0.8
    assert user_signal(memory_preference=True) == 0.6
    assert user_signal() == 0.0

    print()
    print("Self-test PASS")
