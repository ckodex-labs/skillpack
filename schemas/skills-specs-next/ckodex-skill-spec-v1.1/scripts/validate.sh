#!/usr/bin/env bash
# CKODEX Skill Spec v1.1 — bundle self-test.
# Validates the bundled schemas, examples, and JSON-LD context.
# Exits 0 on 0F/0W. Exits non-zero on any failure.

set -eu
cd "$(dirname "$0")/.."

PACK="ckodex-skill-spec-v1.1"
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
  README.md SPEC.md CHANGELOG.md MANIFEST.json \
  schemas/skill.v1.schema.json \
  schemas/skill.v1.1.schema.json \
  schemas/skillsbundle.v1.schema.json \
  schemas/skill-context-v1.jsonld \
  examples/ckodex-oscal-v1.1.skill.json \
  examples/minimal-v1.1.skill.json \
  examples/third-party-v1-compat.skill.json \
  scripts/validate.py \
  scripts/gen_synopsis.py
do
  if [ -f "$f" ]; then ok "$f"; else fail "$f missing"; fi
done

# ────────────────────────────────────────────────────────────────
section "JSON parse — schemas + JSON-LD + manifests"
for f in schemas/*.json schemas/*.jsonld examples/*.json MANIFEST.json; do
  if python3 -c "import json,sys; json.load(open('$f'))" 2>/dev/null; then
    ok "$f parses"
  else
    fail "$f parse error"
  fi
done

# ────────────────────────────────────────────────────────────────
section "Schema validation — examples against bundled schemas"
if ! python3 -c "import jsonschema, referencing" 2>/dev/null; then
  warn "jsonschema/referencing not installed — skipping schema validation"
  warn "  install with: python3 -m pip install --user jsonschema referencing"
else
  if python3 scripts/validate.py \
       examples/ckodex-oscal-v1.1.skill.json \
       examples/minimal-v1.1.skill.json \
       examples/third-party-v1-compat.skill.json \
     >/tmp/validate.log 2>&1
  then
    ok "all examples validated"
    sed 's/^/    /' /tmp/validate.log
  else
    fail "example validation failed"
    sed 's/^/    /' /tmp/validate.log
  fi
fi

# ────────────────────────────────────────────────────────────────
section "MANIFEST.json — sha256 ledger integrity"
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
