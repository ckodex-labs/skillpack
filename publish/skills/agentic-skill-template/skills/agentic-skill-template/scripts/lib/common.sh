#!/usr/bin/env bash
# common.sh — shared primitives for envelope-returning scripts.
#
# Sourced by every script in scripts/. Provides:
#   emit_ok       : success envelope on stdout, exit 0
#   emit_error    : retriable error envelope on stdout, exit 0
#   die           : fatal config-error envelope on stderr, exit 1
#   mask_token    : print "****<last4>" — never echo raw tokens
#   require_cmd   : assert a binary is on PATH
#   require_https : reject non-HTTPS URLs
#   require_env   : assert an env var is non-empty
#
# Contract: every script writes exactly one envelope. Success and retriable
# operational errors land on stdout/exit 0. Fatal config errors land on
# stderr/exit 1. Agents read stdout first; if empty and exit != 0, stderr.

set -euo pipefail

# Command name is derived from the calling script for the metadata block.
# Override by setting CKODEX_COMMAND before sourcing.
: "${CKODEX_COMMAND:=$(basename "${BASH_SOURCE[1]:-unknown}" .sh)}"
: "${CKODEX_VERSION:=0.1.0}"

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    die "MISSING_COMMAND" "required command not found on PATH: ${cmd}"
  fi
}

require_env() {
  local var="$1"
  if [[ -z "${!var:-}" ]]; then
    die "MISSING_ENV" "required environment variable not set: ${var}"
  fi
}

require_https() {
  local url="$1"
  if [[ ! "${url}" =~ ^https:// ]]; then
    die "INVALID_URL_SCHEME" "URL must use https://, got: ${url%%://*}://..."
  fi
}

mask_token() {
  local tok="${1:-}"
  local len=${#tok}
  if (( len <= 4 )); then
    printf '****'
  else
    printf '****%s' "${tok: -4}"
  fi
}

# Success: envelope on stdout, exit 0.
# Usage: emit_ok '<json data object>'
# Note: the default empty object is built without `${1:-{}}` because bash
# closes the parameter expansion at the first `}`, appending a stray brace.
emit_ok() {
  local data="${1:-}"
  if [[ -z "${data}" ]]; then
    data='{}'
  fi
  jq -cn \
    --argjson data "${data}" \
    --arg command "${CKODEX_COMMAND}" \
    --arg version "${CKODEX_VERSION}" \
    '{ok: true, data: $data, metadata: {command: $command, version: $version}}'
  exit 0
}

# Retriable operational error: envelope on stdout, exit 0.
# Usage: emit_error CODE "message" [retriable=true]
emit_error() {
  local code="${1:?emit_error: code required}"
  local message="${2:?emit_error: message required}"
  local retriable="${3:-true}"
  jq -cn \
    --arg code "${code}" \
    --arg message "${message}" \
    --argjson retriable "${retriable}" \
    --arg command "${CKODEX_COMMAND}" \
    --arg version "${CKODEX_VERSION}" \
    '{ok: false, error: {code: $code, message: $message, retriable: $retriable}, metadata: {command: $command, version: $version}}'
  exit 0
}

# Fatal config error: envelope on stderr, exit 1. Agents must NOT retry.
# Usage: die CODE "message"
die() {
  local code="${1:?die: code required}"
  local message="${2:?die: message required}"
  jq -cn \
    --arg code "${code}" \
    --arg message "${message}" \
    --arg command "${CKODEX_COMMAND}" \
    --arg version "${CKODEX_VERSION}" \
    '{ok: false, error: {code: $code, message: $message, retriable: false}, metadata: {command: $command, version: $version}}' >&2
  exit 1
}
