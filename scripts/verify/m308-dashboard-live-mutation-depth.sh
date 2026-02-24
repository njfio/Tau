#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M308_REPORT_DIR:-${ROOT_DIR}/artifacts/dashboard-live-mutation-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M308_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M308_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M308_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M308_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M308_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  dashboard_status_endpoints_matrix
  dashboard_action_mutation_persistence
  dashboard_stream_reconnect_reset_snapshot
  ops_shell_control_action_form_mutation
  ops_shell_dashboard_contract_matrix
  dashboard_unauthorized_fail_closed
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
    if (cd "${ROOT_DIR}" && bash -lc "${cmd}") >"${log_path}" 2>&1; then
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

  run_step "dashboard_status_endpoints_matrix" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_dashboard_endpoints_return_state_health_widgets_timeline_and_alerts -- --nocapture"
  run_step "dashboard_action_mutation_persistence" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_dashboard_action_endpoint_writes_audit_and_updates_control_state -- --nocapture"
  run_step "dashboard_stream_reconnect_reset_snapshot" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates -- --nocapture"
  run_step "ops_shell_control_action_form_mutation" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3466_c03_ops_control_action_form_submits_dashboard_mutation_and_redirects_with_applied_marker -- --nocapture"
  run_step "ops_shell_dashboard_contract_matrix" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts -- --nocapture"
  run_step "dashboard_unauthorized_fail_closed" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway regression_dashboard_endpoints_reject_unauthorized_requests -- --nocapture"

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
    "suite_id": "m308_dashboard_live_mutation_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing dashboard live mutation report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m308_dashboard_live_mutation_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m308 dashboard live mutation depth verification failed"
  exit 1
fi

echo "m308 dashboard live mutation depth verification passed: ${REPORT_PATH}"
