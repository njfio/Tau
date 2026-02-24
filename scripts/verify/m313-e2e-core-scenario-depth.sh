#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M313_REPORT_DIR:-${ROOT_DIR}/artifacts/e2e-core-scenario-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M313_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M313_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M313_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M313_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M313_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  integration_workspace_runs_package
  integration_agent_tool_memory_roundtrip_conformance
  integration_isolated_memory_state_regression
  integration_channel_scope_filter_integration
  gateway_e2e_harness_lifecycle_session_flow
  gateway_e2e_harness_dashboard_live_contracts
  gateway_openresponses_http_roundtrip_session_persistence
  gateway_ws_session_status_reset_roundtrip
  gateway_auth_session_lifecycle_conformance
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

  run_step "integration_workspace_runs_package" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-integration-tests integration_spec_2608_c01_workspace_runs_integration_package -- --nocapture"
  run_step "integration_agent_tool_memory_roundtrip_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-integration-tests conformance_spec_2608_c02_agent_tool_memory_roundtrip -- --nocapture"
  run_step "integration_isolated_memory_state_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-integration-tests regression_spec_2608_c03_harness_uses_isolated_memory_state -- --nocapture"
  run_step "integration_channel_scope_filter_integration" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-integration-tests integration_spec_3055_c02_channel_scope_filters_same_workspace_records -- --nocapture"
  run_step "gateway_e2e_harness_lifecycle_session_flow" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow -- --nocapture"
  run_step "gateway_e2e_harness_dashboard_live_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts -- --nocapture"
  run_step "gateway_openresponses_http_roundtrip_session_persistence" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_openresponses_http_roundtrip_persists_session_state -- --nocapture"
  run_step "gateway_ws_session_status_reset_roundtrip" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_gateway_ws_session_status_and_reset_roundtrip -- --nocapture"
  run_step "gateway_auth_session_lifecycle_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture"

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
    "suite_id": "m313_e2e_core_scenario_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing e2e core-depth report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m313_e2e_core_scenario_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m313 e2e core scenario depth verification failed"
  exit 1
fi

echo "m313 e2e core scenario depth verification passed: ${REPORT_PATH}"
