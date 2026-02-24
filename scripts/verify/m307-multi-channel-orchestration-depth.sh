#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M307_REPORT_DIR:-${ROOT_DIR}/artifacts/multi-channel-orchestration-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M307_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M307_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M307_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M307_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M307_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  c5_01_c5_02_c5_07_live_runner_routing_matrix
  c5_03_whatsapp_webhook_valid_signature
  c5_04_whatsapp_webhook_invalid_signature
  c5_05_c5_06_gateway_channel_lifecycle_contract
  c5_05_c5_06_lifecycle_state_roundtrip
  c5_08_media_attachment_handling
  c5_08_media_attachment_bounds_regression
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

  run_step "c5_01_c5_02_c5_07_live_runner_routing_matrix" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel integration_spec_3402_c01_c02_c07_live_runner_routes_telegram_and_discord_to_distinct_sessions -- --nocapture"
  run_step "c5_03_whatsapp_webhook_valid_signature" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel integration_whatsapp_webhook_ingests_signed_cloud_payload -- --nocapture"
  run_step "c5_04_whatsapp_webhook_invalid_signature" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel regression_whatsapp_webhook_rejects_invalid_signature -- --nocapture"
  run_step "c5_05_c5_06_gateway_channel_lifecycle_contract" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway integration_spec_2670_c01_channel_lifecycle_endpoint_supports_logout_and_status_contract -- --nocapture"
  run_step "c5_05_c5_06_lifecycle_state_roundtrip" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel integration_login_status_logout_probe_flow_roundtrips_channel_state -- --nocapture"
  run_step "c5_08_media_attachment_handling" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel integration_runner_media_understanding_enriches_context_and_logs_reason_codes -- --nocapture"
  run_step "c5_08_media_attachment_bounds_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-multi-channel regression_runner_media_understanding_is_bounded_and_duplicate_event_idempotent -- --nocapture"

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
    "suite_id": "m307_multi_channel_orchestration_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing multi-channel orchestration report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m307_multi_channel_orchestration_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m307 multi-channel orchestration depth verification failed"
  exit 1
fi

echo "m307 multi-channel orchestration depth verification passed: ${REPORT_PATH}"
