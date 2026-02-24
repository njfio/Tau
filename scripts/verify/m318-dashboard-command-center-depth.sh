#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M318_REPORT_DIR:-${ROOT_DIR}/artifacts/dashboard-command-center-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M318_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M318_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M318_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M318_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M318_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  m308_dashboard_live_mutation_depth_contract
  m314_dashboard_operator_workflow_depth_contract
  ops_shell_command_center_snapshot_markers
  ops_shell_timeline_chart_snapshot_markers
  ops_shell_timeline_range_invalid_default_regression
  ops_shell_alert_feed_snapshot_markers
  ops_shell_connector_health_snapshot_markers
  ops_shell_control_snapshot_markers
  ops_shell_control_confirmation_payload_markers
  ops_control_action_missing_action_redirect
  ops_control_action_invalid_action_fail_closed
  ops_last_action_reason_row_contract
  dashboard_live_data_stream_matrix_contract
  dashboard_ops_runbook_api_surface_contract
)

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

run_step() {
  local id="$1"
  shift
  local cmd="$*"
  local log_path="${REPORT_DIR}/${id}.log"
  local status="pass"

  echo "==> ${id}"
  if [[ "${MOCK_MODE}" == "1" ]]; then
    if [[ -n "${MOCK_FAIL_PATTERN}" ]] && [[ "${id}" == *"${MOCK_FAIL_PATTERN}"* ]]; then
      status="fail"
    fi
    printf 'mock-mode command: %s\nmock-mode status: %s\n' "${cmd}" "${status}" >"${log_path}"
  else
    if (cd "${ROOT_DIR}" && bash -c "${cmd}") >"${log_path}" 2>&1; then
      status="pass"
    else
      status="fail"
    fi
  fi

  if [[ "${status}" == "fail" ]]; then
    overall="fail"
    echo "    FAIL (${log_path})"
  else
    echo "    PASS (${log_path})"
  fi
  printf '%s|%s|%s|%s\n' "${id}" "${status}" "${log_path}" "${cmd}" >> "${STEPS_TMP}"
}

require_cmd jq
require_cmd python3
require_cmd rg

if [[ "${VERIFY_ONLY}" != "1" ]]; then
  mkdir -p "${REPORT_DIR}"
  : > "${STEPS_TMP}"

  run_step "m308_dashboard_live_mutation_depth_contract" \
    "TAU_M308_CARGO_TARGET_DIR=${TARGET_DIR} bash scripts/verify/m308-dashboard-live-mutation-depth.sh"
  run_step "m314_dashboard_operator_workflow_depth_contract" \
    "TAU_M314_CARGO_TARGET_DIR=${TARGET_DIR} bash scripts/verify/m314-dashboard-operator-workflow-depth.sh"
  run_step "ops_shell_command_center_snapshot_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2806_c01_c02_c03_ops_shell_command_center_markers_reflect_dashboard_snapshot -- --nocapture"
  run_step "ops_shell_timeline_chart_snapshot_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2814_c01_c02_c03_ops_shell_timeline_chart_markers_reflect_snapshot_and_range_query -- --nocapture"
  run_step "ops_shell_timeline_range_invalid_default_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2814_c03_ops_shell_timeline_range_invalid_query_defaults_to_1h -- --nocapture"
  run_step "ops_shell_alert_feed_snapshot_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2818_c01_c02_ops_shell_alert_feed_row_markers_reflect_dashboard_snapshot -- --nocapture"
  run_step "ops_shell_connector_health_snapshot_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2822_c01_c02_ops_shell_connector_health_rows_reflect_multi_channel_connectors -- --nocapture"
  run_step "ops_shell_control_snapshot_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot -- --nocapture"
  run_step "ops_shell_control_confirmation_payload_markers" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway functional_spec_2826_c03_ops_shell_control_markers_include_confirmation_payload -- --nocapture"
  run_step "ops_control_action_missing_action_redirect" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3466_c01_ops_control_action_missing_action_redirects_with_missing_marker -- --nocapture"
  run_step "ops_control_action_invalid_action_fail_closed" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway regression_spec_3466_c02_ops_control_action_invalid_action_fails_closed_with_redirect_marker -- --nocapture"
  run_step "ops_last_action_reason_row_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3482_c03_ops_shell_last_action_reason_row_renders_fixture_reason -- --nocapture"
  run_step "dashboard_live_data_stream_matrix_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3454_c04_d12_dashboard_live_data_stream_matrix -- --nocapture"
  run_step "dashboard_ops_runbook_api_surface_contract" \
    "rg -n 'GET /dashboard/health' docs/guides/dashboard-ops.md && rg -n 'GET /dashboard/widgets' docs/guides/dashboard-ops.md && rg -n 'GET /dashboard/queue-timeline' docs/guides/dashboard-ops.md && rg -n 'GET /dashboard/alerts' docs/guides/dashboard-ops.md && rg -n 'POST /dashboard/actions' docs/guides/dashboard-ops.md && rg -n 'GET /dashboard/stream' docs/guides/dashboard-ops.md"

  python3 - \
    "${GENERATED_AT}" \
    "${overall}" \
    "${REPORT_PATH}" \
    "${STEPS_TMP}" <<'PY'
import json
import sys
from pathlib import Path

generated_at, overall, report_path_raw, steps_tmp_raw = sys.argv[1:]
report_path = Path(report_path_raw)
steps_tmp = Path(steps_tmp_raw)

steps = []
for line in steps_tmp.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    step_id, status, log, command = line.split("|", 3)
    steps.append({"id": step_id, "status": status, "log": log, "command": command})

report = {
    "schema_version": 1,
    "suite_id": "m318_dashboard_command_center_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing dashboard command-center report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m318_dashboard_command_center_depth"' "${REPORT_PATH}" >/dev/null
jq -e '.generated_at | type == "string"' "${REPORT_PATH}" >/dev/null
jq -e '.overall == "pass" or .overall == "fail"' "${REPORT_PATH}" >/dev/null
jq -e '.steps | type == "array"' "${REPORT_PATH}" >/dev/null
jq -e 'all(.steps[]; (.id | type == "string") and (.status == "pass" or .status == "fail") and (.log | type == "string") and (.command | type == "string"))' "${REPORT_PATH}" >/dev/null
jq -e 'if .overall == "pass" then all(.steps[]; .status == "pass") else any(.steps[]; .status == "fail") end' "${REPORT_PATH}" >/dev/null

for step_id in "${required_steps[@]}"; do
  jq -e --arg id "${step_id}" '(.steps | map(select(.id == $id)) | length) == 1' "${REPORT_PATH}" >/dev/null
done

echo "verification report: ${REPORT_PATH}"

if [[ "$(jq -r '.overall' "${REPORT_PATH}")" != "pass" ]]; then
  echo "m318 dashboard command-center depth verification failed"
  exit 1
fi

echo "m318 dashboard command-center depth verification passed: ${REPORT_PATH}"
