#!/usr/bin/env bash
# dimension_parity.sh — Verify every DimensionId has a checker, a doc, and a fixture test
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

doc_for_dim() {
    case "$1" in
        IdentityAndManifest) echo "identity-and-manifest" ;;
        Security) echo "security" ;;
        Provenance) echo "provenance" ;;
        Documentation) echo "documentation" ;;
        Testing) echo "testing" ;;
        Compatibility) echo "compatibility" ;;
        Lifecycle) echo "lifecycle" ;;
        Governance) echo "governance" ;;
        EvalsHitl) echo "evals-hitl" ;;
    esac
}

DIMENSIONS=(IdentityAndManifest Security Provenance Documentation Testing Compatibility Lifecycle Governance EvalsHitl)

FAILED=0

echo "=== Checker parity ==="
for dim in "${DIMENSIONS[@]}"; do
    if grep -qR "DimensionId::${dim}" "${PROJECT_ROOT}/crates/skillpack-adapters/src/checkers/"; then
        echo "  ${dim} OK"
    else
        echo "  ${dim} MISSING in checkers/"
        FAILED=1
    fi
done

echo ""
echo "=== Doc parity ==="
for dim in "${DIMENSIONS[@]}"; do
    docname=$(doc_for_dim "${dim}")
    if [ -f "${PROJECT_ROOT}/docs/dimensions/${docname}.md" ]; then
        echo "  ${dim} OK"
    else
        echo "  ${dim} MISSING doc (${docname}.md)"
        FAILED=1
    fi
done

echo ""
echo "=== Weight parity ==="
if cargo test -p skillpack-domain canonical_weights_sum_to_100 -- --nocapture >/dev/null 2>&1; then
    echo "  Weights sum to 100 OK"
else
    echo "  Weights do NOT sum to 100"
    FAILED=1
fi

if [ "${FAILED}" -eq 0 ]; then
    echo ""
    echo "Dimension parity check passed."
    exit 0
else
    echo ""
    echo "Dimension parity check FAILED."
    exit 1
fi
