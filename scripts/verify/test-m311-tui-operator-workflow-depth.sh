#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m311-tui-operator-workflow-depth.sh"

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
  tui_shell_binary_operator_panels_conformance
  tui_shell_renderer_operator_panels_conformance
  tui_operator_shell_auth_training_metrics_functional
  tui_shell_live_watch_parse_controls_integration
  tui_shell_live_watch_zero_iterations_regression
  tui_shell_live_watch_marker_contract
  tui_shell_live_watch_help_contract
  tui_live_shell_loads_artifacts_functional
  tui_live_shell_malformed_json_diagnostics_conformance
  tui_live_shell_malformed_jsonl_diagnostics_conformance
  tui_live_shell_missing_artifacts_regression
  tui_operator_shell_control_plane_markers_regression
  tui_operator_shell_auth_markers_regression
)

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

report_dir="${tmp_dir}/artifacts/tui-operator-workflow-depth"
report_path="${report_dir}/verification-report.json"
pass_log="${tmp_dir}/pass.log"

set +e
TAU_M311_MOCK_MODE="1" \
TAU_M311_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${pass_log}" 2>&1
pass_rc=$?
set -e

assert_equals "0" "${pass_rc}" "pass run exit code"

if [[ ! -f "${report_path}" ]]; then
  echo "assertion failed (pass run): expected report at ${report_path}" >&2
  exit 1
fi

assert_equals "1" "$(jq -r '.schema_version' "${report_path}")" "schema version"
assert_equals "m311_tui_operator_workflow_depth" "$(jq -r '.suite_id' "${report_path}")" "suite id"
assert_equals "pass" "$(jq -r '.overall' "${report_path}")" "overall pass"
assert_equals "${#required_steps[@]}" "$(jq -r '.steps | length' "${report_path}")" "step count"
assert_contains "$(cat "${pass_log}")" "m311 tui operator workflow depth verification passed" "pass output"

for step_id in "${required_steps[@]}"; do
  count="$(jq -r --arg id "${step_id}" '.steps | map(select(.id == $id)) | length' "${report_path}")"
  assert_equals "1" "${count}" "required step ${step_id} present"
done

set +e
TAU_M311_MOCK_MODE="1" \
TAU_M311_MOCK_FAIL_PATTERN="tui_live_shell_malformed_jsonl_diagnostics_conformance" \
TAU_M311_REPORT_DIR="${report_dir}" \
"${VERIFY_SCRIPT}" >"${tmp_dir}/fail.log" 2>&1
fail_rc=$?
set -e

assert_equals "1" "${fail_rc}" "fail run exit code"
assert_equals "fail" "$(jq -r '.overall' "${report_path}")" "overall fail"
assert_equals "fail" "$(jq -r '.steps[] | select(.id == "tui_live_shell_malformed_jsonl_diagnostics_conformance") | .status' "${report_path}")" "failed step status"
assert_contains "$(cat "${tmp_dir}/fail.log")" "m311 tui operator workflow depth verification failed" "fail output"

tmp_report="${tmp_dir}/report-tampered.json"
jq ' .steps |= map(select(.id != "tui_operator_shell_auth_markers_regression")) ' "${report_path}" > "${tmp_report}"
mv "${tmp_report}" "${report_path}"

set +e
missing_output="$(
  TAU_M311_VERIFY_ONLY="1" \
  TAU_M311_REPORT_DIR="${report_dir}" \
  "${VERIFY_SCRIPT}" 2>&1
)"
missing_rc=$?
set -e

if [[ ${missing_rc} -eq 0 ]]; then
  echo "assertion failed (missing markers): expected non-zero exit code" >&2
  exit 1
fi

if [[ "${missing_output}" == *"m311 tui operator workflow depth verification passed"* ]]; then
  echo "assertion failed (missing markers): verify-only must not report pass" >&2
  exit 1
fi

echo "m311-tui-operator-workflow-depth verification tests passed"
