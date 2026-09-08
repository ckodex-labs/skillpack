#!/usr/bin/env bash
# agent.test.sh — verify the orchestrator agent's frontmatter, routing
# coverage, and safety-constraint anchors.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
AGENT_MD="${ROOT}/agents/agentic-engineer.md"
SKILL_ROOT="${ROOT}/skills/agentic-skill-template"
fail=0

echo "1) agents/ exists and orchestrator file is present"
if [[ -f "${AGENT_MD}" ]]; then
  echo "  ok  : agents/agentic-engineer.md present"
else
  echo "  FAIL: agents/agentic-engineer.md missing"; fail=1; exit "${fail}"
fi

echo "2) frontmatter has name|description|model|tools and description ≤1024 chars"
fm="$(awk '/^---$/{c++; next} c==1 {print} c==2 {exit}' "${AGENT_MD}")"
name="$(awk -F': ' '/^name:/{print $2; exit}' <<<"${fm}")"
desc="$(awk -F': ' '/^description:/{$1=""; sub(/^ /,""); print; exit}' <<<"${fm}")"
model="$(awk -F': ' '/^model:/{print $2; exit}' <<<"${fm}")"
tools="$(awk -F': ' '/^tools:/{$1=""; sub(/^ /,""); print; exit}' <<<"${fm}")"
desc_len=${#desc}

if [[ "${name}" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] && [[ ${#name} -le 64 ]]; then
  echo "  ok  : name '${name}' matches ^[a-z0-9]+(-[a-z0-9]+)*\$ and ≤64 chars"
else
  echo "  FAIL: name '${name}' invalid"; fail=1
fi

if (( desc_len >= 1 && desc_len <= 1024 )); then
  echo "  ok  : description ${desc_len} chars (1..1024)"
else
  echo "  FAIL: description ${desc_len} chars (must be 1..1024)"; fail=1
fi

if [[ -n "${model}" ]]; then
  echo "  ok  : model: ${model}"
else
  echo "  FAIL: model: missing from frontmatter"; fail=1
fi

if [[ -n "${tools}" ]]; then
  echo "  ok  : tools: ${tools}"
else
  echo "  FAIL: tools: missing from frontmatter"; fail=1
fi

echo "3) every anchor in the routing table resolves to an existing script or doc"
# Extract every backticked scripts/*.sh or references/*.md reference inside the
# routing table section and verify the target exists. The routing table opens
# at '## Routing table' and closes at the next '---'.
routing_block="$(awk '
  /^## Routing table/{flag=1; next}
  flag && /^---$/{exit}
  flag{print}
' "${AGENT_MD}")"

# Pull every `scripts/<file>.sh` and `references/<file>.md` (incl. flag args).
# Portable to bash 3.2 (macOS): no `mapfile` / `readarray`.
target_count=0
while IFS= read -r t; do
  [[ -z "${t}" ]] && continue
  target_count=$((target_count + 1))
  path="${SKILL_ROOT}/${t}"
  if [[ -e "${path}" ]]; then
    echo "  ok  : routing target ${t}"
  else
    echo "  FAIL: routing target ${t} not found at ${path}"; fail=1
  fi
done < <(
  echo "${routing_block}" \
    | grep -oE '`(scripts|references)/[A-Za-z0-9_./-]+(\.sh|\.md)' \
    | sed 's/^`//' \
    | sort -u
)

if (( target_count == 0 )); then
  echo "  FAIL: routing table contained no script/reference anchors"; fail=1
fi

echo "4) channel + exit-code contract is documented"
if grep -qE 'stdout' "${AGENT_MD}" \
   && grep -qE 'stderr' "${AGENT_MD}" \
   && grep -qE 'exit' "${AGENT_MD}" \
   && grep -qiE 'retriable' "${AGENT_MD}"; then
  echo "  ok  : stdout|stderr|exit|retriable all documented"
else
  echo "  FAIL: channel + exit-code contract anchors missing"; fail=1
fi

echo "5) safety constraints mention HTTPS, dry-run, signature, token-mask"
missing=()
grep -qiE 'https'                  "${AGENT_MD}" || missing+=("HTTPS")
grep -qiE 'dry.?run'               "${AGENT_MD}" || missing+=("dry-run")
grep -qiE 'signature|cosign|slsa'  "${AGENT_MD}" || missing+=("signature")
grep -qiE 'mask|token'             "${AGENT_MD}" || missing+=("token-mask")

if [[ ${#missing[@]} -eq 0 ]]; then
  echo "  ok  : HTTPS|dry-run|signature|token-mask anchors present"
else
  echo "  FAIL: missing safety anchors: ${missing[*]}"; fail=1
fi

exit "${fail}"
