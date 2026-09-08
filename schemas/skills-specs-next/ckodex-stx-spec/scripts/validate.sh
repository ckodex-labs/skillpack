#!/usr/bin/env bash
# scripts/validate.sh — STX spec pack bundle self-test
#
# Eight sections, all targeting 0F / 0W:
#   §1 required files present
#   §2 JSON parse + JSON Schema 2020-12 validity (3 schemas + 1 JSON-LD)
#   §3 OpenAPI / proto / GraphQL syntactic parse (stdlib-friendly checks)
#   §4 reference publisher self-test
#   §5 reference consumer self-test
#   §6 conformance vectors 10/10
#   §7 examples JSON parse (6 files)
#   §8 MANIFEST.json sha256 ledger consistency

set -u
PACK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PACK_ROOT" || { echo "FATAL: cannot cd to pack root"; exit 2; }

PY="${PYTHON:-python3}"
FAILS=0
WARNS=0

ok()   { printf "  OK    %s\n" "$1"; }
fail() { printf "  FAIL  %s\n" "$1"; FAILS=$((FAILS+1)); }
warn() { printf "  WARN  %s\n" "$1"; WARNS=$((WARNS+1)); }

section() { printf "\n=== %s ===\n" "$1"; }

# ───────────────────────────────────────────────────────────────
section "§1 required files present"

REQUIRED=(
  "SKILL.md" "README.md" "CHANGELOG.md" "skill.json"
  "schemas/stx-domain.v1.schema.json"
  "schemas/stx-event.v1.schema.json"
  "schemas/stx-query-response.v1.schema.json"
  "schemas/stx-context-v1.jsonld"
  "protocols/openapi.yaml"
  "protocols/stx.proto"
  "protocols/schema.graphql"
  "reference/__init__.py"
  "reference/publisher.py"
  "reference/consumer.py"
  "conformance/vectors.json"
  "conformance/run.py"
  "references/SPEC.md"
  "references/publisher.md"
  "references/consumer.md"
  "references/transports.md"
  "examples/handshake-request.json"
  "examples/handshake-response.json"
  "examples/event-tier-promotion.json"
  "examples/event-evidence-emit.json"
  "examples/query-skill-pool.json"
  "examples/query-skill-pool-response.json"
  "scripts/validate.sh"
  "diagrams/domain-graph.html"
  "diagrams/domain-graph.mmd"
  "diagrams/domain-graph.render-manifest.json"
  "diagrams/publish-subscribe.html"
  "diagrams/publish-subscribe.mmd"
  "diagrams/publish-subscribe.render-manifest.json"
)

for f in "${REQUIRED[@]}"; do
  if [ -f "$f" ]; then ok "$f"; else fail "missing: $f"; fi
done

# ───────────────────────────────────────────────────────────────
section "§2 JSON parse + JSON Schema 2020-12 validity"

JSON_FILES=(
  "skill.json"
  "schemas/stx-domain.v1.schema.json"
  "schemas/stx-event.v1.schema.json"
  "schemas/stx-query-response.v1.schema.json"
  "schemas/stx-context-v1.jsonld"
  "conformance/vectors.json"
)
for f in "${JSON_FILES[@]}"; do
  if "$PY" -c "import json,sys; json.load(open('$f'))" 2>/dev/null; then
    ok "JSON parse: $f"
  else
    fail "JSON parse error: $f"
  fi
done

# Confirm the three normative schemas declare $schema = draft 2020-12 and $id
for f in schemas/stx-domain.v1.schema.json schemas/stx-event.v1.schema.json schemas/stx-query-response.v1.schema.json; do
  if "$PY" -c "
import json,sys
d=json.load(open('$f'))
assert d.get('\$schema')=='https://json-schema.org/draft/2020-12/schema', 'wrong \$schema'
assert d.get('\$id','').startswith('https://schemas.ckodex.org/'), 'missing canonical \$id'
" 2>/dev/null; then
    ok "Draft 2020-12 + \$id: $f"
  else
    fail "schema metadata wrong: $f"
  fi
done

# Discriminate Schema vs JSON-LD: the .jsonld file MUST NOT carry $schema (it's a context)
if "$PY" -c "
import json
d=json.load(open('schemas/stx-context-v1.jsonld'))
assert '@context' in d, 'JSON-LD missing @context'
assert '\$schema' not in d, 'JSON-LD must not be a JSON Schema'
" 2>/dev/null; then
  ok "JSON-LD context shape: stx-context-v1.jsonld"
else
  fail "JSON-LD context shape wrong"
fi

# defaultPrivacyMode const = "audit-private" — binding for vector P1
if "$PY" -c "
import json
d=json.load(open('schemas/stx-domain.v1.schema.json'))
cap = d['\$defs']['CapabilityDescriptor']
assert cap['properties']['defaultPrivacyMode'].get('\$ref','').endswith('/PrivacyMode'), 'defaultPrivacyMode ref'
# We don't const it (so non-v1 forks can change it), but require statement of default privacy mode invariant in SPEC.md
" 2>/dev/null; then
  ok "CapabilityDescriptor schema shape OK"
else
  fail "CapabilityDescriptor schema shape wrong"
fi

# ───────────────────────────────────────────────────────────────
section "§3 protocol files parse (REST / gRPC / GraphQL)"

# OpenAPI: minimal stdlib check — YAML loadable if PyYAML available, else line/key sanity
if "$PY" -c "
try:
    import yaml
    d = yaml.safe_load(open('protocols/openapi.yaml'))
    assert d['openapi'].startswith('3.1'), 'not OpenAPI 3.1'
    assert 'paths' in d and '/capability' in d['paths'], 'missing /capability path'
    print('OPENAPI_YAML_OK')
except ImportError:
    # fall back: byte-level grep for required top-level keys
    text = open('protocols/openapi.yaml').read()
    assert 'openapi: 3.1.0' in text
    assert '/capability:' in text
    assert '/sessions:' in text
    assert '/events:' in text
    print('OPENAPI_YAML_GREP_OK')
" 2>/dev/null; then
  ok "OpenAPI parse: protocols/openapi.yaml"
else
  fail "OpenAPI parse failed"
fi

# proto3: check syntax declaration + service block present
if grep -q '^syntax = "proto3";' protocols/stx.proto \
    && grep -q '^service SkillsTransparencyExchange' protocols/stx.proto \
    && grep -q '^  rpc GetCapability' protocols/stx.proto \
    && grep -q '^  rpc StreamEvents' protocols/stx.proto; then
  ok "proto3 service shape: protocols/stx.proto"
else
  fail "proto3 service shape wrong"
fi

# GraphQL: check Query + Subscription roots and the union
if grep -q '^type Query {' protocols/schema.graphql \
    && grep -q '^type Subscription {' protocols/schema.graphql \
    && grep -q '^union GraphNodeUnion' protocols/schema.graphql \
    && grep -q '  capability: CapabilityDescriptor!' protocols/schema.graphql; then
  ok "GraphQL SDL shape: protocols/schema.graphql"
else
  fail "GraphQL SDL shape wrong"
fi

# ───────────────────────────────────────────────────────────────
section "§4 reference publisher self-test"

if PUB_OUT="$("$PY" reference/publisher.py 2>&1)" && echo "$PUB_OUT" | grep -q '^Self-test PASS$'; then
  ok "reference/publisher.py self-test PASS"
else
  fail "reference/publisher.py self-test"
  echo "$PUB_OUT" | sed 's/^/    /'
fi

# ───────────────────────────────────────────────────────────────
section "§5 reference consumer self-test"

if CON_OUT="$("$PY" reference/consumer.py 2>&1)" && echo "$CON_OUT" | grep -q '^Self-test PASS$'; then
  ok "reference/consumer.py self-test PASS"
else
  fail "reference/consumer.py self-test"
  echo "$CON_OUT" | sed 's/^/    /'
fi

# ───────────────────────────────────────────────────────────────
section "§6 conformance vectors 10/10"

if CONF_OUT="$("$PY" conformance/run.py 2>&1)" \
    && echo "$CONF_OUT" | tail -3 | grep -q '10/10 PASS, 0 FAIL'; then
  ok "conformance/run.py 10/10 PASS"
else
  fail "conformance vectors did not reach 10/10"
  echo "$CONF_OUT" | sed 's/^/    /'
fi

# ───────────────────────────────────────────────────────────────
section "§7 examples JSON parse"

EXAMPLES=(
  "examples/handshake-request.json"
  "examples/handshake-response.json"
  "examples/event-tier-promotion.json"
  "examples/event-evidence-emit.json"
  "examples/query-skill-pool.json"
  "examples/query-skill-pool-response.json"
)
for f in "${EXAMPLES[@]}"; do
  if "$PY" -c "import json; json.load(open('$f'))" 2>/dev/null; then
    ok "$f"
  else
    fail "JSON parse: $f"
  fi
done

# Spot-check: handshake-response.json defaultPrivacyMode == audit-private
if "$PY" -c "
import json
d = json.load(open('examples/handshake-response.json'))
assert d['defaultPrivacyMode'] == 'audit-private', 'wrong default'
" 2>/dev/null; then
  ok "handshake-response.defaultPrivacyMode == audit-private"
else
  fail "handshake-response default mismatch"
fi

# ───────────────────────────────────────────────────────────────
section "§8 MANIFEST.json sha256 ledger"

if [ -f MANIFEST.json ]; then
  if "$PY" - <<'PYEOF'
import hashlib, json, os, sys
from pathlib import Path

m = json.load(open('MANIFEST.json'))
fails = []
files_listed = set()

for entry in m['files']:
    p = Path(entry['path'])
    files_listed.add(str(p))
    if not p.exists():
        fails.append(f"MANIFEST lists missing file: {p}")
        continue
    h = hashlib.sha256(p.read_bytes()).hexdigest()
    expected = entry['sha256']
    if h != expected:
        fails.append(f"sha256 mismatch: {p}\n    expected {expected}\n    actual   {h}")

# Walk filesystem; flag any file not in MANIFEST (excluding build/cache)
EXCLUDE_DIRS = {'__pycache__', '.git', 'target', 'demo-out', 'node_modules'}
EXCLUDE_SUFFIXES = ('.pyc', '.pyo')
for root, dirs, files in os.walk('.'):
    dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]
    for f in files:
        p = Path(root, f)
        # canonicalize: drop leading './'
        rel = str(p).lstrip('./').replace(os.sep, '/')
        if rel == 'MANIFEST.json':
            continue
        if any(rel.endswith(s) for s in EXCLUDE_SUFFIXES):
            continue
        if rel not in files_listed:
            fails.append(f"file not in MANIFEST: {rel}")

if fails:
    for f in fails:
        print(f"FAIL {f}")
    sys.exit(1)
print(f"MANIFEST: {len(m['files'])} entries, all sha256 verified, no orphan files")
PYEOF
  then
    ok "MANIFEST.json sha256 ledger consistent"
  else
    fail "MANIFEST.json sha256 ledger mismatch"
  fi
else
  warn "MANIFEST.json absent (run scripts/gen_manifest.py first)"
fi

# ───────────────────────────────────────────────────────────────
printf "\n────────────────────────────────────────────────────\n"
printf "STATUS: %dF / %dW" "$FAILS" "$WARNS"
if [ "$FAILS" -eq 0 ] && [ "$WARNS" -eq 0 ]; then
  printf " — PASS\n"
  exit 0
fi
printf " — FAIL\n"
exit 1
