#!/usr/bin/env python3
"""
conformance/run.py — exercises all 10 STX v1 vectors against the reference
publisher + consumer modules. Exit 0 if every vector passes.

Usage:
  python3 conformance/run.py
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

PACK_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PACK_ROOT))

from reference.publisher import (
    Publisher, sha256_hex, DEFAULT_PRIVACY,
)
from reference.consumer import (
    validate_query_response, redact_for_mode,
)


VECTORS = json.loads((PACK_ROOT / "conformance" / "vectors.json").read_text())


# ───────────────────────────────────────────────────────────────
# P1 — capability handshake defaults
# ───────────────────────────────────────────────────────────────

def run_p1(vec) -> tuple[bool, str]:
    p = Publisher(harness_id="urn:ckodex:harness:p1",
                  harness_version="1.0.0", framework_version="v16.0")
    cap = p.capability
    if cap.get("stxVersion") != "v1": return False, "stxVersion != v1"
    if cap.get("defaultPrivacyMode") != "audit-private": return False, "defaultPrivacyMode != audit-private"
    if "audit-private" not in cap.get("privacyModesSupported", []): return False, "audit-private not in supported"
    if not (set(cap.get("transports", [])) & {"rest", "grpc", "graphql"}):
        return False, "no normative transport advertised"
    return True, "handshake defaults correct"


# ───────────────────────────────────────────────────────────────
# P2 — session lifecycle + skill promotion
# ───────────────────────────────────────────────────────────────

def run_p2(vec) -> tuple[bool, str]:
    p = Publisher(harness_id="urn:ckodex:harness:p2",
                  harness_version="1.0.0", framework_version="v16.0")
    s = p.open_session("urn:ckodex:session:p2", gal=3, privacy_mode="audit-private")
    rec = p.register_skill(s.id, sha256_hex("ckodex-oscal"), "ckodex-oscal")
    p.transition(s.id, rec.id, to_tier="L2", trigger="score_threshold", score=0.55)
    p.transition(s.id, rec.id, to_tier="L3", trigger="intent_to_invoke", score=0.72)

    if rec.tier != "L3":
        return False, f"final tier {rec.tier} != L3"
    if len(s.events) < 6:
        return False, f"only {len(s.events)} events (expected >= 6: open + register + 2x[evidence,transition])"
    # Normative ordering: at GAL >= 3, evidence_emitted precedes its tier_transition
    # so the transition's evidenceBundleId never references an unseen bundle.
    expected_order = ["session_open", "skill_registered",
                       "evidence_emitted", "tier_transition",
                       "evidence_emitted", "tier_transition"]
    actual = [e["eventType"] for e in s.events[:6]]
    if actual != expected_order:
        return False, f"event order {actual} != {expected_order}"
    seqs = [e["sequence"] for e in s.events]
    if seqs != sorted(seqs) or len(set(seqs)) != len(seqs):
        return False, f"non-monotonic sequence {seqs}"
    return True, f"6-event opening (evidence-before-transition), final L3, monotonic"


# ───────────────────────────────────────────────────────────────
# P3 — GAL evidence invariant
# ───────────────────────────────────────────────────────────────

def run_p3(vec) -> tuple[bool, str]:
    p = Publisher(harness_id="urn:ckodex:harness:p3",
                  harness_version="1.0.0", framework_version="v16.0")
    s = p.open_session("urn:ckodex:session:p3", gal=3, privacy_mode="audit-private")
    rec = p.register_skill(s.id, sha256_hex("ckodex-oscal"), "ckodex-oscal")
    tt = p.transition(s.id, rec.id, to_tier="L2", trigger="score_threshold", score=0.55)
    if tt.evidence_bundle_id is None:
        return False, "GAL=3 transition missing evidenceBundleId"
    ev = s.evidence[tt.evidence_bundle_id]
    if ev.predicate != "ckodex/skill-lifecycle@v1":
        return False, f"predicate {ev.predicate!r}"
    if not ev.digest.startswith("sha256:") or len(ev.digest) != 71:
        return False, f"digest shape {ev.digest!r}"
    return True, "evidence bundle emitted with correct predicate + digest"


# ───────────────────────────────────────────────────────────────
# P4 — public-anchor name redaction
# ───────────────────────────────────────────────────────────────

def run_p4(vec) -> tuple[bool, str]:
    p = Publisher(harness_id="urn:ckodex:harness:p4",
                  harness_version="1.0.0", framework_version="v16.0")
    s = p.open_session("urn:ckodex:session:p4", gal=2, privacy_mode="public-anchor")
    rec = p.register_skill(s.id, sha256_hex("secret-skill"), "secret-skill")
    resp = p.query_session(s.id, privacy_mode="public-anchor")
    node = resp["result"]["nodes"][0]["node"]
    if "skillName" in node:
        return False, "skillName not redacted under public-anchor"
    if "skillName" not in resp.get("redacted", []):
        return False, "redacted list missing skillName"
    if "score" in node or "tokenCost" in node:
        return False, "score/tokenCost not redacted under public-anchor"
    return True, "skillName + score + tokenCost redacted"


# ───────────────────────────────────────────────────────────────
# P5 — monotonic sequence under load
# ───────────────────────────────────────────────────────────────

def run_p5(vec) -> tuple[bool, str]:
    p = Publisher(harness_id="urn:ckodex:harness:p5",
                  harness_version="1.0.0", framework_version="v16.0")
    s = p.open_session("urn:ckodex:session:p5", gal=2, privacy_mode="audit-private")
    for i in range(5):
        rec = p.register_skill(s.id, sha256_hex(f"skill-{i}"), f"skill-{i}")
        p.transition(s.id, rec.id, to_tier="L2", trigger="score_threshold", score=0.5)
    seqs = [e["sequence"] for e in s.events]
    if seqs != sorted(seqs):
        return False, "sequence not strictly increasing"
    if len(set(seqs)) != len(seqs):
        return False, "duplicate sequence numbers"
    return True, f"{len(seqs)} events, strictly monotonic"


# ───────────────────────────────────────────────────────────────
# C1 — envelope validation
# ───────────────────────────────────────────────────────────────

def run_c1(vec) -> tuple[bool, str]:
    valid = vec["input"]["valid_envelope"]
    invalid = vec["input"]["invalid_envelope"]
    r1 = validate_query_response(valid)
    if not r1.ok:
        return False, f"valid envelope rejected: {r1.errors}"
    r2 = validate_query_response(invalid)
    if r2.ok:
        return False, "invalid envelope accepted"
    if not any("queryId" in e for e in r2.errors):
        return False, f"queryId not flagged: {r2.errors}"
    return True, "valid passes, invalid (missing queryId) fails"


# ───────────────────────────────────────────────────────────────
# C2 — node type discriminator
# ───────────────────────────────────────────────────────────────

def run_c2(vec) -> tuple[bool, str]:
    resp = {
        "apiVersion": "ckodex.org/stx/v1", "kind": "StxQueryResponse",
        "queryId": "q-c2", "respondedAt": "2026-05-25T00:00:00Z",
        "result": {"nodes": [vec["input"]["node"]]},
    }
    r = validate_query_response(resp)
    if r.ok:
        return False, "MalformedTypeXYZ accepted"
    if not any("@type" in e for e in r.errors):
        return False, f"@type not flagged: {r.errors}"
    return True, "unknown @type rejected"


# ───────────────────────────────────────────────────────────────
# C3 — privacy-mode invariant
# ───────────────────────────────────────────────────────────────

def run_c3(vec) -> tuple[bool, str]:
    resp = {
        "apiVersion": "ckodex.org/stx/v1", "kind": "StxQueryResponse",
        "queryId": "q-c3", "respondedAt": "2026-05-25T00:00:00Z",
        "privacyMode": vec["input"]["privacy_mode"],
        "result": {"nodes": [vec["input"]["node"]]},
    }
    r = validate_query_response(resp)
    if r.ok:
        return False, "public-anchor leak accepted"
    if not any("privacy violation" in e for e in r.errors):
        return False, f"violation not labeled: {r.errors}"
    return True, "public-anchor + skillName flagged as privacy violation"


# ───────────────────────────────────────────────────────────────
# C4 — canonical skill ID
# ───────────────────────────────────────────────────────────────

def run_c4(vec) -> tuple[bool, str]:
    resp = {
        "apiVersion": "ckodex.org/stx/v1", "kind": "StxQueryResponse",
        "queryId": "q-c4", "respondedAt": "2026-05-25T00:00:00Z",
        "result": {"nodes": [vec["input"]["node"]]},
    }
    r = validate_query_response(resp)
    if r.ok:
        return False, "non-sha256 skillId accepted"
    if not any("skillId" in e for e in r.errors):
        return False, f"skillId not flagged: {r.errors}"
    return True, "non-canonical skillId rejected"


# ───────────────────────────────────────────────────────────────
# C5 — score range
# ───────────────────────────────────────────────────────────────

def run_c5(vec) -> tuple[bool, str]:
    resp = {
        "apiVersion": "ckodex.org/stx/v1", "kind": "StxQueryResponse",
        "queryId": "q-c5", "respondedAt": "2026-05-25T00:00:00Z",
        "result": {"nodes": [vec["input"]["node"]]},
    }
    r = validate_query_response(resp)
    if r.ok:
        return False, "score=1.5 accepted"
    if not any("score" in e for e in r.errors):
        return False, f"score not flagged: {r.errors}"
    return True, "out-of-range score rejected"


# ───────────────────────────────────────────────────────────────
# Main
# ───────────────────────────────────────────────────────────────

RUNNERS = {
    "P1": run_p1, "P2": run_p2, "P3": run_p3, "P4": run_p4, "P5": run_p5,
    "C1": run_c1, "C2": run_c2, "C3": run_c3, "C4": run_c4, "C5": run_c5,
}


def main() -> int:
    passed = 0
    failed = 0
    for vec in VECTORS["vectors"]:
        vid = vec["id"]
        print(f"--- {vid} [{vec['role']}] {vec['name']} ---")
        runner = RUNNERS.get(vid)
        if runner is None:
            print(f"  SKIP no runner for {vid}")
            continue
        try:
            ok, msg = runner(vec)
        except Exception as e:
            ok, msg = False, f"exception: {e!r}"
        if ok:
            print(f"  PASS  {msg}")
            passed += 1
        else:
            print(f"  FAIL  {msg}")
            failed += 1
        print()
    total = passed + failed
    print(f"=== Conformance: {passed}/{total} PASS, {failed} FAIL ===")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
