#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M303_REPORT_DIR:-${ROOT_DIR}/artifacts/auth-workflow-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M303_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M303_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M303_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M303_VERIFY_ONLY:-0}"
overall="pass"

required_steps=(
  provider_auth_workflow_conformance
  gateway_bootstrap_token_mode_contract
  gateway_bootstrap_localhost_dev_mode_contract
  gateway_bootstrap_password_session_mode_contract
  gateway_auth_session_lifecycle_conformance
  gateway_localhost_dev_allows_no_bearer
  gateway_invalid_password_rejected
  gateway_mode_mismatch_rejected
  gateway_malformed_json_rejected
  gateway_lowercase_bearer_accepted
  gateway_password_session_expiry_fail_closed
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

  run_step "provider_auth_workflow_conformance" \
    "cargo test -p tau-provider --test auth_workflow_conformance -- --nocapture"
  run_step "gateway_bootstrap_token_mode_contract" \
    "cargo test -p tau-gateway functional_spec_2786_c01_gateway_auth_bootstrap_endpoint_reports_token_mode_contract -- --nocapture"
  run_step "gateway_bootstrap_localhost_dev_mode_contract" \
    "cargo test -p tau-gateway functional_spec_2786_c02_gateway_auth_bootstrap_maps_localhost_dev_to_none_mode -- --nocapture"
  run_step "gateway_bootstrap_password_session_mode_contract" \
    "cargo test -p tau-gateway functional_spec_3426_c02_gateway_auth_bootstrap_reports_password_session_mode_contract -- --nocapture"
  run_step "gateway_auth_session_lifecycle_conformance" \
    "cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture"
  run_step "gateway_localhost_dev_allows_no_bearer" \
    "cargo test -p tau-gateway integration_localhost_dev_mode_allows_requests_without_bearer_token -- --nocapture"
  run_step "gateway_invalid_password_rejected" \
    "cargo test -p tau-gateway regression_gateway_auth_session_rejects_invalid_password -- --nocapture"
  run_step "gateway_mode_mismatch_rejected" \
    "cargo test -p tau-gateway regression_spec_3426_c06_gateway_auth_session_rejects_mode_mismatch -- --nocapture"
  run_step "gateway_malformed_json_rejected" \
    "cargo test -p tau-gateway regression_spec_3426_c07_gateway_auth_session_rejects_malformed_json -- --nocapture"
  run_step "gateway_lowercase_bearer_accepted" \
    "cargo test -p tau-gateway regression_spec_3426_c08_gateway_accepts_lowercase_bearer_authorization_scheme -- --nocapture"
  run_step "gateway_password_session_expiry_fail_closed" \
    "cargo test -p tau-gateway regression_gateway_password_session_token_expires_and_fails_closed -- --nocapture"

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
    "suite_id": "m303_auth_workflow_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing auth workflow report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m303_auth_workflow_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m303 auth workflow depth verification failed"
  exit 1
fi

echo "m303 auth workflow depth verification passed: ${REPORT_PATH}"
