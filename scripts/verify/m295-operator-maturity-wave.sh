#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${ROOT_DIR}/artifacts/operator-maturity-wave"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"

mkdir -p "${REPORT_DIR}"
: > "${STEPS_TMP}"

overall="pass"

run_step() {
  local id="$1"
  shift
  local cmd="$*"
  local log_path="${REPORT_DIR}/${id}.log"

  echo "==> ${id}"
  if (cd "${ROOT_DIR}" && bash -lc "${cmd}") >"${log_path}" 2>&1; then
    echo "    PASS (${log_path})"
    printf '%s|pass|%s\n' "${id}" "${log_path}" >> "${STEPS_TMP}"
  else
    echo "    FAIL (${log_path})"
    overall="fail"
    printf '%s|fail|%s\n' "${id}" "${log_path}" >> "${STEPS_TMP}"
  fi
}

run_step "tui_shell_conformance" \
  "cargo test -p tau-tui spec_c01_shell_renderer_includes_all_operator_panels -- --nocapture"
run_step "rl_e2e_conformance" \
  "cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture"
run_step "rl_e2e_harness_binary" \
  "cargo run -p tau-trainer --bin rl_e2e_harness -- --run-id m295-verification --output-dir artifacts/rl-e2e"
run_step "auth_provider_matrix" \
  "cargo test -p tau-provider --test auth_workflow_conformance -- --nocapture"
run_step "auth_coding_agent_matrix" \
  "cargo test -p tau-coding-agent auth_provider::auth_and_provider::spec_c04_auth_matrix_accepts_openrouter_provider_filter -- --nocapture"
run_step "auth_gateway_session_lifecycle" \
  "cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture"

{
  echo "{"
  echo "  \"schema_version\": 1,"
  echo "  \"suite_id\": \"m295_operator_maturity_wave\","
  echo "  \"overall\": \"${overall}\","
  echo "  \"steps\": ["
  index=0
  total_steps="$(wc -l < "${STEPS_TMP}" | tr -d ' ')"
  while IFS='|' read -r step_id step_status step_log; do
    index=$((index + 1))
    comma=","
    if [[ "${index}" -eq "${total_steps}" ]]; then
      comma=""
    fi
    echo "    {"
    echo "      \"id\": \"${step_id}\","
    echo "      \"status\": \"${step_status}\","
    echo "      \"log\": \"${step_log}\""
    echo "    }${comma}"
  done < "${STEPS_TMP}"
  echo "  ]"
  echo "}"
} > "${REPORT_PATH}"

rm -f "${STEPS_TMP}"
echo "verification report: ${REPORT_PATH}"

if [[ "${overall}" != "pass" ]]; then
  echo "operator maturity verification failed"
  exit 1
fi

echo "operator maturity verification passed"
