#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m314-dashboard-operator-workflow-depth.sh"

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
  ops_chat_selector_syncs_sessions
  ops_sessions_shell_row_contracts
  ops_session_detail_lineage_usage_contracts
  ops_session_detail_graph_contracts
  ops_chat_new_session_redirect_contracts
  ops_sessions_branch_lineage_flow
  ops_session_reset_confirmation_flow
  ops_session_detail_non_empty_message_coverage
  ops_memory_graph_selected_node_contracts
  ops_tools_inventory_contracts
  ops_last_action_readable_rows_contract
)

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

report_dir="${tmp_dir}/artifacts/dashboard-operator-workflow-depth"
report_path="${report_dir}/verification-report.json"
pass_log="${tmp_dir}/pass.log"

set +e
TAU_M314_MOCK_MODE="1" \
TAU_M314_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${pass_log}" 2>&1
pass_rc=$?
set -e

assert_equals "0" "${pass_rc}" "pass run exit code"

if [[ ! -f "${report_path}" ]]; then
  echo "assertion failed (pass run): expected report at ${report_path}" >&2
  exit 1
fi

assert_equals "1" "$(jq -r '.schema_version' "${report_path}")" "schema version"
assert_equals "m314_dashboard_operator_workflow_depth" "$(jq -r '.suite_id' "${report_path}")" "suite id"
assert_equals "pass" "$(jq -r '.overall' "${report_path}")" "overall pass"
assert_equals "${#required_steps[@]}" "$(jq -r '.steps | length' "${report_path}")" "step count"
assert_contains "$(cat "${pass_log}")" "m314 dashboard operator workflow depth verification passed" "pass output"

for step_id in "${required_steps[@]}"; do
  count="$(jq -r --arg id "${step_id}" '.steps | map(select(.id == $id)) | length' "${report_path}")"
  assert_equals "1" "${count}" "required step ${step_id} present"
done

set +e
TAU_M314_MOCK_MODE="1" \
TAU_M314_MOCK_FAIL_PATTERN="ops_session_reset_confirmation_flow" \
TAU_M314_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${tmp_dir}/fail.log" 2>&1
fail_rc=$?
set -e

assert_equals "1" "${fail_rc}" "fail run exit code"
assert_equals "fail" "$(jq -r '.overall' "${report_path}")" "overall fail"
assert_equals "fail" "$(jq -r '.steps[] | select(.id == "ops_session_reset_confirmation_flow") | .status' "${report_path}")" "failed step status"
assert_contains "$(cat "${tmp_dir}/fail.log")" "m314 dashboard operator workflow depth verification failed" "fail output"

tmp_report="${tmp_dir}/report-tampered.json"
jq ' .steps |= map(select(.id != "ops_last_action_readable_rows_contract")) ' "${report_path}" > "${tmp_report}"
mv "${tmp_report}" "${report_path}"

set +e
missing_output="$(
  TAU_M314_VERIFY_ONLY="1" \
  TAU_M314_REPORT_DIR="${report_dir}" \
  "${VERIFY_SCRIPT}" 2>&1
)"
missing_rc=$?
set -e

if [[ ${missing_rc} -eq 0 ]]; then
  echo "assertion failed (missing markers): expected non-zero exit code" >&2
  exit 1
fi

if [[ "${missing_output}" == *"m314 dashboard operator workflow depth verification passed"* ]]; then
  echo "assertion failed (missing markers): verify-only must not report pass" >&2
  exit 1
fi

echo "m314-dashboard-operator-workflow-depth verification tests passed"
