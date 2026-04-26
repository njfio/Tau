#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

LAUNCHER="${REPO_ROOT}/scripts/run/tau-unified.sh"
LAUNCHER_TEST="${REPO_ROOT}/scripts/run/test-tau-unified.sh"
RUN_CONTRACT_TEST="${REPO_ROOT}/scripts/dev/test-prove-tau-product.sh"
GUIDE="${REPO_ROOT}/docs/guides/canonical-product-proof.md"
PRODUCT_PROOF_STATUS_JSON=""
PRODUCT_PROOF_WEBCHAT_HTML=""
PRODUCT_PROOF_SESSIONS_JSON=""
PRODUCT_PROOF_MEMORY_JSON=""
PRODUCT_PROOF_RUNTIME_STARTED="false"
REPORT_PATH=""
WEBCHAT_SMOKE="false"
SESSIONS_SMOKE="false"
MEMORY_SMOKE="false"

usage() {
  cat <<'EOF'
Usage: scripts/dev/prove-tau-product.sh [--check|--run] [--webchat-smoke] [--sessions-smoke] [--memory-smoke] [--report <path>]

Modes:
  --check  Validate the canonical Tau product-proof command surface without starting the real runtime.
    --run    Execute the live product proof: up, status, gateway/status, live-shell, down.
    --webchat-smoke
      With --run, also fetch /webchat and assert stable product-surface markers.
    --sessions-smoke
      With --run, also fetch /gateway/sessions and assert the sessions API JSON shape.
    --memory-smoke
      With --run, also fetch /gateway/memory/default and assert the read JSON shape.
  --report Write a machine-readable JSON evidence report to the given path after success.
  --help   Show this help.
EOF
}

die() {
  echo "error: $1" >&2
  exit 1
}

assert_file() {
  local path="$1"
  [[ -f "${path}" ]] || die "missing file: ${path#"${REPO_ROOT}/"}"
}

assert_executable() {
  local path="$1"
  [[ -x "${path}" ]] || die "missing executable: ${path#"${REPO_ROOT}/"}"
}

assert_contains_file() {
  local path="$1"
  local needle="$2"
  grep -Fq -- "${needle}" "${path}" || die "${path#"${REPO_ROOT}/"} does not contain: ${needle}"
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "required command not found: ${command_name}"
}

prepare_report_parent() {
  local report_path="$1"
  local report_parent
  report_parent="$(dirname "${report_path}")"
  [[ -d "${report_parent}" ]] || mkdir -p "${report_parent}"
}

write_check_report() {
  [[ -n "${REPORT_PATH}" ]] || return 0
  require_command python3
  prepare_report_parent "${REPORT_PATH}"
  python3 - "${REPORT_PATH}" <<'PY'
from __future__ import annotations

import json
import sys

payload = {
    "mode": "check",
    "status": "passed",
    "checks": {
        "guide_contract": "passed",
        "launcher_contract": "passed",
        "run_contract": "passed",
    },
}

with open(sys.argv[1], "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

write_run_report() {
  [[ -n "${REPORT_PATH}" ]] || return 0
  local bind="$1"
  local auth_mode="$2"
  local model="$3"
  local status_url="$4"
  local webchat_url="$5"
  local webchat_smoke="$6"
  local sessions_url="$7"
  local sessions_smoke="$8"
  local memory_url="$9"
  local memory_smoke="${10}"
  require_command python3
  prepare_report_parent "${REPORT_PATH}"
  python3 - "${REPORT_PATH}" "${bind}" "${auth_mode}" "${model}" "${status_url}" "${webchat_url}" "${webchat_smoke}" "${sessions_url}" "${sessions_smoke}" "${memory_url}" "${memory_smoke}" <<'PY'
from __future__ import annotations

import json
import sys

_, report_path, bind, auth_mode, model, status_url, webchat_url, webchat_smoke, sessions_url, sessions_smoke, memory_url, memory_smoke = sys.argv
completed_steps = ["up", "status", "gateway_status"]

payload = {
    "mode": "run",
    "status": "passed",
    "bind": bind,
    "auth_mode": auth_mode,
    "model": model,
    "gateway_status_url": status_url,
}

if webchat_smoke == "true":
    payload["webchat_url"] = webchat_url
    completed_steps.append("webchat")

if sessions_smoke == "true":
  payload["gateway_sessions_url"] = sessions_url
  completed_steps.append("sessions_api")

if memory_smoke == "true":
  payload["gateway_memory_url"] = memory_url
  completed_steps.append("memory_api")

completed_steps.extend(["tui", "down"])
payload["completed_steps"] = completed_steps

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

run_check() {
  assert_executable "${LAUNCHER}"
  assert_file "${LAUNCHER_TEST}"
  assert_file "${RUN_CONTRACT_TEST}"
  assert_file "${GUIDE}"

  bash -n "${LAUNCHER}"
  bash -n "${LAUNCHER_TEST}"
  bash -n "${RUN_CONTRACT_TEST}"

  assert_contains_file "${GUIDE}" "./scripts/run/tau-unified.sh up --auth-mode localhost-dev"
  assert_contains_file "${GUIDE}" "./scripts/run/tau-unified.sh status"
  assert_contains_file "${GUIDE}" "curl -sS http://127.0.0.1:8791/gateway/status | jq"
  assert_contains_file "${GUIDE}" "./scripts/run/tau-unified.sh tui --live-shell --iterations 1 --interval-ms 1000 --no-color"
  assert_contains_file "${GUIDE}" "./scripts/run/tau-unified.sh down"

  bash "${LAUNCHER_TEST}"
  bash "${RUN_CONTRACT_TEST}"

  write_check_report
  echo "Tau product-proof check passed: launcher contract and canonical guide are in sync"
}

validate_gateway_status_json() {
  local json_path="$1"
  if command -v jq >/dev/null 2>&1; then
    jq -e 'type == "object"' "${json_path}" >/dev/null || die "gateway/status response is not a JSON object"
  else
    require_command python3
    python3 - "${json_path}" <<'PY'
from __future__ import annotations

import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

if not isinstance(payload, dict):
    raise SystemExit("gateway/status response is not a JSON object")
PY
  fi
}

validate_webchat_page() {
  local html_path="$1"
  local marker
  for marker in "Tau Gateway Webchat" "Dashboard" "dashboardStatus"; do
    grep -Fq -- "${marker}" "${html_path}" || die "webchat response missing expected marker: ${marker}"
  done
}

validate_gateway_sessions_json() {
  local json_path="$1"
  if command -v jq >/dev/null 2>&1; then
    jq -e 'type == "object" and (.sessions | type == "array")' "${json_path}" >/dev/null || die "gateway sessions response is not a JSON object with sessions array"
  else
    require_command python3
    python3 - "${json_path}" <<'PY'
from __future__ import annotations

import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

if not isinstance(payload, dict) or not isinstance(payload.get("sessions"), list):
    raise SystemExit("gateway sessions response is not a JSON object with sessions array")
PY
  fi
}

validate_gateway_memory_json() {
  local json_path="$1"
  if command -v jq >/dev/null 2>&1; then
    jq -e 'type == "object" and (.exists | type == "boolean")' "${json_path}" >/dev/null || die "gateway memory response is not a JSON object with exists boolean"
  else
    require_command python3
    python3 - "${json_path}" <<'PY'
from __future__ import annotations

import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

if not isinstance(payload, dict) or not isinstance(payload.get("exists"), bool):
    raise SystemExit("gateway memory response is not a JSON object with exists boolean")
PY
  fi
}

run_live() {
  require_command cargo

  assert_executable "${LAUNCHER}"

  local bind="${TAU_UNIFIED_BIND:-127.0.0.1:8791}"
  local auth_mode="${TAU_UNIFIED_AUTH_MODE:-localhost-dev}"
  local model="${TAU_UNIFIED_MODEL:-gpt-5.3-codex}"
  local curl_bin="${TAU_PRODUCT_PROOF_CURL_BIN:-curl}"
  local status_attempts="${TAU_PRODUCT_PROOF_STATUS_ATTEMPTS:-30}"
  local status_retry_delay="${TAU_PRODUCT_PROOF_STATUS_RETRY_DELAY:-1}"
  local status_url="http://${bind}/gateway/status"
  local webchat_url="http://${bind}/webchat"
  local sessions_url="http://${bind}/gateway/sessions"
  local memory_url="http://${bind}/gateway/memory/default"
  local status_json
  status_json="$(mktemp)"
  local webchat_html
  webchat_html="$(mktemp)"
  local sessions_json
  sessions_json="$(mktemp)"
  local memory_json
  memory_json="$(mktemp)"
  PRODUCT_PROOF_STATUS_JSON="${status_json}"
  PRODUCT_PROOF_WEBCHAT_HTML="${webchat_html}"
  PRODUCT_PROOF_SESSIONS_JSON="${sessions_json}"
  PRODUCT_PROOF_MEMORY_JSON="${memory_json}"
  PRODUCT_PROOF_RUNTIME_STARTED="false"

  require_command "${curl_bin}"

  cleanup() {
    if [[ -n "${PRODUCT_PROOF_STATUS_JSON}" ]]; then
      rm -f "${PRODUCT_PROOF_STATUS_JSON}"
    fi
    if [[ -n "${PRODUCT_PROOF_WEBCHAT_HTML}" ]]; then
      rm -f "${PRODUCT_PROOF_WEBCHAT_HTML}"
    fi
    if [[ -n "${PRODUCT_PROOF_SESSIONS_JSON}" ]]; then
      rm -f "${PRODUCT_PROOF_SESSIONS_JSON}"
    fi
    if [[ -n "${PRODUCT_PROOF_MEMORY_JSON}" ]]; then
      rm -f "${PRODUCT_PROOF_MEMORY_JSON}"
    fi
    if [[ "${PRODUCT_PROOF_RUNTIME_STARTED}" == "true" ]]; then
      "${LAUNCHER}" down >/dev/null 2>&1 || true
    fi
  }
  trap cleanup EXIT INT TERM

  "${LAUNCHER}" up --auth-mode "${auth_mode}" --bind "${bind}" --model "${model}"
  PRODUCT_PROOF_RUNTIME_STARTED="true"

  "${LAUNCHER}" status

  local attempt
  for ((attempt = 1; attempt <= status_attempts; attempt += 1)); do
    if "${curl_bin}" -fsS "${status_url}" >"${status_json}"; then
      break
    fi
    if [[ "${attempt}" == "${status_attempts}" ]]; then
      die "gateway status endpoint did not become ready: ${status_url}"
    fi
    sleep "${status_retry_delay}"
  done
  validate_gateway_status_json "${status_json}"

  if [[ "${WEBCHAT_SMOKE}" == "true" ]]; then
    "${curl_bin}" -fsS "${webchat_url}" >"${webchat_html}" || die "webchat endpoint did not respond: ${webchat_url}"
    validate_webchat_page "${webchat_html}"
  fi

  if [[ "${SESSIONS_SMOKE}" == "true" ]]; then
    "${curl_bin}" -fsS "${sessions_url}" >"${sessions_json}" || die "gateway sessions endpoint did not respond: ${sessions_url}"
    validate_gateway_sessions_json "${sessions_json}"
  fi

  if [[ "${MEMORY_SMOKE}" == "true" ]]; then
    "${curl_bin}" -fsS "${memory_url}" >"${memory_json}" || die "gateway memory endpoint did not respond: ${memory_url}"
    validate_gateway_memory_json "${memory_json}"
  fi

  "${LAUNCHER}" tui --live-shell --iterations 1 --interval-ms 1000 --no-color

  "${LAUNCHER}" down
  PRODUCT_PROOF_RUNTIME_STARTED="false"
  trap - EXIT INT TERM
  rm -f "${status_json}"
  rm -f "${webchat_html}"
  rm -f "${sessions_json}"
  rm -f "${memory_json}"
  PRODUCT_PROOF_STATUS_JSON=""
  PRODUCT_PROOF_WEBCHAT_HTML=""
  PRODUCT_PROOF_SESSIONS_JSON=""
  PRODUCT_PROOF_MEMORY_JSON=""

  write_run_report "${bind}" "${auth_mode}" "${model}" "${status_url}" "${webchat_url}" "${WEBCHAT_SMOKE}" "${sessions_url}" "${SESSIONS_SMOKE}" "${memory_url}" "${MEMORY_SMOKE}"
  echo "Tau product proof passed: runtime up/status/gateway/live-shell/down completed"
}

mode="--check"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --check|--run)
      mode="$1"
      shift
      ;;
    --report)
      shift
      [[ $# -gt 0 ]] || die "--report requires a path"
      REPORT_PATH="$1"
      shift
      ;;
    --webchat-smoke)
      WEBCHAT_SMOKE="true"
      shift
      ;;
    --sessions-smoke)
      SESSIONS_SMOKE="true"
      shift
      ;;
    --memory-smoke)
      MEMORY_SMOKE="true"
      shift
      ;;
    --help|-h)
      mode="--help"
      shift
      ;;
    *)
      usage >&2
      die "unknown argument: $1"
      ;;
  esac
done

if [[ "${WEBCHAT_SMOKE}" == "true" && "${mode}" != "--run" ]]; then
  die "--webchat-smoke requires --run"
fi

if [[ "${SESSIONS_SMOKE}" == "true" && "${mode}" != "--run" ]]; then
  die "--sessions-smoke requires --run"
fi

if [[ "${MEMORY_SMOKE}" == "true" && "${mode}" != "--run" ]]; then
  die "--memory-smoke requires --run"
fi

case "${mode}" in
  --check)
    run_check
    ;;
  --run)
    run_live
    ;;
  --help|-h)
    usage
    ;;
  *)
    usage >&2
    die "unknown mode: ${mode}"
    ;;
esac