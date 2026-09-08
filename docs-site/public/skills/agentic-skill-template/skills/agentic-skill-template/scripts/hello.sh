#!/usr/bin/env bash
# hello.sh — minimal envelope-returning script.
#
# Demonstrates:
#   - emit_ok on success (stdout, exit 0)
#   - emit_error on retriable operational failure (stdout, exit 0)
#   - die on fatal config error (stderr, exit 1)
#   - require_cmd / require_https / mask_token helpers
#
# Usage:
#   bash hello.sh --help
#   bash hello.sh --name <name>
#   bash hello.sh --name <name> --fail-mode <retriable|fatal|none>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Consumed by emit_ok/emit_error/die inside the sourced common.sh; shellcheck
# cannot see across the source boundary, hence the disables.
# shellcheck disable=SC2034
CKODEX_COMMAND=hello
# shellcheck source=lib/common.sh
# shellcheck disable=SC1091
. "${SCRIPT_DIR}/lib/common.sh"

usage() {
  cat <<'EOF'
Usage: hello.sh --name <name> [--fail-mode <retriable|fatal|none>]

Demonstrates the envelope contract by greeting <name>. Optionally simulates
each failure channel for testing the dual-channel contract.

Options:
  --name <name>           Name to greet. Required.
  --fail-mode <mode>      Simulate a failure. One of:
                            retriable -> stdout envelope, exit 0
                            fatal     -> stderr envelope, exit 1
                            none      -> success path (default)
  --help                  Print this usage text and exit 0.

Examples:
  bash hello.sh --name world
  bash hello.sh --name world --fail-mode retriable
  bash hello.sh --name '' --fail-mode fatal      # demonstrates die
EOF
}

NAME=""
FAIL_MODE="none"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h)
      usage; exit 0
      ;;
    --name)
      NAME="${2:-}"; shift 2
      ;;
    --fail-mode)
      FAIL_MODE="${2:-}"; shift 2
      ;;
    *)
      die "UNKNOWN_FLAG" "unknown flag: $1 (use --help)"
      ;;
  esac
done

require_cmd jq

if [[ -z "${NAME}" ]]; then
  die "MISSING_ARG" "--name is required"
fi

case "${FAIL_MODE}" in
  none)
    emit_ok "$(jq -cn --arg n "${NAME}" '{greeting: ("hello, " + $n), name: $n}')"
    ;;
  retriable)
    emit_error "SIMULATED_TRANSPORT_ERROR" \
      "simulated upstream 503 for ${NAME} — agent should retry" \
      true
    ;;
  fatal)
    die "SIMULATED_FATAL" "simulated fatal config error for ${NAME}"
    ;;
  *)
    die "INVALID_FAIL_MODE" "--fail-mode must be one of: none, retriable, fatal"
    ;;
esac
