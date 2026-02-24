#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M311_REPORT_DIR:-${ROOT_DIR}/artifacts/tui-operator-workflow-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M311_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M311_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M311_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M311_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M311_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

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

if [[ "${VERIFY_ONLY}" != "1" ]]; then
  mkdir -p "${REPORT_DIR}"
  : > "${STEPS_TMP}"

  run_step "tui_shell_binary_operator_panels_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui conformance_tui_shell_binary_renders_operator_panels -- --nocapture"
  run_step "tui_shell_renderer_operator_panels_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c01_shell_renderer_includes_all_operator_panels -- --nocapture"
  run_step "tui_operator_shell_auth_training_metrics_functional" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_operator_shell_renderer_includes_auth_rows_and_training_metrics -- --nocapture"
  run_step "tui_shell_live_watch_parse_controls_integration" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui integration_spec_3474_c01_parse_args_accepts_shell_live_watch_mode_controls -- --nocapture"
  run_step "tui_shell_live_watch_zero_iterations_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui regression_spec_3474_c02_parse_args_rejects_shell_live_zero_iterations -- --nocapture"
  run_step "tui_shell_live_watch_marker_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_spec_3474_c03_live_watch_marker_contract_is_deterministic -- --nocapture"
  run_step "tui_shell_live_watch_help_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_spec_3474_c04_help_text_exposes_shell_live_watch_flags -- --nocapture"
  run_step "tui_live_shell_loads_artifacts_functional" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_live_shell_frame_loads_dashboard_and_training_artifacts -- --nocapture"
  run_step "tui_live_shell_malformed_json_diagnostics_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics -- --nocapture"
  run_step "tui_live_shell_malformed_jsonl_diagnostics_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics -- --nocapture"
  run_step "tui_live_shell_missing_artifacts_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui regression_live_shell_frame_handles_missing_artifacts_without_panicking -- --nocapture"
  run_step "tui_operator_shell_control_plane_markers_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c28_regression_operator_shell_status_and_alert_panels_require_control_plane_markers -- --nocapture"
  run_step "tui_operator_shell_auth_markers_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c28_regression_operator_shell_auth_panel_requires_auth_mode_and_required_markers -- --nocapture"

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
    "suite_id": "m311_tui_operator_workflow_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing tui workflow report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m311_tui_operator_workflow_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m311 tui operator workflow depth verification failed"
  exit 1
fi

echo "m311 tui operator workflow depth verification passed: ${REPORT_PATH}"
