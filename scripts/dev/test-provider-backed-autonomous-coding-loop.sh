#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FIXTURE_PATH="${REPO_ROOT}/tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
TASK_ID="repo_spec_to_pr_feature_delivery"

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

load_repo_dotenv_if_present

MODE="${1:-}"
MODEL="${TAU_PROVIDER_PROOF_MODEL:-openai/gpt-4.1-mini}"
AUTH_MODE="${TAU_PROVIDER_PROOF_AUTH_MODE:-api-key}"
MAX_TOKENS="${TAU_PROVIDER_PROOF_MAX_TOKENS:-1024}"
LIVE_ENABLED="${TAU_LIVE_PROVIDER_PROOF:-0}"
REPORT_DIR="${TAU_PROVIDER_PROOF_REPORT_DIR:-}"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local description="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${description}): expected output to contain '${needle}'" >&2
    printf '%s\n' "${haystack}" >&2
    exit 1
  fi
}

save_report() {
  local source_path="$1"
  local name="$2"
  if [[ -n "${REPORT_DIR}" ]]; then
    mkdir -p "${REPORT_DIR}"
    cp "${source_path}" "${REPORT_DIR}/${name}"
  fi
}

run_provider_case() {
  local tmpdir="$1"
  local report_path="${tmpdir}/provider-success-report.json"
  local state_root="${tmpdir}/provider-success-state"
  local repo_root="${tmpdir}/provider-success-repo"

  cargo run -p tau-coding-agent --quiet --bin tau_live_coding_loop_harness -- \
    --fixture "${FIXTURE_PATH}" \
    --task-id "${TASK_ID}" \
    --mode provider-success \
    --state-root "${state_root}" \
    --repo-root "${repo_root}" \
    --output "${report_path}" \
    --run-id "m334-provider-live" \
    --provider-model "${MODEL}" \
    --provider-auth-mode "${AUTH_MODE}" \
    --provider-max-tokens "${MAX_TOKENS}" \
    --started-unix-ms 1800000200000 >/dev/null

  python3 - "${report_path}" "${MODEL}" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
model = sys.argv[2]
report = json.loads(report_path.read_text(encoding="utf-8"))

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == 1, "schema version mismatch")
require(report["mode"] == "provider_success", "mode mismatch")
require(report["passed"] is True, f"provider case did not pass: {report.get('failure_reasons')}")
require(report["phase"] == "pr_ready", f"expected pr_ready, got {report['phase']}")
require(len(report["commit_hash"]) == 40, "commit hash missing")
require("status.txt" in report["changed_files"], "status.txt change missing")
statuses = {item["status"] for item in report["verifier_transcript"]}
require({"failed", "succeeded"} <= statuses, f"expected red+green verifier, got {statuses}")
require(report["pr_ready"]["status"] == "manual_ready", "manual pr-ready state missing")
provider = report["provider"]
require(provider["mode"] == "live", "provider mode should be live")
require(provider["dispatched"] is True, "provider dispatch flag missing")
require(provider["parse_status"] in {"parsed", "extracted_json"}, f"provider parse failed: {provider}")
require(provider["edit_relative_path"] == "status.txt", "provider edit path mismatch")
require(provider["response_text_sha256"], "provider response hash missing")
require(provider["response_text_bytes"] > 0, "provider response byte count missing")
require(provider["provider"] == model.split("/", 1)[0], "provider name mismatch")
require(provider["model"] == model.split("/", 1)[1], "model name mismatch")
print(f"provider live report ok: {report_path}")
PY
  save_report "${report_path}" "provider-live-report.json"
}

run_malformed_case() {
  local tmpdir="$1"
  local report_path="${tmpdir}/provider-malformed-report.json"
  local state_root="${tmpdir}/provider-malformed-state"
  local repo_root="${tmpdir}/provider-malformed-repo"

  set +e
  cargo run -p tau-coding-agent --quiet --bin tau_live_coding_loop_harness -- \
    --fixture "${FIXTURE_PATH}" \
    --task-id "${TASK_ID}" \
    --mode provider-success \
    --state-root "${state_root}" \
    --repo-root "${repo_root}" \
    --output "${report_path}" \
    --run-id "m334-provider-malformed" \
    --provider-model "${MODEL}" \
    --provider-auth-mode "${AUTH_MODE}" \
    --provider-max-tokens "${MAX_TOKENS}" \
    --mock-provider-response "not-json" \
    --started-unix-ms 1800000300000 >/dev/null
  local status=$?
  set -e

  if [[ "${status}" -eq 0 ]]; then
    echo "malformed provider response unexpectedly passed" >&2
    exit 1
  fi

  python3 - "${report_path}" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
report = json.loads(report_path.read_text(encoding="utf-8"))

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["mode"] == "provider_success", "mode mismatch")
require(report["passed"] is False, "malformed case should fail")
require(report["phase"] == "blocked", f"expected blocked, got {report['phase']}")
require(report["commit_hash"] is None, "malformed case should not commit")
require(report["pr_ready"] is None, "malformed case should not prepare PR")
provider = report["provider"]
require(provider["mode"] == "mock", "mock provider mode missing")
require(provider["parse_status"] == "malformed", f"expected malformed parse, got {provider}")
require(provider["reason_code"] == "provider_output_malformed", "reason code mismatch")
require(provider["response_text_sha256"], "response hash missing")
print("provider malformed report ok")
PY
  save_report "${report_path}" "provider-malformed-report.json"
}

if [[ ! -f "${FIXTURE_PATH}" ]]; then
  echo "missing benchmark fixture: ${FIXTURE_PATH}" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

malformed_output="$(run_malformed_case "${tmpdir}")"
assert_contains "${malformed_output}" "provider malformed report ok" "malformed provider case"

if [[ "${MODE}" == "--mock-malformed" ]]; then
  echo "provider-backed autonomous coding loop malformed-output test passed"
  exit 0
fi

if [[ "${LIVE_ENABLED}" != "1" ]]; then
  echo "provider live proof skipped; set TAU_LIVE_PROVIDER_PROOF=1 to run live provider-backed proof"
  exit 0
fi

provider_output="$(run_provider_case "${tmpdir}")"
assert_contains "${provider_output}" "provider live report ok" "provider live case"

echo "provider-backed autonomous coding loop proof passed"
