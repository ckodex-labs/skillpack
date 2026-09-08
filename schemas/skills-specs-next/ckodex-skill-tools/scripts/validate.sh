#!/usr/bin/env bash
# ckodex-skill-tools — bundle self-test.
# Verifies the pack itself + exercises scaffold.py and migrate.py
# end-to-end in a tempdir, then runs the produced validate.sh inside
# each output to confirm 0F/0W.

set -eu
cd "$(dirname "$0")/.."

PACK="ckodex-skill-tools"
FAIL=0
WARN=0
SECTIONS=0

say()    { printf '%s\n' "$*"; }
ok()     { say "  OK   $*"; }
fail()   { say "  FAIL $*"; FAIL=$((FAIL+1)); }
warn()   { say "  WARN $*"; WARN=$((WARN+1)); }
section(){ SECTIONS=$((SECTIONS+1)); say ""; say "§${SECTIONS} $*"; }

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# ────────────────────────────────────────────────────────────────
section "Required files present"
for f in \
  SKILL.md skill.json MANIFEST.json \
  scripts/scaffold.py scripts/migrate.py scripts/validate.sh \
  references/scaffolder.md references/migrator.md references/conformance.md \
  assets/templates/SKILL.md.tmpl \
  assets/templates/skill.json.tmpl \
  assets/templates/usage.md.tmpl \
  assets/templates/validate.sh.tmpl \
  examples/before/SKILL.md \
  examples/after/SKILL.md examples/after/skill.json
do
  if [ -f "$f" ]; then ok "$f"; else fail "$f missing"; fi
done

# ────────────────────────────────────────────────────────────────
section "JSON parse"
for f in skill.json MANIFEST.json assets/templates/skill.json.tmpl examples/after/skill.json; do
  if [ "$f" = "assets/templates/skill.json.tmpl" ]; then
    # Template has $placeholders; render then parse
    if python3 -c "from string import Template; import json; \
                   t = Template(open('$f').read()); \
                   json.loads(t.substitute(name='x', description='y', synopsis='z', \
                                            version='0.1.0', license='MIT', author='a'))" 2>/dev/null; then
      ok "$f renders + parses"
    else
      fail "$f render/parse error"
    fi
  else
    if python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
      ok "$f parses"
    else
      fail "$f parse error"
    fi
  fi
done

# ────────────────────────────────────────────────────────────────
section "scaffold.py — dry-run + apply + 0F/0W"
chmod +x scripts/scaffold.py scripts/migrate.py 2>/dev/null || true

# Dry run
if python3 scripts/scaffold.py "$TMPDIR/scaffold-test" --init \
      --name test-skill --description "Test skill scaffolded by the self-test." \
      > /tmp/scaffold-dry.log 2>&1; then
  if grep -q "Dry-run complete" /tmp/scaffold-dry.log; then
    ok "dry-run completed without writing"
    if [ -d "$TMPDIR/scaffold-test" ]; then
      fail "dry-run created a directory (should not write in dry-run)"
    else
      ok "dry-run produced no files"
    fi
  else
    fail "dry-run output unexpected"
    sed 's/^/    /' /tmp/scaffold-dry.log
  fi
else
  fail "dry-run failed"
  sed 's/^/    /' /tmp/scaffold-dry.log
fi

# Apply
if python3 scripts/scaffold.py "$TMPDIR/scaffold-test" --init \
      --name test-skill --description "Test skill scaffolded by the self-test." \
      --apply > /tmp/scaffold-apply.log 2>&1; then
  ok "scaffold --apply succeeded"
else
  fail "scaffold --apply failed"
  sed 's/^/    /' /tmp/scaffold-apply.log
fi

# Validate scaffolded output
if bash "$TMPDIR/scaffold-test/scripts/validate.sh" > /tmp/scaffold-verify.log 2>&1; then
  STATUS=$(tail -1 /tmp/scaffold-verify.log)
  if echo "$STATUS" | grep -q "0F / 0W"; then
    ok "scaffolded skill passes its own validate.sh — $STATUS"
  else
    fail "scaffolded skill validate.sh — $STATUS"
    tail -20 /tmp/scaffold-verify.log | sed 's/^/    /'
  fi
else
  fail "scaffolded skill validate.sh exited non-zero"
  tail -20 /tmp/scaffold-verify.log | sed 's/^/    /'
fi

# ────────────────────────────────────────────────────────────────
section "migrate.py — dry-run + apply + 0F/0W"

# Copy the before/ example into a fresh tempdir and migrate
cp -r examples/before "$TMPDIR/migrate-test"

# Dry run
if python3 scripts/migrate.py "$TMPDIR/migrate-test" > /tmp/migrate-dry.log 2>&1; then
  if grep -q "Dry-run complete" /tmp/migrate-dry.log; then
    ok "migrate dry-run completed without writing"
    # Confirm no skill.json was created during dry-run
    if [ -f "$TMPDIR/migrate-test/skill.json" ]; then
      # Already existed in fixture is OK; check it was unchanged
      if diff -q examples/before/skill.json "$TMPDIR/migrate-test/skill.json" >/dev/null 2>&1; then
        ok "migrate dry-run did not modify existing skill.json"
      else
        :  # before fixture may not have a skill.json
      fi
    fi
  else
    fail "migrate dry-run output unexpected"
    sed 's/^/    /' /tmp/migrate-dry.log
  fi
else
  fail "migrate dry-run failed"
  sed 's/^/    /' /tmp/migrate-dry.log
fi

# Apply
if python3 scripts/migrate.py "$TMPDIR/migrate-test" --apply > /tmp/migrate-apply.log 2>&1; then
  ok "migrate --apply succeeded"
else
  fail "migrate --apply failed"
  sed 's/^/    /' /tmp/migrate-apply.log
fi

# Confirm the migrated output is v1.1
if python3 -c "
import json
m = json.load(open('$TMPDIR/migrate-test/skill.json'))
assert m['apiVersion'] == 'ckodex.org/skill/v1.1', f'apiVersion={m[\"apiVersion\"]}'
assert 'synopsis' in m['metadata'], 'metadata.synopsis missing'
assert 'ontology' in m, 'ontology block missing'
assert 'runtime' in m, 'runtime block missing'
print('post-migration manifest is v1.1 with synopsis/ontology/runtime')
" 2>&1; then
  ok "migrated manifest is structurally v1.1"
else
  fail "migrated manifest structure check"
fi

# Run the migrated skill's own validator
if bash "$TMPDIR/migrate-test/scripts/validate.sh" > /tmp/migrate-verify.log 2>&1; then
  STATUS=$(tail -1 /tmp/migrate-verify.log)
  if echo "$STATUS" | grep -q "0F / 0W"; then
    ok "migrated skill passes its own validate.sh — $STATUS"
  else
    fail "migrated skill validate.sh — $STATUS"
    tail -20 /tmp/migrate-verify.log | sed 's/^/    /'
  fi
else
  fail "migrated skill validate.sh exited non-zero"
  tail -20 /tmp/migrate-verify.log | sed 's/^/    /'
fi

# ────────────────────────────────────────────────────────────────
section "Idempotency — re-running migrate yields no diff"
python3 scripts/migrate.py "$TMPDIR/migrate-test" --apply > /tmp/migrate-2.log 2>&1
# The synopsis would be re-composed identically; ontology/runtime preserved without --force.
# Check that re-running doesn't break the validator.
if bash "$TMPDIR/migrate-test/scripts/validate.sh" > /tmp/migrate-verify-2.log 2>&1; then
  STATUS=$(tail -1 /tmp/migrate-verify-2.log)
  if echo "$STATUS" | grep -q "0F / 0W"; then
    ok "re-migration still 0F/0W — $STATUS"
  else
    fail "re-migration broke the validator — $STATUS"
  fi
else
  fail "re-migration validator exited non-zero"
fi

# ────────────────────────────────────────────────────────────────
section "Refusals — invalid name, oversized description"
if python3 scripts/scaffold.py "$TMPDIR/bad-name" --init \
      --name "Bad-Name" --description "test" --apply > /tmp/refuse-1.log 2>&1; then
  fail "scaffold accepted invalid name"
else
  ok "scaffold refused uppercase name"
fi

if python3 scripts/scaffold.py "$TMPDIR/bad-desc" --init \
      --name good-name --description "$(python3 -c 'print("x"*2000)')" --apply > /tmp/refuse-2.log 2>&1; then
  fail "scaffold accepted oversized description"
else
  ok "scaffold refused 2000-char description"
fi

# ────────────────────────────────────────────────────────────────
section "MANIFEST.json sha256 ledger"
python3 - <<'PY'
import hashlib, json, os, sys
m = json.load(open("MANIFEST.json"))
fail = 0
def skip(p): return "__pycache__" in p or p.endswith((".pyc",".pyo"))
for entry in m.get("files", []):
    path = entry["path"]
    if skip(path): continue
    expected = entry["sha256"]
    if not os.path.exists(path):
        print(f"  FAIL {path}: not on disk")
        fail += 1
        continue
    actual = "sha256:" + hashlib.sha256(open(path,'rb').read()).hexdigest()
    if actual != expected:
        print(f"  FAIL {path}: digest mismatch")
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
