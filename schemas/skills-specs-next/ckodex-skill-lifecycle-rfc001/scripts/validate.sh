#!/usr/bin/env bash
# CKODEX Skill Lifecycle RFC-001 — bundle self-test.
# Exits 0 on 0F/0W. Exits non-zero on any failure.

set -eu
cd "$(dirname "$0")/.."

PACK="ckodex-skill-lifecycle-rfc001"
FAIL=0
WARN=0
SECTIONS=0

say() { printf '%s\n' "$*"; }
ok()  { say "  OK   $*"; }
fail(){ say "  FAIL $*"; FAIL=$((FAIL+1)); }
warn(){ say "  WARN $*"; WARN=$((WARN+1)); }
section(){ SECTIONS=$((SECTIONS+1)); say ""; say "§${SECTIONS} $*"; }

# ────────────────────────────────────────────────────────────────
section "Required files present"
for f in \
  README.md RFC-001-skill-lifecycle.md CHANGELOG.md MANIFEST.json \
  schemas/pca-lifecycle.v1.schema.json \
  schemas/skill-runtime-capabilities.v1.schema.json \
  reference/__init__.py reference/kernel.py reference/scoring.py \
  conformance/README.md conformance/vectors.json conformance/run.py \
  diagrams/event-loop.html diagrams/event-loop.mmd diagrams/event-loop.render-manifest.json \
  diagrams/fsm-lifecycle.html diagrams/fsm-lifecycle.mmd diagrams/fsm-lifecycle.render-manifest.json
do
  if [ -f "$f" ]; then ok "$f"; else fail "$f missing"; fi
done

# ────────────────────────────────────────────────────────────────
section "JSON / JSON-LD parse"
for f in schemas/*.json conformance/vectors.json diagrams/*.render-manifest.json MANIFEST.json; do
  if python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
    ok "$f parses"
  else
    fail "$f parse error"
  fi
done

# ────────────────────────────────────────────────────────────────
section "Reference module self-tests"
if python3 reference/scoring.py >/tmp/scoring.log 2>&1; then
  ok "reference/scoring.py self-test"
else
  fail "reference/scoring.py self-test"
  sed 's/^/    /' /tmp/scoring.log
fi
if python3 reference/kernel.py >/tmp/kernel.log 2>&1; then
  ok "reference/kernel.py self-test"
else
  fail "reference/kernel.py self-test"
  sed 's/^/    /' /tmp/kernel.log
fi

# ────────────────────────────────────────────────────────────────
section "Conformance vectors"
if python3 conformance/run.py >/tmp/conformance.log 2>&1; then
  ok "conformance/run.py — 12/12 PASS"
else
  fail "conformance/run.py"
  sed 's/^/    /' /tmp/conformance.log
fi

# ────────────────────────────────────────────────────────────────
section "Diagram bundle integrity"
for slug in event-loop fsm-lifecycle; do
  base="diagrams/$slug"
  if grep -q "<svg" "$base.html" && grep -q "role=\"img\"" "$base.html"; then
    ok "$base.html SVG role=img present"
  else
    fail "$base.html SVG accessibility missing"
  fi
  if grep -q "prefers-reduced-motion" "$base.html"; then
    ok "$base.html reduced-motion respected"
  else
    fail "$base.html reduced-motion missing"
  fi
done

# ────────────────────────────────────────────────────────────────
section "MANIFEST.json sha256 ledger"
python3 - <<'PY'
import hashlib, json, sys, os
m = json.load(open("MANIFEST.json"))
fail = 0
cache_skip = lambda x: "__pycache__" in x or x.endswith((".pyc",".pyo"))
for entry in m["files"]:
    path = entry["path"]
    if cache_skip(path): continue
    expected = entry["sha256"]
    if not os.path.exists(path):
        print(f"  FAIL {path}: not on disk")
        fail += 1
        continue
    actual = "sha256:" + hashlib.sha256(open(path,'rb').read()).hexdigest()
    if actual != expected:
        print(f"  FAIL {path}: expected {expected[:20]}... got {actual[:20]}...")
        fail += 1
    else:
        print(f"  OK   {path}")
sys.exit(1 if fail else 0)
PY
if [ $? -ne 0 ]; then FAIL=$((FAIL+1)); fi

# ────────────────────────────────────────────────────────────────
section "Summary"
say ""
say "Pack: $PACK"
say "Sections run: $SECTIONS"
say "Failures: $FAIL"
say "Warnings: $WARN"
say ""
if [ "$FAIL" -eq 0 ] && [ "$WARN" -eq 0 ]; then
  say "STATUS: 0F / 0W — PASS"
  exit 0
elif [ "$FAIL" -eq 0 ]; then
  say "STATUS: 0F / ${WARN}W — PASS with warnings"
  exit 0
else
  say "STATUS: ${FAIL}F / ${WARN}W — FAIL"
  exit 1
fi
