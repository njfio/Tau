#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m303-auth-workflow-depth.sh"

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

if [[ ! -x "${VERIFY_SCRIPT}" ]]; then
  echo "error: verification script missing or not executable: ${VERIFY_SCRIPT}" >&2
  exit 1
fi

required_steps=(
  provider_auth_workflow_conformance
  gateway_bootstrap_token_mode_contract
  gateway_bootstrap_localhost_dev_mode_contract
  gateway_bootstrap_password_session_mode_contract
  gateway_auth_session_lifecycle_conformance
  gateway_localhost_dev_allows_no_bearer
  gateway_invalid_password_rejected
  gateway_mode_mismatch_rejected
  gateway_malformed_json_rejected
  gateway_lowercase_bearer_accepted
  gateway_password_session_expiry_fail_closed
)

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

report_dir="${tmp_dir}/artifacts/auth-workflow-depth"
report_path="${report_dir}/verification-report.json"
pass_log="${tmp_dir}/pass.log"

set +e
TAU_M303_MOCK_MODE="1" \
TAU_M303_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${pass_log}" 2>&1
pass_rc=$?
set -e

assert_equals "0" "${pass_rc}" "pass run exit code"

if [[ ! -f "${report_path}" ]]; then
  echo "assertion failed (pass run): expected report at ${report_path}" >&2
  exit 1
fi

assert_equals "1" "$(jq -r '.schema_version' "${report_path}")" "schema version"
assert_equals "m303_auth_workflow_depth" "$(jq -r '.suite_id' "${report_path}")" "suite id"
assert_equals "pass" "$(jq -r '.overall' "${report_path}")" "overall pass"
assert_equals "${#required_steps[@]}" "$(jq -r '.steps | length' "${report_path}")" "step count"
assert_contains "$(cat "${pass_log}")" "m303 auth workflow depth verification passed" "pass output"

for step_id in "${required_steps[@]}"; do
  count="$(jq -r --arg id "${step_id}" '.steps | map(select(.id == $id)) | length' "${report_path}")"
  assert_equals "1" "${count}" "required step ${step_id} present"
done

set +e
TAU_M303_MOCK_MODE="1" \
TAU_M303_MOCK_FAIL_PATTERN="gateway_mode_mismatch_rejected" \
TAU_M303_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${tmp_dir}/fail.log" 2>&1
fail_rc=$?
set -e

assert_equals "1" "${fail_rc}" "fail run exit code"
assert_equals "fail" "$(jq -r '.overall' "${report_path}")" "overall fail"
assert_equals "fail" "$(jq -r '.steps[] | select(.id == "gateway_mode_mismatch_rejected") | .status' "${report_path}")" "failed step status"
assert_contains "$(cat "${tmp_dir}/fail.log")" "m303 auth workflow depth verification failed" "fail output"

tmp_report="${tmp_dir}/report-tampered.json"
jq ' .steps |= map(select(.id != "gateway_malformed_json_rejected")) ' "${report_path}" > "${tmp_report}"
mv "${tmp_report}" "${report_path}"

set +e
missing_output="$(
  TAU_M303_VERIFY_ONLY="1" \
  TAU_M303_REPORT_DIR="${report_dir}" \
  "${VERIFY_SCRIPT}" 2>&1
)"
missing_rc=$?
set -e

if [[ ${missing_rc} -eq 0 ]]; then
  echo "assertion failed (missing markers): expected non-zero exit code" >&2
  exit 1
fi

if [[ "${missing_output}" == *"m303 auth workflow depth verification passed"* ]]; then
  echo "assertion failed (missing markers): verify-only must not report pass" >&2
  exit 1
fi

echo "m303-auth-workflow-depth verification tests passed"
