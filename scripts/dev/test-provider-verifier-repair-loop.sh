#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FIXTURE_PATH="${REPO_ROOT}/tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
TASK_ID="repo_spec_to_pr_feature_delivery"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3798-repair-loop-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-provider-repair-loop.XXXXXX")"
trap 'rm -rf "${WORK_DIR}"' EXIT

export CARGO_TARGET_DIR="${TARGET_DIR}"

load_repo_dotenv_if_present() {
  local dotenv_path="${REPO_ROOT}/.env"
  [[ -f "${dotenv_path}" ]] || return 0

  local line key value
  while IFS= read -r line || [[ -n "${line}" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [[ -n "${line}" && "${line}" != \#* ]] || continue
    if [[ "${line}" == export[[:space:]]* ]]; then
      line="${line#export}"
      line="${line#"${line%%[![:space:]]*}"}"
    fi
    [[ "${line}" == *=* ]] || continue
    key="${line%%=*}"
    value="${line#*=}"
    key="${key%"${key##*[![:space:]]}"}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    [[ "${key}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue
    [[ -z "${!key:-}" ]] || continue
    if [[ "${#value}" -ge 2 ]]; then
      if [[ "${value}" == \"*\" && "${value}" == *\" ]]; then
        value="${value:1:${#value}-2}"
      elif [[ "${value}" == \'*\' && "${value}" == *\' ]]; then
        value="${value:1:${#value}-2}"
      fi
    fi
    export "${key}=${value}"
  done <"${dotenv_path}"
}

save_report() {
  local source_path="$1"
  local name="$2"
  if [[ -n "${TAU_PROVIDER_PROOF_REPORT_DIR:-}" ]]; then
    mkdir -p "${TAU_PROVIDER_PROOF_REPORT_DIR}"
    cp "${source_path}" "${TAU_PROVIDER_PROOF_REPORT_DIR}/${name}"
  fi
}

load_repo_dotenv_if_present

MODEL="${TAU_PROVIDER_PROOF_MODEL:-openai/gpt-4.1-mini}"
AUTH_MODE="${TAU_PROVIDER_PROOF_AUTH_MODE:-api-key}"
MAX_TOKENS="${TAU_PROVIDER_PROOF_MAX_TOKENS:-1024}"
LIVE_REPAIR_ENABLED="${TAU_LIVE_PROVIDER_REPAIR_PROOF:-0}"

REPO="${WORK_DIR}/repo"
STATE="${WORK_DIR}/state"
REPORT="${WORK_DIR}/repair-report.json"
EXHAUSTED_REPO="${WORK_DIR}/exhausted-repo"
EXHAUSTED_STATE="${WORK_DIR}/exhausted-state"
EXHAUSTED_REPORT="${WORK_DIR}/exhausted-report.json"

mkdir -p "${REPO}"
git -C "${REPO}" init -b master >/dev/null
git -C "${REPO}" config user.email tau-repair@example.test
git -C "${REPO}" config user.name "Tau Repair Test"
printf 'fail\n' >"${REPO}/status.txt"
printf 'missing\n' >"${REPO}/notes.txt"
git -C "${REPO}" add .
git -C "${REPO}" commit -m "Seed provider repair fixture" >/dev/null

cd "${REPO_ROOT}"

cargo run -q -p tau-coding-agent --bin tau_live_coding_loop_harness -- \
  --fixture "${FIXTURE_PATH}" \
  --task-id "${TASK_ID}" \
  --mode real-repo \
  --state-root "${STATE}" \
  --repo-root "${REPO}" \
  --output "${REPORT}" \
  --run-id "m334-provider-repair-loop" \
  --started-unix-ms 1800000700000 \
  --provider-model openai/test-model \
  --provider-auth-mode api-key \
  --provider-repair-attempts 1 \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q repaired notes.txt" \
  --mock-provider-response '{"relative_path":"status.txt","contents":"pass\n","reason_code":"provider_first_attempt"}' \
  --mock-provider-response '{"relative_path":"notes.txt","contents":"repaired\n","reason_code":"provider_repair_attempt"}' >/dev/null

python3 - "${REPORT}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    report = json.load(handle)

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["passed"] is True, report)
require(report["phase"] == "pr_ready", report["phase"])
require(report["commit_hash"] and len(report["commit_hash"]) == 40, report["commit_hash"])
require(report["pr_ready"]["status"] == "manual_ready", report["pr_ready"])
require(set(report["changed_files"]) == {"status.txt", "notes.txt"}, report["changed_files"])
require(report["provider"]["edit_relative_path"] == "notes.txt", report["provider"])

attempts = report["provider_attempts"]
require(len(attempts) == 2, attempts)
require(attempts[0]["attempt_index"] == 1, attempts)
require(attempts[0]["repair_context_included"] is False, attempts[0])
require(attempts[0]["edit_relative_path"] == "status.txt", attempts[0])
require(attempts[1]["attempt_index"] == 2, attempts)
require(attempts[1]["repair_context_included"] is True, attempts[1])
require(attempts[1]["failed_verifier_count"] >= 1, attempts[1])
require(attempts[1]["diff_context_bytes"] and attempts[1]["diff_context_bytes"] > 0, attempts[1])
require(attempts[1]["edit_relative_path"] == "notes.txt", attempts[1])

statuses = [item["status"] for item in report["verifier_transcript"]]
require(statuses.count("failed") >= 2, statuses)
require(statuses.count("succeeded") >= 3, statuses)
PY

save_report "${REPORT}" "provider-repair-report.json"
echo "provider verifier repair loop passed: ${REPORT}"

mkdir -p "${EXHAUSTED_REPO}"
git -C "${EXHAUSTED_REPO}" init -b master >/dev/null
git -C "${EXHAUSTED_REPO}" config user.email tau-repair@example.test
git -C "${EXHAUSTED_REPO}" config user.name "Tau Repair Test"
printf 'fail\n' >"${EXHAUSTED_REPO}/status.txt"
printf 'missing\n' >"${EXHAUSTED_REPO}/notes.txt"
git -C "${EXHAUSTED_REPO}" add .
git -C "${EXHAUSTED_REPO}" commit -m "Seed exhausted provider repair fixture" >/dev/null

if cargo run -q -p tau-coding-agent --bin tau_live_coding_loop_harness -- \
  --fixture "${FIXTURE_PATH}" \
  --task-id "${TASK_ID}" \
  --mode real-repo \
  --state-root "${EXHAUSTED_STATE}" \
  --repo-root "${EXHAUSTED_REPO}" \
  --output "${EXHAUSTED_REPORT}" \
  --run-id "m334-provider-repair-loop-exhausted" \
  --started-unix-ms 1800000800000 \
  --provider-model openai/test-model \
  --provider-auth-mode api-key \
  --provider-repair-attempts 0 \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q repaired notes.txt" \
  --mock-provider-response '{"relative_path":"status.txt","contents":"pass\n","reason_code":"provider_first_attempt"}' >/dev/null
then
  echo "expected exhausted provider repair loop to exit nonzero" >&2
  exit 1
fi

python3 - "${EXHAUSTED_REPORT}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    report = json.load(handle)

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["passed"] is False, report)
require(report["phase"] == "blocked", report["phase"])
require(report["blocked_reason"] == "provider_repair_attempts_exhausted", report["blocked_reason"])
require(report["commit_hash"] is None, report["commit_hash"])
require(report["pr_ready"] is None, report["pr_ready"])
attempts = report["provider_attempts"]
require(len(attempts) == 1, attempts)
require(attempts[0]["attempt_index"] == 1, attempts[0])
require(attempts[0]["repair_context_included"] is False, attempts[0])
require(attempts[0]["edit_relative_path"] == "status.txt", attempts[0])
statuses = [item["status"] for item in report["verifier_transcript"]]
require(statuses.count("failed") >= 2, statuses)
require(statuses.count("succeeded") >= 1, statuses)
PY

save_report "${EXHAUSTED_REPORT}" "provider-repair-exhausted-report.json"
echo "provider verifier repair loop exhausted safely: ${EXHAUSTED_REPORT}"

if [[ "${LIVE_REPAIR_ENABLED}" != "1" ]]; then
  echo "provider live repair proof skipped; set TAU_LIVE_PROVIDER_REPAIR_PROOF=1 to run provider-backed repair"
  exit 0
fi

LIVE_REPO="${WORK_DIR}/live-repair-repo"
LIVE_STATE="${WORK_DIR}/live-repair-state"
LIVE_REPORT="${WORK_DIR}/live-repair-report.json"

mkdir -p "${LIVE_REPO}"
git -C "${LIVE_REPO}" init -b master >/dev/null
git -C "${LIVE_REPO}" config user.email tau-repair@example.test
git -C "${LIVE_REPO}" config user.name "Tau Repair Test"
printf 'fail\n' >"${LIVE_REPO}/status.txt"
printf 'missing\n' >"${LIVE_REPO}/notes.txt"
git -C "${LIVE_REPO}" add .
git -C "${LIVE_REPO}" commit -m "Seed live provider repair fixture" >/dev/null

set +e
cargo run -q -p tau-coding-agent --bin tau_live_coding_loop_harness -- \
  --fixture "${FIXTURE_PATH}" \
  --task-id "${TASK_ID}" \
  --mode real-repo \
  --state-root "${LIVE_STATE}" \
  --repo-root "${LIVE_REPO}" \
  --output "${LIVE_REPORT}" \
  --run-id "m334-provider-live-repair-loop" \
  --started-unix-ms 1800000900000 \
  --provider-model "${MODEL}" \
  --provider-auth-mode "${AUTH_MODE}" \
  --provider-max-tokens "${MAX_TOKENS}" \
  --provider-repair-attempts 1 \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q repaired notes.txt" \
  --mock-provider-response '{"relative_path":"status.txt","contents":"pass\n","reason_code":"provider_first_attempt"}' >/dev/null
live_status=$?
set -e

save_report "${LIVE_REPORT}" "provider-live-repair-report.json"
if [[ "${live_status}" -ne 0 ]]; then
  echo "provider live verifier repair loop failed; report saved to ${LIVE_REPORT}" >&2
  exit "${live_status}"
fi

python3 - "${LIVE_REPORT}" "${MODEL}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    report = json.load(handle)
model = sys.argv[2]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["passed"] is True, report)
require(report["phase"] == "pr_ready", report["phase"])
require(report["commit_hash"] and len(report["commit_hash"]) == 40, report["commit_hash"])
require("notes.txt" in report["changed_files"], report["changed_files"])
attempts = report["provider_attempts"]
require(len(attempts) == 2, attempts)
require(attempts[0]["mode"] == "mock", attempts[0])
require(attempts[0]["repair_context_included"] is False, attempts[0])
repair = attempts[1]
require(repair["mode"] == "live", repair)
require(repair["dispatched"] is True, repair)
require(repair["parse_status"] == "parsed", repair)
require(repair["repair_context_included"] is True, repair)
require(repair["failed_verifier_count"] >= 1, repair)
require(repair["diff_context_bytes"] and repair["diff_context_bytes"] > 0, repair)
require("notes.txt" in (repair["edit_relative_path"] or ""), repair)
require(repair["response_text_sha256"], repair)
require(repair["provider"] == model.split("/", 1)[0], repair)
require(repair["model"] == model.split("/", 1)[1], repair)
statuses = [item["status"] for item in report["verifier_transcript"]]
require(statuses.count("failed") >= 2, statuses)
require(statuses.count("succeeded") >= 3, statuses)
PY

save_report "${LIVE_REPORT}" "provider-live-repair-report.json"
echo "provider live verifier repair loop passed: ${LIVE_REPORT}"
