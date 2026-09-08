#!/usr/bin/env bash
# structure.test.sh — verify the template's directory layout, manifest files,
# and frontmatter caps.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SKILL_ROOT="${ROOT}/skills/agentic-skill-template"
fail=0

echo "1) required files exist"
required=(
  ".claude-plugin/plugin.json"
  "catalog.json"
  "README.md"
  "agents/agentic-engineer.md"
  "skills/agentic-skill-template/SKILL.md"
  "skills/agentic-skill-template/scripts/hello.sh"
  "skills/agentic-skill-template/scripts/lib/common.sh"
  "skills/agentic-skill-template/tests/structure.test.sh"
  "skills/agentic-skill-template/tests/scripts.test.sh"
  "skills/agentic-skill-template/tests/agent.test.sh"
  "skills/agentic-skill-template/references/EXAMPLE.md"
)
for f in "${required[@]}"; do
  if [[ -f "${ROOT}/${f}" ]]; then
    echo "  ok  : ${f}"
  else
    echo "  FAIL: missing ${f}"; fail=1
  fi
done

echo "2) plugin.json is valid JSON and has required fields"
plugin_json="${ROOT}/.claude-plugin/plugin.json"
if jq -e '.name and .version and .description and .license' "${plugin_json}" >/dev/null 2>&1; then
  echo "  ok  : plugin.json schema"
else
  echo "  FAIL: plugin.json missing name|version|description|license"; fail=1
fi

echo "3) catalog.json is valid JSON and lists the skill + agent"
catalog_json="${ROOT}/catalog.json"
if jq -e '.skills[0].path == "skills/agentic-skill-template/SKILL.md" and .agents[0].path == "agents/agentic-engineer.md"' \
     "${catalog_json}" >/dev/null 2>&1; then
  echo "  ok  : catalog.json paths"
else
  echo "  FAIL: catalog.json skill/agent paths do not match expected layout"; fail=1
fi

echo "4) SKILL.md frontmatter satisfies the schema (name pattern, description ≤1024)"
skill_md="${SKILL_ROOT}/SKILL.md"
fm="$(awk '/^---$/{c++; next} c==1 {print} c==2 {exit}' "${skill_md}")"
name="$(awk -F': ' '/^name:/{print $2; exit}' <<<"${fm}")"
desc="$(awk -F': ' '/^description:/{$1=""; sub(/^ /,""); print; exit}' <<<"${fm}")"
desc_len=${#desc}

if [[ "${name}" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] && [[ ${#name} -le 64 ]]; then
  echo "  ok  : name '${name}' matches ^[a-z0-9]+(-[a-z0-9]+)*$ and ≤64 chars"
else
  echo "  FAIL: name '${name}' invalid"; fail=1
fi

if (( desc_len >= 1 && desc_len <= 1024 )); then
  echo "  ok  : description ${desc_len} chars (1..1024)"
else
  echo "  FAIL: description ${desc_len} chars (must be 1..1024)"; fail=1
fi

echo "5) SKILL.md body ≤500 lines (progressive disclosure)"
body_lines="$(wc -l < "${skill_md}" | tr -d ' ')"
if (( body_lines <= 500 )); then
  echo "  ok  : SKILL.md is ${body_lines} lines"
else
  echo "  FAIL: SKILL.md is ${body_lines} lines (>500 burns post-compaction budget)"; fail=1
fi

echo "6) name in SKILL.md matches parent directory name"
parent_dir="$(basename "${SKILL_ROOT}")"
if [[ "${name}" == "${parent_dir}" ]]; then
  echo "  ok  : frontmatter name matches parent dir '${parent_dir}'"
else
  echo "  FAIL: frontmatter name '${name}' != parent dir '${parent_dir}'"; fail=1
fi

exit "${fail}"
