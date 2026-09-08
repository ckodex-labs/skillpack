"""
STX consumer reference implementation. Pure stdlib.

Models what a client does after receiving a query response:
  - validate envelope shape (apiVersion, kind, queryId, respondedAt)
  - validate per-node @type discriminator
  - enforce privacy-mode invariants (e.g. public-anchor MUST NOT carry skillName)
  - apply consumer-side redaction if the consumer wants to further restrict view

This module is transport-agnostic. The same validate_query_response()
runs on whatever the REST handler, gRPC unmarshaller, or GraphQL
resolver returned, modulo their native deserialization.
"""
from __future__ import annotations

import re
from dataclasses import dataclass

API_VERSION    = "ckodex.org/stx/v1"
KIND_RESPONSE  = "StxQueryResponse"
KIND_EVENT     = "StxEvent"

VALID_TYPES = {"Harness", "Session", "SkillState", "TierTransition",
                "EvidenceBundle", "AttestationChain", "PolicyGate"}
VALID_PRIVACY = {"debug-local", "audit-private", "public-anchor", "regulated-export"}
VALID_TIERS = {"L0", "L1", "L2", "L3", "L4R", "L4X"}

URN_RE = re.compile(r"^urn:ckodex:[a-z]+:[A-Za-z0-9._:-]+$")
SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")


# ───────────────────────────────────────────────────────────────
# Result class
# ───────────────────────────────────────────────────────────────

@dataclass
class QueryResult:
    ok: bool
    errors: list[str]
    response: dict

    def summary(self) -> str:
        if self.ok:
            n = len(self.response.get("result", {}).get("nodes", []))
            return f"OK · {n} node(s) returned"
        return f"FAIL · {len(self.errors)} error(s): {self.errors[0]}"


# ───────────────────────────────────────────────────────────────
# Validation
# ───────────────────────────────────────────────────────────────

def validate_query_response(resp: dict) -> QueryResult:
    """Validate an StxQueryResponse envelope + nodes against the domain."""
    errs: list[str] = []

    if not isinstance(resp, dict):
        return QueryResult(False, ["response not a JSON object"], {})

    if resp.get("apiVersion") != API_VERSION:
        errs.append(f"apiVersion {resp.get('apiVersion')!r} != {API_VERSION!r}")
    if resp.get("kind") != KIND_RESPONSE:
        errs.append(f"kind {resp.get('kind')!r} != {KIND_RESPONSE!r}")
    if not resp.get("queryId"):
        errs.append("queryId missing")
    if not resp.get("respondedAt"):
        errs.append("respondedAt missing")

    privacy = resp.get("privacyMode")
    if privacy and privacy not in VALID_PRIVACY:
        errs.append(f"unknown privacyMode {privacy!r}")

    result = resp.get("result") or {}
    nodes = result.get("nodes") or []
    if not isinstance(nodes, list):
        errs.append("result.nodes must be a list")
        return QueryResult(False, errs, resp)

    for i, n in enumerate(nodes):
        path = f"result.nodes[{i}]"
        if not isinstance(n, dict):
            errs.append(f"{path}: not an object"); continue
        t = n.get("@type")
        body = n.get("node")
        if t not in VALID_TYPES:
            errs.append(f"{path}.@type {t!r} not in {sorted(VALID_TYPES)}")
            continue
        if not isinstance(body, dict):
            errs.append(f"{path}.node: not an object"); continue
        errs.extend(_validate_node(t, body, path, privacy))

    return QueryResult(not errs, errs, resp)


def _validate_node(t: str, n: dict, path: str, privacy: str | None) -> list[str]:
    errs: list[str] = []

    # Common URN check on .id if present
    if "id" in n and not URN_RE.match(str(n.get("id", ""))):
        errs.append(f"{path}.id not a CKODEX URN: {n['id']!r}")

    if t == "SkillState":
        # tier in enum
        if n.get("tier") not in VALID_TIERS:
            errs.append(f"{path}.tier {n.get('tier')!r} not in {sorted(VALID_TIERS)}")
        # skillId is a canonical sha256
        if n.get("skillId") and not SHA256_RE.match(str(n["skillId"])):
            errs.append(f"{path}.skillId not sha256:<hex>: {n['skillId']!r}")
        # score in [0,1]
        if "score" in n:
            try:
                s = float(n["score"])
                if not (0.0 <= s <= 1.0):
                    errs.append(f"{path}.score {s} not in [0,1]")
            except (TypeError, ValueError):
                errs.append(f"{path}.score not numeric")
        # privacy-mode invariant: public-anchor MUST NOT carry skillName
        if privacy == "public-anchor" and "skillName" in n:
            errs.append(f"{path}.skillName present under public-anchor (privacy violation)")

    elif t == "TierTransition":
        if n.get("fromTier") not in VALID_TIERS:
            errs.append(f"{path}.fromTier {n.get('fromTier')!r} invalid")
        if n.get("toTier") not in VALID_TIERS:
            errs.append(f"{path}.toTier {n.get('toTier')!r} invalid")

    elif t == "EvidenceBundle":
        if n.get("predicate") != "ckodex/skill-lifecycle@v1":
            errs.append(f"{path}.predicate must be 'ckodex/skill-lifecycle@v1'")
        if n.get("digest") and not SHA256_RE.match(str(n["digest"])):
            errs.append(f"{path}.digest not sha256:<hex>")

    return errs


# ───────────────────────────────────────────────────────────────
# Client-side redaction (consumer further restricts what it persists)
# ───────────────────────────────────────────────────────────────

_REDACT_BY_MODE = {
    "audit-private":   ["tokenCost"],
    "public-anchor":   ["skillName", "score", "turnsSinceLastReference", "tokenCost", "locked"],
}


def redact_for_mode(resp: dict, target_mode: str) -> dict:
    """Strip fields the consumer is unwilling to store at target_mode."""
    if target_mode not in VALID_PRIVACY:
        raise ValueError(f"unknown mode: {target_mode}")
    out = dict(resp)
    fields_to_strip = _REDACT_BY_MODE.get(target_mode, [])
    if not fields_to_strip:
        return out
    result = dict(out.get("result", {}))
    new_nodes = []
    stripped: list[str] = []
    for n in result.get("nodes", []):
        nb = dict(n.get("node", {}))
        for f in fields_to_strip:
            if f in nb:
                del nb[f]
                stripped.append(f)
        new_nodes.append({"@type": n["@type"], "node": nb})
    result["nodes"] = new_nodes
    out["result"] = result
    out["redacted"] = sorted(set(list(out.get("redacted", [])) + stripped))
    return out


# ───────────────────────────────────────────────────────────────
# Tiny client (over an injected callable transport)
# ───────────────────────────────────────────────────────────────

class Consumer:
    """Consumer reference — wraps a transport callable.

    The transport callable takes (method, params) and returns a dict that
    looks like an StxQueryResponse. Use it to wrap your REST/gRPC/GraphQL
    client in your own code.
    """

    def __init__(self, transport):
        self.transport = transport

    def capability(self) -> dict:
        return self.transport("get_capability", {})

    def get_session(self, session_id: str, *, privacy_mode: str = "audit-private") -> QueryResult:
        resp = self.transport("get_session", {"session_id": session_id, "privacy_mode": privacy_mode})
        return validate_query_response(resp)


# ───────────────────────────────────────────────────────────────
# Module self-test
# ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    # Build a valid response and validate it
    resp = {
        "apiVersion": API_VERSION, "kind": KIND_RESPONSE,
        "queryId": "q-test-1", "respondedAt": "2026-05-25T12:00:00.000000Z",
        "privacyMode": "audit-private",
        "result": {
            "nodes": [
                {"@type": "SkillState", "node": {
                    "id": "urn:ckodex:state:abc",
                    "sessionId": "urn:ckodex:session:s1",
                    "skillId": "sha256:" + "a"*64,
                    "skillName": "ckodex-oscal",
                    "tier": "L3",
                    "score": 0.72,
                    "tokenCost": 3500,
                }}
            ],
            "pageInfo": {"hasNext": False, "total": 1},
        },
    }
    r = validate_query_response(resp)
    assert r.ok, f"valid response rejected: {r.errors}"
    print("OK   valid response validates")

    # Inject a privacy violation: public-anchor carrying skillName
    bad = {**resp, "privacyMode": "public-anchor"}
    r = validate_query_response(bad)
    assert not r.ok, "privacy violation should fail validation"
    assert any("privacy violation" in e for e in r.errors), r.errors
    print("OK   public-anchor + skillName flagged as violation")

    # Invalid tier
    bad2 = {**resp}
    bad2["result"] = {**resp["result"], "nodes": [
        {"@type": "SkillState", "node": {**resp["result"]["nodes"][0]["node"], "tier": "L5"}}
    ]}
    r = validate_query_response(bad2)
    assert not r.ok and any("tier" in e for e in r.errors)
    print("OK   invalid tier rejected")

    # Invalid skillId (not sha256)
    bad3 = {**resp}
    bad3["result"] = {**resp["result"], "nodes": [
        {"@type": "SkillState", "node": {**resp["result"]["nodes"][0]["node"], "skillId": "not-a-digest"}}
    ]}
    r = validate_query_response(bad3)
    assert not r.ok and any("skillId" in e for e in r.errors)
    print("OK   non-canonical skillId rejected")

    # Score out of range
    bad4 = {**resp}
    bad4["result"] = {**resp["result"], "nodes": [
        {"@type": "SkillState", "node": {**resp["result"]["nodes"][0]["node"], "score": 1.5}}
    ]}
    r = validate_query_response(bad4)
    assert not r.ok and any("score" in e for e in r.errors)
    print("OK   out-of-range score rejected")

    # Consumer-side redaction
    redacted = redact_for_mode(resp, "public-anchor")
    rn = redacted["result"]["nodes"][0]["node"]
    assert "skillName" not in rn
    assert "score" not in rn
    assert "tokenCost" not in rn
    assert "skillName" in redacted["redacted"]
    print("OK   consumer-side redaction strips public-anchor-forbidden fields")

    print()
    print("Self-test PASS")
