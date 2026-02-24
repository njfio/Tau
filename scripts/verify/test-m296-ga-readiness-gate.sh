#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m296-ga-readiness-gate.sh"

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

require_cmd jq

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

report_dir="${tmp_dir}/artifacts/operator-ga-readiness"
report_path="${report_dir}/verification-report.json"
pass_log="${tmp_dir}/pass.log"

set +e
TAU_M296_MOCK_MODE="1" \
TAU_M296_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" --generated-at "2026-02-23T00:00:00Z" >"${pass_log}" 2>&1
pass_rc=$?
set -e

if [[ ${pass_rc} -ne 0 ]]; then
  echo "assertion failed (pass run): expected zero exit code" >&2
  cat "${pass_log}" >&2
  exit 1
fi

if [[ ! -f "${report_path}" ]]; then
  echo "assertion failed (pass run): expected report at ${report_path}" >&2
  exit 1
fi

assert_equals "m296_ga_readiness_gate" "$(jq -r '.suite_id' "${report_path}")" "suite id"
assert_equals "pass" "$(jq -r '.overall' "${report_path}")" "overall pass"
assert_equals "9" "$(jq -r '.steps | length' "${report_path}")" "step count"
assert_equals "6" "$(jq -r '.signoff_criteria | length' "${report_path}")" "signoff count"
assert_equals "ready" "$(jq -r '.closeout_summary.status' "${report_path}")" "closeout pass status"
assert_equals "proof-summary-missing" "$(jq -r '.rollback_trigger_matrix.trigger_ids[0]' "${report_path}")" "rollback trigger seed"
assert_contains "$(cat "${pass_log}")" "ga readiness verification passed" "pass command output"

set +e
TAU_M296_MOCK_MODE="1" \
TAU_M296_MOCK_SKIP_PATTERN="auth_live_validation_matrix" \
TAU_M296_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" --generated-at "2026-02-23T00:00:00Z" >"${tmp_dir}/skip.log" 2>&1
skip_rc=$?
set -e

assert_equals "0" "${skip_rc}" "skip run exit code"
assert_equals "skip" "$(jq -r '.steps[] | select(.id == "auth_live_validation_matrix") | .status' "${report_path}")" "auth live skip status"
assert_equals "pass" "$(jq -r '.signoff_criteria[] | select(.id == "auth_live_validation") | .status' "${report_path}")" "auth live signoff with skip"
assert_equals "ready" "$(jq -r '.closeout_summary.status' "${report_path}")" "closeout skip status"
assert_contains "$(cat "${tmp_dir}/skip.log")" "ga readiness verification passed" "skip command output"

set +e
TAU_M296_MOCK_MODE="1" \
TAU_M296_MOCK_FAIL_PATTERN="rollback_contract_checks" \
TAU_M296_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" --generated-at "2026-02-23T00:00:00Z" >"${tmp_dir}/fail.log" 2>&1
fail_rc=$?
set -e

assert_equals "1" "${fail_rc}" "fail run exit code"
assert_equals "fail" "$(jq -r '.overall' "${report_path}")" "overall fail"
assert_equals "fail" "$(jq -r '.steps[] | select(.id == "rollback_contract_checks") | .status' "${report_path}")" "failing step status"
assert_equals "fail" "$(jq -r '.signoff_criteria[] | select(.id == "rollback_contract") | .status' "${report_path}")" "rollback signoff fail"
assert_equals "blocked" "$(jq -r '.closeout_summary.status' "${report_path}")" "closeout fail status"
assert_contains "$(cat "${tmp_dir}/fail.log")" "ga readiness verification failed" "fail command output"

echo "m296-ga-readiness-gate tests passed"
