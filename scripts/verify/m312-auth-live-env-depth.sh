#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M312_REPORT_DIR:-${ROOT_DIR}/artifacts/auth-live-env-depth}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="${TAU_M312_GENERATED_AT:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"
MOCK_MODE="${TAU_M312_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M312_MOCK_FAIL_PATTERN:-}"
VERIFY_ONLY="${TAU_M312_VERIFY_ONLY:-0}"
TARGET_DIR="${TAU_M312_CARGO_TARGET_DIR:-target-fast}"
overall="pass"

required_steps=(
  m296_live_auth_validation_contract
  m303_auth_workflow_depth_contract
  m303_auth_workflow_depth_suite
  m309_auth_credential_lifecycle_contract
  m309_auth_credential_lifecycle_suite
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

  run_step "m296_live_auth_validation_contract" \
    "bash scripts/verify/test-m296-live-auth-validation.sh"
  run_step "m303_auth_workflow_depth_contract" \
    "bash scripts/verify/test-m303-auth-workflow-depth.sh"
  run_step "m303_auth_workflow_depth_suite" \
    "CARGO_TARGET_DIR=${TARGET_DIR} bash scripts/verify/m303-auth-workflow-depth.sh"
  run_step "m309_auth_credential_lifecycle_contract" \
    "bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh"
  run_step "m309_auth_credential_lifecycle_suite" \
    "CARGO_TARGET_DIR=${TARGET_DIR} bash scripts/verify/m309-auth-credential-lifecycle-depth.sh"

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
    "suite_id": "m312_auth_live_env_depth",
    "generated_at": generated_at,
    "overall": overall,
    "steps": steps,
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY
  rm -f "${STEPS_TMP}"
fi

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "error: missing auth live env depth report: ${REPORT_PATH}" >&2
  exit 1
fi

jq -e '.schema_version == 1' "${REPORT_PATH}" >/dev/null
jq -e '.suite_id == "m312_auth_live_env_depth"' "${REPORT_PATH}" >/dev/null
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
  echo "m312 auth live env depth verification failed"
  exit 1
fi

echo "m312 auth live env depth verification passed: ${REPORT_PATH}"
