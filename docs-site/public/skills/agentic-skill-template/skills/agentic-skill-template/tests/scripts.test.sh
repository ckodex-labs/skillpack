#!/usr/bin/env bash
# scripts.test.sh — shellcheck, --help smoke, and envelope contract regression.
set -euo pipefail

SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../scripts" && pwd)"
fail=0

echo "1) shellcheck all scripts"
if ! command -v shellcheck >/dev/null 2>&1; then
  echo "  WARN: shellcheck not installed — skipping; install with 'brew install shellcheck'"
else
  while read -r f; do
    if shellcheck -x "${f}"; then
      echo "  ok  : shellcheck ${f##*/}"
    else
      echo "  FAIL: shellcheck ${f##*/}"; fail=1
    fi
  done < <(find "${SCRIPTS_DIR}" -name '*.sh' -type f)
fi

echo "2) every script supports --help without arguments (exit 0 or 64)"
while read -r f; do
  rc=0; out="$(bash "${f}" --help 2>&1)" || rc=$?
  if [[ "${rc}" -eq 0 || "${rc}" -eq 64 ]]; then
    if echo "${out}" | grep -qiE 'usage|help'; then
      echo "  ok  : --help ${f##*/}"
    else
      echo "  FAIL: --help ${f##*/} missing usage text"; fail=1
    fi
  else
    echo "  FAIL: --help ${f##*/} exit code ${rc}"; fail=1
  fi
done < <(find "${SCRIPTS_DIR}" -maxdepth 1 -name '*.sh' -type f)

echo "3) success path: hello --name demo → stdout envelope, ok:true, exit 0"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT
rc=0
bash "${SCRIPTS_DIR}/hello.sh" --name demo \
  >"${tmpdir}/out" 2>"${tmpdir}/err" || rc=$?

if [[ "${rc}" -ne 0 ]]; then
  echo "  FAIL: success path exit=${rc} (expected 0)"; fail=1
elif ! jq -e '.ok == true and .data.greeting == "hello, demo" and .data.name == "demo"' \
       >/dev/null < "${tmpdir}/out" 2>/dev/null; then
  echo "  FAIL: success envelope wrong shape"
  echo "       stdout: $(head -c 200 "${tmpdir}/out")"
  fail=1
elif [[ -s "${tmpdir}/err" ]]; then
  echo "  FAIL: success path wrote to stderr (should be empty)"
  echo "       stderr: $(head -c 200 "${tmpdir}/err")"
  fail=1
else
  echo "  ok  : success → stdout envelope, ok:true, exit 0"
fi

echo "4) retriable error: hello --fail-mode retriable → stdout envelope, retriable:true, exit 0"
rc=0
bash "${SCRIPTS_DIR}/hello.sh" --name demo --fail-mode retriable \
  >"${tmpdir}/out" 2>"${tmpdir}/err" || rc=$?

if [[ "${rc}" -ne 0 ]]; then
  echo "  FAIL: retriable path exit=${rc} (expected 0)"; fail=1
elif ! jq -e '.ok == false and .error.retriable == true and .error.code == "SIMULATED_TRANSPORT_ERROR"' \
       >/dev/null < "${tmpdir}/out" 2>/dev/null; then
  echo "  FAIL: retriable envelope wrong shape"
  echo "       stdout: $(head -c 200 "${tmpdir}/out")"
  fail=1
elif [[ -s "${tmpdir}/err" ]]; then
  echo "  FAIL: retriable path wrote to stderr (should be empty)"
  echo "       stderr: $(head -c 200 "${tmpdir}/err")"
  fail=1
else
  echo "  ok  : retriable → stdout envelope, retriable:true, exit 0"
fi

echo "5) fatal error: hello --fail-mode fatal → stderr envelope, retriable:false, exit 1"
rc=0
bash "${SCRIPTS_DIR}/hello.sh" --name demo --fail-mode fatal \
  >"${tmpdir}/out" 2>"${tmpdir}/err" || rc=$?

if [[ "${rc}" -ne 1 ]]; then
  echo "  FAIL: fatal path exit=${rc} (expected 1)"; fail=1
elif ! jq -e '.ok == false and .error.retriable == false and .error.code == "SIMULATED_FATAL"' \
       >/dev/null < "${tmpdir}/err" 2>/dev/null; then
  echo "  FAIL: fatal envelope wrong shape on stderr"
  echo "       stderr: $(head -c 200 "${tmpdir}/err")"
  fail=1
elif [[ -s "${tmpdir}/out" ]]; then
  echo "  FAIL: fatal path wrote to stdout (should be empty)"
  echo "       stdout: $(head -c 200 "${tmpdir}/out")"
  fail=1
else
  echo "  ok  : fatal → stderr envelope, retriable:false, exit 1"
fi

echo "6) missing required arg: hello (no --name) → stderr envelope, MISSING_ARG, exit 1"
rc=0
bash "${SCRIPTS_DIR}/hello.sh" \
  >"${tmpdir}/out" 2>"${tmpdir}/err" || rc=$?

if [[ "${rc}" -ne 1 ]]; then
  echo "  FAIL: missing-arg path exit=${rc} (expected 1)"; fail=1
elif ! jq -e '.ok == false and .error.code == "MISSING_ARG"' \
       >/dev/null < "${tmpdir}/err" 2>/dev/null; then
  echo "  FAIL: missing-arg envelope wrong shape"
  echo "       stderr: $(head -c 200 "${tmpdir}/err")"
  fail=1
else
  echo "  ok  : missing required arg → stderr envelope, MISSING_ARG, exit 1"
fi

exit "${fail}"
