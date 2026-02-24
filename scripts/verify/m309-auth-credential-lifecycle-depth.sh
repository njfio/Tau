#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M309_REPORT_DIR:-${ROOT_DIR}/artifacts/auth-credential-lifecycle-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M309_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M309_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M309_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M309_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M309_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  provider_auth_workflow_conformance
  gateway_auth_session_lifecycle_conformance
  integration_auth_set_status_rotate_revoke_lifecycle
  integration_auth_status_totals_with_filter
  integration_auth_status_empty_store_regression
  resolve_secret_reads_store_secret
  resolve_secret_rejects_revoked_secret
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
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-provider --test auth_workflow_conformance -- --nocapture"
  run_step "gateway_auth_session_lifecycle_conformance" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture"
  run_step "integration_auth_set_status_rotate_revoke_lifecycle" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-coding-agent functional_execute_integration_auth_command_set_status_rotate_revoke_lifecycle -- --nocapture"
  run_step "integration_auth_status_totals_with_filter" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-coding-agent functional_execute_integration_auth_command_status_reports_totals_with_filter -- --nocapture"
  run_step "integration_auth_status_empty_store_regression" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-coding-agent regression_execute_integration_auth_command_status_handles_empty_store -- --nocapture"
  run_step "resolve_secret_reads_store_secret" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-coding-agent functional_resolve_secret_from_cli_or_store_id_reads_integration_secret -- --nocapture"
  run_step "resolve_secret_rejects_revoked_secret" \
    "CARGO_TARGET_DIR=${TARGET_DIR} cargo test -p tau-coding-agent regression_resolve_secret_from_cli_or_store_id_rejects_revoked_secret -- --nocapture"

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
    "suite_id": "m309_auth_credential_lifecycle_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing auth credential lifecycle report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m309_auth_credential_lifecycle_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m309 auth credential lifecycle depth verification failed"
  exit 1
fi

echo "m309 auth credential lifecycle depth verification passed: ${REPORT_PATH}"
