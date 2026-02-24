#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M315_REPORT_DIR:-${ROOT_DIR}/artifacts/e2e-operator-route-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M315_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M315_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M315_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M315_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M315_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  ops_memory_route_search_and_empty_state
  ops_memory_scope_filter_contracts
  ops_memory_type_filter_contracts
  ops_memory_create_submission_contracts
  ops_memory_edit_submission_contracts
  ops_memory_delete_submission_contracts
  ops_memory_detail_panel_contracts
  ops_memory_graph_route_contracts
  ops_tools_inventory_contracts
  ops_tools_detail_usage_contracts
  ops_tools_jobs_list_contracts
  ops_channels_health_contracts
  ops_channels_action_contracts
  ops_config_training_safety_diagnostics_routes
  ops_config_route_profile_policy_contracts
  ops_training_route_contracts
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

  run_step "ops_memory_route_search_and_empty_state" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2905_c01_c02_c03_ops_memory_route_renders_relevant_search_results_and_empty_state -- --nocapture"
  run_step "ops_memory_scope_filter_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2909_c01_c02_c03_ops_memory_scope_filters_narrow_results -- --nocapture"
  run_step "ops_memory_type_filter_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2913_c01_c02_c03_ops_memory_type_filter_narrows_results -- --nocapture"
  run_step "ops_memory_create_submission_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2917_c02_c03_ops_memory_create_submission_persists_entry_and_sets_status_markers -- --nocapture"
  run_step "ops_memory_edit_submission_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2921_c02_c03_ops_memory_edit_submission_updates_existing_entry_and_sets_status_markers -- --nocapture"
  run_step "ops_memory_delete_submission_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3060_c02_c03_ops_memory_delete_submission_requires_confirmation_and_deletes_confirmed_entry -- --nocapture"
  run_step "ops_memory_detail_panel_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3064_c02_c03_ops_memory_detail_panel_renders_embedding_and_relation_markers_for_selected_entry -- --nocapture"
  run_step "ops_memory_graph_route_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3068_c02_ops_memory_graph_route_renders_node_and_edge_markers -- --nocapture"
  run_step "ops_tools_inventory_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3106_c02_ops_tools_route_lists_registered_inventory_rows -- --nocapture"
  run_step "ops_tools_detail_usage_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3112_c03_ops_tools_route_renders_tool_detail_usage_contracts -- --nocapture"
  run_step "ops_tools_jobs_list_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3116_c03_ops_tools_route_renders_jobs_list_contracts -- --nocapture"
  run_step "ops_channels_health_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3128_c03_ops_channels_route_renders_channel_health_contracts -- --nocapture"
  run_step "ops_channels_action_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3132_c03_ops_channels_route_renders_channel_action_contracts -- --nocapture"
  run_step "ops_config_training_safety_diagnostics_routes" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3140_c04_ops_routes_render_config_training_safety_diagnostics_panels -- --nocapture"
  run_step "ops_config_route_profile_policy_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3144_c03_ops_config_route_renders_profile_policy_contract_markers -- --nocapture"
  run_step "ops_training_route_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3148_c04_ops_training_route_renders_training_contract_markers -- --nocapture"

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
    "suite_id": "m315_e2e_operator_route_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing operator-route e2e report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m315_e2e_operator_route_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m315 e2e operator-route depth verification failed"
  exit 1
fi

echo "m315 e2e operator-route depth verification passed: ${REPORT_PATH}"
