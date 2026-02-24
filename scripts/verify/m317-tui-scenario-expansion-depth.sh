#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M317_REPORT_DIR:-${ROOT_DIR}/artifacts/tui-scenario-expansion-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M317_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M317_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M317_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M317_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M317_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  m311_tui_operator_workflow_depth_contract
  tui_demo_single_frame_functional
  tui_demo_multi_frame_integration
  tui_demo_invalid_frames_regression
  tui_parse_shell_mode_overrides_spec
  tui_parse_shell_profile_missing_value_spec
  tui_parse_shell_live_mode_state_dir_spec
  tui_parse_shell_live_state_dir_missing_value_regression
  tui_shell_live_watch_parse_controls_integration
  tui_shell_live_watch_zero_iterations_regression
  tui_shell_live_watch_help_contract
  quickstart_tui_shell_live_watch_docs_contract
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

  run_step "m311_tui_operator_workflow_depth_contract" \
    "TAU_M311_CARGO_TARGET_DIR=${TARGET_DIR} bash scripts/verify/m311-tui-operator-workflow-depth.sh"
  run_step "tui_demo_single_frame_functional" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_tui_demo_binary_renders_single_frame_without_color -- --nocapture"
  run_step "tui_demo_multi_frame_integration" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui integration_tui_demo_binary_renders_multiple_frames -- --nocapture"
  run_step "tui_demo_invalid_frames_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui regression_tui_demo_binary_rejects_invalid_frames_argument -- --nocapture"
  run_step "tui_parse_shell_mode_overrides_spec" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c01_parse_args_accepts_shell_mode_and_overrides -- --nocapture"
  run_step "tui_parse_shell_profile_missing_value_spec" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c02_parse_args_rejects_shell_profile_without_value -- --nocapture"
  run_step "tui_parse_shell_live_mode_state_dir_spec" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui spec_c03_parse_args_accepts_shell_live_mode_and_state_dir -- --nocapture"
  run_step "tui_parse_shell_live_state_dir_missing_value_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui regression_parse_args_rejects_shell_live_state_dir_without_value -- --nocapture"
  run_step "tui_shell_live_watch_parse_controls_integration" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui integration_spec_3474_c01_parse_args_accepts_shell_live_watch_mode_controls -- --nocapture"
  run_step "tui_shell_live_watch_zero_iterations_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui regression_spec_3474_c02_parse_args_rejects_shell_live_zero_iterations -- --nocapture"
  run_step "tui_shell_live_watch_help_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-tui functional_spec_3474_c04_help_text_exposes_shell_live_watch_flags -- --nocapture"
  run_step "quickstart_tui_shell_live_watch_docs_contract" \
    "rg -n 'cargo run -p tau-tui -- --frames' docs/guides/quickstart.md && rg -n 'shell-live .*--watch' README.md"

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
    "suite_id": "m317_tui_scenario_expansion_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing tui scenario-expansion report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m317_tui_scenario_expansion_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m317 tui scenario expansion depth verification failed"
  exit 1
fi

echo "m317 tui scenario expansion depth verification passed: ${REPORT_PATH}"
