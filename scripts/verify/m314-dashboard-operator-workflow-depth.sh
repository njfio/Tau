#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M314_REPORT_DIR:-${ROOT_DIR}/artifacts/dashboard-operator-workflow-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M314_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M314_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M314_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M314_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M314_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

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

  run_step "ops_chat_selector_syncs_sessions" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2834_c02_c03_ops_chat_selector_syncs_discovered_sessions_and_active_state -- --nocapture"
  run_step "ops_sessions_shell_row_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2838_c02_c03_ops_sessions_shell_renders_discovered_rows_and_chat_links -- --nocapture"
  run_step "ops_session_detail_lineage_usage_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2842_c02_c04_ops_session_detail_shell_renders_lineage_rows_and_usage_markers -- --nocapture"
  run_step "ops_session_detail_graph_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2846_c02_c03_ops_session_detail_shell_renders_graph_node_and_edge_rows -- --nocapture"
  run_step "ops_chat_new_session_redirect_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2872_c02_c03_c04_ops_chat_new_session_creates_redirect_and_preserves_hidden_panel_contracts -- --nocapture"
  run_step "ops_sessions_branch_lineage_flow" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2885_c02_c03_c04_ops_sessions_branch_creates_lineage_derived_target_session -- --nocapture"
  run_step "ops_session_reset_confirmation_flow" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2889_c02_c03_c04_ops_session_detail_post_reset_clears_target_and_preserves_other_sessions -- --nocapture"
  run_step "ops_session_detail_non_empty_message_coverage" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2897_c01_c02_c04_ops_session_detail_renders_complete_non_empty_message_coverage -- --nocapture"
  run_step "ops_memory_graph_selected_node_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3086_c02_ops_memory_graph_selected_node_shows_detail_panel_contracts -- --nocapture"
  run_step "ops_tools_inventory_contracts" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3106_c02_ops_tools_route_lists_registered_inventory_rows -- --nocapture"
  run_step "ops_last_action_readable_rows_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_3478_c03_ops_shell_last_action_section_renders_readable_rows -- --nocapture"

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
    "suite_id": "m314_dashboard_operator_workflow_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing dashboard operator workflow report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m314_dashboard_operator_workflow_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m314 dashboard operator workflow depth verification failed"
  exit 1
fi

echo "m314 dashboard operator workflow depth verification passed: ${REPORT_PATH}"
