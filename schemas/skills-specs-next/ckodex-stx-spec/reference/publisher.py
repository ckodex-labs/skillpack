"""
STX publisher reference implementation. Pure stdlib.

Models what a harness exposes:
  - capability descriptor (handshake)
  - in-memory graph: Harness ⊃ Sessions ⊃ SkillStates ⊃ TierTransitions ⊃ EvidenceBundles
  - per-session monotonic event log
  - privacy-mode-aware redaction at the boundary

This is a contract demonstrator, not a production server. Wire to your
gRPC/FastAPI/Strawberry layer of choice; the data shape is normative.
"""
from __future__ import annotations

import json
import hashlib
import re
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from itertools import count
from typing import Any, Iterable

# ───────────────────────────────────────────────────────────────
# Constants — must stay aligned with stx-domain.v1.schema.json
# ───────────────────────────────────────────────────────────────

API_VERSION    = "ckodex.org/stx/v1"
PREDICATE      = "ckodex/skill-lifecycle@v1"
POLICY_VERSION = "ckodex-skill-lifecycle/v0.2"

VALID_TIERS   = {"L0", "L1", "L2", "L3", "L4R", "L4X"}
VALID_PRIVACY = {"debug-local", "audit-private", "public-anchor", "regulated-export"}
DEFAULT_PRIVACY = "audit-private"

# Publish-allowlist for human skill names under public-anchor.
PUBLIC_NAME_ALLOWLIST: set[str] = set()


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z")


def sha256_hex(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


# ───────────────────────────────────────────────────────────────
# Records
# ───────────────────────────────────────────────────────────────

@dataclass
class SkillStateRecord:
    id: str                       # urn:ckodex:state:<uuid>
    session_id: str
    skill_id: str                 # canonical sha256:<hex>
    skill_name: str
    skill_version: str = ""
    tier: str = "L1"
    score: float = 0.0
    turns_since_last_reference: int = 0
    token_cost: int = 80
    locked: bool = False
    transition_ids: list[str] = field(default_factory=list)


@dataclass
class TierTransitionRecord:
    id: str
    skill_state_id: str
    timestamp: str
    from_tier: str
    to_tier: str
    trigger: str
    score: float | None = None
    evidence_bundle_id: str | None = None


@dataclass
class EvidenceBundleRecord:
    id: str
    subject: str
    timestamp: str
    privacy_mode: str
    digest: str
    rekor_anchor: str | None = None
    transition_ids: list[str] = field(default_factory=list)
    attestation_chain_id: str | None = None
    predicate: str = PREDICATE
    policy_version: str = POLICY_VERSION


@dataclass
class SessionState:
    id: str                       # urn:ckodex:session:<uuid>
    harness_id: str
    started_at: str
    current_gal: int = 2
    privacy_mode: str = DEFAULT_PRIVACY
    skill_pool_tokens_budget: int = 8192
    skill_pool_tokens_current: int = 0
    ended_at: str | None = None
    skill_states: dict[str, SkillStateRecord] = field(default_factory=dict)
    transitions: dict[str, TierTransitionRecord] = field(default_factory=dict)
    evidence: dict[str, EvidenceBundleRecord] = field(default_factory=dict)
    _event_counter: count = field(default_factory=lambda: count())
    events: list[dict] = field(default_factory=list)


# ───────────────────────────────────────────────────────────────
# Capability handshake
# ───────────────────────────────────────────────────────────────

def build_capability_descriptor(
    *,
    harness_id: str,
    harness_version: str,
    framework_version: str,
    transports: Iterable[str] = ("rest", "grpc", "graphql"),
    privacy_modes: Iterable[str] = ("debug-local", "audit-private", "public-anchor", "regulated-export"),
    max_events_per_second: int = 100,
    max_query_depth: int = 6,
) -> dict:
    """Construct the CapabilityDescriptor. Validates the default-privacy invariant."""
    transports = list(transports)
    privacy_modes = list(privacy_modes)
    assert transports, "publisher must advertise at least one transport"
    assert DEFAULT_PRIVACY in privacy_modes, "audit-private MUST be in privacyModesSupported"
    return {
        "stxVersion": "v1",
        "transports": transports,
        "privacyModesSupported": privacy_modes,
        "defaultPrivacyMode": DEFAULT_PRIVACY,
        "maxEventsPerSecond": max_events_per_second,
        "maxQueryDepth": max_query_depth,
        "harnessId": harness_id,
        "harnessVersion": harness_version,
        "frameworkVersion": framework_version,
    }


# ───────────────────────────────────────────────────────────────
# Privacy-mode redaction
# ───────────────────────────────────────────────────────────────

def _redact_skill_state(state: dict, mode: str) -> tuple[dict, list[str]]:
    """Apply privacy-mode redaction to a SkillState dict. Returns (redacted_dict, redacted_paths)."""
    redacted_paths: list[str] = []
    out = dict(state)
    if mode == "debug-local":
        return out, redacted_paths
    if mode == "audit-private":
        # Strip token_cost / locked from external view; keep name (privileged consumers).
        if "tokenCost" in out:
            del out["tokenCost"]
            redacted_paths.append("tokenCost")
        return out, redacted_paths
    if mode == "public-anchor":
        # Hide human name unless on the allowlist; hide score and turn counts.
        if "skillName" in out and out["skillName"] not in PUBLIC_NAME_ALLOWLIST:
            del out["skillName"]
            redacted_paths.append("skillName")
        for f in ("score", "turnsSinceLastReference", "tokenCost", "locked"):
            if f in out:
                del out[f]
                redacted_paths.append(f)
        return out, redacted_paths
    if mode == "regulated-export":
        # Full visibility; redaction is the signing layer's problem.
        return out, redacted_paths
    raise ValueError(f"unknown privacy mode: {mode}")


# ───────────────────────────────────────────────────────────────
# Publisher
# ───────────────────────────────────────────────────────────────

class Publisher:
    """The harness-side STX state container + emitter."""

    def __init__(self, *, harness_id: str, harness_version: str, framework_version: str):
        self.harness_id = harness_id
        self.harness_version = harness_version
        self.framework_version = framework_version
        self.sessions: dict[str, SessionState] = {}
        self.capability = build_capability_descriptor(
            harness_id=harness_id,
            harness_version=harness_version,
            framework_version=framework_version,
        )

    # ─── Session lifecycle ───────────────────────────────────

    def open_session(self, session_id: str, *, gal: int = 2, privacy_mode: str = DEFAULT_PRIVACY) -> SessionState:
        if privacy_mode not in VALID_PRIVACY:
            raise ValueError(f"invalid privacy_mode: {privacy_mode}")
        s = SessionState(id=session_id, harness_id=self.harness_id,
                          started_at=now_iso(), current_gal=gal,
                          privacy_mode=privacy_mode)
        self.sessions[session_id] = s
        self._emit(s, "session_open", {"sessionId": session_id, "currentGal": gal,
                                        "privacyMode": privacy_mode})
        return s

    def close_session(self, session_id: str) -> None:
        s = self.sessions[session_id]
        s.ended_at = now_iso()
        self._emit(s, "session_close", {"sessionId": session_id, "endedAt": s.ended_at})

    # ─── Skill state + transitions ───────────────────────────

    def register_skill(self, session_id: str, skill_id: str, skill_name: str,
                       *, state_id: str | None = None,
                       skill_version: str = "") -> SkillStateRecord:
        s = self.sessions[session_id]
        state_id = state_id or f"urn:ckodex:state:{skill_id[7:19]}"
        rec = SkillStateRecord(id=state_id, session_id=session_id,
                                skill_id=skill_id, skill_name=skill_name,
                                skill_version=skill_version, tier="L1")
        s.skill_states[state_id] = rec
        s.skill_pool_tokens_current += rec.token_cost
        self._emit(s, "skill_registered", {"skillStateId": state_id, "skillId": skill_id,
                                            "tier": "L1"})
        return rec

    def transition(self, session_id: str, state_id: str, *,
                   to_tier: str, trigger: str, score: float | None = None,
                   emit_evidence: bool = True) -> TierTransitionRecord:
        if to_tier not in VALID_TIERS:
            raise ValueError(f"invalid tier: {to_tier}")
        s = self.sessions[session_id]
        sk = s.skill_states[state_id]
        from_tier = sk.tier
        sk.tier = to_tier
        new_cost = {"L0": 10, "L1": 80, "L2": 400, "L3": 3500, "L4R": 1500, "L4X": 0}[to_tier]
        s.skill_pool_tokens_current += (new_cost - sk.token_cost)
        sk.token_cost = new_cost

        tid = f"urn:ckodex:transition:{state_id[-12:]}:{len(sk.transition_ids)}"
        tt = TierTransitionRecord(id=tid, skill_state_id=state_id,
                                   timestamp=now_iso(),
                                   from_tier=from_tier, to_tier=to_tier,
                                   trigger=trigger, score=score)
        sk.transition_ids.append(tid)
        s.transitions[tid] = tt

        # Emit evidence iff GAL ≥ 3 OR privacy_mode != debug-local
        if emit_evidence and (s.current_gal >= 3 or s.privacy_mode != "debug-local"):
            ev = self._emit_evidence_bundle(s, tt)
            tt.evidence_bundle_id = ev.id

        self._emit(s, "tier_transition", {
            "transitionId": tid, "skillStateId": state_id,
            "fromTier": from_tier, "toTier": to_tier,
            "trigger": trigger, "evidenceBundleId": tt.evidence_bundle_id,
        })
        return tt

    def _emit_evidence_bundle(self, s: SessionState, tt: TierTransitionRecord) -> EvidenceBundleRecord:
        bundle_id = f"urn:ckodex:evidence:{tt.id.split(':')[-2]}:{len(s.evidence)}"
        digest = sha256_hex(json.dumps({
            "predicate": PREDICATE, "subject": tt.skill_state_id,
            "from": tt.from_tier, "to": tt.to_tier, "trigger": tt.trigger,
        }, sort_keys=True))
        ev = EvidenceBundleRecord(
            id=bundle_id, subject=tt.skill_state_id, timestamp=tt.timestamp,
            privacy_mode=s.privacy_mode, digest=digest, transition_ids=[tt.id],
        )
        s.evidence[bundle_id] = ev
        self._emit(s, "evidence_emitted", {"evidenceBundleId": bundle_id, "digest": digest})
        return ev

    # ─── Event emission (with monotonic sequence) ────────────

    def _emit(self, s: SessionState, event_type: str, payload: dict) -> dict:
        seq = next(s._event_counter)
        evt = {
            "apiVersion": API_VERSION,
            "kind": "StxEvent",
            "sessionId": s.id,
            "sequence": seq,
            "timestamp": now_iso(),
            "eventType": event_type,
            "payload": payload,
            "privacyMode": s.privacy_mode,
        }
        s.events.append(evt)
        return evt

    # ─── Query surface (consumer-facing) ─────────────────────

    def serialize_skill_state(self, state: SkillStateRecord, privacy_mode: str) -> tuple[dict, list[str]]:
        d = {
            "id": state.id, "sessionId": state.session_id,
            "skillId": state.skill_id, "skillName": state.skill_name,
            "skillVersion": state.skill_version, "tier": state.tier,
            "score": state.score, "turnsSinceLastReference": state.turns_since_last_reference,
            "tokenCost": state.token_cost, "locked": state.locked,
            "transitionIds": list(state.transition_ids),
        }
        return _redact_skill_state(d, privacy_mode)

    def query_session(self, session_id: str, privacy_mode: str = DEFAULT_PRIVACY) -> dict:
        s = self.sessions[session_id]
        nodes = []
        redacted: list[str] = []
        for ss in s.skill_states.values():
            node, rpaths = self.serialize_skill_state(ss, privacy_mode)
            nodes.append({"@type": "SkillState", "node": node})
            redacted.extend(rpaths)
        return {
            "apiVersion": API_VERSION,
            "kind": "StxQueryResponse",
            "queryId": f"q-{session_id[-8:]}",
            "respondedAt": now_iso(),
            "privacyMode": privacy_mode,
            "result": {
                "nodes": nodes,
                "pageInfo": {"hasNext": False, "total": len(nodes)},
            },
            "redacted": sorted(set(redacted)),
        }


# ───────────────────────────────────────────────────────────────
# Module self-test
# ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    p = Publisher(harness_id="urn:ckodex:harness:demo",
                  harness_version="1.0.0",
                  framework_version="v16.0")

    # Capability handshake
    cap = p.capability
    assert cap["stxVersion"] == "v1"
    assert cap["defaultPrivacyMode"] == "audit-private", "DEFAULT_PRIVACY violated"
    assert set(cap["transports"]) >= {"rest", "grpc", "graphql"}, "all three transports required"
    print("OK   capability handshake")

    # Open session, register skill, transition L1→L2→L3
    s = p.open_session("urn:ckodex:session:s1", gal=3, privacy_mode="audit-private")
    rec = p.register_skill(s.id, sha256_hex("ckodex-oscal"), "ckodex-oscal")
    t1 = p.transition(s.id, rec.id, to_tier="L2", trigger="score_threshold", score=0.55)
    t2 = p.transition(s.id, rec.id, to_tier="L3", trigger="intent_to_invoke", score=0.72)

    # Evidence MUST be emitted at GAL=3
    assert t1.evidence_bundle_id is not None, "GAL≥3 missing evidence on t1"
    assert t2.evidence_bundle_id is not None, "GAL≥3 missing evidence on t2"
    print("OK   GAL=3 evidence emission")

    # Monotonic per-session sequence
    sequences = [e["sequence"] for e in s.events]
    assert sequences == sorted(sequences), "events out of order"
    assert len(set(sequences)) == len(sequences), "duplicate sequence"
    print(f"OK   monotonic event sequence ({len(s.events)} events)")

    # Public-anchor redacts human name
    p2 = Publisher(harness_id="urn:ckodex:harness:demo2",
                   harness_version="1.0.0", framework_version="v16.0")
    s2 = p2.open_session("urn:ckodex:session:s2", gal=2, privacy_mode="public-anchor")
    rec2 = p2.register_skill(s2.id, sha256_hex("secret-skill"), "secret-skill-name")
    resp = p2.query_session(s2.id, privacy_mode="public-anchor")
    node = resp["result"]["nodes"][0]["node"]
    assert "skillName" not in node, "public-anchor failed to redact skillName"
    assert "skillName" in resp["redacted"]
    print("OK   public-anchor redaction")

    print()
    print("Self-test PASS")
