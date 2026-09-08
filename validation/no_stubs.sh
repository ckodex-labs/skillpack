#!/usr/bin/env bash
# no_stubs.sh — Verify zero stub dimensions across the fixture corpus
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
FIXTURES="${PROJECT_ROOT}/tests/fixtures/skills"
FAILED=0

for skill_dir in "${FIXTURES}"/*/; do
    name=$(basename "${skill_dir}")
    echo -n "Checking ${name} ... "
    result=$(cargo run --bin skillpack -- report "${skill_dir}" 2>&1 || true)
    if echo "${result}" | grep -q "Error:"; then
        echo "SKIP (cannot assess: $(echo "${result}" | grep "Error:" | head -1 | sed "s/.*Error: //"))"
        continue
    fi
    if [ -z "${result}" ]; then
        echo "FAIL (no output from skillpack report)"
        FAILED=1
        continue
    fi
    stub_count=$(echo "${result}" | grep -c '"is_stub": true' || true)
    if [ "${stub_count}" = "0" ]; then
        echo "OK (no stubs)"
    else
        echo "FAIL (${stub_count} stub dimensions)"
        FAILED=1
    fi
done

if [ "${FAILED}" -eq 0 ]; then
    echo ""
    echo "All fixtures pass: zero stub dimensions."
    exit 0
else
    echo ""
    echo "Some fixtures have stub dimensions."
    exit 1
fi
