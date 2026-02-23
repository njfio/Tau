#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${ROOT_DIR}/artifacts/operator-reliability-wave"
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

run_step "restart_events_runtime_recovery" \
  "cargo test -p tau-events integration_restart_recovery_runs_due_oneshot_and_keeps_periodic -- --nocapture"
run_step "reconnect_dashboard_contract_markers" \
  "cargo test -p tau-dashboard-ui spec_c06_stream_contract_declares_reconnect_backoff_strategy -- --nocapture"
run_step "reconnect_gateway_dashboard_stream" \
  "cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates -- --nocapture"
run_step "degraded_training_status_fallback" \
  "cargo test -p tau-gateway regression_spec_2685_c02_training_status_endpoint_returns_unavailable_payload_when_missing -- --nocapture"
run_step "auth_fail_closed_dashboard_endpoints" \
  "cargo test -p tau-gateway regression_dashboard_endpoints_reject_unauthorized_requests -- --nocapture"
run_step "rl_e2e_harness_conformance" \
  "cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture"

{
  echo "{"
  echo "  \"schema_version\": 1,"
  echo "  \"suite_id\": \"m296_integrated_reliability_wave\","
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
  echo "integrated reliability verification failed"
  exit 1
fi

echo "integrated reliability verification passed"
